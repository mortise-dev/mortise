//! The Summary tab (S4.1 to S4.3), as far as files, the coverage report and git give it.

use eframe::egui::{
    Align2, Color32, Rect, Role, ScrollArea, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{AppSnapshot, Git, Module, ModuleState};

use crate::draw::{self, StampState};
use crate::strings;
use crate::theme::{Face, Palette, Type, radius, size, ty};

/// A column of the "By module" table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Module,
    Files,
    Lines,
    Tests,
    Coverage,
    Commits,
}

impl Column {
    const ALL: [Column; 6] = [
        Column::Module,
        Column::Files,
        Column::Lines,
        Column::Tests,
        Column::Coverage,
        Column::Commits,
    ];

    fn label(self) -> &'static str {
        match self {
            Column::Module => strings::COL_MODULE,
            Column::Files => strings::COL_FILES,
            Column::Lines => strings::COL_LINES,
            Column::Tests => strings::COL_TESTS,
            Column::Coverage => strings::COL_COVERAGE,
            Column::Commits => strings::COL_COMMITS,
        }
    }

    /// The width of a number column. The module column takes the rest.
    fn width(self) -> f32 {
        match self {
            Column::Module => 0.0,
            Column::Files | Column::Tests => 56.0,
            Column::Lines => 64.0,
            Column::Coverage => 74.0,
            Column::Commits => 116.0,
        }
    }
}

/// The order of the "By module" table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub column: Column,
    pub ascending: bool,
}

impl Default for Sort {
    fn default() -> Self {
        Self {
            column: Column::Module,
            ascending: true,
        }
    }
}

impl Sort {
    /// A click on a header: sort by it, or reverse when it is the sorted column.
    /// Numbers start with the highest, names with A.
    fn toggle(&mut self, column: Column) {
        if self.column == column {
            self.ascending = !self.ascending;
        } else {
            *self = Self {
                column,
                ascending: column == Column::Module,
            };
        }
    }

    fn apply(self, app: &AppSnapshot) -> Vec<&Module> {
        let mut modules: Vec<&Module> = app.modules.iter().collect();
        modules.sort_by(|a, b| {
            let order = match self.column {
                Column::Module => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                Column::Files => a.files.cmp(&b.files),
                Column::Lines => a.lines.cmp(&b.lines),
                Column::Tests => a.tests.cmp(&b.tests),
                Column::Coverage => percent(a).cmp(&percent(b)),
                Column::Commits => commits(a).cmp(&commits(b)),
            };
            if self.ascending {
                order
            } else {
                order.reverse()
            }
        });
        modules
    }
}

fn percent(module: &Module) -> Option<u8> {
    module.coverage.and_then(|coverage| coverage.percent())
}

fn commits(module: &Module) -> Option<u32> {
    module.churn.map(|churn| churn.commits)
}

pub struct SummaryState<'a> {
    pub sort: &'a mut Sort,
    /// Seconds since 1970, for "2 days ago".
    pub now: i64,
}

/// Paints the Summary of the whole app or of one module into `body`, which scrolls.
/// Returns the name of a module whose row was clicked.
pub fn summary(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    app: &AppSnapshot,
    module: Option<&Module>,
    state: SummaryState<'_>,
) -> Option<String> {
    let mut clicked = None;
    let salt = module.map_or("all", |module| module.name.as_str());
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        ScrollArea::vertical()
            .id_salt(("summary", salt))
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let width = ui.available_width();
                // Painted twice: once without ink to learn the height, once for real.
                let height = match module {
                    None => {
                        all_modules(
                            ui,
                            pal,
                            pos2(0.0, 0.0),
                            width,
                            app,
                            &mut state.sort.clone(),
                            state.now,
                            false,
                        )
                        .0
                    }
                    Some(module) => one_module(
                        ui,
                        pal,
                        pos2(0.0, 0.0),
                        width,
                        app,
                        module,
                        state.now,
                        false,
                    ),
                };
                let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
                match module {
                    None => {
                        clicked =
                            all_modules(ui, pal, rect.min, width, app, state.sort, state.now, true)
                                .1
                    }
                    Some(module) => {
                        one_module(ui, pal, rect.min, width, app, module, state.now, true);
                    }
                }
            });
    });
    clicked
}

/// The Summary of "All modules". Returns its height and a module that was clicked.
/// With `paint` false nothing is drawn and nothing can be clicked.
#[expect(clippy::too_many_arguments)]
fn all_modules(
    ui: &Ui,
    pal: &Palette,
    origin: eframe::egui::Pos2,
    width: f32,
    app: &AppSnapshot,
    sort: &mut Sort,
    now: i64,
    paint: bool,
) -> (f32, Option<String>) {
    let left = origin.x + 16.0;
    let inner = width - 32.0;
    let mut y = origin.y + 14.0;
    let mut clicked = None;

    // A note when part of config/modules.php could not be read without PHP.
    if let Some(note) = strings::assumed(app) {
        y = note_alert(ui, pal, pos2(left, y), inner, &note, paint) + 12.0;
    }

    // The stats strip. Findings and Runtime join it in later milestones.
    let on = if app.layout.uses_file_activator() {
        strings::of(app.modules_on(), app.modules.len())
    } else {
        strings::STATE_UNKNOWN.to_owned()
    };
    let coverage = app.coverage().and_then(|coverage| coverage.percent());
    let cells = [
        Stat {
            key: strings::MODULES_ON,
            value: on,
            known: true,
            tooltip: None,
        },
        Stat {
            key: strings::LINES_OF_PHP,
            value: strings::count(app.lines()),
            known: true,
            tooltip: None,
        },
        Stat {
            key: strings::TEST_COVERAGE,
            value: coverage.map_or(strings::UNKNOWN.to_owned(), strings::percent),
            known: coverage.is_some(),
            tooltip: app.coverage_report.as_ref().map(|report| {
                strings::coverage_from(&app.path_shown(&report.path), report.written, now)
            }),
        },
    ];
    y = stats(ui, pal, pos2(left, y), inner, &cells, paint) + 12.0;

    // What cannot be read from files, in a sentence.
    let sentence = if !app.layout.uses_file_activator() {
        Some(strings::state_not_in_a_file(app))
    } else if app.modules.is_empty() {
        Some(strings::no_modules(app))
    } else {
        None
    };
    if let Some(sentence) = sentence {
        let text = draw::galley_wrapped(ui, &sentence, ty::BODY, pal.ink_2, inner.min(560.0));
        let text_h = text.size().y;
        if paint {
            let rect = draw::paint_galley(ui, pos2(left, y), Align2::LEFT_TOP, text);
            draw::announce(ui, rect, Role::Label, &sentence);
        }
        y += text_h + 12.0;
    }
    if app.modules.is_empty() {
        return (y - origin.y + 4.0, None);
    }

    // Coverage by module, best first. Two columns when there is room.
    let column_w = if inner >= 720.0 {
        ((inner - 14.0) / 2.0).floor()
    } else {
        inner
    };
    if coverage.is_some() {
        let mut covered: Vec<(&str, u8)> = app
            .modules
            .iter()
            .filter_map(|module| Some((module.name.as_str(), percent(module)?)))
            .collect();
        covered.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        let bars: Vec<Bar> = covered
            .iter()
            .map(|(name, percent)| Bar {
                label: name,
                share: f32::from(*percent) / 100.0,
                value: strings::percent(*percent),
            })
            .collect();
        let note = strings::tests(app.tests());
        y = bar_box(
            ui,
            pal,
            pos2(left, y),
            column_w,
            strings::COVERAGE_BY_MODULE,
            &note,
            &bars,
            paint,
        ) + 12.0;
    }

    // By module: a table that sorts.
    let modules = sort.apply(app);
    let table_h = 41.0 - 6.0 + size::TABLE_HEAD_H + modules.len() as f32 * size::TABLE_ROW_H + 1.0;
    let table = Rect::from_min_size(pos2(left, y), vec2(inner, table_h));
    if paint {
        let note = strings::sorted_by(sort.column.label(), sort.ascending);
        let top = draw::box_head(ui, pal, table, strings::BY_MODULE, &note) - 6.0;
        clicked = by_module(
            ui,
            pal,
            Rect::from_min_max(pos2(table.left(), top), table.max),
            &modules,
            sort,
            app,
        );
    }
    y = table.bottom() + 16.0;

    (y - origin.y, clicked)
}

/// The Summary of one module. Returns its height.
#[expect(clippy::too_many_arguments)]
fn one_module(
    ui: &Ui,
    pal: &Palette,
    origin: eframe::egui::Pos2,
    width: f32,
    app: &AppSnapshot,
    module: &Module,
    now: i64,
    paint: bool,
) -> f32 {
    let left = origin.x + 16.0;
    let inner = width - 32.0;
    let top = origin.y + 14.0;
    let two = inner >= 720.0;
    let column_w = if two {
        ((inner - 14.0) / 2.0).floor()
    } else {
        inner
    };

    // Left: the files by kind.
    let most = module.kinds.first().map_or(1, |kind| kind.files.max(1)) as f32;
    let bars: Vec<Bar> = module
        .kinds
        .iter()
        .map(|kind| Bar {
            label: &kind.kind,
            share: kind.files as f32 / most,
            value: strings::count(u64::from(kind.files)),
        })
        .collect();
    let note = strings::files_and_lines(module.files, module.lines);
    let kinds_bottom = bar_box(
        ui,
        pal,
        pos2(left, top),
        column_w,
        strings::FILES_BY_KIND,
        &note,
        &bars,
        paint,
    );

    // Right, or under it in a narrow window: the state of the module.
    let state_min = if two {
        pos2(left + column_w + 14.0, top)
    } else {
        pos2(left, kinds_bottom + 12.0)
    };
    let mut lines: Vec<(&str, String, Color32, StateMark)> = Vec::new();
    let (word, color, mark) = match module.state {
        ModuleState::On => (strings::ON, pal.good, StateMark::Good),
        ModuleState::Off => (strings::OFF, pal.off, StateMark::Off),
        ModuleState::Unknown => (strings::STATE_UNKNOWN, pal.ink_2, StateMark::None),
    };
    lines.push((strings::MODULE_STATE, word.to_owned(), color, mark));
    lines.push((
        strings::COL_TESTS,
        strings::count(u64::from(module.tests)),
        pal.ink,
        StateMark::None,
    ));
    let coverage = percent(module).map_or(strings::UNKNOWN.to_owned(), strings::percent);
    lines.push((strings::TEST_COVERAGE, coverage, pal.ink, StateMark::None));
    // Which migrations ran needs PHP. Until then: how many files there are.
    let migrations = module
        .kinds
        .iter()
        .find(|kind| kind.kind == "Migrations")
        .map_or(0, |kind| kind.files);
    lines.push((
        strings::MIGRATIONS,
        strings::files(u64::from(migrations)),
        pal.ink,
        StateMark::None,
    ));
    let churn = match (&app.git, module.churn) {
        (Git::Repository { .. }, Some(churn)) => {
            strings::churn(churn.commits, churn.last_commit, now)
        }
        (Git::NoGit, _) => strings::GIT_NOT_FOUND.to_owned(),
        _ => strings::NOT_A_REPOSITORY.to_owned(),
    };
    lines.push((strings::GIT_CHURN, churn, pal.ink, StateMark::None));

    let state_h = 4.0 + lines.len() as f32 * size::LIST_LINE_H + 4.0;
    let state_rect = Rect::from_min_size(state_min, vec2(column_w, state_h));
    if paint {
        draw::outlined(
            ui,
            state_rect,
            radius::CONTROL,
            Color32::TRANSPARENT,
            pal.line_2,
        );
        let mut y = state_rect.top() + 4.0;
        for (i, (name, value, color, mark)) in lines.iter().enumerate() {
            let mid = y + size::LIST_LINE_H / 2.0;
            draw::label(
                ui,
                pos2(state_rect.left() + 13.0, mid),
                Align2::LEFT_CENTER,
                name,
                ty::BODY,
                pal.ink,
            );
            let value_type = if *mark == StateMark::None {
                ty::BODY
            } else {
                ty::LABEL
            };
            let text = draw::label(
                ui,
                pos2(state_rect.right() - 13.0, mid),
                Align2::RIGHT_CENTER,
                value,
                value_type,
                *color,
            );
            let at = Rect::from_min_size(
                pos2(
                    text.left() - 5.0 - size::STATE_MARK,
                    mid - size::STATE_MARK / 2.0,
                ),
                Vec2::splat(size::STATE_MARK),
            );
            match mark {
                StateMark::Good => draw::mark_good(ui, at, *color),
                StateMark::Off => draw::mark_off(ui, at, *color),
                StateMark::None => {}
            }
            if i + 1 < lines.len() {
                draw::hline(
                    ui,
                    state_rect.left() + 13.0..state_rect.right() - 13.0,
                    y + size::LIST_LINE_H - 1.0,
                    pal.line,
                );
            }
            y += size::LIST_LINE_H;
        }
    }

    let mut bottom = kinds_bottom.max(state_rect.bottom());
    if !app.layout.uses_file_activator() {
        let sentence = strings::state_not_in_a_file(app);
        let text = draw::galley_wrapped(ui, &sentence, ty::BODY, pal.ink_2, inner.min(560.0));
        let text_h = text.size().y;
        if paint {
            let rect = draw::paint_galley(ui, pos2(left, bottom + 12.0), Align2::LEFT_TOP, text);
            draw::announce(ui, rect, Role::Label, &sentence);
        }
        bottom += 12.0 + text_h;
    }
    bottom + 16.0 - origin.y
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateMark {
    None,
    Good,
    Off,
}

struct Stat {
    key: &'static str,
    value: String,
    /// False for a value that is a word such as "unknown", not a figure.
    known: bool,
    tooltip: Option<String>,
}

/// The stats strip: one box, a cell per figure, hairlines between. Each cell keeps
/// the width it has when all five are there. Returns its bottom edge.
fn stats(
    ui: &Ui,
    pal: &Palette,
    min: eframe::egui::Pos2,
    width: f32,
    cells: &[Stat],
    paint: bool,
) -> f32 {
    let cell_w = (width / 5.0)
        .floor()
        .max(150.0)
        .min(width / cells.len() as f32);
    let strip = Rect::from_min_size(min, vec2(cell_w * cells.len() as f32, 60.0));
    if !paint {
        return strip.bottom();
    }
    draw::outlined(ui, strip, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
    for (i, cell) in cells.iter().enumerate() {
        let x = strip.left() + i as f32 * cell_w;
        if i > 0 {
            draw::vline(ui, x, strip.top() + 1.0..strip.bottom() - 1.0, pal.line);
        }
        draw::label(
            ui,
            pos2(x + 12.0, strip.top() + 9.0),
            Align2::LEFT_TOP,
            cell.key,
            ty::SECONDARY,
            pal.ink_2,
        );
        let (value_type, color) = if cell.known {
            (ty::FIGURE, pal.ink)
        } else {
            (ty::BODY, pal.ink_2)
        };
        let value = draw::label(
            ui,
            pos2(x + 12.0, strip.top() + 9.0 + 16.0 + 13.0),
            Align2::LEFT_CENTER,
            &cell.value,
            value_type,
            color,
        );
        if let Some(tooltip) = &cell.tooltip {
            let hover = ui.interact(
                value,
                ui.scope_id().with(("stat", cell.key)),
                Sense::hover(),
            );
            draw::tooltip(hover, pal, tooltip);
        }
    }
    strip.bottom()
}

struct Bar<'a> {
    label: &'a str,
    /// How much of the track is filled, 0 to 1.
    share: f32,
    value: String,
}

/// A box of bars: a name, a 6 px track with an ink fill, a value. Returns its bottom edge.
#[expect(clippy::too_many_arguments)]
fn bar_box(
    ui: &Ui,
    pal: &Palette,
    min: eframe::egui::Pos2,
    width: f32,
    title: &str,
    note: &str,
    bars: &[Bar<'_>],
    paint: bool,
) -> f32 {
    const ROW: f32 = 21.0;
    let rect = Rect::from_min_size(min, vec2(width, 41.0 + bars.len() as f32 * ROW + 8.0));
    if !paint {
        return rect.bottom();
    }
    let mut y = draw::box_head(ui, pal, rect, title, note);
    let label_type = Type {
        size: 12.0,
        ..ty::BODY
    };
    let value_type = Type {
        size: 11.5,
        ..ty::CODE
    };
    let labels_w = bars
        .iter()
        .map(|bar| {
            draw::galley(ui, bar.label, label_type, Color32::PLACEHOLDER)
                .size()
                .x
        })
        .fold(88.0_f32, f32::max)
        .min(140.0)
        .ceil();
    for bar in bars {
        let mid = y + ROW / 2.0;
        let label = draw::galley_elided(ui, bar.label, label_type, pal.ink, labels_w);
        let label_rect = draw::paint_galley(
            ui,
            pos2(rect.left() + 13.0, mid),
            Align2::LEFT_CENTER,
            label,
        );
        draw::announce(ui, label_rect, Role::Label, bar.label);
        draw::label(
            ui,
            pos2(rect.right() - 13.0, mid),
            Align2::RIGHT_CENTER,
            &bar.value,
            value_type,
            pal.ink,
        );

        let track = Rect::from_min_max(
            pos2(
                rect.left() + 13.0 + labels_w + 10.0,
                (mid - size::BAR_H / 2.0).round(),
            ),
            pos2(
                rect.right() - 13.0 - 34.0 - 10.0,
                (mid - size::BAR_H / 2.0).round() + size::BAR_H,
            ),
        );
        ui.painter().rect_filled(track, 0, pal.well);
        let fill = Rect::from_min_size(
            track.min,
            vec2(
                (track.width() * bar.share.clamp(0.0, 1.0)).round(),
                size::BAR_H,
            ),
        );
        ui.painter().rect_filled(fill, 0, pal.ink);
        y += ROW;
    }
    rect.bottom()
}

/// The table of modules (C8): a header whose cells sort, then a row per module.
fn by_module(
    ui: &Ui,
    pal: &Palette,
    rect: Rect,
    modules: &[&Module],
    sort: &mut Sort,
    app: &AppSnapshot,
) -> Option<String> {
    let mut clicked = None;
    let left = rect.left() + 14.0;
    let right = rect.right() - 10.0;

    // Columns from the right edge: numbers have fixed widths, the module takes the rest.
    let mut edges = [(0.0, 0.0); 6];
    let mut x = right;
    for (i, column) in Column::ALL.iter().enumerate().rev() {
        if *column == Column::Module {
            edges[i] = (left, x);
        } else {
            edges[i] = (x - column.width(), x);
            x -= column.width() + 12.0;
        }
    }

    // Header: 26 px, 11.5 px weight 600 in ink-2, line-2 underneath. A cell is a button.
    let head_bottom = rect.top() + size::TABLE_HEAD_H;
    draw::hline(
        ui,
        rect.left() + 1.0..rect.right() - 1.0,
        head_bottom - 1.0,
        pal.line_2,
    );
    for (i, column) in Column::ALL.iter().enumerate() {
        let (from, to) = edges[i];
        let cell = Rect::from_min_max(pos2(from, rect.top()), pos2(to, head_bottom - 1.0));
        let response = ui.interact(
            cell,
            ui.scope_id().with(("by-module-head", i)),
            Sense::click(),
        );
        response.widget_info(|| WidgetInfo::labeled(Role::ColumnHeader, true, column.label()));
        if response.clicked() {
            sort.toggle(*column);
        }
        let sorted = sort.column == *column;
        let color = if sorted || response.hovered() {
            pal.ink
        } else {
            pal.ink_2
        };
        let mid = cell.center().y;
        let text = if *column == Column::Module {
            draw::paint_text(
                ui,
                pos2(from, mid),
                Align2::LEFT_CENTER,
                column.label(),
                ty::LABEL,
                color,
            )
        } else {
            draw::paint_text(
                ui,
                pos2(to, mid),
                Align2::RIGHT_CENTER,
                column.label(),
                ty::LABEL,
                color,
            )
        };
        if sorted {
            // A chevron: down for A to Z and low to high, up for the reverse.
            let tip = if *column == Column::Module {
                text.right() + 9.0
            } else {
                text.left() - 9.0
            };
            let (a, b) = if sort.ascending {
                (-1.5, 1.5)
            } else {
                (1.5, -1.5)
            };
            let points = vec![
                pos2(tip - 3.5, mid + a),
                pos2(tip, mid + b),
                pos2(tip + 3.5, mid + a),
            ];
            ui.painter().add(eframe::egui::Shape::line(
                points,
                eframe::egui::Stroke::new(1.5, pal.ink),
            ));
        }
    }

    // Rows: 28 px, a hairline between them. Numbers are right aligned.
    for (row, module) in modules.iter().enumerate() {
        let top = head_bottom + row as f32 * size::TABLE_ROW_H;
        let row_rect = Rect::from_min_max(
            pos2(rect.left() + 1.0, top),
            pos2(rect.right() - 1.0, top + size::TABLE_ROW_H),
        );
        let response = ui.interact(
            row_rect,
            ui.scope_id().with(("by-module-row", &module.name)),
            Sense::click(),
        );
        response.widget_info(|| WidgetInfo::labeled(Role::Row, true, &module.name));
        if response.hovered() {
            ui.painter().rect_filled(row_rect, 0, pal.well);
        }
        if response.clicked() {
            clicked = Some(module.name.clone());
        }
        if row + 1 < modules.len() {
            draw::hline(
                ui,
                row_rect.left()..row_rect.right(),
                row_rect.bottom() - 1.0,
                pal.line,
            );
        }

        let mid = row_rect.center().y;
        let off = module.state == ModuleState::Off;
        let stamp_min = pos2(left, (mid - size::STAMP_H / 2.0).round());
        let stamp = draw::stamp(
            ui,
            pal,
            stamp_min,
            &module.code,
            StampState {
                selected: false,
                off,
            },
        );
        let name_type = ty::BODY.with_face(Face::Sans600);
        let name_w = (edges[0].1 - stamp.right() - 8.0 - 12.0).max(0.0);
        let name = draw::galley_elided(
            ui,
            &module.name,
            name_type,
            if off { pal.ink_2 } else { pal.ink },
            name_w,
        );
        draw::paint_galley(
            ui,
            pos2(stamp.right() + 8.0, mid),
            Align2::LEFT_CENTER,
            name,
        );

        let unknown = (strings::UNKNOWN.to_owned(), pal.ink_3);
        let cells = [
            (strings::count(module.files), pal.ink),
            (strings::count(module.lines), pal.ink),
            (strings::count(u64::from(module.tests)), pal.ink),
            percent(module).map_or(unknown.clone(), |percent| {
                (strings::percent(percent), pal.ink)
            }),
            match (&app.git, commits(module)) {
                (Git::Repository { .. }, Some(commits)) => {
                    (strings::count(u64::from(commits)), pal.ink)
                }
                _ => unknown,
            },
        ];
        for (i, (text, color)) in cells.iter().enumerate() {
            let (_, to) = edges[i + 1];
            draw::paint_text(
                ui,
                pos2(to, mid),
                Align2::RIGHT_CENTER,
                text,
                ty::BODY,
                *color,
            );
        }
    }
    clicked
}

/// A note (C19): panel fill, line-2 outline, no mark. Returns its bottom edge.
fn note_alert(
    ui: &Ui,
    pal: &Palette,
    min: eframe::egui::Pos2,
    width: f32,
    text: &str,
    paint: bool,
) -> f32 {
    let galley = draw::galley_wrapped(ui, text, ty::BODY, pal.ink, width - 24.0);
    let rect = Rect::from_min_size(min, vec2(width, galley.size().y + 18.0));
    if paint {
        draw::outlined(ui, rect, radius::CONTROL, pal.panel, pal.line_2);
        let text_rect =
            draw::paint_galley(ui, rect.min + vec2(12.0, 9.0), Align2::LEFT_TOP, galley);
        draw::announce(ui, text_rect, Role::Label, text);
    }
    rect.bottom()
}
