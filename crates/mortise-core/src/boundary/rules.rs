//! The boundary rules of an app, kept in `mortise.json` in the app folder so that a
//! team shares them. The file also says whether `mortise check` fails CI, and may set
//! the coverage limit and report of the app.
//!
//! It is the only file Mortise writes into an app without a command, and only when
//! the interface asks after the user confirmed. No secret is ever written to it.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The name of the rules file in the app folder.
pub const RULES_FILE: &str = "mortise.json";

/// The two rules of this version, and whether a broken one fails CI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    /// No circular use: two modules must not use each other, directly or through others.
    pub no_circular_use: bool,
    /// Other modules use only a module's public surface.
    pub public_surface_only: bool,
    /// For a module without an accepted public surface, the folders that count as
    /// public: a class in one of them may be used by other modules.
    pub public_folders: Vec<String>,
    /// "Fail CI when a rule breaks": `mortise check` exits 1 when it finds a problem.
    /// Off, it prints the same report and exits 0.
    pub fail_ci: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            no_circular_use: true,
            public_surface_only: true,
            public_folders: vec!["Contracts".to_owned(), "Events".to_owned()],
            fail_ci: true,
        }
    }
}

/// The coverage limit and report of an app: the `coverage` object of `mortise.json`,
/// or the flags of `mortise check`. A value that is set wins over the settings of the
/// window.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageSettings {
    /// A module covered less than this many per cent is a warning. 0 to 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u8>,
    /// The clover report, relative to the app folder or absolute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<PathBuf>,
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

/// The shape of the file. Every key may be left out; a missing one takes its default.
///
/// ```json
/// {
///     "rules": {
///         "no-circular-use": true,
///         "public-surface-only": { "enabled": true, "folders": ["Contracts", "Events"] }
///     },
///     "ci": { "fail": true },
///     "coverage": { "limit": 70, "report": "build/coverage/clover.xml" }
/// }
/// ```
#[derive(Deserialize)]
struct FileShape {
    #[serde(default)]
    rules: RulesInFile,
    #[serde(default)]
    ci: CiInFile,
    #[serde(default)]
    coverage: CoverageSettings,
}

#[derive(Serialize, Deserialize)]
struct CiInFile {
    #[serde(default = "yes")]
    fail: bool,
}

impl Default for CiInFile {
    fn default() -> Self {
        Self { fail: true }
    }
}

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

impl Default for RulesInFile {
    fn default() -> Self {
        Self {
            no_circular_use: true,
            public_surface_only: PublicSurfaceInFile::default(),
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
        let (rules, _, source) = load_file(root);
        (rules, source)
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
        // Other keys of `ci` are kept too.
        let fail = serde_json::Value::Bool(self.fail_ci);
        match file.get_mut("ci") {
            Some(serde_json::Value::Object(ci)) => {
                ci.insert("fail".to_owned(), fail);
            }
            _ => {
                let ci = serde_json::to_value(CiInFile { fail: self.fail_ci })
                    .expect("ci is plain data");
                file.insert("ci".to_owned(), ci);
            }
        }

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
        // A temporary file beside it, then put in its place in one step, so that the
        // file is never half written. Its name is unique, so no file of the user is
        // touched.
        let mut builder = tempfile::Builder::new();
        builder.prefix(".mortise.json.").suffix(".tmp");
        // A temporary file is for its owner only. Ask for what `std::fs::write` gives.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            builder.permissions(std::fs::Permissions::from_mode(0o666));
        }
        let mut temporary = builder.tempfile_in(root).map_err(io)?;
        temporary.write_all(&json).map_err(io)?;
        // This replaces the file on Windows too.
        temporary.persist(&path).map_err(|error| io(error.error))?;
        Ok(())
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

/// Reads `mortise.json` in `root`: the rules and the coverage settings. A file that
/// cannot be read as a whole gives the defaults for both.
pub(crate) fn load_file(root: &Path) -> (Rules, CoverageSettings, RulesSource) {
    let Ok(json) = std::fs::read(root.join(RULES_FILE)) else {
        return (
            Rules::default(),
            CoverageSettings::default(),
            RulesSource::Defaults,
        );
    };
    // An object first: serde would also take a list for the fields of a struct.
    let file = serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&json)
        .and_then(|file| serde_json::from_value::<FileShape>(file.into()))
        .map_err(|error| error.to_string())
        .and_then(|file| match file.coverage.limit {
            Some(limit) if limit > 100 => Err(format!("coverage limit {limit} is over 100")),
            _ => Ok(file),
        });
    match file {
        Ok(file) => (
            Rules {
                no_circular_use: file.rules.no_circular_use,
                public_surface_only: file.rules.public_surface_only.enabled,
                public_folders: file.rules.public_surface_only.folders,
                fail_ci: file.ci.fail,
            },
            file.coverage,
            RulesSource::File,
        ),
        Err(message) => (
            Rules::default(),
            CoverageSettings::default(),
            RulesSource::Invalid(message),
        ),
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
            r#"{"team": "shop", "rules": {"no-circular-use": true}, "ci": {"runner": "linux"}, "coverage": {"limit": 70}}"#,
        )
        .unwrap();
        // A file of the user with a name close to the temporary one.
        let partial = app.path().join("mortise.json.partial");
        std::fs::write(&partial, "mine").unwrap();

        let rules = Rules {
            no_circular_use: false,
            public_surface_only: true,
            public_folders: vec!["Contracts".into(), "Api".into()],
            fail_ci: false,
        };
        rules.save(app.path()).unwrap();
        assert_eq!(Rules::load(app.path()), (rules, RulesSource::File));
        assert_eq!(load_file(app.path()).1.limit, Some(70));

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
    },
    "ci": {
        "runner": "linux",
        "fail": false
    },
    "coverage": {
        "limit": 70
    }
}
"#
        );
        assert_eq!(std::fs::read_to_string(&partial).unwrap(), "mine");
        assert_eq!(
            std::fs::read_dir(app.path()).unwrap().count(),
            2,
            "nothing is left beside it"
        );
        // Readable as a file that `std::fs::write` makes.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode();
            assert_eq!(mode(&path), mode(&partial));
        }
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
        assert!(rules.fail_ci, "CI fails when the file does not say");

        std::fs::write(&path, r#"{"coverage": {"report": "build/clover.xml"}}"#).unwrap();
        let (rules, coverage, source) = load_file(app.path());
        assert_eq!((rules, source), (Rules::default(), RulesSource::File));
        assert_eq!(
            coverage,
            CoverageSettings {
                limit: None,
                report: Some("build/clover.xml".into()),
            }
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
            r#"{"ci": {"fail": "no"}}"#,
            r#"{"coverage": {"limit": 101}}"#,
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
