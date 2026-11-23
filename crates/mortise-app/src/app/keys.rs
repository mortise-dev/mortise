//! The keys of the window: shortcuts, focus between the regions, and moves in the index.

use eframe::egui::{self, Event, Key, KeyboardShortcut, Modifiers, Ui};
use mortise_core::MODULE_MAKE;

use super::{MortiseApp, index_width};
use crate::app::rail;
use crate::app::workspace::{self, Tab};
use crate::index::view::{self as index, Selection};
use crate::runner::view::{self as runner_view, DrawerAction, Shown};
use crate::settings::view::Section;
use crate::ui::draw;

impl MortiseApp {
    // --- Keys ---

    pub(super) fn keys(&mut self, ui: &Ui) {
        // The focus ring shows while the keyboard is in use and goes when the pointer is.
        let (key_pressed, pointer_pressed) = ui.input(|input| {
            let key = input
                .events
                .iter()
                .any(|event| matches!(event, Event::Key { pressed: true, .. }));
            (key, input.pointer.any_pressed())
        });
        if key_pressed {
            self.keyboard = true;
        }
        if pointer_pressed {
            self.keyboard = false;
        }

        let chord = |modifiers: Modifiers, key: Key| {
            ui.input_mut(|input| input.consume_shortcut(&KeyboardShortcut::new(modifiers, key)))
        };
        // Which region holds focus (K1). Focus is on the index when its selected row
        // holds it, and also when nothing holds it.
        let in_list = draw::take_list_focus(ui.ctx());
        let on_row = index::take_row_focus(ui.ctx());
        let focused = ui.memory(|memory| memory.focused());
        let on_index = focused.is_none() || on_row;
        let focused_at = focused
            .filter(|_| !on_row)
            .and_then(|id| ui.ctx().read_response(id))
            .map(|response| response.rect.center());
        let in_workspace = focused_at.is_some_and(|at| self.work_rect.contains(at));

        // Focus on its way into the workspace: from the tab row to the first control
        // of the tab. A tab without a control gives focus back to the index.
        match self.entering_workspace {
            2 => {
                ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::Next));
                self.entering_workspace = 1;
            }
            1 => {
                if let Some(id) = focused.filter(|_| !in_workspace) {
                    ui.memory_mut(|memory| memory.surrender_focus(id));
                }
                self.entering_workspace = 0;
            }
            _ => {}
        }

        // The command palette, from anywhere. The same key closes it.
        if chord(Modifiers::COMMAND, Key::K) {
            if self.palette.is_some() {
                self.palette = None;
            } else if self.dialog.is_none() {
                self.open_palette();
            }
        }
        // Copy the output of the drawer: before Add app, which would take Cmd Shift O too.
        let free = self.palette.is_none() && self.dialog.is_none() && self.adding.is_none();
        let copy_output = free && chord(Modifiers::COMMAND | Modifiers::SHIFT, Key::O);
        // Add app: the folder dialog opens at once. A dialog holds the keys.
        if self.palette.is_none() && self.dialog.is_none() && chord(Modifiers::COMMAND, Key::O) {
            self.pick(ui.ctx());
        }
        // Esc closes the topmost thing: the add panel, then the toast, then Settings,
        // then the text of the filter, then the filter field itself.
        let filter_id = index::filter_id(ui);
        let filter_focused = ui.memory(|memory| memory.has_focus(filter_id));
        if chord(Modifiers::NONE, Key::Escape) {
            let open = self.current.map(|current| &mut self.apps[current]);
            if self.dialog.is_some() {
                self.dialog = None;
            } else if self.palette.is_some() {
                self.palette = None;
            } else if self.adding.is_some() {
                self.adding = None;
            } else if self.toast.is_some() {
                self.toast = None;
            } else if self.settings_open.is_some() {
                // Settings take the place of the workbench, so they are above it.
                // Esc leaves a field first, and egui has already taken its focus.
                // The next Esc closes Settings.
                if !self.settings_field_was_focused {
                    self.close_settings = true;
                }
            } else if let Some(open) = open {
                // A scan of the public surface on screen: Esc stops it.
                let scanning = match &open.selection {
                    Selection::Module(name) if open.tab == Tab::Boundary => open
                        .boundary
                        .surfaces
                        .get_mut(name)
                        .filter(|surface| open.boundary.surface_view && surface.is_scanning()),
                    _ => None,
                };
                let picker = open.tab == Tab::Generate;
                if let Some(surface) = scanning {
                    surface.stop();
                } else if picker
                    && (open.generate.filter_was_focused() || open.filter.is_empty())
                    && open.generate.escape()
                {
                    // Esc clears the filter of the picker (S9.1).
                } else if !open.filter.is_empty() {
                    // Esc clears the text first and leaves the field second. egui has
                    // already taken focus from the field for this Esc: give it back.
                    open.filter.clear();
                    self.focus_filter = self.filter_was_focused;
                } else if !self.filter_was_focused && open.findings_open {
                    // Back from a second view.
                    open.findings_open = false;
                }
            }
        }
        // A dialog, the palette and the add panel hold the keys until they close.
        if self.adding.is_some() || self.dialog.is_some() || self.palette.is_some() {
            return;
        }

        // Settings, and the shortcuts, which are a section of them.
        let typing = ui.ctx().text_edit_focused();
        if chord(Modifiers::COMMAND, Key::Comma) {
            self.open_settings(Section::Apps);
        }
        if !typing && chord(Modifiers::NONE, Key::Questionmark) {
            self.open_settings(Section::Keyboard);
        }
        // Settings take the place of the workbench: its keys rest until Esc.
        if self.settings_open.is_some() {
            return;
        }

        // F6 and Shift F6: to the next or the previous region. Rail, index, workspace,
        // and the runner drawer while it is open.
        let backwards = chord(Modifiers::SHIFT, Key::F6);
        if (backwards || chord(Modifiers::NONE, Key::F6))
            && let Some(current) = self.current
        {
            let regions = if self.runner.shown == Shown::Open {
                4
            } else {
                3
            };
            let in_drawer = focused_at.is_some_and(|at| self.drawer_rect.contains(at));
            let region = if on_index {
                1
            } else if in_drawer {
                3
            } else if in_workspace {
                2
            } else {
                0
            };
            let target = if backwards {
                (region + regions - 1) % regions
            } else {
                (region + 1) % regions
            };
            let id = match target {
                0 => rail::current_tile(ui.ctx()),
                2 => Some(workspace::tab_id(ui, self.apps[current].tab)),
                3 => Some(runner_view::handle_id()),
                _ => None,
            };
            ui.memory_mut(|memory| match (id, focused) {
                (Some(id), _) => memory.request_focus(id),
                // The index: its selected row has the keys when nothing else holds focus.
                (None, Some(focused)) => memory.surrender_focus(focused),
                (None, None) => {}
            });
            self.keyboard = true;
        }

        // Collapse or open the module index. Mortise then stops deciding its width
        // until the window is resized.
        if chord(Modifiers::COMMAND, Key::B) {
            let set = f32::from(self.settings.general.index_width);
            let (_, collapsed) = index_width(self.window_w, self.index_choice, set);
            self.index_choice = Some(!collapsed);
        }
        // The new module form, anywhere an app is open (design D7). Nothing runs.
        if chord(Modifiers::COMMAND, Key::N) && self.app().is_some() {
            self.open_generate(None, MODULE_MAKE);
        }
        // Filter modules: / when not typing, or the chord for keyboards where / needs Shift.
        if chord(Modifiers::COMMAND, Key::F) || (!typing && chord(Modifiers::NONE, Key::Slash)) {
            self.index_choice = Some(false);
            self.focus_filter = true;
        }
        // Enter in the filter keeps the first match and gives the keys back to the list.
        if filter_focused && chord(Modifiers::NONE, Key::Enter) {
            ui.memory_mut(|memory| memory.surrender_focus(filter_id));
        }

        // The runner drawer, where not typing. Cmd Shift R comes before Cmd R.
        if !typing && let Some(current) = self.current {
            if chord(Modifiers::CTRL, Key::Backtick) || chord(Modifiers::COMMAND, Key::J) {
                self.runner.toggle();
            }
            if chord(Modifiers::COMMAND | Modifiers::SHIFT, Key::R) {
                self.run_again(current);
            }
            if chord(Modifiers::COMMAND | Modifiers::SHIFT, Key::H) {
                self.runner.toggle_history();
            }
            if copy_output {
                self.drawer_action(ui.ctx(), current, DrawerAction::CopyOutput);
            }
            if chord(Modifiers::COMMAND, Key::Period) {
                self.ask_stop(current);
            }
        }

        // Read the app again.
        if chord(Modifiers::COMMAND, Key::R)
            && let Some(current) = self.current
        {
            // From disk and from the actuator.
            self.apps[current].live.refresh();
            if let Some(connected) = &self.apps[current].runtime {
                connected.live.refresh();
            }
        }
        // Select app 1 to 9 on the rail.
        let digits = [
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Num5,
            Key::Num6,
            Key::Num7,
            Key::Num8,
            Key::Num9,
        ];
        for (index, key) in digits.into_iter().enumerate() {
            if chord(Modifiers::COMMAND | Modifiers::ALT, key) {
                self.select_app(index);
            }
        }

        // Single keys never act while a text field has focus.
        if ui.ctx().text_edit_focused() {
            return;
        }
        let Some(app) = self.app() else {
            return;
        };
        let Some(current_app) = self.current else {
            return;
        };
        let pressed = |key: Key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));

        // S: change the order of the index. The choice is kept per app.
        if pressed(Key::S) {
            self.toggle_order();
        }
        let problems_first = self.settings.apps[current_app].problems_first;
        let open = &mut self.apps[current_app];
        // E: the selected module in the editor.
        if pressed(Key::E)
            && let Selection::Module(name) = &open.selection
            && let Some(module) = app.module(name)
        {
            let folder = module.dir.clone();
            self.open_in_editor(&folder, 1);
            return;
        }
        let open = &mut self.apps[current_app];
        // G: the Generate tab with the filter of the picker focused (S9.1).
        if pressed(Key::G) {
            open.tab = Tab::Generate;
            open.findings_open = false;
            open.generate.open_picker(true);
        }
        // F: the findings list of the app.
        if pressed(Key::F) {
            open.selection = Selection::All;
            open.tab = Tab::Summary;
            open.findings_open = true;
            open.finding_row = 0;
        }

        // Enter or Right: from the index into the workspace. Esc brings focus back.
        if on_index && (pressed(Key::Enter) || pressed(Key::ArrowRight)) {
            ui.memory_mut(|memory| {
                // The arrow is not also a move of focus to the right.
                memory.move_focus(egui::FocusDirection::None);
                memory.request_focus(workspace::tab_id(ui, open.tab));
            });
            self.entering_workspace = 2;
        }

        // J and K, or Down and Up: the next or previous row. Home and End: the first or last.
        // Only the rows the filter leaves. A list or table that has focus keeps these
        // keys for itself, and the arrows move focus when another control has it.
        let modules = index::ordered(&app, &open.filter, problems_first);
        let rows = modules.len() + 1;
        let current = match &open.selection {
            Selection::All => 0,
            Selection::Module(name) => modules
                .iter()
                .position(|module| &module.name == name)
                .map_or(0, |i| i + 1),
        };
        let mut target = current;
        if !in_list && (pressed(Key::J) || (on_index && pressed(Key::ArrowDown))) {
            target = (current + 1).min(rows - 1);
        }
        if !in_list && (pressed(Key::K) || (on_index && pressed(Key::ArrowUp))) {
            target = current.saturating_sub(1);
        }
        if on_index && pressed(Key::Home) {
            target = 0;
        }
        if on_index && pressed(Key::End) {
            target = rows - 1;
        }
        if target != current {
            open.selection = match target {
                0 => Selection::All,
                row => Selection::Module(modules[row - 1].name.clone()),
            };
            self.reveal = true;
        }

        // 1 to 5: the tabs. The choice of tab is kept while moving through modules.
        for (tab, key) in Tab::ALL.into_iter().zip(digits) {
            if pressed(key) {
                open.tab = tab;
            }
        }
    }
}
