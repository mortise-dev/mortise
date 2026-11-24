//! Metrics: what the Metrics tab of the app on screen shows (design D4, D7).

use std::sync::Arc;
use std::time::Duration;

use mortise_core::{Answer, AppSnapshot, Environment, Figures, NotKept, Range, Scope};

use super::{Connected, MortiseApp};
use crate::app::workspace::Tab;
use crate::metrics::view::{MetricsTab, MetricsView, TableView};

/// The figures the tab drew last, with what they were worked out from.
pub(super) struct Kept {
    serial: u64,
    app: Arc<AppSnapshot>,
    answer: Arc<Answer>,
    /// The module of the scope, `None` for all modules.
    module: Option<String>,
    range: Range,
    figures: Figures,
}

/// What the Metrics tab is drawn from, besides the environment.
pub(super) struct TabInputs<'a> {
    /// The display name of the app.
    pub name: &'a str,
    pub environments: &'a [Environment],
    /// The module selected in the index.
    pub module: Option<&'a str>,
    /// How often metrics are read.
    pub pace: Duration,
}

impl MortiseApp {
    /// Tells each environment whether its Metrics tab is open, from the state at the end of the
    /// frame: the workspace of the app was drawn, nothing took its place since, and its tab is
    /// still Metrics. The worker reads `prometheus` only while the tab is open. Settings, the add
    /// panel, another app or another tab close it, in the frame they come.
    pub(super) fn follow_metrics(&self, drawn: Option<usize>) {
        let shown = drawn.filter(|&index| {
            self.current == Some(index) && self.adding.is_none() && self.settings_open.is_none()
        });
        for (index, open) in self.apps.iter().enumerate() {
            let Some(connected) = &open.runtime else {
                continue;
            };
            if shown == Some(index) && open.tab == Tab::Metrics {
                connected.live.metrics_seen();
            } else {
                connected.live.metrics_closed();
            }
        }
    }
}

/// What the Metrics tab shows in this frame, first match wins.
pub(super) fn tab<'a>(
    connected: Option<&'a mut Connected>,
    metrics: &mut MetricsTab,
    app: &Arc<AppSnapshot>,
    inputs: TabInputs<'a>,
) -> MetricsView<'a> {
    let Some(connected) = connected else {
        return MetricsView::NotConnected { app: inputs.name };
    };
    let environment = connected.environment.name.as_str();
    let Some(answer) = connected.live.state().answer else {
        return MetricsView::Waiting { environment };
    };
    let Answer::Answered(runtime) = answer.as_ref() else {
        return MetricsView::Unreached {
            environment,
            answer,
        };
    };
    if !runtime.exposes("prometheus") {
        return MetricsView::NotExposed { environment };
    }
    let state = connected.metrics.state();
    match state.last {
        // Also while the newest read is too old to give the rates of now.
        None => MetricsView::Waiting { environment },
        Some(Err(NotKept::NoMetrics(why))) => MetricsView::NoMetrics { environment, why },
        Some(Err(NotKept::NotMetrics)) => MetricsView::NotMetrics { environment },
        Some(Err(NotKept::TooManySeries { limit })) => MetricsView::TooMany { environment, limit },
        Some(Ok(())) => {
            let scope = inputs.module.map_or(Scope::All, Scope::Module);
            // Worked out again only after a read, or for another snapshot, answer, scope or range.
            let kept = match connected.figures.take() {
                Some(kept)
                    if kept.serial == state.serial
                        && Arc::ptr_eq(&kept.app, app)
                        && Arc::ptr_eq(&kept.answer, &answer)
                        && kept.module.as_deref() == inputs.module
                        && kept.range == metrics.range =>
                {
                    kept
                }
                _ => Kept {
                    serial: state.serial,
                    app: Arc::clone(app),
                    answer: Arc::clone(&answer),
                    module: inputs.module.map(str::to_owned),
                    range: metrics.range,
                    figures: connected.metrics.figures(
                        app,
                        runtime.routes.as_deref(),
                        scope,
                        metrics.range,
                    ),
                },
            };
            let figures = &connected.figures.insert(kept).figures;
            // A row that is gone is no longer selected.
            let present = |of| {
                figures
                    .rows
                    .iter()
                    .chain(&figures.outside)
                    .any(|row| &row.of == of)
            };
            if metrics.row.as_ref().is_some_and(|of| !present(of)) {
                metrics.row = None;
            }
            MetricsView::Table(TableView {
                environment,
                environments: inputs
                    .environments
                    .iter()
                    .map(|environment| environment.name.as_str())
                    .collect(),
                figures,
                placed: runtime.routes.is_some(),
                scope,
                range: metrics.range,
                row: metrics.row.clone(),
                pace: inputs.pace,
                ago: state.ago.unwrap_or_default(),
            })
        }
    }
}
