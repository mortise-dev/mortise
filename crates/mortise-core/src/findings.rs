//! Findings: what Mortise works out from the files of an app.
//!
//! A finding is a problem (it breaks a rule or a check) or a warning (it is under a
//! limit the app set). Findings are never called health: that word belongs to what
//! the running app reports.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::boundary::links::{Cycle, Link};
use crate::boundary::rules::Rules;
use crate::modules::module::{Manifest, Module, ModuleState};
use crate::package::PackageData;

/// The coverage limit of an app until it is changed: a module under it is a warning.
pub const DEFAULT_COVERAGE_LIMIT: u8 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Breaks a rule or a check.
    Problem,
    /// Under a limit the app set, or a name in the rules that is no module of the app.
    Warning,
}

/// What Mortise checks. The first three are the boundary rules of the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Check {
    NoCircularUse,
    PublicSurfaceOnly,
    /// Each module with a list in `allowed-use` uses only the modules of its list.
    AllowedUse,
    ValidManifest,
    NamespaceMatchesFolder,
    NoStaleStatus,
    CoverageLimit,
    /// From the report of `module:lang`. It runs only once the user checked translations.
    TranslationsComplete,
}

impl Check {
    /// True for the boundary rules, false for the checks on files.
    pub fn is_rule(self) -> bool {
        matches!(
            self,
            Check::NoCircularUse | Check::PublicSurfaceOnly | Check::AllowedUse
        )
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
    /// Files of the module use a module that its list in `allowed-use` does not name.
    OutsideAllowedUse { of: String, files: usize },
    /// `allowed-use` names a module that the app does not have: as a module with a
    /// list, or in a list.
    UnknownAllowedModule { entry: String },
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
    /// A locale of the module lacks keys or files that its other locales have.
    /// `from` is the other locale when the module has exactly two.
    MissingTranslations {
        locale: String,
        keys: usize,
        files: usize,
        from: Option<String>,
    },
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

/// Work in progress on a module. It is not a finding: it changes no mark and no count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    pub module: String,
    /// How many of its migrations wait to run, from the last migration state read.
    pub waiting: usize,
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
    /// The modules with migrations waiting, by module. Never counted as findings.
    pub todo: Vec<Todo>,
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
    /// What the package said in the reads the user started.
    pub(crate) package: &'a PackageData,
}

/// Runs every check.
pub(crate) fn find(subject: &Subject<'_>) -> Findings {
    let mut list: Vec<Finding> = Vec::new();
    let mut checks: Vec<CheckResult> = Vec::new();
    let modules = subject.modules;
    let shown = |path: &Path| crate::app::shown(subject.root, path);
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

    let in_app = |name: &str| modules.iter().any(|module| module.name == name);
    let allowed_use = subject.rules.allowed_use.as_ref();
    if let Some(allowed) = allowed_use.filter(|allowed| allowed.enabled) {
        let mut found: Vec<Finding> = subject
            .links
            .iter()
            .filter(|link| link.outside_allowed)
            .map(|link| Finding {
                severity: Severity::Problem,
                check: Check::AllowedUse,
                module: Some(link.from.clone()),
                detail: Detail::OutsideAllowedUse {
                    of: link.to.clone(),
                    files: link.files(),
                },
            })
            .collect();
        // A name that is no module, so that a typing mistake does not pass in silence.
        let unknown: BTreeSet<&str> = allowed
            .modules
            .iter()
            .flat_map(|(module, list)| std::iter::once(module).chain(list))
            .map(String::as_str)
            .filter(|name| !in_app(name))
            .collect();
        found.extend(unknown.into_iter().map(|entry| Finding {
            severity: Severity::Warning,
            check: Check::AllowedUse,
            module: None,
            detail: Detail::UnknownAllowedModule {
                entry: entry.to_owned(),
            },
        }));
        let checked = modules
            .iter()
            .filter(|module| allowed.modules.contains_key(&module.name))
            .count();
        ran(Check::AllowedUse, checked, found);
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

    // Only a module of the app counts: the report can be older than a delete.
    if let Some(report) = &subject.package.translations {
        let checked = report
            .modules
            .iter()
            .filter(|module| module.checked && in_app(&module.name))
            .count();
        let found = report
            .modules
            .iter()
            .filter(|module| in_app(&module.name))
            .flat_map(|module| {
                module.locales.iter().filter_map(|(locale, missing)| {
                    let keys = missing.missing_keys.values().map(Vec::len).sum::<usize>();
                    let files = missing.missing_files.len();
                    let from = match module.locales.keys().collect::<Vec<_>>()[..] {
                        [a, b] => Some(if a == locale { b } else { a }.clone()),
                        _ => None,
                    };
                    (keys + files > 0).then(|| Finding {
                        severity: Severity::Warning,
                        check: Check::TranslationsComplete,
                        module: Some(module.name.clone()),
                        detail: Detail::MissingTranslations {
                            locale: locale.clone(),
                            keys,
                            files,
                            from,
                        },
                    })
                })
            })
            .collect();
        // A report with no module of two locales checked nothing: no result.
        if checked > 0 {
            ran(Check::TranslationsComplete, checked, found);
        }
    }

    // The to do list is of the default connection, the one the app migrates.
    let todo = subject
        .package
        .migrations_for("")
        .into_iter()
        .flatten()
        .filter(|(name, state)| !state.waiting.is_empty() && in_app(name))
        .map(|(name, state)| Todo {
            module: name.clone(),
            waiting: state.waiting.len(),
        })
        .collect();

    // Problems first, then warnings; within each by module, the app's own last.
    list.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .then_with(|| a.module.is_none().cmp(&b.module.is_none()))
            .then_with(|| a.module.cmp(&b.module))
            .then_with(|| a.check.cmp(&b.check))
    });
    Findings { list, checks, todo }
}
