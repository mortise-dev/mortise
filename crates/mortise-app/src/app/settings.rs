//! Settings: what the general settings change in the window, the editor, and the menu bar of macOS.

use std::path::Path;

use eframe::egui;
use mortise_core::ThemeChoice;

use super::MortiseApp;
#[cfg(target_os = "macos")]
use super::index_width;
use crate::app::chrome::Chrome;
#[cfg(target_os = "macos")]
use crate::palette::view::Command;
use crate::settings::view::{Section, SettingsState};
use crate::ui::toast::Toast;

impl MortiseApp {
    /// Gives the app the menu bar of macOS. Only the real window has one.
    #[cfg(target_os = "macos")]
    pub fn with_menu_bar(mut self, ctx: &egui::Context) -> Self {
        self.menu = crate::app::menu::MenuBar::install(ctx);
        self
    }

    /// What was chosen in the menu bar. A Settings field in use is left first, so
    /// that its edit applies in this frame, before the choice is done at its end.
    #[cfg(target_os = "macos")]
    pub(super) fn menu_chosen(&self, ctx: &egui::Context) -> Vec<crate::app::menu::MenuAction> {
        let chosen = self
            .menu
            .as_ref()
            .map(|menu| menu.chosen())
            .unwrap_or_default();
        if !chosen.is_empty() && self.settings_open.is_some() {
            ctx.memory_mut(egui::Memory::stop_text_input);
        }
        chosen
    }

    /// Does what was chosen in the menu bar: the same commands as the palette.
    #[cfg(target_os = "macos")]
    pub(super) fn menu_actions(
        &mut self,
        ctx: &egui::Context,
        chosen: Vec<crate::app::menu::MenuAction>,
    ) {
        use crate::app::menu::MenuAction;
        // The choices run after this frame was painted: the next frame shows them.
        if !chosen.is_empty() {
            ctx.request_repaint();
        }
        for action in chosen {
            self.palette_scope = self.current.unwrap_or(0);
            match action {
                MenuAction::Settings => self.open_settings(Section::Apps),
                MenuAction::Shortcuts => self.open_settings(Section::Keyboard),
                // A dialog holds the app until it closes, as for the key.
                MenuAction::AddApp if self.dialog.is_none() => self.pick(ctx),
                MenuAction::AddApp => {}
                MenuAction::Palette => self.open_palette(),
                MenuAction::AllModules => self.run(Command::AllModules),
                MenuAction::Findings => self.run(Command::Findings),
                MenuAction::Theme(theme) => self.run(Command::Theme(theme)),
                MenuAction::ToggleIndex => {
                    let set = f32::from(self.settings.general.index_width);
                    let (_, collapsed) = index_width(self.window_w, self.index_choice, set);
                    self.index_choice = Some(!collapsed);
                }
                MenuAction::ReadAgain => {
                    if let Some(current) = self.current {
                        self.apps[current].live.refresh();
                        if let Some(connected) = &self.apps[current].runtime {
                            connected.live.refresh();
                        }
                    }
                }
            }
        }
    }

    /// What the operating system says about motion, asked at most once a second.
    pub(super) fn system_reduces_motion(&mut self) -> bool {
        let fresh = |(_, at): &(bool, std::time::Instant)| at.elapsed().as_secs() < 1;
        match self.system_motion.filter(fresh) {
            Some((reduced, _)) => reduced,
            None => {
                let reduced = system_reduces_motion();
                self.system_motion = Some((reduced, std::time::Instant::now()));
                reduced
            }
        }
    }

    // --- Settings ---

    pub(super) fn open_settings(&mut self, section: Section) {
        self.adding = None;
        self.palette = None;
        self.settings_open = Some(SettingsState::open(section, self.current.unwrap_or(0)));
    }

    /// Applies what Settings chose for the whole of Mortise, and gives the frame
    /// of the window that follows from it.
    pub(super) fn apply_general(&mut self, ctx: &egui::Context) -> Chrome {
        let general = &self.settings.general;
        let theme = match general.theme {
            ThemeChoice::System => egui::ThemePreference::System,
            ThemeChoice::Light => egui::ThemePreference::Light,
            ThemeChoice::Dark => egui::ThemePreference::Dark,
        };
        if ctx.options(|options| options.theme_preference) != theme {
            ctx.set_theme(theme);
        }
        // Text size scales every size in the interface.
        let zoom = f32::from(general.text_size) / 100.0;
        if (ctx.zoom_factor() - zoom).abs() > 0.001 {
            ctx.set_zoom_factor(zoom);
        }

        // Where Mortise draws the frame, the user can move the controls or take
        // the platform's own title bar instead.
        let chrome = match self.chrome {
            Chrome::Custom { .. } if general.system_title_bar => Chrome::System,
            Chrome::Custom { .. } => Chrome::Custom {
                controls_left: general.window_controls_left,
            },
            other => other,
        };
        let decorated = chrome == Chrome::System;
        if matches!(self.chrome, Chrome::Custom { .. }) && decorated != self.decorated {
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(decorated));
            self.decorated = decorated;
        }
        chrome
    }

    /// Opens a file or a folder in the editor the user chose.
    pub(super) fn open_in_editor(&mut self, path: &Path, line: u32) {
        let command = self.settings.general.editor_command().to_owned();
        if let Err(error) = mortise_core::open_in_editor(&command, path, line) {
            self.toast = Some(Toast::failed(format!(
                "Could not open the editor: \"{error}\". Choose the editor command in Settings."
            )));
        }
    }
}

/// True when the operating system is set to reduce motion.
///
/// macOS is asked. On Windows and Linux neither egui nor winit gives the setting,
/// and reading it needs code this crate does not allow: there "System" moves as "Full".
#[cfg(target_os = "macos")]
fn system_reduces_motion() -> bool {
    objc2_app_kit::NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
}

#[cfg(not(target_os = "macos"))]
fn system_reduces_motion() -> bool {
    false
}
