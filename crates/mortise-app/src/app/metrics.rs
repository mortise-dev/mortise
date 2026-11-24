//! Metrics: what the Metrics tab of the app on screen shows (design D4, D7).

use std::sync::Arc;

use mortise_core::{Answer, AppSnapshot, Figures, NotKept, Range, Scope};

use super::{Connected, MortiseApp};
use crate::app::workspace::Tab;
use crate::metrics::view::MetricsView;

/// The figures the tab drew last, with what they were worked out from.
pub(super) struct Kept {
    serial: u64,
    app: Arc<AppSnapshot>,
    answer: Arc<Answer>,
    figures: Figures,
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
    app: &Arc<AppSnapshot>,
    name: &'a str,
) -> MetricsView<'a> {
    let Some(connected) = connected else {
        return MetricsView::NotConnected { app: name };
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
            // Worked out again only after a read, or for another snapshot or answer.
            let kept = match connected.figures.take() {
                Some(kept)
                    if kept.serial == state.serial
                        && Arc::ptr_eq(&kept.app, app)
                        && Arc::ptr_eq(&kept.answer, &answer) =>
                {
                    kept
                }
                _ => Kept {
                    serial: state.serial,
                    app: Arc::clone(app),
                    answer: Arc::clone(&answer),
                    figures: connected.metrics.figures(
                        app,
                        runtime.routes.as_deref(),
                        Scope::All,
                        Range::Hour,
                    ),
                },
            };
            MetricsView::Table {
                environment,
                placed: runtime.routes.is_some(),
                figures: &connected.figures.insert(kept).figures,
            }
        }
    }
}
