//! Settings, the editor, and how the window gives width to the workspace.

mod common;

use common::{Desk, shop_api, wait_until};
use eframe::egui::{self, Key, Modifiers, Role, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::{MortiseApp, Selection};
use mortise_core::{CommandLine, Editor, IndexRows, RunnerKind, ThemeChoice};

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

    // Over the Findings list, Esc closes Settings and leaves the list as it was.
    harness.key_press(Key::F);
    harness.run();
    open_settings(&mut harness);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Settings")
            .is_none()
    );
    assert!(harness.state().findings_open());
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

    // A monogram that is not letters and digits is not taken: the field shows the saved one.
    harness
        .get_by_role_and_label(Role::TextInput, "Monogram")
        .focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Monogram")
        .type_text("?!");
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Name")
        .focus();
    harness.run_steps(2);
    assert_eq!(desk.saved().apps[0].monogram, "SH");
    assert_eq!(
        harness
            .get_by_role_and_label(Role::TextInput, "Monogram")
            .value()
            .as_deref(),
        Some("SH")
    );

    // Esc in a field keeps what was typed and leaves the field. The next Esc closes Settings.
    harness
        .get_by_role_and_label(Role::TextInput, "Coverage limit")
        .focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Coverage limit")
        .type_text("25");
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(desk.saved().apps[0].coverage_limit, 25);
    harness.get_by_role_and_label(Role::Heading, "Settings");
    harness.key_press(Key::Escape);
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Settings")
            .is_none()
    );

    // A click on Back while a field holds an edit keeps the edit and goes back.
    open_settings(&mut harness);
    harness
        .get_by_role_and_label(Role::TextInput, "Coverage limit")
        .focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Coverage limit")
        .type_text("35");
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Back to shop-api")
        .click();
    harness.run();
    assert_eq!(desk.saved().apps[0].coverage_limit, 35);
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Settings")
            .is_none()
    );
}

fn open_php(harness: &mut Harness<'static, MortiseApp>) {
    open_settings(harness);
    harness
        .get_by_role_and_label(Role::ListBoxOption, "PHP")
        .click();
    harness.run();
}

/// Replaces the text of the command field of "Run PHP with".
fn type_command(harness: &mut Harness<'static, MortiseApp>, line: &str) {
    harness
        .get_by_role_and_label(Role::TextInput, "Run PHP with")
        .focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Run PHP with")
        .type_text(line);
    harness.run();
}

#[test]
fn the_kind_of_runner_follows_the_command_and_is_kept() {
    let desk = Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
    let mut harness = desk.start();
    open_php(&mut harness);
    harness.get_by_role_and_label(Role::Heading, "PHP");
    // Nothing is chosen yet: the field shows the proposal and nothing is saved.
    harness.get_by_label("Not found: Herd, Sail, a compose file.");
    assert_eq!(desk.saved().apps[0].php.runner, None);

    harness
        .get_by_role_and_label(Role::RadioButton, "Docker")
        .click();
    harness.run();
    let docker = |service: &str| {
        Some(RunnerKind::Docker {
            service: service.to_owned(),
        })
    };
    assert_eq!(desk.saved().apps[0].php.runner, docker("app"));

    // The service is edited in the command.
    type_command(&mut harness, "docker compose exec -T web php artisan");
    harness
        .get_by_role_and_label(Role::TextInput, "Working folder")
        .focus();
    harness.run();
    assert_eq!(desk.saved().apps[0].php.runner, docker("web"));
    harness.get_by_label("docker compose exec -T web php artisan");

    // Another app is selected while the field still has focus: the edit is kept,
    // and the field of the other app shows its own command.
    type_command(&mut harness, "docker compose exec -T worker php artisan");
    harness.get_by_role_and_label(Role::Row, "crm").click();
    harness.run();
    assert_eq!(desk.saved().apps[0].php.runner, docker("worker"));
    assert_eq!(
        harness
            .get_by_role_and_label(Role::TextInput, "Run PHP with")
            .value(),
        Some(RunnerKind::Herd.line())
    );
}

#[test]
fn test_says_why_php_did_not_answer() {
    let (_desk, mut harness) = shop_api();
    open_php(&mut harness);
    harness
        .get_by_role_and_label(Role::RadioButton, "Custom")
        .click();
    harness.run();
    let typed = "mortise-no-such-php artisan";
    type_command(&mut harness, typed);
    // The exact line that Test runs is on screen before it runs.
    let runs = CommandLine::artisan(&RunnerKind::from_line(typed), &["--version"]).unwrap();
    harness.get_by_label(&format!("Test runs {}", runs.display()));

    // Settings close while the test runs. It goes on, and its result is there on return.
    harness
        .get_by_role_and_label(Role::Button, "Test how PHP runs")
        .click();
    harness.step();
    harness.key_press(Key::Escape);
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Settings")
            .is_none()
    );
    open_php(&mut harness);
    let failed = "mortise-no-such-php did not answer: \"mortise-no-such-php: command not found\".";
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while harness.query_by_label_contains(failed).is_none() {
        assert!(
            std::time::Instant::now() < deadline,
            "waited 10 s for: {failed}"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
        harness.run_steps(2);
    }
    harness.get_by_label("No answer: mortise-no-such-php: command not found");

    // The result is not shown under a runner it did not test. The table keeps it.
    harness
        .get_by_role_and_label(Role::RadioButton, "Docker")
        .click();
    harness.run();
    assert!(harness.query_by_label_contains(failed).is_none());
    harness.get_by_label("No answer: mortise-no-such-php: command not found");
}

#[test]
fn removing_an_app_asks_first_and_leaves_its_folder() {
    let desk = Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
    let mut harness = desk.start();
    open_settings(&mut harness);

    // The click that leaves an edited field keeps the edit and asks.
    harness
        .get_by_role_and_label(Role::TextInput, "Coverage limit")
        .focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Coverage limit")
        .type_text("35");
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Remove shop-api from Mortise")
        .click();
    harness.run();
    assert_eq!(desk.saved().apps[0].coverage_limit, 35);
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
    harness.get_by_label("27 of 27 shown.");

    harness
        .get_by_role_and_label(Role::TextInput, "Find a shortcut")
        .focus();
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Find a shortcut")
        .type_text("palette");
    harness.run();
    harness.get_by_label("3 of 27 shown.");
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
        .get_by_role_and_label(Role::Button, "Open the selected file in the editor")
        .click();
    harness.run_steps(2);
    // The shell makes the file before it writes the line, so wait for the line.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let opened = loop {
        let opened = std::fs::read_to_string(&out).unwrap_or_default();
        if opened.ends_with('\n') {
            break opened;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the editor was not started"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
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
    let fits = harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .rect();
    // The Findings cell has no room for both states on one line: they wrap.
    let problems = harness.get_by_label("2 problems").rect();
    let warning = harness.get_by_label("1 warning").rect();
    assert!(
        warning.top() >= problems.bottom(),
        "{problems:?} {warning:?}"
    );

    // Rows that do not fit: the strip adds the 10 px track of the scrollbar at its
    // right, so that the scrollbar does not cover the marks.
    harness.set_size(egui::vec2(960.0, 400.0));
    harness.run();
    let scrolls = harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .rect();
    assert_eq!(scrolls.right() - fits.right(), 10.0);
    harness.set_size(egui::vec2(960.0, 600.0));
    harness.run();

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

#[test]
fn a_name_given_in_settings_is_the_name_everywhere() {
    let (desk, mut harness) = shop_api();
    open_settings(&mut harness);
    let name = harness.get_by_role_and_label(Role::TextInput, "Name");
    name.focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Name")
        .type_text("Shop");
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Monogram")
        .focus();
    harness.run();
    assert_eq!(desk.saved().apps[0].name, "Shop");

    // The index, the rail and the title bar say the new name; the folder keeps its own.
    // The first Escape leaves the field, the second closes Settings.
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press(Key::Escape);
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Shop");
    harness.get_by_role_and_label(Role::ListBoxOption, "Shop");
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "shop-api")
            .is_none()
    );

    // Findings, Boundary and the palette say it too.
    harness.key_press(Key::F);
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Findings in Shop");
    harness.get_by_role_and_label(
        Role::Row,
        "Notifications is 39% covered. The limit for Shop is 50%.",
    );
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press(Key::Num2);
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Rules for Shop");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run_steps(3);
    harness.get_by_label("Findings in Shop");
    harness
        .get_by_role_and_label(Role::TextInput, "Search or run a command")
        .type_text("check");
    harness.run_steps(3);
    harness.get_by_label("Modules in Shop");
}
