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
    harness.get_by_role_and_label(Role::Tab, "Metrics").click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Metrics are not in this version");
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

#[test]
fn the_index_says_how_much_of_each_module_is_covered() {
    let (_desk, harness) = shop_api();
    // The second line of a row is part of the row, so it is read from the summary instead.
    harness.get_by_label("Lines of PHP");
    harness.get_by_label("861");
    harness.get_by_label("Test coverage");
    harness.get_by_label("68%");
    harness.get_by_role_and_label(Role::Heading, "Coverage by module");
    harness.get_by_label("18 tests");
    harness.get_by_label("88%");
    harness.get_by_label("39%");
}

#[test]
fn the_table_of_modules_sorts_and_a_row_selects_its_module() {
    let (_desk, mut harness) = shop_api();
    let order =
        |harness: &egui_kittest::Harness<'static, mortise_app::app::MortiseApp>| -> Vec<String> {
            harness
                .get_all_by_role(Role::Row)
                .filter_map(|row| row.accesskit_node().label())
                .collect()
        };
    harness.get_by_label("sorted by name");
    assert_eq!(order(&harness)[0], "Billing");

    // A number column starts with the highest.
    harness
        .get_by_role_and_label(Role::ColumnHeader, "Coverage")
        .click();
    harness.run();
    harness.get_by_label("sorted by coverage, highest first");
    assert_eq!(
        order(&harness),
        [
            "Users",
            "Billing",
            "Catalog",
            "Inventory",
            "Shipping",
            "Checkout",
            "Notifications"
        ]
    );
    // Again reverses.
    harness
        .get_by_role_and_label(Role::ColumnHeader, "Coverage")
        .click();
    harness.run();
    assert_eq!(order(&harness)[0], "Notifications");

    harness
        .get_by_role_and_label(Role::ColumnHeader, "Lines")
        .click();
    harness.run();
    assert_eq!(order(&harness)[0], "Checkout");

    harness.get_by_role_and_label(Role::Row, "Checkout").click();
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
    harness.get_by_role_and_label(Role::Heading, "Files by kind");
    harness.get_by_label("20 files, 212 lines");
    harness.get_by_label("Listeners");
    harness.get_by_label("Git churn");
    harness.get_by_label("not a repository");
    harness.get_by_label("54%");
}

#[test]
fn the_filter_narrows_the_index_and_esc_clears_it_then_leaves() {
    let (_desk, mut harness) = shop_api();
    let rows =
        |harness: &egui_kittest::Harness<'static, mortise_app::app::MortiseApp>| -> Vec<String> {
            harness
                .get_all_by_role(Role::ListBoxOption)
                .filter_map(|row| row.accesskit_node().label())
                .skip(1) // the tile on the rail
                .collect()
        };

    // The slash moves focus into the filter.
    harness.key_press(Key::Slash);
    harness.run();
    let field = harness.get_by_role_and_label(Role::TextInput, "Filter modules");
    assert!(field.is_focused());

    // Letters match the name or the code; "All modules" stays; the first match is selected.
    field.type_text("ch");
    harness.run();
    assert_eq!(rows(&harness), ["All modules", "Checkout"]);
    harness.get_by_label("1 of 7 modules");
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );

    // While typing, single keys are letters, not commands.
    assert_eq!(harness.state().tab(), Tab::Summary);
    harness
        .get_by_role_and_label(Role::TextInput, "Filter modules")
        .type_text("1");
    harness.run();
    assert_eq!(harness.state().tab(), Tab::Summary);
    harness.get_by_label("No module matches \"ch1\". Press Esc to clear the filter.");
    harness.get_by_label("0 of 7 modules");

    // Esc clears the text first and leaves the field second.
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(rows(&harness).len(), 8);
    assert!(
        harness
            .get_by_role_and_label(Role::TextInput, "Filter modules")
            .is_focused()
    );
    harness.key_press(Key::Escape);
    harness.run();
    assert!(
        !harness
            .get_by_role_and_label(Role::TextInput, "Filter modules")
            .is_focused()
    );

    // The code matches too, and J and K move through what the filter leaves.
    harness.key_press_modifiers(eframe::egui::Modifiers::COMMAND, Key::F);
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Filter modules")
        .type_text("in");
    harness.run();
    assert_eq!(
        rows(&harness),
        ["All modules", "Billing", "Inventory", "Shipping"]
    );
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Billing".into())
    );
    harness.key_press(Key::J);
    harness.key_press(Key::J);
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Shipping".into())
    );
}

#[test]
fn ctrl_b_collapses_the_index_to_its_strip_and_opens_it_again() {
    let (_desk, mut harness) = shop_api();
    harness.get_by_label("Sorted: by name");

    harness.key_press_modifiers(eframe::egui::Modifiers::COMMAND, Key::B);
    harness.run();
    assert!(harness.query_by_label("Sorted: by name").is_none());
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Filter modules")
            .is_none()
    );
    // The rows are still there, and J and K still work.
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Billing".into())
    );
    harness.get_by_role_and_label(Role::ListBoxOption, "Checkout");

    harness.key_press_modifiers(eframe::egui::Modifiers::COMMAND, Key::B);
    harness.run();
    harness.get_by_label("Sorted: by name");
}
