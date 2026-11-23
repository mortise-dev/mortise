//! The menu bar of macOS: Mortise, File, View, Window and Help, with the same
//! commands as the palette. Windows and Linux have no menu bar: there the palette
//! is the menu.
//!
//! The items of Mortise carry no key in the menu. Their keys are handled by the
//! window, and a key that the menu handled too would act twice.

use std::sync::mpsc::{Receiver, channel};

use eframe::egui;
use mortise_core::ThemeChoice;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

use crate::strings;

/// What an item of the menu bar asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Settings,
    AddApp,
    Palette,
    AllModules,
    Findings,
    ToggleIndex,
    ReadAgain,
    Theme(ThemeChoice),
    Shortcuts,
}

/// The menu bar. It stays in place while this is alive.
pub struct MenuBar {
    _menu: Menu,
    actions: Vec<(muda::MenuId, MenuAction)>,
    chosen: Receiver<muda::MenuId>,
}

impl MenuBar {
    /// Builds the menu bar and gives it to the application. Call on the main thread,
    /// once the application exists. `ctx` is woken when an item is chosen.
    pub fn install(ctx: &egui::Context) -> Option<Self> {
        let mut actions = Vec::new();
        let mut item = |text: &str, action: MenuAction| {
            let item = MenuItem::new(text, true, None);
            actions.push((item.id().clone(), action));
            item
        };
        let separator = PredefinedMenuItem::separator;

        let settings = item("Settings…", MenuAction::Settings);
        let app = Submenu::with_items(
            strings::BRAND,
            true,
            &[
                &PredefinedMenuItem::about(None, None),
                &separator(),
                &settings,
                &separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &separator(),
                &PredefinedMenuItem::quit(None),
            ],
        )
        .ok()?;

        let add = item("Add App…", MenuAction::AddApp);
        let file = Submenu::with_items("File", true, &[&add]).ok()?;

        let palette = item("Command Palette", MenuAction::Palette);
        let all = item(strings::ALL_MODULES, MenuAction::AllModules);
        let findings = item(strings::FINDINGS, MenuAction::Findings);
        let index = item("Collapse or Open the Module Index", MenuAction::ToggleIndex);
        let read = item("Read the App Again", MenuAction::ReadAgain);
        let system = item("Theme: System", MenuAction::Theme(ThemeChoice::System));
        let light = item("Theme: Light", MenuAction::Theme(ThemeChoice::Light));
        let dark = item("Theme: Dark", MenuAction::Theme(ThemeChoice::Dark));
        let view = Submenu::with_items(
            "View",
            true,
            &[
                &palette,
                &separator(),
                &all,
                &findings,
                &separator(),
                &index,
                &read,
                &separator(),
                &system,
                &light,
                &dark,
            ],
        )
        .ok()?;

        let window = Submenu::with_items(
            "Window",
            true,
            &[
                &PredefinedMenuItem::minimize(None),
                &PredefinedMenuItem::maximize(None),
                &separator(),
                &PredefinedMenuItem::bring_all_to_front(None),
            ],
        )
        .ok()?;

        let shortcuts = item("Keyboard Shortcuts", MenuAction::Shortcuts);
        let help = Submenu::with_items("Help", true, &[&shortcuts]).ok()?;

        let menu = Menu::with_items(&[&app, &file, &view, &window, &help]).ok()?;
        menu.init_for_nsapp();
        window.set_as_windows_menu_for_nsapp();
        help.set_as_help_menu_for_nsapp();

        // An item is chosen outside a frame: wake the window so that it acts.
        let (sender, chosen) = channel();
        let ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let _ = sender.send(event.id);
            ctx.request_repaint();
        }));
        Some(Self {
            _menu: menu,
            actions,
            chosen,
        })
    }

    /// What was chosen in the menu bar since the last frame.
    pub fn chosen(&self) -> Vec<MenuAction> {
        self.chosen
            .try_iter()
            .filter_map(|id| {
                self.actions
                    .iter()
                    .find(|(known, _)| *known == id)
                    .map(|(_, action)| *action)
            })
            .collect()
    }
}
