//! Starting a program: the setup every program Mortise starts shares.

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
