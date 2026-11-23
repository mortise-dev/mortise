//! How PHP runs for an app: the kind of runner, the command line, and starting it.
//!
//! A command line is a program and a list of arguments. It never goes through a shell.

#[cfg(feature = "run")]
mod detect;
#[cfg(feature = "run")]
pub use detect::{Detected, detect, herd_config_dir};
pub(crate) mod process;

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
