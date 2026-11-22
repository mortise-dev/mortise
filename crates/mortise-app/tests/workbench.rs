//! The workbench, run headless: what is on screen, and what the keys and the pointer do.

mod common;

use common::{Desk, shop_api};
use eframe::egui::{Key, Role};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::Selection;
use mortise_app::chrome::Chrome;
use mortise_app::workspace::Tab;

#[test]
fn shows_the_app_and_its_modules() {
    let (_desk, harness) = shop_api();

    harness.get_by_role_and_label(Role::Heading, "shop-api");
    harness.get_by_label("7 of 7 on");
    harness.get_by_label("Laravel 13.2, package 13.3.0");
    harness.get_by_label("Sorted: by name");

    // The tile on the rail, then "All modules", then the modules by name.
    let rows: Vec<_> = harness
        .get_all_by_role(Role::ListBoxOption)
        .filter_map(|row| row.accesskit_node().label())
        .collect();
    assert_eq!(
        rows,
        [
            "shop-api",
            "All modules",
            "Billing",
            "Catalog",
            "Checkout",
            "Inventory",
            "Notifications",
            "Shipping",
            "Users"
        ]
    );

    // The home screen: "All modules", Summary.
    assert_eq!(harness.state().selection(), Selection::All);
    assert_eq!(harness.state().tab(), Tab::Summary);
    harness.get_by_role_and_label(Role::Heading, "All modules");
    harness.get_by_label("Modules on");
    harness.get_by_label("7 of 7");
}

#[test]
fn j_and_k_move_through_the_modules_and_the_workspace_follows() {
    let (_desk, mut harness) = shop_api();
    let module = |name: &str| Selection::Module(name.into());

    harness.key_press(Key::J);
    harness.run();
    assert_eq!(harness.state().selection(), module("Billing"));
    harness.get_by_role_and_label(Role::Heading, "Billing");
    harness.get_by_label("Modules/Billing");
    harness.get_by_label("Module state");
    harness.get_by_label("On");

    harness.key_press(Key::ArrowDown);
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(harness.state().selection(), module("Checkout"));

    harness.key_press(Key::K);
    harness.run();
    assert_eq!(harness.state().selection(), module("Catalog"));

    harness.key_press(Key::End);
    harness.run();
    assert_eq!(harness.state().selection(), module("Users"));
    // The last row stays the last row.
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(harness.state().selection(), module("Users"));

    harness.key_press(Key::Home);
    harness.run();
    assert_eq!(harness.state().selection(), Selection::All);
    harness.key_press(Key::ArrowUp);
    harness.run();
    assert_eq!(harness.state().selection(), Selection::All);
}

#[test]
fn the_digits_open_the_tabs_and_the_tab_is_kept_while_moving() {
    let (_desk, mut harness) = shop_api();

    for (key, tab) in [
        (Key::Num2, Tab::Boundary),
        (Key::Num3, Tab::Generate),
        (Key::Num4, Tab::Actions),
        (Key::Num5, Tab::Metrics),
        (Key::Num1, Tab::Summary),
    ] {
        harness.key_press(key);
        harness.run();
        assert_eq!(harness.state().tab(), tab);
    }

    harness.key_press(Key::Num3);
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(harness.state().tab(), Tab::Generate);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Billing".into())
    );
}

#[test]
fn a_tab_of_a_later_phase_explains_itself() {
    let (_desk, mut harness) = shop_api();
    for (tab, title) in [
        ("Generate", "Generate is not in this version"),
        ("Actions", "Actions are not in this version"),
        ("Metrics", "Metrics are not in this version"),
    ] {
        harness.get_by_role_and_label(Role::Tab, tab).click();
        harness.run();
        harness.get_by_role_and_label(Role::Heading, title);
    }
    // Every tab is always there, and none is disabled.
    assert_eq!(harness.get_all_by_role(Role::Tab).count(), 5);
}

#[test]
fn a_click_selects_a_module() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Shipping")
        .click();
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Shipping".into())
    );
    harness.get_by_role_and_label(Role::Heading, "Shipping");
}

#[test]
fn a_module_that_is_turned_off_says_so() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    common::turn_off(&root, "Notifications");
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();

    harness.get_by_label("6 of 7 on");
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Notifications")
        .click();
    harness.run();
    harness.get_by_label("Module state");
    harness.get_by_label("Off");
}

#[test]
fn the_window_controls_are_drawn_only_where_mortise_draws_the_frame() {
    let (_desk, custom) = shop_api();
    for name in ["Minimise", "Maximise", "Close"] {
        custom.get_by_role_and_label(Role::Button, name);
    }

    let mut desk = Desk::new();
    desk.chrome = Chrome::Mac;
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mac = desk.start();
    for name in ["Minimise", "Maximise", "Close"] {
        assert!(
            mac.query_by_role_and_label(Role::Button, name).is_none(),
            "{name}"
        );
    }
}

#[test]
fn an_app_whose_folder_is_gone_says_why() {
    let desk = Desk::new();
    let root = desk.path().join("shop-api");
    desk.knows(&[(&root, "SA")]);
    let harness = desk.start();
    harness.get_by_role_and_label(Role::Heading, "shop-api cannot be read");
    harness.get_by_label_contains("The folder has no artisan file");
}
