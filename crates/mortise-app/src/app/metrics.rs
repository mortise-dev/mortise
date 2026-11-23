//! Metrics: what the Metrics tab of the app on screen shows (design D4, D7).

use std::sync::Arc;

use mortise_core::{Answer, AppSnapshot, Figures, LAST_MINUTE, NotKept};

use super::Connected;
use crate::metrics::view::MetricsView;

/// The figures the tab drew last, with what they were worked out from.
pub(super) struct Kept {
    serial: u64,
    app: Arc<AppSnapshot>,
    answer: Arc<Answer>,
    figures: Figures,
}

/// What the Metrics tab shows in this frame, first match wins. Call it only in a frame that
/// draws the tab: it tells the worker that the tab is open.
pub(super) fn tab<'a>(
    connected: Option<&'a mut Connected>,
    app: &Arc<AppSnapshot>,
    name: &'a str,
) -> MetricsView<'a> {
    let Some(connected) = connected else {
        return MetricsView::NotConnected { app: name };
    };
    // The worker reads `prometheus` only while the tab draws itself.
    connected.live.metrics_seen();
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
        None => MetricsView::Waiting { environment },
        Some(Err(NotKept::NoMetrics(why))) => MetricsView::NoMetrics { environment, why },
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
                    figures: connected
                        .metrics
                        .figures(app, runtime.routes.as_deref(), LAST_MINUTE),
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
