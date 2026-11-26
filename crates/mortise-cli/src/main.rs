//! `mortise`, the command line tool of Mortise.
//!
//! It reads an app through `mortise-core`, as the window does, so the two cannot
//! disagree on a rule. It runs no PHP and writes nothing in the app. `check` writes only
//! to standard output and standard error. `push` also sends the push document to
//! Mortise Cloud, and writes it to a file when asked.

mod check;
mod github;
mod push;
mod send;
mod strings;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use check::Format;

/// No finding fails the check. For `push`: sent, or not sent for a reason that must not
/// fail the build.
const PASSED: u8 = 0;
/// A finding fails the check.
const FAILED: u8 = 1;
/// An unknown command or flag, a missing or wrong value, no token.
const USAGE: u8 = 2;
/// The folder cannot be read as an app.
const NOT_AN_APP: u8 = 3;
/// The cloud refused the document, or it is over a limit of the cloud.
const REFUSED: u8 = 4;
/// The report could not be written to standard output, other than to a closed pipe.
const NOT_WRITTEN: u8 = 5;

fn main() -> ExitCode {
    let code = match parse(std::env::args_os().skip(1)) {
        Ok(Command::Help) => {
            print!("{}", strings::HELP);
            PASSED
        }
        Ok(Command::Version) => {
            println!("mortise {}", env!("CARGO_PKG_VERSION"));
            PASSED
        }
        Ok(Command::Check(options)) => check::run(&options),
        Ok(Command::Push(options)) => push::run(&options),
        Err(message) => {
            say(&strings::usage_error(&message));
            USAGE
        }
    };
    ExitCode::from(code)
}

/// Text that comes from the app or the command line, made safe to print. A control
/// character or a line separator, such as a line end in a file name, is written as its
/// escape (`\n`). So no content of the app starts a line that CI reads as a workflow
/// command.
fn printable(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') {
                c.escape_default().to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// Prints a diagnostic on standard error.
fn say(text: &str) {
    eprintln!("{}", printable(text));
}

enum Command {
    Help,
    Version,
    Check(check::Options),
    Push(push::Options),
}

/// Reads the arguments after the name of the program.
fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Err(strings::NO_COMMAND.to_owned());
    };
    match first.to_str() {
        Some("check") => parse_check(Args(args)),
        Some("push") => parse_push(Args(args)),
        Some("help" | "--help" | "-h") => Ok(Command::Help),
        Some("--version" | "-V") => Ok(Command::Version),
        _ => Err(strings::unknown_command(&first.to_string_lossy())),
    }
}

/// The arguments of a command, one at a time. A flag takes its value as the next
/// argument or after `=`, and only when it asks for one.
struct Args<I>(I);

enum Arg {
    /// An argument that is not a flag: the app folder.
    Folder(OsString),
    Help,
    Flag {
        name: String,
        /// The value after `=`.
        inline: Option<OsString>,
    },
}

impl<I: Iterator<Item = OsString>> Args<I> {
    fn next(&mut self) -> Option<Arg> {
        let arg = self.0.next()?;
        let Some(flag) = arg
            .to_str()
            .filter(|arg| arg.starts_with('-') && arg.len() > 1)
        else {
            return Some(Arg::Folder(arg));
        };
        if matches!(flag, "--help" | "-h") {
            return Some(Arg::Help);
        }
        Some(match flag.split_once('=') {
            Some((name, value)) => Arg::Flag {
                name: name.to_owned(),
                inline: Some(OsString::from(value)),
            },
            None => Arg::Flag {
                name: flag.to_owned(),
                inline: None,
            },
        })
    }

    /// The value of the flag `name`. A separate value that starts with `-` is a flag left
    /// without its value, as in `--output --dry-run`. A value of that form goes after `=`.
    fn value(&mut self, name: &str, inline: Option<OsString>) -> Result<OsString, String> {
        if let Some(value) = inline {
            return Ok(value);
        }
        let value = self.0.next().ok_or_else(|| strings::missing_value(name))?;
        if value.to_string_lossy().starts_with('-') {
            return Err(strings::flag_as_value(name, &value.to_string_lossy()));
        }
        Ok(value)
    }

    /// The value of `name` as text that `valid` accepts, else the error of `wrong`.
    fn text(
        &mut self,
        name: &str,
        inline: Option<OsString>,
        valid: impl Fn(&str) -> bool,
        wrong: impl Fn(&str) -> String,
    ) -> Result<String, String> {
        let value = self.value(name, inline)?;
        match value.to_str() {
            Some(text) if valid(text) => Ok(text.to_owned()),
            _ => Err(wrong(&value.to_string_lossy())),
        }
    }
}

/// Keeps the one folder a command takes.
fn folder(path: &mut Option<PathBuf>, command: &str, arg: OsString) -> Result<(), String> {
    if path.replace(PathBuf::from(&arg)).is_some() {
        return Err(strings::extra_argument(command, &arg.to_string_lossy()));
    }
    Ok(())
}

/// `check [PATH] [--coverage <clover.xml>] [--coverage-limit <0-100>]
/// [--format text|json|github] [--fail-on warnings]`.
fn parse_check(mut args: Args<impl Iterator<Item = OsString>>) -> Result<Command, String> {
    let mut path: Option<PathBuf> = None;
    let mut options = check::Options {
        path: PathBuf::from("."),
        coverage: None,
        coverage_limit: None,
        format: Format::Text,
        fail_on_warnings: false,
    };
    while let Some(arg) = args.next() {
        let (name, inline) = match arg {
            Arg::Folder(arg) => {
                folder(&mut path, "check", arg)?;
                continue;
            }
            Arg::Help => return Ok(Command::Help),
            Arg::Flag { name, inline } => (name, inline),
        };
        match name.as_str() {
            "--coverage" => options.coverage = Some(PathBuf::from(args.value(&name, inline)?)),
            "--coverage-limit" => {
                let limit = args.text(
                    &name,
                    inline,
                    |text| {
                        text.trim_end_matches('%')
                            .parse::<u8>()
                            .is_ok_and(|limit| limit <= 100)
                    },
                    strings::wrong_limit,
                )?;
                options.coverage_limit = limit.trim_end_matches('%').parse().ok();
            }
            "--format" => {
                let format = args.text(
                    &name,
                    inline,
                    |text| matches!(text, "text" | "json" | "github"),
                    strings::wrong_format,
                )?;
                options.format = match format.as_str() {
                    "json" => Format::Json,
                    "github" => Format::Github,
                    _ => Format::Text,
                };
            }
            "--fail-on" => {
                args.text(
                    &name,
                    inline,
                    |text| text == "warnings",
                    strings::wrong_fail_on,
                )?;
                options.fail_on_warnings = true;
            }
            _ => return Err(strings::unknown_flag("check", &name)),
        }
    }
    if let Some(path) = path {
        options.path = path;
    }
    Ok(Command::Check(options))
}

/// `push [PATH] [--coverage <clover.xml>] [--environment <name>] [--dry-run]
/// [--output <file>] [--commit <sha>] [--branch <name>] [--pull-request <n>]
/// [--base-branch <name>] [--base-commit <sha>] [--repository <owner/name>]`.
fn parse_push(mut args: Args<impl Iterator<Item = OsString>>) -> Result<Command, String> {
    let mut path: Option<PathBuf> = None;
    let mut options = push::Options::default();
    while let Some(arg) = args.next() {
        let (name, inline) = match arg {
            Arg::Folder(arg) => {
                folder(&mut path, "push", arg)?;
                continue;
            }
            Arg::Help => return Ok(Command::Help),
            Arg::Flag { name, inline } => (name, inline),
        };
        let sha = |text: &str| push::is_sha(text);
        let wrong_sha = |value: &str| strings::wrong_sha(&name, value);
        let any = |text: &str| !text.trim().is_empty();
        let empty = |_: &str| strings::missing_value(&name);
        match name.as_str() {
            "--dry-run" => {
                if inline.is_some() {
                    return Err(strings::takes_no_value(&name));
                }
                options.dry_run = true;
            }
            "--coverage" => options.coverage = Some(PathBuf::from(args.value(&name, inline)?)),
            "--output" => options.output = Some(PathBuf::from(args.value(&name, inline)?)),
            "--environment" => {
                options.environment = Some(args.text(
                    &name,
                    inline,
                    push::is_environment,
                    strings::wrong_environment,
                )?);
            }
            "--commit" => options.commit = Some(args.text(&name, inline, sha, wrong_sha)?),
            "--base-commit" => {
                options.base_commit = Some(args.text(&name, inline, sha, wrong_sha)?);
            }
            "--branch" => options.branch = Some(args.text(&name, inline, any, empty)?),
            "--base-branch" => options.base_branch = Some(args.text(&name, inline, any, empty)?),
            "--pull-request" => {
                let number = args.text(
                    &name,
                    inline,
                    |text| text.parse::<u64>().is_ok_and(|n| n >= 1),
                    strings::wrong_pull_request,
                )?;
                options.pull_request = number.parse().ok();
            }
            "--repository" => {
                options.repository = Some(args.text(
                    &name,
                    inline,
                    push::is_repository,
                    strings::wrong_repository,
                )?);
            }
            _ => return Err(strings::unknown_flag("push", &name)),
        }
    }
    options.path = path.unwrap_or_else(|| PathBuf::from("."));
    Ok(Command::Push(options))
}
