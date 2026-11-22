//! Mortise's own settings: the apps it knows, kept in one JSON file on this computer.
//!
//! No secret is ever written here, and nothing here is written into an app folder.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::app::AppSnapshot;
use crate::code::propose_monogram;

/// An app Mortise knows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEntry {
    /// The app folder. Absolute.
    pub root: PathBuf,
    /// The name shown for the app. The name of its folder until the user changes it.
    pub name: String,
    /// The two letters on the app's tile.
    pub monogram: String,
    /// The code of each module, by module name. A code is kept once given.
    #[serde(default)]
    pub codes: BTreeMap<String, String>,
}

/// Everything Mortise keeps between runs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// The apps, in the order of the rail.
    #[serde(default)]
    pub apps: Vec<AppEntry>,
    /// The app that was open last, as an index into `apps`.
    #[serde(default)]
    pub current: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("cannot read or write {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{} is not valid: {message}", path.display())]
    Invalid { path: PathBuf, message: String },
}

impl Settings {
    /// Where the settings file is: `settings.json` in the config folder of the
    /// platform, or in the folder named by `MORTISE_CONFIG_DIR`.
    pub fn default_path() -> Option<PathBuf> {
        let dir = match std::env::var_os("MORTISE_CONFIG_DIR") {
            Some(dir) if !dir.is_empty() => PathBuf::from(dir),
            _ => directories::ProjectDirs::from("", "", "Mortise")?
                .config_dir()
                .to_owned(),
        };
        Some(dir.join("settings.json"))
    }

    /// Loads the settings. A file that does not exist yet gives empty settings.
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        let json = match std::fs::read(path) {
            Ok(json) => json,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(source) => {
                return Err(SettingsError::Io {
                    path: path.to_owned(),
                    source,
                });
            }
        };
        let mut settings: Self =
            serde_json::from_slice(&json).map_err(|error| SettingsError::Invalid {
                path: path.to_owned(),
                message: error.to_string(),
            })?;
        if settings
            .current
            .is_some_and(|current| current >= settings.apps.len())
        {
            settings.current = None;
        }
        Ok(settings)
    }

    /// Saves the settings. The file is written beside its place and then moved over
    /// it, so a crash cannot leave half a file.
    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        let io = |source| SettingsError::Io {
            path: path.to_owned(),
            source,
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        let json = serde_json::to_vec_pretty(self).expect("settings are plain data");
        let partial = path.with_extension("json.partial");
        std::fs::write(&partial, json).map_err(io)?;
        std::fs::rename(&partial, path).map_err(io)
    }

    /// The place of the app in `root` among the apps, if Mortise knows it.
    pub fn position(&self, root: &Path) -> Option<usize> {
        let wanted = same_folder(root);
        self.apps
            .iter()
            .position(|app| same_folder(&app.root) == wanted)
    }

    /// Adds an app after the others and returns its place. The monogram differs
    /// from those of the other apps.
    pub fn add(&mut self, app: &AppSnapshot) -> usize {
        let taken: BTreeSet<String> = self.apps.iter().map(|app| app.monogram.clone()).collect();
        self.apps.push(AppEntry {
            root: app.root.clone(),
            name: app.name.clone(),
            monogram: propose_monogram(&app.name, &taken),
            codes: app.codes(),
        });
        self.apps.len() - 1
    }

    /// Forgets an app. Its folder and files stay as they are.
    pub fn remove(&mut self, index: usize) -> AppEntry {
        let entry = self.apps.remove(index);
        self.current = match self.current {
            Some(current) if current == index => None,
            Some(current) if current > index => Some(current - 1),
            other => other,
        };
        entry
    }
}

/// A folder in a form that is equal for two ways of writing the same folder.
fn same_folder(path: &Path) -> PathBuf {
    std::fs::canonicalize(path)
        .or_else(|_| std::path::absolute(path))
        .unwrap_or_else(|_| path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, monogram: &str) -> AppEntry {
        AppEntry {
            root: PathBuf::from("/code").join(name),
            name: name.into(),
            monogram: monogram.into(),
            codes: BTreeMap::from([("Checkout".to_owned(), "CHK".to_owned())]),
        }
    }

    #[test]
    fn a_missing_file_is_empty_settings() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            Settings::load(&dir.path().join("settings.json")).unwrap(),
            Settings::default()
        );
    }

    #[test]
    fn what_is_saved_is_loaded_again() {
        let dir = tempfile::tempdir().unwrap();
        // The folder does not exist yet.
        let path = dir.path().join("mortise").join("settings.json");
        let settings = Settings {
            apps: vec![entry("shop-api", "SA"), entry("crm", "CR")],
            current: Some(1),
        };
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap(), settings);
        // Nothing is left beside the file.
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
    }

    #[test]
    fn a_broken_file_is_an_error_and_is_not_touched() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert!(matches!(
            Settings::load(&path),
            Err(SettingsError::Invalid { .. })
        ));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not json");
    }

    #[test]
    fn a_current_app_that_is_gone_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, r#"{"apps": [], "current": 3}"#).unwrap();
        assert_eq!(Settings::load(&path).unwrap().current, None);
    }

    #[test]
    fn removing_an_app_keeps_the_current_one() {
        let mut settings = Settings {
            apps: vec![entry("a", "AA"), entry("b", "BB"), entry("c", "CC")],
            current: Some(2),
        };
        assert_eq!(settings.remove(0).name, "a");
        assert_eq!(settings.current, Some(1));
        assert_eq!(settings.remove(1).name, "c");
        assert_eq!(settings.current, None);
    }

    #[test]
    fn the_same_folder_written_two_ways_is_one_app() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("shop-api");
        std::fs::create_dir(&root).unwrap();
        let settings = Settings {
            apps: vec![AppEntry {
                root: root.clone(),
                ..entry("shop-api", "SA")
            }],
            current: None,
        };
        assert_eq!(settings.position(&root), Some(0));
        assert_eq!(
            settings.position(&root.join("..").join("shop-api")),
            Some(0)
        );
        assert_eq!(settings.position(&dir.path().join("other")), None);
    }
}
