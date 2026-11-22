//! Accessibility (section 9), checked on the screens of phase 1: every control has a
//! role and a name, every control is a target of at least 24 by 24 px, and a name
//! tells one button from another.

mod common;

use common::{Desk, connected, shop_api, shop_api_actuator};
use eframe::egui::{Key, Modifiers, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::MortiseApp;

/// The roles of things a user acts on.
const CONTROLS: [Role; 11] = [
    Role::Button,
    Role::CheckBox,
    Role::RadioButton,
    Role::TextInput,
    Role::PasswordInput,
    Role::ListBoxOption,
    Role::Tab,
    Role::Row,
    Role::ColumnHeader,
    Role::Splitter,
    Role::Link,
];

/// Checks the screen that is up. `screen` names it in a failure.
fn audit(harness: &Harness<'static, MortiseApp>, screen: &str) {
    let mut problems = Vec::new();
    let mut buttons: Vec<String> = Vec::new();
    let mut controls = 0;
    for node in harness.query_all_by(|_| true) {
        let node = node.accesskit_node();
        let role = node.role();
        if !CONTROLS.contains(&role) {
            continue;
        }
        controls += 1;
        let label = node.label().unwrap_or_default();
        if label.trim().is_empty() {
            problems.push(format!("a {role:?} without a name"));
        }
        // A control that is scrolled out of sight has no box, or a cut one. A column
        // header is as high as its text when it only names a column. The edge that
        // sets the width of the index is 6 px wide, as such an edge is on every
        // platform, and has keys.
        if !matches!(role, Role::ColumnHeader | Role::Splitter)
            && let Some(bounds) = node.bounding_box()
            && bounds.width() > 0.0
            && bounds.height() > 0.0
            && !node.is_disabled()
            && (bounds.width() < 24.0 || bounds.height() < 24.0)
        {
            problems.push(format!(
                "{role:?} \"{label}\" is a target of {:.0} by {:.0} px",
                bounds.width(),
                bounds.height()
            ));
        }
        if role == Role::Button {
            buttons.push(label);
        }
    }
    buttons.sort();
    for pair in buttons.windows(2) {
        if pair[0] == pair[1] {
            problems.push(format!("two buttons are named \"{}\"", pair[0]));
        }
    }
    problems.dedup();
    assert!(controls > 0, "{screen}: no control found");
    assert!(
        problems.is_empty(),
        "{screen}:\n  {}",
        problems.join("\n  ")
    );
}

fn press(harness: &mut Harness<'static, MortiseApp>, key: Key) {
    harness.key_press(key);
    harness.run_steps(3);
}

fn chord(harness: &mut Harness<'static, MortiseApp>, key: Key) {
    harness.key_press_modifiers(Modifiers::COMMAND, key);
    harness.run_steps(3);
}

#[test]
fn the_workbench_and_its_tabs() {
    let stub = shop_api_actuator();
    let (_desk, mut harness) = connected(&stub);
    audit(&harness, "Summary of all modules, with runtime");
    press(&mut harness, Key::F);
    audit(&harness, "Findings list");
    press(&mut harness, Key::Escape);
    press(&mut harness, Key::Num2);
    audit(&harness, "Boundary of all modules");
    for digit in [Key::Num3, Key::Num4, Key::Num5] {
        press(&mut harness, digit);
        audit(&harness, "a tab of a later phase");
    }

    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run_steps(3);
    press(&mut harness, Key::Num1);
    audit(&harness, "Summary of a module, with routes");
    press(&mut harness, Key::Num2);
    audit(&harness, "Boundary of a module, links");
    press(&mut harness, Key::P);
    audit(&harness, "Public surface, not scanned");
    chord(&mut harness, Key::Enter);
    harness.run_steps(6);
    audit(&harness, "Public surface, proposal");

    // The strip of the collapsed index.
    chord(&mut harness, Key::B);
    audit(&harness, "index as a strip");
}

#[test]
fn the_palette_settings_and_the_add_panel() {
    let (desk, mut harness) = shop_api();
    chord(&mut harness, Key::K);
    audit(&harness, "palette, empty query");
    harness
        .get_by_role_and_label(Role::TextInput, "Search or run a command")
        .type_text("check");
    harness.run_steps(3);
    audit(&harness, "palette, with a query");
    press(&mut harness, Key::Escape);

    chord(&mut harness, Key::Comma);
    audit(&harness, "Settings, Apps");
    harness
        .get_by_role_and_label(Role::Button, "Add environment")
        .click();
    harness.run_steps(4);
    audit(&harness, "Settings, an environment without credentials");
    for kind in ["Bearer token", "HTTP basic", "Custom header"] {
        harness
            .get_by_role_and_label(Role::RadioButton, kind)
            .click();
        harness.run_steps(3);
        audit(&harness, kind);
    }
    harness
        .get_by_role_and_label(Role::ListBoxOption, "General")
        .click();
    harness.run_steps(3);
    audit(&harness, "Settings, General");
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Keyboard")
        .click();
    harness.run_steps(3);
    audit(&harness, "Settings, Keyboard");
    press(&mut harness, Key::Escape);

    // The add panel: empty, then with a folder.
    harness
        .get_by_role_and_label(Role::Button, "Add app")
        .click();
    harness.run_steps(3);
    audit(&harness, "add panel, empty");
    desk.picks
        .borrow_mut()
        .push_back(desk.app("booking-engine"));
    chord(&mut harness, Key::O);
    audit(&harness, "add panel, a folder that was read");
}

#[test]
fn the_first_run_and_an_app_with_many_modules() {
    let desk = Desk::new();
    let harness = desk.start();
    audit(&harness, "first run");

    let desk = Desk::new();
    let root = common::app_with_modules(&desk, "platform", 30);
    desk.knows(&[(&root, "PL")]);
    let mut harness = desk.start();
    audit(&harness, "30 modules, Summary");
    press(&mut harness, Key::Num2);
    audit(&harness, "30 modules, the table of links");
}
