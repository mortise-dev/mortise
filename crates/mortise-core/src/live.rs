//! An app that is kept fresh: read on a worker thread, and read again when its files change.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};

use crate::app::{AppSnapshot, ReadError, read_app_with};

/// How long file changes are collected before the app is read again.
const QUIET: Duration = Duration::from_millis(300);

/// What a [`LiveApp`] holds at this moment.
#[derive(Debug, Clone)]
pub enum LiveState {
    /// The first read has not ended.
    Reading,
    /// The last read. It stays until a later read ends.
    Ready(Arc<AppSnapshot>),
    /// The last read failed.
    Failed(Arc<ReadError>),
}

enum Command {
    /// Read again now.
    Refresh,
    /// Watch the files of the app, or stop watching.
    Watch(bool),
    Stop,
}

/// An app folder with a worker thread that reads it.
///
/// The interface never changes a snapshot: it asks for the current one with
/// [`LiveApp::state`] and is told through `on_change` that a newer one is there.
pub struct LiveApp {
    state: Arc<Mutex<LiveState>>,
    commands: Sender<Command>,
}

impl LiveApp {
    /// Starts reading the app in `root`. `codes` are the codes its modules already
    /// have. `on_change` is called from the worker thread after every read.
    pub fn open(
        root: PathBuf,
        codes: BTreeMap<String, String>,
        on_change: impl Fn() + Send + 'static,
    ) -> Self {
        let state = Arc::new(Mutex::new(LiveState::Reading));
        let (commands, inbox) = channel();
        let worker = Worker {
            root,
            codes,
            state: Arc::clone(&state),
            inbox,
            // The watcher reports into the same inbox.
            outbox: commands.clone(),
            on_change: Box::new(on_change),
            watcher: None,
            watching: false,
        };
        std::thread::Builder::new()
            .name("mortise-app-reader".to_owned())
            .spawn(move || worker.run())
            .expect("a thread can be started");
        Self { state, commands }
    }

    /// The current state. Cheap: a snapshot is shared, not copied.
    pub fn state(&self) -> LiveState {
        self.state
            .lock()
            .expect("the state is never poisoned")
            .clone()
    }

    /// Reads the app again, now.
    pub fn refresh(&self) {
        let _ = self.commands.send(Command::Refresh);
    }

    /// Starts or stops reading the app again when its files change.
    pub fn watch(&self, on: bool) {
        let _ = self.commands.send(Command::Watch(on));
    }
}

impl Drop for LiveApp {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
    }
}

struct Worker {
    root: PathBuf,
    codes: BTreeMap<String, String>,
    state: Arc<Mutex<LiveState>>,
    inbox: Receiver<Command>,
    outbox: Sender<Command>,
    on_change: Box<dyn Fn() + Send>,
    watcher: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    watching: bool,
}

impl Worker {
    fn run(mut self) {
        self.read();
        while let Ok(command) = self.inbox.recv() {
            match command {
                Command::Refresh => self.read(),
                Command::Watch(on) => {
                    self.watching = on;
                    self.watcher = None;
                    if on {
                        self.start_watching();
                    }
                }
                Command::Stop => break,
            }
        }
    }

    fn read(&mut self) {
        let state = match read_app_with(&self.root, &self.codes) {
            Ok(app) => {
                // A code is kept once given, also for a module that is gone for a while.
                self.codes.extend(app.codes());
                LiveState::Ready(Arc::new(app))
            }
            Err(error) => LiveState::Failed(Arc::new(error)),
        };
        // The modules folder can move when the config changes.
        let moved = match (&state, &self.watcher) {
            (LiveState::Ready(app), Some(_)) => !self.watched().contains(&app.layout.modules_dir),
            _ => false,
        };
        *self.state.lock().expect("the state is never poisoned") = state;
        if moved {
            self.watcher = None;
            self.start_watching();
        }
        (self.on_change)();
    }

    /// The folders to watch: the app folder and its config folder for the few files
    /// Mortise reads there, and the whole modules folder.
    fn watched(&self) -> Vec<PathBuf> {
        let modules_dir = match &*self.state.lock().expect("the state is never poisoned") {
            LiveState::Ready(app) => app.layout.modules_dir.clone(),
            _ => self.root.join("Modules"),
        };
        vec![self.root.clone(), self.root.join("config"), modules_dir]
    }

    fn start_watching(&mut self) {
        let outbox = self.outbox.clone();
        let root = self.root.clone();
        let handler = move |result: DebounceEventResult| {
            let relevant = result.is_ok_and(|events| {
                events
                    .iter()
                    .any(|event| event.paths.iter().any(|path| is_relevant(&root, path)))
            });
            if relevant {
                let _ = outbox.send(Command::Refresh);
            }
        };
        let Ok(mut watcher) = new_debouncer(QUIET, None, handler) else {
            return;
        };
        let [root, config, modules] = &self.watched()[..] else {
            return;
        };
        // A folder that does not exist cannot be watched. That is not an error here:
        // the app folder is watched, so its creation is seen.
        let _ = watcher.watch(root, RecursiveMode::NonRecursive);
        let _ = watcher.watch(config, RecursiveMode::NonRecursive);
        let _ = watcher.watch(modules, RecursiveMode::Recursive);
        self.watcher = Some(watcher);
    }
}

/// False for what Mortise never reads and what changes in bulk: installed packages,
/// git's own files, logs and caches. Only the part of the path inside the app counts.
fn is_relevant(root: &Path, path: &Path) -> bool {
    let inside = path.strip_prefix(root).unwrap_or(path);
    !inside.components().any(|part| {
        matches!(
            part,
            Component::Normal(name)
                if ["vendor", "node_modules", ".git", "storage"].iter().any(|skip| name == *skip)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_packages_and_git_files_are_not_relevant() {
        let root = Path::new("/home/storage/shop-api");
        let relevant = |path: &str| is_relevant(root, &root.join(path));
        assert!(relevant("modules_statuses.json"));
        assert!(relevant("Modules/Blog/app/Models/Post.php"));
        assert!(!relevant("vendor/composer/installed.json"));
        assert!(!relevant("Modules/Blog/node_modules/x/index.js"));
        assert!(!relevant(".git/index"));
        assert!(!relevant("storage/logs/laravel.log"));
    }
}
