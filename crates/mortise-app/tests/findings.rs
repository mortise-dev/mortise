//! Findings: the marks in the index, the pip on the rail, the alert on a Summary,
//! the Findings list, and the order "problems first".

mod common;

use std::collections::BTreeMap;

use common::{Desk, fixture, shop_api};
use eframe::egui::{Key, Role, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::{MortiseApp, Selection};
use mortise_app::boundary::BoundaryAction;
use mortise_app::workspace::Tab;
use mortise_app::{findings_view, fonts, theme};
use mortise_core::{
    Action, PackageData, ReadCache, ReadOptions, Target, document, read_app_cached,
};

fn modules(harness: &Harness<'static, MortiseApp>) -> Vec<String> {
    harness
        .get_all_by_role(Role::ListBoxOption)
        .filter_map(|row| row.accesskit_node().label())
        .skip(2) // the tile on the rail, and "All modules"
        .collect()
}

fn rows<State>(harness: &Harness<'_, State>) -> Vec<String> {
    harness
        .query_all_by_role(Role::Row)
        .filter_map(|row| row.accesskit_node().label())
        .collect()
}

#[test]
fn the_home_screen_names_the_module_that_needs_attention() {
    let (_desk, harness) = shop_api();
    // The figure in the stats strip, and the column of the table.
    assert_eq!(harness.get_all_by_label("Findings").count(), 2);
    harness.get_by_label("2 problems");
    harness.get_by_label("1 warning");
    harness.get_by_label(
        "Checkout has 2 problems. It and Shipping use each other (6 files), and 3 of its files \
         read Billing\\Models\\Invoice directly.",
    );
    harness.get_by_role_and_label(Role::Button, "Show all 3 findings");
}

#[test]
fn the_summary_of_a_module_says_what_is_wrong_with_it() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run();
    harness.get_by_label(
        "2 problems. Checkout and Shipping use each other (2 files here, 4 files there), and 3 \
         files read Modules\\Billing\\Models\\Invoice directly.",
    );

    harness
        .get_by_role_and_label(Role::ListBoxOption, "Notifications")
        .click();
    harness.run();
    harness.get_by_label("Notifications is 39% covered. The limit for shop-api is 50%.");

    // Shipping is in the circle that Checkout carries: it says where it is counted.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Shipping")
        .click();
    harness.run();
    harness.get_by_label("Part of a circle counted on Checkout");

    // A module without findings has no alert.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Billing")
        .click();
    harness.run();
    assert!(harness.query_by_label_contains("problem").is_none());
    assert!(
        harness
            .query_by_label_contains("Part of a circle")
            .is_none()
    );
}

#[test]
fn f_opens_the_findings_list_and_esc_returns() {
    let (_desk, mut harness) = shop_api();
    harness.key_press(Key::J);
    harness.key_press(Key::Num2);
    harness.run();

    // From anywhere, F lands on the list of the whole app.
    harness.key_press(Key::F);
    harness.run();
    assert!(harness.state().findings_open());
    assert_eq!(harness.state().selection(), Selection::All);
    assert_eq!(harness.state().tab(), Tab::Summary);
    harness.get_by_role_and_label(Role::Heading, "Findings in shop-api");
    harness.get_by_label("All modules /");
    assert_eq!(
        rows(&harness),
        [
            "Checkout and Shipping use each other. 2 files in Checkout, 4 files in Shipping.",
            "3 files in Checkout read Modules\\Billing\\Models\\Invoice directly. Billing offers \
             Contracts and Events.",
            "Notifications is 39% covered. The limit for shop-api is 50%.",
        ]
    );
    harness.get_by_label("Rule: no circular use");
    harness.get_by_label("Rule: Contracts and Events only");
    harness.get_by_label("Coverage limit");

    // The checks that found nothing, with what each looked at.
    harness.get_by_role_and_label(Role::Heading, "Checks without a finding");
    harness.get_by_label("3 of 6 checks");
    harness.get_by_label("Every module.json is valid JSON");
    harness.get_by_label("Declared namespace matches the folder");
    harness.get_by_label("No entry in modules_statuses.json without a module");
    harness.get_by_label("7 entries");

    harness.key_press(Key::Escape);
    harness.run();
    assert!(!harness.state().findings_open());
    harness.get_by_role_and_label(Role::Heading, "All modules");
}

#[test]
fn a_finding_opens_where_it_lives() {
    let (_desk, mut harness) = shop_api();
    // Boundary was left on a public surface: a link opens on the links.
    harness.key_press(Key::J);
    harness.key_press(Key::Num2);
    harness.key_press(Key::P);
    harness.run();
    harness.key_press(Key::F);
    harness.run();
    assert!(harness.state().findings_open());

    harness
        .get_by_role_and_label(
            Role::Row,
            "Checkout and Shipping use each other. 2 files in Checkout, 4 files in Shipping.",
        )
        .click();
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
    assert_eq!(harness.state().tab(), Tab::Boundary);
    harness.get_by_role_and_label(Role::Heading, "Checkout and Shipping use each other");

    harness.key_press(Key::F);
    harness.run();
    harness
        .get_by_role_and_label(
            Role::Row,
            "Notifications is 39% covered. The limit for shop-api is 50%.",
        )
        .click();
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Notifications".into())
    );
    assert_eq!(harness.state().tab(), Tab::Summary);
    assert!(!harness.state().findings_open());
}

#[test]
fn s_puts_problems_first_and_the_choice_is_kept_for_the_app() {
    let (desk, mut harness) = shop_api();
    assert_eq!(modules(&harness)[0], "Billing");
    harness.get_by_role_and_label(Role::Button, "Sorted: by name");

    harness.key_press(Key::S);
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Sorted: problems first");
    // Problems, then warnings, then the rest by name.
    assert_eq!(
        modules(&harness),
        [
            "Checkout",
            "Notifications",
            "Billing",
            "Catalog",
            "Inventory",
            "Shipping",
            "Users"
        ]
    );
    // J now reaches the module with a problem in one key.
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
    assert!(desk.saved().apps[0].problems_first);

    // A click on the order changes it back.
    harness
        .get_by_role_and_label(Role::Button, "Sorted: problems first")
        .click();
    harness.run();
    assert_eq!(modules(&harness)[0], "Billing");
    assert!(!desk.saved().apps[0].problems_first);

    // The next start has the order that was chosen.
    harness.key_press(Key::S);
    harness.run();
    let harness = desk.start();
    assert_eq!(modules(&harness)[0], "Checkout");
}

#[test]
fn an_app_without_findings_says_what_was_checked() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    std::fs::write(
        root.join("mortise.json"),
        r#"{"rules": {"no-circular-use": false, "public-surface-only": {"enabled": false}}}"#,
    )
    .unwrap();
    std::fs::remove_dir_all(root.join("build")).unwrap();
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();

    harness.get_by_label("Good");
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Show all 3 findings")
            .is_none()
    );
    harness.key_press(Key::F);
    harness.run();
    harness.get_by_label("No findings. 7 modules checked against 0 rules and 3 checks.");
    assert!(rows(&harness).is_empty());
}

#[test]
fn broken_files_of_the_package_are_findings() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    std::fs::write(
        root.join("Modules/Users/module.json"),
        "{\n    \"name\": \"Users\",\n",
    )
    .unwrap();
    let statuses = root.join("modules_statuses.json");
    let json = std::fs::read_to_string(&statuses).unwrap();
    std::fs::write(&statuses, json.replace('{', "{\n    \"Legacy\": true,")).unwrap();
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();

    harness.key_press(Key::F);
    harness.run();
    let rows = rows(&harness);
    assert!(
        rows.contains(
            &"Modules/Users/module.json is not valid JSON: EOF while parsing a value at line 3."
                .to_owned()
        )
    );
    assert!(rows.contains(&"modules_statuses.json lists Legacy, which has no folder.".to_owned()));
    assert_eq!(harness.state().app().unwrap().findings.problems(), 4);
}

/// The output of a recording of `fixtures/output`, a line each.
fn recording(name: &str) -> Vec<String> {
    let path = fixture("output").join(format!("{name}.json"));
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let stdout = json["stdout"].as_str().unwrap();
    stdout.lines().map(str::to_owned).collect()
}

#[test]
fn package_data_adds_translation_warnings_and_work_to_do() {
    // The state of Checkout has one migration waiting; fr lacks 2 keys of en.
    let mut package = PackageData::default();
    let state = document(&recording("module-migrate-status-waiting")).unwrap();
    package.merge_migrations(state, &Target::Module("Checkout".into()), &[], 0);
    package.merge_translations(document(&recording("module-lang-json")).unwrap());
    let options = ReadOptions {
        package,
        ..ReadOptions::default()
    };
    let root = fixture("shop-api");
    let app =
        read_app_cached(&root, &BTreeMap::new(), &options, &mut ReadCache::default()).unwrap();

    let mut harness = Harness::builder()
        .with_size((1100.0, 800.0))
        .with_context_setup(fonts::install)
        .build_ui_state(
            move |ui, (selected, opened): &mut (usize, Option<BoundaryAction>)| {
                let body = ui.max_rect();
                let pal = theme::palette(Theme::Light);
                if let Some(action) =
                    findings_view::findings(ui, pal, body, &app, "shop-api", selected)
                {
                    *opened = Some(action);
                }
            },
            (0, None),
        );
    harness.run();
    let rows = rows(&harness);
    assert_eq!(
        rows[2..],
        [
            "Checkout: 2 keys in en are missing in fr.",
            "Notifications is 39% covered. The limit for shop-api is 50%.",
            "Checkout has 1 migration waiting.",
        ]
    );
    harness.get_by_label("To do, not counted as findings");
    harness.get_by_label("3 of 7 checks");

    // Each opens the Actions tab of Checkout on its action.
    for (row, action) in [
        (
            "Checkout: 2 keys in en are missing in fr.",
            Action::CheckTranslations,
        ),
        ("Checkout has 1 migration waiting.", Action::Migrate),
    ] {
        harness.get_by_role_and_label(Role::Row, row).click();
        harness.run();
        assert_eq!(
            harness.state().1,
            Some(BoundaryAction::OpenModuleAction {
                module: "Checkout".into(),
                action
            })
        );
    }
}
