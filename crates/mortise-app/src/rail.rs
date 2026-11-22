//! The app rail (S2.1): 52 px wide, one 34 px tile per app, then the add tile.

use eframe::egui::scroll_area::ScrollBarVisibility;
use eframe::egui::{
    Align2, Color32, Rect, Role, ScrollArea, Sense, Shape, Stroke, Ui, UiBuilder, Vec2, WidgetInfo,
    pos2, vec2,
};

use crate::draw;
use crate::strings;
use crate::theme::{Face, Palette, Type, radius, size, stroke, ty};

pub struct Tile<'a> {
    pub name: &'a str,
    pub monogram: &'a str,
    /// How many findings of the kind problem the app has. Any raises the pip.
    pub problems: usize,
}

/// Cmd on macOS, Ctrl elsewhere.
fn command_key(ui: &Ui) -> &'static str {
    strings::command_key(ui.ctx().os() == eframe::egui::os::OperatingSystem::Mac)
}

fn current_tile_id() -> eframe::egui::Id {
    eframe::egui::Id::unique("mortise-rail-current-tile")
}

/// The tile of the app on screen, to give it focus.
pub fn current_tile(ctx: &eframe::egui::Context) -> Option<eframe::egui::Id> {
    ctx.data(|data| data.get_temp::<Option<eframe::egui::Id>>(current_tile_id()))
        .flatten()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailAction {
    Select(usize),
    /// Move the app at the first place to the second.
    Move(usize, usize),
    Add,
    Settings,
}

/// The gap between two tiles.
const GAP: f32 = 8.0;

/// Paints the rail into `rect`. `adding` is true while the add panel is open: the
/// add tile then has the focus ring.
pub fn rail(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    tiles: &[Tile<'_>],
    current: Option<usize>,
    adding: bool,
    settings_open: bool,
) -> Option<RailAction> {
    ui.painter().rect_filled(rect, 0, pal.frame);
    draw::vline(
        ui,
        rect.right() - 1.0,
        rect.top()..rect.bottom(),
        pal.line_2,
    );

    let mut action = None;
    let x = rect.left() + ((rect.width() - 1.0 - size::RAIL_TILE) / 2.0).round();
    let step = size::RAIL_TILE + GAP;
    // Settings stay at the bottom of the rail.
    let settings = Rect::from_min_size(
        pos2(x, rect.bottom() - 10.0 - size::RAIL_TILE),
        Vec2::splat(size::RAIL_TILE),
    );
    let rect = Rect::from_min_max(rect.min, pos2(rect.right(), settings.top()));
    let fits = 10.0 + tiles.len() as f32 * step + size::RAIL_TILE + 10.0 <= rect.height();

    // When the tiles do not fit, the add tile stays at the bottom and the tiles scroll above it.
    let list_bottom = if fits {
        rect.bottom()
    } else {
        rect.bottom() - 10.0 - size::RAIL_TILE - GAP
    };
    let list = Rect::from_min_max(rect.min, pos2(rect.right() - 1.0, list_bottom));
    let mut add_min = pos2(x, rect.bottom() - 10.0 - size::RAIL_TILE);

    let output = ui
        .scope_builder(UiBuilder::new().max_rect(list), |ui| {
            ScrollArea::vertical()
                .id_salt("rail")
                .auto_shrink(false)
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    ui.add_space(10.0);
                    for (i, tile) in tiles.iter().enumerate() {
                        let (slot, _) = ui
                            .allocate_exact_size(vec2(ui.available_width(), step), Sense::hover());
                        let tile_rect =
                            Rect::from_min_size(pos2(x, slot.top()), Vec2::splat(size::RAIL_TILE));
                        let selected = current == Some(i) && !adding;
                        let response = app_tile(ui, pal, tile_rect, tile, selected);
                        if response.clicked() {
                            action = Some(RailAction::Select(i));
                        }
                        // The order of the rail: a tile that is dragged over another
                        // takes its place. Alt with Up or Down does the same by key.
                        let last = tiles.len() - 1;
                        if response.dragged()
                            && let Some(pointer) = response.interact_pointer_pos()
                        {
                            let over = ((pointer.y - tile_rect.top()) / step).floor() as isize;
                            let target = (i as isize + over).clamp(0, last as isize) as usize;
                            if target != i {
                                action = Some(RailAction::Move(i, target));
                            }
                        }
                        if response.has_focus() {
                            use eframe::egui::{Key, Modifiers};
                            let (up, down) = ui.input_mut(|input| {
                                (
                                    input.consume_key(Modifiers::ALT, Key::ArrowUp),
                                    input.consume_key(Modifiers::ALT, Key::ArrowDown),
                                )
                            });
                            if up && i > 0 {
                                action = Some(RailAction::Move(i, i - 1));
                            } else if down && i < last {
                                action = Some(RailAction::Move(i, i + 1));
                            }
                        }
                    }
                    if fits {
                        add_min = pos2(x, ui.cursor().top());
                    }
                })
        })
        .inner;

    // No scrollbar in a 52 px column: a line at the edge shows that there is more.
    if !fits {
        let offset = output.state.clamped_offset().y;
        if offset > 0.5 {
            draw::hline(ui, list.left()..list.right(), list.top(), pal.ink_2);
        }
        if offset + output.inner_rect.height() < output.content_size.y - 0.5 {
            draw::hline(
                ui,
                list.left()..list.right(),
                list.bottom() - 1.0,
                pal.ink_2,
            );
        }
    }

    if add_tile(
        ui,
        pal,
        Rect::from_min_size(add_min, Vec2::splat(size::RAIL_TILE)),
        adding,
    ) {
        action = Some(RailAction::Add);
    }
    if settings_tile(ui, pal, settings, settings_open) {
        action = Some(RailAction::Settings);
    }
    action
}

/// A tile with the monogram of an app. The selected tile is a key block.
fn app_tile(
    ui: &Ui,
    pal: &Palette,
    rect: Rect,
    tile: &Tile<'_>,
    selected: bool,
) -> eframe::egui::Response {
    let response = ui.interact(
        rect,
        ui.scope_id().with(("rail-tile", tile.name)),
        Sense::click_and_drag(),
    );
    if selected {
        // For F6: the tile that takes focus when focus moves to the rail.
        ui.ctx()
            .data_mut(|data| data.insert_temp(current_tile_id(), Some(response.id)));
    }
    response.widget_info(|| WidgetInfo::selected(Role::ListBoxOption, true, selected, tile.name));

    // On frame, an outline is ink-2: line-2 is under 3 to 1 there in the light theme.
    let (fill, outline, text) = if selected {
        (pal.key, pal.key, pal.key_fg)
    } else if response.is_pointer_button_down_on() {
        (pal.line, pal.ink_2, pal.ink_2)
    } else if response.hovered() {
        (pal.well, pal.ink_2, pal.ink_2)
    } else {
        (pal.panel, pal.ink_2, pal.ink_2)
    };
    draw::outlined(ui, rect, radius::CONTROL, fill, outline);
    draw::paint_text(
        ui,
        rect.center(),
        Align2::CENTER_CENTER,
        tile.monogram,
        ty::MONOGRAM,
        text,
    );

    // The pip: the app has at least one problem. Warnings and a runtime that is
    // down do not raise it.
    if tile.problems > 0 {
        let center = pos2(rect.right() - 0.5, rect.top() + 0.5);
        ui.painter().circle_filled(center, 6.0, pal.frame);
        draw::mark_bad(
            ui,
            Rect::from_center_size(center, Vec2::splat(9.0)),
            pal.bad,
            pal.frame,
        );
    }

    draw::tooltip(
        response,
        pal,
        &strings::app_with_problems(tile.name, tile.problems),
    )
}

/// The add tile: a dashed outline and a plus.
fn add_tile(ui: &Ui, pal: &Palette, rect: Rect, active: bool) -> bool {
    let response = ui.interact(rect, ui.scope_id().with("rail-add"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, strings::ADD_APP));

    let fill = if response.is_pointer_button_down_on() {
        pal.line
    } else if response.hovered() {
        pal.well
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, radius::CONTROL, fill);
    let inner = rect.shrink(0.5);
    let corners = [
        inner.left_top(),
        inner.right_top(),
        inner.right_bottom(),
        inner.left_bottom(),
    ];
    for (from, to) in corners.into_iter().zip(corners.into_iter().cycle().skip(1)) {
        let line = Stroke::new(stroke::HAIRLINE, pal.ink_2);
        ui.painter()
            .extend(Shape::dashed_line(&[from, to], line, 3.0, 2.0));
    }
    let plus = Type {
        size: 15.0,
        line: 15.0,
        face: Face::Sans400,
        spacing: 0.0,
    };
    draw::paint_text(
        ui,
        rect.center(),
        Align2::CENTER_CENTER,
        "+",
        plus,
        pal.ink_2,
    );
    if active || response.has_focus() {
        draw::focus_ring(ui, pal, rect, radius::CONTROL);
    }

    let clicked = response.clicked();
    let keys = format!("{} O", command_key(ui));
    draw::tooltip_keys(response, pal, strings::ADD_APP, &keys);
    clicked
}

/// The Settings tile: an icon without an outline. Selected, it is a key block.
fn settings_tile(ui: &Ui, pal: &Palette, rect: Rect, selected: bool) -> bool {
    let response = ui.interact(rect, ui.scope_id().with("rail-settings"), Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, "Settings"));
    let (fill, color) = if selected {
        (pal.key, pal.key_fg)
    } else if response.is_pointer_button_down_on() {
        (pal.line, pal.ink)
    } else if response.hovered() {
        (pal.well, pal.ink)
    } else {
        (Color32::TRANSPARENT, pal.ink)
    };
    ui.painter().rect_filled(rect, radius::CONTROL, fill);
    draw::icon(
        ui,
        Rect::from_center_size(rect.center(), Vec2::splat(size::ICON)),
        draw::Icon::Settings,
        color,
    );
    if response.has_focus() {
        draw::focus_ring(ui, pal, rect, radius::CONTROL);
    }
    let clicked = response.clicked();
    let keys = format!("{} ,", command_key(ui));
    draw::tooltip_keys(response, pal, "Settings", &keys);
    clicked
}
