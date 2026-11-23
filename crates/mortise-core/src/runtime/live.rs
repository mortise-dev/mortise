//! An environment that is kept fresh: asked on a worker thread, again and again.
//!
//! `health` is asked every 15 s while the window is visible. Everything else is asked
//! when the environment is opened, every 5 minutes, and when the user reads the app again.

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::app::AppSnapshot;
use crate::modules::module::Module;
use crate::runtime::actuator::{Answer, Route, Secret, Task, ask, ask_status};
use crate::runtime::environment::Environment;

/// How often things are asked.
#[derive(Debug, Clone, Copy)]
pub struct Pace {
    /// Between two requests for `health`.
    pub status: Duration,
    /// Between two requests for everything.
    pub everything: Duration,
    /// The window counts as visible this long after the interface last drew itself.
    pub seen: Duration,
}

impl Default for Pace {
    fn default() -> Self {
        Self {
            status: Duration::from_secs(15),
            everything: Duration::from_secs(5 * 60),
            seen: Duration::from_secs(3),
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
        let shared = Arc::new((
            Mutex::new(Shared {
                state: RuntimeState {
                    asking: true,
                    ..RuntimeState::default()
                },
                refresh: true,
                seen: Instant::now(),
                stop: false,
            }),
            Condvar::new(),
        ));
        let worker = Arc::clone(&shared);
        std::thread::Builder::new()
            .name("mortise-runtime".to_owned())
            .spawn(move || run(&worker, &environment, &secret, pace, &on_change))
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
) {
    let (lock, wake) = shared;
    let mut last_status: Option<Instant> = None;
    let mut last_everything: Option<Instant> = None;
    loop {
        // Wait until something is due.
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
                    break;
                }
                if visible && due(last_status, pace.status) {
                    // Without an answer to add the status to, everything is asked for.
                    if !answered || due(last_everything, pace.everything) {
                        shared.refresh = true;
                    }
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
                shared = wake
                    .wait_timeout(shared, nap.max(Duration::from_millis(5)))
                    .expect("the runtime state is never poisoned")
                    .0;
            }
            let everything = std::mem::take(&mut shared.refresh);
            shared.state.asking = true;
            (everything, shared.state.answer.clone())
        };
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

/// The scheduled tasks of one module. Only a task whose target is a class name can
/// be given to a module.
pub fn tasks_of<'a>(app: &AppSnapshot, module: &str, tasks: &'a [Task]) -> Vec<&'a Task> {
    tasks
        .iter()
        .filter(|task| module_of(app, &task.target).is_some_and(|found| found.name == module))
        .collect()
}
