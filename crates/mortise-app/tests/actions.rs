//! The Actions tab (S10.1 to S10.5) against the fake artisan: the line on screen is
//! the line that runs, the questions of the confirmation table, the closed group of
//! the destructive actions and their dialogs, the toasts, and the sentence of a
//! migration that failed.

mod common;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use common::Desk;
use eframe::egui::{Event, Key, Modifiers, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::{MortiseApp, Selection};
use mortise_app::workspace::Tab;
use mortise_core::RunnerKind;
use mortise_core::history::History;

/// shop-api, known and open, run through the fake artisan with `words` before the
/// artisan arguments.
fn desk(words: &str) -> (Desk, PathBuf) {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    through_fake(&desk, &root, words);
    (desk, root)
}

/// Makes the app in `root` known and open, run through the fake artisan with `words`
/// before the artisan arguments.
fn through_fake(desk: &Desk, root: &std::path::Path, words: &str) {
    desk.knows(&[(root, "SA")]);
    let mut settings = desk.saved();
    settings.apps[0].php.runner = Some(RunnerKind::Custom {
        line: format!("\"{}\"{words}", env!("CARGO_BIN_EXE_app-fake-artisan")),
    });
    settings.save(&desk.settings_path()).unwrap();
}

/// Selects a module in the index and opens the Actions tab.
fn actions_of(harness: &mut Harness<'static, MortiseApp>, module: &str) {
    harness
        .get_by_role_and_label(Role::ListBoxOption, module)
        .click();
    harness.run_steps(3);
    harness.key_press(Key::Num4);
    harness.run_steps(3);
}

fn select(harness: &mut Harness<'static, MortiseApp>, action: &str) {
    harness
        .get_by_role_and_label(Role::ListBoxOption, action)
        .click();
    harness.run_steps(3);
}

/// Ctrl Enter, or Cmd Enter: run the selected action.
fn run(harness: &mut Harness<'static, MortiseApp>) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.run_steps(3);
}

/// Waits until a label holds `text`: a run needs real time to pass.
fn wait_for(harness: &mut Harness<'static, MortiseApp>, text: &str) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while harness.query_by_label_contains(text).is_none() {
        assert!(Instant::now() < deadline, "waited 20 s for: {text}");
        std::thread::sleep(Duration::from_millis(20));
        harness.run_steps(2);
    }
}

/// The runs kept in the history: what ran. A run is kept once a frame sees its end.
fn runs(harness: &mut Harness<'static, MortiseApp>, desk: &Desk, root: &std::path::Path) -> usize {
    std::thread::sleep(Duration::from_millis(300));
    harness.run_steps(3);
    History::open(&desk.data_dir(), root).entries().len()
}

#[test]
fn migrate_runs_the_line_on_screen_and_the_toast_counts_its_migration() {
    let (desk, root) = desk("");
    let mut harness = desk.start();
    // A key runs only a line that was on screen: the tab and Ctrl Enter in one
    // frame run nothing.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run_steps(3);
    let press = |key, modifiers| Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    };
    let events = &mut harness.input_mut().events;
    events.push(press(Key::Num4, Modifiers::NONE));
    events.push(press(Key::Enter, Modifiers::COMMAND));
    harness.run_steps(3);
    assert_eq!(harness.state().tab(), Tab::Actions);
    assert!(harness.query_by_label_contains("$ ").is_none());
    // Migrate is selected, and its line is the line that runs (deviation S37).
    harness.get_by_role_and_label(Role::Heading, "Migrate Checkout");
    harness
        .query_by_label_contains("module:migrate --no-interaction --ansi Checkout")
        .unwrap();

    // An option changes the line before anything runs.
    harness
        .get_by_role_and_label(Role::CheckBox, "Seed afterwards")
        .click();
    harness.run_steps(3);
    harness
        .query_by_label_contains("--ansi Checkout --seed")
        .unwrap();
    // A dry run cannot seed: the package seeds after a pretend migrate.
    let dry_run = "Dry run: print the SQL, run none of it";
    harness
        .get_by_role_and_label(Role::CheckBox, dry_run)
        .click();
    harness.run_steps(3);
    let seed = harness.get_by_role_and_label(Role::CheckBox, "Seed afterwards");
    assert!(seed.accesskit_node().is_disabled());
    harness
        .query_by_label_contains("--ansi Checkout --pretend")
        .unwrap();
    // Another module, selected on another tab, starts the options again.
    harness.key_press(Key::Num1);
    harness.run_steps(3);
    select(&mut harness, "Billing");
    select(&mut harness, "Checkout");
    harness.key_press(Key::Num4);
    harness.run_steps(3);
    assert!(
        harness
            .query_by_label_contains("--ansi Checkout --pretend")
            .is_none()
    );

    // Ctrl Shift C copies the line to the clipboard and runs nothing.
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::C);
    harness.run_steps(3);
    harness.get_by_label("Copied the command.");
    assert_eq!(runs(&mut harness, &desk, &root), 0);

    run(&mut harness);
    wait_for(&mut harness, "Migrated Checkout: 1 migration ran.");
    let entries = History::open(&desk.data_dir(), &root).entries();
    assert_eq!(entries[0].args, ["module:migrate", "Checkout"]);
}

#[test]
fn turning_off_a_module_that_others_use_asks_and_names_them() {
    let (desk, _root) = desk("");
    let mut harness = desk.start();
    actions_of(&mut harness, "Checkout");
    select(&mut harness, "Turn off");
    harness.get_by_label(
        "Notifications and Shipping use Checkout. Mortise asks before it turns off a module \
         that others use.",
    );
    harness
        .get_by_role_and_label(Role::Button, "Turn off")
        .click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Turn off Checkout?");
    harness.get_by_label(
        "Notifications and Shipping use it. They stay on and can fail where they call Checkout.",
    );
    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Cancel")
            .is_focused()
    );
    harness
        .get_by_role_and_label(Role::Button, "Turn off Checkout")
        .click();
    harness.run_steps(3);
    wait_for(&mut harness, "Turned off Checkout.");
}

#[test]
fn roll_back_asks_and_its_dry_run_only_checks_the_option() {
    let (desk, root) = desk("");
    let mut harness = desk.start();
    actions_of(&mut harness, "Checkout");
    select(&mut harness, "Roll back");
    run(&mut harness);
    harness.get_by_role_and_label(Role::Heading, "Roll back Checkout?");
    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Cancel")
            .is_focused()
    );
    // The dry run on the left of the dialog checks the option on the panel (S38).
    let dry_runs = harness.get_all_by_role_and_label(Role::Button, "Dry run");
    dry_runs.last().unwrap().click();
    harness.run_steps(3);
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Roll back Checkout?")
            .is_none()
    );
    harness
        .query_by_label_contains(
            "module:migrate-rollback --no-interaction --ansi Checkout --pretend",
        )
        .unwrap();
    assert_eq!(runs(&mut harness, &desk, &root), 0);

    // A roll back that was confirmed asks again before Run again runs it.
    harness
        .get_by_role_and_label(Role::CheckBox, "Dry run: print the SQL, run none of it")
        .click();
    harness.run_steps(3);
    run(&mut harness);
    let roll_backs = harness.get_all_by_role_and_label(Role::Button, "Roll back");
    roll_backs.last().unwrap().click();
    harness.run_steps(3);
    wait_for(&mut harness, "Rolled back Checkout: 1 migration.");
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Run module:migrate-rollback Checkout again?");
    harness.get_by_label(
        "Mortise asked before this command ran last time. It runs the same line again.",
    );
    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Cancel")
            .is_focused()
    );
    harness
        .get_by_role_and_label(Role::Button, "Cancel")
        .click();
    harness.run_steps(3);
    assert_eq!(runs(&mut harness, &desk, &root), 1);
}

#[test]
fn production_asks_and_the_line_carries_force() {
    let (desk, root) = desk("");
    std::fs::write(root.join(".env"), "APP_ENV=production\n").unwrap();
    let mut harness = desk.start();
    actions_of(&mut harness, "Checkout");
    harness
        .query_by_label_contains("module:migrate --no-interaction --ansi Checkout --force")
        .unwrap();
    run(&mut harness);
    harness.get_by_role_and_label(Role::Heading, "Migrate Checkout in production?");
    harness.get_by_label(
        "shop-api reports the environment production. The command runs with --force on that \
         database.",
    );
    harness
        .get_by_role_and_label(Role::Button, "Cancel")
        .click();
    harness.run_steps(3);

    // A destructive action does not run in production: its line can only be copied.
    select(&mut harness, "Destructive");
    select(&mut harness, "Reset migrations");
    harness.get_by_label(
        "shop-api reports the environment production. Mortise does not run this command \
         there. Copy it and run it where you can watch the database.",
    );
    let button = harness.get_by_role_and_label(Role::Button, "Reset migrations");
    assert!(button.accesskit_node().is_disabled());
    run(&mut harness);
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Reset the migrations of Checkout?")
            .is_none()
    );
    harness
        .get_by_role_and_label(Role::Button, "Copy command")
        .click();
    harness.run_steps(3);
    harness.get_by_label("Copied the command.");
    assert_eq!(runs(&mut harness, &desk, &root), 0);
}

#[test]
fn the_header_opens_the_panel_and_run_tests_is_off() {
    let (desk, root) = desk("");
    let mut harness = desk.start();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::Button, "Run tests")
        .click();
    harness.run_steps(3);
    assert_eq!(harness.state().tab(), Tab::Actions);
    harness.get_by_role_and_label(Role::Heading, "Run tests of Checkout");

    // The placeholder line cannot run or be copied, and says why.
    let button = harness.get_by_role_and_label(Role::Button, "Run tests");
    assert!(button.accesskit_node().is_disabled());
    button.hover();
    harness.run_steps(12);
    harness.get_by_label("The package has no command for the tests of one module yet.");
    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Copy command")
            .accesskit_node()
            .is_disabled()
    );
    run(&mut harness);
    assert_eq!(runs(&mut harness, &desk, &root), 0);

    // K, in the list, moves to Seed.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Run tests")
        .focus();
    harness.run_steps(2);
    harness.key_press(Key::K);
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Seed Checkout");
    assert_eq!(
        harness.state().selection(),
        Selection::Module("Checkout".into())
    );
}

#[test]
fn run_tests_on_13_4_runs_module_test_and_check_translations_keeps_its_report() {
    let desk = Desk::new();
    let root = common::app_with_package(&desk, "shop-api", "v13.4.0");
    through_fake(&desk, &root, "");
    let mut harness = desk.start();
    actions_of(&mut harness, "Checkout");
    select(&mut harness, "Run tests");
    harness.get_by_label("2 tests in Modules/Checkout/tests.");
    run(&mut harness);
    wait_for(&mut harness, "1 test passed in Checkout.");
    let entries = History::open(&desk.data_dir(), &root).entries();
    assert_eq!(entries[0].args, ["module:test", "Checkout", "--json"]);

    // The options of `module:test` change the line (design D8).
    type_in(&mut harness, "Filter, a test name or pattern", "Order");
    harness
        .get_by_role_and_label(Role::CheckBox, "Run in parallel")
        .click();
    harness.run_steps(3);
    harness
        .query_by_label_contains("--ansi Checkout --filter=Order --parallel --json")
        .unwrap();

    // Its report goes into the package data, although keys are missing (exit 1).
    select(&mut harness, "Check translations");
    run(&mut harness);
    common::wait_until(&mut harness, "the report of Checkout", |app| {
        app.app().is_some_and(|app| {
            app.package_data
                .translations
                .as_ref()
                .is_some_and(|report| report.modules.iter().any(|m| m.name == "Checkout"))
        })
    });
    let entries = History::open(&desk.data_dir(), &root).entries();
    assert_eq!(entries[0].args, ["module:lang", "Checkout", "--json"]);
}

#[test]
fn a_failed_migration_is_named_and_the_toast_shows_the_output() {
    // The fake ends only once the gate file exists, so the drawer is hidden first.
    let (desk, root) = desk("");
    let gate = desk.path().join("gate");
    let mut settings = desk.saved();
    settings.apps[0].php.runner = Some(RunnerKind::Custom {
        line: format!(
            "\"{}\" gate \"{}\"",
            env!("CARGO_BIN_EXE_app-fake-artisan"),
            gate.display()
        ),
    });
    settings.save(&desk.settings_path()).unwrap();
    let migrations = root.join("Modules/Shipping/database/migrations");
    std::fs::create_dir_all(&migrations).unwrap();
    let name = "2026_11_22_140500_add_tracking_code_to_shipments_table";
    std::fs::write(migrations.join(format!("{name}.php")), "<?php\n").unwrap();
    let mut harness = desk.start();
    actions_of(&mut harness, "Shipping");
    run(&mut harness);
    // Hidden, the drawer cannot show the failure: a toast does.
    harness.key_press_modifiers(Modifiers::CTRL, Key::Backtick);
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Button, "Show");
    std::fs::write(&gate, "").unwrap();
    wait_for(
        &mut harness,
        "module:migrate Shipping stopped with exit code 1.",
    );
    harness
        .get_by_role_and_label(Role::Button, "Show output")
        .click();
    harness.run_steps(3);
    harness.get_by_label(&format!(
        "module:migrate Shipping stopped with exit code 1 at {name}. The error from the \
         database is quoted above. Fix the migration, or try a dry run to read the SQL."
    ));
    harness.get_by_role_and_label(Role::Button, "Open migration file");

    // Dry run opens the Migrate panel with the option checked, and runs nothing.
    harness
        .get_by_role_and_label(Role::Button, "Dry run of module:migrate Shipping")
        .click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Migrate Shipping");
    harness
        .query_by_label_contains("module:migrate --no-interaction --ansi Shipping --pretend")
        .unwrap();
    assert_eq!(runs(&mut harness, &desk, &root), 1);
}

#[test]
fn a_run_turns_every_run_button_off() {
    let (desk, _root) = desk(" hold");
    let mut harness = desk.start();
    actions_of(&mut harness, "Checkout");
    run(&mut harness);
    wait_for(&mut harness, "Running Migration Checkout Module");
    select(&mut harness, "Seed");
    let seed = harness.get_by_role_and_label(Role::Button, "Seed");
    assert!(seed.accesskit_node().is_disabled());
    seed.hover();
    harness.run_steps(12);
    harness.get_by_label("module:migrate Checkout is running");

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Period);
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::Button, "Stop command")
        .click();
    wait_for(&mut harness, "Stopped after");
}

/// Gives the field of this name focus and types `text` into it.
fn type_in(harness: &mut Harness<'static, MortiseApp>, field: &str, text: &str) {
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .focus();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .type_text(text);
    harness.run_steps(3);
}

/// True while a row of this name is in the list.
fn listed(harness: &Harness<'static, MortiseApp>, row: &str) -> bool {
    harness
        .query_by_role_and_label(Role::ListBoxOption, row)
        .is_some()
}

#[test]
fn the_destructive_group_opens_with_its_keys_and_a_reset_dry_run_runs_at_once() {
    let (desk, root) = desk("");
    let mut harness = desk.start();
    actions_of(&mut harness, "Billing");
    // Closed when the tab opens.
    assert!(!listed(&harness, "Reset migrations"));
    let group = harness.get_by_role_and_label(Role::ListBoxOption, "Destructive");
    assert_eq!(group.accesskit_node().data().is_expanded(), Some(false));

    // J stops at the row of the group, Enter opens it, and J moves on into it.
    select(&mut harness, "Check translations");
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Check translations")
        .focus();
    harness.run_steps(2);
    harness.key_press(Key::J);
    harness.run_steps(3);
    assert!(
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Destructive")
            .is_focused()
    );
    harness.key_press(Key::Enter);
    harness.run_steps(3);
    harness.get_by_label("Dropping all tables is under \"All modules\".");
    harness.key_press(Key::J);
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Reset the migrations of Billing");

    // Dry run checks the option, and a dry run of reset runs without a question.
    harness
        .get_by_role_and_label(Role::Button, "Dry run")
        .click();
    harness.run_steps(3);
    // Billing has no recording: the dry run of Checkout prints "⇂", which the
    // bundled mono font does not have.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.run_steps(3);
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Reset the migrations of Billing?")
            .is_none()
    );
    let deadline = Instant::now() + Duration::from_secs(20);
    while History::open(&desk.data_dir(), &root).entries().is_empty() {
        assert!(Instant::now() < deadline, "the dry run did not end");
        std::thread::sleep(Duration::from_millis(20));
        harness.run_steps(2);
    }
    let entries = History::open(&desk.data_dir(), &root).entries();
    assert_eq!(
        entries[0].args,
        ["module:migrate-reset", "Billing", "--pretend"]
    );

    // Another module closes the group, and its own group is closed.
    select(&mut harness, "Checkout");
    assert!(!listed(&harness, "Delete module"));
    harness.get_by_role_and_label(Role::Heading, "Migrate Checkout");
}

#[test]
fn delete_asks_for_the_name_and_selects_all_modules_when_it_ends() {
    let (desk, root) = desk("");
    let mut harness = desk.start();
    actions_of(&mut harness, "Checkout");
    select(&mut harness, "Destructive");
    select(&mut harness, "Delete module");
    let field = "Type Checkout to confirm";
    let confirm = |harness: &Harness<'static, MortiseApp>| {
        let buttons = harness.get_all_by_role_and_label(Role::Button, "Delete module");
        !buttons.last().unwrap().accesskit_node().is_disabled()
    };

    // Ctrl Enter only opens the dialog, with focus on Cancel.
    run(&mut harness);
    harness.get_by_role_and_label(Role::Heading, "Delete the module Checkout?");
    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Cancel")
            .is_focused()
    );
    // The app is not in a repository: the panel and the dialog say git cannot tell.
    let no_git = "Mortise cannot tell whether these files are in git.";
    assert_eq!(harness.get_all_by_label(no_git).count(), 2);
    // The name must match exactly. Esc in the field cancels.
    type_in(&mut harness, field, "checkout");
    assert!(!confirm(&harness));
    assert!(
        harness
            .get_by_role_and_label(Role::TextInput, field)
            .is_focused()
    );
    harness.key_press(Key::Escape);
    harness.run_steps(3);
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, field)
            .is_none()
    );

    run(&mut harness);
    type_in(&mut harness, field, "Checkout");
    assert!(confirm(&harness));
    // Enter does not confirm, in the field or on the red button, with or without
    // the modifier that opened the dialog.
    harness.key_press(Key::Enter);
    harness.run_steps(3);
    harness
        .get_all_by_role_and_label(Role::Button, "Delete module")
        .last()
        .unwrap()
        .focus();
    harness.run_steps(2);
    assert!(
        harness
            .get_all_by_role_and_label(Role::Button, "Delete module")
            .last()
            .unwrap()
            .is_focused()
    );
    for modifiers in [Modifiers::NONE, Modifiers::SHIFT, Modifiers::COMMAND] {
        harness.key_press_modifiers(modifiers, Key::Enter);
        harness.run_steps(2);
    }
    // Tab from Cancel to the red button and Enter in one frame.
    harness
        .get_by_role_and_label(Role::Button, "Cancel")
        .focus();
    harness.run_steps(2);
    harness.key_press(Key::Tab);
    harness.key_press(Key::Enter);
    harness.run_steps(2);
    harness.get_by_role_and_label(Role::Heading, "Delete the module Checkout?");
    assert_eq!(runs(&mut harness, &desk, &root), 0);

    harness
        .get_all_by_role_and_label(Role::Button, "Delete module")
        .last()
        .unwrap()
        .click();
    harness.run_steps(3);
    wait_for(&mut harness, "Deleted the module Checkout.");
    let entries = History::open(&desk.data_dir(), &root).entries();
    assert_eq!(entries[0].args, ["module:delete", "Checkout", "--force"]);
    assert_eq!(harness.state().selection(), Selection::All);
    assert_eq!(harness.state().tab(), Tab::Actions);

    // Run again asks in the same dialog, for the name again.
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Delete the module Checkout?");
    harness.get_by_role_and_label(Role::TextInput, field);
    assert!(!confirm(&harness));
    type_in(&mut harness, field, "Checkout");

    // The app turns to production while the dialog is open, and the user confirms
    // before Mortise reads the app again: the confirmation reads `.env` itself and
    // does not run. The panel says why and keeps the line to copy.
    std::fs::write(root.join(".env"), "APP_ENV=production\n").unwrap();
    harness
        .get_all_by_role_and_label(Role::Button, "Delete module")
        .last()
        .unwrap()
        .click();
    harness.run_steps(3);
    common::wait_until(&mut harness, "the app reports production", |app| {
        app.app()
            .is_some_and(|app| app.app_env.as_deref() == Some("production"))
    });
    harness.run_steps(3);
    harness.get_by_label(
        "shop-api reports the environment production. Mortise does not run this command \
         there. Copy it and run it where you can watch the database.",
    );
    assert_eq!(runs(&mut harness, &desk, &root), 1);
}
