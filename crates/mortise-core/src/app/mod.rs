//! Reading one app folder into a snapshot.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::boundary::links::{self, Cycle, Link, ModuleSource};
use crate::boundary::rules::{self, CoverageSettings, Rules, RulesSource};
use crate::findings::{self, DEFAULT_COVERAGE_LIMIT, Findings, Subject};
use crate::modules::clover::{self, Coverage};
use crate::modules::code::propose_codes;
use crate::modules::composer::{Lock, Version};
use crate::modules::git::{self, Git};
use crate::modules::kind::Kinds;
use crate::modules::layout::Layout;
use crate::modules::module::{Module, ModuleState, ReadCache, find_modules};
use crate::package::PackageData;

#[cfg(feature = "watch")]
pub(crate) mod live;

const LARAVEL: &str = "laravel/framework";
const PACKAGE: &str = "nwidart/laravel-modules";
const ACTUATOR: &str = "nwidart/laravel-actuator";

/// The coverage report an app's coverage figures come from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageReport {
    /// The clover file. Absolute.
    pub path: PathBuf,
    /// When the file was written, in seconds since 1970.
    pub written: Option<i64>,
}

/// Everything Mortise knows about an app from one read of its folder.
///
/// A snapshot is never changed. A later read gives a new one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSnapshot {
    /// The app folder: the one that holds `artisan`. Absolute.
    pub root: PathBuf,
    /// The name of the app folder.
    pub name: String,
    /// The version of `laravel/framework`. Absent when the lock file does not list it.
    pub laravel: Option<Version>,
    /// The version of `nwidart/laravel-modules`.
    pub package: Version,
    /// The version of `nwidart/laravel-actuator`, when the app has it.
    pub actuator: Option<Version>,
    /// `APP_URL` from `.env`, to propose the base URL of an environment.
    pub app_url: Option<String>,
    /// `APP_ENV` from `.env`. It tells Mortise to ask before a command in production.
    /// `APP_URL` and `APP_ENV` are the only lines Mortise reads from that file.
    pub app_env: Option<String>,
    pub layout: Layout,
    /// The modules, sorted by name.
    pub modules: Vec<Module>,
    /// The coverage report, when one was found.
    pub coverage_report: Option<CoverageReport>,
    pub git: Git,
    /// Which module uses which, by the name of the module that uses and then of the
    /// one that is used.
    pub links: Vec<Link>,
    /// The groups of modules that are on and use each other.
    pub cycles: Vec<Cycle>,
    /// The boundary rules the links were checked against.
    pub rules: Rules,
    pub rules_source: RulesSource,
    /// Why the `coverage` object of `mortise.json` is ignored, when it cannot be read.
    /// The parser's message. Coverage then comes from the settings.
    pub coverage_ignored: Option<String>,
    /// What the rules and the checks on files found.
    pub findings: Findings,
    /// A module covered less than this many per cent is a warning.
    pub coverage_limit: u8,
    /// What the package said in the reads the user started, from [`ReadOptions::package`].
    /// The read fills in the tables of each module's ran migrations.
    pub package_data: PackageData,
}

impl AppSnapshot {
    /// How many modules are turned on.
    pub fn modules_on(&self) -> usize {
        self.modules
            .iter()
            .filter(|module| module.state == ModuleState::On)
            .count()
    }

    /// Files in all modules together.
    pub fn files(&self) -> u64 {
        self.modules.iter().map(|module| module.files).sum()
    }

    /// Lines of PHP in all modules together.
    pub fn lines(&self) -> u64 {
        self.modules.iter().map(|module| module.lines).sum()
    }

    /// Tests in all modules together.
    pub fn tests(&self) -> u32 {
        self.modules.iter().map(|module| module.tests).sum()
    }

    /// The coverage of all modules together. `None` when no module has a figure.
    pub fn coverage(&self) -> Option<Coverage> {
        let mut total: Option<Coverage> = None;
        for coverage in self.modules.iter().filter_map(|module| module.coverage) {
            total.get_or_insert_default().add(coverage);
        }
        total
    }

    /// The link from one module to another, if the first uses the second.
    pub fn link(&self, from: &str, to: &str) -> Option<&Link> {
        self.links
            .iter()
            .find(|link| link.from == from && link.to == to)
    }

    /// The module of the circle of `module` that `module` uses directly: the link a
    /// circle finding opens on. `None` when `module` is in no circle.
    pub fn circle_neighbour(&self, module: &str) -> Option<&str> {
        crate::boundary::links::circle_neighbour(&self.links, module)
    }

    /// The links of modules that `name` uses.
    pub fn uses<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Link> {
        self.links.iter().filter(move |link| link.from == name)
    }

    /// The links of modules that use `name`.
    pub fn used_by<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Link> {
        self.links.iter().filter(move |link| link.to == name)
    }

    /// The links that break a rule.
    pub fn problem_links(&self) -> impl Iterator<Item = &Link> {
        self.links
            .iter()
            .filter(|link| link.is_problem(&self.rules))
    }

    /// The modules that carry a circle `name` is in without carrying it, by name. Empty
    /// while "No circular use" is off, because then no circle is a finding.
    pub fn circles_counted_elsewhere<'a>(&'a self, name: &str) -> Vec<&'a str> {
        let mut carriers: Vec<&str> = Vec::new();
        if self.rules.no_circular_use {
            for cycle in &self.cycles {
                let member = cycle.modules.iter().any(|module| module == name);
                if member && cycle.carrier != name && !carriers.contains(&cycle.carrier.as_str()) {
                    carriers.push(&cycle.carrier);
                }
            }
        }
        carriers.sort_unstable();
        carriers
    }

    /// The module with this name.
    pub fn module(&self, name: &str) -> Option<&Module> {
        self.modules.iter().find(|module| module.name == name)
    }

    /// The code of every module, by module name. Store it, and pass it to the next read.
    pub fn codes(&self) -> BTreeMap<String, String> {
        self.modules
            .iter()
            .map(|module| (module.name.clone(), module.code.clone()))
            .collect()
    }

    /// The modules folder as the user writes it: relative to the app folder when it
    /// is inside it, with `/` between the parts.
    pub fn modules_dir_shown(&self) -> String {
        self.path_shown(&self.layout.modules_dir)
    }

    /// A path as the user writes it: relative to the app folder when it is inside it.
    pub fn path_shown(&self, path: &Path) -> String {
        shown(&self.root, path)
    }
}

fn shown(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Why a folder could not be read as an app.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    /// The folder has no `artisan` file and no `composer.json` that requires `laravel/framework`.
    #[error("{} is not a Laravel app", root.display())]
    NotLaravel { root: PathBuf },
    /// The folder has no `composer.lock`, so no version can be read.
    #[error("{} has no composer.lock", root.display())]
    NoLock { root: PathBuf },
    /// `composer.lock` has no entry for `nwidart/laravel-modules`.
    #[error("{} does not use nwidart/laravel-modules", root.display())]
    NoPackage { root: PathBuf },
    /// `composer.lock` is not valid JSON.
    #[error("{} is not valid JSON: {message}", path.display())]
    InvalidLock { path: PathBuf, message: String },
    /// The coverage report given for the read, such as with `mortise check --coverage`,
    /// cannot be read or is no clover report.
    #[error("cannot read the coverage report {}: {message}", path.display())]
    CoverageReport { path: PathBuf, message: String },
    #[error("cannot read {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// `APP_ENV` from the `.env` file of the app in `root`, read now. A confirmation
/// checks it again just before a destructive command starts.
pub fn read_app_env(root: &Path) -> Option<String> {
    read_env(root).1
}

/// `APP_URL` and `APP_ENV` from the `.env` file of the app, in one read.
/// No other line is looked at.
fn read_env(root: &Path) -> (Option<String>, Option<String>) {
    match std::fs::read_to_string(root.join(".env")) {
        Ok(env) => (env_value(&env, "APP_URL"), env_value(&env, "APP_ENV")),
        Err(_) => (None, None),
    }
}

/// The value of one name in the text of a `.env` file. As in dotenv, a later line
/// of the same name replaces an earlier one.
fn env_value(env: &str, name: &str) -> Option<String> {
    let value = env.lines().rev().find_map(|line| {
        let line = line.trim_start();
        let line = line.strip_prefix("export ").unwrap_or(line);
        line.strip_prefix(name)?.trim_start().strip_prefix('=')
    })?;
    let value = value.trim();
    let value = match value.chars().next() {
        // A quoted value ends at the closing quote.
        Some(quote @ ('"' | '\'')) => value[1..].split(quote).next().unwrap_or_default(),
        // A plain value ends where a comment starts.
        _ => value.split(" #").next().unwrap_or_default().trim_end(),
    };
    (!value.is_empty()).then(|| value.to_owned())
}

/// Reads the app in `root`. Reads only: nothing in the folder is changed and nothing is run.
pub fn read_app(root: &Path) -> Result<AppSnapshot, ReadError> {
    read_app_with(root, &BTreeMap::new())
}

/// Reads the app in `root`, keeping the codes its modules already have.
///
/// `codes` maps a module name to its code. A module that is not in it gets a new
/// code that differs from all of them.
pub fn read_app_with(
    root: &Path,
    codes: &BTreeMap<String, String>,
) -> Result<AppSnapshot, ReadError> {
    read_app_cached(
        root,
        codes,
        &ReadOptions::default(),
        &mut ReadCache::default(),
    )
}

/// What the user set for an app in Mortise, as far as reading it is concerned.
///
/// The coverage limit and report come, first to last, from `coverage_given`, from the
/// `coverage` object of `mortise.json`, and from `coverage_limit` and `coverage_report`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOptions {
    /// A module covered less than this many per cent is a warning.
    pub coverage_limit: u8,
    /// The clover report, relative to the app folder or absolute. `None` looks in
    /// `phpunit.xml` and the usual places.
    pub coverage_report: Option<PathBuf>,
    /// Coverage given for this read only, such as the flags of `mortise check`. A value
    /// that is set wins over `mortise.json`.
    pub coverage_given: CoverageSettings,
    /// What the package said in the reads the user started. Kept in memory only.
    pub package: PackageData,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            coverage_limit: DEFAULT_COVERAGE_LIMIT,
            coverage_report: None,
            coverage_given: CoverageSettings::default(),
            package: PackageData::default(),
        }
    }
}

/// Reads the app in `root`. Files that did not change since they were put in
/// `cache` are not read again.
pub fn read_app_cached(
    root: &Path,
    codes: &BTreeMap<String, String>,
    options: &ReadOptions,
    cache: &mut ReadCache,
) -> Result<AppSnapshot, ReadError> {
    let root = std::path::absolute(root).map_err(|source| ReadError::Io {
        path: root.to_owned(),
        source,
    })?;

    if !root.join("artisan").is_file() && !requires_laravel(&root) {
        return Err(ReadError::NotLaravel { root });
    }

    let lock_path = root.join("composer.lock");
    let lock = match std::fs::read(&lock_path) {
        Ok(json) => Lock::parse(&json).map_err(|error| ReadError::InvalidLock {
            path: lock_path,
            message: error.to_string(),
        })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(ReadError::NoLock { root });
        }
        Err(source) => {
            return Err(ReadError::Io {
                path: lock_path,
                source,
            });
        }
    };
    let Some(package) = lock.version(PACKAGE) else {
        return Err(ReadError::NoPackage { root });
    };

    let (layout, config) = Layout::read(&root);
    let kinds = Kinds::new(config.as_ref(), &layout.app_folder);
    let (found, status_entries) = find_modules(&layout, &kinds, cache);

    // A code is kept once given. Only modules without one get a proposal.
    let taken: BTreeSet<String> = codes.values().cloned().collect();
    let new_names = found
        .iter()
        .map(|module| module.name.as_str())
        .filter(|name| !codes.contains_key(*name));
    let mut proposed = propose_codes(new_names, &taken).into_iter();
    let codes: Vec<String> = found
        .iter()
        .map(|module| match codes.get(&module.name) {
            Some(code) => code.clone(),
            None => proposed.next().unwrap_or_default(),
        })
        .collect();

    // A module's folder as git and the coverage report name it.
    let folders: Vec<String> = found
        .iter()
        .map(|module| shown(&root, &module.dir))
        .collect();

    // The rules, and the coverage settings that the app keeps beside them.
    let rules::RulesFile {
        rules,
        source: rules_source,
        coverage: in_file,
        coverage_ignored,
    } = rules::load_file(&root);
    let coverage_limit = options
        .coverage_given
        .limit
        .or(in_file.limit)
        .unwrap_or(options.coverage_limit);

    // A report given for this read must be read; a fault is an error, not a fallback.
    let report = match &options.coverage_given.report {
        Some(given) => {
            let path = root.join(given);
            let fault = |message: String| ReadError::CoverageReport {
                path: path.clone(),
                message,
            };
            let xml = std::fs::read(&path).map_err(|error| fault(error.to_string()))?;
            match clover::read_report(&xml) {
                (files, None) => Some((path, files)),
                (_, Some(message)) => return Err(fault(message)),
            }
        }
        // The report the user named, if it is there; else the one Mortise finds.
        None => in_file
            .report
            .as_ref()
            .or(options.coverage_report.as_ref())
            .map(|path| root.join(path))
            .filter(|path| path.is_file())
            .or_else(|| clover::find_report(&root))
            .and_then(|path| {
                let xml = std::fs::read(&path).ok()?;
                Some((path, clover::read_report(&xml).0))
            }),
    };
    let (report_path, report) = report.unzip();
    let coverage_of = |folder: &str| -> Option<Coverage> {
        // The report has the paths of the machine that ran the tests, so only the
        // part from the modules folder on can be compared.
        let inside = format!("/{folder}/");
        let at_start = format!("{folder}/");
        let mut total: Option<Coverage> = None;
        for file in report.as_ref()? {
            if file.path.contains(&inside) || file.path.starts_with(&at_start) {
                total.get_or_insert_default().add(file.coverage);
            }
        }
        total
    };

    // Which module uses which, checked against the rules of the app.
    let sources: Vec<ModuleSource<'_>> = found
        .iter()
        .zip(&folders)
        .map(|(module, folder)| ModuleSource {
            name: &module.name,
            on: module.state != ModuleState::Off,
            namespace: &module.namespace,
            surface: module.surface.as_ref(),
            files: module
                .scan
                .sources
                .iter()
                // A test is not a boundary of the running app.
                .filter(|source| !source.is_test)
                .map(|source| {
                    (
                        format!("{folder}/{}", source.relative),
                        source.analysis.refs.as_slice(),
                    )
                })
                .collect(),
            declared: module
                .scan
                .sources
                .iter()
                .flat_map(|source| {
                    source
                        .analysis
                        .declares
                        .iter()
                        .map(|declared| declared.class.as_str())
                })
                .collect(),
        })
        .collect();
    let (links, cycles) = links::build(&sources, &rules);
    drop(sources);

    let git = git::read(&root);
    let (churn, uncommitted) = match git {
        Git::Repository { .. } => {
            let dirs: Vec<PathBuf> = found.iter().map(|module| module.dir.clone()).collect();
            (git::churn(&root, &folders), git::uncommitted(&root, &dirs))
        }
        _ => (BTreeMap::new(), vec![None; found.len()]),
    };

    let modules: Vec<Module> = found
        .into_iter()
        .zip(codes)
        .zip(&folders)
        .zip(uncommitted)
        .map(|(((module, code), folder), uncommitted)| Module {
            name: module.name,
            code,
            dir: module.dir,
            state: module.state,
            description: module.description,
            priority: module.priority,
            manifest: module.manifest,
            surface: module.surface,
            namespace: module.namespace,
            files: module.scan.files,
            lines: module.scan.lines,
            kinds: module.scan.kinds,
            commands: module.scan.commands.into_iter().collect(),
            tests: module.scan.tests,
            coverage: coverage_of(folder),
            churn: churn.get(folder).copied(),
            uncommitted,
        })
        .collect();

    let findings = findings::find(&Subject {
        root: &root,
        modules: &modules,
        links: &links,
        cycles: &cycles,
        rules: &rules,
        status_entries: &status_entries,
        statuses_file: &shown(&root, &layout.statuses_file),
        has_coverage_report: report_path.is_some(),
        coverage_limit,
        package: &options.package,
    });

    let package_data = options.package.with_tables(&modules);
    let (app_url, app_env) = read_env(&root);
    Ok(AppSnapshot {
        name: root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.display().to_string()),
        laravel: lock.version(LARAVEL),
        package,
        actuator: lock.version(ACTUATOR),
        app_url,
        app_env,
        layout,
        modules,
        coverage_report: report_path.map(|path| CoverageReport {
            written: std::fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|since| since.as_secs() as i64),
            path,
        }),
        git,
        links,
        cycles,
        rules,
        rules_source,
        coverage_ignored,
        findings,
        coverage_limit,
        package_data,
        root,
    })
}

fn requires_laravel(root: &Path) -> bool {
    std::fs::read(root.join("composer.json"))
        .ok()
        .and_then(|json| serde_json::from_slice::<serde_json::Value>(&json).ok())
        .is_some_and(|composer| {
            ["require", "require-dev"]
                .iter()
                .any(|section| composer.get(section).and_then(|s| s.get(LARAVEL)).is_some())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_url_and_the_environment_of_the_app_are_read_from_their_lines() {
        let env = "APP_NAME=Shop\nAPP_ENV=production\nAPP_URLS=no\nAPP_URL=https://shop-api.test\nDB_PASSWORD=secret\n";
        assert_eq!(
            env_value(env, "APP_URL").as_deref(),
            Some("https://shop-api.test")
        );
        assert_eq!(
            env_value("APP_URL=\"http://localhost:8000\" # dev", "APP_URL").as_deref(),
            Some("http://localhost:8000")
        );
        assert_eq!(
            env_value("APP_URL = http://localhost # dev", "APP_URL").as_deref(),
            Some("http://localhost")
        );
        assert_eq!(
            env_value("export APP_URL='http://a.test'", "APP_URL").as_deref(),
            Some("http://a.test")
        );
        assert_eq!(env_value("APP_URL=", "APP_URL"), None);
        assert_eq!(env_value("# APP_URL=http://old.test", "APP_URL"), None);
        assert_eq!(env_value("APP_ENV=local", "APP_URL"), None);
        assert_eq!(env_value(env, "APP_ENV").as_deref(), Some("production"));
        assert_eq!(
            env_value("APP_URL=http://a.test\nAPP_URL=http://b.test", "APP_URL").as_deref(),
            Some("http://b.test")
        );
    }
}
