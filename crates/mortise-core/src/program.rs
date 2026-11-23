//! Starting a program: the setup every program Mortise starts shares, and the
//! split of a command line into words.

use std::ffi::OsStr;
use std::process::{Command, Stdio};

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

/// Splits a command into words at spaces and tabs. A part in double quotes is one
/// word. A backslash is an ordinary letter, so a Windows path needs no escape.
pub(crate) fn split_words(command: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut started = false;
    for letter in command.chars() {
        match letter {
            '"' => {
                quoted = !quoted;
                started = true;
            }
            ' ' | '\t' if !quoted => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            _ => {
                word.push(letter);
                started = true;
            }
        }
    }
    if started {
        words.push(word);
    }
    words
}
