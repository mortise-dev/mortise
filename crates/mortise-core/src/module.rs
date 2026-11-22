//! The modules of an app: found on disk the way the package finds them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

use crate::layout::Layout;

/// Whether a module is turned on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleState {
    /// The statuses file has the JSON value `true` for the module.
    On,
    /// Anything else, including no entry: the package treats a missing entry as off.
    Off,
    /// The app does not use the file activator, so the state is not in a file.
    Unknown,
}

/// What reading a module's `module.json` gave.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Manifest {
    Valid,
    /// Not valid JSON. The message is the parser's, without the position.
    InvalidJson {
        message: String,
        line: usize,
    },
    /// Valid JSON without a `name` that is a string. The package needs one.
    NoName,
}

/// One module of an app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Module {
    /// The `name` of `module.json`. The folder name when the file gives none.
    pub name: String,
    /// The three letter code on the module's stamp.
    pub code: String,
    /// The module's folder. Absolute.
    pub dir: PathBuf,
    pub state: ModuleState,
    pub description: String,
    /// The load order the package uses, lowest first.
    pub priority: i64,
    pub manifest: Manifest,
    /// Files in the module's folder. Hidden files, `vendor`, `node_modules` and
    /// whatever `.gitignore` excludes are not counted.
    pub files: u64,
}

/// A module as read from disk, before it has a code.
pub(crate) struct FoundModule {
    pub(crate) name: String,
    pub(crate) dir: PathBuf,
    pub(crate) state: ModuleState,
    pub(crate) description: String,
    pub(crate) priority: i64,
    pub(crate) manifest: Manifest,
    pub(crate) files: u64,
}

/// Finds the modules of an app: every folder directly under the modules folder
/// that holds a `module.json`. Sorted by name.
pub(crate) fn find_modules(layout: &Layout) -> Vec<FoundModule> {
    let statuses = read_statuses(&layout.statuses_file);

    let Ok(entries) = std::fs::read_dir(&layout.modules_dir) else {
        return Vec::new();
    };
    let mut modules: Vec<FoundModule> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|dir| dir.join("module.json").is_file())
        .filter_map(|dir| {
            let folder = dir.file_name()?.to_str()?.to_owned();
            // The package's glob does not match folders that start with a dot.
            if folder.starts_with('.') {
                return None;
            }
            let (manifest, fields) = read_manifest(&dir.join("module.json"));
            let name = fields.name.unwrap_or(folder);
            let state = if !layout.uses_file_activator() {
                ModuleState::Unknown
            } else if statuses.get(&name) == Some(&serde_json::Value::Bool(true)) {
                ModuleState::On
            } else {
                ModuleState::Off
            };
            Some(FoundModule {
                files: count_files(&dir),
                name,
                dir,
                state,
                description: fields.description,
                priority: fields.priority,
                manifest,
            })
        })
        .collect();

    modules.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.dir.cmp(&b.dir))
    });
    modules
}

#[derive(Default)]
struct ManifestFields {
    name: Option<String>,
    description: String,
    priority: i64,
}

fn read_manifest(path: &Path) -> (Manifest, ManifestFields) {
    let json = std::fs::read(path).unwrap_or_default();
    let value: serde_json::Value = match serde_json::from_slice(&json) {
        Ok(value) => value,
        Err(error) => {
            // serde_json prints "<message> at line L column C". Keep the message.
            let text = error.to_string();
            let message = text.split(" at line ").next().unwrap_or(&text).to_owned();
            let invalid = Manifest::InvalidJson {
                message,
                line: error.line(),
            };
            return (invalid, ManifestFields::default());
        }
    };

    let fields = ManifestFields {
        name: value
            .get("name")
            .and_then(|name| name.as_str())
            .map(str::to_owned),
        description: value
            .get("description")
            .and_then(|description| description.as_str())
            .unwrap_or_default()
            .to_owned(),
        priority: value
            .get("priority")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0),
    };
    let manifest = if fields.name.is_some() {
        Manifest::Valid
    } else {
        Manifest::NoName
    };
    (manifest, fields)
}

/// The entries of the statuses file. Empty when the file is missing or is not a JSON object.
fn read_statuses(path: &Path) -> BTreeMap<String, serde_json::Value> {
    std::fs::read(path)
        .ok()
        .and_then(|json| serde_json::from_slice(&json).ok())
        .unwrap_or_default()
}

fn count_files(dir: &Path) -> u64 {
    WalkBuilder::new(dir)
        // Apply .gitignore files also when the app is not a git repository.
        .require_git(false)
        .filter_entry(|entry| {
            let is_dir = entry.file_type().is_some_and(|kind| kind.is_dir());
            !(is_dir && matches!(entry.file_name().to_str(), Some("vendor" | "node_modules")))
        })
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn counts_files_but_not_hidden_ignored_or_installed_ones() {
        let dir = tempfile::tempdir().unwrap();
        let module = dir.path();
        write(&module.join("module.json"), "{}");
        write(&module.join("app/Models/Post.php"), "<?php");
        write(&module.join("resources/views/index.blade.php"), "");
        write(&module.join("tests/Unit/.gitkeep"), "");
        write(&module.join("node_modules/left-pad/index.js"), "");
        write(&module.join("vendor/autoload.php"), "<?php");
        write(&module.join(".gitignore"), "/build\n");
        write(&module.join("build/app.js"), "");
        assert_eq!(count_files(module), 3);
    }

    #[test]
    fn reads_the_fields_the_package_reads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("module.json");

        write(
            &path,
            r#"{"name": "Blog", "description": "Posts", "priority": 5}"#,
        );
        let (manifest, fields) = read_manifest(&path);
        assert_eq!(manifest, Manifest::Valid);
        assert_eq!(fields.name.as_deref(), Some("Blog"));
        assert_eq!((fields.description.as_str(), fields.priority), ("Posts", 5));

        write(&path, r#"{"alias": "blog"}"#);
        assert_eq!(read_manifest(&path).0, Manifest::NoName);

        write(&path, "{\n  \"name\": \"Blog\",\n");
        let (manifest, fields) = read_manifest(&path);
        assert_eq!(
            manifest,
            Manifest::InvalidJson {
                message: "EOF while parsing a value".into(),
                line: 3
            }
        );
        assert_eq!(fields.name, None);
    }

    #[test]
    fn a_statuses_file_that_is_missing_or_not_an_object_has_no_entries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("modules_statuses.json");
        assert!(read_statuses(&path).is_empty());
        for content in ["", "[]", "[true]", "not json", "null"] {
            write(&path, content);
            assert!(read_statuses(&path).is_empty(), "{content:?}");
        }
        write(&path, r#"{"Blog": true}"#);
        assert_eq!(read_statuses(&path).len(), 1);
    }
}
