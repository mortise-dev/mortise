//! Updates: when the window asks for a newer Mortise, and what it does with the answer.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use mortise_core::UpdateCheck;

use super::MortiseApp;
use super::strings as app_strings;
use crate::ui::toast::{Toast, ToastAction};
use crate::update::{CHECK_EVERY, DOWNLOAD_PAGE, Outcome, Phase, Updates, strings};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

impl MortiseApp {
    /// At start: asks for a newer version if the setting is on and the last answer is
    /// a day old. A newer version found within that day is shown again.
    pub(super) fn start_updates(&mut self, ctx: &egui::Context) {
        let Some(updates) = &mut self.updates else {
            return;
        };
        if !self.settings.general.check_for_updates {
            return;
        }
        // Claimed in the file, so two windows that start together ask once.
        let claimed = match &self.settings_path {
            Some(settings) => UpdateCheck::claim(&UpdateCheck::path(settings), now(), CHECK_EVERY),
            None => Some(UpdateCheck::default()),
        };
        if let Some(check) = claimed {
            self.update_check = check;
            updates.check(ctx, false);
        } else if let Phase::Available(version) = updates.phase().clone() {
            self.update_notice(&version);
        }
    }

    /// Keeps that the window asks now, before the answer: a check that fails, or a
    /// window that closes first, still counts for the day.
    fn asked_for_updates(&mut self) {
        self.update_check.at = now();
        self.save_update_check();
    }

    fn save_update_check(&mut self) {
        let Some(settings) = &self.settings_path else {
            return;
        };
        // Not being able to keep it costs one more request at the next start.
        let _ = self.update_check.save(&UpdateCheck::path(settings));
    }

    /// True while Mortise replaces itself, with a toast that says so: no command starts then.
    pub(super) fn installing_update(&mut self) -> bool {
        let installing = self.updates.as_ref().is_some_and(Updates::is_installing);
        if installing {
            self.toast = Some(Toast::failed(app_strings::INSTALLING_NO_COMMAND));
        }
        installing
    }

    /// On the way out: an install that runs ends first.
    pub(super) fn finish_update(&mut self) {
        if let Some(updates) = &mut self.updates {
            updates.finish();
        }
    }

    /// The toast that says a newer version exists. It stays until it is closed.
    fn update_notice(&mut self, version: &str) {
        let can_install = self.updates.as_ref().is_some_and(Updates::can_install);
        self.toast = Some(if can_install {
            Toast::done(strings::available(version))
                .staying()
                .with_action(strings::INSTALL_AND_RESTART, ToastAction::InstallUpdate)
        } else {
            Toast::done(strings::available_by_hand(version))
                .staying()
                .with_action(strings::OPEN_DOWNLOAD_PAGE, ToastAction::OpenDownloadPage)
        });
    }

    /// Every frame: what a check, a download or an install ended with.
    pub(super) fn poll_updates(&mut self, ctx: &egui::Context) {
        let Some(updates) = &mut self.updates else {
            return;
        };
        // A test of PHP in Settings or in the add panel is a command too. Its end is
        // taken in here, because Settings does that only while it is drawn. None
        // starts while Mortise replaces itself.
        let installing = updates.is_installing();
        let tests = self
            .apps
            .iter_mut()
            .map(|open| &mut open.php_test)
            .chain(self.adding.as_mut().map(|panel| &mut panel.php_test));
        let mut testing = false;
        for test in tests {
            test.poll();
            test.hold(installing);
            testing |= test.running();
        }
        let running = testing || self.apps.iter().any(|open| open.busy().is_some());
        let before = updates.phase().clone();
        let instance = &mut self.instance;
        for outcome in updates.poll(ctx, running, &mut || instance.alone()) {
            match outcome {
                Outcome::Checked { newer, asked } => {
                    // Settings shows the answer to a check the user asked for.
                    if let (Some(version), false) = (&newer, asked) {
                        self.update_notice(version);
                    }
                    self.update_check = UpdateCheck { at: now(), newer };
                    self.save_update_check();
                }
                Outcome::Failed(error) => self.toast = Some(Toast::failed(error)),
                Outcome::Restart => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        }
        // The toast follows the download: it must not say "downloading" once that ended.
        let Some(updates) = &self.updates else {
            return;
        };
        match updates.phase() {
            now if *now == before => {}
            Phase::Waiting(_) => self.toast = Some(Toast::done(strings::WAITING).staying()),
            Phase::Installing(_) => {
                self.toast = Some(Toast::done(strings::INSTALLING).staying());
            }
            _ => {}
        }
    }

    pub(super) fn check_for_updates(&mut self, ctx: &egui::Context) {
        if let Some(updates) = &mut self.updates
            && updates.can_check()
        {
            updates.check(ctx, true);
            self.asked_for_updates();
        }
    }

    pub(super) fn install_update(&mut self, ctx: &egui::Context) {
        let Some(updates) = &mut self.updates else {
            return;
        };
        updates.install(ctx);
        if let Phase::Downloading(version) = updates.phase() {
            self.toast = Some(Toast::done(strings::downloading_notice(version)).staying());
        }
    }

    pub(super) fn open_download_page(&mut self, ctx: &egui::Context) {
        self.toast = None;
        ctx.open_url(egui::OpenUrl::new_tab(DOWNLOAD_PAGE));
    }
}

/// The mark of an open window of Mortise: a lock on `instance.lock` beside the settings.
///
/// Every window holds it shared. A window that is about to replace the app takes it
/// alone, which works only when no other window is open. So an install never runs
/// under another window: its commands are not cut off, and two windows do not replace
/// the app at once.
pub(super) struct Instance {
    file: Option<std::fs::File>,
    alone: bool,
}

impl Instance {
    /// `settings` is where the settings are kept. Without it there is no other window to know of.
    pub(super) fn open(settings: Option<&std::path::Path>) -> Self {
        let file = settings.and_then(|settings| {
            std::fs::create_dir_all(settings.parent()?).ok()?;
            std::fs::File::create(settings.with_file_name("instance.lock")).ok()
        });
        if let Some(file) = &file {
            // Refused while another window installs. This window then goes unmarked.
            let _ = file.try_lock_shared();
        }
        Self { file, alone: false }
    }

    /// True when no other window of Mortise is open. From then on this window holds
    /// the lock alone.
    fn alone(&mut self) -> bool {
        let Some(file) = &self.file else {
            return true;
        };
        if !self.alone {
            let _ = file.unlock();
            self.alone = file.try_lock().is_ok();
            if !self.alone {
                let _ = file.try_lock_shared();
            }
        }
        self.alone
    }
}
