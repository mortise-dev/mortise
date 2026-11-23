//! A read: a package command whose only purpose is data.
//!
//! A read starts as a run does, through [`Runs::start`], so it shares the lock of
//! one command per app and the time limit of the app. It keeps standard output and
//! standard error apart, and it is not kept in the history.

use std::path::Path;
use std::time::Instant;

use super::PhpSettings;
use super::ansi::plain;
use super::process::{Run, RunEnd, RunEvent, Runs, StartError, Stream};

/// A read that runs.
#[derive(Debug)]
pub struct Read {
    run: Run,
    stdout: Vec<String>,
    stderr: Vec<String>,
}

/// How a read ended, with its lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadEnd {
    pub end: RunEnd,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

impl Read {
    /// Starts artisan with `args` in the app, for at most the app's time limit.
    pub fn start(
        runs: &Runs,
        app: &Path,
        php: &PhpSettings,
        args: &[String],
    ) -> Result<Read, StartError> {
        let run = runs.start(app, php, args, php.time_limit())?;
        Ok(Read {
            run,
            stdout: Vec::new(),
            stderr: Vec::new(),
        })
    }

    /// Takes what arrived. It never blocks. `Some` once the read ended; the read
    /// is over then and gives nothing more.
    pub fn poll(&mut self) -> Option<ReadEnd> {
        let events = self.run.events();
        self.take(events)
    }

    /// Waits until the read ended or `deadline` passed. `Some` once it ended.
    pub fn wait(&mut self, deadline: Instant) -> Option<ReadEnd> {
        let events = self.run.wait(deadline);
        self.take(events)
    }

    /// Asks the read to end its process tree. Safe to call twice or after the end.
    pub fn stop(&self) {
        self.run.stop();
    }

    fn take(&mut self, events: Vec<RunEvent>) -> Option<ReadEnd> {
        for event in events {
            match event {
                RunEvent::Line {
                    stream: Stream::Out,
                    text,
                } => self.stdout.push(text),
                RunEvent::Line {
                    stream: Stream::Err,
                    text,
                } => self.stderr.push(text),
                // A cut document does not parse, so it is never read in part.
                RunEvent::Cut => {}
                RunEvent::Ended(end) => {
                    return Some(ReadEnd {
                        end,
                        stdout: std::mem::take(&mut self.stdout),
                        stderr: std::mem::take(&mut self.stderr),
                    });
                }
            }
        }
        None
    }
}

impl ReadEnd {
    /// The last non-empty line of standard error, without escape codes. It says
    /// why a read failed that gave no document.
    pub fn error_line(&self) -> Option<String> {
        self.stderr
            .iter()
            .rev()
            .map(|line| plain(line).trim().to_string())
            .find(|line| !line.is_empty())
    }
}
