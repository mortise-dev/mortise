//! Starting a program: the setup every program Mortise starts shares, and, with
//! the `run` feature, running one while its lines are read, and ending its whole
//! process tree.

#[cfg(feature = "run")]
use std::collections::BTreeMap;
#[cfg(feature = "run")]
use std::ffi::OsString;
#[cfg(feature = "run")]
use std::io::{BufRead, BufReader, Read};
#[cfg(feature = "run")]
use std::path::{Path, PathBuf};
#[cfg(feature = "run")]
use std::process::{Child, ExitStatus};
#[cfg(feature = "run")]
use std::process::{Command, Stdio};
#[cfg(feature = "run")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "run")]
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender};
#[cfg(feature = "run")]
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
#[cfg(feature = "run")]
use std::thread::JoinHandle;
#[cfg(feature = "run")]
use std::time::{Duration, Instant, SystemTime};

#[cfg(feature = "run")]
use super::history::OUTPUT_LIMIT;
#[cfg(feature = "run")]
use super::{CommandLine, Failure, PhpSettings};
#[cfg(feature = "run")]
use crate::program::command;

/// How long a loop waits for a line before it looks at the program again.
#[cfg(feature = "run")]
const TICK: Duration = Duration::from_millis(50);

/// How long the readers may still read after the program ended. A program that a
/// grandchild outlives can leave a pipe open.
#[cfg(feature = "run")]
const GRACE: Duration = Duration::from_millis(250);

/// How long the readers may still read after the kill of what is left of the
/// group, before they are left behind.
#[cfg(feature = "run")]
const DETACH: Duration = Duration::from_secs(1);

/// The longest the readers may go on sending lines after the program ended. A
/// child that writes without end is not waited for past it.
#[cfg(feature = "run")]
const DRAIN: Duration = Duration::from_secs(10);

/// How long a stopped program has to end after it was asked to, before it is killed.
#[cfg(feature = "run")]
const STOP_GRACE: Duration = Duration::from_secs(3);

/// The longest a stop takes to end a run, from [`Run::stop`] to [`RunEvent::Ended`]:
/// `STOP_GRACE` to the kill, `GRACE` and `DETACH` for the readers, and a margin
/// for a few `TICK`s and, on Windows, `taskkill`. A stop that comes while a
/// child that outlived the program still writes takes `STOP_GRACE` and `DETACH`.
#[cfg(feature = "run")]
pub const STOP_WAIT: Duration = STOP_GRACE
    .saturating_add(GRACE)
    .saturating_add(DETACH)
    .saturating_add(Duration::from_millis(500));

/// The most text kept of one line in bytes, and, by [`capture`], of one stream.
/// A newline counts as one byte, so a flood of empty lines is bounded too.
#[cfg(feature = "run")]
const KEEP: usize = 64 * 1024;

/// The pipe a line came from.
#[cfg(feature = "run")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Out,
    Err,
}

/// The commands that run now, one per app at most. A clone shares the same list.
#[cfg(feature = "run")]
#[derive(Debug, Clone, Default)]
pub struct Runs {
    /// The app folder, as given to [`Runs::start`], to the label of its command.
    running: Arc<Mutex<BTreeMap<PathBuf, String>>>,
}

#[cfg(feature = "run")]
impl Runs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts artisan with `args` through the runner of `php`, in `php.folder(app)`,
    /// for at most `limit`, which is `php.time_limit()` for a user's run. A second
    /// start in the same app is refused while the first runs.
    pub fn start(
        &self,
        app: &Path,
        php: &PhpSettings,
        args: &[String],
        limit: Duration,
    ) -> Result<Run, StartError> {
        self.start_with_input(app, php, args, limit, None)
    }

    /// [`Runs::start`], with `input` written to the standard input of the program,
    /// which then ends. Without input the standard input of the program is empty.
    pub fn start_with_input(
        &self,
        app: &Path,
        php: &PhpSettings,
        args: &[String],
        limit: Duration,
        input: Option<Vec<u8>>,
    ) -> Result<Run, StartError> {
        let label = label(args);
        let folder = php.folder(app);
        let line = {
            let mut running = self.lock();
            if let Some(command) = running.get(app) {
                return Err(StartError::Busy {
                    command: command.clone(),
                });
            }
            let runner = php.runner.as_ref().ok_or(StartError::NoRunner)?;
            let words: Vec<&str> = args.iter().map(String::as_str).collect();
            let line = CommandLine::artisan(runner, &words)
                .ok_or(StartError::Failed(Failure::NoCommand))?;
            if !folder.is_dir() {
                return Err(StartError::Failed(Failure::NoFolder { folder }));
            }
            running.insert(app.to_owned(), label.clone());
            line
        };
        let process = match Process::start(&line, &folder, input) {
            Ok(process) => process,
            Err(error) => {
                self.lock().remove(app);
                return Err(StartError::Failed(not_started(error, &line.program)));
            }
        };
        let started = Instant::now();
        let stop = Arc::new(AtomicBool::new(false));
        let (sender, events) = mpsc::channel();
        let owner = Owner {
            process,
            started,
            limit,
            stop: Arc::clone(&stop),
            events: sender,
            runs: self.clone(),
            app: app.to_owned(),
        };
        std::thread::spawn(move || owner.run());
        Ok(Run {
            line,
            label,
            args: args.to_vec(),
            started_at: SystemTime::now(),
            started,
            events,
            stop,
        })
    }

    /// The label of the command that runs in `app`, if one does.
    pub fn running(&self, app: &Path) -> Option<String> {
        self.lock().get(app).cloned()
    }

    fn lock(&self) -> MutexGuard<'_, BTreeMap<PathBuf, String>> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Why a command did not start.
#[cfg(feature = "run")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartError {
    /// The app runs another command, with this label.
    Busy { command: String },
    /// No way to run PHP is chosen for the app.
    NoRunner,
    /// `NoCommand`, `NoFolder`, `NotFound` or `NotStarted`.
    Failed(Failure),
}

/// A command that started. Its owner thread sends what happens to it.
#[cfg(feature = "run")]
#[derive(Debug)]
pub struct Run {
    /// What runs; `display()` is the `$` line.
    pub line: CommandLine,
    /// The artisan words, quoted as the `$` line quotes them.
    pub label: String,
    /// The artisan words as given to [`Runs::start`].
    pub args: Vec<String>,
    pub started_at: SystemTime,
    started: Instant,
    events: Receiver<RunEvent>,
    stop: Arc<AtomicBool>,
}

#[cfg(feature = "run")]
impl Run {
    /// Asks the run to end its process tree. Safe to call twice or after the end.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    /// The time since the start.
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// What arrived since the last call. It never blocks.
    pub fn events(&self) -> Vec<RunEvent> {
        self.events.try_iter().collect()
    }

    /// What arrives until the run ended or `deadline` passed, with
    /// [`RunEvent::Ended`] last when it came. It sleeps between events. After a
    /// [`Run::stop`], a deadline [`STOP_WAIT`] away leaves room for the end.
    pub fn wait(&self, deadline: Instant) -> Vec<RunEvent> {
        let mut arrived = Vec::new();
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.events.recv_timeout(left) {
                Ok(event) => {
                    let ended = matches!(event, RunEvent::Ended(_));
                    arrived.push(event);
                    if ended {
                        return arrived;
                    }
                }
                Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
                    return arrived;
                }
            }
        }
    }
}

/// What a run sends, in order. [`RunEvent::Ended`] comes last.
#[cfg(feature = "run")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunEvent {
    /// A line without its newline, with its escape codes. `cut` is true when the
    /// line was longer than `KEEP` bytes and `text` holds only its start.
    Line {
        stream: Stream,
        text: String,
        cut: bool,
    },
    /// The output passed [`OUTPUT_LIMIT`]; the later lines are dropped.
    Cut,
    /// The run is over. No thread of the run is left, and the app accepts a new start.
    Ended(RunEnd),
}

/// How a run ended.
#[cfg(feature = "run")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnd {
    /// The program ended by itself. `code` is `None` when a signal that Mortise
    /// did not send ended it.
    Exited {
        code: Option<i32>,
        elapsed: Duration,
    },
    /// Mortise stopped it. `limit` is the time limit, when that stopped it.
    Stopped {
        elapsed: Duration,
        limit: Option<Duration>,
    },
}

/// Whether a stop of the command with these artisan words asks first. A migration
/// that stops half way can leave the database between two states.
#[cfg(feature = "run")]
pub fn stop_asks_first(args: &[String]) -> bool {
    const ASK: [&str; 6] = [
        "module:migrate",
        "module:migrate-rollback",
        "module:migrate-refresh",
        "module:migrate-reset",
        "module:migrate-fresh",
        "module:seed",
    ];
    args.first()
        .is_some_and(|word| ASK.contains(&word.as_str()))
}

/// The artisan words one space apart, each quoted as [`CommandLine::display`]
/// quotes an argument on this platform.
#[cfg(feature = "run")]
fn label(args: &[String]) -> String {
    // `display` quotes each argument on its own, and a bare program adds no `& `.
    let line = CommandLine {
        program: "artisan".into(),
        args: args.to_vec(),
    };
    let shown = line.display();
    shown
        .strip_prefix("artisan")
        .unwrap_or(&shown)
        .trim_start()
        .to_string()
}

/// The failure of a program that did not start.
#[cfg(feature = "run")]
pub(crate) fn not_started(error: std::io::Error, program: &str) -> Failure {
    if error.kind() == std::io::ErrorKind::NotFound {
        Failure::NotFound {
            program: program.to_owned(),
        }
    } else {
        Failure::NotStarted {
            program: program.to_owned(),
            message: error.to_string(),
        }
    }
}

/// The thread that holds the program of a run.
#[cfg(feature = "run")]
struct Owner {
    process: Process,
    started: Instant,
    limit: Duration,
    stop: Arc<AtomicBool>,
    events: Sender<RunEvent>,
    runs: Runs,
    app: PathBuf,
}

#[cfg(feature = "run")]
impl Owner {
    /// Forwards the lines until the program ends, by itself, by a stop or at the
    /// limit. Then it ends as D2 of the design says: the readers joined, the app
    /// removed from [`Runs`], and [`RunEvent::Ended`] last.
    fn run(self) {
        let Owner {
            mut process,
            started,
            limit,
            stop,
            events,
            runs,
            app,
        } = self;
        let mut sent = 0;
        let mut cut = false;
        let mut forward = |stream, text: String, line_cut| {
            if cut {
                return;
            }
            sent += text.len() + 1;
            if sent > OUTPUT_LIMIT {
                cut = true;
                let _ = events.send(RunEvent::Cut);
            } else {
                let _ = events.send(RunEvent::Line {
                    stream,
                    text,
                    cut: line_cut,
                });
            }
        };
        // `stopped` is `Some` when Mortise ended it, with the limit when that did.
        // `ended` is true once `Process::stop` ran.
        let mut ended = false;
        let stopped = loop {
            match process.exited() {
                Ok(true) => break None,
                Ok(false) => {}
                Err(_) => {
                    process.stop(&mut forward);
                    ended = true;
                    break None;
                }
            }
            let at_limit = started.elapsed() >= limit;
            if at_limit || stop.load(Ordering::Relaxed) {
                process.stop(&mut forward);
                ended = true;
                break Some(at_limit.then_some(limit));
            }
            process.read(TICK, &mut forward);
        };
        let elapsed = started.elapsed();
        let (status, late) = process.finish(ended, &stop, &mut forward);
        let end = match stopped {
            Some(limit) => RunEnd::Stopped { elapsed, limit },
            None if late => RunEnd::Stopped {
                elapsed: started.elapsed(),
                limit: None,
            },
            None => RunEnd::Exited {
                code: status.and_then(|status| status.code()),
                elapsed,
            },
        };
        runs.lock().remove(&app);
        let _ = events.send(RunEvent::Ended(end));
    }
}

/// How many lines may wait between the readers and the owner. A reader that
/// finds the queue full waits, so a flood of output stays bounded in memory.
#[cfg(feature = "run")]
const QUEUE: usize = 1024;

/// A started program and the threads that read its pipes.
#[cfg(feature = "run")]
struct Process {
    child: Child,
    lines: Receiver<(Stream, String, bool)>,
    readers: Vec<JoinHandle<()>>,
}

#[cfg(feature = "run")]
impl Process {
    /// Starts `line` in `folder`, on Unix as the leader of a new process group,
    /// with both pipes read into one channel.
    fn start(line: &CommandLine, folder: &Path, input: Option<Vec<u8>>) -> std::io::Result<Self> {
        let mut command = prepare(line, folder)?;
        if input.is_some() {
            command.stdin(Stdio::piped());
        }
        // Its children join the group, so a signal to the group reaches the tree.
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = command.spawn()?;
        // Written from a thread of its own, so that a program that reads its input
        // late cannot block the start. The pipe closes when the thread ends.
        if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
            std::thread::spawn(move || {
                use std::io::Write;
                let _ = stdin.write_all(&input);
            });
        }
        let (sender, lines) = mpsc::sync_channel(QUEUE);
        let readers = [
            read_lines(child.stdout.take(), Stream::Out, sender.clone()),
            read_lines(child.stderr.take(), Stream::Err, sender),
        ];
        Ok(Self {
            child,
            lines,
            readers: readers.into_iter().flatten().collect(),
        })
    }

    /// Waits up to `wait` for a line, then gives `forward` the lines that arrived,
    /// at most [`QUEUE`], so the caller looks at the program again soon.
    /// It returns whether a line came.
    fn read(&self, wait: Duration, forward: &mut impl FnMut(Stream, String, bool)) -> bool {
        match self.lines.recv_timeout(wait) {
            Ok((stream, text, cut)) => forward(stream, text, cut),
            Err(RecvTimeoutError::Timeout) => return false,
            Err(RecvTimeoutError::Disconnected) => {
                std::thread::sleep(wait);
                return false;
            }
        }
        for (stream, text, cut) in self.lines.try_iter().take(QUEUE) {
            forward(stream, text, cut);
        }
        true
    }

    /// Whether the program has exited. On Unix it is not reaped, so its pid, which
    /// is the id of its group, stays taken until [`Process::finish`] kills the group.
    #[cfg(unix)]
    fn exited(&mut self) -> std::io::Result<bool> {
        use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
        let options = WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT;
        let status = waitid(WaitId::Pid(Pid::from_child(&self.child)), options)?;
        Ok(status.is_some())
    }

    /// Whether the program has exited. The handle that `Child` holds keeps the pid
    /// from being used again.
    #[cfg(not(unix))]
    fn exited(&mut self) -> std::io::Result<bool> {
        Ok(self.child.try_wait()?.is_some())
    }

    /// Ends the process tree: it asks first, and kills after [`STOP_GRACE`].
    /// It reads the lines meanwhile. [`Process::finish`] then kills what is left.
    fn stop(&mut self, forward: &mut impl FnMut(Stream, String, bool)) {
        self.terminate();
        let asked = Instant::now();
        while !self.exited().unwrap_or(true) {
            if asked.elapsed() >= STOP_GRACE {
                self.kill();
                return;
            }
            self.read(TICK, forward);
        }
    }

    /// Asks the tree to end: SIGTERM to the group.
    #[cfg(unix)]
    fn terminate(&mut self) {
        self.signal(rustix::process::Signal::TERM);
    }

    /// Ends the tree at once: SIGKILL to the group.
    #[cfg(unix)]
    fn kill(&mut self) {
        self.signal(rustix::process::Signal::KILL);
    }

    /// Sends `signal` to the group of the program. `ESRCH` means that the group is
    /// gone. On another error the leader alone gets SIGKILL, so a wait for it ends.
    #[cfg(unix)]
    fn signal(&mut self, signal: rustix::process::Signal) {
        use rustix::process::{Pid, kill_process_group};
        match kill_process_group(Pid::from_child(&self.child), signal) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => {}
            Err(_) => {
                let _ = self.child.kill();
            }
        }
    }

    /// Ends the tree: `taskkill /T` ends the children too, so `php.exe` under
    /// `herd.bat` ends. When `taskkill` fails, the program alone is ended.
    /// Not run on Windows.
    #[cfg(not(unix))]
    fn terminate(&mut self) {
        let pid = self.child.id().to_string();
        let ended = command("taskkill")
            .args(["/PID", pid.as_str(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if !ended {
            let _ = self.child.kill();
        }
    }

    /// Ends the program at once.
    #[cfg(not(unix))]
    fn kill(&mut self) {
        let _ = self.child.kill();
    }

    /// The end, once the program has exited or was stopped. `stopped` is true when
    /// [`Process::stop`] already ran. `stop` is the flag of a stop that the user or
    /// the caller may still ask for. The readers get [`GRACE`] to finish. Then, on Unix,
    /// what is left in the group is killed, even a child that closed its pipes, and
    /// the program is reaped. The readers are joined and `forward` gets their last
    /// lines. It returns how the program ended, and whether a stop ended what was
    /// left of the tree.
    ///
    /// A stop that is asked for while a child that outlived the program still
    /// writes, or that was asked for as the program ended and has not started,
    /// ends what is left as [`Process::stop`] does, with [`Process::stop_the_rest`],
    /// whether or not a child still holds a pipe. After a stop, the readers are not
    /// waited for past [`GRACE`] and [`DETACH`], so the run ends within [`STOP_WAIT`].
    ///
    /// Known limit: a process that left the group (`setsid`), or on Windows a child
    /// of a program that ended by itself or that `taskkill` missed, can keep a pipe
    /// open. Windows cannot name the children of an ended program without a job
    /// object. A reader that still reads [`DETACH`] later is left behind; it ends
    /// when the pipe closes or the queue is gone.
    fn finish(
        &mut self,
        stopped: bool,
        stop: &AtomicBool,
        forward: &mut impl FnMut(Stream, String, bool),
    ) -> (Option<ExitStatus>, bool) {
        let stopping = || stop.load(Ordering::Relaxed);
        let mut done = self.wait_for_readers(GRACE, (!stopped).then_some(stop), forward);
        let mut late = !stopped && stopping();
        let status = if late {
            let (status, rest_done) = self.stop_the_rest(done, forward);
            done = rest_done;
            status
        } else {
            // The program is not reaped yet, so the group id cannot belong to another group.
            #[cfg(unix)]
            self.kill();
            self.child.wait().ok()
        };
        if !done {
            let watch = !stopped && !late;
            done = self.wait_for_readers(DETACH, watch.then_some(stop), forward);
            late |= watch && !done && stopping();
        }
        for reader in std::mem::take(&mut self.readers) {
            if reader.is_finished() {
                let _ = reader.join();
            }
        }
        // A reader left behind may still send; the queue bounds what is taken.
        for (stream, text, cut) in self.lines.try_iter().take(QUEUE) {
            forward(stream, text, cut);
        }
        (status, late)
    }

    /// Ends what is left of the tree after the program exited, as [`Process::stop`]
    /// does: SIGTERM to the group, and SIGKILL to what is still there after
    /// [`STOP_GRACE`]. It reads the lines meanwhile and reaps the program. `done`
    /// says whether the readers already finished. It returns the status of the
    /// program and whether the readers finished.
    ///
    /// On Unix the SIGTERM goes first, while the unreaped program keeps the group id
    /// ours. The program is then reaped, because a group whose leader is a zombie
    /// still answers signal 0 on Linux. Then signal 0 to the group, every [`TICK`],
    /// tells whether a member is left, also one that closed its pipes. The SIGKILL
    /// after [`STOP_GRACE`] goes to a group whose member answered a [`TICK`] before.
    ///
    /// On Windows `taskkill` runs on the program, which has exited, and Windows
    /// cannot name what is left: the readers get [`STOP_GRACE`], and a child that
    /// closed its pipes is not waited for.
    fn stop_the_rest(
        &mut self,
        done: bool,
        forward: &mut impl FnMut(Stream, String, bool),
    ) -> (Option<ExitStatus>, bool) {
        self.terminate();
        #[cfg(unix)]
        {
            use rustix::process::{Pid, test_kill_process_group};
            let asked = Instant::now();
            let status = self.child.wait().ok();
            let group = Pid::from_child(&self.child);
            while test_kill_process_group(group).is_ok() {
                if asked.elapsed() >= STOP_GRACE {
                    self.kill();
                    break;
                }
                // With both readers gone, this sleeps for a tick.
                self.read(TICK, forward);
            }
            let finished = self.readers.iter().all(JoinHandle::is_finished);
            (status, done || finished)
        }
        #[cfg(not(unix))]
        {
            let done = done || self.wait_for_readers(STOP_GRACE, None, forward);
            self.kill();
            (self.child.wait().ok(), done)
        }
    }

    /// Reads until both readers finished, for at most `wait`. True when they did.
    /// With `stop`, a reader that waits for room in the queue is not stuck, so
    /// `wait` starts again while lines come, up to [`DRAIN`] in all, and the wait
    /// ends early once `stop` is set.
    fn wait_for_readers(
        &self,
        wait: Duration,
        stop: Option<&AtomicBool>,
        forward: &mut impl FnMut(Stream, String, bool),
    ) -> bool {
        let last = Instant::now() + DRAIN;
        let mut until = Instant::now() + wait;
        loop {
            if self.readers.iter().all(JoinHandle::is_finished) {
                return true;
            }
            if stop.is_some_and(|stop| stop.load(Ordering::Relaxed)) {
                return false;
            }
            let left = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            if self.read(left.min(TICK), forward) && stop.is_some() {
                until = (Instant::now() + wait).min(last);
            }
        }
    }
}

/// What a program printed and how it ended.
#[cfg(feature = "run")]
#[derive(Debug, Default)]
pub(crate) struct Captured {
    /// `None` when the program was stopped or ended by a signal.
    pub code: Option<i32>,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    /// The program ran past the limit and was stopped.
    pub timed_out: bool,
}

/// Runs `line` in `folder` and keeps its lines, at most [`KEEP`] bytes of each
/// stream, for at most `limit`. At the limit it ends the whole process tree.
/// An error means the program did not start.
#[cfg(feature = "run")]
pub(crate) fn capture(
    line: &CommandLine,
    folder: &Path,
    limit: Duration,
) -> std::io::Result<Captured> {
    let mut process = Process::start(line, folder, None)?;
    let started = Instant::now();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut kept = [0, 0];
    let mut keep = |stream, text: String, _cut| {
        let (kept, lines) = match stream {
            Stream::Out => (&mut kept[0], &mut stdout),
            Stream::Err => (&mut kept[1], &mut stderr),
        };
        // Once a stream is full, its later lines are dropped too.
        if *kept + text.len() > KEEP {
            *kept = KEEP + 1;
        } else {
            *kept += text.len() + 1;
            lines.push(text);
        }
    };
    let mut timed_out = false;
    while !process.exited()? {
        if started.elapsed() >= limit {
            process.stop(&mut keep);
            timed_out = true;
            break;
        }
        process.read(TICK, &mut keep);
    }
    let (status, _) = process.finish(timed_out, &AtomicBool::new(false), &mut keep);
    Ok(Captured {
        code: status
            .and_then(|status| status.code())
            .filter(|_| !timed_out),
        stdout,
        stderr,
        timed_out,
    })
}

/// The command for `line` in `folder`, with both pipes and the `PATH` of [`child_path`].
/// An error means the program must not start.
#[cfg(feature = "run")]
fn prepare(line: &CommandLine, folder: &Path) -> std::io::Result<Command> {
    let program = program_path(&line.program, folder);
    if cfg!(windows) {
        refuse_batch_with_percent(&program)?;
    }
    let mut command = command(program);
    command
        .args(&line.args)
        .current_dir(folder)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let home = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf());
    if let Some(path) = child_path(std::env::var_os("PATH"), home.as_deref()) {
        command.env("PATH", path);
    }
    Ok(command)
}

/// Refuses a `.bat` or `.cmd` whose path holds `%`. On Windows std starts a batch
/// file through `cmd.exe`, which would expand the `%` in its path. Windows ignores
/// dots and spaces at the end of a name, so they do not hide the extension. The
/// folder that a lookup in `PATH` finds is not checked.
#[cfg(feature = "run")]
fn refuse_batch_with_percent(program: &Path) -> std::io::Result<()> {
    let name = program.to_string_lossy();
    let name = name.trim_end_matches(['.', ' ']).to_ascii_lowercase();
    if (name.ends_with(".bat") || name.ends_with(".cmd")) && name.contains('%') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "the path of the batch file holds %, which cmd.exe would expand",
        ));
    }
    Ok(())
}

/// The program to start for the word `program`. A relative path with a separator
/// (`/`, or `\\` on Windows) is joined to `folder`: std leaves a relative program with
/// `current_dir` to the platform. A bare name stays a lookup in `PATH`.
#[cfg(feature = "run")]
fn program_path(program: &str, folder: &Path) -> PathBuf {
    let separators: &[char] = if cfg!(windows) { &['/', '\\'] } else { &['/'] };
    if program.contains(separators) {
        folder.join(program)
    } else {
        PathBuf::from(program)
    }
}

/// The `PATH` of Mortise with the folders of [`extra_folders`] at the end.
/// `None` when the folders cannot be joined; the child then keeps the `PATH` of Mortise.
#[cfg(feature = "run")]
fn child_path(path: Option<OsString>, home: Option<&Path>) -> Option<OsString> {
    let mut folders: Vec<PathBuf> = path.iter().flat_map(std::env::split_paths).collect();
    folders.extend(extra_folders(home));
    std::env::join_paths(folders).ok()
}

/// Where Herd, Homebrew and Docker put their programs. A desktop app on macOS starts
/// with a `PATH` that lacks them. Assumed from their documentation, not checked on
/// every platform.
#[cfg(feature = "run")]
fn extra_folders(home: Option<&Path>) -> Vec<PathBuf> {
    let at_home = |folder: &str| home.map(|home| home.join(folder));
    let folders = if cfg!(target_os = "macos") {
        vec![
            at_home("Library/Application Support/Herd/bin"),
            Some("/opt/homebrew/bin".into()),
            Some("/usr/local/bin".into()),
            at_home(".docker/bin"),
        ]
    } else if cfg!(windows) {
        vec![at_home(r".config\herd\bin")]
    } else {
        vec![Some("/usr/local/bin".into()), at_home(".local/bin")]
    };
    folders.into_iter().flatten().collect()
}

/// Reads `pipe` line by line on a thread of its own and sends each line with `stream`.
/// A line is cut at [`KEEP`] bytes, and sent with `true` then. The thread ends when the pipe closes or nobody
/// listens any more. `None` when there is no pipe.
#[cfg(feature = "run")]
fn read_lines(
    pipe: Option<impl Read + Send + 'static>,
    stream: Stream,
    sender: SyncSender<(Stream, String, bool)>,
) -> Option<JoinHandle<()>> {
    let pipe = pipe?;
    Some(std::thread::spawn(move || {
        let mut reader = BufReader::new(pipe);
        let mut bytes = Vec::new();
        loop {
            bytes.clear();
            // `take` bounds the buffer when a stream has no newline.
            match reader
                .by_ref()
                .take(KEEP as u64)
                .read_until(b'\n', &mut bytes)
            {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
            let mut cut = false;
            if bytes.last() == Some(&b'\n') {
                bytes.pop();
            } else if bytes.len() == KEEP {
                // An error shows again at the next read, which ends the thread.
                cut = skip_line(&mut reader).unwrap_or(true);
            }
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
            let text = String::from_utf8_lossy(&bytes).into_owned();
            if sender.send((stream, text, cut)).is_err() {
                return;
            }
        }
    }))
}

/// Reads and drops the rest of a line, up to and with its newline. True when it
/// dropped a byte other than the line's end (`\r` and `\n`).
#[cfg(feature = "run")]
fn skip_line(reader: &mut impl BufRead) -> std::io::Result<bool> {
    let mut dropped = false;
    loop {
        let buffer = match reader.fill_buf() {
            Ok(buffer) => buffer,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        if buffer.is_empty() {
            return Ok(dropped);
        }
        let end = buffer.iter().position(|byte| *byte == b'\n');
        let rest = &buffer[..end.unwrap_or(buffer.len())];
        dropped |= rest.iter().any(|byte| *byte != b'\r');
        match end {
            Some(at) => {
                reader.consume(at + 1);
                return Ok(dropped);
            }
            None => {
                let read = buffer.len();
                reader.consume(read);
            }
        }
    }
}

#[cfg(all(test, feature = "run"))]
mod tests {
    use super::*;

    /// Starts `script` in `sh` in `folder`, and reads until the program exited and
    /// `ready` holds. It gives the program and the lines read.
    #[cfg(unix)]
    fn exited_with_its_child(
        script: &str,
        folder: &Path,
        ready: impl Fn(&Process, &[String]) -> bool,
    ) -> (Process, Vec<String>) {
        let line = CommandLine {
            program: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
        };
        let mut process = Process::start(&line, folder, None).unwrap();
        let mut lines = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !(process.exited().unwrap() && ready(&process, &lines)) {
            assert!(Instant::now() < deadline, "{lines:?}");
            process.read(TICK, &mut |_, text, _| lines.push(text));
        }
        (process, lines)
    }

    /// The owner reads the stop flag after it sees that the program exited, so a
    /// stop asked for as the program ends is pending when the readers drain. Here
    /// the program has exited, and its child is ready, before the flag is set.
    #[cfg(unix)]
    #[test]
    fn a_stop_pending_when_the_readers_drain_asks_the_rest_to_end() {
        let folder =
            std::env::temp_dir().join(format!("mortise-pending-stop-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let pending = AtomicBool::new(true);

        // The child writes, and says when SIGTERM comes.
        let (mut process, mut lines) = exited_with_its_child(
            r#"(trap 'echo term; exit 0' TERM; while :; do echo tick; sleep 0.05; done) &"#,
            &folder,
            |_, lines| lines.iter().any(|line| line == "tick"),
        );
        let (_, late) = process.finish(false, &pending, &mut |_, text, _| lines.push(text));
        assert!(late);
        assert!(lines.iter().any(|line| line == "term"), "{lines:?}");

        // The child closed its pipes, so the readers are done, and it leaves a file
        // when SIGTERM comes. It ends at SIGTERM, before the kill.
        let (mut process, _) = exited_with_its_child(
            r#"(trap 'touch term; exit 0' TERM; exec >&- 2>&-; touch ready; while :; do sleep 0.05; done) &"#,
            &folder,
            |process, _| {
                folder.join("ready").exists() && process.readers.iter().all(JoinHandle::is_finished)
            },
        );
        let asked = Instant::now();
        let (_, late) = process.finish(false, &pending, &mut |_, _, _| {});
        assert!(late);
        assert!(asked.elapsed() < STOP_GRACE, "{:?}", asked.elapsed());
        assert!(folder.join("term").exists());
        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn a_stop_of_a_migration_or_a_seed_asks_first() {
        let words = |words: &[&str]| {
            words
                .iter()
                .map(|word| word.to_string())
                .collect::<Vec<_>>()
        };
        assert!(stop_asks_first(&words(&["module:migrate", "Blog"])));
        assert!(stop_asks_first(&words(&["module:migrate-fresh"])));
        assert!(stop_asks_first(&words(&["module:seed", "Blog"])));
        assert!(!stop_asks_first(&words(&["module:make", "module:migrate"])));
        assert!(!stop_asks_first(&words(&["migrate"])));
        assert!(!stop_asks_first(&[]));
    }

    #[test]
    fn a_program_with_a_separator_starts_in_the_folder() {
        let folder = Path::new("/code/shop-api");
        assert_eq!(program_path("php", folder), PathBuf::from("php"));
        assert_eq!(
            program_path("./vendor/bin/sail", folder),
            folder.join("./vendor/bin/sail")
        );
        assert_eq!(
            program_path("vendor/bin/sail", folder),
            folder.join("vendor/bin/sail")
        );
        let windows = program_path(r"vendor\bin\sail", folder);
        if cfg!(windows) {
            assert_eq!(windows, folder.join(r"vendor\bin\sail"));
        } else {
            assert_eq!(windows, PathBuf::from(r"vendor\bin\sail"));
        }
        let absolute = std::env::temp_dir().join("php");
        let absolute = absolute.to_str().unwrap();
        assert_eq!(program_path(absolute, folder), PathBuf::from(absolute));
    }

    #[test]
    fn a_batch_file_with_percent_in_its_path_is_refused() {
        for program in [
            r"C:\100%\herd.bat",
            r"C:\%USERPROFILE%\herd.CMD",
            "%a%.Bat. ",
        ] {
            let error = refuse_batch_with_percent(Path::new(program)).unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput, "{program}");
        }
        // A refusal, from here or from std, is a command that did not start, with its text.
        let refusal = std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "batch file arguments are invalid",
        );
        assert_eq!(
            not_started(refusal, "herd.bat"),
            Failure::NotStarted {
                program: "herd.bat".to_owned(),
                message: "batch file arguments are invalid".to_owned(),
            }
        );
        for program in [r"C:\100%\php.exe", r"C:\herd\herd.bat", "herd.bat"] {
            assert!(
                refuse_batch_with_percent(Path::new(program)).is_ok(),
                "{program}"
            );
        }
    }

    #[test]
    fn the_path_of_mortise_comes_first() {
        let home = Path::new("/home/ada");
        let path = std::env::join_paths(["/usr/bin", "/bin"]).unwrap();
        let joined = child_path(Some(path), Some(home)).unwrap();
        let folders: Vec<PathBuf> = std::env::split_paths(&joined).collect();
        assert_eq!(
            folders[..2],
            [PathBuf::from("/usr/bin"), PathBuf::from("/bin")]
        );
        assert_eq!(folders[2..], extra_folders(Some(home)));
        assert!(folders.iter().any(|folder| folder.starts_with(home)));
    }
}
