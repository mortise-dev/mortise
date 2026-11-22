//! Reading one app folder into a snapshot.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::clover::{self, Coverage};
use crate::code::propose_codes;
use crate::composer::{Lock, Version};
use crate::findings::{self, DEFAULT_COVERAGE_LIMIT, Findings, Subject};
use crate::git::{self, Git};
use crate::kind::Kinds;
use crate::layout::Layout;
use crate::links::{self, Cycle, Link, ModuleSource};
use crate::module::{Module, ModuleState, ReadCache, find_modules};
use crate::rules::{Rules, RulesSource};

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
    /// `APP_URL` from `.env`, to propose the base URL of an environment. It is the
    /// only line Mortise reads from that file.
    pub app_url: Option<String>,
    pub layout: Layout,
    /// The modules, sorted by name.
    pub modules: Vec<Module>,
    /// The coverage report, when one was found.
    pub coverage_report: Option<CoverageReport>,
    pub git: Git,
    /// Which module uses which, by the name of the module that uses and then of the
    /// one that is used.
    pub links: Vec<Link>,
    /// The groups of modules that use each other.
    pub cycles: Vec<Cycle>,
    /// The boundary rules the links were checked against.
    pub rules: Rules,
    pub rules_source: RulesSource,
    /// What the rules and the checks on files found.
    pub findings: Findings,
    /// A module covered less than this many per cent is a warning.
    pub coverage_limit: u8,
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
        crate::links::circle_neighbour(&self.links, module)
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
    #[error("cannot read {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// `APP_URL` from the `.env` file of the app. No other line is looked at.
fn app_url(root: &Path) -> Option<String> {
    let env = std::fs::read_to_string(root.join(".env")).ok()?;
    env_value(&env, "APP_URL")
}

/// The value of one name in the text of a `.env` file.
fn env_value(env: &str, name: &str) -> Option<String> {
    let value = env.lines().find_map(|line| {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOptions {
    /// A module covered less than this many per cent is a warning.
    pub coverage_limit: u8,
    /// The clover report, relative to the app folder or absolute. `None` looks in
    /// `phpunit.xml` and the usual places.
    pub coverage_report: Option<PathBuf>,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            coverage_limit: DEFAULT_COVERAGE_LIMIT,
            coverage_report: None,
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

    // The report the user named, if it is there; else the one Mortise finds.
    let report_path = options
        .coverage_report
        .as_ref()
        .map(|path| root.join(path))
        .filter(|path| path.is_file())
        .or_else(|| clover::find_report(&root));
    let report = report_path
        .as_ref()
        .and_then(|path| std::fs::read(path).ok())
        .map(|xml| clover::read_report(&xml));
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
    let (rules, rules_source) = Rules::load(&root);
    let sources: Vec<ModuleSource<'_>> = found
        .iter()
        .zip(&folders)
        .map(|(module, folder)| ModuleSource {
            name: &module.name,
            namespace: &module.namespace,
            files: module
                .scan
                .sources
                .iter()
                .map(|(relative, analysis)| {
                    (format!("{folder}/{relative}"), analysis.refs.as_slice())
                })
                .collect(),
            declared: module
                .scan
                .sources
                .iter()
                .flat_map(|(_, analysis)| {
                    analysis
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
    let churn = match git {
        Git::Repository { .. } => git::churn(&root, &folders),
        _ => BTreeMap::new(),
    };

    let modules: Vec<Module> = found
        .into_iter()
        .zip(codes)
        .zip(&folders)
        .map(|((module, code), folder)| Module {
            name: module.name,
            code,
            dir: module.dir,
            state: module.state,
            description: module.description,
            priority: module.priority,
            manifest: module.manifest,
            namespace: module.namespace,
            files: module.scan.files,
            lines: module.scan.lines,
            kinds: module.scan.kinds,
            tests: module.scan.tests,
            coverage: coverage_of(folder),
            churn: churn.get(folder).copied(),
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
        coverage_limit: options.coverage_limit,
    });

    Ok(AppSnapshot {
        name: root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.display().to_string()),
        laravel: lock.version(LARAVEL),
        package,
        actuator: lock.version(ACTUATOR),
        app_url: app_url(&root),
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
        findings,
        coverage_limit: options.coverage_limit,
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
    fn the_url_of_the_app_is_read_from_its_line() {
        let env = "APP_NAME=Shop\nAPP_URLS=no\nAPP_URL=https://shop-api.test\nDB_PASSWORD=secret\n";
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
    }
}
