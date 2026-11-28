//! What git knows about an app: the branch, and how often each module changed.
//!
//! Mortise runs the `git` program and only reads. Without git, or outside a
//! repository, this data is absent and nothing else is affected.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde::{Deserialize, Serialize};

/// The window for "commits in 30 days".
pub const CHURN_DAYS: u32 = 30;

/// The git state of an app folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Git {
    /// The folder is in a repository.
    Repository {
        /// The branch that is checked out. `None` when HEAD is detached.
        branch: Option<String>,
        /// The short id of the commit that is checked out. `None` before the first commit.
        commit: Option<String>,
    },
    /// The folder is not in a git repository.
    NotARepository,
    /// The `git` program could not be started.
    NoGit,
}

/// How often a folder changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Churn {
    /// Commits that touched the folder in the last [`CHURN_DAYS`] days.
    pub commits: u32,
    /// When the folder was last touched by a commit, in seconds since 1970.
    /// `None` when no commit ever touched it.
    pub last_commit: Option<i64>,
}

/// Git in `root` with `args`, set up so that it only reads.
fn command(root: &Path, args: &[&str]) -> std::process::Command {
    let mut command = crate::program::command("git");
    command
        .arg("-C")
        .arg(root)
        // Paths as they are, not quoted and escaped.
        .args(["--no-pager", "-c", "core.quotepath=false"])
        // Never start a file system monitor that the repository configures.
        .args(["-c", "core.fsmonitor=false"])
        .args(args)
        // Never take a lock or write the index: Mortise only reads.
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        // A program that git starts, such as one that checks a signature, gets no token.
        .env_remove("MORTISE_TOKEN")
        .stderr(Stdio::null());
    command
}

/// The output of a git command, or `None` when it fails.
fn output(mut command: std::process::Command) -> Option<String> {
    let output = command.output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Runs git in `root` and gives its output, or `None` when it fails.
pub(crate) fn git(root: &Path, args: &[&str]) -> Option<String> {
    output(command(root, args))
}

/// As [`git`], for a question about one object: git answers from the objects the
/// checkout holds and downloads none. A partial clone would otherwise fetch a missing
/// object from its remote and write it. Git before 2.34 does not know
/// `GIT_NO_LAZY_FETCH`, so no transport is allowed either, which every Git obeys.
#[cfg(feature = "push")]
pub(crate) fn git_local(root: &Path, args: &[&str]) -> Option<String> {
    let mut command = command(root, args);
    command
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_ALLOW_PROTOCOL", "none");
    output(command)
}

/// Writes the files of `tree`, a tree of the repository of `root`, into the folder `into`,
/// which is outside the repository. `index` is a file for git to keep beside it. The
/// working tree and the index of the repository stay as they are. A clone without file
/// contents downloads the files of this tree it lacks, in one request. False when git fails.
///
/// The files are those of a checkout of the tree, without what could run a program or
/// lead out of `into`: no filter program and no hook of the computer runs, and a link is
/// written as a file that holds its target. `no_hooks` is a folder that does not exist.
#[cfg(feature = "push")]
pub(crate) fn export(root: &Path, tree: &str, into: &Path, index: &Path, no_hooks: &Path) -> bool {
    // Every filter that a setting of the computer or the repository names, such as the
    // one of Git LFS. An empty program is no filter.
    let filters: std::collections::BTreeSet<String> = git(
        root,
        &["config", "--name-only", "--get-regexp", r"^filter\."],
    )
    .unwrap_or_default()
    .lines()
    .filter_map(|key| Some(key.strip_prefix("filter.")?.rsplit_once('.')?.0.to_owned()))
    .collect();
    let mut settings = vec![
        // Every file of the tree, whatever the checkout leaves out.
        "core.sparseCheckout=false".to_owned(),
        // The bytes as they are committed.
        "core.autocrlf=false".to_owned(),
        "core.symlinks=false".to_owned(),
        // The index is one file beside `into`, with no part in the repository.
        "core.splitIndex=false".to_owned(),
        // A hook would run with the environment of this process, which holds the token.
        format!("core.hooksPath={}", no_hooks.display()),
    ];
    for filter in &filters {
        settings.push(format!("filter.{filter}.smudge="));
        settings.push(format!("filter.{filter}.process="));
        settings.push(format!("filter.{filter}.required=false"));
    }
    let mut args: Vec<&str> = settings
        .iter()
        .flat_map(|setting| ["-c", setting.as_str()])
        .collect();
    args.extend(["--no-replace-objects", "read-tree", "-m", "-u", tree]);
    let mut command = command(root, &args);
    command
        .env("GIT_INDEX_FILE", index)
        .env("GIT_WORK_TREE", into)
        .stdout(Stdio::null());
    command.status().is_ok_and(|status| status.success())
}

/// Reads the branch and commit of the app in `root`.
pub(crate) fn read(root: &Path) -> Git {
    let started = crate::program::command("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok();
    if !started {
        return Git::NoGit;
    }
    if git(root, &["rev-parse", "--is-inside-work-tree"])
        .as_deref()
        .map(str::trim)
        != Some("true")
    {
        return Git::NotARepository;
    }
    let text = |args: &[&str]| {
        git(root, args)
            .map(|out| out.trim().to_owned())
            .filter(|out| !out.is_empty())
    };
    Git::Repository {
        // Works before the first commit too, and fails for a detached HEAD.
        branch: text(&["symbolic-ref", "--short", "-q", "HEAD"]),
        commit: text(&["rev-parse", "--short", "-q", "--verify", "HEAD"]),
    }
}

/// Counts the commits of the last [`CHURN_DAYS`] days for each folder, and finds
/// when each was last touched. `folders` are paths relative to `root`, with `/`.
pub(crate) fn churn(root: &Path, folders: &[String]) -> BTreeMap<String, Churn> {
    churn_from(root, "HEAD", &format!("{CHURN_DAYS}.days"), None, folders)
}

/// As [`churn`], as it was at `commit`, a full SHA committed at `time`: the commits of
/// the [`CHURN_DAYS`] days before it, in its own history.
#[cfg(feature = "push")]
pub(crate) fn churn_at(
    root: &Path,
    commit: &str,
    time: i64,
    folders: &[String],
) -> BTreeMap<String, Churn> {
    // A number of this size is seconds since 1970 to git.
    let since = time - i64::from(CHURN_DAYS) * 86_400;
    // An older commit can carry a later date, such as after a clock was set right. It is
    // not in the 30 days before this one.
    let until = time.to_string();
    churn_from(root, commit, &since.to_string(), Some(&until), folders)
}

/// The end of a `git log` call that gives each commit as a marker line with its time,
/// then the files it touched, relative to the app folder.
const LOG_OF_FILES: [&str; 6] = [
    "--format=%x01%ct",
    // A setting can print the state of the signature, and start a program for it.
    "--no-show-signature",
    "--name-only",
    "--relative",
    // A moved file counts for the folder it left and the one it joined. Finding a
    // rename compares file contents, which a clone without blobs has to download.
    "--no-renames",
    "--",
];

/// The churn of `folders` in the history of `from`, counting the commits after `since`,
/// a date as `git log --since` takes it. With `until`, the history is that of a past
/// commit: no commit after that date counts, and a commit dated before `since` does not
/// hide older commits that are dated after it, such as after a clock was set right.
fn churn_from(
    root: &Path,
    from: &str,
    since: &str,
    until: Option<&str>,
    folders: &[String],
) -> BTreeMap<String, Churn> {
    let mut churn: BTreeMap<String, Churn> = BTreeMap::new();
    if folders.is_empty() {
        return churn;
    }

    // A past commit is counted in its own history, not in one that stands in for it.
    let original: &[&str] = match until {
        Some(_) => &["--no-replace-objects"],
        None => &[],
    };
    let until = until.map(|until| format!("--until={until}"));
    // Git from 2.37 can walk the whole history and keep the commits after a date. An
    // older Git, and the count of the checkout, stop at the first commit before it.
    let whole = format!("--since-as-filter={since}");
    let since = format!("--since={since}");
    let log = |since: &str| {
        let mut args = original.to_vec();
        args.extend(["log", from, since]);
        args.extend(until.as_deref());
        args.extend(LOG_OF_FILES);
        args.extend(folders.iter().map(String::as_str));
        git(root, &args)
    };
    let log = until
        .as_ref()
        .and_then(|_| log(&whole))
        .or_else(|| log(&since))
        .unwrap_or_default();
    for commit in log.split('\u{1}').skip(1) {
        let mut lines = commit.lines();
        let Some(time) = lines
            .next()
            .and_then(|time| time.trim().parse::<i64>().ok())
        else {
            continue;
        };
        let mut touched: Vec<&String> = lines
            .filter(|file| !file.is_empty())
            .filter_map(|file| {
                folders.iter().find(|folder| {
                    file.strip_prefix(folder.as_str())
                        .is_some_and(|rest| rest.starts_with('/'))
                })
            })
            .collect();
        touched.sort();
        touched.dedup();
        for folder in touched {
            let entry = churn.entry(folder.clone()).or_default();
            entry.commits += 1;
            // The log is newest first, but a merge can bring an older time.
            entry.last_commit = entry.last_commit.max(Some(time));
        }
    }

    // A folder that was quiet in the window: when was it touched at all?
    for folder in folders {
        if churn.contains_key(folder) {
            continue;
        }
        let mut args = original.to_vec();
        args.extend(["log", "-1", "--format=%ct", "--no-show-signature", from]);
        args.extend(until.as_deref());
        args.extend(["--", folder]);
        let last = git(root, &args).and_then(|time| time.trim().parse().ok());
        churn.insert(
            folder.clone(),
            Churn {
                commits: 0,
                last_commit: last,
            },
        );
    }
    churn
}

/// For each folder in `dirs`, the files git cannot restore: changed, staged,
/// added, deleted and untracked. Ignored files are left out. `None` for a folder
/// git cannot tell about: outside the repository of `root`, a repository of its
/// own such as a submodule, or every folder when git fails.
pub(crate) fn uncommitted(root: &Path, dirs: &[PathBuf]) -> Vec<Option<u32>> {
    let unknown = vec![None; dirs.len()];
    let Some(top) = git(root, &["rev-parse", "--show-toplevel"])
        .and_then(|top| std::fs::canonicalize(top.trim_end_matches(['\n', '\r'])).ok())
    else {
        return unknown;
    };
    // Each folder relative to the top of the repository, with `/`, as git prints
    // the paths of a status. A folder in a repository of its own, such as a
    // submodule, has a `.git` in it or above it below the top: this status does not
    // list its files.
    let relative: Vec<Option<String>> = dirs
        .iter()
        .map(|dir| {
            let dir = std::fs::canonicalize(dir).ok()?;
            let nested = dir
                .ancestors()
                .take_while(|folder| *folder != top)
                .any(|folder| folder.join(".git").exists());
            if nested {
                return None;
            }
            let parts: Vec<String> = dir
                .strip_prefix(&top)
                .ok()?
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect();
            (!parts.is_empty()).then(|| parts.join("/"))
        })
        .collect();
    let mut counts: Vec<Option<u32>> = relative
        .iter()
        .map(|folder| folder.as_ref().map(|_| 0))
        .collect();
    let mut args = vec![
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
        "--no-renames",
        "--",
    ];
    args.extend(relative.iter().flatten().map(String::as_str));
    if args.len() == 6 {
        return counts;
    }
    let Some(status) = git(&top, &args) else {
        return unknown;
    };
    for entry in status.split('\0') {
        // `XY path`: two status letters, a space, the path from the top.
        let Some(path) = entry.get(3..) else {
            continue;
        };
        for (folder, count) in relative.iter().zip(counts.iter_mut()) {
            let Some(rest) = folder
                .as_deref()
                .and_then(|folder| path.strip_prefix(folder))
            else {
                continue;
            };
            match rest {
                // The folder itself is listed: it is a repository of its own, whose
                // files this status does not list.
                "" | "/" => *count = None,
                _ if rest.starts_with('/') => {
                    if let Some(count) = count {
                        *count += 1;
                    }
                }
                _ => {}
            }
        }
    }
    counts
}
