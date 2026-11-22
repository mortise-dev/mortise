//! Reading one app folder into a snapshot.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::code::propose_codes;
use crate::composer::{Lock, Version};
use crate::layout::Layout;
use crate::module::{Module, ModuleState, find_modules};

const LARAVEL: &str = "laravel/framework";
const PACKAGE: &str = "nwidart/laravel-modules";
const ACTUATOR: &str = "nwidart/laravel-actuator";

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
    pub layout: Layout,
    /// The modules, sorted by name.
    pub modules: Vec<Module>,
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

    /// The modules folder as the user writes it: relative to the app folder when it
    /// is inside it, with `/` between the parts.
    pub fn modules_dir_shown(&self) -> String {
        self.path_shown(&self.layout.modules_dir)
    }

    /// A path as the user writes it: relative to the app folder when it is inside it.
    pub fn path_shown(&self, path: &Path) -> String {
        let shown = path.strip_prefix(&self.root).unwrap_or(path);
        shown
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
    }
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

/// Reads the app in `root`. Reads only: nothing in the folder is changed and nothing is run.
pub fn read_app(root: &Path) -> Result<AppSnapshot, ReadError> {
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

    let layout = Layout::read(&root);
    let found = find_modules(&layout);
    let codes = propose_codes(
        found.iter().map(|module| module.name.as_str()),
        &BTreeSet::new(),
    );
    let modules = found
        .into_iter()
        .zip(codes)
        .map(|(module, code)| Module {
            name: module.name,
            code,
            dir: module.dir,
            state: module.state,
            description: module.description,
            priority: module.priority,
            manifest: module.manifest,
            files: module.files,
        })
        .collect();

    Ok(AppSnapshot {
        name: root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.display().to_string()),
        laravel: lock.version(LARAVEL),
        package,
        actuator: lock.version(ACTUATOR),
        layout,
        modules,
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
