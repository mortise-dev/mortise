//! The push document: one read of an app, in the shape that Mortise Cloud takes.
//!
//! The shape is a contract with the cloud (`mortise-cloud/docs/spec/02-snapshot-contract.md`,
//! version 1). A member that is removed, renamed or changes its type needs a new
//! [`SCHEMA_VERSION`]. `tests/push.rs` compares the document of a fixture with a committed file.

mod past;
mod repository;

use std::path::{Component, Path, PathBuf};

use serde::Serialize;

use crate::app::{AppSnapshot, CoverageReport, lexical};
use crate::boundary::links::{Cycle, Link};
use crate::boundary::rules::{Rules, RulesSource};
use crate::findings::{Detail, Findings};
use crate::modules::composer::Version;
use crate::modules::layout::Layout;
use crate::modules::module::Module;

pub use past::{PastCommit, PastError, read_past_app, weekly_commits};
pub use repository::{Repository, commit_time, read_repository};

/// The version of the push document that this Mortise writes.
pub const SCHEMA_VERSION: u32 = 1;

/// The most modules, links, references and findings that the cloud takes.
const MAX_MODULES: usize = 2_000;
const MAX_LINKS: usize = 50_000;
const MAX_REFERENCES: usize = 100_000;
const MAX_FINDINGS: usize = 50_000;
/// The longest path or class name of a reference, in characters.
pub const MAX_TEXT: usize = 1_024;
/// The largest document, as JSON before compression, in bytes.
const MAX_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;
/// The largest body as sent, after gzip, in bytes.
const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

/// What `mortise push` sends: an envelope, and the snapshot of the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PushDocument {
    pub schema_version: u32,
    /// When Mortise read the app, RFC 3339.
    pub generated_at: String,
    pub tool: PushTool,
    /// The app folder relative to the repository root, `/` between parts. `.` at the root.
    pub app_path: String,
    pub git: PushGit,
    pub ci: PushCi,
    /// Set only when this run deploys the commit, such as `production`.
    pub environment: Option<String>,
    /// Set only by a run that sends past commits. `None` on every other push.
    pub backfill: Option<PushBackfill>,
    pub snapshot: PushSnapshot,
}

/// The program that wrote the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PushTool {
    /// Always `mortise`.
    pub name: String,
    pub version: String,
}

/// The commit that was read, and the pull request it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PushGit {
    /// `owner/name` on GitHub.
    pub repository: Option<String>,
    /// The full SHA of the commit that was read. On a GitHub pull request, the merge commit.
    pub commit: String,
    /// The last commit of the pull request branch.
    pub head_commit: Option<String>,
    /// The branch pushed to, or the head branch of the pull request.
    pub branch: Option<String>,
    pub base_branch: Option<String>,
    /// The tip of the base branch that the merge commit was made with.
    pub base_commit: Option<String>,
    pub pull_request: Option<u64>,
    /// True when the clone was shallow. Churn is then left out.
    pub shallow: bool,
    /// The committer date of `commit`, RFC 3339 in UTC. `None` when git gives none.
    pub committed_at: Option<String>,
}

/// The place of a document in a run that sends past commits: number `position` of the
/// `of` documents the run plans to send. Both count from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PushBackfill {
    pub position: u32,
    pub of: u32,
}

/// The CI run, for a link back from the cloud.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PushCi {
    /// `github-actions` or `other`.
    pub provider: String,
    pub run_id: Option<String>,
    pub run_attempt: Option<u32>,
    pub run_url: Option<String>,
}

/// The envelope of a document, as the caller knows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushContext {
    pub generated_at: String,
    /// The version of Mortise that read the app.
    pub tool_version: String,
    pub app_path: String,
    pub git: PushGit,
    pub ci: PushCi,
    pub environment: Option<String>,
    pub backfill: Option<PushBackfill>,
}

/// [`AppSnapshot`] without what must not leave CI: no absolute path, nothing from `.env`,
/// no state of the working copy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PushSnapshot {
    pub name: String,
    pub laravel: Option<Version>,
    pub package: Version,
    pub actuator: Option<Version>,
    /// Its folders relative to the app.
    pub layout: Layout,
    /// Each folder relative to the app. `uncommitted` is always `None`.
    pub modules: Vec<Module>,
    /// Its path relative to the app.
    pub coverage_report: Option<CoverageReport>,
    pub links: Vec<Link>,
    pub cycles: Vec<Cycle>,
    pub rules: Rules,
    pub rules_source: RulesSource,
    pub coverage_ignored: Option<String>,
    pub findings: Findings,
    pub coverage_limit: u8,
}

/// Builds the push document of `app`.
///
/// Every field of the snapshot is named here, so a new one does not compile until it is
/// kept or dropped.
pub fn push_document(app: AppSnapshot, context: PushContext) -> PushDocument {
    let AppSnapshot {
        root,
        name,
        laravel,
        package,
        actuator,
        app_url: _,
        app_env: _,
        layout,
        modules,
        coverage_report,
        git: _,
        links,
        cycles,
        rules,
        rules_source,
        coverage_ignored,
        findings,
        coverage_limit,
        package_data: _,
    } = app;
    let shallow = context.git.shallow;
    let layout = Layout {
        modules_dir: relative(&root, &layout.modules_dir),
        statuses_file: relative(&root, &layout.statuses_file),
        ..layout
    };
    let modules = modules
        .into_iter()
        .map(|module| Module {
            dir: relative(&root, &module.dir),
            // Churn from a shallow clone counts only the commits it has.
            churn: if shallow { None } else { module.churn },
            // Files not committed mean nothing in CI.
            uncommitted: None,
            ..module
        })
        .collect();
    let coverage_report = coverage_report.map(|report| CoverageReport {
        path: relative(&root, &report.path),
        ..report
    });
    // The core writes a path inside the app relative to it, and one outside it, such as
    // in a modules folder elsewhere, as it is. Every one becomes relative here.
    let mut links = links;
    for reference in links.iter_mut().flat_map(|link| &mut link.refs) {
        relative_text(&root, &mut reference.file);
    }
    let mut findings = findings;
    for finding in &mut findings.list {
        match &mut finding.detail {
            Detail::InvalidManifest { file, .. }
            | Detail::NoManifestName { file }
            | Detail::StaleStatus { file, .. } => relative_text(&root, file),
            Detail::NamespaceMismatch { folder, .. } => relative_text(&root, folder),
            Detail::CircularUse { .. }
            | Detail::ReadsInternals { .. }
            | Detail::OutsideAllowedUse { .. }
            | Detail::UnknownAllowedModule { .. }
            | Detail::LowCoverage { .. }
            | Detail::MissingTranslations { .. } => {}
        }
    }
    PushDocument {
        schema_version: SCHEMA_VERSION,
        generated_at: context.generated_at,
        tool: PushTool {
            name: "mortise".to_owned(),
            version: context.tool_version,
        },
        app_path: context.app_path,
        git: context.git,
        ci: context.ci,
        environment: context.environment,
        backfill: context.backfill,
        snapshot: PushSnapshot {
            name,
            laravel,
            package,
            actuator,
            layout,
            modules,
            coverage_report,
            links,
            cycles,
            rules,
            rules_source,
            coverage_ignored,
            findings,
            coverage_limit,
        },
    }
}

/// What the cloud would refuse for its size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushLimit {
    /// More items than the cloud takes.
    TooMany {
        what: Counted,
        count: usize,
        limit: usize,
    },
    /// A path, or a class name of a reference, longer than [`MAX_TEXT`] characters.
    TooLong { text: String },
}

/// What a [`PushLimit::TooMany`] counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counted {
    Modules,
    Links,
    References,
    Findings,
    /// Bytes of the JSON before compression.
    DocumentBytes,
    /// Bytes of the body as sent.
    BodyBytes,
}

impl PushDocument {
    /// The first limit of the contract that the document is over, if any. The sizes in
    /// bytes are checked by [`byte_limit`] once the document is written.
    pub fn over_limit(&self) -> Option<PushLimit> {
        let snapshot = &self.snapshot;
        let references = snapshot.links.iter().map(|link| link.refs.len()).sum();
        let counts = [
            (Counted::Modules, snapshot.modules.len(), MAX_MODULES),
            (Counted::Links, snapshot.links.len(), MAX_LINKS),
            (Counted::References, references, MAX_REFERENCES),
            (
                Counted::Findings,
                snapshot.findings.list.len(),
                MAX_FINDINGS,
            ),
        ];
        if let Some((what, count, limit)) = counts.into_iter().find(|(_, n, max)| n > max) {
            return Some(PushLimit::TooMany { what, count, limit });
        }
        // Every path the schema bounds, and the class names of references.
        let path = |path: &PathBuf| path.to_string_lossy().into_owned();
        let layout = [&snapshot.layout.modules_dir, &snapshot.layout.statuses_file];
        let finding_paths =
            snapshot
                .findings
                .list
                .iter()
                .filter_map(|finding| match &finding.detail {
                    Detail::InvalidManifest { file, .. }
                    | Detail::NoManifestName { file }
                    | Detail::StaleStatus { file, .. } => Some(file.clone()),
                    Detail::NamespaceMismatch { folder, .. } => Some(folder.clone()),
                    _ => None,
                });
        std::iter::once(self.app_path.clone())
            .chain(layout.into_iter().map(path))
            .chain(snapshot.modules.iter().map(|module| path(&module.dir)))
            .chain(
                snapshot
                    .coverage_report
                    .iter()
                    .map(|report| path(&report.path)),
            )
            .chain(
                snapshot
                    .links
                    .iter()
                    .flat_map(|link| &link.refs)
                    .flat_map(|reference| [reference.file.clone(), reference.class.clone()]),
            )
            .chain(finding_paths)
            .find(|text| text.chars().count() > MAX_TEXT)
            .map(|text| PushLimit::TooLong { text })
    }
}

/// The limit a written document is over: `document` bytes of JSON, `body` bytes as sent.
pub fn byte_limit(document: usize, body: usize) -> Option<PushLimit> {
    let too_many =
        |what, count, limit| (count > limit).then_some(PushLimit::TooMany { what, count, limit });
    too_many(Counted::DocumentBytes, document, MAX_DOCUMENT_BYTES)
        .or_else(|| too_many(Counted::BodyBytes, body, MAX_BODY_BYTES))
}

/// `path` relative to `root`, with `/` between parts. A path outside `root` climbs out
/// with `..`. Without a common start, such as on another Windows drive, only the file
/// name is kept: no absolute path leaves CI.
fn relative(root: &Path, path: &Path) -> PathBuf {
    // Parts compare as components, so `//server/share` is the same start as `\\server\share`.
    let root = lexical(root);
    let path = lexical(path);
    let root: Vec<Component<'_>> = root.components().collect();
    let path: Vec<Component<'_>> = path.components().collect();
    let common = root.iter().zip(&path).take_while(|(a, b)| a == b).count();
    let name = |part: &Component<'_>| part.as_os_str().to_string_lossy().into_owned();
    if common == 0 {
        return PathBuf::from(path.last().map(name).unwrap_or_else(|| ".".into()));
    }
    let mut relative: Vec<String> = vec!["..".into(); root.len() - common];
    relative.extend(path[common..].iter().map(name));
    if relative.is_empty() {
        return PathBuf::from(".");
    }
    PathBuf::from(relative.join("/"))
}

/// `text`, a path as the core writes it, made relative to `root` when it is absolute.
fn relative_text(root: &Path, text: &mut String) {
    let path = Path::new(text.as_str());
    if path.is_absolute() || path.has_root() {
        *text = relative(root, path).to_string_lossy().into_owned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_outside_the_app_climbs_out_and_never_stays_absolute() {
        let root = Path::new("/ci/repo/app");
        let shown = |path: &str| relative(root, Path::new(path));
        assert_eq!(
            shown("/ci/repo/app/Modules/Billing"),
            Path::new("Modules/Billing")
        );
        assert_eq!(shown("/ci/repo/app"), Path::new("."));
        assert_eq!(
            shown("/ci/repo/build/../coverage/clover.xml"),
            Path::new("../coverage/clover.xml")
        );
        // A relative path has no common start with an absolute root.
        assert_eq!(shown("clover.xml"), Path::new("clover.xml"));
    }

    #[cfg(windows)]
    #[test]
    fn a_unc_path_as_the_core_writes_it_keeps_its_folders() {
        let root = Path::new(r"\\server\share\repo");
        let mut text = "//server/share/shared/Modules/Billing/app/Invoice.php".to_owned();
        relative_text(root, &mut text);
        assert_eq!(text, "../shared/Modules/Billing/app/Invoice.php");
    }
}
