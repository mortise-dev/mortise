//! Opening a file in the user's editor, at a line.

use std::path::Path;
use std::process::Stdio;

use serde::{Deserialize, Serialize};

/// The editor that opens files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Editor {
    #[default]
    VsCode,
    PhpStorm,
    Zed,
    /// A command the user wrote.
    Custom,
}

impl Editor {
    /// The command of the editor, with `{file}` and `{line}` where they go.
    /// `None` for [`Editor::Custom`], whose command the user writes.
    pub fn command(self) -> Option<&'static str> {
        match self {
            Editor::VsCode => Some("code --goto {file}:{line}"),
            Editor::PhpStorm => Some("phpstorm --line {line} {file}"),
            Editor::Zed => Some("zed {file}:{line}"),
            Editor::Custom => None,
        }
    }
}

/// The editor did not start. `program` is the first word of the command and
/// `message` is the operating system's own text.
#[derive(Debug, thiserror::Error)]
#[error("{program}: {message}")]
pub struct EditorError {
    pub program: String,
    pub message: String,
}

/// Splits a command into its program and arguments, with the file and the line put in.
///
/// Words are split at spaces; a part in double quotes is one word. The file and
/// the line are put in after splitting, so a path with spaces stays one argument.
pub fn editor_arguments(command: &str, file: &Path, line: u32) -> Vec<String> {
    let file = file.to_string_lossy();
    split_words(command)
        .into_iter()
        .map(|word| {
            word.replace("{file}", &file)
                .replace("{line}", &line.to_string())
        })
        .collect()
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

/// Starts the editor on a file at a line. Does not wait for it to close.
pub fn open_in_editor(command: &str, file: &Path, line: u32) -> Result<(), EditorError> {
    let mut words = editor_arguments(command, file, line).into_iter();
    let program = words.next().unwrap_or_default();
    crate::run::process::command(&program)
        .args(words)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| EditorError {
            program,
            message: error.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_and_the_line_are_put_into_the_command() {
        let file = Path::new("/code/shop api/Modules/Billing/app/Models/Invoice.php");
        assert_eq!(
            editor_arguments(Editor::VsCode.command().unwrap(), file, 12),
            [
                "code",
                "--goto",
                "/code/shop api/Modules/Billing/app/Models/Invoice.php:12"
            ]
        );
        assert_eq!(
            editor_arguments(Editor::PhpStorm.command().unwrap(), file, 7),
            [
                "phpstorm",
                "--line",
                "7",
                "/code/shop api/Modules/Billing/app/Models/Invoice.php"
            ]
        );
        assert_eq!(
            editor_arguments(
                r#""/Applications/My Editor/bin/edit"  -l {line}  {file}"#,
                Path::new("a.php"),
                3
            ),
            ["/Applications/My Editor/bin/edit", "-l", "3", "a.php"]
        );
        assert!(editor_arguments("   ", file, 1).is_empty());
    }

    #[test]
    fn an_editor_that_is_not_there_gives_its_name_and_the_systems_message() {
        let error =
            open_in_editor("mortise-no-such-editor {file}", Path::new("a.php"), 1).unwrap_err();
        assert_eq!(error.program, "mortise-no-such-editor");
        assert!(!error.message.is_empty());
        assert!(error.to_string().starts_with("mortise-no-such-editor: "));
    }
}
