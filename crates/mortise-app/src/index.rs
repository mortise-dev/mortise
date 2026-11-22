//! The module index (S3.1): the app, a filter, then "All modules" and one row per module.

use eframe::egui::{
    self, Align2, FontId, Rect, RichText, Role, ScrollArea, Sense, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{ALL_CODE, AppSnapshot, FindingState, Module, ModuleState};

use crate::app::Selection;
use crate::draw::{self, Icon, StampState};
use crate::strings;
use crate::theme::{Face, Palette, radius, size, stroke, ty};

/// Above this many modules a row has one line.
const TWO_LINE_LIMIT: usize = 20;

/// What the index needs from the app besides the snapshot.
pub struct IndexState<'a> {
    pub selection: &'a mut Selection,
    /// The text of the filter field.
    pub filter: &'a mut String,
    /// True in the frame the filter field should take focus.
    pub focus_filter: bool,
    /// True while the keyboard is in use: the focus ring is drawn.
    pub keyboard: bool,
    /// True in the frame after the selection moved by key: the row is scrolled into view.
    pub reveal: bool,
    /// True when the index is collapsed to its strip of stamps.
    pub collapsed: bool,
    /// The order: problems, then warnings, then the rest by name. False is by name.
    pub problems_first: bool,
    pub mac: bool,
}

/// What a row says at its right end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowState {
    /// A module that is turned off: no finding state.
    Off,
    /// The state of a module cannot be read, and it has no findings to show instead.
    None,
    Finding(FindingState),
}

/// The finding state of a module, for its row and for the order of the index.
fn row_state(app: &AppSnapshot, module: &Module) -> RowState {
    match module.state {
        ModuleState::Off => RowState::Off,
        _ => RowState::Finding(app.findings.state_of(&module.name)),
    }
}

/// The modules the index shows, in its order: those a filter leaves (its letters
/// are in the name or in the code, in any case), by name or with problems first.
pub fn ordered<'a>(app: &'a AppSnapshot, filter: &str, problems_first: bool) -> Vec<&'a Module> {
    let wanted = filter.trim().to_lowercase();
    let mut modules: Vec<&Module> = app
        .modules
        .iter()
        .filter(|module| {
            wanted.is_empty()
                || module.name.to_lowercase().contains(&wanted)
                || module.code.to_lowercase().contains(&wanted)
        })
        .collect();
    if problems_first {
        // The sort is stable, so the order by name is kept within each group.
        modules.sort_by_key(|module| attention(app, module));
    }
    modules
}

/// 0 for a module with a problem, 1 for one with a warning, 2 for the rest.
fn attention(app: &AppSnapshot, module: &Module) -> u8 {
    match row_state(app, module) {
        RowState::Finding(FindingState::Problems(_)) => 0,
        RowState::Finding(FindingState::Warning) => 1,
        _ => 2,
    }
}

/// The id of the filter field, to give it focus from a key.
pub fn filter_id(ui: &Ui) -> egui::Id {
    ui.scope_id().with("index-filter")
}

struct Row<'a> {
    selection: Selection,
    code: &'a str,
    name: &'a str,
    meta: String,
    state: RowState,
}

/// Paints the index into `rect`. The rect includes the 1 px joint at its right edge.
/// Returns true when the order of the list was asked to change.
pub fn index(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    app: &AppSnapshot,
    state: IndexState<'_>,
) -> bool {
    ui.painter().rect_filled(rect, 0, pal.panel);
    // The joint between index and workspace. The selected row is painted over it.
    draw::vline(
        ui,
        rect.right() - 1.0,
        rect.top()..rect.bottom(),
        pal.line_2,
    );
    let inner = rect.left()..rect.right() - 1.0;
    let IndexState {
        selection,
        filter,
        focus_filter,
        keyboard,
        reveal,
        collapsed,
        problems_first,
        mac,
    } = state;
    let mut toggle_order = false;

    let modules = ordered(app, filter, problems_first);
    // "All modules" carries the problems of the whole app.
    let all_state = match (app.findings.problems(), app.findings.warnings()) {
        (0, 0) => FindingState::Good,
        (0, _) => FindingState::Warning,
        (problems, _) => FindingState::Problems(problems),
    };
    let all = Row {
        selection: Selection::All,
        code: ALL_CODE,
        name: strings::ALL_MODULES,
        meta: strings::files_covered(
            app.files(),
            app.coverage().and_then(|coverage| coverage.percent()),
        ),
        state: RowState::Finding(all_state),
    };
    let (head_bottom, foot_top) = if collapsed {
        // The strip: stamps and marks only, and the key that opens the index again.
        let foot_top = rect.bottom() - 32.0;
        draw::hline(ui, inner.clone(), foot_top, pal.line);
        let keys = format!("{} B", strings::command_key(mac));
        let cap_w = draw::keycap_width(ui, &keys);
        let cap_right = ((inner.start + inner.end) / 2.0 + cap_w / 2.0).round();
        draw::keycap(ui, pal, pos2(cap_right, foot_top + 16.0), &keys);
        (rect.top(), foot_top)
    } else {
        let head_bottom = head(ui, pal, rect, app, filter, focus_filter, keyboard);
        // Foot: how many modules the filter leaves, or the order of the list.
        let foot_top = rect.bottom() - (8.0 + 15.0 + 8.0 + 1.0);
        draw::hline(ui, inner.clone(), foot_top, pal.line);
        let at = pos2(inner.start + 12.0, foot_top + 1.0 + 8.0);
        if filter.trim().is_empty() {
            let order = if problems_first {
                strings::SORTED_PROBLEMS_FIRST
            } else {
                strings::SORTED_BY_NAME
            };
            let text = draw::paint_text(ui, at, Align2::LEFT_TOP, order, ty::SECONDARY, pal.ink_2);
            // The order is a control: S, or a click on it, changes it.
            let response = ui.interact(
                text.expand(5.0),
                ui.scope_id().with("index-order"),
                Sense::click(),
            );
            response.widget_info(|| WidgetInfo::labeled(Role::Button, true, order));
            if response.hovered() {
                draw::paint_text(ui, at, Align2::LEFT_TOP, order, ty::SECONDARY, pal.ink);
            }
            if keyboard && response.has_focus() {
                draw::focus_ring(ui, pal, text.expand(3.0), radius::STAMP);
            }
            toggle_order = response.clicked();
            draw::tooltip(response, pal, strings::CHANGE_THE_ORDER);
        } else {
            let shown = strings::of_modules(modules.len(), app.modules.len());
            draw::label(ui, at, Align2::LEFT_TOP, &shown, ty::SECONDARY, pal.ink_2);
        }
        (head_bottom, foot_top)
    };

    // The list reaches the right edge of the index, so that the selected row can cover the joint.
    let list = Rect::from_min_max(pos2(rect.left(), head_bottom), pos2(rect.right(), foot_top));
    let compact = app.modules.len() > TWO_LINE_LIMIT;
    let row_h = if compact {
        size::INDEX_ROW_COMPACT_H
    } else {
        size::INDEX_ROW_H
    };
    let no_match = (!filter.trim().is_empty() && modules.is_empty())
        .then(|| strings::no_module_matches(filter.trim()));
    // With many modules and problems first, those that need attention get a heading.
    let needing = modules
        .iter()
        .filter(|module| attention(app, module) < 2)
        .count();
    let pinned = compact && problems_first && !collapsed && needing > 0;

    ui.scope_builder(UiBuilder::new().max_rect(list), |ui| {
        ScrollArea::vertical()
            .id_salt("index")
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.add_space(8.0);
                let mut show = |ui: &mut Ui, row: Row<'_>| {
                    let (slot, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::hover());
                    // Rows start 8 px in from the left edge of the index.
                    let row_rect =
                        Rect::from_min_max(pos2(slot.left() + 8.0, slot.top()), slot.max);
                    let selected = *selection == row.selection;
                    let look = RowLook {
                        selected,
                        focused: selected && keyboard,
                        compact,
                        collapsed,
                    };
                    if index_row(ui, pal, row_rect, &row, look) {
                        *selection = row.selection;
                    }
                    if selected && reveal {
                        ui.scroll_to_rect(row_rect.expand2(vec2(0.0, 8.0)), None);
                    }
                };
                show(ui, all);
                if pinned {
                    group_heading(ui, pal, strings::NEEDS_ATTENTION, needing);
                }
                for (i, module) in modules.iter().enumerate() {
                    if pinned && i == needing {
                        ui.add_space(8.0);
                    }
                    show(ui, module_row(app, module));
                }
                if let Some(text) = &no_match {
                    let width = ui.available_width() - 14.0 - 13.0;
                    let (slot, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 60.0), Sense::hover());
                    if !collapsed {
                        draw::paragraph(
                            ui,
                            pos2(slot.left() + 14.0, slot.top() + 8.0),
                            text,
                            ty::SECONDARY,
                            pal.ink_2,
                            width,
                        );
                    }
                }
                ui.add_space(8.0);
            });
    });
    toggle_order
}

/// The row of one module: its code, its name, its files and coverage, its state.
fn module_row<'a>(app: &AppSnapshot, module: &'a Module) -> Row<'a> {
    let state = row_state(app, module);
    let coverage = module.coverage.and_then(|coverage| coverage.percent());
    Row {
        selection: Selection::Module(module.name.clone()),
        code: &module.code,
        name: &module.name,
        meta: if state == RowState::Off {
            strings::files_turned_off(module.files)
        } else {
            strings::files_covered(module.files, coverage)
        },
        // Without a state that can be read there is no check mark to give.
        state: if module.state == ModuleState::Unknown
            && state == RowState::Finding(FindingState::Good)
        {
            RowState::None
        } else {
            state
        },
    }
}

/// The heading of a group of rows: its name and how many modules it holds.
fn group_heading(ui: &mut Ui, pal: &Palette, name: &str, count: usize) {
    let (slot, _) = ui.allocate_exact_size(vec2(ui.available_width(), 27.0), Sense::hover());
    let mid = slot.top() + 9.0 + 7.5;
    draw::label(
        ui,
        pos2(slot.left() + 8.0 + 14.0, mid),
        Align2::LEFT_CENTER,
        name,
        ty::LABEL,
        pal.ink_2,
    );
    draw::label(
        ui,
        pos2(slot.right() - 1.0 - 12.0, mid),
        Align2::RIGHT_CENTER,
        &count.to_string(),
        ty::SECONDARY,
        pal.ink_2,
    );
}

/// The head of the index: the app, how many modules are on, the versions, the filter.
/// Returns its bottom edge.
fn head(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    app: &AppSnapshot,
    filter: &mut String,
    focus_filter: bool,
    keyboard: bool,
) -> f32 {
    let left = rect.left() + 12.0;
    let right = rect.right() - 1.0 - 12.0;
    let on_text = if app.layout.uses_file_activator() {
        strings::of_on(app.modules_on(), app.modules.len())
    } else {
        strings::modules(app.modules.len())
    };
    let on = draw::label(
        ui,
        pos2(right, rect.top() + 12.0 + 18.0),
        Align2::RIGHT_BOTTOM,
        &on_text,
        ty::SECONDARY,
        pal.ink_2,
    );
    let name = draw::galley_elided(ui, &app.name, ty::APP_NAME, pal.ink, on.left() - 8.0 - left);
    let name = draw::paint_galley(ui, pos2(left, rect.top() + 12.0), Align2::LEFT_TOP, name);
    draw::announce(ui, name, Role::Heading, &app.name);
    draw::label(
        ui,
        pos2(left, rect.top() + 12.0 + 20.0 + 1.0),
        Align2::LEFT_TOP,
        &strings::versions(app.laravel.as_ref(), &app.package),
        ty::SECONDARY,
        pal.ink_2,
    );

    // The filter field (C3): 26 px high, a search icon, a key hint that says how to
    // get in and, once inside, how to get out.
    let field = Rect::from_min_max(
        pos2(left, rect.top() + 12.0 + 20.0 + 1.0 + 16.0 + 9.0),
        pos2(right, rect.top() + 84.0),
    );
    let id = filter_id(ui);
    let focused = ui.memory(|memory| memory.has_focus(id));
    let outline = if focused {
        pal.ink
    } else if ui.rect_contains_pointer(field) {
        pal.ink_2
    } else {
        pal.line_2
    };
    draw::outlined(ui, field, radius::CONTROL, pal.surface, outline);
    let glass = Rect::from_min_size(
        pos2(field.left() + 8.0, field.center().y - 6.5),
        Vec2::splat(13.0),
    );
    let has_text = !filter.is_empty();
    draw::icon(
        ui,
        glass,
        Icon::Search,
        if focused || has_text {
            pal.ink
        } else {
            pal.ink_3
        },
    );
    let keys = if focused || has_text { "Esc" } else { "/" };
    let cap = draw::keycap(ui, pal, pos2(field.right() - 8.0, field.center().y), keys);

    let text_rect = Rect::from_min_max(
        pos2(glass.right() + 7.0, field.top() + 1.0),
        pos2(cap.left() - 6.0, field.bottom() - 1.0),
    );
    let hint = RichText::new(strings::FILTER_MODULES)
        .color(pal.ink_3)
        .font(FontId::new(12.0, Face::Sans400.family()));
    let edit = egui::TextEdit::singleline(filter)
        .id(id)
        .frame(egui::Frame::NONE)
        .margin(egui::Margin::ZERO)
        .font(FontId::new(12.0, Face::Sans400.family()))
        .text_color(pal.ink)
        .desired_width(text_rect.width())
        .align(Align2::LEFT_CENTER)
        .hint_text(hint);
    let response = ui.put(text_rect, edit);
    response.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, strings::FILTER_MODULES));
    if focus_filter {
        response.request_focus();
    }
    if keyboard && focused {
        draw::focus_ring(ui, pal, field, radius::CONTROL);
    }

    let bottom = field.bottom() + 10.0;
    draw::hline(ui, rect.left()..rect.right() - 1.0, bottom - 1.0, pal.line);
    bottom
}

#[derive(Debug, Clone, Copy)]
struct RowLook {
    selected: bool,
    /// The focus ring is drawn around the row.
    focused: bool,
    /// One line: 26 px, no second line.
    compact: bool,
    /// The index is a strip: the stamp and the mark only.
    collapsed: bool,
}

/// Paints one row (C7). Returns true when it was clicked.
fn index_row(ui: &Ui, pal: &Palette, rect: Rect, row: &Row<'_>, look: RowLook) -> bool {
    // The interaction comes first so that hover is known before painting.
    let response = ui.interact(
        rect,
        ui.scope_id().with(("index-row", row.code)),
        Sense::click(),
    );
    response
        .widget_info(|| WidgetInfo::selected(Role::ListBoxOption, true, look.selected, row.name));

    let behind = if look.selected {
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
        pal.surface
    } else if response.hovered() {
        let fill = Rect::from_min_max(rect.min, pos2(rect.right() - 1.0, rect.bottom()));
        ui.painter().rect_filled(fill, 0, pal.well);
        pal.well
    } else {
        pal.panel
    };

    let mid = rect.center().y;
    let off = row.state == RowState::Off;
    let stamp_state = StampState {
        selected: look.selected,
        off,
    };
    let stamp_min = pos2(rect.left() + 15.0, (mid - size::STAMP_H / 2.0).round());
    let stamp = draw::stamp(ui, pal, stamp_min, row.code, stamp_state);

    // The state at the right end: a mark, and a word or a number where there is room.
    let marks_right = if look.collapsed {
        stamp.right() + 5.0 + size::STATE_MARK
    } else {
        rect.right() - 1.0 - 12.0
    };
    let (word, color) = match row.state {
        RowState::Off => (strings::OFF.to_owned(), pal.off),
        RowState::Finding(FindingState::Problems(n)) => (n.to_string(), pal.bad),
        RowState::Finding(FindingState::Warning) => (String::new(), pal.warn),
        RowState::Finding(FindingState::Good) => (String::new(), pal.good),
        RowState::None => (String::new(), pal.off),
    };
    let mut mark_right = marks_right;
    if !word.is_empty() && !look.collapsed {
        let text = draw::paint_text(
            ui,
            pos2(marks_right, mid),
            Align2::RIGHT_CENTER,
            &word,
            ty::LABEL,
            color,
        );
        mark_right = text.left() - 5.0;
    }
    let mark = Rect::from_min_size(
        pos2(
            mark_right - size::STATE_MARK,
            (mid - size::STATE_MARK / 2.0).round(),
        ),
        Vec2::splat(size::STATE_MARK),
    );
    match row.state {
        RowState::Off => draw::mark_off(ui, mark, color),
        RowState::Finding(FindingState::Problems(_)) => draw::mark_bad(ui, mark, color, behind),
        RowState::Finding(FindingState::Warning) => draw::mark_warn(ui, mark, color),
        RowState::Finding(FindingState::Good) => draw::mark_good(ui, mark, color),
        RowState::None => {}
    }

    if !look.collapsed {
        let text_left = rect.left() + 15.0 + 36.0 + 9.0;
        let text_w = (mark.left() - 9.0 - text_left).max(0.0);
        let name_type = if off {
            ty::ROW_NAME.with_face(Face::Sans500)
        } else {
            ty::ROW_NAME
        };
        let name_color = if off { pal.ink_2 } else { pal.ink };
        let name = draw::galley_elided(ui, row.name, name_type, name_color, text_w);
        if look.compact {
            draw::paint_galley(ui, pos2(text_left, mid), Align2::LEFT_CENTER, name);
        } else {
            // Name over the second line: 17 and 15 px, centred in the row together.
            let top = (mid - 16.0).round();
            draw::paint_galley(ui, pos2(text_left, top), Align2::LEFT_TOP, name);
            let meta = draw::galley_elided(ui, &row.meta, ty::SECONDARY, pal.ink_2, text_w);
            draw::paint_galley(ui, pos2(text_left, top + 17.0), Align2::LEFT_TOP, meta);
        }
    }

    if look.focused {
        // The ring is 1 px outside the row, so it needs more room than the list's clip gives.
        let clip = ui.clip_rect().expand2(vec2(4.0, 0.0));
        ui.painter().with_clip_rect(clip).rect_stroke(
            rect.expand(1.0),
            1,
            Stroke::new(stroke::FOCUS_RING, pal.ink),
            StrokeKind::Outside,
        );
    }

    let clicked = response.clicked();
    if look.collapsed {
        draw::tooltip(response, pal, row.name);
    }
    clicked
}
