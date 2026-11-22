//! An app that is kept fresh: read on a worker thread, and read again when its files change.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};

use crate::app::{AppSnapshot, ReadError, ReadOptions, read_app_cached};
use crate::module::{Progress, ReadCache};

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
    /// Read everything again now, also the files that did not change.
    Refresh,
    /// Files changed: read again, but only the files that changed.
    Changed,
    /// Watch the files of the app, or stop watching.
    Watch(bool),
    /// Read with other options from now on, and read again.
    Options(ReadOptions),
    Stop,
}

/// An app folder with a worker thread that reads it.
///
/// The interface never changes a snapshot: it asks for the current one with
/// [`LiveApp::state`] and is told through `on_change` that a newer one is there.
pub struct LiveApp {
    state: Arc<Mutex<LiveState>>,
    watching: Arc<AtomicBool>,
    commands: Sender<Command>,
    progress: Arc<Progress>,
}

impl LiveApp {
    /// Starts reading the app in `root`. `codes` are the codes its modules already
    /// have. `on_change` is called from the worker thread after every read.
    pub fn open(
        root: PathBuf,
        codes: BTreeMap<String, String>,
        options: ReadOptions,
        on_change: impl Fn() + Send + 'static,
    ) -> Self {
        let state = Arc::new(Mutex::new(LiveState::Reading));
        let watching = Arc::new(AtomicBool::new(false));
        let (commands, inbox) = channel();
        let cache = ReadCache::default();
        let progress = cache.progress();
        let worker = Worker {
            root,
            codes,
            options,
            cache,
            state: Arc::clone(&state),
            inbox,
            // The watcher reports into the same inbox.
            outbox: commands.clone(),
            on_change: Box::new(on_change),
            watcher: None,
            watched: Vec::new(),
            watching: Arc::clone(&watching),
        };
        std::thread::Builder::new()
            .name("mortise-app-reader".to_owned())
            .spawn(move || worker.run())
            .expect("a thread can be started");
        Self {
            state,
            watching,
            commands,
            progress,
        }
    }

    /// How far the read that is going on is: the PHP files read, and how many there are.
    pub fn progress(&self) -> (usize, usize) {
        self.progress.get()
    }

    /// True once the watcher that [`LiveApp::watch`] asked for is in place.
    pub fn is_watching(&self) -> bool {
        self.watching.load(Ordering::Relaxed)
    }

    /// The current state. Cheap: a snapshot is shared, not copied.
    pub fn state(&self) -> LiveState {
        self.state
            .lock()
            .expect("the state is never poisoned")
            .clone()
    }

    /// Reads the app again, now, also the files that did not change.
    pub fn refresh(&self) {
        let _ = self.commands.send(Command::Refresh);
    }

    /// Changes what the user set for the app, and reads it again.
    pub fn set_options(&self, options: ReadOptions) {
        let _ = self.commands.send(Command::Options(options));
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

/// A folder to watch, and whether the folders inside it are watched too.
type Watched = (PathBuf, RecursiveMode);

struct Worker {
    root: PathBuf,
    codes: BTreeMap<String, String>,
    options: ReadOptions,
    cache: ReadCache,
    state: Arc<Mutex<LiveState>>,
    inbox: Receiver<Command>,
    outbox: Sender<Command>,
    on_change: Box<dyn Fn() + Send>,
    watcher: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    /// What the watcher watches. Empty while it is off.
    watched: Vec<Watched>,
    watching: Arc<AtomicBool>,
}

impl Worker {
    fn run(mut self) {
        self.read();
        while let Ok(command) = self.inbox.recv() {
            match command {
                Command::Refresh => {
                    self.cache.clear();
                    self.read();
                }
                Command::Changed => self.read(),
                Command::Options(options) => {
                    self.options = options;
                    self.read();
                }
                Command::Watch(true) => self.start_watching(),
                Command::Watch(false) => {
                    self.watcher = None;
                    self.watched.clear();
                    self.watching.store(false, Ordering::Relaxed);
                }
                Command::Stop => break,
            }
        }
    }

    fn read(&mut self) {
        let state = match read_app_cached(&self.root, &self.codes, &self.options, &mut self.cache) {
            Ok(app) => {
                // A code is kept once given, also for a module that is gone for a while.
                self.codes.extend(app.codes());
                LiveState::Ready(Arc::new(app))
            }
            Err(error) => LiveState::Failed(Arc::new(error)),
        };
        *self.state.lock().expect("the state is never poisoned") = state;
        // The modules folder or the coverage report can move when a config changes.
        if self.watcher.is_some() && self.wanted() != self.watched {
            self.start_watching();
        }
        (self.on_change)();
    }

    /// The statuses file the config names, or the default one before the first read.
    fn statuses_file(&self) -> PathBuf {
        match &*self.state.lock().expect("the state is never poisoned") {
            LiveState::Ready(app) => app.layout.statuses_file.clone(),
            _ => self.root.join("modules_statuses.json"),
        }
    }

    /// What should be watched: the app folder and its config folder for the few files
    /// Mortise reads there, the whole modules folder, the folders of the statuses file
    /// and of the coverage report, and git's folder for a new commit or another branch.
    fn wanted(&self) -> Vec<Watched> {
        let (modules_dir, report) = match &*self.state.lock().expect("the state is never poisoned")
        {
            LiveState::Ready(app) => (
                app.layout.modules_dir.clone(),
                app.coverage_report
                    .as_ref()
                    .map(|report| report.path.clone()),
            ),
            _ => (self.root.join("Modules"), None),
        };
        let mut wanted = vec![
            (self.root.clone(), RecursiveMode::NonRecursive),
            (self.root.join("config"), RecursiveMode::NonRecursive),
            (self.root.join(".git"), RecursiveMode::NonRecursive),
            // A tool that uses libgit2 changes only the refs and this log on a commit.
            // When `.git` is a file, as in a worktree, this folder is not there and
            // is not watched.
            (
                self.root.join(".git").join("logs"),
                RecursiveMode::NonRecursive,
            ),
            (modules_dir, RecursiveMode::Recursive),
        ];
        let statuses = self.statuses_file();
        if let Some(folder) = statuses.parent() {
            let covered = wanted.iter().any(|(watched, mode)| {
                watched == folder
                    || (*mode == RecursiveMode::Recursive && folder.starts_with(watched))
            });
            if !covered {
                wanted.push((folder.to_owned(), RecursiveMode::NonRecursive));
            }
        }
        if let Some(folder) = report.as_deref().and_then(Path::parent) {
            wanted.push((folder.to_owned(), RecursiveMode::NonRecursive));
        }
        wanted
    }

    fn start_watching(&mut self) {
        self.watcher = None;
        self.watched = self.wanted();

        let outbox = self.outbox.clone();
        let root = self.root.clone();
        let statuses = self.statuses_file();
        // The statuses file counts wherever it is, also in `storage`. On macOS the
        // watcher reports paths without symbolic links, so that form counts too.
        let real_statuses = statuses
            .parent()
            .and_then(|folder| std::fs::canonicalize(folder).ok())
            .zip(statuses.file_name())
            .map(|(folder, name)| folder.join(name));
        let statuses: Vec<PathBuf> = std::iter::once(statuses).chain(real_statuses).collect();
        let handler = move |result: DebounceEventResult| {
            let relevant = result.is_ok_and(|events| {
                events.iter().any(|event| {
                    event
                        .paths
                        .iter()
                        .any(|path| statuses.contains(path) || is_relevant(&root, path))
                })
            });
            if relevant {
                let _ = outbox.send(Command::Changed);
            }
        };
        let Ok(mut watcher) = new_debouncer(QUIET, None, handler) else {
            return;
        };
        for (folder, mode) in &self.watched {
            // A folder that does not exist cannot be watched. That is not an error here:
            // the app folder is watched, so its creation is seen.
            let _ = watcher.watch(folder, *mode);
        }
        self.watcher = Some(watcher);
        self.watching.store(true, Ordering::Relaxed);
    }
}

/// False for what Mortise never reads and what changes in bulk: installed packages,
/// logs and caches, and git's own files except the few that change with a commit or
/// a checkout. Only the part of the path inside the app counts.
fn is_relevant(root: &Path, path: &Path) -> bool {
    let inside = path.strip_prefix(root).unwrap_or(path);
    let mut parts = inside.components().filter_map(|part| match part {
        Component::Normal(name) => name.to_str(),
        _ => None,
    });
    if inside.starts_with(".git") {
        // The index is rewritten by every `git status` of every editor: not that.
        let git: Vec<&str> = parts.skip(1).collect();
        return matches!(
            git.as_slice(),
            ["HEAD" | "COMMIT_EDITMSG" | "ORIG_HEAD" | "packed-refs"] | ["logs", "HEAD"]
        );
    }
    // A coverage report is a file Mortise reads, wherever it is written.
    if inside
        .extension()
        .is_some_and(|extension| extension == "xml")
    {
        return true;
    }
    !parts.any(|name| matches!(name, "vendor" | "node_modules" | ".git" | "storage"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_packages_logs_and_most_git_files_are_not_relevant() {
        let root = Path::new("/home/storage/shop-api");
        let relevant = |path: &str| is_relevant(root, &root.join(path));
        assert!(relevant("modules_statuses.json"));
        assert!(relevant("Modules/Blog/app/Models/Post.php"));
        assert!(relevant("build/coverage/clover.xml"));
        assert!(relevant("storage/coverage/clover.xml"));
        assert!(!relevant("vendor/composer/installed.json"));
        assert!(!relevant("Modules/Blog/node_modules/x/index.js"));
        assert!(!relevant("storage/logs/laravel.log"));

        assert!(relevant(".git/HEAD"));
        assert!(relevant(".git/COMMIT_EDITMSG"));
        assert!(relevant(".git/logs/HEAD"));
        assert!(!relevant(".git/logs/refs/heads/main"));
        assert!(!relevant(".git/index"));
        assert!(!relevant(".git/index.lock"));
        assert!(!relevant(".git/objects/ab/cdef"));
    }
}
