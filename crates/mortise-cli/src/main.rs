//! `mortise`, the command line tool of Mortise.
//!
//! It reads an app through `mortise-core`, as the window does, so the two cannot
//! disagree on a rule. It runs no PHP, writes nothing in the app, and writes only to
//! standard output and standard error.

mod check;
mod strings;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use check::{Format, Options};

/// No finding fails the check.
const PASSED: u8 = 0;
/// A finding fails the check.
const FAILED: u8 = 1;
/// An unknown command or flag, a missing or wrong value.
const USAGE: u8 = 2;
/// The folder cannot be read as an app.
const NOT_AN_APP: u8 = 3;
// 4 is kept for `mortise push`: the cloud refused the document.

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
        Err(message) => {
            eprintln!("{}", strings::usage_error(&message));
            USAGE
        }
    };
    ExitCode::from(code)
}

enum Command {
    Help,
    Version,
    Check(Options),
}

/// Reads the arguments after the name of the program.
fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Err(strings::NO_COMMAND.to_owned());
    };
    match first.to_str() {
        Some("check") => parse_check(args),
        Some("help" | "--help" | "-h") => Ok(Command::Help),
        Some("--version" | "-V") => Ok(Command::Version),
        _ => Err(strings::unknown_command(&first.to_string_lossy())),
    }
}

/// `check [PATH] [--coverage <clover.xml>] [--coverage-limit <0-100>]
/// [--format text|json|github] [--fail-on warnings]`. A flag takes its value as the
/// next argument or after `=`.
fn parse_check(mut args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    let mut path: Option<PathBuf> = None;
    let mut options = Options {
        path: PathBuf::from("."),
        coverage: None,
        coverage_limit: None,
        format: Format::Text,
        fail_on_warnings: false,
    };
    while let Some(arg) = args.next() {
        let Some(flag) = arg
            .to_str()
            .filter(|arg| arg.starts_with('-') && arg.len() > 1)
        else {
            if path.replace(PathBuf::from(&arg)).is_some() {
                return Err(strings::extra_argument(&arg.to_string_lossy()));
            }
            continue;
        };
        if matches!(flag, "--help" | "-h") {
            return Ok(Command::Help);
        }
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(OsString::from(value))),
            None => (flag, None),
        };
        let value = inline
            .or_else(|| args.next())
            .ok_or_else(|| strings::missing_value(name));
        match name {
            "--coverage" => options.coverage = Some(PathBuf::from(value?)),
            "--coverage-limit" => {
                let value = value?;
                let limit = value
                    .to_str()
                    .and_then(|text| text.trim_end_matches('%').parse::<u8>().ok())
                    .filter(|limit| *limit <= 100)
                    .ok_or_else(|| strings::wrong_limit(&value.to_string_lossy()))?;
                options.coverage_limit = Some(limit);
            }
            "--format" => {
                let value = value?;
                options.format = match value.to_str() {
                    Some("text") => Format::Text,
                    Some("json") => Format::Json,
                    Some("github") => Format::Github,
                    _ => return Err(strings::wrong_format(&value.to_string_lossy())),
                };
            }
            "--fail-on" => {
                let value = value?;
                if value != "warnings" {
                    return Err(strings::wrong_fail_on(&value.to_string_lossy()));
                }
                options.fail_on_warnings = true;
            }
            _ => return Err(strings::unknown_flag(name)),
        }
    }
    if let Some(path) = path {
        options.path = path;
    }
    Ok(Command::Check(options))
}
