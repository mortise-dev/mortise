//! The Summary tab (S4.1 to S4.3), as far as files, the coverage report and git give it.

use eframe::egui::{
    Align2, Color32, Rect, Role, ScrollArea, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{AppSnapshot, Check, FindingState, Git, Module, ModuleState, Severity};

use crate::boundary::BoundaryAction;
use crate::draw::{self, Button, StampState};
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
    Uses,
    UsedBy,
    Commits,
    Findings,
}

impl Column {
    const ALL: [Column; 9] = [
        Column::Module,
        Column::Files,
        Column::Lines,
        Column::Tests,
        Column::Coverage,
        Column::Uses,
        Column::UsedBy,
        Column::Commits,
        Column::Findings,
    ];

    fn label(self) -> &'static str {
        match self {
            Column::Module => strings::COL_MODULE,
            Column::Files => strings::COL_FILES,
            Column::Lines => strings::COL_LINES,
            Column::Tests => strings::COL_TESTS,
            Column::Coverage => strings::COL_COVERAGE,
            Column::Uses => strings::COL_USES,
            Column::UsedBy => strings::COL_USED_BY,
            Column::Commits => strings::COL_COMMITS,
            Column::Findings => strings::COL_FINDINGS,
        }
    }

    /// The width of a number column. The module column takes the rest.
    fn width(self) -> f32 {
        match self {
            Column::Module => 0.0,
            Column::Files | Column::Tests => 56.0,
            Column::Lines => 64.0,
            Column::Coverage => 74.0,
            Column::Uses => 44.0,
            Column::UsedBy => 58.0,
            Column::Commits => 116.0,
            Column::Findings => 112.0,
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
                Column::Uses => app.uses(&a.name).count().cmp(&app.uses(&b.name).count()),
                Column::UsedBy => app
                    .used_by(&a.name)
                    .count()
                    .cmp(&app.used_by(&b.name).count()),
                Column::Commits => commits(a).cmp(&commits(b)),
                // Good is the lowest, so that "highest first" starts with the problems.
                Column::Findings => app
                    .findings
                    .state_of(&b.name)
                    .cmp(&app.findings.state_of(&a.name)),
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
) -> Option<BoundaryAction> {
    let mut clicked = None;
    let mut show_findings = false;
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
                            &mut false,
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
                        let findings = &mut show_findings;
                        let (now, sort) = (state.now, state.sort);
                        clicked =
                            all_modules(ui, pal, rect.min, width, app, sort, now, findings, true).1
                    }
                    Some(module) => {
                        one_module(ui, pal, rect.min, width, app, module, state.now, true);
                    }
                }
            });
    });
    if show_findings {
        return Some(BoundaryAction::ShowFindings);
    }
    clicked.map(BoundaryAction::Select)
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
    show_findings: &mut bool,
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
            findings: None,
        },
        Stat {
            key: strings::LINES_OF_PHP,
            value: strings::count(app.lines()),
            known: true,
            tooltip: None,
            findings: None,
        },
        Stat {
            key: strings::TEST_COVERAGE,
            value: coverage.map_or(strings::UNKNOWN.to_owned(), strings::percent),
            known: coverage.is_some(),
            tooltip: app.coverage_report.as_ref().map(|report| {
                strings::coverage_from(&app.path_shown(&report.path), report.written, now)
            }),
            findings: None,
        },
        Stat {
            key: strings::FINDINGS,
            value: String::new(),
            known: true,
            tooltip: None,
            findings: Some((app.findings.problems(), app.findings.warnings())),
        },
    ];
    y = stats(ui, pal, pos2(left, y), inner, &cells, paint) + 12.0;

    // The module that needs attention most, and the way to every finding.
    let all = app.findings.list.len();
    let show = strings::show_all_findings(all);
    let alert = match strings::app_alert(app) {
        Some((module, rest)) => Some((Severity::Problem, module, rest)),
        None => app.findings.list.first().map(|first| {
            (
                Severity::Warning,
                String::new(),
                strings::finding(first, app),
            )
        }),
    };
    if let Some((severity, bold, rest)) = alert {
        let (bottom, clicked) = alert_box(
            ui,
            pal,
            pos2(left, y),
            inner,
            severity,
            &bold,
            &rest,
            Some((&show, "F")),
            paint,
        );
        *show_findings |= clicked;
        y = bottom + 12.0;
    }

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
        let low = |name: &str| {
            app.findings
                .of(name)
                .any(|finding| finding.check == Check::CoverageLimit)
        };
        let bars: Vec<Bar> = covered
            .iter()
            .map(|(name, percent)| Bar {
                label: name,
                share: f32::from(*percent) / 100.0,
                value: strings::percent(*percent),
                low: low(name),
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
    let mut top = origin.y + 14.0;
    // What is wrong with the module, at the top of the view it concerns.
    if let Some((severity, bold, rest)) = strings::module_alert(app, &module.name) {
        let at = pos2(left, top);
        top = alert_box(ui, pal, at, inner, severity, &bold, &rest, None, paint).0 + 12.0;
    }
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
            low: false,
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

    // Right, or under it in a narrow window: what the module uses and what uses it,
    // then the state of the module.
    let side = if two {
        pos2(left + column_w + 14.0, top)
    } else {
        pos2(left, kinds_bottom + 12.0)
    };
    let dependencies_bottom = dependencies(ui, pal, side, column_w, app, module, paint);
    let state_min = match dependencies_bottom {
        Some(bottom) => pos2(side.x, bottom + 12.0),
        None => side,
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
    /// For the Findings cell: how many problems and how many warnings.
    findings: Option<(usize, usize)>,
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
        if let Some((problems, warnings)) = cell.findings {
            // Each state has a mark and a word; colour is only state.
            let mid = strip.top() + 9.0 + 16.0 + 13.0;
            let mut at = x + 12.0;
            let state_type = Type {
                size: 12.0,
                ..ty::LABEL
            };
            let mut state = |ui: &Ui, text: String, color: Color32, kind: u8| {
                let mark = Rect::from_min_size(pos2(at, mid - 6.0), Vec2::splat(size::STATE_MARK));
                match kind {
                    0 => draw::mark_bad(ui, mark, color, pal.surface),
                    1 => draw::mark_warn(ui, mark, color),
                    _ => draw::mark_good(ui, mark, color),
                }
                let word = draw::label(
                    ui,
                    pos2(mark.right() + 5.0, mid),
                    Align2::LEFT_CENTER,
                    &text,
                    state_type,
                    color,
                );
                at = word.right() + 8.0;
            };
            if problems > 0 {
                state(ui, strings::problems(problems), pal.bad, 0);
            }
            if warnings > 0 {
                state(ui, strings::warnings(warnings), pal.warn, 1);
            }
            if problems + warnings == 0 {
                state(ui, strings::GOOD.to_owned(), pal.good, 2);
            }
            continue;
        }
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
    /// Under the limit: the fill and the value take the warning colour.
    low: bool,
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
            if bar.low {
                value_type.with_face(Face::Mono600)
            } else {
                value_type
            },
            if bar.low { pal.warn } else { pal.ink },
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
        ui.painter()
            .rect_filled(fill, 0, if bar.low { pal.warn } else { pal.ink });
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
    let mut edges = [(0.0, 0.0); 9];
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
        let text = if matches!(column, Column::Module | Column::Findings) {
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
            let tip = if matches!(column, Column::Module | Column::Findings) {
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
            (app.uses(&module.name).count().to_string(), pal.ink),
            (app.used_by(&module.name).count().to_string(), pal.ink),
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

        // The finding state: a mark and a word, at the left of its column.
        let (from, _) = edges[8];
        let (word, color, kind) = match (module.state, app.findings.state_of(&module.name)) {
            (ModuleState::Off, _) => (strings::OFF.to_owned(), pal.off, 3),
            (_, FindingState::Problems(n)) => (strings::problems(n), pal.bad, 0),
            (_, FindingState::Warning) => {
                let low = app
                    .findings
                    .of(&module.name)
                    .all(|finding| finding.check == Check::CoverageLimit);
                let word = if low {
                    strings::LOW_COVERAGE
                } else {
                    strings::WARNING
                };
                (word.to_owned(), pal.warn, 1)
            }
            (_, FindingState::Good) => (strings::GOOD.to_owned(), pal.good, 2),
        };
        let mark = Rect::from_min_size(pos2(from, mid - 6.0), Vec2::splat(size::STATE_MARK));
        let behind = if response.hovered() {
            pal.well
        } else {
            pal.surface
        };
        match kind {
            0 => draw::mark_bad(ui, mark, color, behind),
            1 => draw::mark_warn(ui, mark, color),
            2 => draw::mark_good(ui, mark, color),
            _ => draw::mark_off(ui, mark, color),
        }
        draw::paint_text(
            ui,
            pos2(mark.right() + 5.0, mid),
            Align2::LEFT_CENTER,
            &word,
            ty::LABEL,
            color,
        );
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

/// The Dependencies box of a module: one line per module it uses, with the most
/// references first, then the modules that use it. Nothing when it has no links.
/// Returns its bottom edge.
fn dependencies(
    ui: &Ui,
    pal: &Palette,
    min: eframe::egui::Pos2,
    width: f32,
    app: &AppSnapshot,
    module: &Module,
    paint: bool,
) -> Option<f32> {
    let mut uses: Vec<_> = app.uses(&module.name).collect();
    uses.sort_by(|a, b| {
        b.refs
            .len()
            .cmp(&a.refs.len())
            .then_with(|| a.to.cmp(&b.to))
    });
    let used_by: Vec<_> = app.used_by(&module.name).collect();
    if uses.is_empty() && used_by.is_empty() {
        return None;
    }
    let lines = uses.len() + usize::from(!used_by.is_empty());
    let rect = Rect::from_min_size(
        min,
        vec2(width, 41.0 + lines as f32 * size::LIST_LINE_H + 8.0),
    );
    if !paint {
        return Some(rect.bottom());
    }
    let note = strings::out_in(uses.len(), used_by.len());
    let mut y = draw::box_head(ui, pal, rect, strings::DEPENDENCIES, &note);
    let mono = Type {
        size: 11.5,
        ..ty::CODE
    };
    for (i, link) in uses.iter().enumerate() {
        let mid = y + size::LIST_LINE_H / 2.0;
        let text = strings::a_uses_b(strings::USES, &link.to).replacen(" uses ", " ", 1);
        let name = draw::label(
            ui,
            pos2(rect.left() + 13.0, mid),
            Align2::LEFT_CENTER,
            &text,
            ty::BODY,
            pal.ink,
        );
        if let Some(used) = app.module(&link.to) {
            let state = StampState {
                selected: false,
                off: used.state == ModuleState::Off,
            };
            draw::stamp(
                ui,
                pal,
                pos2(name.right() + 7.0, (mid - size::STAMP_H / 2.0).round()),
                &used.code,
                state,
            );
        }
        let problem = link.is_problem(&app.rules);
        let refs = strings::refs_state(link, &app.rules);
        let (value_type, color) = if problem {
            (ty::LABEL, pal.bad)
        } else {
            (mono, pal.ink_2)
        };
        let value = draw::label(
            ui,
            pos2(rect.right() - 13.0, mid),
            Align2::RIGHT_CENTER,
            &refs,
            value_type,
            color,
        );
        if problem {
            let mark = Rect::from_min_size(
                pos2(
                    value.left() - 5.0 - size::STATE_MARK,
                    mid - size::STATE_MARK / 2.0,
                ),
                Vec2::splat(size::STATE_MARK),
            );
            draw::mark_bad(ui, mark, pal.bad, pal.surface);
        }
        if i + 1 < lines {
            draw::hline(
                ui,
                rect.left() + 13.0..rect.right() - 13.0,
                y + size::LIST_LINE_H - 1.0,
                pal.line,
            );
        }
        y += size::LIST_LINE_H;
    }
    if !used_by.is_empty() {
        let mid = y + size::LIST_LINE_H / 2.0;
        let names: Vec<&str> = used_by.iter().map(|link| link.from.as_str()).collect();
        let counts: Vec<String> = used_by
            .iter()
            .map(|link| link.refs.len().to_string())
            .collect();
        let total: usize = used_by.iter().map(|link| link.refs.len()).sum();
        let plural = if total == 1 { "ref" } else { "refs" };
        let text = format!("{} {}", strings::USED_BY, names.join(", "));
        draw::label(
            ui,
            pos2(rect.left() + 13.0, mid),
            Align2::LEFT_CENTER,
            &text,
            ty::BODY,
            pal.ink,
        );
        let value = format!("{} {plural}", counts.join(", "));
        draw::label(
            ui,
            pos2(rect.right() - 13.0, mid),
            Align2::RIGHT_CENTER,
            &value,
            mono,
            pal.ink_2,
        );
    }
    Some(rect.bottom())
}

/// An alert (C19) at the top of the view it concerns. A problem has the bad-soft
/// fill, the bad outline and the problem mark; a warning has the surface fill, the
/// warn outline and the wedge. It may carry one small button on the right.
/// Returns its bottom edge and whether the button was pressed.
#[expect(clippy::too_many_arguments)]
fn alert_box(
    ui: &Ui,
    pal: &Palette,
    min: eframe::egui::Pos2,
    width: f32,
    severity: Severity,
    bold: &str,
    rest: &str,
    button: Option<(&str, &str)>,
    paint: bool,
) -> (f32, bool) {
    let button = button.map(|(label, keys)| Button {
        label,
        small: true,
        keys: Some(keys),
        ..Default::default()
    });
    let button_w = button
        .as_ref()
        .map_or(0.0, |button| button.width(ui) + 12.0);
    let text_x = min.x + 13.0 + size::STATE_MARK + 9.0;
    let text_w = min.x + width - 13.0 - button_w - text_x;
    let parts = [
        (bold, ty::BODY.with_face(Face::Sans700), pal.ink),
        (rest, ty::BODY, pal.ink),
    ];
    let galley = draw::rich_galley(ui, &parts, text_w);
    let rect = Rect::from_min_size(min, vec2(width, (galley.size().y + 20.0).max(40.0)));
    if !paint {
        return (rect.bottom(), false);
    }
    let (fill, outline) = match severity {
        Severity::Problem => (pal.bad_soft, pal.bad),
        Severity::Warning => (pal.surface, pal.warn),
    };
    draw::outlined(ui, rect, radius::CONTROL, fill, outline);
    let mark = Rect::from_min_size(rect.min + vec2(13.0, 13.0), Vec2::splat(size::STATE_MARK));
    match severity {
        Severity::Problem => draw::mark_bad(ui, mark, pal.bad, fill),
        Severity::Warning => draw::mark_warn(ui, mark, pal.warn),
    }
    let text = draw::paint_galley(
        ui,
        pos2(text_x, rect.top() + 10.0),
        Align2::LEFT_TOP,
        galley,
    );
    draw::announce(ui, text, Role::Label, &format!("{bold}{rest}"));
    let clicked = button.is_some_and(|button| {
        let at = pos2(rect.right() - 13.0 - button.width(ui), rect.top() + 9.0);
        button.show(ui, pal, at, false).clicked()
    });
    (rect.bottom(), clicked)
}
