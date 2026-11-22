//! The window frame: the title bar of section 8 and, where Mortise draws the
//! whole frame, the window controls and the edges that resize.

use eframe::egui::viewport::ResizeDirection;
use eframe::egui::{
    Align2, CursorIcon, Rect, Role, Sense, Ui, ViewportBuilder, ViewportCommand, WidgetInfo, pos2,
    vec2,
};

use crate::draw::{self, Icon};
use crate::strings;
use crate::theme::{Palette, radius, size, ty};

/// Who draws the frame of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chrome {
    /// T1, macOS: the system draws its three buttons at the left, over Mortise's bar.
    Mac,
    /// T2 and T3, Windows and Linux: no system decorations, Mortise draws the controls.
    Custom { controls_left: bool },
    /// T4, the fallback: the platform's own title bar, with Mortise's bar under it.
    System,
}

impl Chrome {
    /// The frame for the platform Mortise was built for.
    pub fn native() -> Self {
        if cfg!(target_os = "macos") {
            Chrome::Mac
        } else {
            Chrome::Custom {
                controls_left: false,
            }
        }
    }

    /// Applies the frame to the window before it is created.
    pub fn viewport(self, viewport: ViewportBuilder) -> ViewportBuilder {
        match self {
            Chrome::Mac => viewport
                .with_fullsize_content_view(true)
                .with_titlebar_shown(false)
                .with_title_shown(false)
                .with_titlebar_buttons_shown(true),
            Chrome::Custom { .. } => viewport.with_decorations(false),
            Chrome::System => viewport,
        }
    }
}

/// Paints the title bar into `bar` and handles dragging, double click and the window controls.
pub fn title_bar(
    ui: &Ui,
    pal: &Palette,
    bar: Rect,
    chrome: Chrome,
    frame: &eframe::Frame,
    app_name: Option<&str>,
) -> bool {
    ui.painter().rect_filled(bar, 0, pal.frame);
    draw::hline(ui, bar.left()..bar.right(), bar.bottom() - 1.0, pal.line_2);

    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));

    // Any empty part of the bar drags the window; a double click zooms or maximises.
    // Registered first, so the controls painted after it are on top of it.
    if chrome != Chrome::System {
        let drag = ui.interact(
            bar,
            ui.scope_id().with("title-bar"),
            Sense::click_and_drag(),
        );
        if drag.double_clicked() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!maximized));
        } else if drag.drag_started() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
    }

    let controls_w = 3.0 * size::WINDOW_CONTROL_W;
    let mut left = bar.left() + 12.0;
    let mut right = bar.right() - 12.0;
    match chrome {
        Chrome::Mac => {
            left = bar.left() + size::MAC_BUTTONS_W;
            // Centre the system's buttons in the bar, which is higher than the system's own.
            frame.set_traffic_lights_position(ui.ctx(), bar.y_range(), 13.0);
        }
        Chrome::Custom {
            controls_left: true,
        } => {
            let controls = Rect::from_min_size(bar.min, vec2(controls_w, bar.height() - 1.0));
            window_controls(
                ui,
                pal,
                controls,
                &[Icon::Close, Icon::Minimise, Icon::Maximise],
                maximized,
            );
            left = controls.right() + 12.0;
        }
        Chrome::Custom {
            controls_left: false,
        } => {
            let min = pos2(bar.right() - controls_w, bar.top());
            let controls = Rect::from_min_size(min, vec2(controls_w, bar.height() - 1.0));
            window_controls(
                ui,
                pal,
                controls,
                &[Icon::Minimise, Icon::Maximise, Icon::Close],
                maximized,
            );
            right = controls.left() - 12.0;
        }
        Chrome::System => {}
    }

    // Left: the mark and the name.
    let mid = bar.center().y;
    let mark = Rect::from_min_size(pos2(left, mid - 8.0), vec2(16.0, 16.0));
    draw::brand_mark(ui, pal, mark);
    draw::label(
        ui,
        pos2(mark.right() + 8.0, mid),
        Align2::LEFT_CENTER,
        strings::BRAND,
        ty::ROW_NAME,
        pal.ink,
    );

    // Middle: the entry to the command palette. It narrows under 1100 px.
    let narrow = bar.width() < 1100.0;
    let find_w = if narrow { 200.0 } else { 320.0 };
    let find = Rect::from_center_size(pos2(bar.center().x.round(), mid), vec2(find_w, 22.0));
    // On frame, an outline is ink-2: line-2 is under 3 to 1 there in the light theme.
    // The entry opens the command palette.
    let entry = ui.interact(find, ui.scope_id().with("palette-entry"), Sense::click());
    entry.widget_info(|| WidgetInfo::labeled(Role::Button, true, strings::SEARCH));
    let fill = if entry.hovered() { pal.well } else { pal.panel };
    draw::outlined(ui, find, radius::CONTROL, fill, pal.ink_2);
    let glass = Rect::from_min_size(pos2(find.left() + 8.0, mid - 6.5), vec2(13.0, 13.0));
    draw::icon(ui, glass, Icon::Search, pal.ink_2);
    draw::label(
        ui,
        pos2(glass.right() + 8.0, mid),
        Align2::LEFT_CENTER,
        if narrow {
            strings::SEARCH_SHORT
        } else {
            strings::SEARCH
        },
        ty::SECONDARY,
        pal.ink_2,
    );
    let mac = ui.ctx().os() == eframe::egui::os::OperatingSystem::Mac;
    let keys = format!("{} K", strings::command_key(mac));
    draw::keycap(ui, pal, pos2(find.right() - 8.0, mid), &keys);

    // Right: the current app.
    if let Some(app_name) = app_name {
        draw::label(
            ui,
            pos2(right, mid),
            Align2::RIGHT_CENTER,
            app_name,
            ty::SECONDARY,
            pal.ink_2,
        );
    }
    entry.clicked()
}

/// Minimise, maximise or restore, and close: 44 px wide each, the full height of the bar.
/// All three take the well fill on hover; close is not red, because colour is kept for state.
fn window_controls(ui: &Ui, pal: &Palette, rect: Rect, order: &[Icon], maximized: bool) {
    for (i, &control) in order.iter().enumerate() {
        let min = pos2(rect.left() + i as f32 * size::WINDOW_CONTROL_W, rect.top());
        let button = Rect::from_min_size(min, vec2(size::WINDOW_CONTROL_W, rect.height()));
        let (name, glyph, command) = match control {
            Icon::Minimise => (
                strings::MINIMISE,
                Icon::Minimise,
                ViewportCommand::Minimized(true),
            ),
            Icon::Close => (strings::CLOSE, Icon::Close, ViewportCommand::Close),
            _ if maximized => (
                strings::RESTORE,
                Icon::Restore,
                ViewportCommand::Maximized(false),
            ),
            _ => (
                strings::MAXIMISE,
                Icon::Maximise,
                ViewportCommand::Maximized(true),
            ),
        };

        let response = ui.interact(
            button,
            ui.scope_id().with(("window-control", i)),
            Sense::click(),
        );
        response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
        if response.is_pointer_button_down_on() {
            ui.painter().rect_filled(button, 0, pal.line);
        } else if response.hovered() {
            ui.painter().rect_filled(button, 0, pal.well);
        }
        draw::icon(
            ui,
            Rect::from_center_size(button.center(), vec2(14.0, 14.0)),
            glyph,
            pal.ink,
        );
        if response.clicked() {
            ui.ctx().send_viewport_cmd(command);
        }
    }
}

/// The edges and corners of a window without system decorations: dragging one resizes.
/// Call last, so the edges are on top of everything else.
pub fn resize_edges(ui: &Ui, chrome: Chrome) {
    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if !matches!(chrome, Chrome::Custom { .. }) || maximized {
        return;
    }

    const EDGE: f32 = 4.0;
    const CORNER: f32 = 10.0;
    let window = ui.max_rect();
    let (l, r, t, b) = (window.left(), window.right(), window.top(), window.bottom());
    let zones = [
        (
            Rect::from_min_max(pos2(l + CORNER, t), pos2(r - CORNER, t + EDGE)),
            ResizeDirection::North,
            CursorIcon::ResizeNorth,
        ),
        (
            Rect::from_min_max(pos2(l + CORNER, b - EDGE), pos2(r - CORNER, b)),
            ResizeDirection::South,
            CursorIcon::ResizeSouth,
        ),
        (
            Rect::from_min_max(pos2(l, t + CORNER), pos2(l + EDGE, b - CORNER)),
            ResizeDirection::West,
            CursorIcon::ResizeWest,
        ),
        (
            Rect::from_min_max(pos2(r - EDGE, t + CORNER), pos2(r, b - CORNER)),
            ResizeDirection::East,
            CursorIcon::ResizeEast,
        ),
        (
            Rect::from_min_max(pos2(l, t), pos2(l + CORNER, t + CORNER)),
            ResizeDirection::NorthWest,
            CursorIcon::ResizeNorthWest,
        ),
        (
            Rect::from_min_max(pos2(r - CORNER, t), pos2(r, t + CORNER)),
            ResizeDirection::NorthEast,
            CursorIcon::ResizeNorthEast,
        ),
        (
            Rect::from_min_max(pos2(l, b - CORNER), pos2(l + CORNER, b)),
            ResizeDirection::SouthWest,
            CursorIcon::ResizeSouthWest,
        ),
        (
            Rect::from_min_max(pos2(r - CORNER, b - CORNER), pos2(r, b)),
            ResizeDirection::SouthEast,
            CursorIcon::ResizeSouthEast,
        ),
    ];
    for (i, (zone, direction, cursor)) in zones.into_iter().enumerate() {
        let response = ui.interact(zone, ui.scope_id().with(("resize-edge", i)), Sense::drag());
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(cursor);
        }
        if response.drag_started() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::BeginResize(direction));
        }
    }
}
