//! The Metrics tab: what the environment answered (S7.1, S7.2), and a plain table of requests
//! and jobs per minute for each module. No figure has a colour: there are no limits yet.

use std::sync::Arc;

use eframe::egui::{
    Align2, Color32, Rect, Role, ScrollArea, Sense, Ui, UiBuilder, Vec2, pos2, vec2,
};
use mortise_core::{Answer, AppSnapshot, Figures, NoMetrics, Rates};

use crate::app::workspace::WorkAction;
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
    /// The figures of the last minute. `placed` is false when `mappings` is not exposed.
    Table {
        environment: &'a str,
        figures: &'a Figures,
        placed: bool,
    },
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
        MetricsView::Table {
            environment,
            figures,
            placed,
        } => {
            table(ui, pal, body, app, environment, figures, *placed);
            None
        }
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

/// The width of the column of the code stamp, and of each figure.
const STAMP_COLUMN: f32 = 34.0;
const REQUESTS_COLUMN: f32 = 90.0;
const JOBS_COLUMN: f32 = 70.0;

/// The table of the last minute: one row per module, then the row of what is in no module,
/// in a box that scrolls. Lines under it say why a figure is missing.
fn table(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    app: &AppSnapshot,
    environment: &str,
    figures: &Figures,
    placed: bool,
) {
    let mut notes = Vec::new();
    if figures.covered.is_none() {
        notes.push(strings::ONE_READ.to_owned());
    }
    if !placed {
        notes.push(strings::not_placed(environment));
    }
    // The modules, each with its code, then the row of what is in no module.
    let mut rows: Vec<(Option<&str>, &str, &Rates)> = figures
        .modules
        .iter()
        .map(|row| {
            let code = app
                .modules
                .iter()
                .find(|module| module.name == row.module)
                .map(|module| module.code.as_str());
            (code, row.module.as_str(), &row.rates)
        })
        .collect();
    rows.push((None, crate::strings::NOT_IN_A_MODULE, &figures.outside));
    let figure = |rate: f64| match figures.covered {
        Some(_) => strings::per_minute(rate),
        None => strings::NO_RATE.to_owned(),
    };

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
                let table_h = size::TABLE_HEAD_H + rows.len() as f32 * size::TABLE_ROW_H + 1.0;
                let notes_h: f32 = notes.iter().map(|(galley, _)| 8.0 + galley.size().y).sum();
                let height = 14.0 + table_h + notes_h + 16.0;
                let (area, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

                let rect = Rect::from_min_size(
                    pos2(area.left() + 16.0, area.top() + 14.0),
                    vec2(inner, table_h),
                );
                draw::outlined(ui, rect, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
                let left = rect.left() + 14.0;
                let right = rect.right() - 12.0;
                let jobs = (right - JOBS_COLUMN, right);
                let requests = (jobs.0 - 12.0 - REQUESTS_COLUMN, jobs.0 - 12.0);
                let name_left = left + STAMP_COLUMN + 8.0;
                let name_w = (requests.0 - 12.0 - name_left).max(0.0);

                // Header: 26 px, 11.5 px weight 600 in ink-2, line-2 underneath.
                let head_bottom = rect.top() + size::TABLE_HEAD_H;
                draw::hline(
                    ui,
                    rect.left() + 1.0..rect.right() - 1.0,
                    head_bottom - 1.0,
                    pal.line_2,
                );
                let mid = rect.top() + size::TABLE_HEAD_H / 2.0;
                let headers = [
                    (
                        crate::strings::COL_MODULE,
                        pos2(name_left, mid),
                        Align2::LEFT_CENTER,
                    ),
                    (
                        strings::COL_REQUESTS,
                        pos2(requests.1, mid),
                        Align2::RIGHT_CENTER,
                    ),
                    (strings::COL_JOBS, pos2(jobs.1, mid), Align2::RIGHT_CENTER),
                ];
                for (label, at, align) in headers {
                    let cell = draw::paint_text(ui, at, align, label, ty::LABEL, pal.ink_2);
                    draw::announce(ui, cell, Role::ColumnHeader, label);
                }

                // Rows: 28 px, a hairline between them. Figures are right aligned.
                for (i, (code, name, rates)) in rows.iter().enumerate() {
                    let top = head_bottom + i as f32 * size::TABLE_ROW_H;
                    let mid = top + size::TABLE_ROW_H / 2.0;
                    if i + 1 < rows.len() {
                        draw::hline(
                            ui,
                            rect.left() + 1.0..rect.right() - 1.0,
                            top + size::TABLE_ROW_H - 1.0,
                            pal.line,
                        );
                    }
                    if let Some(code) = code {
                        let at = pos2(left, (mid - size::STAMP_H / 2.0).round());
                        let stamp = draw::stamp(ui, pal, at, code, StampState::default());
                        draw::announce(ui, stamp, Role::Cell, code);
                    }
                    let (face, color) = match code {
                        Some(_) => (Face::Sans600, pal.ink),
                        None => (Face::Sans400, pal.ink_2),
                    };
                    let galley =
                        draw::galley_elided(ui, name, ty::BODY.with_face(face), color, name_w);
                    let cell =
                        draw::paint_galley(ui, pos2(name_left, mid), Align2::LEFT_CENTER, galley);
                    draw::announce(ui, cell, Role::Cell, name);
                    let cells = [
                        (figure(rates.requests_per_minute), requests.1),
                        (figure(rates.jobs_per_minute), jobs.1),
                    ];
                    for (text, x) in cells {
                        let cell = draw::paint_text(
                            ui,
                            pos2(x, mid),
                            Align2::RIGHT_CENTER,
                            &text,
                            ty::BODY,
                            pal.ink,
                        );
                        draw::announce(ui, cell, Role::Cell, &text);
                    }
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
}
