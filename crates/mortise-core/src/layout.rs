//! Where the package keeps things in an app: read from `config/modules.php` without running PHP.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::php::{ConfigValue, read_config};

/// A setting of `config/modules.php` that Mortise needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutKey {
    /// `namespace`
    Namespace,
    /// `paths.modules`
    ModulesFolder,
    /// `paths.app_folder`
    AppFolder,
    /// `activators.file.statuses-file`
    StatusesFile,
    /// `activator`
    Activator,
}

impl LayoutKey {
    /// The key as it is written in `config/modules.php`.
    pub fn config_key(self) -> &'static str {
        match self {
            LayoutKey::Namespace => "namespace",
            LayoutKey::ModulesFolder => "paths.modules",
            LayoutKey::AppFolder => "paths.app_folder",
            LayoutKey::StatusesFile => "activators.file.statuses-file",
            LayoutKey::Activator => "activator",
        }
    }
}

/// Where the modules of an app live and how their state is kept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// The folder that holds one folder per module. Absolute.
    pub modules_dir: PathBuf,
    /// The namespace all modules share, `Modules` by default.
    pub namespace: String,
    /// The folder inside a module that holds its classes, `app/` since version 11 of the package.
    pub app_folder: String,
    /// The file that records which modules are on. Absolute.
    pub statuses_file: PathBuf,
    /// The activator the app uses. Only `file` keeps the state in `statuses_file`.
    pub activator: String,
    /// True when the app has a `config/modules.php`.
    pub has_config: bool,
    /// Settings that are in the config file but need PHP to evaluate.
    /// The package default is used for each, which can be wrong.
    pub assumed: Vec<LayoutKey>,
}

impl Layout {
    /// Reads the layout of the app in `root`. A missing config file means the package defaults.
    pub(crate) fn read(root: &Path) -> Self {
        let mut layout = Self {
            modules_dir: root.join("Modules"),
            namespace: "Modules".into(),
            app_folder: "app/".into(),
            statuses_file: root.join("modules_statuses.json"),
            activator: "file".into(),
            has_config: false,
            assumed: Vec::new(),
        };

        let Ok(source) = std::fs::read(root.join("config").join("modules.php")) else {
            return layout;
        };
        layout.has_config = true;

        let Some(config) = read_config(&source) else {
            // The file does not return an array at its top level, so nothing in it can be trusted.
            layout.assumed = vec![
                LayoutKey::Namespace,
                LayoutKey::ModulesFolder,
                LayoutKey::AppFolder,
                LayoutKey::StatusesFile,
                LayoutKey::Activator,
            ];
            return layout;
        };

        // A top-level key of the app's file replaces the package's whole value for that key,
        // so a missing nested key is not the default at run time. Mortise still uses the
        // default here: it is what the package's own fallbacks do for the modules folder,
        // and for the rest there is nothing better to show.
        let mut text = |path: &[&str], key: LayoutKey, target: &mut String| match config.at(path) {
            Some(ConfigValue::Str(value)) => *target = value.clone(),
            Some(_) => layout.assumed.push(key),
            None => {}
        };
        text(&["namespace"], LayoutKey::Namespace, &mut layout.namespace);
        text(
            &["paths", "app_folder"],
            LayoutKey::AppFolder,
            &mut layout.app_folder,
        );
        text(&["activator"], LayoutKey::Activator, &mut layout.activator);

        let mut path = |path: &[&str], key: LayoutKey, target: &mut PathBuf| match config.at(path) {
            Some(ConfigValue::BasePath(relative)) => *target = root.join(relative),
            // A plain string is a path as PHP would see it: absolute, or relative to
            // wherever PHP runs. Only the absolute form can be followed from here.
            Some(ConfigValue::Str(value)) if Path::new(value).is_absolute() => {
                *target = PathBuf::from(value);
            }
            Some(_) => layout.assumed.push(key),
            None => {}
        };
        path(
            &["paths", "modules"],
            LayoutKey::ModulesFolder,
            &mut layout.modules_dir,
        );
        path(
            &["activators", "file", "statuses-file"],
            LayoutKey::StatusesFile,
            &mut layout.statuses_file,
        );

        layout
    }

    /// True when the on or off state of modules is in the statuses file.
    pub fn uses_file_activator(&self) -> bool {
        self.activator == "file"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with_config(config: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        if let Some(config) = config {
            std::fs::create_dir(dir.path().join("config")).unwrap();
            std::fs::write(dir.path().join("config/modules.php"), config).unwrap();
        }
        dir
    }

    #[test]
    fn no_config_file_means_the_package_defaults() {
        let app = app_with_config(None);
        let layout = Layout::read(app.path());
        assert_eq!(layout.modules_dir, app.path().join("Modules"));
        assert_eq!(layout.namespace, "Modules");
        assert_eq!(layout.app_folder, "app/");
        assert_eq!(
            layout.statuses_file,
            app.path().join("modules_statuses.json")
        );
        assert!(layout.uses_file_activator());
        assert!(!layout.has_config);
        assert!(layout.assumed.is_empty());
    }

    #[test]
    fn reads_what_the_app_changed() {
        let app = app_with_config(Some(
            r#"<?php return [
                'namespace' => 'Acme',
                'paths' => ['modules' => base_path('src/Domains'), 'app_folder' => 'src/'],
                'activators' => ['file' => ['statuses-file' => base_path('storage/modules.json')]],
                'activator' => 'database',
            ];"#,
        ));
        let layout = Layout::read(app.path());
        assert_eq!(layout.modules_dir, app.path().join("src/Domains"));
        assert_eq!(layout.namespace, "Acme");
        assert_eq!(layout.app_folder, "src/");
        assert_eq!(
            layout.statuses_file,
            app.path().join("storage/modules.json")
        );
        assert!(!layout.uses_file_activator());
        assert!(layout.has_config);
        assert!(layout.assumed.is_empty());
    }

    #[test]
    fn what_needs_php_is_assumed_and_named() {
        let app = app_with_config(Some(
            r#"<?php return [
                'namespace' => env('MODULES_NAMESPACE', 'Modules'),
                'paths' => ['modules' => __DIR__ . '/../Modules', 'app_folder' => 'app/'],
            ];"#,
        ));
        let layout = Layout::read(app.path());
        assert_eq!(layout.namespace, "Modules");
        assert_eq!(layout.modules_dir, app.path().join("Modules"));
        assert_eq!(
            layout.assumed,
            vec![LayoutKey::Namespace, LayoutKey::ModulesFolder]
        );
    }

    #[test]
    fn a_relative_string_path_cannot_be_followed() {
        let app = app_with_config(Some("<?php return ['paths' => ['modules' => 'Modules']];"));
        let layout = Layout::read(app.path());
        assert_eq!(layout.modules_dir, app.path().join("Modules"));
        assert_eq!(layout.assumed, vec![LayoutKey::ModulesFolder]);
    }

    #[test]
    fn a_config_file_that_returns_nothing_assumes_everything() {
        let app = app_with_config(Some("<?php $config = [];"));
        let layout = Layout::read(app.path());
        assert!(layout.has_config);
        assert_eq!(layout.assumed.len(), 5);
    }
}
