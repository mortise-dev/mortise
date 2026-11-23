//! Starting a program: the setup every program Mortise starts shares, and, with
//! the `run` feature, running one to its end while its lines are read.

use std::ffi::OsStr;
#[cfg(feature = "run")]
use std::ffi::OsString;
#[cfg(feature = "run")]
use std::io::{BufRead, BufReader, Read};
#[cfg(feature = "run")]
use std::path::{Path, PathBuf};
#[cfg(feature = "run")]
use std::process::Child;
use std::process::{Command, Stdio};
#[cfg(feature = "run")]
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
#[cfg(feature = "run")]
use std::time::{Duration, Instant};

#[cfg(feature = "run")]
use super::CommandLine;

/// A command with no standard input and, on Windows, no console window.
pub(crate) fn command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    command.stdin(Stdio::null());
    // The app has no console on Windows, so each run of a program would open a window.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

/// How long the loop waits for a line before it looks at the program again.
#[cfg(feature = "run")]
const TICK: Duration = Duration::from_millis(50);

/// How long the loop still reads after the program ended. A program that a
/// grandchild outlives can leave a pipe open.
#[cfg(feature = "run")]
const GRACE: Duration = Duration::from_millis(250);

/// The pipe a line came from.
#[cfg(feature = "run")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stream {
    Out,
    Err,
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

/// Runs `line` in `folder` and keeps its lines, for at most `limit`.
/// An error means the program did not start.
#[cfg(feature = "run")]
pub(crate) fn capture(
    line: &CommandLine,
    folder: &Path,
    limit: Duration,
) -> std::io::Result<Captured> {
    let (mut child, lines) = start(line, folder)?;
    let started = Instant::now();
    let mut captured = Captured::default();
    let mut ended: Option<Instant> = None;
    let mut open = true;
    loop {
        if open {
            match lines.recv_timeout(TICK) {
                Ok((Stream::Out, text)) => captured.stdout.push(text),
                Ok((Stream::Err, text)) => captured.stderr.push(text),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => open = false,
            }
        } else {
            std::thread::sleep(TICK);
        }
        if ended.is_none()
            && let Some(status) = child.try_wait()?
        {
            captured.code = status.code();
            ended = Some(Instant::now());
        }
        match ended {
            Some(at) if !open || at.elapsed() >= GRACE => return Ok(captured),
            None if started.elapsed() >= limit => {
                // The program may have ended in the meantime; then there is nothing to stop.
                let _ = child.kill();
                let _ = child.wait();
                captured.timed_out = true;
                return Ok(captured);
            }
            _ => {}
        }
    }
}

/// Starts `line` in `folder` with both pipes read into one channel.
#[cfg(feature = "run")]
fn start(
    line: &CommandLine,
    folder: &Path,
) -> std::io::Result<(Child, Receiver<(Stream, String)>)> {
    let mut child = prepare(line, folder).spawn()?;
    let (sender, lines) = mpsc::channel();
    read_lines(child.stdout.take(), Stream::Out, sender.clone());
    read_lines(child.stderr.take(), Stream::Err, sender);
    Ok((child, lines))
}

/// The command for `line` in `folder`, with both pipes and the `PATH` of [`child_path`].
#[cfg(feature = "run")]
fn prepare(line: &CommandLine, folder: &Path) -> Command {
    // std leaves a relative program with `current_dir` to the platform, so it is made absolute.
    let program = if line.program.starts_with("./") || line.program.starts_with(".\\") {
        folder.join(&line.program).into_os_string()
    } else {
        OsString::from(&line.program)
    };
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
    command
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
/// The thread ends when the pipe closes or nobody listens any more.
#[cfg(feature = "run")]
fn read_lines(
    pipe: Option<impl Read + Send + 'static>,
    stream: Stream,
    sender: Sender<(Stream, String)>,
) {
    let Some(pipe) = pipe else { return };
    std::thread::spawn(move || {
        let mut reader = BufReader::new(pipe);
        let mut bytes = Vec::new();
        loop {
            bytes.clear();
            match reader.read_until(b'\n', &mut bytes) {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
            if bytes.last() == Some(&b'\n') {
                bytes.pop();
            }
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
            let text = String::from_utf8_lossy(&bytes).into_owned();
            if sender.send((stream, text)).is_err() {
                return;
            }
        }
    });
}

#[cfg(all(test, feature = "run"))]
mod tests {
    use super::*;

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
