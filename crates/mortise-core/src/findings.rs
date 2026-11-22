//! Findings: what Mortise works out from the files of an app.
//!
//! A finding is a problem (it breaks a rule or a check) or a warning (it is under a
//! limit the app set). Findings are never called health: that word belongs to what
//! the running app reports.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::links::{Cycle, Link};
use crate::module::{Manifest, Module, ModuleState};
use crate::rules::Rules;

/// The coverage limit of an app until it is changed: a module under it is a warning.
pub const DEFAULT_COVERAGE_LIMIT: u8 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Breaks a rule or a check.
    Problem,
    /// Under a limit the app set.
    Warning,
}

/// What Mortise checks. The first two are the boundary rules of the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Check {
    NoCircularUse,
    PublicSurfaceOnly,
    ValidManifest,
    NamespaceMatchesFolder,
    NoStaleStatus,
    CoverageLimit,
}

impl Check {
    /// True for the boundary rules, false for the checks on files.
    pub fn is_rule(self) -> bool {
        matches!(self, Check::NoCircularUse | Check::PublicSurfaceOnly)
    }
}

/// What exactly was found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Detail {
    /// Modules use each other. `files` has, for each module of the circle, how
    /// many of its files use another module of it.
    CircularUse { files: Vec<(String, usize)> },
    /// Files of the module read classes of another module that are not part of
    /// its public surface.
    ReadsInternals {
        of: String,
        files: usize,
        classes: Vec<String>,
    },
    /// `module.json` is not valid JSON. The message is the parser's.
    InvalidManifest {
        file: String,
        message: String,
        line: usize,
    },
    /// `module.json` has no `name`.
    NoManifestName { file: String },
    /// The namespace the module declares does not end in the name of its folder.
    NamespaceMismatch { namespace: String, folder: String },
    /// The statuses file lists a module that has no folder.
    StaleStatus { entry: String, file: String },
    /// The module is covered less than the limit of the app.
    LowCoverage { percent: u8, limit: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub check: Check,
    /// The module the finding is counted on. `None` for a finding of the app itself.
    pub module: Option<String>,
    pub detail: Detail,
}

/// A check that ran, with how many things it looked at and how many findings it gave.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub check: Check,
    /// Modules for most checks; entries for the statuses file.
    pub checked: usize,
    pub findings: usize,
}

/// The finding state of a module: its worst finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingState {
    /// At least one problem, with how many.
    Problems(usize),
    Warning,
    Good,
}

/// The findings of an app and the checks they came from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Findings {
    /// Problems first, then warnings; within each by module.
    pub list: Vec<Finding>,
    /// Every check that ran, in the order of [`Check`].
    pub checks: Vec<CheckResult>,
}

impl Findings {
    pub fn problems(&self) -> usize {
        self.list
            .iter()
            .filter(|finding| finding.severity == Severity::Problem)
            .count()
    }

    pub fn warnings(&self) -> usize {
        self.list
            .iter()
            .filter(|finding| finding.severity == Severity::Warning)
            .count()
    }

    /// The findings counted on one module.
    pub fn of<'a>(&'a self, module: &'a str) -> impl Iterator<Item = &'a Finding> {
        self.list
            .iter()
            .filter(move |finding| finding.module.as_deref() == Some(module))
    }

    /// The worst finding of one module.
    pub fn state_of(&self, module: &str) -> FindingState {
        let problems = self
            .of(module)
            .filter(|finding| finding.severity == Severity::Problem)
            .count();
        if problems > 0 {
            FindingState::Problems(problems)
        } else if self.of(module).next().is_some() {
            FindingState::Warning
        } else {
            FindingState::Good
        }
    }

    /// How many rules and how many checks ran.
    pub fn rules_and_checks(&self) -> (usize, usize) {
        let rules = self
            .checks
            .iter()
            .filter(|result| result.check.is_rule())
            .count();
        (rules, self.checks.len() - rules)
    }
}

/// What the checks look at.
pub(crate) struct Subject<'a> {
    pub(crate) root: &'a Path,
    pub(crate) modules: &'a [Module],
    pub(crate) links: &'a [Link],
    pub(crate) cycles: &'a [Cycle],
    pub(crate) rules: &'a Rules,
    /// The names in the statuses file, and the file as the user writes it.
    pub(crate) status_entries: &'a [String],
    pub(crate) statuses_file: &'a str,
    /// Without a coverage report the coverage limit is not checked.
    pub(crate) has_coverage_report: bool,
    pub(crate) coverage_limit: u8,
}

/// Runs every check.
pub(crate) fn find(subject: &Subject<'_>) -> Findings {
    let mut list: Vec<Finding> = Vec::new();
    let mut checks: Vec<CheckResult> = Vec::new();
    let modules = subject.modules;
    let shown = |path: &Path| -> String {
        path.strip_prefix(subject.root)
            .unwrap_or(path)
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
    };
    let mut ran = |check: Check, checked: usize, found: Vec<Finding>| {
        checks.push(CheckResult {
            check,
            checked,
            findings: found.len(),
        });
        list.extend(found);
    };

    if subject.rules.no_circular_use {
        // A circle is counted once, on the module with fewer files in it.
        let found = subject
            .cycles
            .iter()
            .map(|cycle| Finding {
                severity: Severity::Problem,
                check: Check::NoCircularUse,
                module: Some(cycle.carrier.clone()),
                detail: Detail::CircularUse {
                    files: cycle.files.clone(),
                },
            })
            .collect();
        ran(Check::NoCircularUse, modules.len(), found);
    }

    if subject.rules.public_surface_only {
        let found = subject
            .links
            .iter()
            .filter(|link| link.internal_files() > 0)
            .map(|link| {
                let mut classes: Vec<String> = link
                    .refs
                    .iter()
                    .filter(|reference| reference.internal)
                    .map(|reference| reference.class.clone())
                    .collect();
                classes.sort();
                classes.dedup();
                Finding {
                    severity: Severity::Problem,
                    check: Check::PublicSurfaceOnly,
                    module: Some(link.from.clone()),
                    detail: Detail::ReadsInternals {
                        of: link.to.clone(),
                        files: link.internal_files(),
                        classes,
                    },
                }
            })
            .collect();
        ran(Check::PublicSurfaceOnly, modules.len(), found);
    }

    let found = modules
        .iter()
        .filter_map(|module| {
            let file = shown(&module.dir.join("module.json"));
            let detail = match &module.manifest {
                Manifest::Valid => return None,
                Manifest::InvalidJson { message, line } => Detail::InvalidManifest {
                    file,
                    message: message.clone(),
                    line: *line,
                },
                Manifest::NoName => Detail::NoManifestName { file },
            };
            Some(Finding {
                severity: Severity::Problem,
                check: Check::ValidManifest,
                module: Some(module.name.clone()),
                detail,
            })
        })
        .collect();
    ran(Check::ValidManifest, modules.len(), found);

    let found = modules
        .iter()
        .filter(|module| module.manifest == Manifest::Valid)
        .filter_map(|module| {
            let folder = module.dir.file_name()?.to_str()?;
            let last = module.namespace.rsplit('\\').next().unwrap_or_default();
            (last != folder).then(|| Finding {
                severity: Severity::Problem,
                check: Check::NamespaceMatchesFolder,
                module: Some(module.name.clone()),
                detail: Detail::NamespaceMismatch {
                    namespace: module.namespace.clone(),
                    folder: shown(&module.dir),
                },
            })
        })
        .collect();
    ran(Check::NamespaceMatchesFolder, modules.len(), found);

    let found = subject
        .status_entries
        .iter()
        .filter(|entry| !modules.iter().any(|module| &module.name == *entry))
        .map(|entry| Finding {
            severity: Severity::Problem,
            check: Check::NoStaleStatus,
            module: None,
            detail: Detail::StaleStatus {
                entry: entry.clone(),
                file: subject.statuses_file.to_owned(),
            },
        })
        .collect();
    ran(Check::NoStaleStatus, subject.status_entries.len(), found);

    // Without a report coverage is unknown, which is not a finding.
    if subject.has_coverage_report {
        let found = modules
            .iter()
            .filter(|module| module.state != ModuleState::Off)
            .filter_map(|module| {
                let percent = module.coverage?.percent()?;
                (percent < subject.coverage_limit).then(|| Finding {
                    severity: Severity::Warning,
                    check: Check::CoverageLimit,
                    module: Some(module.name.clone()),
                    detail: Detail::LowCoverage {
                        percent,
                        limit: subject.coverage_limit,
                    },
                })
            })
            .collect();
        ran(Check::CoverageLimit, modules.len(), found);
    }

    // Problems first, then warnings; within each by module, the app's own last.
    list.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .then_with(|| a.module.is_none().cmp(&b.module.is_none()))
            .then_with(|| a.module.cmp(&b.module))
            .then_with(|| a.check.cmp(&b.check))
    });
    Findings { list, checks }
}
