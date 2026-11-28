//! Past commits of the checkout: which ones a backfill reads, and the app as it was at one.
//!
//! Each commit is read on its own. Nothing is compared and nothing is kept: the files of
//! a commit go to a temporary folder outside the app, are read as any app is, and are
//! removed. The working tree and the index of the checkout stay as they are.

use std::collections::BTreeMap;
use std::path::Path;

use crate::app::{AppSnapshot, ReadError, ReadOptions, read_app_cached, shown};
use crate::modules::git::{churn_at, export, git, git_local};
use crate::modules::module::ReadCache;

/// A commit in the history of the checkout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PastCommit {
    /// The full SHA.
    pub commit: String,
    /// Its committer date, in seconds since 1970.
    pub committed_at: i64,
}

/// Why a past commit gives no app.
#[derive(Debug, thiserror::Error)]
pub enum PastError {
    /// The commit has no app folder at the path of the app.
    #[error("the commit has no app folder")]
    NoFolder,
    /// Git could not write the files of the commit, such as when a clone without file
    /// contents cannot download them.
    #[error("git could not give the files of the commit")]
    NoFiles,
    /// The files are there, and are not an app that Mortise reads.
    #[error(transparent)]
    NotAnApp(#[from] ReadError),
}

/// The last commit of each week on the first-parent history of `HEAD`, newest first,
/// among the commits committed at or after `since` (seconds since 1970). A week is an
/// ISO week in UTC, Monday to Sunday, by committer date. `None` when git fails.
///
/// Git answers from the commits the checkout holds and downloads nothing.
pub fn weekly_commits(root: &Path, since: i64) -> Option<Vec<PastCommit>> {
    let list = git_local(
        root,
        &[
            "--no-replace-objects",
            "rev-list",
            "--first-parent",
            // Each line: the committer date, a space, the SHA.
            "--timestamp",
            &format!("--max-age={since}"),
            "HEAD",
            "--",
        ],
    )?;
    let commits = list.lines().filter_map(|line| {
        let (time, commit) = line.split_once(' ')?;
        Some(PastCommit {
            commit: commit.to_owned(),
            committed_at: time.parse().ok()?,
        })
    });
    Some(last_of_each_week(commits, since))
}

/// The week a time is in: weeks counted from Monday 29 December 1969, 00:00 UTC.
fn week(time: i64) -> i64 {
    // 1 January 1970 was a Thursday, 3 days after a Monday.
    (time.div_euclid(86_400) + 3).div_euclid(7)
}

/// Of `commits`, in the order of the history with the newest first, the one with the
/// latest time in each week, newest week first. Of two with the same time, the one
/// nearer to the tip. A commit before `since` is left out.
fn last_of_each_week(commits: impl Iterator<Item = PastCommit>, since: i64) -> Vec<PastCommit> {
    let mut weeks: BTreeMap<i64, PastCommit> = BTreeMap::new();
    for commit in commits.filter(|commit| commit.committed_at >= since) {
        match weeks.entry(week(commit.committed_at)) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(commit);
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                if commit.committed_at > entry.get().committed_at {
                    entry.insert(commit);
                }
            }
        }
    }
    weeks.into_values().rev().collect()
}

/// Reads the app as it was at `commit`. `root` is the app folder in the checkout, and
/// `app_path` its path from the top of the repository, as in
/// [`Repository`](super::Repository).
///
/// The snapshot has no coverage, and the churn of each module counts the 30 days before
/// the commit. Its paths are in a temporary folder that is gone when this returns: make
/// the push document from it, which keeps no absolute path.
///
/// A clone without file contents downloads the files of the app folder at this commit,
/// in one request.
pub fn read_past_app(
    root: &Path,
    app_path: &str,
    commit: &PastCommit,
) -> Result<AppSnapshot, PastError> {
    let sha = commit.commit.as_str();
    // Only a SHA reaches git: another text could read as an option or a path.
    if sha.is_empty() || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(PastError::NoFolder);
    }
    // A clone without file contents still holds the folders of every commit, so it
    // downloads nothing for this.
    let id = |name: &str| {
        git(root, &["rev-parse", "-q", "--verify", name]).map(|id| id.trim().to_owned())
    };
    let named = if app_path == "." {
        sha.to_owned()
    } else {
        format!("{sha}:{app_path}")
    };
    // A folder, not a file of that name.
    let tree = id(&named)
        .and_then(|found| id(&format!("{found}^{{tree}}")))
        .ok_or(PastError::NoFolder)?;

    let folder = tempfile::tempdir().map_err(|_| PastError::NoFiles)?;
    // The app keeps the name of its folder, which the snapshot shows.
    let name = std::fs::canonicalize(root)
        .ok()
        .and_then(|root| root.file_name().map(ToOwned::to_owned))
        .unwrap_or_else(|| "app".into());
    let files = folder.path().join(name);
    std::fs::create_dir(&files).map_err(|_| PastError::NoFiles)?;
    if !export(root, &tree, &files, &folder.path().join("index")) {
        return Err(PastError::NoFiles);
    }

    let options = ReadOptions {
        no_coverage: true,
        ..ReadOptions::default()
    };
    let mut app = read_app_cached(
        &files,
        &BTreeMap::new(),
        &options,
        &mut ReadCache::default(),
    )?;
    // The temporary folder is in no repository. The history is in the checkout.
    let folders: Vec<String> = app
        .modules
        .iter()
        .map(|module| shown(&app.root, &module.dir))
        .collect();
    let churn = churn_at(root, sha, commit.committed_at, &folders);
    for (module, folder) in app.modules.iter_mut().zip(&folders) {
        module.churn = churn.get(folder).copied();
        module.uncommitted = None;
    }
    Ok(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_week_runs_from_monday_to_sunday_in_utc_and_keeps_its_last_commit() {
        let commit = |name: &str, committed_at: i64| PastCommit {
            commit: name.to_owned(),
            committed_at,
        };
        // Monday 23 November 2026, 00:00 UTC.
        let monday = 1_795_392_000;
        let day = 86_400;
        let history = [
            commit("monday-first-second", monday),
            // A merge can bring an older date nearer to the tip.
            commit("friday-before", monday - 3 * day),
            commit("sunday-last-second", monday - 1),
            commit("sunday-same-second-further", monday - 1),
            commit("two-weeks-before", monday - 10 * day),
            commit("before-the-period", monday - 30 * day),
        ];
        let picked = last_of_each_week(history.into_iter(), monday - 20 * day);
        let names: Vec<&str> = picked.iter().map(|commit| commit.commit.as_str()).collect();
        assert_eq!(
            names,
            [
                "monday-first-second",
                "sunday-last-second",
                "two-weeks-before"
            ]
        );
    }
}
