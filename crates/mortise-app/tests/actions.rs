//! The Actions tab (S10.1 to S10.3) against the fake artisan: the line on screen is
//! the line that runs, the questions of the confirmation table, the toasts, and the
//! sentence of a migration that failed.

mod common;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use common::Desk;
use eframe::egui::{Key, Modifiers, Role};
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
    desk.knows(&[(&root, "SA")]);
    let mut settings = desk.saved();
    settings.apps[0].php.runner = Some(RunnerKind::Custom {
        line: format!("\"{}\"{words}", env!("CARGO_BIN_EXE_app-fake-artisan")),
    });
    settings.save(&desk.settings_path()).unwrap();
    (desk, root)
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

/// The runs kept in the history: what ran.
fn runs(desk: &Desk, root: &std::path::Path) -> usize {
    std::thread::sleep(Duration::from_millis(300));
    History::open(&desk.data_dir(), root).entries().len()
}

#[test]
fn migrate_runs_the_line_on_screen_and_the_toast_counts_its_migration() {
    let (desk, root) = desk("");
    let mut harness = desk.start();
    actions_of(&mut harness, "Checkout");
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
    harness
        .get_by_role_and_label(Role::CheckBox, "Seed afterwards")
        .click();
    harness.run_steps(3);

    // Copy puts the line on the clipboard and runs nothing.
    harness
        .get_by_role_and_label(Role::Button, "Copy command")
        .click();
    harness.run_steps(3);
    harness.get_by_label("Copied the command.");
    assert_eq!(runs(&desk, &root), 0);

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
    assert_eq!(runs(&desk, &root), 0);
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
    assert_eq!(runs(&desk, &root), 0);
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
    assert_eq!(runs(&desk, &root), 0);

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
fn a_failed_migration_is_named_and_the_toast_shows_the_output() {
    let (desk, root) = desk("");
    let migrations = root.join("Modules/Shipping/database/migrations");
    std::fs::create_dir_all(&migrations).unwrap();
    let name = "2026_11_22_140500_add_tracking_code_to_shipments_table";
    std::fs::write(migrations.join(format!("{name}.php")), "<?php\n").unwrap();
    let mut harness = desk.start();
    actions_of(&mut harness, "Shipping");
    run(&mut harness);
    // Hidden, the drawer cannot show the failure: a toast does.
    harness.key_press_modifiers(Modifiers::CTRL, Key::Backtick);
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
    assert_eq!(runs(&desk, &root), 1);
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
