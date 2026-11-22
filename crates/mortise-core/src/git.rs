//! What git knows about an app: the branch, and how often each module changed.
//!
//! Mortise runs the `git` program and only reads. Without git, or outside a
//! repository, this data is absent and nothing else is affected.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};

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

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        // Paths as they are, not quoted and escaped.
        .args(["--no-pager", "-c", "core.quotepath=false"])
        .args(args)
        // Never take a lock or write the index: Mortise only reads.
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Reads the branch and commit of the app in `root`.
pub(crate) fn read(root: &Path) -> Git {
    let started = Command::new("git")
        .arg("--version")
        .stdin(Stdio::null())
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
    let mut churn: BTreeMap<String, Churn> = BTreeMap::new();
    if folders.is_empty() {
        return churn;
    }

    // One call for the window: each commit as a marker line with its time, then
    // the files it touched, relative to the app folder.
    let since = format!("--since={CHURN_DAYS}.days");
    let mut args = vec![
        "log",
        &since,
        "--format=%x01%ct",
        "--name-only",
        "--relative",
        "--",
    ];
    args.extend(folders.iter().map(String::as_str));
    let log = git(root, &args).unwrap_or_default();
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
        let last = git(root, &["log", "-1", "--format=%ct", "--", folder])
            .and_then(|time| time.trim().parse().ok());
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
