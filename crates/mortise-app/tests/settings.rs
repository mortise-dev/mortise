//! Settings, the editor, and how the window gives width to the workspace.

mod common;

use common::{Desk, shop_api, wait_until};
use eframe::egui::{self, Key, Modifiers, Role, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::{MortiseApp, Selection};
use mortise_core::{Editor, IndexRows, ThemeChoice};

fn open_settings(harness: &mut Harness<'static, MortiseApp>) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
}

#[test]
fn settings_take_the_place_of_the_workbench_and_esc_returns() {
    let (_desk, mut harness) = shop_api();
    open_settings(&mut harness);
    harness.get_by_role_and_label(Role::Heading, "Settings");
    harness.get_by_label("Kept on this computer");
    harness.get_by_role_and_label(Role::Heading, "Apps");
    harness.get_by_role_and_label(Role::Button, "Back to shop-api");
    // The keys of the workbench rest.
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(harness.state().selection(), Selection::All);

    harness
        .get_by_role_and_label(Role::ListBoxOption, "General")
        .click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "General");
    harness.get_by_label("every size in the interface scales with it");

    harness.key_press(Key::Escape);
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "All modules");
}

#[test]
fn a_choice_in_general_applies_at_once_and_is_kept() {
    let (desk, mut harness) = shop_api();
    open_settings(&mut harness);
    harness
        .get_by_role_and_label(Role::ListBoxOption, "General")
        .click();
    harness.run();

    harness
        .get_by_role_and_label(Role::RadioButton, "Dark")
        .click();
    harness.run();
    assert_eq!(harness.ctx.theme(), Theme::Dark);
    harness
        .get_by_role_and_label(Role::RadioButton, "125%")
        .click();
    harness.run();
    assert!((harness.ctx.zoom_factor() - 1.25).abs() < 0.001);
    harness
        .get_by_role_and_label(Role::RadioButton, "One line")
        .click();
    harness
        .get_by_role_and_label(Role::RadioButton, "Zed")
        .click();
    harness.run();
    harness.get_by_label("zed {file}:{line}");

    let general = desk.saved().general;
    assert_eq!(general.theme, ThemeChoice::Dark);
    assert_eq!(general.text_size, 125);
    assert_eq!(general.index_rows, IndexRows::OneLine);
    assert_eq!(general.editor, Editor::Zed);

    // The next start has the same choices.
    let harness = desk.start();
    assert_eq!(harness.ctx.theme(), Theme::Dark);
    assert!((harness.ctx.zoom_factor() - 1.25).abs() < 0.001);
}

#[test]
fn the_fields_of_an_app_apply_when_they_lose_focus() {
    let (desk, mut harness) = shop_api();
    open_settings(&mut harness);
    harness.get_by_label("Laravel 13.2, package 13.3.0");

    // A lower coverage limit: Notifications, at 39%, is no longer under it.
    assert_eq!(harness.state().app().unwrap().findings.warnings(), 1);
    let limit = harness.get_by_role_and_label(Role::TextInput, "Coverage limit");
    limit.focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Coverage limit")
        .type_text("30");
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Name")
        .focus();
    wait_until(&mut harness, "the limit is 30", |app| {
        app.app().is_some_and(|app| app.coverage_limit == 30)
    });
    assert_eq!(harness.state().app().unwrap().findings.warnings(), 0);
    assert_eq!(desk.saved().apps[0].coverage_limit, 30);

    // The monogram: two letters, shown on the rail.
    let monogram = harness.get_by_role_and_label(Role::TextInput, "Monogram");
    monogram.focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Monogram")
        .type_text("sh");
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Name")
        .focus();
    harness.run();
    assert_eq!(desk.saved().apps[0].monogram, "SH");
}

#[test]
fn removing_an_app_asks_first_and_leaves_its_folder() {
    let desk = Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
    let mut harness = desk.start();
    open_settings(&mut harness);

    harness
        .get_by_role_and_label(Role::Button, "Remove shop-api from Mortise")
        .click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Remove shop-api from Mortise?");
    harness.get_by_label_contains("The folder and its files stay as they are.");
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().app_names(), ["shop-api", "crm"]);

    harness
        .get_by_role_and_label(Role::Button, "Remove shop-api from Mortise")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Remove app")
        .click();
    wait_until(&mut harness, "crm is the only app", |app| {
        app.app_names() == ["crm"]
    });
    assert_eq!(harness.state().current(), Some(0));
    assert_eq!(desk.saved().apps.len(), 1);
    assert!(shop.join("artisan").is_file());
}

#[test]
fn the_question_mark_shows_the_shortcuts_and_they_can_be_searched() {
    let (_desk, mut harness) = shop_api();
    harness.key_press(Key::Questionmark);
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Keyboard");
    harness.get_by_label("Open the command palette");
    harness.get_by_label("18 of 18 shown.");

    harness
        .get_by_role_and_label(Role::TextInput, "Find a shortcut")
        .focus();
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Find a shortcut")
        .type_text("palette");
    harness.run();
    harness.get_by_label("3 of 18 shown.");
    assert!(
        harness
            .query_by_label("Change the order of the index")
            .is_none()
    );
}

#[test]
fn the_settings_tile_on_the_rail_opens_and_closes_them() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::Button, "Settings")
        .click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Apps");
    harness
        .get_by_role_and_label(Role::Button, "Settings")
        .click();
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "All modules");
}

#[test]
fn an_editor_that_does_not_start_says_so_and_where_to_change_it() {
    let (desk, _) = shop_api();
    let mut settings = desk.saved();
    settings.general.editor = Editor::Custom;
    settings.general.editor_command = "mortise-no-such-editor {file}:{line}".to_owned();
    settings.save(&desk.settings_path()).unwrap();
    let mut harness = desk.start();

    harness.key_press(Key::J);
    harness.key_press(Key::E);
    harness.run_steps(3);
    harness.get_by_label_contains("Could not open the editor: \"mortise-no-such-editor: ");
    harness.get_by_label_contains("Choose the editor command in Settings.");
}

#[cfg(unix)]
#[test]
fn an_editor_that_starts_is_given_the_file_and_the_line() {
    let (desk, _) = shop_api();
    let out = desk.path().join("opened.txt");
    let script = desk.path().join("editor.sh");
    std::fs::write(
        &script,
        format!("#!/bin/sh\necho \"$1\" > '{}'\n", out.display()),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut settings = desk.saved();
    settings.general.editor = Editor::Custom;
    settings.general.editor_command = format!("{} {{file}}:{{line}}", script.display());
    settings.save(&desk.settings_path()).unwrap();
    let mut harness = desk.start();

    // The file of a link, at the line that names the class.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.key_press(Key::Num2);
    harness.run();
    harness
        .get_all_by_role_and_label(Role::Button, "Open in editor")
        .last()
        .unwrap()
        .click();
    harness.run_steps(2);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !out.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "the editor was not started"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let opened = std::fs::read_to_string(&out).unwrap();
    assert!(
        opened
            .trim()
            .ends_with("Modules/Checkout/app/Http/Controllers/OrderController.php:5"),
        "{opened}"
    );
    assert!(
        harness
            .query_by_label_contains("Could not open the editor")
            .is_none()
    );
}

#[test]
fn a_narrow_window_collapses_the_index_and_ctrl_b_opens_it() {
    let mut desk = Desk::new();
    desk.size = egui::vec2(960.0, 600.0);
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();

    // 960 px: the index is its strip, by itself.
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Filter modules")
            .is_none()
    );
    harness.get_by_role_and_label(Role::ListBoxOption, "Checkout");

    // Ctrl B opens it, and the workspace gives way.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::B);
    harness.run();
    harness.get_by_role_and_label(Role::TextInput, "Filter modules");

    // A wide window has the full index by itself.
    let (_desk, wide) = shop_api();
    wide.get_by_role_and_label(Role::TextInput, "Filter modules");
    let labels: Vec<_> = wide
        .get_all_by_role(Role::ListBoxOption)
        .filter_map(|row| row.accesskit_node().label())
        .collect();
    assert_eq!(labels.len(), 9);
}
