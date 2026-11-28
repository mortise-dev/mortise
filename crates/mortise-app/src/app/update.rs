//! Updates: when the window asks for a newer Mortise, and what it does with the answer.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use mortise_core::UpdateCheck;

use super::MortiseApp;
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
        let last = self.settings.update.as_ref().map_or(0, |check| check.at);
        if now().saturating_sub(last) >= CHECK_EVERY {
            updates.check(ctx, false);
        } else if let Phase::Available(version) = updates.phase().clone() {
            self.update_notice(&version);
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
        let busy = self.apps.iter().any(|open| open.busy().is_some());
        let Some(updates) = &mut self.updates else {
            return;
        };
        for outcome in updates.poll(ctx, busy) {
            match outcome {
                Outcome::Checked { newer, asked } => {
                    // Settings shows the answer to a check the user asked for.
                    if let (Some(version), false) = (&newer, asked) {
                        self.update_notice(version);
                    }
                    self.settings.update = Some(UpdateCheck { at: now(), newer });
                    self.save();
                }
                Outcome::Failed(error) => self.toast = Some(Toast::failed(error)),
                Outcome::Restart => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        }
    }

    pub(super) fn check_for_updates(&mut self, ctx: &egui::Context) {
        if let Some(updates) = &mut self.updates {
            updates.check(ctx, true);
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
