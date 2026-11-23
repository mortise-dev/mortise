//! Palette: open the command palette, and do what a row of it says.

use super::MortiseApp;
use crate::app::workspace::Tab;
use crate::apps::view::AddPanel;
use crate::index::view::Selection;
use crate::palette::view::{Command, PaletteState};
use crate::settings::view::Section;

impl MortiseApp {
    // --- Palette ---

    pub(super) fn open_palette(&mut self) {
        self.palette = Some(PaletteState::default());
        self.palette_scope = self.current.unwrap_or(0);
    }

    /// Runs a row of the palette, in the app the palette searched.
    pub(super) fn run(&mut self, command: Command) {
        self.palette = None;
        // A command that goes somewhere leaves Settings.
        if !matches!(
            command,
            Command::Settings | Command::PhpSettings | Command::Theme(_) | Command::Runner
        ) {
            self.settings_open = None;
        }
        let scope = self.palette_scope;
        // Commands about modules and findings act in the app the palette searched.
        if matches!(
            command,
            Command::Module(_)
                | Command::Surface(_)
                | Command::AllModules
                | Command::Findings
                | Command::Finding { .. }
                | Command::Action { .. }
                | Command::Generate { .. }
        ) {
            if self.current != Some(scope) {
                self.select_app(scope);
            }
            self.adding = None;
        }
        match command {
            Command::Module(name) => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::Module(name);
                    self.reveal = true;
                }
            }
            Command::Surface(name) => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::Module(name);
                    open.tab = Tab::Boundary;
                    open.boundary.surface_view = true;
                    self.reveal = true;
                }
            }
            Command::AllModules => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::All;
                    open.tab = Tab::Summary;
                    open.findings_open = false;
                }
            }
            Command::Findings => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::All;
                    open.tab = Tab::Summary;
                    open.findings_open = true;
                    open.finding_row = 0;
                }
            }
            Command::Finding { module, other } => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::Module(module);
                    match other {
                        Some(other) => {
                            open.tab = Tab::Boundary;
                            open.boundary.surface_view = false;
                            open.boundary.other = Some(other);
                            open.boundary.file = 0;
                        }
                        None => open.tab = Tab::Summary,
                    }
                    self.reveal = true;
                }
            }
            Command::Action { module, action } => {
                if self.current != Some(scope) {
                    self.select_app(scope);
                }
                self.open_action(module, action);
            }
            Command::Generate { module, generator } => self.open_generate(module, &generator),
            Command::App(index) => self.select_app(index),
            Command::AddApp => self.adding = Some(AddPanel::empty()),
            Command::Settings => self.open_settings(Section::Apps),
            Command::PhpSettings => self.open_settings(Section::Php),
            Command::Runner => self.runner.toggle(),
            Command::Theme(theme) => {
                self.settings.general.theme = theme;
                self.save();
            }
        }
    }
}
