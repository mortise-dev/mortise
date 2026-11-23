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
            let coverage = module.coverage.and_then(|coverage| coverage.percent());
            (
                module.name.as_str(),
                module.code.as_str(),
                module.files,
                module.lines,
                module.tests,
                coverage,
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            ("Billing", "BLN", 19, 203, 5, Some(81)),
            ("Catalog", "CTL", 10, 90, 4, Some(77)),
            ("Checkout", "CHK", 20, 212, 2, Some(54)),
            ("Inventory", "INV", 9, 75, 2, Some(72)),
            ("Notifications", "NTF", 9, 68, 0, Some(39)),
            ("Shipping", "SHP", 14, 128, 2, Some(66)),
            ("Users", "USR", 10, 85, 3, Some(88)),
        ]
    );
    assert!(
        app.modules
            .iter()
            .all(|module| module.state == ModuleState::On)
    );
    assert_eq!(app.modules_on(), 7);
    assert_eq!(app.files(), 91);
    assert_eq!(app.lines(), 861);
    assert_eq!(app.tests(), 18);
    // 160 of 234 statements. The file outside the modules folder does not count.
    assert_eq!(app.coverage().unwrap().percent(), Some(68));
    assert_eq!(
        app.path_shown(&app.coverage_report.as_ref().unwrap().path),
        "build/coverage/clover.xml"
    );
    assert!(app.coverage_report.as_ref().unwrap().written.is_some());

    let kinds = |name: &str| -> Vec<(String, u32)> {
        let module = app
            .modules
            .iter()
            .find(|module| module.name == name)
            .unwrap();
        module
            .kinds
            .iter()
            .map(|kind| (kind.kind.clone(), kind.files))
            .collect()
    };
    let checkout = kinds("Checkout");
    assert_eq!(checkout[0], ("Listeners".to_owned(), 4));
    assert_eq!(checkout[1], ("Services".to_owned(), 3));
    assert_eq!(checkout.iter().map(|(_, files)| files).sum::<u32>(), 20);
    assert!(kinds("Billing").contains(&("Commands".to_owned(), 2)));
    assert!(kinds("Billing").contains(&("Migrations".to_owned(), 1)));

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
fn without_a_coverage_report_coverage_is_unknown() {
    let app = copy_of("shop-api");
    std::fs::remove_dir_all(app.path().join("build")).unwrap();
    let app = read_app(app.path()).unwrap();
    assert_eq!(app.coverage_report, None);
    assert_eq!(app.coverage(), None);
    assert!(app.modules.iter().all(|module| module.coverage.is_none()));
}

#[test]
fn a_module_that_is_not_in_the_report_has_no_coverage() {
    let app = copy_of("shop-api");
    let report = app.path().join("build/coverage/clover.xml");
    let xml = std::fs::read_to_string(&report).unwrap();
    std::fs::write(&report, xml.replace("Modules/Users/", "Modules/Members/")).unwrap();

    let app = read_app(app.path()).unwrap();
    let users = app
        .modules
        .iter()
        .find(|module| module.name == "Users")
        .unwrap();
    assert_eq!(users.coverage, None);
    // 138 of 209 statements without the 22 of 25 of Users.
    assert_eq!(app.coverage().unwrap().percent(), Some(66));
}

#[test]
fn the_report_named_in_phpunit_xml_is_read() {
    let app = copy_of("shop-api");
    std::fs::rename(
        app.path().join("build/coverage/clover.xml"),
        app.path().join("cov.xml"),
    )
    .unwrap();
    assert_eq!(read_app(app.path()).unwrap().coverage(), None);
    std::fs::write(
        app.path().join("phpunit.xml"),
        r#"<phpunit><coverage><report><clover outputFile="cov.xml"/></report></coverage></phpunit>"#,
    )
    .unwrap();
    let app = read_app(app.path()).unwrap();
    assert_eq!(
        app.path_shown(&app.coverage_report.as_ref().unwrap().path),
        "cov.xml"
    );
    assert_eq!(app.coverage().unwrap().percent(), Some(68));
}

#[test]
fn a_folder_that_is_not_a_repository_has_no_git_data() {
    let app = copy_of("shop-api");
    let app = read_app(app.path()).unwrap();
    assert_eq!(app.git, mortise_core::Git::NotARepository);
    assert!(app.modules.iter().all(|module| module.churn.is_none()));
    assert!(
        app.modules
            .iter()
            .all(|module| module.uncommitted.is_none())
    );
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

/// Prints what Mortise reads from the shop-api fixture. Run with `-- --ignored print`.
#[test]
#[ignore = "prints the snapshot for a person to read"]
fn print_shop_api() {
    let app = read_app(&fixture("shop-api")).unwrap();
    println!(
        "lines {} tests {} coverage {:?} git {:?}",
        app.lines(),
        app.tests(),
        app.coverage().and_then(|c| c.percent()),
        app.git
    );
    for module in &app.modules {
        println!(
            "{:14} {} files {:3} lines {:4} tests {:2} cov {:?} kinds {:?}",
            module.name,
            module.code,
            module.files,
            module.lines,
            module.tests,
            module.coverage.and_then(|c| c.percent()),
            module
                .kinds
                .iter()
                .map(|k| format!("{} {}", k.kind, k.files))
                .collect::<Vec<_>>()
        );
    }
}

/// Prints the links of the shop-api fixture. Run with `-- --ignored print_links`.
#[test]
#[ignore = "prints the links for a person to read"]
fn print_links_of_shop_api() {
    let app = read_app(&fixture("shop-api")).unwrap();
    for link in &app.links {
        println!(
            "{} uses {}: {} refs, {} files, {} internal, circular {}",
            link.from,
            link.to,
            link.refs.len(),
            link.files(),
            link.internal_files(),
            link.circular
        );
    }
    println!("cycles {:?}", app.cycles);
}

#[test]
fn finds_which_module_uses_which() {
    let app = read_app(&fixture("shop-api")).unwrap();
    // A test of Billing uses Order of Checkout. Test files make no links, so
    // Billing uses no module.
    let links: Vec<_> = app
        .links
        .iter()
        .map(|link| {
            (
                link.from.as_str(),
                link.to.as_str(),
                link.files(),
                link.internal_files(),
                link.circular,
            )
        })
        .collect();
    assert_eq!(
        links,
        [
            ("Checkout", "Billing", 6, 3, false),
            ("Checkout", "Catalog", 1, 0, false),
            ("Checkout", "Shipping", 2, 0, true),
            ("Checkout", "Users", 1, 0, false),
            ("Inventory", "Catalog", 1, 0, false),
            ("Notifications", "Billing", 1, 0, false),
            ("Notifications", "Checkout", 1, 0, false),
            ("Shipping", "Checkout", 4, 0, true),
        ]
    );

    // The files of a link: those that read internals first, each with its line.
    let billing = app.link("Checkout", "Billing").unwrap();
    let refs: Vec<_> = billing
        .refs
        .iter()
        .map(|reference| {
            (
                reference.file.as_str(),
                reference.line,
                reference.class.as_str(),
                reference.internal,
            )
        })
        .collect();
    assert_eq!(
        refs,
        [
            (
                "Modules/Checkout/app/Http/Controllers/OrderController.php",
                5,
                r"Modules\Billing\Models\Invoice",
                true
            ),
            (
                "Modules/Checkout/app/Listeners/CreateInvoiceOnPaid.php",
                5,
                r"Modules\Billing\Models\Invoice",
                true
            ),
            (
                "Modules/Checkout/app/Services/InvoiceSync.php",
                5,
                r"Modules\Billing\Models\Invoice",
                true
            ),
            (
                "Modules/Checkout/app/Listeners/MarkOrderPaid.php",
                5,
                r"Modules\Billing\Events\InvoicePaid",
                false
            ),
            (
                "Modules/Checkout/app/Listeners/SendReceipt.php",
                5,
                r"Modules\Billing\Events\InvoicePaid",
                false
            ),
            (
                "Modules/Checkout/app/Services/PaymentService.php",
                5,
                r"Modules\Billing\Contracts\PaymentGateway",
                false
            ),
        ]
    );

    // Checkout and Shipping use each other. Counted on Checkout, the side with fewer files.
    assert_eq!(app.cycles.len(), 1);
    assert_eq!(app.cycles[0].modules, ["Checkout", "Shipping"]);
    assert_eq!(app.cycles[0].carrier, "Checkout");
    assert_eq!(
        app.cycles[0].files,
        [("Checkout".to_owned(), 2), ("Shipping".to_owned(), 4)]
    );

    let problems: Vec<_> = app
        .problem_links()
        .map(|link| (link.from.as_str(), link.to.as_str()))
        .collect();
    assert_eq!(
        problems,
        [
            ("Checkout", "Billing"),
            ("Checkout", "Shipping"),
            ("Shipping", "Checkout")
        ]
    );
    assert_eq!(app.uses("Checkout").count(), 4);
    assert_eq!(app.used_by("Checkout").count(), 2);
    assert_eq!(app.rules_source, mortise_core::RulesSource::Defaults);
    assert_eq!(app.module("Billing").unwrap().namespace, r"Modules\Billing");
}

#[test]
fn the_rules_of_mortise_json_decide_what_is_a_problem() {
    let app = copy_of("shop-api");
    std::fs::write(
        app.path().join("mortise.json"),
        r#"{"rules": {"no-circular-use": false, "public-surface-only": {"enabled": true, "folders": ["Contracts", "Events", "Models"]}}}"#,
    )
    .unwrap();
    let app = read_app(app.path()).unwrap();
    assert_eq!(app.rules_source, mortise_core::RulesSource::File);
    // Models is public now, and a circle is allowed: nothing breaks a rule.
    assert_eq!(app.problem_links().count(), 0);
    assert_eq!(app.link("Checkout", "Billing").unwrap().internal_files(), 0);
    // The circle is still known, but no module is told it is counted elsewhere.
    assert_eq!(app.cycles.len(), 1);
    for member in ["Checkout", "Shipping"] {
        assert!(app.circles_counted_elsewhere(member).is_empty());
    }
}

#[test]
fn a_module_without_composer_json_takes_its_namespace_from_the_config() {
    let app = copy_of("shop-api");
    std::fs::remove_file(app.path().join("Modules/Billing/composer.json")).unwrap();
    let app = read_app(app.path()).unwrap();
    assert_eq!(app.module("Billing").unwrap().namespace, r"Modules\Billing");
    assert_eq!(app.link("Checkout", "Billing").unwrap().files(), 6);
}

/// The lines of the standard output of a recording in `fixtures/output`.
fn recorded(name: &str) -> Vec<String> {
    let path = fixture("output").join(format!("{name}.json"));
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    json["stdout"]
        .as_str()
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

#[test]
fn finds_what_breaks_a_rule_or_a_check() {
    use mortise_core::{
        Check, Detail, FindingState, PackageData, ReadCache, ReadOptions, Severity, Target, Todo,
        document, read_app_cached,
    };

    // The user read the migration state of every module and checked the translations
    // of Checkout: 2 keys of en are missing in fr.
    let mut package = PackageData::default();
    let state = document(&recorded("module-migrate-status-all")).unwrap();
    package.merge_migrations(state, &Target::All, &[], 0);
    package.merge_translations(document(&recorded("module-lang-json")).unwrap());
    let options = ReadOptions {
        package,
        ..ReadOptions::default()
    };
    let root = fixture("shop-api");
    let app =
        read_app_cached(&root, &BTreeMap::new(), &options, &mut ReadCache::default()).unwrap();
    let findings = &app.findings;
    assert_eq!((findings.problems(), findings.warnings()), (2, 2));

    // A circle is counted once, on Checkout: the side with fewer files to change.
    let circle = &findings.list[0];
    assert_eq!(
        (circle.severity, circle.check),
        (Severity::Problem, Check::NoCircularUse)
    );
    assert_eq!(circle.module.as_deref(), Some("Checkout"));
    assert_eq!(
        circle.detail,
        Detail::CircularUse {
            files: vec![("Checkout".into(), 2), ("Shipping".into(), 4)]
        }
    );

    let internals = &findings.list[1];
    assert_eq!(internals.check, Check::PublicSurfaceOnly);
    assert_eq!(internals.module.as_deref(), Some("Checkout"));
    assert_eq!(
        internals.detail,
        Detail::ReadsInternals {
            of: "Billing".into(),
            files: 3,
            classes: vec![r"Modules\Billing\Models\Invoice".into()],
        }
    );

    // A translation warning counts like any warning.
    let translations = &findings.list[2];
    assert_eq!(
        (translations.severity, translations.check),
        (Severity::Warning, Check::TranslationsComplete)
    );
    assert_eq!(translations.module.as_deref(), Some("Checkout"));
    assert_eq!(
        translations.detail,
        Detail::MissingTranslations {
            locale: "fr".into(),
            keys: 2,
            files: 0,
            from: Some("en".into())
        }
    );

    let coverage = &findings.list[3];
    assert_eq!(
        (coverage.severity, coverage.check),
        (Severity::Warning, Check::CoverageLimit)
    );
    assert_eq!(coverage.module.as_deref(), Some("Notifications"));
    assert_eq!(
        coverage.detail,
        Detail::LowCoverage {
            percent: 39,
            limit: 50
        }
    );

    assert_eq!(findings.state_of("Checkout"), FindingState::Problems(2));
    assert_eq!(findings.state_of("Notifications"), FindingState::Warning);
    assert_eq!(findings.state_of("Billing"), FindingState::Good);
    // Shipping is in the circle, but the circle is counted on Checkout.
    assert_eq!(findings.state_of("Shipping"), FindingState::Good);
    assert_eq!(app.circles_counted_elsewhere("Shipping"), ["Checkout"]);
    assert!(app.circles_counted_elsewhere("Checkout").is_empty());
    assert!(app.circles_counted_elsewhere("Billing").is_empty());

    // Waiting migrations are to do, not findings: Billing above is still good.
    assert_eq!(
        findings.todo,
        [
            Todo {
                module: "Billing".into(),
                waiting: 2
            },
            Todo {
                module: "Checkout".into(),
                waiting: 1
            }
        ]
    );

    // Two rules and five checks ran. Three of the seven found nothing.
    // Translations looked at the one module with two locales.
    assert_eq!(findings.rules_and_checks(), (2, 5));
    let translations = findings.checks.last().unwrap();
    assert_eq!(
        (
            translations.check,
            translations.checked,
            translations.findings
        ),
        (Check::TranslationsComplete, 1, 1)
    );
    let quiet: Vec<_> = findings
        .checks
        .iter()
        .filter(|result| result.findings == 0)
        .map(|result| (result.check, result.checked))
        .collect();
    assert_eq!(
        quiet,
        [
            (Check::ValidManifest, 7),
            (Check::NamespaceMatchesFolder, 7),
            (Check::NoStaleStatus, 7)
        ]
    );
}

#[test]
fn finds_broken_files_of_the_package() {
    use mortise_core::{Check, Detail, Severity};

    let app = copy_of("shop-api");
    let modules = app.path().join("Modules");
    // module.json that is not valid JSON.
    std::fs::write(
        modules.join("Users/module.json"),
        "{\n    \"name\": \"Users\",\n",
    )
    .unwrap();
    // A namespace that does not match the folder.
    let composer = modules.join("Catalog/composer.json");
    let json = std::fs::read_to_string(&composer).unwrap();
    std::fs::write(
        &composer,
        json.replace(r"Modules\\Catalog\\", r"Modules\\Catalogue\\"),
    )
    .unwrap();
    // An entry in the statuses file without a module.
    let statuses = app.path().join("modules_statuses.json");
    let json = std::fs::read_to_string(&statuses).unwrap();
    std::fs::write(&statuses, json.replace("{", "{\n    \"Legacy\": true,")).unwrap();

    let app = read_app(app.path()).unwrap();
    let by_check = |check: Check| -> Vec<_> {
        app.findings
            .list
            .iter()
            .filter(|finding| finding.check == check)
            .collect()
    };

    let manifest = by_check(Check::ValidManifest);
    assert_eq!(manifest.len(), 1);
    assert_eq!(manifest[0].severity, Severity::Problem);
    assert_eq!(manifest[0].module.as_deref(), Some("Users"));
    assert_eq!(
        manifest[0].detail,
        Detail::InvalidManifest {
            file: "Modules/Users/module.json".into(),
            message: "EOF while parsing a value".into(),
            line: 3,
        }
    );

    let namespace = by_check(Check::NamespaceMatchesFolder);
    assert_eq!(namespace.len(), 1);
    assert_eq!(namespace[0].module.as_deref(), Some("Catalog"));
    assert_eq!(
        namespace[0].detail,
        Detail::NamespaceMismatch {
            namespace: r"Modules\Catalogue".into(),
            folder: "Modules/Catalog".into()
        }
    );

    let stale = by_check(Check::NoStaleStatus);
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].module, None);
    assert_eq!(
        stale[0].detail,
        Detail::StaleStatus {
            entry: "Legacy".into(),
            file: "modules_statuses.json".into()
        }
    );
    let result = app
        .findings
        .checks
        .iter()
        .find(|result| result.check == Check::NoStaleStatus)
        .unwrap();
    assert_eq!((result.checked, result.findings), (8, 1));
}

#[test]
fn a_rule_that_is_off_is_not_checked_and_no_report_means_no_coverage_check() {
    use mortise_core::Check;

    let app = copy_of("shop-api");
    std::fs::write(
        app.path().join("mortise.json"),
        r#"{"rules": {"no-circular-use": false, "public-surface-only": {"enabled": false}}}"#,
    )
    .unwrap();
    std::fs::remove_dir_all(app.path().join("build")).unwrap();

    let app = read_app(app.path()).unwrap();
    assert!(app.findings.list.is_empty(), "{:?}", app.findings.list);
    assert_eq!(app.findings.rules_and_checks(), (0, 3));
    assert!(
        !app.findings
            .checks
            .iter()
            .any(|result| result.check == Check::CoverageLimit)
    );
}

#[test]
fn a_module_that_is_off_gets_no_coverage_warning_and_breaks_no_rule() {
    let app = copy_of("shop-api");
    let statuses = app.path().join("modules_statuses.json");
    let json = std::fs::read_to_string(&statuses).unwrap();
    let json = ["Notifications", "Billing", "Shipping"]
        .iter()
        .fold(json, |json, name| {
            json.replace(&format!("\"{name}\": true"), &format!("\"{name}\": false"))
        });
    std::fs::write(&statuses, json).unwrap();
    let app = read_app(app.path()).unwrap();
    assert_eq!(app.findings.warnings(), 0);

    // Checkout reads internals of Billing, which is off: no finding, but the link stays.
    // The circle of Checkout and Shipping does not run with Shipping off.
    assert_eq!(app.findings.problems(), 0);
    assert_eq!(app.problem_links().count(), 0);
    let billing = app.link("Checkout", "Billing").unwrap();
    assert_eq!((billing.files(), billing.internal_files()), (6, 0));
    for member in ["Checkout", "Shipping"] {
        assert!(app.circles_counted_elsewhere(member).is_empty());
    }
}
