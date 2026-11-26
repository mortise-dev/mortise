//! A read: a package command whose only purpose is data.
//!
//! A read starts as a run does, through [`Runs::start`], so it shares the lock of
//! one command per app and the time limit of the app. It keeps standard output and
//! standard error apart, and it is not kept in the history.

use std::path::Path;
use std::time::Instant;

use serde::de::DeserializeOwned;

use super::PhpSettings;
use super::ansi::plain;
use super::process::{Run, RunEnd, RunEvent, Runs, StartError, Stream};
use crate::package::{DataError, whole_document};

/// A read that runs.
#[derive(Debug)]
pub struct Read {
    run: Run,
    stdout: Vec<String>,
    stderr: Vec<String>,
    stdout_cut: bool,
}

/// How a read ended, with its lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadEnd {
    pub end: RunEnd,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    /// The last non-empty line of standard output was cut, or the output passed
    /// its limit and its later lines were dropped. [`ReadEnd::document`] then gives
    /// [`DataError::Cut`].
    pub stdout_cut: bool,
}

impl Read {
    /// Starts artisan with `args` in the app, for at most the app's time limit.
    pub fn start(
        runs: &Runs,
        app: &Path,
        php: &PhpSettings,
        args: &[String],
    ) -> Result<Read, StartError> {
        Self::start_with_input(runs, app, php, args, None)
    }

    /// [`Read::start`], with `input` on the standard input of the command.
    pub fn start_with_input(
        runs: &Runs,
        app: &Path,
        php: &PhpSettings,
        args: &[String],
        input: Option<Vec<u8>>,
    ) -> Result<Read, StartError> {
        let run = runs.start_with_input(app, php, args, php.time_limit(), input)?;
        Ok(Read {
            run,
            stdout: Vec::new(),
            stderr: Vec::new(),
            stdout_cut: false,
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
                    cut,
                } => {
                    // An empty line after a cut one leaves the mark.
                    self.stdout_cut = cut || (self.stdout_cut && text.trim().is_empty());
                    self.stdout.push(text);
                }
                RunEvent::Line {
                    stream: Stream::Err,
                    text,
                    ..
                } => self.stderr.push(text),
                // The later lines are dropped, the document with them: a line kept
                // before is not the document.
                RunEvent::Cut => self.stdout_cut = true,
                RunEvent::Ended(end) => {
                    return Some(ReadEnd {
                        end,
                        stdout: std::mem::take(&mut self.stdout),
                        stderr: std::mem::take(&mut self.stderr),
                        stdout_cut: self.stdout_cut,
                    });
                }
            }
        }
        None
    }
}

impl ReadEnd {
    /// The document on standard output, read as `T` ([`whole_document`]).
    pub fn document<T: DeserializeOwned>(&self) -> Result<T, DataError> {
        whole_document(&self.stdout, self.stdout_cut)
    }

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
