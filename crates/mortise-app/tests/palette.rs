//! The command palette, run headless.

mod common;

use common::{Desk, shop_api, wait_until};
use eframe::egui::{Key, Modifiers, Role, Shape};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::MortiseApp;
use mortise_app::index::Selection;
use mortise_app::palette;
use mortise_app::workspace::Tab;
use mortise_core::history::History;
use mortise_core::{Catalog, MODULE_MAKE, Version, catalog_cache};

/// The rows of the palette. Rows of the rail and the index are behind it.
fn palette_rows(harness: &Harness<'static, MortiseApp>) -> Vec<String> {
    let all: Vec<String> = harness
        .get_all_by_role(Role::ListBoxOption)
        .filter_map(|row| row.accesskit_node().label())
        .collect();
    // The workbench has the tiles of the rail, "All modules" and the modules.
    let behind = harness.state().app_names().len()
        + 1
        + harness.state().app().map_or(0, |app| app.modules.len());
    all[behind..].to_vec()
}

fn open_palette(harness: &mut Harness<'static, MortiseApp>) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run_steps(3);
}

fn type_query(harness: &mut Harness<'static, MortiseApp>, text: &str) {
    harness
        .get_by_role_and_label(Role::TextInput, "Search or run a command")
        .type_text(text);
    harness.run_steps(3);
}

#[test]
fn an_empty_query_shows_places_apps_and_what_ran() {
    let desk = Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    // A module with a long name, for a long row.
    let long = shop.join("Modules/CustomerNotifications");
    common::copy_dir(&shop.join("Modules/Notifications"), &long);
    common::rename_in_files(&long, "Notifications", "CustomerNotifications");
    desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
    // A migrate 2 minutes ago, a refresh and a model before it, in the history of shop-api.
    let now_ms = common::ran(
        &desk,
        &shop,
        &[
            ("module:make-model Invoice Billing", 5),
            ("module:migrate-refresh CustomerNotifications", 3),
            ("module:migrate Checkout", 2),
        ],
    );
    let mut harness = desk.start();
    harness.key_press(Key::J);
    harness.run();

    open_palette(&mut harness);
    harness.get_by_label("Recent");
    harness.get_by_label("Go to");
    assert_eq!(
        palette_rows(&harness),
        [
            "Billing",
            "Migrate Checkout",
            "Refresh migrations CustomerNotifications",
            "Generate a model in Billing",
            "All modules",
            "Findings in shop-api",
            "Settings",
            "crm",
            "Add app"
        ]
    );
    harness.get_by_label("Up and Down to move, Enter to run, Tab to switch app");
    let app = harness.state().app().expect("the app is read");
    let history = History::open(&desk.data_dir(), &shop);
    let entries = history.entries();
    let source = palette::Source {
        scope: Some((0, &app)),
        apps: &[("shop-api".to_owned(), 0)],
        recent: &[],
        history: &entries,
        catalog: mortise_core::Catalog::v13_3(),
        now: now_ms / 1000,
        selected: None,
        mac: false,
    };
    let found = palette::entries("", &source);
    assert_eq!(
        [&found[0].hint, &found[2].hint],
        ["ran 2 minutes ago", "Form"]
    );
    assert_eq!(found[0].detail.as_deref(), Some("module:migrate Checkout"));
    assert_eq!(found[2].detail.as_deref(), Some("module:make-model"));

    // Where a text is painted.
    let painted = |harness: &Harness<'static, MortiseApp>, text: &str| {
        harness
            .output()
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Text(shape) if shape.galley.text() == text => {
                    Some(shape.visual_bounding_rect())
                }
                _ => None,
            })
    };
    // The long detail of the current refresh row ends before its age.
    harness.key_press(Key::ArrowDown);
    harness.run_steps(2);
    harness.key_press(Key::ArrowDown);
    harness.run_steps(2);
    let detail = painted(&harness, "module:migrate-refresh CustomerNotifications").unwrap();
    let age = painted(&harness, "ran 3 minutes ago").unwrap();
    assert!(detail.right() < age.left(), "{detail:?} runs into {age:?}");

    // The current row keeps its age, and Enter on it opens its panel and runs nothing.
    harness.key_press(Key::ArrowUp);
    harness.run_steps(2);
    assert!(painted(&harness, "ran 2 minutes ago").is_some());
    harness.key_press(Key::Enter);
    harness.run_steps(3);
    assert_eq!(harness.state().tab(), Tab::Actions);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
    harness.get_by_role_and_label(Role::Heading, "Migrate Checkout");
    assert_eq!(history.entries().len(), 3);
}

#[test]
fn a_query_finds_modules_and_findings_and_enter_opens_the_first() {
    let (_desk, mut harness) = shop_api();
    open_palette(&mut harness);
    type_query(&mut harness, "check");

    harness.get_by_label("Modules in shop-api");
    harness.get_by_label("Actions for Checkout");
    let app = harness.state().app().expect("the app is read");
    let source = palette::Source {
        scope: Some((0, &app)),
        apps: &[("shop-api".to_owned(), 0)],
        recent: &[],
        history: &[],
        catalog: mortise_core::Catalog::v13_3(),
        now: 0,
        selected: None,
        mac: false,
    };
    let found = palette::entries("check", &source);
    let migrate = found
        .iter()
        .find(|entry| entry.label == "Migrate Checkout")
        .expect("the action is found");
    assert_eq!(migrate.group, "Actions for Checkout");
    assert_eq!(migrate.detail.as_deref(), Some("module:migrate Checkout"));
    assert_eq!(
        migrate.command,
        palette::Command::Action {
            module: Some("Checkout".into()),
            action: mortise_core::Action::Migrate
        }
    );
    assert_eq!(
        palette_rows(&harness),
        [
            "Checkout",
            "Check translations in Checkout",
            "Turn off Checkout",
            "Migrate Checkout",
            "Roll back Checkout",
            "Checkout and Shipping use each other",
            "3 files in Checkout read Billing internals",
            "Scan the public surface of Checkout"
        ]
    );
    harness.key_press(Key::Enter);
    harness.run_steps(3);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
    // The palette closed.
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Search or run a command")
            .is_none()
    );

    // A destructive action is listed only by its own word, and opens its panel.
    let labels = |query: &str| -> Vec<String> {
        palette::entries(query, &source)
            .into_iter()
            .map(|entry| entry.label)
            .collect()
    };
    assert!(!labels("refresh checkout").contains(&"Drop all tables and migrate".to_owned()));
    open_palette(&mut harness);
    type_query(&mut harness, "delete checkout");
    assert_eq!(palette_rows(&harness)[0], "Delete module Checkout");
    harness.key_press(Key::Enter);
    harness.run_steps(3);
    assert_eq!(harness.state().tab(), Tab::Actions);
    harness.get_by_role_and_label(Role::Heading, "Delete the module Checkout");
    harness.get_by_role_and_label(Role::ListBoxOption, "Reset migrations");
}

#[test]
fn down_moves_to_a_finding_and_enter_opens_it_where_it_lives() {
    let (_desk, mut harness) = shop_api();
    // Boundary was left on a public surface: a link opens on the links.
    harness.key_press(Key::J);
    harness.key_press(Key::Num2);
    harness.key_press(Key::P);
    harness.run();
    open_palette(&mut harness);
    type_query(&mut harness, "each other");
    assert_eq!(
        palette_rows(&harness),
        ["Checkout and Shipping use each other"]
    );
    harness.key_press(Key::Enter);
    harness.run_steps(3);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
    assert_eq!(harness.state().tab(), Tab::Boundary);
    harness.get_by_role_and_label(Role::Heading, "Checkout and Shipping use each other");

    // Down, Down, Up lands on the second row. The Summary has no rows of its own
    // that the count of palette rows would mistake for its own.
    harness.key_press(Key::Num1);
    harness.run();
    open_palette(&mut harness);
    type_query(&mut harness, "in");
    assert_eq!(
        palette_rows(&harness)[..3],
        ["Inventory", "Billing", "Shipping"]
    );
    harness.key_press(Key::ArrowDown);
    harness.key_press(Key::ArrowDown);
    harness.key_press(Key::ArrowUp);
    harness.run_steps(2);
    harness.key_press(Key::Enter);
    harness.run_steps(3);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Billing".into())
    );
}

#[test]
fn nothing_found_says_what_to_try_and_esc_closes() {
    let (_desk, mut harness) = shop_api();
    open_palette(&mut harness);
    type_query(&mut harness, "zzq");
    harness.get_by_label_contains("Nothing matches \"zzq\".");
    assert!(palette_rows(&harness).is_empty());

    harness.key_press(Key::Escape);
    harness.run_steps(3);
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Search or run a command")
            .is_none()
    );
    // While it was open, the keys of the workbench did nothing.
    assert_eq!(harness.state().selection(), Selection::All);
}

#[test]
fn tab_searches_in_the_next_app_and_a_row_of_it_opens_that_app() {
    let desk = Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
    let mut harness = desk.start();

    // From Settings: a command that goes somewhere leaves them.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    open_palette(&mut harness);
    harness.key_press(Key::Tab);
    harness.run_steps(3);
    type_query(&mut harness, "users");
    harness.get_by_label("Modules in crm");
    harness.key_press(Key::Enter);
    wait_until(&mut harness, "crm is on screen", |app| {
        app.current() == Some(1)
    });
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Users".into())
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Settings")
            .is_none()
    );
}

#[test]
fn the_entry_in_the_title_bar_opens_the_palette_and_a_click_runs_a_row() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::Button, "Search or run a command")
        .click();
    harness.run_steps(3);
    type_query(&mut harness, "ship");
    harness
        .get_all_by_role_and_label(Role::ListBoxOption, "Shipping")
        .last()
        .unwrap()
        .click();
    harness.run_steps(3);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Shipping".into())
    );

    // "php" finds Settings on the PHP section.
    open_palette(&mut harness);
    type_query(&mut harness, "php");
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Settings: PHP")
        .click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "PHP");
}

#[test]
fn a_generator_row_opens_its_form_and_runs_nothing() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run_steps(3);
    open_palette(&mut harness);
    type_query(&mut harness, "model");
    harness.get_by_label("Generators");
    let row = "Generate a model in Checkout";
    assert!(palette_rows(&harness).iter().any(|label| label == row));
    harness
        .get_by_role_and_label(Role::ListBoxOption, row)
        .click();
    harness.run_steps(3);
    assert_eq!(harness.state().tab(), Tab::Generate);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
    harness.get_by_role_and_label(Role::Heading, "Generate in Checkout");
    assert!(
        harness
            .get_by_role_and_label(Role::TextInput, "Model name")
            .is_focused()
    );
}

#[test]
fn a_recent_generator_without_a_module_argument_opens_under_all_modules() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    common::package_version(&root, "v13.4.0");
    desk.knows(&[(&root, "SA")]);
    // An app's own generator, read from the app, that takes no module.
    let mut catalog = Catalog::v13_4().clone();
    let mut widget = catalog.get(MODULE_MAKE).unwrap().clone();
    widget.name = "app:make-widget".to_owned();
    catalog.generators.push(widget);
    let version = Version {
        raw: "v13.4.0".to_owned(),
        alias: None,
    };
    catalog_cache::store(&desk.data_dir(), &root, &version, &catalog, 0).unwrap();
    common::ran(&desk, &root, &[("app:make-widget Widget", 2)]);
    let mut harness = desk.start();

    open_palette(&mut harness);
    harness
        .get_by_role_and_label(Role::ListBoxOption, "app:make-widget")
        .click();
    harness.run_steps(3);
    assert_eq!(harness.state().selection(), Selection::All);
    harness.get_by_role_and_label(Role::Heading, "Generate in shop-api");
    harness.get_by_role_and_label(Role::TextInput, "app:make-widget name");
}
