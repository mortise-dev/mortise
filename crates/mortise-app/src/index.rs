//! The module index (S3.1): the app, then "All modules" and one row per module.

use eframe::egui::{
    Align2, Rect, Role, ScrollArea, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{ALL_CODE, AppSnapshot, ModuleState};

use crate::app::Selection;
use crate::draw::{self, StampState};
use crate::strings;
use crate::theme::{Palette, size, ty};

/// What the index needs to know from the app besides the snapshot.
pub struct IndexState<'a> {
    pub selection: &'a mut Selection,
    /// True while the keyboard is in use: the focus ring is drawn.
    pub keyboard: bool,
    /// True in the frame after the selection moved by key: the row is scrolled into view.
    pub reveal: bool,
}

struct Row<'a> {
    selection: Selection,
    code: &'a str,
    name: &'a str,
    meta: String,
    off: bool,
}

/// Paints the index into `rect`. The rect includes the 1 px joint at its right edge.
pub fn index(ui: &mut Ui, pal: &Palette, rect: Rect, app: &AppSnapshot, state: IndexState<'_>) {
    ui.painter().rect_filled(rect, 0, pal.panel);
    // The joint between index and workspace. The selected row is painted over it.
    draw::vline(
        ui,
        rect.right() - 1.0,
        rect.top()..rect.bottom(),
        pal.line_2,
    );

    let inner = rect.left()..rect.right() - 1.0;

    // Head: the app, how many modules are on, the versions.
    let head_bottom = rect.top() + 12.0 + 20.0 + 1.0 + 16.0 + 10.0;
    let on_text = if app.layout.uses_file_activator() {
        strings::of_on(app.modules_on(), app.modules.len())
    } else {
        strings::modules(app.modules.len())
    };
    let on = draw::label(
        ui,
        pos2(inner.end - 12.0, rect.top() + 12.0 + 18.0),
        Align2::RIGHT_BOTTOM,
        &on_text,
        ty::SECONDARY,
        pal.ink_2,
    );
    let name_w = on.left() - 8.0 - (inner.start + 12.0);
    let name = draw::galley_elided(ui, &app.name, ty::APP_NAME, pal.ink, name_w);
    let name = draw::paint_galley(
        ui,
        pos2(inner.start + 12.0, rect.top() + 12.0),
        Align2::LEFT_TOP,
        name,
    );
    draw::announce(ui, name, Role::Heading, &app.name);
    draw::label(
        ui,
        pos2(inner.start + 12.0, rect.top() + 12.0 + 20.0 + 1.0),
        Align2::LEFT_TOP,
        &strings::versions(app.laravel.as_ref(), &app.package),
        ty::SECONDARY,
        pal.ink_2,
    );
    draw::hline(ui, inner.clone(), head_bottom - 1.0, pal.line);

    // Foot: the order of the list.
    let foot_top = rect.bottom() - (8.0 + 15.0 + 8.0 + 1.0);
    draw::hline(ui, inner.clone(), foot_top, pal.line);
    draw::label(
        ui,
        pos2(inner.start + 12.0, foot_top + 1.0 + 8.0),
        Align2::LEFT_TOP,
        strings::SORTED_BY_NAME,
        ty::SECONDARY,
        pal.ink_2,
    );

    // The rows. "All modules" is always first.
    let rows = std::iter::once(Row {
        selection: Selection::All,
        code: ALL_CODE,
        name: strings::ALL_MODULES,
        meta: strings::files(app.files()),
        off: false,
    })
    .chain(app.modules.iter().map(|module| {
        let off = module.state == ModuleState::Off;
        Row {
            selection: Selection::Module(module.name.clone()),
            code: &module.code,
            name: &module.name,
            meta: if off {
                strings::files_turned_off(module.files)
            } else {
                strings::files(module.files)
            },
            off,
        }
    }));

    // The list reaches the right edge of the index, so that the selected row can cover the joint.
    let list = Rect::from_min_max(pos2(rect.left(), head_bottom), pos2(rect.right(), foot_top));
    let IndexState {
        selection,
        keyboard,
        reveal,
    } = state;
    ui.scope_builder(UiBuilder::new().max_rect(list), |ui| {
        ScrollArea::vertical()
            .id_salt("index")
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.add_space(8.0);
                for row in rows {
                    let (slot, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), size::INDEX_ROW_H),
                        Sense::hover(),
                    );
                    // Rows start 8 px in from the left edge of the index.
                    let row_rect =
                        Rect::from_min_max(pos2(slot.left() + 8.0, slot.top()), slot.max);
                    let selected = *selection == row.selection;
                    if index_row(ui, pal, row_rect, &row, selected, selected && keyboard) {
                        *selection = row.selection;
                    }
                    if selected && reveal {
                        ui.scroll_to_rect(row_rect.expand2(vec2(0.0, 8.0)), None);
                    }
                }
                ui.add_space(8.0);
            });
    });
}

/// Paints one row (C7). Returns true when it was clicked.
fn index_row(
    ui: &Ui,
    pal: &Palette,
    rect: Rect,
    row: &Row<'_>,
    selected: bool,
    focused: bool,
) -> bool {
    // The interaction comes first so that hover is known before painting.
    let response = ui.interact(
        rect,
        ui.scope_id().with(("index-row", row.code)),
        Sense::click(),
    );
    response.widget_info(|| WidgetInfo::selected(Role::ListBoxOption, true, selected, row.name));

    if selected {
        // Surface fill, a line on three sides, and 1 px over the joint into the workspace.
        ui.painter().rect_filled(rect, 0, pal.surface);
        draw::hline(ui, rect.left()..rect.right(), rect.top(), pal.line_2);
        draw::hline(
            ui,
            rect.left()..rect.right(),
            rect.bottom() - 1.0,
            pal.line_2,
        );
        draw::vline(ui, rect.left(), rect.top()..rect.bottom(), pal.line_2);
        draw::peg(ui, pal, rect);
    } else if response.hovered() {
        let fill = Rect::from_min_max(rect.min, pos2(rect.right() - 1.0, rect.bottom()));
        ui.painter().rect_filled(fill, 0, pal.well);
    }

    let mid = rect.center().y;
    let stamp_state = StampState {
        selected,
        off: row.off,
    };
    let stamp_min = pos2(rect.left() + 15.0, (mid - size::STAMP_H / 2.0).round());
    draw::stamp(ui, pal, stamp_min, row.code, stamp_state);

    // The marks at the right end. A module that is off has the empty frame and the word.
    let marks_right = rect.right() - 1.0 - 12.0;
    let mut text_right = marks_right;
    if row.off {
        let word = draw::paint_text(
            ui,
            pos2(marks_right, mid),
            Align2::RIGHT_CENTER,
            strings::OFF,
            ty::LABEL,
            pal.off,
        );
        let mark = Rect::from_min_size(
            pos2(
                word.left() - 5.0 - size::STATE_MARK,
                (mid - size::STATE_MARK / 2.0).round(),
            ),
            Vec2::splat(size::STATE_MARK),
        );
        draw::mark_off(ui, mark, pal.off);
        text_right = mark.left() - 9.0;
    }

    // Name over the second line: 17 and 15 px, centred in the row together.
    let text_left = rect.left() + 15.0 + 36.0 + 9.0;
    let text_w = (text_right - text_left).max(0.0);
    let top = (mid - 16.0).round();
    let name_type = if row.off {
        ty::ROW_NAME.with_face(crate::theme::Face::Sans500)
    } else {
        ty::ROW_NAME
    };
    let name_color = if row.off { pal.ink_2 } else { pal.ink };
    let name = draw::galley_elided(ui, row.name, name_type, name_color, text_w);
    draw::paint_galley(ui, pos2(text_left, top), Align2::LEFT_TOP, name);
    let meta = draw::galley_elided(ui, &row.meta, ty::SECONDARY, pal.ink_2, text_w);
    draw::paint_galley(ui, pos2(text_left, top + 17.0), Align2::LEFT_TOP, meta);

    if focused {
        // The ring is 1 px outside the row, so it needs more room than the list's clip gives.
        let clip = ui.clip_rect().expand2(vec2(4.0, 0.0));
        ui.painter().with_clip_rect(clip).rect_stroke(
            rect.expand(1.0),
            1,
            eframe::egui::Stroke::new(crate::theme::stroke::FOCUS_RING, pal.ink),
            eframe::egui::StrokeKind::Outside,
        );
    }

    response.clicked()
}
