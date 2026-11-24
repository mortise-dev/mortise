//! The public surface of a module (S5.5 to S5.7): the second view of the Boundary tab.
//!
//! A scan proposes what other modules, HTTP clients and the command line may use.
//! It never starts by itself and it writes nothing. Accepting a proposal needs a
//! place in the package to store it, which does not exist yet.

use std::time::Instant;

use eframe::egui::{
    Align2, Key, KeyboardShortcut, Modifiers, Rect, Role, Ui, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{AppSnapshot, Module, Proposal, SurfaceEntry, SurfaceKind, SurfaceScan};

use crate::app::workspace::WorkAction;
use crate::boundary::strings;
use crate::ui::draw::{self, Button, ButtonKind, Icon};
use crate::ui::draw::{EmptyMark, empty_state};
use crate::ui::theme::{Palette, Type, radius, size, ty};

/// How many files a scan reads in one frame.
const FILES_A_FRAME: usize = 8;

/// What the view keeps for one module.
#[derive(Default)]
pub struct ModuleSurface {
    scan: Option<SurfaceScan>,
    pub proposal: Option<Proposal>,
    /// When the proposal was made.
    scanned: Option<Instant>,
    /// The text of the field that adds an entry by hand.
    add_text: String,
    /// The add field takes focus in the next frame.
    focus_add: bool,
    /// The first entry of the proposal takes focus in the next frame.
    focus_first: bool,
}

/// What a scan takes from the running app, when an environment is connected.
pub struct SurfaceInputs<'a> {
    /// The URIs of the module's routes. `None` when the mappings were not read.
    pub routes: Option<Vec<String>>,
    /// The scheduled tasks of the app: a target and its schedule.
    pub scheduled: Vec<(String, String)>,
    /// The name of the environment the routes were read from.
    pub environment: &'a str,
}

impl ModuleSurface {
    pub fn is_scanning(&self) -> bool {
        self.scan.is_some()
    }

    /// How long ago the proposal was made, in seconds.
    pub fn scanned(&self) -> Option<u64> {
        self.proposal.as_ref()?;
        self.scanned.map(|at| at.elapsed().as_secs())
    }

    /// Starts a scan. One that runs is left alone.
    pub fn start(&mut self, app: &AppSnapshot, module: &str) {
        if self.scan.is_none() {
            self.scan = SurfaceScan::start(app, module);
        }
    }

    /// Stops the scan. It wrote nothing, so the view is as it was.
    pub fn stop(&mut self) {
        self.scan = None;
    }

    /// Reads a few more files. When all are read the proposal is made; it keeps
    /// what the developer chose in the one before.
    fn advance(&mut self, inputs: &SurfaceInputs<'_>) {
        let Some(scan) = &mut self.scan else {
            return;
        };
        if !scan.step(FILES_A_FRAME) {
            return;
        }
        let Some(scan) = self.scan.take() else {
            return;
        };
        let mut proposal = scan.finish(inputs.routes.as_deref(), &inputs.scheduled);
        if let Some(earlier) = &self.proposal {
            proposal.keep_choices_of(earlier);
        }
        self.proposal = Some(proposal);
        self.scanned = Some(Instant::now());
        self.focus_first = true;
    }
}

/// Where the view is painted.
pub struct Place<'a> {
    pub pal: &'a Palette,
    /// The part of the workspace body under the control that switches the views.
    pub area: Rect,
    pub left: f32,
    pub width: f32,
    pub keyboard: bool,
    pub mac: bool,
}

/// Paints the public surface of a module from `area.top()` down. Returns its
/// bottom edge and what was asked for.
pub fn surface(
    ui: &mut Ui,
    place: &Place<'_>,
    app: &AppSnapshot,
    module: &Module,
    state: &mut ModuleSurface,
    inputs: &SurfaceInputs<'_>,
) -> (f32, Option<WorkAction>) {
    let pal = place.pal;
    let typing = ui.ctx().text_edit_focused();
    let run = KeyboardShortcut::new(Modifiers::COMMAND, Key::Enter);
    let run_keys = format!("{} Enter", crate::strings::command_key(place.mac));

    if state.is_scanning() {
        state.advance(inputs);
        // The next files are read in the next frame.
        ui.ctx().request_repaint();
    }
    if let Some(scan) = &state.scan {
        let (bottom, stop) = scanning(ui, place, scan, inputs);
        if stop {
            state.stop();
        }
        return (bottom, None);
    }

    if state.proposal.is_none() {
        // S5.5: not scanned yet. A scan never starts by itself.
        let title = strings::no_surface_yet(&module.name);
        let text = strings::scan_explained(&module.name, &app.rules.public_folders);
        let at = empty_state(
            ui,
            pal,
            place.area,
            EmptyMark::Icon(Icon::Boundary),
            &title,
            &[&text],
            size::CONTROL_H,
        );
        let label = strings::scan_module(&module.name);
        let button = Button {
            label: &label,
            kind: ButtonKind::Primary,
            keys: Some(&run_keys),
            ..Default::default()
        };
        let clicked = button.show(ui, pal, at, place.keyboard).clicked();
        if clicked || ui.input_mut(|input| input.consume_shortcut(&run)) {
            state.start(app, &module.name);
            ui.ctx().request_repaint();
        }
        return (at.y + size::CONTROL_H, None);
    }

    // S5.7: the proposal.
    let mut action = None;
    let mut scan_again = false;
    // A: to the field that adds an entry by hand.
    if !typing && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::A)) {
        state.focus_add = true;
    }
    let focus_first = std::mem::take(&mut state.focus_first);
    let focus_add = std::mem::take(&mut state.focus_add);
    let ModuleSurface {
        proposal: Some(proposal),
        add_text,
        ..
    } = state
    else {
        return (place.area.top(), None);
    };

    let rows = rows(proposal, inputs);
    const ROW: f32 = size::TABLE_ROW_H;
    let verdict = (!add_text.trim().is_empty()).then(|| proposal.check(add_text));
    let foot_h = 44.0 + if verdict.is_some() { 18.0 } else { 0.0 };
    let title = strings::proposed_surface_of(&proposal.module);
    let note = strings::found_proposed(proposal.found(), proposal.proposed());
    let head = 41.0 - 6.0 + draw::box_head_extra(ui, place.width, &title, &note);
    let table_top = place.area.top() + head;
    let height = head + size::TABLE_HEAD_H + rows.len() as f32 * ROW + foot_h;
    let rect = Rect::from_min_size(
        pos2(place.left, place.area.top()),
        vec2(place.width, height),
    );
    draw::box_head(ui, pal, rect, &title, &note);

    // The columns: the entry, why it is here, who uses it today.
    let entry_x = rect.left() + 46.0;
    let why_x = (rect.left() + rect.width() * 0.455).round();
    let used_x = (rect.left() + rect.width() * 0.728).round();
    let right = rect.right() - 13.0;
    let head_bottom = table_top + size::TABLE_HEAD_H;
    for (x, label) in [
        (entry_x, strings::COL_ENTRY),
        (why_x, strings::COL_WHY),
        (used_x, strings::COL_USED_TODAY),
    ] {
        let at = pos2(x, table_top + size::TABLE_HEAD_H / 2.0);
        let cell = draw::paint_text(ui, at, Align2::LEFT_CENTER, label, ty::LABEL, pal.ink_2);
        draw::announce(ui, cell, Role::ColumnHeader, label);
    }
    draw::hline(
        ui,
        rect.left() + 1.0..rect.right() - 1.0,
        head_bottom - 1.0,
        pal.line_2,
    );

    let cell_code = Type {
        size: 11.5,
        ..ty::CODE
    };
    let mut toggled = None;
    // The entries are one stop: J and K move through them, Space takes one in or
    // leaves it out. When a scan ends, focus is on the first entry.
    let entries = rows
        .iter()
        .filter(|row| matches!(row, Row::Entry(_)))
        .count();
    let nav = draw::ListNav::new(ui, ("surface", &proposal.module), entries);
    if focus_first && entries > 0 {
        nav.focus(ui);
    }
    let mut entry_row = 0;
    for (i, row) in rows.iter().enumerate() {
        let top = head_bottom + i as f32 * ROW;
        let mid = top + ROW / 2.0;
        let row_rect = Rect::from_min_max(
            pos2(rect.left() + 1.0, top),
            pos2(rect.right() - 1.0, top + ROW),
        );
        draw::hline(
            ui,
            row_rect.left()..row_rect.right(),
            row_rect.bottom() - 1.0,
            pal.line,
        );
        match row {
            Row::Group { title, count } => {
                let at = pos2(rect.left() + 14.0, mid + 2.0);
                let heading =
                    draw::paint_text(ui, at, Align2::LEFT_CENTER, title, ty::LABEL, pal.ink);
                draw::announce(ui, heading, Role::Heading, title);
                let count_type = Type {
                    size: 11.5,
                    ..ty::BODY
                };
                let at = pos2(right, mid + 2.0);
                draw::paint_text(ui, at, Align2::RIGHT_CENTER, count, count_type, pal.ink_2);
            }
            Row::Text(text) => {
                let galley = draw::galley_elided(ui, text, ty::BODY, pal.ink_2, right - entry_x);
                let text_rect =
                    draw::paint_galley(ui, pos2(entry_x, mid), Align2::LEFT_CENTER, galley);
                draw::announce(ui, text_rect, Role::Label, text);
            }
            Row::Entry(index) => {
                let entry = &proposal.entries[*index];
                let response = ui.interact(row_rect, nav.id(entry_row), nav.sense(entry_row));
                nav.after(ui, entry_row, &response);
                entry_row += 1;
                response.widget_info(|| {
                    WidgetInfo::selected(Role::CheckBox, true, entry.proposed, &entry.name)
                });
                if response.hovered() {
                    ui.painter()
                        .rect_filled(row_rect.shrink2(vec2(0.0, 0.5)), 0, pal.well);
                }
                // Space on a row includes it or leaves it out.
                if response.clicked() {
                    toggled = Some(*index);
                }
                let mark = Rect::from_min_size(
                    pos2(rect.left() + 14.0, (mid - size::CHECKBOX / 2.0).round()),
                    Vec2::splat(size::CHECKBOX),
                );
                draw::checkbox_box(ui, pal, mark, entry.proposed, false, response.hovered());
                let name = draw::galley_elided(
                    ui,
                    &entry.name,
                    cell_code,
                    pal.ink,
                    why_x - 12.0 - entry_x,
                );
                draw::paint_galley(ui, pos2(entry_x, mid), Align2::LEFT_CENTER, name);
                let why = why(entry);
                let why_galley =
                    draw::galley_elided(ui, &why, ty::BODY, pal.ink_2, used_x - 12.0 - why_x);
                let why_rect =
                    draw::paint_galley(ui, pos2(why_x, mid), Align2::LEFT_CENTER, why_galley);
                draw::announce(ui, why_rect, Role::Cell, &why);

                // Who uses it today. A class that is used and is left out carries the wedge.
                let (used, in_use) = used_today(entry);
                let left_out = entry.kind == SurfaceKind::OtherClass && !entry.proposed && in_use;
                let mut x = used_x;
                let (used_type, color) = if left_out {
                    let wedge =
                        Rect::from_min_size(pos2(x, mid - 6.0), Vec2::splat(size::STATE_MARK));
                    draw::mark_warn(ui, wedge, pal.warn);
                    x = wedge.right() + 5.0;
                    (ty::LABEL, pal.warn)
                } else if in_use {
                    (ty::BODY, pal.ink)
                } else {
                    (ty::BODY, pal.ink_2)
                };
                let used_galley = draw::galley_elided(ui, &used, used_type, color, right - x);
                let used_rect =
                    draw::paint_galley(ui, pos2(x, mid), Align2::LEFT_CENTER, used_galley);
                draw::announce(ui, used_rect, Role::Cell, &used);
                if place.keyboard && response.has_focus() {
                    draw::focus_ring(ui, pal, row_rect, radius::STAMP);
                }
            }
        }
    }
    if let Some(index) = toggled {
        proposal.entries[index].proposed = !proposal.entries[index].proposed;
    }

    // The foot: add an entry by hand, scan again, accept.
    let foot = Rect::from_min_max(
        pos2(rect.left() + 1.0, head_bottom + rows.len() as f32 * ROW),
        pos2(rect.right() - 1.0, rect.bottom() - 1.0),
    );
    ui.painter().rect_filled(foot, 0, pal.panel);
    let field_w = 380.0_f32.min(foot.width() * 0.42);
    let field = Rect::from_min_size(
        pos2(foot.left() + 12.0, foot.top() + 8.0),
        vec2(field_w, size::FIELD_H),
    );
    let response = draw::text_field(
        ui,
        pal,
        field,
        strings::ADD_BY_HAND,
        add_text,
        true,
        place.keyboard,
    );
    if focus_add {
        response.request_focus();
    }
    if add_text.is_empty() && !response.has_focus() {
        let at = pos2(field.left() + 9.0, field.center().y);
        draw::paint_text(
            ui,
            at,
            Align2::LEFT_CENTER,
            strings::ADD_BY_HAND,
            ty::BODY,
            pal.ink_3,
        );
        draw::keycap(ui, pal, pos2(field.right() - 8.0, field.center().y), "A");
    }
    // Enter in the field adds the entry, when the module has something of that name.
    if response.lost_focus()
        && ui.input(|input| input.key_pressed(Key::Enter))
        && proposal.add(add_text)
    {
        add_text.clear();
    }
    if let Some(verdict) = &verdict {
        let text = strings::typed_verdict(verdict, &proposal.module);
        let at = pos2(field.left(), field.bottom() + 4.0);
        draw::label(ui, at, Align2::LEFT_TOP, &text, ty::SECONDARY, pal.ink_2);
    }

    let accept_label = strings::accept_entries(proposal.proposed());
    let accept = Button {
        label: &accept_label,
        kind: ButtonKind::Primary,
        keys: Some(&run_keys),
        disabled: true,
        ..Default::default()
    };
    let again = Button {
        label: strings::SCAN_AGAIN,
        ..Default::default()
    };
    let copy = Button {
        label: strings::COPY_AS_TEXT,
        ..Default::default()
    };
    let button_y = foot.top() + 8.0 + (size::FIELD_H - size::CONTROL_H) / 2.0;
    let mut x = foot.right() - 12.0 - accept.width(ui);
    // Accepting needs a place in the package to store the surface.
    let response = accept.show(ui, pal, pos2(x, button_y), place.keyboard);
    draw::tooltip(response, pal, strings::CANNOT_STORE_SURFACE);
    x -= 8.0 + again.width(ui);
    if again
        .show(ui, pal, pos2(x, button_y), place.keyboard)
        .clicked()
    {
        scan_again = true;
    }
    x -= 8.0 + copy.width(ui);
    if copy
        .show(ui, pal, pos2(x, button_y), place.keyboard)
        .clicked()
    {
        action = Some(WorkAction::Copy {
            text: proposal.as_text(),
            said: strings::COPIED_PROPOSAL,
        });
    }
    if scan_again {
        state.start(app, &module.name);
        ui.ctx().request_repaint();
    }
    (rect.bottom(), action)
}

/// One row of the proposal.
enum Row {
    /// The heading of a kind, with how many entries it has.
    Group {
        title: String,
        count: String,
    },
    Entry(usize),
    /// A sentence under an entry, or in the place of the entries of a kind.
    Text(String),
}

fn rows(proposal: &Proposal, inputs: &SurfaceInputs<'_>) -> Vec<Row> {
    let routes: usize = proposal
        .of(SurfaceKind::Route)
        .map(|(_, entry)| entry.routes)
        .sum();
    let routes_title = if proposal.routes_read {
        strings::routes_group(inputs.environment)
    } else {
        crate::strings::ROUTES.to_owned()
    };
    let groups = [
        (SurfaceKind::Contract, strings::CONTRACTS.to_owned(), None),
        (SurfaceKind::Event, strings::EVENTS.to_owned(), None),
        (
            SurfaceKind::OtherClass,
            strings::OTHER_CLASSES.to_owned(),
            None,
        ),
        (SurfaceKind::Route, routes_title, Some(routes)),
        (
            SurfaceKind::Command,
            strings::CONSOLE_COMMANDS.to_owned(),
            None,
        ),
    ];
    let mut rows = Vec::new();
    if proposal.entries.is_empty() {
        rows.push(Row::Text(strings::nothing_to_propose(&proposal.module)));
    }
    for (kind, title, total) in groups {
        let entries: Vec<(usize, &SurfaceEntry)> = proposal.of(kind).collect();
        let no_routes = kind == SurfaceKind::Route && !proposal.routes_read;
        if entries.is_empty() && !no_routes {
            continue;
        }
        let count = if no_routes {
            String::new()
        } else {
            total.unwrap_or(entries.len()).to_string()
        };
        rows.push(Row::Group { title, count });
        if no_routes {
            rows.push(Row::Text(strings::ROUTES_NOT_READ.to_owned()));
        }
        for (index, entry) in entries {
            rows.push(Row::Entry(index));
            if kind == SurfaceKind::OtherClass && !entry.proposed && !entry.used_by.is_empty() {
                rows.push(Row::Text(strings::not_proposed(&entry.used_by)));
            }
        }
    }
    rows
}

/// Why an entry is in the list.
fn why(entry: &SurfaceEntry) -> String {
    match entry.kind {
        SurfaceKind::Contract => strings::an_interface_in(&entry.folder),
        SurfaceKind::Event => strings::AN_EVENT.to_owned(),
        SurfaceKind::OtherClass => strings::in_folder_used_by(&entry.folder, &entry.used_by),
        SurfaceKind::Route => strings::ROUTES_WITH_PREFIX.to_owned(),
        SurfaceKind::Command => strings::A_CONSOLE_COMMAND.to_owned(),
    }
}

/// Who uses an entry today, and whether anything does.
fn used_today(entry: &SurfaceEntry) -> (String, bool) {
    match entry.kind {
        SurfaceKind::Route => (strings::routes_for_clients(entry.routes), true),
        SurfaceKind::Command => match &entry.schedule {
            Some(schedule) => (strings::the_scheduler(schedule), true),
            None => (strings::NOT_USED_OUTSIDE.to_owned(), false),
        },
        _ if entry.used_by.is_empty() => (strings::NOT_USED_OUTSIDE.to_owned(), false),
        _ => (strings::used_by_modules(&entry.used_by), true),
    }
}

/// S5.6: the scan as it reads. A bar for the files, a line per kind of thing
/// looked for. Returns its bottom edge and true when the scan is to stop.
fn scanning(
    ui: &Ui,
    place: &Place<'_>,
    scan: &SurfaceScan,
    inputs: &SurfaceInputs<'_>,
) -> (f32, bool) {
    let pal = place.pal;
    let width = place.width.min(560.0);
    let left = place.left;
    let mut y = place.area.top();

    // The progress bar (C17).
    let (read, total) = (scan.read(), scan.total());
    let share = if total == 0 {
        1.0
    } else {
        read as f32 / total as f32
    };
    let label = crate::strings::reading_files(read, total);
    y = draw::progress(ui, pal, pos2(left, y), width, &label, share) + 12.0;

    // What was found so far. What is known in full carries the check.
    let counts = scan.counts();
    let routes = match &inputs.routes {
        Some(routes) => (
            crate::strings::routes_read_from(routes.len(), inputs.environment),
            true,
        ),
        None => (strings::NOT_READ.to_owned(), false),
    };
    let lines = [
        (
            strings::CLASSES_OTHERS_USE,
            strings::classes_in_files(counts.used_classes, counts.used_files),
            true,
        ),
        (
            strings::CONTRACTS,
            strings::interfaces(counts.contracts),
            false,
        ),
        (strings::EVENTS, strings::events(counts.events), false),
        (crate::strings::ROUTES, routes.0, routes.1),
        (
            strings::CONSOLE_COMMANDS,
            strings::commands(counts.commands),
            false,
        ),
    ];
    let title = strings::scanning(scan.module());
    let head = draw::box_head_extra(ui, width, &title, strings::NOTHING_IS_WRITTEN);
    let rect = Rect::from_min_size(
        pos2(left, y),
        vec2(
            width,
            41.0 + head + lines.len() as f32 * size::LIST_LINE_H + 2.0,
        ),
    );
    let mut line_y = draw::box_head(ui, pal, rect, &title, strings::NOTHING_IS_WRITTEN) - 4.0;
    for (i, (name, value, done)) in lines.iter().enumerate() {
        let mid = line_y + size::LIST_LINE_H / 2.0;
        draw::label(
            ui,
            pos2(rect.left() + 13.0, mid),
            Align2::LEFT_CENTER,
            name,
            ty::BODY,
            pal.ink,
        );
        let (value_type, color) = if *done {
            (
                Type {
                    size: 11.5,
                    ..ty::LABEL
                },
                pal.good,
            )
        } else {
            (ty::BODY, pal.ink_2)
        };
        let at = pos2(rect.right() - 13.0, mid);
        let text = draw::label(ui, at, Align2::RIGHT_CENTER, value, value_type, color);
        if *done {
            let mark = Rect::from_min_size(
                pos2(text.left() - 5.0 - size::STATE_MARK, mid - 6.0),
                Vec2::splat(size::STATE_MARK),
            );
            draw::mark_good(ui, mark, color);
        }
        if i + 1 < lines.len() {
            let line = line_y + size::LIST_LINE_H - 1.0;
            draw::hline(ui, rect.left() + 13.0..rect.right() - 13.0, line, pal.line);
        }
        line_y += size::LIST_LINE_H;
    }
    y = rect.bottom() + 12.0;

    // A scan writes nothing, so stopping needs no confirmation.
    let stop = Button {
        label: strings::STOP_THE_SCAN,
        icon: Some(Icon::Stop),
        keys: Some(crate::strings::KEY_ESC),
        ..Default::default()
    };
    let stopped = stop.show(ui, pal, pos2(left, y), place.keyboard).clicked();
    (y + size::CONTROL_H, stopped)
}
