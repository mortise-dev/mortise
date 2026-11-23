//! The Summary tab (S4.1 to S4.3), as far as files, the coverage report and git give it.

use std::sync::Arc;

use eframe::egui::{
    Align2, Color32, Galley, Rect, Role, ScrollArea, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2,
    vec2,
};
use mortise_core::{
    AppSnapshot, Check, FindingState, Git, Module, ModuleState, routes_by_module, routes_of,
    tasks_of,
};

use crate::app::workspace::WorkAction;
use crate::runtime::view::{self as runtime_view, Brief, RuntimeAction, RuntimeView};
use crate::summary::strings;
use crate::ui::draw::{self, StampState};
use crate::ui::theme::{Face, Palette, Type, radius, size, ty};

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
            Column::Module => crate::strings::COL_MODULE,
            Column::Files => crate::strings::COL_FILES,
            Column::Lines => strings::COL_LINES,
            Column::Tests => strings::COL_TESTS,
            Column::Coverage => strings::COL_COVERAGE,
            Column::Uses => crate::strings::COL_USES,
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
    /// What the running app reports, and whether one is connected.
    pub runtime: &'a RuntimeView<'a>,
    /// True while the keyboard is in use: focus rings are drawn.
    pub keyboard: bool,
}

/// What the parts of a Summary need besides the app, and what was asked for in them.
struct Shared<'a> {
    /// The display name of the app.
    name: &'a str,
    now: i64,
    runtime: &'a RuntimeView<'a>,
    keyboard: bool,
    show_findings: bool,
    asked: Option<RuntimeAction>,
}

/// Paints the Summary of the whole app or of one module into `body`, which scrolls.
/// Returns the name of a module whose row was clicked.
pub fn summary(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    app: &AppSnapshot,
    name: &str,
    module: Option<&Module>,
    state: SummaryState<'_>,
) -> Option<WorkAction> {
    let mut clicked = None;
    let mut shared = Shared {
        name,
        now: state.now,
        runtime: state.runtime,
        keyboard: state.keyboard,
        show_findings: false,
        asked: None,
    };
    let salt = module.map_or("all", |module| module.name.as_str());
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        ScrollArea::vertical()
            .id_salt(("summary", salt))
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let width = ui.available_width();
                let origin = pos2(0.0, 0.0);
                // Painted twice: once without ink to learn the height, once for real.
                let height = match module {
                    None => {
                        let sort = &mut state.sort.clone();
                        all_modules(ui, pal, origin, width, app, sort, &mut shared, false).0
                    }
                    Some(module) => {
                        one_module(ui, pal, origin, width, app, module, &mut shared, false)
                    }
                };
                let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
                match module {
                    None => {
                        let sort = state.sort;
                        clicked =
                            all_modules(ui, pal, rect.min, width, app, sort, &mut shared, true).1
                    }
                    Some(module) => {
                        one_module(ui, pal, rect.min, width, app, module, &mut shared, true);
                    }
                }
            });
    });
    if let Some(asked) = shared.asked {
        return Some(WorkAction::Runtime(asked));
    }
    if shared.show_findings {
        return Some(WorkAction::ShowFindings);
    }
    clicked.map(WorkAction::Select)
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
    shared: &mut Shared<'_>,
    paint: bool,
) -> (f32, Option<String>) {
    let now = shared.now;
    let left = origin.x + 16.0;
    let inner = width - 32.0;
    let mut y = origin.y + 14.0;
    let mut clicked = None;

    // A note when part of config/modules.php could not be read without PHP.
    if let Some(note) = strings::assumed(app, shared.name) {
        y = note_alert(ui, pal, pos2(left, y), inner, &note, paint) + 12.0;
    }

    // The stats strip. Runtime joins it when an environment answered.
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
            value: crate::strings::count(app.lines()),
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
            key: crate::strings::FINDINGS,
            value: String::new(),
            known: true,
            tooltip: None,
            findings: Some((app.findings.problems(), app.findings.warnings())),
        },
    ];
    let runtime = shared.runtime;
    let environment = runtime.environment().to_owned();
    let runtime_cell = runtime.brief().map(|brief| (brief, environment.as_str()));
    y = stats(ui, pal, pos2(left, y), inner, &cells, runtime_cell, paint) + 12.0;

    // The module that needs attention most, and the way to every finding.
    let all = app.findings.list.len();
    let show = strings::show_all_findings(all);
    if let Some((severity, bold, rest)) = strings::app_alert(app, shared.name) {
        let (bottom, clicked) = draw::alert_box(
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
        shared.show_findings |= clicked;
        y = bottom + 12.0;
    }

    // What cannot be read from files, in a sentence.
    let sentence = if !app.layout.uses_file_activator() {
        Some(strings::state_not_in_a_file(app, shared.name))
    } else if app.modules.is_empty() {
        Some(crate::strings::no_modules(app, shared.name))
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

    // Coverage by module, best first. Two columns when there is room: coverage and
    // routes at the left, the runtime box at the right.
    let two = inner >= 720.0;
    let column_w = if two {
        ((inner - 14.0) / 2.0).floor()
    } else {
        inner
    };
    let columns_top = y;
    let mut left_bottom = None;
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
        let note = crate::strings::tests(app.tests());
        let title = strings::COVERAGE_BY_MODULE;
        let at = pos2(left, y);
        y = bar_box(ui, pal, at, column_w, title, &note, &bars, None, paint) + 12.0;
        left_bottom = Some(y);
    }

    // Routes by module, from the mappings of the running app. A route is counted for
    // a module when its action is a class of the module.
    if let Some(answered) = runtime.runtime() {
        let at = pos2(left, y);
        y = match &answered.routes {
            Some(routes) => {
                let (by_module, outside) = routes_by_module(app, routes);
                let most = by_module.first().map_or(1, |(_, count)| (*count).max(1)) as f32;
                let bars: Vec<Bar> = by_module
                    .iter()
                    .map(|(name, count)| Bar {
                        label: name,
                        share: *count as f32 / most,
                        value: crate::strings::count(*count as u64),
                        low: false,
                    })
                    .collect();
                let note = crate::strings::routes_read_from(routes.len(), &environment);
                let footer = Some((
                    crate::strings::NOT_IN_A_MODULE,
                    crate::strings::count(outside as u64),
                ));
                let title = strings::ROUTES_BY_MODULE;
                bar_box(ui, pal, at, column_w, title, &note, &bars, footer, paint)
            }
            None => {
                let text = strings::endpoint_not_exposed("mappings", &environment);
                let title = strings::ROUTES_BY_MODULE;
                runtime_view::message_box(ui, pal, at, column_w, title, &text, paint)
            }
        } + 12.0;
        left_bottom = Some(y);
    }

    // The runtime box: beside the two boxes above, or under them in a narrow window.
    let runtime_at = match left_bottom {
        Some(_) if two => pos2(left + column_w + 14.0, columns_top),
        _ => pos2(left, y),
    };
    let (runtime_bottom, asked) = runtime_view::runtime_box(
        ui,
        pal,
        runtime_at,
        column_w,
        shared.name,
        app,
        runtime,
        shared.keyboard,
        paint,
    );
    if asked.is_some() {
        shared.asked = asked;
    }
    y = y.max(runtime_bottom + 12.0);

    // Scheduled tasks, across both columns.
    if let Some(answered) = runtime.runtime() {
        let at = pos2(left, y);
        y = match &answered.tasks {
            Some(tasks) => {
                let note = strings::tasks_read_from(tasks.len(), &environment);
                let tasks: Vec<_> = tasks.iter().collect();
                runtime_view::tasks_box(ui, pal, at, inner, &note, &tasks, Some(app), now, paint)
            }
            None => {
                let text = strings::endpoint_not_exposed("scheduledtasks", &environment);
                let title = crate::strings::SCHEDULED_TASKS;
                runtime_view::message_box(ui, pal, at, inner, title, &text, paint)
            }
        } + 12.0;
    }

    // By module: a table that sorts.
    let modules = sort.apply(app);
    let note = strings::sorted_by(sort.column.label(), sort.ascending);
    let table_h = 41.0 - 6.0
        + draw::box_head_extra(ui, inner, strings::BY_MODULE, &note)
        + size::TABLE_HEAD_H
        + modules.len() as f32 * size::TABLE_ROW_H
        + 1.0;
    let table = Rect::from_min_size(pos2(left, y), vec2(inner, table_h));
    if paint {
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
    shared: &mut Shared<'_>,
    paint: bool,
) -> f32 {
    let now = shared.now;
    let answered = shared.runtime.runtime();
    let environment = shared.runtime.environment();
    let routes = answered
        .and_then(|runtime| runtime.routes.as_deref())
        .map(|routes| routes_of(app, &module.name, routes));
    let tasks = answered
        .and_then(|runtime| runtime.tasks.as_deref())
        .map(|tasks| tasks_of(app, &module.name, tasks))
        .unwrap_or_default();
    let left = origin.x + 16.0;
    let inner = width - 32.0;
    let mut top = origin.y + 14.0;
    // What is wrong with the module, at the top of the view it concerns.
    if let Some((severity, bold, rest)) = strings::module_alert(app, shared.name, &module.name) {
        let at = pos2(left, top);
        top = draw::alert_box(ui, pal, at, inner, severity, &bold, &rest, None, paint).0 + 12.0;
    }
    // A circle is counted once. The other members say where.
    for carrier in app.circles_counted_elsewhere(&module.name) {
        let text = strings::part_of_circle_counted_on(carrier);
        top = note_alert(ui, pal, pos2(left, top), inner, &text, paint) + 12.0;
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
            value: crate::strings::count(u64::from(kind.files)),
            low: false,
        })
        .collect();
    let note = crate::strings::files_and_lines(module.files, module.lines);
    let kinds_bottom = bar_box(
        ui,
        pal,
        pos2(left, top),
        column_w,
        strings::FILES_BY_KIND,
        &note,
        &bars,
        None,
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
        ModuleState::On => (crate::strings::ON, pal.good, StateMark::Good),
        ModuleState::Off => (crate::strings::OFF, pal.off, StateMark::Off),
        ModuleState::Unknown => (strings::STATE_UNKNOWN, pal.ink_2, StateMark::None),
    };
    lines.push((strings::MODULE_STATE, word.to_owned(), color, mark));
    lines.push((
        strings::COL_TESTS,
        crate::strings::count(u64::from(module.tests)),
        pal.ink,
        StateMark::None,
    ));
    let coverage = percent(module).map_or(strings::UNKNOWN.to_owned(), strings::percent);
    lines.push((strings::TEST_COVERAGE, coverage, pal.ink, StateMark::None));
    // The routes of the module, when the running app gave its mappings.
    if let Some(routes) = &routes {
        let with = |name: &str| {
            routes
                .iter()
                .filter(|route| route.middleware.iter().any(|middleware| middleware == name))
                .count()
        };
        let value = strings::routes_of_module(routes.len(), with("api"), with("web"));
        lines.push((crate::strings::ROUTES, value, pal.ink, StateMark::None));
    }
    // Which migrations ran comes from a read of the migration state (design D7).
    // Until then: how many files there are.
    // Of the default connection, the one the app migrates.
    let state = app
        .package_data
        .migrations_for("")
        .and_then(|migrations| migrations.get(&module.name));
    let waiting = state.map_or(0, |state| state.waiting.len());
    let migrations = match state {
        Some(state) => crate::strings::ran(state.ran.len()),
        None => crate::strings::files(u64::from(
            module
                .kinds
                .iter()
                .find(|kind| kind.kind == "Migrations")
                .map_or(0, |kind| kind.files),
        )),
    };
    // With waiting migrations: "9 ran" and then "1 waiting" with its mark (S4.3).
    let mut ran_before = None;
    if waiting > 0 {
        ran_before = Some((lines.len(), migrations));
        let value = crate::strings::waiting(waiting);
        lines.push((crate::strings::MIGRATIONS, value, pal.warn, StateMark::Warn));
    } else {
        lines.push((
            crate::strings::MIGRATIONS,
            migrations,
            pal.ink,
            StateMark::None,
        ));
    }
    let churn = match (&app.git, module.churn) {
        (Git::Repository { .. }, Some(churn)) => {
            strings::churn(churn.commits, churn.last_commit, now)
        }
        (Git::NoGit, _) => crate::strings::GIT_NOT_FOUND.to_owned(),
        _ => crate::strings::NOT_A_REPOSITORY.to_owned(),
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
                StateMark::Warn => draw::mark_warn(ui, at, *color),
                StateMark::None => {}
            }
            if let Some((_, ran)) = ran_before.as_ref().filter(|(line, _)| *line == i) {
                draw::label(
                    ui,
                    pos2(at.left() - 8.0, mid),
                    Align2::RIGHT_CENTER,
                    ran,
                    ty::BODY,
                    pal.ink,
                );
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
        let sentence = strings::state_not_in_a_file(app, shared.name);
        let text = draw::galley_wrapped(ui, &sentence, ty::BODY, pal.ink_2, inner.min(560.0));
        let text_h = text.size().y;
        if paint {
            let rect = draw::paint_galley(ui, pos2(left, bottom + 12.0), Align2::LEFT_TOP, text);
            draw::announce(ui, rect, Role::Label, &sentence);
        }
        bottom += 12.0 + text_h;
    }

    // What the running app says about the module: its status, its scheduled tasks and
    // its routes. The status box is full width above the tasks (deviation).
    if let Some((box_bottom, asked)) = runtime_view::module_runtime_box(
        ui,
        pal,
        pos2(left, bottom + 12.0),
        inner,
        app,
        &module.name,
        shared.runtime,
        shared.keyboard,
        paint,
    ) {
        if asked.is_some() {
            shared.asked = asked;
        }
        bottom = box_bottom;
    }
    if !tasks.is_empty() {
        let note = strings::tasks_count(tasks.len());
        let at = pos2(left, bottom + 12.0);
        bottom = runtime_view::tasks_box(ui, pal, at, inner, &note, &tasks, None, now, paint);
    }
    if let Some(routes) = routes.filter(|routes| !routes.is_empty()) {
        let note = crate::strings::routes_read_from(routes.len(), environment);
        let at = pos2(left, bottom + 12.0);
        bottom = runtime_view::routes_box(ui, pal, at, inner, &note, &routes, paint);
    }
    bottom + 16.0 - origin.y
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateMark {
    None,
    Good,
    Off,
    Warn,
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

/// The type of a state word in the Findings cell.
const STATE_TYPE: Type = Type {
    size: 12.0,
    ..ty::LABEL
};
/// A line of states in the Findings cell, and the gap between two lines.
const STATE_LINE: f32 = 17.0;
const STATE_GAP: f32 = 8.0;
/// The height of the value of a cell, and of the strip with one line of values.
const VALUE_H: f32 = 26.0;
const STRIP_H: f32 = 60.0;

/// A state in the Findings cell: its word, its colour and its mark.
struct StateWord {
    text: String,
    color: Color32,
    mark: FindingState,
}

fn state_words(pal: &Palette, problems: usize, warnings: usize) -> Vec<StateWord> {
    let mut words = Vec::new();
    if problems > 0 {
        words.push(StateWord {
            text: crate::strings::problems(problems),
            color: pal.bad,
            mark: FindingState::Problems(problems),
        });
    }
    if warnings > 0 {
        words.push(StateWord {
            text: crate::strings::warnings(warnings),
            color: pal.warn,
            mark: FindingState::Warning,
        });
    }
    if problems + warnings == 0 {
        words.push(StateWord {
            text: crate::strings::GOOD.to_owned(),
            color: pal.good,
            mark: FindingState::Good,
        });
    }
    words
}

/// Places the states left to right, and on the next line where the cell has no
/// `room` left, as the drawings wrap them. Gives the offset of each from the left
/// of the cell's content, and its line.
fn place_states(ui: &Ui, words: &[StateWord], room: f32) -> Vec<(f32, usize)> {
    let mut placed = Vec::new();
    let (mut at, mut line) = (0.0, 0);
    for word in words {
        let text_w = draw::galley(ui, &word.text, STATE_TYPE, word.color)
            .size()
            .x;
        let width = size::STATE_MARK + 5.0 + text_w;
        if at > 0.0 && at + width > room {
            at = 0.0;
            line += 1;
        }
        placed.push((at, line));
        at += width + 8.0;
    }
    placed
}

/// The stats strip: one box, a cell per figure, hairlines between. Each cell keeps
/// the width it has when all five are there. The strip grows when the states of the
/// Findings cell take more than one line. Returns its bottom edge.
fn stats(
    ui: &Ui,
    pal: &Palette,
    min: eframe::egui::Pos2,
    width: f32,
    cells: &[Stat],
    runtime: Option<(Brief, &str)>,
    paint: bool,
) -> f32 {
    let count = cells.len() + usize::from(runtime.is_some());
    let cell_w = (width / 5.0).floor().max(150.0).min(width / count as f32);
    let states = cells
        .iter()
        .find_map(|cell| cell.findings)
        .map(|(problems, warnings)| {
            let words = state_words(pal, problems, warnings);
            let placed = place_states(ui, &words, cell_w - 24.0);
            (words, placed)
        });
    let lines = states
        .as_ref()
        .and_then(|(_, placed)| placed.last())
        .map_or(1, |(_, line)| line + 1);
    let value_h = (lines as f32 * STATE_LINE + (lines - 1) as f32 * STATE_GAP).max(VALUE_H);
    let strip = Rect::from_min_size(
        min,
        vec2(cell_w * count as f32, STRIP_H - VALUE_H + value_h),
    );
    if !paint {
        return strip.bottom();
    }
    draw::outlined(ui, strip, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
    if let Some((brief, environment)) = runtime {
        // Runtime: a stroke mark, the word of the status, and where it was asked.
        let x = strip.left() + cells.len() as f32 * cell_w;
        draw::vline(ui, x, strip.top() + 1.0..strip.bottom() - 1.0, pal.line);
        draw::label(
            ui,
            pos2(x + 12.0, strip.top() + 9.0),
            Align2::LEFT_TOP,
            crate::strings::RUNTIME,
            ty::SECONDARY,
            pal.ink_2,
        );
        let mid = strip.top() + 9.0 + 16.0 + 13.0;
        let color = brief.tone.color(pal);
        let mark = Rect::from_min_size(pos2(x + 12.0, mid - 6.0), Vec2::splat(size::STATE_MARK));
        draw::runtime_mark(ui, mark, brief.mark, color);
        let word_type = Type {
            size: 12.0,
            ..ty::LABEL
        };
        let at = pos2(mark.right() + 5.0, mid);
        let word = draw::paint_text(ui, at, Align2::LEFT_CENTER, brief.word, word_type, color);
        let on = crate::strings::on_environment(environment);
        let room = strip.right() - 8.0 - (word.right() + 8.0);
        let on = draw::galley_elided(ui, &on, ty::SECONDARY, pal.ink_2, room);
        let on = draw::paint_galley(ui, pos2(word.right() + 8.0, mid), Align2::LEFT_CENTER, on);
        // Read aloud as one thing: "Up on local".
        let said = crate::strings::runtime_tooltip(brief.word, environment, None);
        draw::announce(ui, mark.union(word).union(on), Role::Label, &said);
    }
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
        if let (Some(_), Some((words, placed))) = (cell.findings, &states) {
            // Each state has a mark and a word; colour is only state. One line sits
            // in the middle of the value, as the figures of the other cells do.
            let value_top = strip.top() + 9.0 + 16.0;
            for (word, (offset, line)) in words.iter().zip(placed) {
                let mid = if lines == 1 {
                    value_top + VALUE_H / 2.0
                } else {
                    value_top + STATE_LINE / 2.0 + *line as f32 * (STATE_LINE + STATE_GAP)
                };
                let mark = Rect::from_min_size(
                    pos2(x + 12.0 + offset, mid - 6.0),
                    Vec2::splat(size::STATE_MARK),
                );
                match word.mark {
                    FindingState::Problems(_) => draw::mark_bad(ui, mark, word.color, pal.surface),
                    FindingState::Warning => draw::mark_warn(ui, mark, word.color),
                    FindingState::Good => draw::mark_good(ui, mark, word.color),
                }
                draw::label(
                    ui,
                    pos2(mark.right() + 5.0, mid),
                    Align2::LEFT_CENTER,
                    &word.text,
                    STATE_TYPE,
                    word.color,
                );
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
    footer: Option<(&str, String)>,
    paint: bool,
) -> f32 {
    const ROW: f32 = 21.0;
    // The footer: a line of its own under a hairline, for what is in no bar.
    let footer_h = if footer.is_some() {
        4.0 + size::LIST_LINE_H - 6.0
    } else {
        0.0
    };
    let height = 41.0
        + draw::box_head_extra(ui, width, title, note)
        + bars.len() as f32 * ROW
        + 8.0
        + footer_h;
    let rect = Rect::from_min_size(min, vec2(width, height));
    if !paint {
        return rect.bottom();
    }
    if let Some((name, value)) = &footer {
        let top = rect.bottom() - 8.0 - footer_h + 4.0;
        draw::hline(ui, rect.left() + 13.0..rect.right() - 13.0, top, pal.line);
        let mid = top + 1.0 + (size::LIST_LINE_H - 6.0) / 2.0 + 2.0;
        draw::label(
            ui,
            pos2(rect.left() + 13.0, mid),
            Align2::LEFT_CENTER,
            name,
            ty::BODY,
            pal.ink,
        );
        let value_type = Type {
            size: 11.5,
            ..ty::CODE
        };
        draw::label(
            ui,
            pos2(rect.right() - 13.0, mid),
            Align2::RIGHT_CENTER,
            value,
            value_type,
            pal.ink,
        );
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
/// In a narrow workspace the columns that say least give way to the module name.
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

    // The columns that fit, leaving at least 170 px for the module.
    let mut columns: Vec<Column> = Column::ALL.to_vec();
    let needs = |columns: &[Column]| {
        columns
            .iter()
            .map(|column| column.width() + 12.0)
            .sum::<f32>()
    };
    for give_way in [
        Column::Commits,
        Column::UsedBy,
        Column::Uses,
        Column::Lines,
        Column::Tests,
    ] {
        if right - left - needs(&columns[1..]) >= 170.0 {
            break;
        }
        columns.retain(|column| *column != give_way);
    }
    // From the right edge: numbers have fixed widths, the module takes the rest.
    let mut edges: Vec<(Column, f32, f32)> = Vec::new();
    let mut x = right;
    for column in columns.iter().rev() {
        if *column == Column::Module {
            edges.push((*column, left, x));
        } else {
            edges.push((*column, x - column.width(), x));
            x -= column.width() + 12.0;
        }
    }
    edges.reverse();
    let name_right = edges[0].2;

    // Header: 26 px, 11.5 px weight 600 in ink-2, line-2 underneath. A cell is a button.
    let head_bottom = rect.top() + size::TABLE_HEAD_H;
    draw::hline(
        ui,
        rect.left() + 1.0..rect.right() - 1.0,
        head_bottom - 1.0,
        pal.line_2,
    );
    for (column, from, to) in &edges {
        let at_left = matches!(column, Column::Module | Column::Findings);
        let cell = Rect::from_min_max(pos2(*from, rect.top()), pos2(*to, head_bottom - 1.0));
        let response = ui.interact(
            cell,
            ui.scope_id().with(("by-module-head", column.label())),
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
        let text = if at_left {
            draw::paint_text(
                ui,
                pos2(*from, mid),
                Align2::LEFT_CENTER,
                column.label(),
                ty::LABEL,
                color,
            )
        } else {
            draw::paint_text(
                ui,
                pos2(*to, mid),
                Align2::RIGHT_CENTER,
                column.label(),
                ty::LABEL,
                color,
            )
        };
        if sorted {
            // A chevron: down for A to Z and low to high, up for the reverse.
            let tip = if at_left {
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

    // Rows: 28 px, a hairline between them. Numbers are right aligned. The table is
    // one stop: J and K move through it, Enter selects the module in the index.
    let nav = draw::ListNav::new(ui, "by-module", modules.len());
    for (row, module) in modules.iter().enumerate() {
        let top = head_bottom + row as f32 * size::TABLE_ROW_H;
        let row_rect = Rect::from_min_max(
            pos2(rect.left() + 1.0, top),
            pos2(rect.right() - 1.0, top + size::TABLE_ROW_H),
        );
        let response = ui.interact(row_rect, nav.id(row), nav.sense(row));
        nav.after(ui, row, &response);
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
        let name_w = (name_right - stamp.right() - 8.0 - 12.0).max(0.0);
        let name_color = if off { pal.ink_2 } else { pal.ink };
        let name = draw::galley_elided(
            ui,
            &module.name,
            ty::BODY.with_face(Face::Sans600),
            name_color,
            name_w,
        );
        draw::paint_galley(
            ui,
            pos2(stamp.right() + 8.0, mid),
            Align2::LEFT_CENTER,
            name,
        );

        let unknown = (strings::UNKNOWN.to_owned(), pal.ink_3);
        for (column, from, to) in &edges[1..] {
            let (text, color) = match column {
                Column::Module => continue,
                Column::Files => (crate::strings::count(module.files), pal.ink),
                Column::Lines => (crate::strings::count(module.lines), pal.ink),
                Column::Tests => (crate::strings::count(u64::from(module.tests)), pal.ink),
                Column::Coverage => percent(module).map_or(unknown.clone(), |percent| {
                    (strings::percent(percent), pal.ink)
                }),
                Column::Uses => (app.uses(&module.name).count().to_string(), pal.ink),
                Column::UsedBy => (app.used_by(&module.name).count().to_string(), pal.ink),
                Column::Commits => match (&app.git, commits(module)) {
                    (Git::Repository { .. }, Some(commits)) => {
                        (crate::strings::count(u64::from(commits)), pal.ink)
                    }
                    _ => unknown.clone(),
                },
                Column::Findings => {
                    // The finding state: a mark and a word, at the left of its column.
                    let (word, color, kind) =
                        match (module.state, app.findings.state_of(&module.name)) {
                            (ModuleState::Off, _) => (crate::strings::OFF.to_owned(), pal.off, 3),
                            (_, FindingState::Problems(n)) => {
                                (crate::strings::problems(n), pal.bad, 0)
                            }
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
                            (_, FindingState::Good) => {
                                (crate::strings::GOOD.to_owned(), pal.good, 2)
                            }
                        };
                    let mark =
                        Rect::from_min_size(pos2(*from, mid - 6.0), Vec2::splat(size::STATE_MARK));
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
                    continue;
                }
            };
            draw::paint_text(
                ui,
                pos2(*to, mid),
                Align2::RIGHT_CENTER,
                &text,
                ty::BODY,
                color,
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

/// A line of the Dependencies box, laid out before it is painted: a name with the
/// stamp of the module, and a value at the right.
struct DependencyLine<'a> {
    name: String,
    name_galley: Arc<Galley>,
    stamp: Option<(&'a str, StampState)>,
    value: String,
    value_galley: Arc<Galley>,
    problem: bool,
    height: f32,
}

impl<'a> DependencyLine<'a> {
    /// Lays out the value first, then wraps the name in the room the value and the
    /// stamp leave, as the row of the drawings does. A value wider than half the
    /// line wraps too. The line grows from 27 px by the lines the taller of the two adds.
    fn new(
        ui: &Ui,
        pal: &Palette,
        width: f32,
        name: String,
        stamp: Option<(&'a str, StampState)>,
        value: String,
        problem: bool,
    ) -> Self {
        let mono = Type {
            size: 11.5,
            ..ty::CODE
        };
        let (value_type, color) = if problem {
            (ty::LABEL, pal.bad)
        } else {
            (mono, pal.ink_2)
        };
        let mark_w = if problem { size::STATE_MARK + 5.0 } else { 0.0 };
        let value_galley = draw::galley_wrapped(
            ui,
            &value,
            value_type,
            color,
            (width / 2.0).floor() - mark_w,
        );
        let stamp_w = if stamp.is_some() {
            7.0 + size::STAMP_W
        } else {
            0.0
        };
        let room = width - value_galley.size().x - mark_w - 10.0 - stamp_w;
        let name_galley = draw::galley_wrapped(ui, &name, ty::BODY, pal.ink, room);
        // Each line of text past the first adds its height to the 27 px line.
        let more = (name_galley.size().y - ty::BODY.line)
            .max(value_galley.size().y - value_type.line)
            .max(0.0);
        Self {
            name,
            name_galley,
            stamp,
            value,
            value_galley,
            problem,
            height: size::LIST_LINE_H + more,
        }
    }

    /// Paints the line with its top at `y`, with a hairline under it unless it is the last.
    fn paint(self, ui: &Ui, pal: &Palette, rect: Rect, y: f32, last: bool) {
        let mid = y + self.height / 2.0;
        let name = draw::paint_galley(
            ui,
            pos2(rect.left() + 13.0, mid),
            Align2::LEFT_CENTER,
            self.name_galley,
        );
        draw::announce(ui, name, Role::Label, &self.name);
        if let Some((code, state)) = self.stamp {
            draw::stamp(
                ui,
                pal,
                pos2(name.right() + 7.0, (mid - size::STAMP_H / 2.0).round()),
                code,
                state,
            );
        }
        let value = draw::paint_galley(
            ui,
            pos2(rect.right() - 13.0, mid),
            Align2::RIGHT_CENTER,
            self.value_galley,
        );
        draw::announce(ui, value, Role::Label, &self.value);
        if self.problem {
            let mark = Rect::from_min_size(
                pos2(
                    value.left() - 5.0 - size::STATE_MARK,
                    mid - size::STATE_MARK / 2.0,
                ),
                Vec2::splat(size::STATE_MARK),
            );
            draw::mark_bad(ui, mark, pal.bad, pal.surface);
        }
        if !last {
            draw::hline(
                ui,
                rect.left() + 13.0..rect.right() - 13.0,
                y + self.height - 1.0,
                pal.line,
            );
        }
    }
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
    let room = width - 26.0;
    let mut lines: Vec<DependencyLine<'_>> = uses
        .iter()
        .map(|link| {
            let name = crate::strings::a_uses_b(strings::USES, &link.to).replacen(" uses ", " ", 1);
            let stamp = app.module(&link.to).map(|used| {
                let state = StampState {
                    selected: false,
                    off: used.state == ModuleState::Off,
                };
                (used.code.as_str(), state)
            });
            let problem = link.is_problem(&app.rules);
            let value = crate::strings::refs_state(link, &app.rules);
            DependencyLine::new(ui, pal, room, name, stamp, value, problem)
        })
        .collect();
    if !used_by.is_empty() {
        let names: Vec<&str> = used_by.iter().map(|link| link.from.as_str()).collect();
        let counts: Vec<String> = used_by
            .iter()
            .map(|link| link.refs.len().to_string())
            .collect();
        let total: usize = used_by.iter().map(|link| link.refs.len()).sum();
        let plural = if total == 1 { "ref" } else { "refs" };
        let name = format!("{} {}", strings::USED_BY, names.join(", "));
        let value = format!("{} {plural}", counts.join(", "));
        lines.push(DependencyLine::new(ui, pal, room, name, None, value, false));
    }
    let height: f32 = lines.iter().map(|line| line.height).sum();
    let note = strings::out_in(uses.len(), used_by.len());
    let head = draw::box_head_extra(ui, width, strings::DEPENDENCIES, &note);
    let rect = Rect::from_min_size(min, vec2(width, 41.0 + head + height + 8.0));
    if !paint {
        return Some(rect.bottom());
    }
    let mut y = draw::box_head(ui, pal, rect, strings::DEPENDENCIES, &note);
    let count = lines.len();
    for (i, line) in lines.into_iter().enumerate() {
        let height = line.height;
        line.paint(ui, pal, rect, y, i + 1 == count);
        y += height;
    }
    Some(rect.bottom())
}
