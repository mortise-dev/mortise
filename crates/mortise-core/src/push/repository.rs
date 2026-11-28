//! What git says about the commit a push document is read from.

use std::path::Path;

use crate::modules::git::{git, git_local};

/// The git facts of the envelope, for the app in a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    /// The app folder relative to the top of the repository, `/` between parts.
    /// `.` when the app is the top.
    pub app_path: String,
    /// The full SHA of `HEAD`. `None` before the first commit.
    pub commit: Option<String>,
    /// The branch that is checked out. `None` when `HEAD` is detached.
    pub branch: Option<String>,
    /// True when the clone is shallow.
    pub shallow: bool,
}

/// Reads the git facts of the app in `root`. `None` when git cannot be started or the
/// folder is not in a repository. Reads only, as the churn of the modules does.
pub fn read_repository(root: &Path) -> Option<Repository> {
    let text = |args: &[&str]| {
        git(root, args)
            .map(|out| out.trim().to_owned())
            .filter(|out| !out.is_empty())
    };
    // Both canonical, so that a link in either path, such as `/var` on macOS, agrees.
    let top = std::fs::canonicalize(text(&["rev-parse", "--show-toplevel"])?).ok()?;
    let app = std::fs::canonicalize(root).ok()?;
    let parts: Vec<String> = app
        .strip_prefix(&top)
        .ok()?
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    Some(Repository {
        app_path: if parts.is_empty() {
            ".".to_owned()
        } else {
            parts.join("/")
        },
        commit: text(&["rev-parse", "-q", "--verify", "HEAD"]),
        branch: text(&["symbolic-ref", "--short", "-q", "HEAD"]),
        // Git before 2.15 prints the flag back; that is not "true".
        shallow: text(&["rev-parse", "--is-shallow-repository"]).as_deref() == Some("true"),
    })
}

/// When `commit` was committed, in seconds since 1970: its committer date, not its author
/// date, which a rebase keeps from the first draft. `commit` is a full SHA. `None` when the
/// folder is in no repository, the repository does not hold the commit, or the SHA names
/// another object, such as a tag. A clone that can download a missing commit does not.
pub fn commit_time(root: &Path, commit: &str) -> Option<i64> {
    // Only a SHA reaches git: a name from the environment could read as an option.
    if commit.is_empty() || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let found = git_local(
        root,
        &[
            // The date of the commit that the document names, not of one that stands in for it.
            "--no-replace-objects",
            "show",
            "-s",
            // A setting can print the state of the signature before the date.
            "--no-show-signature",
            "--format=%H %ct",
            &format!("{commit}^{{commit}}"),
            // A commit, not a file of that name.
            "--",
        ],
    )?;
    let (id, time) = found.trim().split_once(' ')?;
    // Git peels a tag to its commit and completes the start of a longer SHA: neither
    // is the commit that the document names.
    if !id.eq_ignore_ascii_case(commit) {
        return None;
    }
    time.parse().ok()
}
