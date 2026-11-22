//! The app: its state, the keys, and the three columns of the workbench.

use std::path::{Path, PathBuf};

use eframe::egui::{
    self, Align2, Event, Key, Modifiers, Rect, Role, Sense, Theme, Ui, WidgetInfo, pos2, vec2,
};
use mortise_core::{AppSnapshot, ReadError, propose_monogram, read_app};

use crate::chrome::{self, Chrome};
use crate::draw;
use crate::index::{self, IndexState};
use crate::strings;
use crate::theme::{self, Palette, radius, size, ty};
use crate::workspace::{self, Tab};

/// The row of the index that is selected: the whole app or one module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    All,
    Module(String),
}

/// What reading the folder given at start gave.
enum Loaded {
    /// No folder was given.
    Nothing,
    App(Box<AppSnapshot>),
    Failed(ReadError),
}

pub struct MortiseApp {
    chrome: Chrome,
    loaded: Loaded,
    selection: Selection,
    tab: Tab,
    /// True while the keyboard is in use, false once the pointer is. The focus ring
    /// is drawn only while it is true.
    keyboard: bool,
    /// The selection moved by key in this frame, so its row must be scrolled into view.
    reveal: bool,
}

impl MortiseApp {
    /// Sets up fonts and style, and reads `folder` if one is given.
    pub fn new(cc: &eframe::CreationContext<'_>, chrome: Chrome, folder: Option<&Path>) -> Self {
        crate::fonts::install(&cc.egui_ctx);
        style(&cc.egui_ctx);

        let loaded = match folder.map(read_app) {
            None => Loaded::Nothing,
            Some(Ok(app)) => Loaded::App(Box::new(app)),
            Some(Err(error)) => Loaded::Failed(error),
        };
        Self {
            chrome,
            loaded,
            selection: Selection::All,
            tab: Tab::Summary,
            // At start, focus is on the selected row of the index.
            keyboard: true,
            reveal: false,
        }
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }

    pub fn app(&self) -> Option<&AppSnapshot> {
        match &self.loaded {
            Loaded::App(app) => Some(app),
            _ => None,
        }
    }

    fn keys(&mut self, ui: &Ui) {
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

        // Single keys never act while a text field has focus.
        if ui.ctx().text_edit_focused() {
            return;
        }
        let Loaded::App(app) = &self.loaded else {
            return;
        };
        let pressed = |key: Key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));

        // J and K, or Down and Up: the next or previous row. Home and End: the first or last.
        let rows = app.modules.len() + 1;
        let current = match &self.selection {
            Selection::All => 0,
            Selection::Module(name) => app
                .modules
                .iter()
                .position(|module| &module.name == name)
                .map_or(0, |i| i + 1),
        };
        let mut target = current;
        if pressed(Key::J) || pressed(Key::ArrowDown) {
            target = (current + 1).min(rows - 1);
        }
        if pressed(Key::K) || pressed(Key::ArrowUp) {
            target = current.saturating_sub(1);
        }
        if pressed(Key::Home) {
            target = 0;
        }
        if pressed(Key::End) {
            target = rows - 1;
        }
        if target != current {
            self.selection = match target {
                0 => Selection::All,
                row => Selection::Module(app.modules[row - 1].name.clone()),
            };
            self.reveal = true;
        }

        // 1 to 5: the tabs. The choice of tab is kept while moving through modules.
        for (tab, key) in
            Tab::ALL
                .into_iter()
                .zip([Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5])
        {
            if pressed(key) {
                self.tab = tab;
            }
        }
    }
}

impl eframe::App for MortiseApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let pal = theme::palette(ui.ctx().theme());
        self.reveal = false;
        self.keys(ui);

        let window = ui.max_rect();
        ui.painter().rect_filled(window, 0, pal.frame);

        let bar = Rect::from_min_size(window.min, vec2(window.width(), size::TITLEBAR_H));
        let app_name = self.app().map(|app| app.name.clone());
        chrome::title_bar(ui, pal, bar, self.chrome, frame, app_name.as_deref());

        let below = Rect::from_min_max(pos2(window.left(), bar.bottom()), window.max);
        let rail_rect = Rect::from_min_size(below.min, vec2(size::RAIL_W, below.height()));
        rail(ui, pal, rail_rect, app_name.as_deref());

        let rest = Rect::from_min_max(pos2(rail_rect.right(), below.top()), below.max);
        match &self.loaded {
            Loaded::App(app) => {
                let index_rect = Rect::from_min_size(rest.min, vec2(size::INDEX_W, rest.height()));
                let work_rect = Rect::from_min_max(pos2(index_rect.right(), rest.top()), rest.max);
                // The workspace first: the selected row of the index is painted over its edge.
                workspace::workspace(ui, pal, work_rect, app, &self.selection, &mut self.tab);
                let state = IndexState {
                    selection: &mut self.selection,
                    keyboard: self.keyboard,
                    reveal: self.reveal,
                };
                index::index(ui, pal, index_rect, app, state);
            }
            Loaded::Nothing => {
                ui.painter().rect_filled(rest, 0, pal.surface);
                let text = [strings::NO_APPS_BODY, strings::NO_APPS_HOW];
                workspace::empty_state(ui, pal, rest, None, strings::NO_APPS_TITLE, &text);
            }
            Loaded::Failed(error) => {
                ui.painter().rect_filled(rest, 0, pal.surface);
                let text = strings::read_error(error);
                workspace::empty_state(ui, pal, rest, None, strings::CANNOT_READ_TITLE, &[&text]);
            }
        }

        chrome::resize_edges(ui, self.chrome);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        let theme = if visuals.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        };
        egui::Rgba::from(theme::palette(theme).frame).to_array()
    }
}

/// The app rail (S2.1): 52 px wide, one 34 px tile per app. The selected tile is a key block.
fn rail(ui: &Ui, pal: &Palette, rect: Rect, app_name: Option<&str>) {
    ui.painter().rect_filled(rect, 0, pal.frame);
    draw::vline(
        ui,
        rect.right() - 1.0,
        rect.top()..rect.bottom(),
        pal.line_2,
    );

    let Some(app_name) = app_name else {
        return;
    };
    let x = rect.left() + ((rect.width() - 1.0 - size::RAIL_TILE) / 2.0).round();
    let tile = Rect::from_min_size(
        pos2(x, rect.top() + 10.0),
        vec2(size::RAIL_TILE, size::RAIL_TILE),
    );
    let response = ui.interact(
        tile,
        ui.scope_id().with(("rail-tile", app_name)),
        Sense::hover(),
    );
    response.widget_info(|| WidgetInfo::selected(Role::ListBoxOption, true, true, app_name));
    response.on_hover_text(app_name);

    draw::outlined(ui, tile, radius::CONTROL, pal.key, pal.key);
    let monogram = propose_monogram(app_name);
    draw::paint_text(
        ui,
        tile.center(),
        Align2::CENTER_CENTER,
        &monogram,
        ty::MONOGRAM,
        pal.key_fg,
    );
}

/// What egui draws by itself (tooltips, scrollbars) takes the Slate colours too.
fn style(ctx: &egui::Context) {
    // Text size is a setting of Mortise, not a zoom key.
    ctx.options_mut(|options| options.zoom_with_keyboard = false);
    for theme in [Theme::Light, Theme::Dark] {
        let pal = theme::palette(theme);
        ctx.style_mut_of(theme, |style| {
            // Nothing moves unless the user or a command caused it.
            style.animation_time = 0.0;
            style.visuals.panel_fill = pal.surface;
            style.visuals.window_fill = pal.key;
            style.visuals.window_stroke = egui::Stroke::NONE;
            style.visuals.window_corner_radius = radius::CONTROL.into();
            style.visuals.popup_shadow = egui::Shadow::NONE;
            style.visuals.override_text_color = None;
            style.visuals.widgets.noninteractive.fg_stroke.color = pal.key_fg;
            style.visuals.extreme_bg_color = pal.surface;
        });
    }
}

/// Starts the app. `folder` is the app folder to open, if any.
pub fn run(chrome: Chrome, folder: Option<PathBuf>) -> eframe::Result {
    let viewport = chrome.viewport(
        egui::ViewportBuilder::default()
            .with_title(strings::BRAND)
            .with_app_id("mortise")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([size::WINDOW_MIN_W, size::WINDOW_MIN_H]),
    );
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        strings::BRAND,
        options,
        Box::new(move |cc| Ok(Box::new(MortiseApp::new(cc, chrome, folder.as_deref())))),
    )
}
