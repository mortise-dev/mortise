//! Reading fixture apps into snapshots.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mortise_core::{LayoutKey, Manifest, ModuleState, ReadError, read_app};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

/// A copy of a fixture that a test may change.
fn copy_of(name: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture(name), dir.path());
    dir
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Every file under a folder with its content, to prove a read changes nothing.
fn contents(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![dir.to_owned()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    files
}

#[test]
fn reads_shop_api() {
    let app = read_app(&fixture("shop-api")).unwrap();

    assert_eq!(app.name, "shop-api");
    assert!(app.root.is_absolute());
    assert_eq!(app.laravel.as_ref().unwrap().short(), "13.2");
    assert_eq!(app.package.full(), "13.3.0");
    assert_eq!(app.actuator.as_ref().unwrap().full(), "1.0.0");

    assert!(app.layout.has_config);
    assert!(app.layout.assumed.is_empty(), "{:?}", app.layout.assumed);
    assert_eq!(app.layout.namespace, "Modules");
    assert_eq!(app.layout.app_folder, "app/");
    assert_eq!(app.modules_dir_shown(), "Modules");

    let seen: Vec<_> = app
        .modules
        .iter()
        .map(|module| {
            (
                module.name.as_str(),
                module.code.as_str(),
                module.state,
                module.files,
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            ("Billing", "BLN", ModuleState::On, 7),
            ("Catalog", "CTL", ModuleState::On, 7),
            ("Checkout", "CHK", ModuleState::On, 7),
            ("Inventory", "INV", ModuleState::On, 7),
            ("Notifications", "NTF", ModuleState::On, 7),
            ("Shipping", "SHP", ModuleState::On, 7),
            ("Users", "USR", ModuleState::On, 7),
        ]
    );
    assert_eq!(app.modules_on(), 7);
    assert_eq!(app.files(), 49);
    assert!(
        app.modules
            .iter()
            .all(|module| module.manifest == Manifest::Valid)
    );
    assert_eq!(app.path_shown(&app.modules[2].dir), "Modules/Checkout");
}

#[test]
fn reading_changes_no_file() {
    let app = copy_of("shop-api");
    let before = contents(app.path());
    read_app(app.path()).unwrap();
    assert_eq!(before, contents(app.path()));
}

#[test]
fn only_the_json_value_true_turns_a_module_on() {
    let app = copy_of("shop-api");
    std::fs::write(
        app.path().join("modules_statuses.json"),
        r#"{"Billing": true, "Catalog": false, "Checkout": 1, "Inventory": "true", "Notifications": null, "shipping": true}"#,
    )
    .unwrap();

    let state: BTreeMap<_, _> = read_app(app.path())
        .unwrap()
        .modules
        .into_iter()
        .map(|module| (module.name, module.state))
        .collect();

    assert_eq!(state["Billing"], ModuleState::On);
    for off in ["Catalog", "Checkout", "Inventory", "Notifications"] {
        assert_eq!(state[off], ModuleState::Off, "{off}");
    }
    // The key is the exact name: "shipping" is not "Shipping".
    assert_eq!(state["Shipping"], ModuleState::Off);
    // No entry at all.
    assert_eq!(state["Users"], ModuleState::Off);
}

#[test]
fn without_a_statuses_file_every_module_is_off() {
    let app = copy_of("shop-api");
    std::fs::remove_file(app.path().join("modules_statuses.json")).unwrap();
    let app = read_app(app.path()).unwrap();
    assert_eq!(app.modules.len(), 7);
    assert_eq!(app.modules_on(), 0);
}

#[test]
fn another_activator_makes_the_state_unknown() {
    let app = copy_of("shop-api");
    std::fs::write(
        app.path().join("config/modules.php"),
        "<?php return ['activator' => 'database'];",
    )
    .unwrap();
    let app = read_app(app.path()).unwrap();
    assert!(
        app.modules
            .iter()
            .all(|module| module.state == ModuleState::Unknown)
    );
    assert_eq!(app.modules_on(), 0);
}

#[test]
fn the_name_comes_from_module_json_not_from_the_folder() {
    let app = copy_of("shop-api");
    let modules = app.path().join("Modules");
    std::fs::rename(modules.join("Billing"), modules.join("billing-v2")).unwrap();

    let app = read_app(app.path()).unwrap();
    let billing = app
        .modules
        .iter()
        .find(|module| module.name == "Billing")
        .unwrap();
    assert_eq!(billing.state, ModuleState::On);
    assert_eq!(app.path_shown(&billing.dir), "Modules/billing-v2");
}

#[test]
fn a_module_with_a_broken_module_json_is_still_listed_by_its_folder() {
    let app = copy_of("shop-api");
    std::fs::write(
        app.path().join("Modules/Users/module.json"),
        "{\n    \"name\": \"Users\",\n",
    )
    .unwrap();

    let app = read_app(app.path()).unwrap();
    assert_eq!(app.modules.len(), 7);
    let users = app
        .modules
        .iter()
        .find(|module| module.name == "Users")
        .unwrap();
    assert!(
        matches!(users.manifest, Manifest::InvalidJson { line: 3, .. }),
        "{:?}",
        users.manifest
    );
    // The folder name matches the entry in the statuses file here.
    assert_eq!(users.state, ModuleState::On);
}

#[test]
fn folders_without_module_json_and_files_are_not_modules() {
    let app = copy_of("shop-api");
    let modules = app.path().join("Modules");
    std::fs::create_dir(modules.join("Drafts")).unwrap();
    std::fs::write(modules.join("README.md"), "notes").unwrap();
    std::fs::create_dir_all(modules.join("Nested/Deep")).unwrap();
    std::fs::write(
        modules.join("Nested/Deep/module.json"),
        r#"{"name": "Deep"}"#,
    )
    .unwrap();

    assert_eq!(read_app(app.path()).unwrap().modules.len(), 7);
}

#[test]
fn the_modules_folder_of_the_config_is_used() {
    let app = copy_of("shop-api");
    std::fs::rename(app.path().join("Modules"), app.path().join("Domains")).unwrap();
    std::fs::write(
        app.path().join("config/modules.php"),
        "<?php return ['paths' => ['modules' => base_path('Domains')]];",
    )
    .unwrap();

    let app = read_app(app.path()).unwrap();
    assert_eq!(app.modules.len(), 7);
    assert_eq!(app.modules_dir_shown(), "Domains");
}

#[test]
fn a_config_that_needs_php_falls_back_and_says_so() {
    let app = copy_of("shop-api");
    std::fs::write(
        app.path().join("config/modules.php"),
        "<?php return ['paths' => ['modules' => env('MODULES_PATH')]];",
    )
    .unwrap();

    let app = read_app(app.path()).unwrap();
    assert_eq!(app.layout.assumed, [LayoutKey::ModulesFolder]);
    assert_eq!(app.modules.len(), 7);
}

#[test]
fn an_app_without_a_modules_folder_has_no_modules() {
    let app = copy_of("shop-api");
    std::fs::remove_dir_all(app.path().join("Modules")).unwrap();
    assert!(read_app(app.path()).unwrap().modules.is_empty());
}

#[test]
fn a_folder_that_is_not_a_laravel_app() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("composer.json"),
        r#"{"require": {"php": "^8.3"}}"#,
    )
    .unwrap();
    assert!(matches!(
        read_app(dir.path()),
        Err(ReadError::NotLaravel { .. })
    ));
}

#[test]
fn a_laravel_app_without_artisan_is_known_by_its_composer_json() {
    let app = copy_of("shop-api");
    std::fs::remove_file(app.path().join("artisan")).unwrap();
    assert!(read_app(app.path()).is_ok());
}

#[test]
fn an_app_without_a_lock_file() {
    let app = copy_of("shop-api");
    std::fs::remove_file(app.path().join("composer.lock")).unwrap();
    assert!(matches!(
        read_app(app.path()),
        Err(ReadError::NoLock { .. })
    ));
}

#[test]
fn an_app_with_a_broken_lock_file() {
    let app = copy_of("shop-api");
    std::fs::write(app.path().join("composer.lock"), "{").unwrap();
    assert!(matches!(
        read_app(app.path()),
        Err(ReadError::InvalidLock { .. })
    ));
}

#[test]
fn an_app_without_the_package() {
    let app = copy_of("shop-api");
    std::fs::write(
        app.path().join("composer.lock"),
        r#"{"packages": [{"name": "laravel/framework", "version": "v13.2.0"}]}"#,
    )
    .unwrap();
    assert!(matches!(
        read_app(app.path()),
        Err(ReadError::NoPackage { .. })
    ));
}

/// Reads the owner's two real apps. Only reads. Run with:
/// `MORTISE_REAL_APPS=/path/to/apps cargo test -p mortise-core -- --ignored real_apps`
#[test]
#[ignore = "needs MORTISE_REAL_APPS, the folder that holds laravel-12 and laravel-13"]
fn real_apps_read_without_failing() {
    let apps = PathBuf::from(std::env::var("MORTISE_REAL_APPS").expect("MORTISE_REAL_APPS"));
    for (folder, laravel, module) in [("laravel-13", "13.", "Blog"), ("laravel-12", "12.", "Shop")]
    {
        let app = read_app(&apps.join(folder)).unwrap();
        assert!(
            app.laravel.as_ref().unwrap().short().starts_with(laravel),
            "{folder}"
        );
        assert!(
            app.layout.assumed.is_empty(),
            "{folder}: {:?}",
            app.layout.assumed
        );
        let names: Vec<_> = app
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect();
        assert_eq!(names, [module], "{folder}");
        assert_eq!(app.modules[0].state, ModuleState::On, "{folder}");
        assert!(app.modules[0].files > 0, "{folder}");
        println!(
            "{folder}: {} {} files, package {}",
            module, app.modules[0].files, app.package.raw
        );
    }
}
