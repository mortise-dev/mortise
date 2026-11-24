//! The Metrics tab: what the environment answered (S7.1, S7.2), and the figures of each module or
//! route over a range, with the chart of one of them (S7.3). No figure has a colour: there are no
//! limits yet.

use std::sync::Arc;
use std::time::Duration;

use eframe::egui::{
    Align2, Color32, Rect, Role, ScrollArea, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{Answer, AppSnapshot, Figures, NoMetrics, Range, Row, RowOf, Scope};

use crate::app::workspace::WorkAction;
use crate::metrics::chart::{self, CHART_H, Plot};
use crate::metrics::strings;
use crate::runtime::view::{Brief, RuntimeAction};
use crate::ui::draw::{self, Button, ButtonKind, EmptyMark, Icon, StampState, empty_state};
use crate::ui::theme::{Face, Palette, radius, size, ty};

/// What the Metrics tab shows, first match wins (design D7).
pub enum MetricsView<'a> {
    /// S7.1: no environment is asked for the app.
    NotConnected { app: &'a str },
    /// No answer yet, or no read of metrics ended yet.
    Waiting { environment: &'a str },
    /// The environment gave no answer, refused the credentials or answered with a status
    /// the actuator does not use. Never `Answer::Answered`.
    Unreached {
        environment: &'a str,
        answer: Arc<Answer>,
    },
    /// S7.2: the environment answers and does not list `prometheus`.
    NotExposed { environment: &'a str },
    /// The last read of `prometheus` gave no metrics.
    NoMetrics {
        environment: &'a str,
        why: NoMetrics,
    },
    /// The last answer had no `# TYPE` line.
    NotMetrics { environment: &'a str },
    /// The last answer had more series than a read keeps. `limit` is the most numbers a read keeps.
    TooMany { environment: &'a str, limit: usize },
    /// S7.3: the figures over the chosen range.
    Table(TableView<'a>),
}

/// What the table of S7.3 draws.
pub struct TableView<'a> {
    pub environment: &'a str,
    /// The environments of the app, by name, for the switch.
    pub environments: Vec<&'a str>,
    pub figures: &'a Figures,
    /// False when `mappings` is not exposed.
    pub placed: bool,
    pub scope: Scope<'a>,
    pub range: Range,
    /// The selected row. The app clears it when no row of `figures` is it.
    pub row: Option<RowOf>,
    /// How often metrics are read.
    pub pace: Duration,
    /// The time since the newest read.
    pub ago: Duration,
}

/// What the Metrics tab keeps of an app (design D6).
#[derive(Debug, Default)]
pub struct MetricsTab {
    pub range: Range,
    /// The selected row, by what it is, as rows sort again after each read.
    pub row: Option<RowOf>,
    /// The module selected in the index that the row belongs to.
    module: Option<String>,
}

impl MetricsTab {
    /// Clears the row when the selection of the index changed.
    pub fn follow(&mut self, module: Option<&str>) {
        if self.module.as_deref() != module {
            self.module = module.map(str::to_owned);
            self.row = None;
        }
    }

    pub fn apply(&mut self, action: MetricsAction) {
        match action {
            MetricsAction::Range(range) => self.range = range,
            MetricsAction::Row(row) => self.row = Some(row),
        }
    }
}

/// What the table of the Metrics tab asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricsAction {
    /// Show another range. The reads stay.
    Range(Range),
    /// Show this row in the chart.
    Row(RowOf),
}

/// Paints the Metrics tab into `body`. Returns what was asked for.
pub fn metrics(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    app: &AppSnapshot,
    view: &MetricsView<'_>,
    keyboard: bool,
) -> Option<WorkAction> {
    let try_again = (
        crate::strings::TRY_AGAIN,
        ButtonKind::Primary,
        RuntimeAction::TryAgain,
    );
    let open_settings = (
        crate::strings::OPEN_SETTINGS,
        ButtonKind::Default,
        RuntimeAction::OpenSettings,
    );
    let asked = match view {
        MetricsView::NotConnected { app } => explain(
            ui,
            pal,
            body,
            strings::NOT_CONNECTED_TITLE,
            &strings::not_connected(app),
            &[(
                strings::CONNECT_ENVIRONMENT,
                ButtonKind::Primary,
                RuntimeAction::OpenSettings,
            )],
            keyboard,
        ),
        MetricsView::Waiting { environment } => {
            let title = strings::reading(environment);
            empty_state(
                ui,
                pal,
                body,
                EmptyMark::Icon(Icon::Metrics),
                &title,
                &[],
                0.0,
            );
            None
        }
        MetricsView::Unreached {
            environment,
            answer,
        } => unreached(ui, pal, body, environment, answer, keyboard),
        MetricsView::NotExposed { environment } => explain(
            ui,
            pal,
            body,
            &strings::not_exposed_title(environment),
            strings::NOT_EXPOSED,
            &[try_again, open_settings],
            keyboard,
        ),
        MetricsView::NoMetrics { environment, why } => {
            // A read that did not reach the app says what the runtime box says.
            let said = match why {
                NoMetrics::Off => Ok((strings::off_title(environment), strings::OFF.to_owned())),
                NoMetrics::Unavailable { url, line } => Ok((
                    strings::unavailable_title(environment),
                    strings::unavailable(environment, url, line),
                )),
                NoMetrics::TooLarge { limit, .. } => Ok((
                    strings::too_large_title(environment),
                    strings::too_large(*limit),
                )),
                NoMetrics::NoAnswer { url, message } => Err(Answer::NoAnswer {
                    url: url.clone(),
                    message: message.clone(),
                }),
                NoMetrics::NotAuthorized { url, status } => Err(Answer::NotAuthorized {
                    url: url.clone(),
                    status: *status,
                }),
                NoMetrics::Unexpected {
                    url,
                    status,
                    location,
                } => Err(Answer::Unexpected {
                    url: url.clone(),
                    status: *status,
                    location: location.clone(),
                }),
            };
            match said {
                Ok((title, text)) => explain(ui, pal, body, &title, &text, &[try_again], keyboard),
                Err(answer) => unreached(ui, pal, body, environment, &answer, keyboard),
            }
        }
        MetricsView::NotMetrics { environment } => explain(
            ui,
            pal,
            body,
            &strings::not_metrics_title(environment),
            &strings::not_metrics(environment),
            &[try_again, open_settings],
            keyboard,
        ),
        MetricsView::TooMany { environment, limit } => explain(
            ui,
            pal,
            body,
            &strings::too_many_title(environment),
            &strings::too_many(*limit),
            &[try_again],
            keyboard,
        ),
        MetricsView::Table(view) => return table(ui, pal, body, app, view, keyboard),
    };
    asked.map(WorkAction::Runtime)
}

/// The sentence of the runtime box for an environment that gave nothing, with a button to try
/// again and one to open Settings.
fn unreached(
    ui: &Ui,
    pal: &Palette,
    body: Rect,
    environment: &str,
    answer: &Answer,
    keyboard: bool,
) -> Option<RuntimeAction> {
    let text = match answer {
        Answer::NoAnswer { url, message } => {
            crate::strings::cannot_reach(environment, url, message)
        }
        Answer::NotAuthorized { url, status } => {
            crate::strings::refused_credentials(environment, url, *status)
        }
        Answer::Unexpected {
            url,
            status,
            location,
        } => crate::strings::answered_unexpected(environment, url, *status, location.as_deref()),
        Answer::Answered(_) => return None,
    };
    // The title is the word the index header shows for the environment.
    let word = Brief::of_answer(answer).word;
    let title = crate::strings::runtime_tooltip(word, environment, None);
    explain(
        ui,
        pal,
        body,
        &title,
        &text,
        &[
            (
                crate::strings::TRY_AGAIN,
                ButtonKind::Primary,
                RuntimeAction::TryAgain,
            ),
            (
                crate::strings::OPEN_SETTINGS,
                ButtonKind::Default,
                RuntimeAction::OpenSettings,
            ),
        ],
        keyboard,
    )
}

/// An empty state of the tab (C16): the metrics icon, a title, a paragraph and a row of buttons.
fn explain(
    ui: &Ui,
    pal: &Palette,
    body: Rect,
    title: &str,
    text: &str,
    buttons: &[(&str, ButtonKind, RuntimeAction)],
    keyboard: bool,
) -> Option<RuntimeAction> {
    let mut at = empty_state(
        ui,
        pal,
        body,
        EmptyMark::Icon(Icon::Metrics),
        title,
        &[text],
        size::CONTROL_H,
    );
    let mut asked = None;
    for (label, kind, action) in buttons {
        let button = Button {
            label,
            kind: *kind,
            ..Default::default()
        };
        if button.show(ui, pal, at, keyboard).clicked() {
            asked = Some(action.clone());
        }
        at.x += button.width(ui) + 8.0;
    }
    asked
}

/// The widths of the columns of S7.3: the code stamp and the figures. The name takes the rest.
const STAMP_COLUMN: f32 = 34.0;
const COLUMNS: [f32; 6] = [90.0, 80.0, 70.0, 70.0, 100.0, 70.0];
const COLUMN_GAP: f32 = 12.0;
/// The space between the top line, the chart box and the table box.
const GAP: f32 = 12.0;
/// The size of the sparkline of a row.
const SPARK: Vec2 = vec2(58.0, 14.0);
/// The space between the two switches, and the least space before the read line on their line.
const SWITCH_GAP: f32 = 10.0;
const READ_GAP: f32 = 16.0;
/// The space between two lines of the top, when the switches or the read line do not fit on one.
const TOP_LINE_GAP: f32 = 8.0;

/// A row of the table as it is shown.
struct Shown<'f> {
    row: &'f Row,
    name: String,
    code: Option<&'f str>,
    /// A route has no jobs and no database time of its own.
    route: bool,
}

/// S7.3: the switches of the environment and the range, the chart of the selected row and the
/// table of modules or routes, in a body that scrolls. Lines under the table say why a figure is
/// missing.
fn table(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    app: &AppSnapshot,
    view: &TableView<'_>,
    keyboard: bool,
) -> Option<WorkAction> {
    // "last read 8 s ago" counts on between reads.
    ui.ctx().request_repaint_after(Duration::from_secs(1));
    let figures = view.figures;
    let mut notes = Vec::new();
    if figures.covered.is_none() {
        notes.push(strings::ONE_READ.to_owned());
    }
    if !view.placed {
        notes.push(strings::not_placed(view.environment));
    }
    // The modules or routes, then the row of what is in no module.
    let mut rows: Vec<Shown<'_>> = figures
        .rows
        .iter()
        .map(|row| match &row.of {
            RowOf::Module(name) => Shown {
                row,
                name: name.clone(),
                code: app.module(name).map(|module| module.code.as_str()),
                route: false,
            },
            RowOf::Route { route, method } => Shown {
                row,
                name: strings::route(method, route.as_deref()),
                code: None,
                route: true,
            },
        })
        .collect();
    if let Some(outside) = &figures.outside {
        rows.push(Shown {
            row: outside,
            name: crate::strings::NOT_IN_A_MODULE.to_owned(),
            code: None,
            route: false,
        });
    }
    let mut selected = view
        .row
        .as_ref()
        .and_then(|of| rows.iter().position(|shown| &shown.row.of == of));
    let shown = selected.map(|at| &rows[at]);
    let title = match (shown, view.scope) {
        (Some(shown), _) => strings::chart_title(&shown.name),
        (None, Scope::All) => strings::CHART_ALL.to_owned(),
        (None, Scope::Module(name)) => strings::chart_title(name),
    };
    let line = shown.map_or(&figures.total.line, |shown| &shown.row.line);
    let now = line.now();
    let note = now.map(strings::now).unwrap_or_default();
    let points = line.runs.iter().flatten().map(|point| point.per_minute);
    let summary = match points.clone().reduce(f64::max) {
        Some(highest) => {
            let lowest = points.reduce(f64::min).unwrap_or(highest);
            strings::summary(now, highest, lowest)
        }
        None => strings::NO_RATE_YET.to_owned(),
    };
    let blank = figures.covered.is_none();
    // Before two reads every figure is an en dash.
    let or_dash = |text: Option<String>| {
        text.filter(|_| !blank)
            .unwrap_or_else(|| strings::NO_RATE.to_owned())
    };
    let rate = |rate: f64| or_dash(Some(strings::per_minute(rate)));

    let mut asked = None;
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        ScrollArea::vertical()
            .id_salt("metrics")
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let width = ui.available_width();
                let inner = width - 32.0;
                let notes: Vec<_> = notes
                    .iter()
                    .map(|note| {
                        let galley =
                            draw::galley_wrapped(ui, note, ty::SECONDARY, pal.ink_2, inner);
                        (galley, note.as_str())
                    })
                    .collect();
                // The top: the environment, the range, and when the last read was. The range
                // switch goes on a second line when it does not fit after the environment switch.
                // The read line goes at the right of the last switch's line when it fits there,
                // else on a line of its own under the switches.
                let labels = Range::ALL.map(strings::range);
                let environment_w = draw::segmented_width(ui, &view.environments);
                let range_w = draw::segmented_width(ui, &labels);
                let range_at = if environment_w + SWITCH_GAP + range_w <= inner {
                    vec2(environment_w + SWITCH_GAP, 0.0)
                } else {
                    vec2(0.0, size::CONTROL_H + TOP_LINE_GAP)
                };
                let switches_h = range_at.y + size::CONTROL_H;
                let read_line = strings::read_every(view.pace, view.ago);
                let read = draw::galley(ui, &read_line, ty::SECONDARY, pal.ink_2);
                let read_apart = range_at.x + range_w + READ_GAP + read.size().x > inner;
                let top_h = if read_apart {
                    switches_h + TOP_LINE_GAP + read.size().y
                } else {
                    switches_h
                };

                let chart_h = 41.0 + draw::box_head_extra(ui, inner, &title, &note) + CHART_H + 8.0;
                let table_h = size::TABLE_HEAD_H + rows.len() as f32 * size::TABLE_ROW_H + 1.0;
                let notes_h: f32 = notes.iter().map(|(galley, _)| 8.0 + galley.size().y).sum();
                let height = 14.0 + top_h + GAP + chart_h + GAP + table_h + notes_h + 16.0;
                let (area, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
                let left = area.left() + 16.0;
                let right = left + inner;

                let top = area.top() + 14.0;
                let (_, clicked) = draw::segmented(
                    ui,
                    pal,
                    pos2(left, top),
                    "metrics-environment",
                    &view.environments,
                    view.environments
                        .iter()
                        .position(|name| *name == view.environment)
                        .unwrap_or(0),
                    keyboard,
                );
                if let Some(name) = clicked
                    .map(|at| view.environments[at])
                    .filter(|name| *name != view.environment)
                {
                    asked = Some(WorkAction::Runtime(RuntimeAction::Choose(name.to_owned())));
                }
                let chosen = Range::ALL
                    .iter()
                    .position(|range| *range == view.range)
                    .unwrap_or(1);
                let range_min = pos2(left, top) + range_at;
                let (_, clicked) = draw::segmented(
                    ui,
                    pal,
                    range_min,
                    "metrics-range",
                    &labels,
                    chosen,
                    keyboard,
                );
                if let Some(at) = clicked.filter(|at| *at != chosen) {
                    asked = Some(WorkAction::Metrics(MetricsAction::Range(Range::ALL[at])));
                }
                let read_rect = if read_apart {
                    draw::paint_galley(
                        ui,
                        pos2(left, top + switches_h + TOP_LINE_GAP),
                        Align2::LEFT_TOP,
                        read,
                    )
                } else {
                    draw::paint_galley(
                        ui,
                        pos2(right, range_min.y + size::CONTROL_H / 2.0),
                        Align2::RIGHT_CENTER,
                        read,
                    )
                };
                draw::announce(ui, read_rect, Role::Label, &read_line);

                // The chart of the selected row, or of everything in the scope.
                let chart_box = Rect::from_min_size(
                    pos2(left, top + top_h + GAP),
                    vec2(inner, chart_h),
                );
                let content = draw::box_head(ui, pal, chart_box, &title, &note);
                let name = strings::chart_name(&title, view.range);
                let plot = Plot {
                    line,
                    span: view.range.span(),
                    name: &name,
                    summary: &summary,
                    from: strings::range_ago(view.range),
                    to: strings::NOW,
                };
                let chart_rect = Rect::from_min_size(
                    pos2(chart_box.left() + 13.0, content),
                    vec2(inner - 26.0, CHART_H),
                );
                chart::chart(ui, pal, chart_rect, &plot);

                // The table.
                let rect =
                    Rect::from_min_size(pos2(left, chart_box.bottom() + GAP), vec2(inner, table_h));
                draw::outlined(ui, rect, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
                let stamp_left = rect.left() + 14.0;
                // A route has no stamp: its name takes the room of the stamp column.
                let name_left = match view.scope {
                    Scope::All => stamp_left + STAMP_COLUMN + COLUMN_GAP,
                    Scope::Module(_) => stamp_left,
                };
                // Each column of figures, from the right edge: its left and right.
                let mut x = rect.right() - 10.0;
                let mut columns = [(0.0, 0.0); 6];
                for (column, w) in columns.iter_mut().zip(COLUMNS).rev() {
                    *column = (x - w, x);
                    x -= w + COLUMN_GAP;
                }
                let [requests, under, errors, jobs, database, last_hour] = columns;
                let name_w = (requests.0 - COLUMN_GAP - name_left).max(0.0);

                // Header: 26 px, 11.5 px weight 600 in ink-2, line-2 underneath.
                let head_bottom = rect.top() + size::TABLE_HEAD_H;
                draw::hline(
                    ui,
                    rect.left() + 1.0..rect.right() - 1.0,
                    head_bottom - 1.0,
                    pal.line_2,
                );
                let mid = rect.top() + size::TABLE_HEAD_H / 2.0;
                let first = match view.scope {
                    Scope::All => crate::strings::COL_MODULE,
                    Scope::Module(_) => strings::COL_ROUTE,
                };
                let headers = [
                    (first, name_left, Align2::LEFT_CENTER),
                    (strings::COL_REQUESTS, requests.1, Align2::RIGHT_CENTER),
                    (strings::COL_UNDER_95, under.1, Align2::RIGHT_CENTER),
                    (strings::COL_ERRORS, errors.1, Align2::RIGHT_CENTER),
                    (strings::COL_JOBS, jobs.1, Align2::RIGHT_CENTER),
                    (strings::COL_DATABASE, database.1, Align2::RIGHT_CENTER),
                    (strings::COL_LAST_HOUR, last_hour.0, Align2::LEFT_CENTER),
                ];
                for (label, x, align) in headers {
                    let cell =
                        draw::paint_text(ui, pos2(x, mid), align, label, ty::LABEL, pal.ink_2);
                    draw::announce(ui, cell, Role::ColumnHeader, label);
                }

                // Rows: 28 px, a hairline between them. The table is one stop: J, K and the
                // arrows move the selected row while it has focus.
                let mut current = selected.unwrap_or(0);
                let nav = draw::ListNav::over(ui, "metrics-rows", rows.len(), &mut current);
                if nav.moved {
                    selected = Some(current);
                }
                // The rows sorted again with focus on one: focus goes to the selected row.
                let focused = ui.memory(|memory| memory.focused());
                if focused.is_some_and(|id| {
                    id != nav.id(current) && (0..rows.len()).any(|row| nav.id(row) == id)
                }) {
                    nav.focus(ui);
                }
                let last = rows.len().saturating_sub(1);
                for (i, shown) in rows.iter().enumerate() {
                    let top = head_bottom + i as f32 * size::TABLE_ROW_H;
                    let row_rect = Rect::from_min_max(
                        pos2(rect.left() + 1.0, top),
                        pos2(rect.right() - 1.0, top + size::TABLE_ROW_H - 1.0),
                    );
                    let response = ui.interact(row_rect, nav.id(i), nav.sense(i));
                    let on = selected == Some(i);
                    response.widget_info(|| WidgetInfo::selected(Role::Row, true, on, &shown.name));
                    if response.clicked() {
                        selected = Some(i);
                        // The keys of the table work at once after a click.
                        draw::focus_soon(ui, nav.id(i));
                    }
                    nav.after(ui, i, &response);
                    if on {
                        ui.painter().rect_filled(row_rect, 0, pal.well);
                        draw::hline(ui, row_rect.left()..row_rect.right(), top - 1.0, pal.line_2);
                        draw::hline(
                            ui,
                            row_rect.left()..row_rect.right(),
                            row_rect.bottom(),
                            pal.line_2,
                        );
                        draw::peg(ui, pal, row_rect);
                    } else {
                        if response.hovered() {
                            ui.painter().rect_filled(row_rect, 0, pal.well);
                        }
                        if i < last {
                            draw::hline(
                                ui,
                                row_rect.left()..row_rect.right(),
                                row_rect.bottom(),
                                pal.line,
                            );
                        }
                    }
                    if keyboard && response.has_focus() {
                        draw::focus_ring(ui, pal, row_rect.shrink(1.0), 0);
                    }

                    let mid = top + size::TABLE_ROW_H / 2.0;
                    if let Some(code) = shown.code {
                        let at = pos2(stamp_left, (mid - size::STAMP_H / 2.0).round());
                        let stamp = draw::stamp(ui, pal, at, code, StampState::default());
                        draw::announce(ui, stamp, Role::Cell, code);
                    }
                    let (face, color) = match (shown.code, shown.route) {
                        (None, false) => (Face::Sans400, pal.ink_2),
                        (Some(_), _) => (Face::Sans600, pal.ink),
                        (None, true) => (Face::Sans400, pal.ink),
                    };
                    let galley = draw::galley_elided(
                        ui,
                        &shown.name,
                        ty::BODY.with_face(face),
                        color,
                        name_w,
                    );
                    let elided = galley.elided;
                    let cell =
                        draw::paint_galley(ui, pos2(name_left, mid), Align2::LEFT_CENTER, galley);
                    let named = draw::announce(ui, cell, Role::Cell, &shown.name);
                    // A cut name gives the whole name on hover: two long routes can end alike.
                    if elided {
                        draw::tooltip(named, pal, &shown.name);
                    }

                    let rates = &shown.row.rates;
                    // A route has no jobs and no database time; without the module label no
                    // database time is known.
                    let unknown = !blank && !shown.route && !figures.database_known;
                    let cells = [
                        (rate(rates.requests_per_minute), requests.1, false),
                        (
                            or_dash(rates.under_95.map(strings::under_95)),
                            under.1,
                            false,
                        ),
                        (or_dash(rates.errors.map(strings::errors)), errors.1, false),
                        (
                            or_dash(
                                (!shown.route).then(|| strings::per_minute(rates.jobs_per_minute)),
                            ),
                            jobs.1,
                            false,
                        ),
                        (
                            if unknown {
                                strings::UNKNOWN.to_owned()
                            } else {
                                or_dash(rates.database.map(strings::database))
                            },
                            database.1,
                            unknown,
                        ),
                    ];
                    for (text, x, unknown) in cells {
                        let cell = draw::paint_text(
                            ui,
                            pos2(x, mid),
                            Align2::RIGHT_CENTER,
                            &text,
                            ty::BODY,
                            pal.ink,
                        );
                        let named = draw::announce(ui, cell, Role::Cell, &text);
                        if unknown {
                            draw::tooltip(named, pal, strings::NEEDS_NEWER_PACKAGE);
                        }
                    }
                    let spark = Rect::from_min_size(
                        pos2(last_hour.0, (mid - SPARK.y / 2.0).round()),
                        SPARK,
                    );
                    let name = strings::spark_name(&shown.name);
                    chart::spark(
                        ui,
                        pal,
                        spark,
                        &shown.row.last_hour,
                        Range::Hour.span(),
                        &name,
                    );
                }
                if let Some(at) = selected.filter(|at| Some(&rows[*at].row.of) != view.row.as_ref())
                {
                    asked = Some(WorkAction::Metrics(MetricsAction::Row(
                        rows[at].row.of.clone(),
                    )));
                }

                let mut y = rect.bottom();
                for (galley, note) in notes {
                    let note_rect = draw::paint_galley(
                        ui,
                        pos2(rect.left(), y + 8.0),
                        Align2::LEFT_TOP,
                        galley,
                    );
                    draw::announce(ui, note_rect, Role::Label, note);
                    y = note_rect.bottom();
                }
            });
    });
    asked
}
