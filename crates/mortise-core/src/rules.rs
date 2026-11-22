//! The boundary rules of an app, kept in `mortise.json` in the app folder so that a
//! team shares them.
//!
//! It is the only file Mortise writes into an app without a command, and only when
//! the interface asks after the user confirmed. No secret is ever written to it.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The name of the rules file in the app folder.
pub const RULES_FILE: &str = "mortise.json";

/// The two rules of this version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    /// No circular use: two modules must not use each other, directly or through others.
    pub no_circular_use: bool,
    /// Other modules use only a module's public surface.
    pub public_surface_only: bool,
    /// For a module without an accepted public surface, the folders that count as
    /// public: a class in one of them may be used by other modules.
    pub public_folders: Vec<String>,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            no_circular_use: true,
            public_surface_only: true,
            public_folders: vec!["Contracts".to_owned(), "Events".to_owned()],
        }
    }
}

/// Where the rules of a snapshot came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulesSource {
    /// The app has no `mortise.json`: the defaults apply.
    Defaults,
    /// Read from `mortise.json`.
    File,
    /// `mortise.json` could not be read as rules: the defaults apply. The message is the parser's.
    Invalid(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RulesError {
    #[error("cannot write {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The file holds something that is not a JSON object, which Mortise will not replace.
    #[error("{} is not a JSON object: {message}", path.display())]
    Invalid { path: PathBuf, message: String },
}

/// The shape of the file:
///
/// ```json
/// {
///     "rules": {
///         "no-circular-use": true,
///         "public-surface-only": { "enabled": true, "folders": ["Contracts", "Events"] }
///     }
/// }
/// ```
#[derive(Serialize, Deserialize)]
struct RulesInFile {
    #[serde(rename = "no-circular-use", default = "yes")]
    no_circular_use: bool,
    #[serde(rename = "public-surface-only", default)]
    public_surface_only: PublicSurfaceInFile,
}

#[derive(Serialize, Deserialize)]
struct PublicSurfaceInFile {
    #[serde(default = "yes")]
    enabled: bool,
    #[serde(default = "default_folders")]
    folders: Vec<String>,
}

impl Default for PublicSurfaceInFile {
    fn default() -> Self {
        Self {
            enabled: true,
            folders: default_folders(),
        }
    }
}

fn yes() -> bool {
    true
}

fn default_folders() -> Vec<String> {
    Rules::default().public_folders
}

impl Rules {
    /// Reads the rules of the app in `root`.
    pub fn load(root: &Path) -> (Self, RulesSource) {
        let Ok(json) = std::fs::read(root.join(RULES_FILE)) else {
            return (Self::default(), RulesSource::Defaults);
        };
        type Object = serde_json::Map<String, serde_json::Value>;
        let rules =
            serde_json::from_slice::<Object>(&json).and_then(|file| match file.get("rules") {
                Some(rules) => serde_json::from_value::<RulesInFile>(rules.clone()),
                // A file without rules is a file whose rules are the defaults.
                None => serde_json::from_str("{}"),
            });
        match rules {
            Ok(rules) => (
                Self {
                    no_circular_use: rules.no_circular_use,
                    public_surface_only: rules.public_surface_only.enabled,
                    public_folders: rules.public_surface_only.folders,
                },
                RulesSource::File,
            ),
            Err(error) => (Self::default(), RulesSource::Invalid(error.to_string())),
        }
    }

    /// Writes the rules to `mortise.json` in `root`. Other keys of the file are kept.
    pub fn save(&self, root: &Path) -> Result<(), RulesError> {
        let path = root.join(RULES_FILE);
        let mut file = match std::fs::read(&path) {
            Ok(json) => serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&json)
                .map_err(|error| RulesError::Invalid {
                    path: path.clone(),
                    message: error.to_string(),
                })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => serde_json::Map::new(),
            Err(source) => return Err(RulesError::Io { path, source }),
        };
        let rules = RulesInFile {
            no_circular_use: self.no_circular_use,
            public_surface_only: PublicSurfaceInFile {
                enabled: self.public_surface_only,
                folders: self.public_folders.clone(),
            },
        };
        file.insert(
            "rules".to_owned(),
            serde_json::to_value(rules).expect("rules are plain data"),
        );

        // Four spaces and a line end, like the other JSON files of a Laravel app.
        let mut json = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut serializer = serde_json::Serializer::with_formatter(&mut json, formatter);
        file.serialize(&mut serializer)
            .expect("a map of JSON values can be written");
        json.push(b'\n');

        let io = |source| RulesError::Io {
            path: path.clone(),
            source,
        };
        let partial = path.with_extension("json.partial");
        std::fs::write(&partial, json).map_err(io)?;
        std::fs::rename(&partial, &path).map_err(io)
    }

    /// True when a class is part of the public surface of its module by the folder
    /// list. `inside` is the class name after the module's namespace:
    /// `Contracts\PaymentGateway`.
    pub fn is_public(&self, inside: &str) -> bool {
        let Some((folder, _)) = inside.split_once('\\') else {
            // A class directly in the module's namespace is in no folder.
            return false;
        };
        self.public_folders
            .iter()
            .any(|public| public.eq_ignore_ascii_case(folder))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_file_means_the_defaults() {
        let app = tempfile::tempdir().unwrap();
        let (rules, source) = Rules::load(app.path());
        assert_eq!(source, RulesSource::Defaults);
        assert!(rules.no_circular_use && rules.public_surface_only);
        assert_eq!(rules.public_folders, ["Contracts", "Events"]);
    }

    #[test]
    fn what_is_saved_is_loaded_again_and_other_keys_are_kept() {
        let app = tempfile::tempdir().unwrap();
        let path = app.path().join(RULES_FILE);
        std::fs::write(
            &path,
            r#"{"team": "shop", "rules": {"no-circular-use": true}}"#,
        )
        .unwrap();

        let rules = Rules {
            no_circular_use: false,
            public_surface_only: true,
            public_folders: vec!["Contracts".into(), "Api".into()],
        };
        rules.save(app.path()).unwrap();
        assert_eq!(Rules::load(app.path()), (rules, RulesSource::File));

        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            text,
            r#"{
    "team": "shop",
    "rules": {
        "no-circular-use": false,
        "public-surface-only": {
            "enabled": true,
            "folders": [
                "Contracts",
                "Api"
            ]
        }
    }
}
"#
        );
        assert_eq!(
            std::fs::read_dir(app.path()).unwrap().count(),
            1,
            "nothing is left beside it"
        );
    }

    #[test]
    fn missing_keys_take_their_default() {
        let app = tempfile::tempdir().unwrap();
        let path = app.path().join(RULES_FILE);
        std::fs::write(
            &path,
            r#"{"rules": {"public-surface-only": {"folders": ["Api"]}}}"#,
        )
        .unwrap();
        let (rules, source) = Rules::load(app.path());
        assert_eq!(source, RulesSource::File);
        assert!(rules.no_circular_use && rules.public_surface_only);
        assert_eq!(rules.public_folders, ["Api"]);

        std::fs::write(&path, "{}").unwrap();
        assert_eq!(
            Rules::load(app.path()),
            (Rules::default(), RulesSource::File)
        );
    }

    #[test]
    fn a_file_that_cannot_be_read_as_rules_gives_the_defaults_and_is_not_replaced() {
        let app = tempfile::tempdir().unwrap();
        let path = app.path().join(RULES_FILE);
        for content in [
            "{ not json",
            "[1, 2]",
            r#"{"rules": {"no-circular-use": "yes"}}"#,
        ] {
            std::fs::write(&path, content).unwrap();
            let (rules, source) = Rules::load(app.path());
            assert_eq!(rules, Rules::default(), "{content}");
            assert!(matches!(source, RulesSource::Invalid(_)), "{content}");
        }
        std::fs::write(&path, "[1, 2]").unwrap();
        assert!(matches!(
            Rules::default().save(app.path()),
            Err(RulesError::Invalid { .. })
        ));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[1, 2]");
    }

    #[test]
    fn a_class_is_public_by_its_first_folder() {
        let rules = Rules::default();
        assert!(rules.is_public(r"Contracts\PaymentGateway"));
        assert!(rules.is_public(r"Events\Billing\InvoicePaid"));
        assert!(rules.is_public(r"contracts\PaymentGateway"));
        assert!(!rules.is_public(r"Models\Invoice"));
        assert!(!rules.is_public("Invoice"));
        assert!(!rules.is_public(r"ContractsExtra\Thing"));
    }
}
