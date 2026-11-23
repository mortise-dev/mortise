//! An environment that is kept fresh: asked on a worker thread, again and again.
//!
//! `health` is asked every 15 s while the window is visible. Everything else is asked
//! when the environment is opened, every 5 minutes, and when the user reads the app again.
//! `prometheus` is asked every 15 s only while the Metrics tab is open and the window is visible,
//! after `health` when both are due. One request is on its way at a time.

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::app::AppSnapshot;
use crate::modules::module::{Module, artisan_command};
use crate::runtime::actuator::{
    Answer, NoMetrics, Route, Secret, Task, ask, ask_metrics, ask_status,
};
use crate::runtime::environment::Environment;

/// How often things are asked.
#[derive(Debug, Clone, Copy)]
pub struct Pace {
    /// Between two requests for `health`.
    pub status: Duration,
    /// Between two requests for everything.
    pub everything: Duration,
    /// The window counts as visible this long after the interface last drew itself.
    /// The Metrics tab counts as open as long after it last drew itself.
    pub seen: Duration,
    /// Between two requests for `prometheus`.
    pub metrics: Duration,
}

impl Default for Pace {
    fn default() -> Self {
        Self {
            status: Duration::from_secs(15),
            everything: Duration::from_secs(5 * 60),
            seen: Duration::from_secs(3),
            metrics: Duration::from_secs(15),
        }
    }
}

/// What a [`LiveRuntime`] holds at this moment.
#[derive(Debug, Clone, Default)]
pub struct RuntimeState {
    /// The last answer. `None` until the first one is there.
    pub answer: Option<Arc<Answer>>,
    /// When the last answer came.
    pub checked: Option<Instant>,
    /// True while a request is on its way.
    pub asking: bool,
}

struct Shared {
    state: RuntimeState,
    /// Everything is asked for at the next turn.
    refresh: bool,
    /// When the interface last drew itself.
    seen: Instant,
    /// When the Metrics tab last drew itself. `None` until it does.
    metrics_seen: Option<Instant>,
    stop: bool,
}

/// An environment with a worker thread that asks it.
pub struct LiveRuntime {
    shared: Arc<(Mutex<Shared>, Condvar)>,
}

impl LiveRuntime {
    /// Starts asking the environment. `on_change` is called from the worker thread
    /// after every answer.
    pub fn open(
        environment: Environment,
        secret: Secret,
        on_change: impl Fn() + Send + 'static,
    ) -> Self {
        Self::open_with(environment, secret, Pace::default(), on_change)
    }

    /// As [`LiveRuntime::open`], at another pace. Tests use it.
    pub fn open_with(
        environment: Environment,
        secret: Secret,
        pace: Pace,
        on_change: impl Fn() + Send + 'static,
    ) -> Self {
        Self::open_with_metrics(environment, secret, pace, on_change, |_| {})
    }

    /// As [`LiveRuntime::open_with`], and hands every answer of `prometheus` to `on_metrics`,
    /// on the worker thread, before `on_change` is called.
    pub fn open_with_metrics(
        environment: Environment,
        secret: Secret,
        pace: Pace,
        on_change: impl Fn() + Send + 'static,
        mut on_metrics: impl FnMut(Result<String, NoMetrics>) + Send + 'static,
    ) -> Self {
        let shared = Arc::new((
            Mutex::new(Shared {
                state: RuntimeState {
                    asking: true,
                    ..RuntimeState::default()
                },
                refresh: true,
                seen: Instant::now(),
                metrics_seen: None,
                stop: false,
            }),
            Condvar::new(),
        ));
        let worker = Arc::clone(&shared);
        std::thread::Builder::new()
            .name("mortise-runtime".to_owned())
            .spawn(move || {
                run(
                    &worker,
                    &environment,
                    &secret,
                    pace,
                    &on_change,
                    &mut on_metrics,
                );
            })
            .expect("a thread can be started");
        Self { shared }
    }

    /// The current state. Cheap: an answer is shared, not copied.
    pub fn state(&self) -> RuntimeState {
        self.lock().state.clone()
    }

    /// Asks for everything again, now.
    pub fn refresh(&self) {
        self.lock().refresh = true;
        self.shared.1.notify_all();
    }

    /// Tells the worker that the interface drew itself: the window is visible.
    /// Without this the worker stops asking.
    pub fn seen(&self) {
        self.lock().seen = Instant::now();
        self.shared.1.notify_all();
    }

    /// Tells the worker that the Metrics tab drew itself. Without this it does not ask for
    /// `prometheus`.
    pub fn metrics_seen(&self) {
        self.lock().metrics_seen = Some(Instant::now());
        self.shared.1.notify_all();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared
            .0
            .lock()
            .expect("the runtime state is never poisoned")
    }
}

impl Drop for LiveRuntime {
    fn drop(&mut self) {
        self.lock().stop = true;
        self.shared.1.notify_all();
    }
}

fn run(
    shared: &(Mutex<Shared>, Condvar),
    environment: &Environment,
    secret: &Secret,
    pace: Pace,
    on_change: &dyn Fn(),
    on_metrics: &mut dyn FnMut(Result<String, NoMetrics>),
) {
    let (lock, wake) = shared;
    let mut last_status: Option<Instant> = None;
    let mut last_everything: Option<Instant> = None;
    // When the last read of `prometheus` ended.
    let mut last_metrics: Option<Instant> = None;
    loop {
        // Wait until something is due.
        let mut metrics = false;
        let (everything, earlier) = {
            let mut shared = lock.lock().expect("the runtime state is never poisoned");
            loop {
                if shared.stop {
                    return;
                }
                let visible = shared.seen.elapsed() < pace.seen;
                let answered = matches!(shared.state.answer.as_deref(), Some(Answer::Answered(_)));
                let due = |last: Option<Instant>, every: Duration| {
                    last.is_none_or(|last| last.elapsed() >= every)
                };
                if shared.refresh {
                    // Metrics are read again too, right after.
                    last_metrics = None;
                    break;
                }
                if visible && due(last_status, pace.status) {
                    // Without an answer to add the status to, everything is asked for.
                    if !answered || due(last_everything, pace.everything) {
                        shared.refresh = true;
                    }
                    break;
                }
                // `prometheus` comes after `health`, while the Metrics tab is open and the
                // last answer lists it.
                let tab = visible
                    && shared
                        .metrics_seen
                        .is_some_and(|seen| seen.elapsed() < pace.seen)
                    && matches!(shared.state.answer.as_deref(),
                        Some(Answer::Answered(runtime)) if runtime.exposes("prometheus"));
                if tab && due(last_metrics, pace.metrics) {
                    metrics = true;
                    break;
                }
                // Sleep until the next request is due, or until the window is seen again.
                let nap = if visible {
                    last_status.map_or(Duration::ZERO, |last| {
                        pace.status.saturating_sub(last.elapsed())
                    })
                } else {
                    Duration::from_secs(3600)
                };
                // Or until metrics are due.
                let nap = match last_metrics {
                    Some(last) if tab => nap.min(pace.metrics.saturating_sub(last.elapsed())),
                    _ => nap,
                };
                shared = wake
                    .wait_timeout(shared, nap.max(Duration::from_millis(5)))
                    .expect("the runtime state is never poisoned")
                    .0;
            }
            if metrics {
                (false, None)
            } else {
                let everything = std::mem::take(&mut shared.refresh);
                shared.state.asking = true;
                (everything, shared.state.answer.clone())
            }
        };
        if metrics {
            // A read of metrics leaves the state of the runtime as it is.
            on_metrics(ask_metrics(environment, secret));
            last_metrics = Some(Instant::now());
            on_change();
            continue;
        }
        on_change();

        let answer = match earlier.as_deref() {
            Some(Answer::Answered(earlier)) if !everything => {
                ask_status(environment, secret, earlier)
            }
            _ => ask(environment, secret),
        };
        let now = Instant::now();
        last_status = Some(now);
        if everything {
            last_everything = Some(now);
        }
        {
            let mut shared = lock.lock().expect("the runtime state is never poisoned");
            shared.state = RuntimeState {
                answer: Some(Arc::new(answer)),
                checked: Some(now),
                asking: false,
            };
        }
        on_change();
    }
}

/// The module a class belongs to: the one whose namespace the class name starts with.
pub fn module_of<'a>(app: &'a AppSnapshot, class: &str) -> Option<&'a Module> {
    let class = class.trim_start_matches('\\');
    app.modules.iter().find(|module| {
        let namespace = module.namespace.trim_matches('\\');
        !namespace.is_empty()
            && class
                .strip_prefix(namespace)
                .is_some_and(|rest| rest.starts_with('\\'))
    })
}

/// The routes of one module: those whose action is a class of the module.
pub fn routes_of<'a>(app: &AppSnapshot, module: &str, routes: &'a [Route]) -> Vec<&'a Route> {
    routes
        .iter()
        .filter(|route| module_of(app, &route.action).is_some_and(|found| found.name == module))
        .collect()
}

/// How many routes each module has, most first, and how many are in no module:
/// closures and the routes of other packages.
pub fn routes_by_module(app: &AppSnapshot, routes: &[Route]) -> (Vec<(String, usize)>, usize) {
    let mut counts: Vec<(String, usize)> = Vec::new();
    let mut outside = 0;
    for route in routes {
        match module_of(app, &route.action) {
            Some(module) => match counts.iter_mut().find(|(name, _)| *name == module.name) {
                Some((_, count)) => *count += 1,
                None => counts.push((module.name.clone(), 1)),
            },
            None => outside += 1,
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    (counts, outside)
}

/// The module a scheduled task belongs to: the module of its class, or the module
/// that declares the command of `php artisan <name>`. A task with a description, a
/// closure or a shell command belongs to none.
///
/// When the package lists the tasks of each module (P10), its list wins here.
pub fn module_of_task<'a>(app: &'a AppSnapshot, task: &Task) -> Option<&'a Module> {
    module_of(app, &task.target).or_else(|| {
        let name = artisan_command(&task.target)?;
        app.modules
            .iter()
            .find(|module| module.commands.iter().any(|command| command == name))
    })
}

/// The scheduled tasks of one module.
pub fn tasks_of<'a>(app: &AppSnapshot, module: &str, tasks: &'a [Task]) -> Vec<&'a Task> {
    tasks
        .iter()
        .filter(|task| module_of_task(app, task).is_some_and(|found| found.name == module))
        .collect()
}
