//! The keyboard model (section 7, K1): the Tab order, the regions, and the keys
//! inside a list, a table, the tab row and a segmented control.

mod common;

use common::shop_api;
use eframe::egui::{Key, Modifiers, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::MortiseApp;
use mortise_app::index::view::Selection;
use mortise_app::workspace::Tab;

/// The role and the name of the control that holds focus.
fn focused(harness: &Harness<'static, MortiseApp>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

fn press(harness: &mut Harness<'static, MortiseApp>, key: Key) {
    harness.key_press(key);
    harness.run_steps(3);
}

/// Presses Tab until focus is back where it started, and gives every stop on the way.
fn tab_stops(harness: &mut Harness<'static, MortiseApp>) -> Vec<(Role, String)> {
    let mut stops: Vec<(Role, String)> = Vec::new();
    for _ in 0..80 {
        press(harness, Key::Tab);
        let Some(stop) = focused(harness) else {
            continue;
        };
        if stops.first() == Some(&stop) {
            return stops;
        }
        stops.push(stop);
    }
    panic!("Tab did not come round: {stops:?}");
}

fn place(stops: &[(Role, String)], role: Role, label: &str) -> usize {
    stops
        .iter()
        .position(|(r, l)| *r == role && l == label)
        .unwrap_or_else(|| panic!("no stop {role:?} {label}: {stops:?}"))
}

#[test]
fn tab_goes_from_the_rail_to_the_index_to_the_workspace() {
    let (_desk, mut harness) = shop_api();
    let stops = tab_stops(&mut harness);

    // The rail, then the index, then the workspace.
    let tile = place(&stops, Role::ListBoxOption, "shop-api");
    let filter = place(&stops, Role::TextInput, "Filter modules");
    let row = place(&stops, Role::ListBoxOption, "All modules");
    let tab = place(&stops, Role::Tab, "Summary");
    let button = place(&stops, Role::Button, "Show all 3 findings");
    assert!(
        tile < filter && filter < row && row < tab && tab < button,
        "{stops:?}"
    );

    // The index is one stop: no other row of it is reached.
    for module in ["Billing", "Catalog", "Checkout", "Users"] {
        assert!(
            !stops.contains(&(Role::ListBoxOption, module.to_owned())),
            "{stops:?}"
        );
    }
    // The tab row is one stop.
    assert_eq!(
        stops.iter().filter(|(role, _)| *role == Role::Tab).count(),
        1
    );
    // The table of modules is one stop.
    assert_eq!(
        stops.iter().filter(|(role, _)| *role == Role::Row).count(),
        1
    );
    // Window controls drawn by Mortise are not in the Tab order.
    assert!(
        !stops
            .iter()
            .any(|(_, label)| label == "Close" || label == "Minimise")
    );
}

#[test]
fn at_start_the_index_has_the_keys_without_a_focused_control() {
    let (_desk, mut harness) = shop_api();
    assert!(focused(&harness).is_none_or(|(role, _)| role == Role::Window));
    press(&mut harness, Key::J);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Billing".into())
    );
    press(&mut harness, Key::ArrowDown);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Catalog".into())
    );
    press(&mut harness, Key::End);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Users".into())
    );
    press(&mut harness, Key::Home);
    assert_eq!(harness.state().selection(), Selection::All);
}

#[test]
fn enter_or_right_moves_focus_into_the_workspace_and_esc_brings_it_back() {
    let (_desk, mut harness) = shop_api();
    // Selecting a module with J leaves focus in the index, and so does a digit.
    press(&mut harness, Key::J);
    press(&mut harness, Key::Num2);
    assert!(focused(&harness).is_none_or(|(role, _)| role == Role::Window));
    press(&mut harness, Key::Enter);
    harness.run_steps(3);
    // The first control of the tab, not the tab row and not the button of the header.
    assert_eq!(
        focused(&harness),
        Some((Role::RadioButton, "Links".to_owned()))
    );
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Billing".into())
    );

    // Esc brings focus back to the index: J moves to the next module again.
    press(&mut harness, Key::Escape);
    assert!(focused(&harness).is_none_or(|(role, _)| role == Role::Window));
    press(&mut harness, Key::J);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Catalog".into())
    );

    // Right does the same as Enter. On "All modules" the first control is the button
    // that shows the findings.
    press(&mut harness, Key::Num1);
    press(&mut harness, Key::Home);
    press(&mut harness, Key::ArrowRight);
    harness.run_steps(3);
    assert_eq!(
        focused(&harness),
        Some((Role::Button, "Show all 3 findings".to_owned()))
    );
    // Enter presses it.
    press(&mut harness, Key::Enter);
    assert!(harness.state().findings_open());
}

#[test]
fn f6_moves_between_the_rail_the_index_and_the_workspace() {
    let (_desk, mut harness) = shop_api();
    // From the index to the workspace: the tab row.
    press(&mut harness, Key::F6);
    assert_eq!(focused(&harness), Some((Role::Tab, "Summary".to_owned())));
    // To the rail: the tile of the app.
    press(&mut harness, Key::F6);
    assert_eq!(
        focused(&harness),
        Some((Role::ListBoxOption, "shop-api".to_owned()))
    );
    // To the index: its row has the keys again.
    press(&mut harness, Key::F6);
    press(&mut harness, Key::J);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Billing".into())
    );

    // Shift F6 goes the other way: from the index to the rail.
    harness.key_press_modifiers(Modifiers::SHIFT, Key::F6);
    harness.run_steps(3);
    assert_eq!(
        focused(&harness),
        Some((Role::ListBoxOption, "shop-api".to_owned()))
    );
    harness.key_press_modifiers(Modifiers::SHIFT, Key::F6);
    harness.run_steps(3);
    assert_eq!(focused(&harness), Some((Role::Tab, "Summary".to_owned())));
}

#[test]
fn left_and_right_change_the_tab_and_a_segmented_control() {
    let (_desk, mut harness) = shop_api();
    press(&mut harness, Key::J);
    press(&mut harness, Key::F6);
    assert_eq!(focused(&harness), Some((Role::Tab, "Summary".to_owned())));
    press(&mut harness, Key::ArrowRight);
    assert_eq!(harness.state().tab(), Tab::Boundary);
    // Focus goes with the tab.
    assert_eq!(focused(&harness), Some((Role::Tab, "Boundary".to_owned())));
    // The module does not change: the arrows belong to the tab row.
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Billing".into())
    );

    // The next stop is the control that switches the two views of the module.
    press(&mut harness, Key::Tab);
    assert_eq!(
        focused(&harness),
        Some((Role::RadioButton, "Links".to_owned()))
    );
    press(&mut harness, Key::ArrowRight);
    assert_eq!(
        focused(&harness),
        Some((Role::RadioButton, "Public surface".to_owned()))
    );
    harness.get_by_role_and_label(Role::Heading, "Billing has no public surface yet");
    press(&mut harness, Key::ArrowLeft);
    assert_eq!(
        focused(&harness),
        Some((Role::RadioButton, "Links".to_owned()))
    );

    press(&mut harness, Key::F6);
    press(&mut harness, Key::F6);
    press(&mut harness, Key::F6);
    assert_eq!(focused(&harness), Some((Role::Tab, "Boundary".to_owned())));
    press(&mut harness, Key::ArrowLeft);
    assert_eq!(harness.state().tab(), Tab::Summary);
}

#[test]
fn j_and_k_move_in_a_table_that_has_focus_and_enter_opens_the_row() {
    let (_desk, mut harness) = shop_api();
    // Tab to the table of modules.
    for _ in 0..80 {
        press(&mut harness, Key::Tab);
        if focused(&harness).is_some_and(|(role, _)| role == Role::Row) {
            break;
        }
    }
    assert_eq!(focused(&harness), Some((Role::Row, "Billing".to_owned())));

    // J and K, Down and Up move in the table. The index does not move.
    press(&mut harness, Key::J);
    assert_eq!(focused(&harness), Some((Role::Row, "Catalog".to_owned())));
    press(&mut harness, Key::ArrowDown);
    assert_eq!(focused(&harness), Some((Role::Row, "Checkout".to_owned())));
    press(&mut harness, Key::K);
    assert_eq!(focused(&harness), Some((Role::Row, "Catalog".to_owned())));
    press(&mut harness, Key::End);
    assert_eq!(focused(&harness), Some((Role::Row, "Users".to_owned())));
    press(&mut harness, Key::Home);
    assert_eq!(focused(&harness), Some((Role::Row, "Billing".to_owned())));
    assert_eq!(harness.state().selection(), Selection::All);

    // Enter selects that module in the index.
    press(&mut harness, Key::J);
    press(&mut harness, Key::Enter);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Catalog".into())
    );
}

#[test]
fn the_findings_list_takes_the_keys_when_it_opens() {
    let (_desk, mut harness) = shop_api();
    press(&mut harness, Key::F);
    assert!(harness.state().findings_open());
    // Focus is on the first finding. J moves in the list, not in the index.
    let first = focused(&harness).expect("a finding has focus");
    assert_eq!(first.0, Role::Row);
    press(&mut harness, Key::J);
    let second = focused(&harness).expect("a finding has focus");
    assert_eq!(second.0, Role::Row);
    assert_ne!(first.1, second.1);
    assert_eq!(harness.state().selection(), Selection::All);

    // Enter opens the finding where it lives.
    press(&mut harness, Key::K);
    press(&mut harness, Key::Enter);
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
    assert_eq!(harness.state().tab(), Tab::Boundary);
}

#[test]
fn the_arrows_move_between_the_modules_of_the_graph() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run_steps(2);
    press(&mut harness, Key::Num2);
    // Tab to the first module of the graph.
    for _ in 0..80 {
        press(&mut harness, Key::Tab);
        if focused(&harness).is_some_and(|(role, label)| {
            role == Role::ListBoxOption && label != "shop-api" && label != "Checkout"
        }) {
            break;
        }
    }
    let start = focused(&harness).expect("a module of the graph has focus");
    // Down goes to the module under it, Up comes back. The index does not move.
    press(&mut harness, Key::ArrowDown);
    let below = focused(&harness).expect("a module of the graph has focus");
    assert_ne!(start, below, "{start:?}");
    press(&mut harness, Key::ArrowUp);
    assert_eq!(focused(&harness), Some(start));
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
}

#[test]
fn the_edge_of_the_index_sets_its_width_with_the_arrows() {
    let (desk, mut harness) = shop_api();
    // The edge is a stop between the index and the workspace.
    let stops = tab_stops(&mut harness);
    let edge = place(&stops, Role::Splitter, "Width of the module index");
    assert!(place(&stops, Role::ListBoxOption, "All modules") < edge);
    assert!(edge < place(&stops, Role::Tab, "Summary"));

    for _ in 0..80 {
        press(&mut harness, Key::Tab);
        if focused(&harness).is_some_and(|(role, _)| role == Role::Splitter) {
            break;
        }
    }
    assert_eq!(desk.saved().general.index_width, 264);
    press(&mut harness, Key::ArrowRight);
    assert_eq!(desk.saved().general.index_width, 272);
    // The width stops at 320 and at 220.
    for _ in 0..12 {
        press(&mut harness, Key::ArrowRight);
    }
    assert_eq!(desk.saved().general.index_width, 320);
    for _ in 0..20 {
        press(&mut harness, Key::ArrowLeft);
    }
    assert_eq!(desk.saved().general.index_width, 220);
    // The module does not change: the arrows belong to the edge.
    assert_eq!(harness.state().selection(), Selection::All);

    // The next start has the same width.
    let harness = desk.start();
    let row = harness.get_by_role_and_label(Role::ListBoxOption, "All modules");
    assert!(row.rect().right() < 52.0 + 221.0, "{:?}", row.rect());
}

#[test]
fn alt_with_up_or_down_moves_an_app_on_the_rail() {
    let desk = common::Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    let pages = desk.app("landing-pages");
    desk.knows(&[(&shop, "SA"), (&crm, "CR"), (&pages, "LP")]);
    let mut harness = desk.start();
    assert_eq!(
        harness.state().app_names(),
        ["shop-api", "crm", "landing-pages"]
    );

    // To the rail: the tile of the app on screen.
    harness.key_press_modifiers(Modifiers::SHIFT, Key::F6);
    harness.run_steps(3);
    assert_eq!(
        focused(&harness),
        Some((Role::ListBoxOption, "shop-api".to_owned()))
    );

    harness.key_press_modifiers(Modifiers::ALT, Key::ArrowDown);
    harness.run_steps(3);
    assert_eq!(
        harness.state().app_names(),
        ["crm", "shop-api", "landing-pages"]
    );
    // The app on screen stays on screen, and focus stays on its tile.
    assert_eq!(harness.state().current(), Some(1));
    assert_eq!(
        focused(&harness),
        Some((Role::ListBoxOption, "shop-api".to_owned()))
    );
    harness.get_by_role_and_label(Role::Heading, "shop-api");

    harness.key_press_modifiers(Modifiers::ALT, Key::ArrowDown);
    harness.run_steps(3);
    // At the end of the rail nothing more happens.
    harness.key_press_modifiers(Modifiers::ALT, Key::ArrowDown);
    harness.run_steps(3);
    assert_eq!(
        harness.state().app_names(),
        ["crm", "landing-pages", "shop-api"]
    );
    assert_eq!(harness.state().current(), Some(2));

    // The order is kept.
    let saved = desk.saved();
    let names: Vec<&str> = saved.apps.iter().map(|app| app.name.as_str()).collect();
    assert_eq!(names, ["crm", "landing-pages", "shop-api"]);
    assert_eq!(saved.current, Some(2));

    harness.key_press_modifiers(Modifiers::ALT, Key::ArrowUp);
    harness.run_steps(3);
    assert_eq!(
        harness.state().app_names(),
        ["crm", "shop-api", "landing-pages"]
    );
}

#[test]
fn a_tooltip_shows_the_shortcut_of_its_control() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::Button, "Settings")
        .hover();
    harness.run_steps(12);
    // The name of the control and its shortcut, both in the tooltip.
    assert!(harness.get_all_by_label("Settings").count() >= 2);
    harness.get_by_label("Ctrl ,");
}
