//! The kind of a file: worked out from the folder it is in, the way the package's
//! generators place files.

use crate::php::ConfigValue;

/// The folders of the package's generators (`paths.generator` in its config) and
/// the name Mortise gives to what is in each.
const DEFAULTS: &[(&str, &str, &str)] = &[
    ("actions", "app/Actions", "Actions"),
    ("casts", "app/Casts", "Casts"),
    ("channels", "app/Broadcasting", "Channels"),
    ("class", "app/Classes", "Classes"),
    ("command", "app/Console", "Commands"),
    ("component-class", "app/View/Components", "Components"),
    ("emails", "app/Emails", "Emails"),
    ("event", "app/Events", "Events"),
    ("enums", "app/Enums", "Enums"),
    ("exceptions", "app/Exceptions", "Exceptions"),
    ("jobs", "app/Jobs", "Jobs"),
    ("job-middleware", "app/Jobs/Middleware", "Job middleware"),
    ("helpers", "app/Helpers", "Helpers"),
    ("interfaces", "app/Interfaces", "Interfaces"),
    ("listener", "app/Listeners", "Listeners"),
    ("model", "app/Models", "Models"),
    ("notifications", "app/Notifications", "Notifications"),
    ("observer", "app/Observers", "Observers"),
    ("policies", "app/Policies", "Policies"),
    ("provider", "app/Providers", "Providers"),
    ("repository", "app/Repositories", "Repositories"),
    ("resource", "app/Transformers", "Resources"),
    ("rules", "app/Rules", "Rules"),
    ("services", "app/Services", "Services"),
    ("scopes", "app/Models/Scopes", "Scopes"),
    ("traits", "app/Traits", "Traits"),
    ("controller", "app/Http/Controllers", "Controllers"),
    ("filter", "app/Http/Middleware", "Middleware"),
    ("request", "app/Http/Requests", "Requests"),
    ("config", "config", "Config"),
    ("factory", "database/factories", "Factories"),
    ("migration", "database/migrations", "Migrations"),
    ("seeder", "database/seeders", "Seeders"),
    ("lang", "lang", "Translations"),
    ("assets", "resources/assets", "Assets"),
    ("views", "resources/views", "Views"),
    ("inertia", "resources/js/Pages", "Pages"),
    (
        "inertia-components",
        "resources/js/Components",
        "Components",
    ),
    ("routes", "routes", "Routes"),
    ("test-feature", "tests/Feature", "Tests"),
    ("test-unit", "tests/Unit", "Tests"),
];

/// The kind of files that are in none of the generator folders.
pub const OTHER: &str = "Other";
/// The kind of test files.
pub const TESTS: &str = "Tests";

/// Maps a path inside a module to a kind.
#[derive(Debug, Clone)]
pub(crate) struct Kinds {
    /// A folder as path parts, and the kind of the files under it.
    folders: Vec<(Vec<String>, &'static str)>,
    /// The folder that holds a module's classes, as path parts.
    app: Vec<String>,
}

impl Kinds {
    /// The folders of the package's defaults, with the paths the app's config
    /// changed. `app_folder` is `app/` unless the app changed it.
    pub(crate) fn new(config: Option<&ConfigValue>, app_folder: &str) -> Self {
        let generator = config.and_then(|config| config.at(&["paths", "generator"]));
        let app = parts(app_folder);
        let mut folders: Vec<(Vec<String>, &'static str)> = Vec::new();
        for (key, default, kind) in DEFAULTS {
            // An entry is `['path' => 'app/Models', ...]`, or the path alone in older configs.
            let path = match generator.and_then(|generator| generator.get(key)) {
                Some(ConfigValue::Str(path)) => path.clone(),
                Some(entry) => match entry.get("path") {
                    Some(ConfigValue::Str(path)) => path.clone(),
                    _ => (*default).to_owned(),
                },
                None => (*default).to_owned(),
            };
            let folder = parts(&path);
            // Before version 11 of the package a module had no app folder: the same
            // folders sit directly in the module.
            if !app.is_empty() && folder.starts_with(&app) && folder.len() > app.len() {
                folders.push((folder[app.len()..].to_vec(), kind));
            }
            folders.push((folder, kind));
        }
        // Any other folder of tests holds tests too.
        folders.push((vec!["tests".to_owned()], TESTS));
        folders.push((vec!["Tests".to_owned()], TESTS));
        Self { folders, app }
    }

    /// The kind of a file, from its path inside the module with `/` between the parts.
    ///
    /// The longest generator folder that holds the file decides. A class in another
    /// folder of the app folder takes the name of that folder, such as `Contracts`.
    pub(crate) fn of(&self, relative: &str) -> String {
        let file = parts(relative);
        let folder = &file[..file.len().saturating_sub(1)];
        let known = self
            .folders
            .iter()
            .filter(|(parts, _)| folder.starts_with(parts))
            .max_by_key(|(parts, _)| parts.len());
        match known {
            Some((_, kind)) => (*kind).to_owned(),
            None if folder.len() > self.app.len() && folder.starts_with(&self.app) => {
                folder[self.app.len()].clone()
            }
            None => OTHER.to_owned(),
        }
    }
}

fn parts(path: &str) -> Vec<String> {
    path.split(['/', '\\'])
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::php::read_config;

    #[test]
    fn the_folder_decides_the_kind() {
        let kinds = Kinds::new(None, "app/");
        assert_eq!(kinds.of("app/Models/Invoice.php"), "Models");
        assert_eq!(kinds.of("app/Models/Scopes/Paid.php"), "Scopes");
        assert_eq!(
            kinds.of("app/Http/Controllers/Api/OrderController.php"),
            "Controllers"
        );
        assert_eq!(kinds.of("app/Http/Requests/StoreOrder.php"), "Requests");
        assert_eq!(
            kinds.of("database/migrations/2026_01_01_000000_create_orders.php"),
            "Migrations"
        );
        assert_eq!(
            kinds.of("resources/views/components/card.blade.php"),
            "Views"
        );
        assert_eq!(kinds.of("tests/Feature/OrderTest.php"), "Tests");
        assert_eq!(kinds.of("tests/Pest.php"), "Tests");
        assert_eq!(kinds.of("routes/api.php"), "Routes");
        assert_eq!(kinds.of("module.json"), "Other");
        assert_eq!(kinds.of("app/Support/Money.php"), "Support");
        assert_eq!(kinds.of("app/Contracts/Billing/Gateway.php"), "Contracts");
        assert_eq!(kinds.of("app/helpers.php"), "Other");
    }

    #[test]
    fn a_module_without_an_app_folder_has_the_same_folders_at_its_top() {
        let kinds = Kinds::new(None, "app/");
        assert_eq!(kinds.of("Models/Invoice.php"), "Models");
        assert_eq!(
            kinds.of("Http/Controllers/OrderController.php"),
            "Controllers"
        );
        assert_eq!(kinds.of("Providers/BlogServiceProvider.php"), "Providers");
    }

    #[test]
    fn the_config_of_the_app_can_move_a_folder() {
        let config = read_config(
            br#"<?php return ['paths' => ['generator' => [
                'model' => ['path' => 'app/Entities', 'generate' => true],
                'controller' => 'src/Controllers',
            ]]];"#,
        )
        .unwrap();
        let kinds = Kinds::new(Some(&config), "app/");
        assert_eq!(kinds.of("app/Entities/Invoice.php"), "Models");
        // No generator owns app/Models any more, so the folder gives the name.
        assert_eq!(kinds.of("app/Models/Invoice.php"), "Models");
        assert_eq!(
            kinds.of("src/Controllers/OrderController.php"),
            "Controllers"
        );
        // What the config does not name keeps the default.
        assert_eq!(kinds.of("app/Events/OrderPaid.php"), "Events");
    }
}
