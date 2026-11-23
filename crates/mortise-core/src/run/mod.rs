//! How PHP runs for an app: the kind of runner, the command line, and starting it.
//!
//! A command line is a program and a list of arguments. It never goes through a shell.
//! With the `run` feature, [`test_php`] runs `artisan --version` through a runner.

#[cfg(feature = "run")]
mod detect;
#[cfg(feature = "run")]
pub use detect::{Detected, detect, herd_config_dir};
pub(crate) mod process;

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::editor::split_words;

/// The program through which PHP runs artisan for an app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RunnerKind {
    Herd,
    Sail,
    /// `docker compose exec` in the named service.
    Docker {
        service: String,
    },
    /// The words of a line the user typed.
    Custom {
        line: String,
    },
}

impl RunnerKind {
    /// The words before the artisan arguments, for this platform.
    /// An empty Custom line gives no words.
    pub fn prefix(&self) -> Vec<String> {
        let words: &[&str] = match self {
            RunnerKind::Herd if cfg!(windows) => &["herd.bat", "php", "artisan"],
            RunnerKind::Herd => &["herd", "php", "artisan"],
            RunnerKind::Sail => &["./vendor/bin/sail", "artisan"],
            RunnerKind::Docker { service } => {
                return ["docker", "compose", "exec", "-T", service, "php", "artisan"]
                    .map(String::from)
                    .to_vec();
            }
            RunnerKind::Custom { line } => return split_words(line),
        };
        words.iter().map(|word| word.to_string()).collect()
    }

    /// The text of the command field: the prefix as words one space apart, or the
    /// Custom line as the user typed it. [`RunnerKind::from_line`] reads it back.
    pub fn line(&self) -> String {
        match self {
            RunnerKind::Custom { line } => line.clone(),
            other => other.prefix().join(" "),
        }
    }

    /// The kind a command field holds. The words of Herd, or of Sail off Windows,
    /// give that kind. `docker compose exec [-T] <service> php artisan` gives Docker
    /// with that service. Any other text gives Custom with the text.
    pub fn from_line(line: &str) -> RunnerKind {
        let words = split_words(line);
        let words: Vec<&str> = words.iter().map(String::as_str).collect();
        for kind in [RunnerKind::Herd, RunnerKind::Sail] {
            if (kind != RunnerKind::Sail || !cfg!(windows)) && words == kind.prefix() {
                return kind;
            }
        }
        let service = match words.as_slice() {
            ["docker", "compose", "exec", "-T", service, "php", "artisan"]
            | ["docker", "compose", "exec", service, "php", "artisan"] => Some(*service),
            _ => None,
        };
        match service {
            Some(service) if !service.starts_with('-') => RunnerKind::Docker {
                service: service.to_string(),
            },
            _ => RunnerKind::Custom {
                line: line.to_string(),
            },
        }
    }
}

/// How PHP runs for one app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PhpSettings {
    /// `None`: not chosen yet. Detection proposes one; nothing runs.
    pub runner: Option<RunnerKind>,
    /// `None`: the app folder. A relative path starts at the app folder.
    pub working_folder: Option<PathBuf>,
    /// Minutes before a command is stopped. 10 by default (S8.2).
    pub time_limit_minutes: u16,
}

impl Default for PhpSettings {
    fn default() -> Self {
        Self {
            runner: None,
            working_folder: None,
            time_limit_minutes: 10,
        }
    }
}

impl PhpSettings {
    /// The folder a command runs in, for the app in `root`.
    pub fn folder(&self, root: &Path) -> PathBuf {
        match &self.working_folder {
            Some(folder) => root.join(folder),
            None => root.to_owned(),
        }
    }

    /// How long a command may run. 0 minutes reads as 1 minute.
    pub fn time_limit(&self) -> Duration {
        Duration::from_secs(u64::from(self.time_limit_minutes.max(1)) * 60)
    }
}

/// A program and its arguments, started without a shell.
#[cfg(feature = "run")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLine {
    pub program: String,
    pub args: Vec<String>,
}

#[cfg(feature = "run")]
impl CommandLine {
    /// The line that runs artisan with `arguments` through `runner`.
    /// `--no-interaction --ansi` come right after the first argument, so a `--`
    /// later in the list cannot turn them into plain words.
    /// `None` when the runner gives no program (an empty Custom line).
    pub fn artisan(runner: &RunnerKind, arguments: &[&str]) -> Option<Self> {
        let mut words = runner.prefix().into_iter();
        let program = words.next()?;
        let mut args: Vec<String> = words.collect();
        let (first, rest) = match arguments.split_first() {
            Some((first, rest)) => (Some(*first), rest),
            None => (None, arguments),
        };
        args.extend(first.map(String::from));
        args.extend(["--no-interaction", "--ansi"].map(String::from));
        args.extend(rest.iter().map(|word| word.to_string()));
        Some(Self { program, args })
    }

    /// The line to show and copy, quoted for the shell of this platform.
    /// The process always gets the list, not this text.
    pub fn display(&self) -> String {
        if cfg!(windows) {
            powershell(self)
        } else {
            posix(self)
        }
    }

    fn words(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.program.as_str()).chain(self.args.iter().map(String::as_str))
    }
}

/// How long "Test" waits for `artisan --version` before it stops it.
#[cfg(feature = "run")]
pub const TEST_LIMIT: Duration = Duration::from_secs(20);

/// What came of a test of the runner of an app.
#[cfg(feature = "run")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestOutcome {
    /// The program ended with code 0. The first line it printed, without colour codes.
    Answered(String),
    Failed(Failure),
}

/// Why a test failed. The interface writes the text; this is data only.
#[cfg(feature = "run")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The runner gives no program: an empty Custom line.
    NoCommand,
    /// The working folder is not a folder. Without this check a missing folder
    /// reads as a missing program on macOS and Linux.
    NoFolder { folder: PathBuf },
    /// The system found no program by that name.
    NotFound { program: String },
    /// The program did not start for another reason, with the message of the system.
    NotStarted { program: String, message: String },
    /// The program ended with another code, or by a signal (`None`). The first line
    /// of stderr that is not empty, else of stdout, else empty; without colour codes.
    Exited { code: Option<i32>, line: String },
    /// The program ran longer than the limit, in whole seconds, and was stopped.
    TimedOut { seconds: u64 },
}

/// Runs `artisan --version` through `runner` in `folder`, for at most `limit`.
/// It blocks, so an interface calls it off its drawing thread.
#[cfg(feature = "run")]
pub fn test_php(runner: &RunnerKind, folder: &Path, limit: Duration) -> TestOutcome {
    let Some(line) = CommandLine::artisan(runner, &["--version"]) else {
        return TestOutcome::Failed(Failure::NoCommand);
    };
    if !folder.is_dir() {
        return TestOutcome::Failed(Failure::NoFolder {
            folder: folder.to_owned(),
        });
    }
    let captured = match process::capture(&line, folder, limit) {
        Ok(captured) => captured,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return TestOutcome::Failed(Failure::NotFound {
                program: line.program,
            });
        }
        Err(error) => {
            return TestOutcome::Failed(Failure::NotStarted {
                program: line.program,
                message: error.to_string(),
            });
        }
    };
    let first = |lines: &[String]| {
        lines
            .iter()
            .map(|text| plain(text))
            .find(|text| !text.trim().is_empty())
    };
    if captured.timed_out {
        TestOutcome::Failed(Failure::TimedOut {
            seconds: limit.as_secs(),
        })
    } else if captured.code == Some(0) {
        TestOutcome::Answered(first(&captured.stdout).unwrap_or_default())
    } else {
        let line = first(&captured.stderr)
            .or_else(|| first(&captured.stdout))
            .unwrap_or_default();
        TestOutcome::Failed(Failure::Exited {
            code: captured.code,
            line,
        })
    }
}

/// `text` without its escape sequences `ESC [ ... final byte` (colours, mostly).
#[cfg(feature = "run")]
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut letters = text.chars().peekable();
    while let Some(letter) = letters.next() {
        if letter == '\u{1b}' && letters.peek() == Some(&'[') {
            letters.next();
            // Parameter and intermediate bytes run up to a final byte in `@` to `~`.
            for letter in letters.by_ref() {
                if ('@'..='~').contains(&letter) {
                    break;
                }
            }
        } else {
            out.push(letter);
        }
    }
    out
}

/// Whether a word can stand without quotes: not empty, and only ASCII letters,
/// digits, `_ - . / : = +` and the letters in `more`.
#[cfg(feature = "run")]
fn bare(word: &str, more: &[char]) -> bool {
    !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-./:=+".contains(c) || more.contains(&c))
}

/// The line for POSIX `sh`. A word that is not bare goes in single quotes, and a
/// `'` inside becomes `'\''`.
#[cfg(feature = "run")]
fn posix(line: &CommandLine) -> String {
    line.words()
        .map(|word| {
            if bare(word, &[]) {
                word.to_string()
            } else {
                format!("'{}'", word.replace('\'', r"'\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The line for PowerShell. A word that is not bare goes in single quotes, which
/// expand nothing. A quote inside is doubled; PowerShell also reads the curly
/// quotes as single quotes. A quoted program needs `& ` before it.
///
/// Known limit: Windows PowerShell 5.1 drops an empty argument and mangles a `"`
/// inside an argument to a program. PowerShell 7.3 and later pass both.
#[cfg(feature = "run")]
fn powershell(line: &CommandLine) -> String {
    const QUOTES: [char; 5] = ['\'', '\u{2018}', '\u{2019}', '\u{201A}', '\u{201B}'];
    let text = line
        .words()
        .map(|word| {
            if bare(word, &['\\']) {
                return word.to_string();
            }
            let mut quoted = String::from("'");
            for c in word.chars() {
                if QUOTES.contains(&c) {
                    quoted.push(c);
                }
                quoted.push(c);
            }
            quoted.push('\'');
            quoted
        })
        .collect::<Vec<_>>()
        .join(" ");
    if bare(&line.program, &['\\']) {
        text
    } else {
        format!("& {text}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_line_reads_each_kind() {
        let docker = RunnerKind::Docker {
            service: "web".into(),
        };
        assert_eq!(
            RunnerKind::from_line(&RunnerKind::Herd.line()),
            RunnerKind::Herd
        );
        assert_eq!(RunnerKind::from_line(&docker.line()), docker);
        assert_eq!(
            RunnerKind::from_line("docker  compose exec web php artisan"),
            docker
        );
        let sail = RunnerKind::from_line("./vendor/bin/sail artisan");
        let custom = |line: &str| RunnerKind::Custom { line: line.into() };
        if cfg!(windows) {
            assert_eq!(sail, custom("./vendor/bin/sail artisan"));
        } else {
            assert_eq!(sail, RunnerKind::Sail);
        }
        for line in [
            "\"/opt/my php/bin/php\" artisan",
            "docker compose exec -T php artisan",
            "herd php artisan tinker",
            "",
        ] {
            assert_eq!(RunnerKind::from_line(line), custom(line), "{line}");
        }
    }

    #[test]
    fn php_settings_give_the_folder_and_the_time_limit() {
        let root = Path::new("/code/shop-api");
        let mut php = PhpSettings {
            time_limit_minutes: 0,
            ..PhpSettings::default()
        };
        assert_eq!(php.folder(root), root);
        assert_eq!(php.time_limit(), Duration::from_secs(60));
        php.working_folder = Some("api".into());
        assert_eq!(php.folder(root), root.join("api"));
    }

    #[test]
    fn runner_kinds_keep_their_json_form() {
        let json = r#"[{"kind":"herd"},{"kind":"sail"},{"kind":"docker","service":"app"},{"kind":"custom","line":"php artisan"}]"#;
        let kinds: Vec<RunnerKind> = serde_json::from_str(json).unwrap();
        assert_eq!(serde_json::to_string(&kinds).unwrap(), json);
    }

    #[cfg(feature = "run")]
    fn line(words: &[&str]) -> CommandLine {
        CommandLine {
            program: words[0].into(),
            args: words[1..].iter().map(|word| word.to_string()).collect(),
        }
    }

    #[cfg(feature = "run")]
    #[test]
    fn artisan_puts_the_flags_after_the_first_argument() {
        let custom = |line: &str| RunnerKind::Custom { line: line.into() };
        let php = custom("php artisan");
        assert_eq!(
            CommandLine::artisan(&php, &["module:make", "Blog", "--", "--ansi"]),
            Some(line(&[
                "php",
                "artisan",
                "module:make",
                "--no-interaction",
                "--ansi",
                "Blog",
                "--",
                "--ansi"
            ]))
        );
        assert_eq!(
            CommandLine::artisan(&php, &[]),
            Some(line(&["php", "artisan", "--no-interaction", "--ansi"]))
        );
        let docker = RunnerKind::Docker {
            service: "app".into(),
        };
        let version = CommandLine::artisan(&docker, &["--version"]).unwrap();
        assert_eq!(version.program, "docker");
        assert_eq!(
            version.args.join(" "),
            "compose exec -T app php artisan --version --no-interaction --ansi"
        );
        assert_eq!(CommandLine::artisan(&custom(" "), &["--version"]), None);
    }

    #[cfg(feature = "run")]
    #[test]
    fn posix_quotes_what_sh_would_read() {
        for (words, shown) in [
            (
                &["php", "module:make", "--name=Blog"][..],
                "php module:make --name=Blog",
            ),
            (&["php", "My Blog"], "php 'My Blog'"),
            (&["php", "it's"], r"php 'it'\''s'"),
            (&["php", r#"say "hi""#], r#"php 'say "hi"'"#),
            (&["php", ""], "php ''"),
            (&["php", "$HOME"], "php '$HOME'"),
            (&["php", r"a\b"], r"php 'a\b'"),
            (
                &[r"C:\Program Files\php\php.exe"],
                r"'C:\Program Files\php\php.exe'",
            ),
            (&["php", "Füße"], "php 'Füße'"),
            (
                &["php", "a,b", "@x", "%PATH%", "a;b", "a*"],
                "php 'a,b' '@x' '%PATH%' 'a;b' 'a*'",
            ),
        ] {
            assert_eq!(posix(&line(words)), shown);
        }
    }

    #[cfg(feature = "run")]
    #[test]
    fn powershell_quotes_what_powershell_would_read() {
        for (words, shown) in [
            (
                &["php", "module:make", "--name=Blog"][..],
                "php module:make --name=Blog",
            ),
            (&["php", "My Blog"], "php 'My Blog'"),
            (
                &["php", "it's", "it\u{2019}s"],
                "php 'it''s' 'it\u{2019}\u{2019}s'",
            ),
            (&["php", r#"say "hi""#], r#"php 'say "hi"'"#),
            (&["php", ""], "php ''"),
            (&["php", "$HOME"], "php '$HOME'"),
            (&["php", r"a\b"], r"php a\b"),
            (
                &[r"C:\Program Files\php\php.exe"],
                r"& 'C:\Program Files\php\php.exe'",
            ),
            (&["php", "Füße"], "php 'Füße'"),
            (
                &["php", "a,b", "@x", "%PATH%", "a;b", "a*"],
                "php 'a,b' '@x' '%PATH%' 'a;b' 'a*'",
            ),
        ] {
            assert_eq!(powershell(&line(words)), shown);
        }
    }
}
