//! Accessibility (section 9), checked on the screens of phase 1 and the runner drawer: every control has a
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
        // A chart and a sparkline are drawn: the name is all a screen reader has of them.
        if role == Role::Image && node.label().unwrap_or_default().trim().is_empty() {
            problems.push("an Image without a name".to_owned());
        }
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
    // A dialog is modal: the page behind it is under a scrim, and its buttons may share a
    // name with the dialog's own (Roll back, Dry run). With a dialog open, names are
    // unique within the dialog; otherwise within the page.
    if let Some(dialog) = harness.query_all_by_role(Role::Dialog).last() {
        buttons = dialog
            .children_recursive()
            .filter(|node| node.accesskit_node().role() == Role::Button)
            .map(|node| node.accesskit_node().label().unwrap_or_default())
            .collect();
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
    // The runtime mark of a row is drawn, and says its status in a name.
    assert!(
        harness.query_by_label_contains(": Up on local").is_some(),
        "the runtime marks of the index rows have no name"
    );
    press(&mut harness, Key::F);
    audit(&harness, "Findings list");
    press(&mut harness, Key::Escape);
    press(&mut harness, Key::Num2);
    audit(&harness, "Boundary of all modules");
    for (digit, screen) in [
        (Key::Num3, "Generate of all modules"),
        (Key::Num4, "Actions of all modules"),
        (Key::Num5, "Metrics, not exposed"),
    ] {
        press(&mut harness, digit);
        audit(&harness, screen);
    }
    chord(&mut harness, Key::N);
    audit(&harness, "New module form");

    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run_steps(3);
    audit(&harness, "Generator picker");
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Model")
        .click();
    harness.run_steps(3);
    audit(&harness, "Model form");
    // Out of the name field, which the form focused.
    press(&mut harness, Key::Escape);
    press(&mut harness, Key::Num1);
    audit(&harness, "Summary of a module, with routes");
    press(&mut harness, Key::Num4);
    audit(&harness, "Actions of a module, Migrate");
    for action in ["Roll back", "Seed", "Destructive", "Delete module"] {
        harness
            .get_by_role_and_label(Role::ListBoxOption, action)
            .click();
        harness.run_steps(3);
        audit(&harness, action);
    }
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

/// S7.3: the environment switch, the ranges, the chart, the rows of the table and the routes of a
/// module.
#[test]
fn the_metrics_table_and_the_routes_of_a_module() {
    let stub = mortise_core::testing::StubActuator::recorded();
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(&[
        common::environment("local", &stub),
        common::environment("staging", &stub),
    ]);
    let mut harness = desk.start();
    common::wait_for_answer(&mut harness);
    // The box of the Summary has the same switch.
    audit(&harness, "Summary with two environments");
    press(&mut harness, Key::Num5);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while harness.query_by_label_contains("Requests/min").is_none() {
        assert!(std::time::Instant::now() < deadline, "the metrics table");
        std::thread::sleep(std::time::Duration::from_millis(20));
        harness.run_steps(2);
    }
    // A second read draws the chart. After one read the chart only says "No rate yet.": wait for the
    // summary of the recorded rates, which repeat, so every rate is 0.
    chord(&mut harness, Key::R);
    while harness
        .query_by_role_and_label(Role::Image, "Requests per minute, all modules, last hour")
        .and_then(|chart| chart.accesskit_node().description())
        .as_deref()
        != Some("Now 0, highest 0, lowest 0 requests per minute.")
    {
        assert!(std::time::Instant::now() < deadline, "the chart summary");
        std::thread::sleep(std::time::Duration::from_millis(20));
        harness.run_steps(2);
    }
    audit(&harness, "Metrics table of all modules");
    // The two switches are groups of radio buttons, and the range is chosen by its name.
    for name in ["local", "staging", "15 min", "1 hour", "24 hours"] {
        harness.get_by_role_and_label(Role::RadioButton, name);
    }
    harness
        .get_by_role_and_label(Role::RadioButton, "24 hours")
        .click();
    harness.run_steps(3);
    audit(&harness, "Metrics, 24 hours");
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Billing")
        .click();
    harness.run_steps(3);
    audit(&harness, "Metrics routes of a module");
}

/// S6.5 c to e: the buttons of the runtime box that tell the user what to do next.
#[test]
fn the_runtime_box_in_trouble() {
    for (letter, buttons) in [
        ('c', &["Try again", "Open settings"][..]),
        ('d', &["Copy line", "Try again"][..]),
        ('e', &["Open settings"][..]),
    ] {
        let desk = Desk::new();
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        let environment = match common::runtime_state(letter) {
            Some(stub) => common::environment("local", &stub),
            None => mortise_core::Environment {
                name: "local".into(),
                base_url: "http://127.0.0.1:9".into(),
                path: "actuator".into(),
                credentials: mortise_core::Credentials::None,
            },
        };
        desk.connect(&[environment]);
        let mut harness = desk.start();
        common::wait_for_answer(&mut harness);
        for button in buttons {
            harness.get_by_role_and_label(Role::Button, button);
        }
        audit(&harness, &format!("Runtime box, state {letter}"));
    }
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
        .get_by_role_and_label(Role::ListBoxOption, "PHP")
        .click();
    harness.run_steps(3);
    audit(&harness, "Settings, PHP");
    test_a_missing_program(&mut harness);
    audit(&harness, "Settings, PHP, a program that is not there");
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
    // S1.3 is the "Run PHP with" part of this panel; S1.4 c is a program that is not there.
    test_a_missing_program(&mut harness);
    audit(
        &harness,
        "add panel, Run PHP with, a program that is not there",
    );
}

/// Chooses Custom, types a program that does not exist and tests it, in the PHP part
/// of the screen that is up.
fn test_a_missing_program(harness: &mut Harness<'static, MortiseApp>) {
    harness
        .get_by_role_and_label(Role::RadioButton, "Custom")
        .click();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::TextInput, "Run PHP with")
        .focus();
    harness.run_steps(2);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Run PHP with")
        .type_text("php-none artisan");
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::Button, "Test how PHP runs")
        .click();
    harness.run_steps(6);
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
    press(&mut harness, Key::Num5);
    audit(&harness, "Metrics, not connected");
}

#[test]
fn the_runner_drawer() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut settings = desk.saved();
    settings.apps[0].php.runner = Some(mortise_core::RunnerKind::Custom {
        line: format!("\"{}\"", env!("CARGO_BIN_EXE_app-fake-artisan")),
    });
    settings.save(&desk.settings_path()).unwrap();
    let args = ["module:migrate", "Shipping"].map(str::to_owned).to_vec();
    let entry = mortise_core::history::Entry {
        id: 1,
        label: args.join(" "),
        args,
        started_ms: 1,
        ended: mortise_core::history::Ended::Failed {
            code: Some(1),
            ms: 412,
        },
        cut: false,
    };
    mortise_core::history::History::open(&desk.data_dir(), &root)
        .record(entry, &[])
        .unwrap();
    let mut harness = desk.start();

    harness.key_press_modifiers(Modifiers::CTRL, Key::Backtick);
    harness.run_steps(3);
    audit(&harness, "runner drawer, history");
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while harness
        .query_by_label_contains("stopped with exit code 1")
        .is_none()
    {
        assert!(std::time::Instant::now() < deadline, "the run did not end");
        std::thread::sleep(std::time::Duration::from_millis(20));
        harness.run_steps(2);
    }
    audit(&harness, "runner drawer, failed");
    harness.key_press_modifiers(Modifiers::CTRL, Key::Backtick);
    harness.run_steps(3);
    audit(&harness, "runner drawer, collapsed");
}

/// A desk for shop-api that runs `line`, with these runs in the history of the app.
fn runner_desk(line: &str, runs: &[&[&str]]) -> Desk {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut settings = desk.saved();
    settings.apps[0].php.runner = Some(mortise_core::RunnerKind::Custom {
        line: line.to_owned(),
    });
    settings.save(&desk.settings_path()).unwrap();
    for (age, words) in runs.iter().enumerate() {
        common::ran(&desk, &root, &[(&words.join(" "), 60 + age as i64)]);
    }
    desk
}

/// The fake artisan, with these words before the artisan arguments.
fn fake(words: &str) -> String {
    format!("\"{}\"{words}", env!("CARGO_BIN_EXE_app-fake-artisan"))
}

fn wait_for(harness: &mut Harness<'static, MortiseApp>, text: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while harness.query_by_label_contains(text).is_none() {
        assert!(
            std::time::Instant::now() < deadline,
            "waited 20 s for: {text}"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
        harness.run_steps(2);
    }
    harness.run_steps(3);
}

fn run_again(harness: &mut Harness<'static, MortiseApp>) {
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
    harness.run_steps(3);
}

#[test]
fn the_runner_drawer_empty_done_running_and_not_started() {
    // S11.1 f: nothing has run yet.
    let desk = runner_desk(&fake(""), &[]);
    let mut harness = desk.start();
    harness.key_press_modifiers(Modifiers::CTRL, Key::Backtick);
    harness.run_steps(3);
    audit(&harness, "runner drawer, empty");

    // S11.1 b: finished.
    let desk = runner_desk(&fake(""), &[&["module:seed", "Blog"]]);
    let mut harness = desk.start();
    run_again(&mut harness);
    run_again(&mut harness);
    wait_for(&mut harness, "module:seed Blog: Done in ");
    audit(&harness, "runner drawer, done");

    // S11.1 a: running, then stopped.
    let desk = runner_desk(&fake(" hold"), &[&["module:migrate", "--all"]]);
    let mut harness = desk.start();
    run_again(&mut harness);
    run_again(&mut harness);
    wait_for(&mut harness, "Running, ");
    audit(&harness, "runner drawer, running");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Period);
    harness.run_steps(3);
    audit(&harness, "runner drawer, stop asks first");
    harness
        .get_by_role_and_label(Role::Button, "Stop command")
        .click();
    wait_for(&mut harness, "Stopped after");
    audit(&harness, "runner drawer, stopped");

    // The command did not start.
    let desk = runner_desk("mortise-no-such-php artisan", &[&["module:make", "Blog"]]);
    let mut harness = desk.start();
    run_again(&mut harness);
    run_again(&mut harness);
    harness.run_steps(3);
    audit(&harness, "runner drawer, did not start");
}

/// Selects a row of the index or of the actions list.
fn select(harness: &mut Harness<'static, MortiseApp>, row: &str) {
    harness
        .get_by_role_and_label(Role::ListBoxOption, row)
        .click();
    harness.run_steps(3);
}

fn type_name(harness: &mut Harness<'static, MortiseApp>, field: &str, text: &str) {
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .focus();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .type_text(text);
    harness.run_steps(3);
}

#[test]
fn the_actions_tab_and_its_dialogs() {
    // Through Herd, so that nothing runs.
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut settings = desk.saved();
    settings.apps[0].php.runner = Some(mortise_core::RunnerKind::Herd);
    settings.save(&desk.settings_path()).unwrap();
    let mut harness = desk.start();

    // S10.1 and S10.2: the panels of a module that is on, and of all modules.
    select(&mut harness, "Checkout");
    press(&mut harness, Key::Num4);
    for row in ["Migrate", "Turn off", "Roll back", "Seed", "Run tests"] {
        select(&mut harness, row);
        audit(&harness, row);
    }
    // S10.3: Roll back asks first.
    select(&mut harness, "Roll back");
    chord(&mut harness, Key::Enter);
    audit(&harness, "Roll back, dialog");
    press(&mut harness, Key::Escape);
    // S10.4 and S10.5: the destructive group, its dialogs and the typed name.
    select(&mut harness, "Destructive");
    for row in ["Reset migrations", "Refresh migrations"] {
        select(&mut harness, row);
        chord(&mut harness, Key::Enter);
        audit(&harness, &format!("{row}, dialog"));
        press(&mut harness, Key::Escape);
    }
    select(&mut harness, "Delete module");
    chord(&mut harness, Key::Enter);
    audit(&harness, "Delete module, dialog");
    type_name(&mut harness, "Type Checkout to confirm", "Checkout");
    audit(&harness, "Delete module, name typed");
    press(&mut harness, Key::Escape);
    select(&mut harness, "All modules");
    select(&mut harness, "Destructive");
    select(&mut harness, "Drop all tables and migrate");
    audit(&harness, "All modules, Destructive");
    chord(&mut harness, Key::Enter);
    audit(&harness, "Drop all tables, dialog");
    type_name(&mut harness, "Type shop-api to confirm", "shop-api");
    audit(&harness, "Drop all tables, name typed");
    press(&mut harness, Key::Escape);

    // S10.4 d: the app reports production.
    std::fs::write(root.join(".env"), "APP_ENV=production\n").unwrap();
    chord(&mut harness, Key::R);
    common::wait_until(&mut harness, "the app reports production", |app| {
        app.app()
            .is_some_and(|app| app.app_env.as_deref() == Some("production"))
    });
    select(&mut harness, "Checkout");
    select(&mut harness, "Destructive");
    select(&mut harness, "Reset migrations");
    audit(&harness, "Reset migrations, production");
}

#[test]
fn the_actions_and_findings_of_an_app_with_package_data() {
    let desk = runner_desk(&fake(""), &[]);
    common::package_version(&desk.path().join("shop-api"), "v13.4.0");
    let mut harness = desk.start();
    select(&mut harness, "Checkout");
    press(&mut harness, Key::Num4);
    audit(&harness, "Migrate, state not read");
    harness
        .get_by_role_and_label(Role::Button, "Read migration state")
        .click();
    wait_for(&mut harness, "2026_11_23_120000_create_order_items_table");
    audit(&harness, "Migrate, state read");
    harness.get_by_role_and_label(Role::Button, "Read again");
    // Run tests has a filter field and Run in parallel.
    select(&mut harness, "Run tests");
    harness.get_by_role_and_label(Role::TextInput, "Filter, a test name or pattern");
    harness.get_by_role_and_label(Role::CheckBox, "Run in parallel");
    audit(&harness, "Run tests, options");
    // The To do group of Findings, after Check translations.
    select(&mut harness, "Check translations");
    chord(&mut harness, Key::Enter);
    common::wait_until(&mut harness, "the translation report", |app| {
        app.app()
            .is_some_and(|app| app.package_data.translations.is_some())
    });
    press(&mut harness, Key::F);
    for row in [
        "Checkout has 1 migration waiting.",
        "2 keys in en are missing in fr.",
    ] {
        harness
            .query_all_by_role(Role::Row)
            .find(|node| {
                node.accesskit_node()
                    .label()
                    .is_some_and(|l| l.contains(row))
            })
            .unwrap_or_else(|| panic!("no row: {row}"));
    }
    audit(&harness, "Findings with a To do group");
}

#[test]
fn the_generate_tab_its_result_and_replace() {
    let desk = runner_desk(&fake(""), &[]);
    let mut harness = desk.start();
    // S9.1: the picker.
    select(&mut harness, "Billing");
    press(&mut harness, Key::Num3);
    audit(&harness, "Generate, picker");
    // S9.2: a form.
    select(&mut harness, "Model");
    type_name(&mut harness, "Model name", "PaymentIntent");
    type_name(
        &mut harness,
        "Fillable attributes",
        "order_id,amount,currency,status",
    );
    for flag in ["Migration", "Policy"] {
        harness.get_by_role_and_label(Role::CheckBox, flag).click();
        harness.run_steps(3);
    }
    audit(&harness, "Generate, model form");
    // S9.5: every file created.
    chord(&mut harness, Key::Enter);
    wait_for(&mut harness, "Created 3 files in Billing.");
    audit(&harness, "Generate, result");
    // S9.6: the policy of Invoice exists, so it is left, and Replace asks.
    harness
        .get_by_role_and_label(Role::Button, "Generate another")
        .click();
    harness.run_steps(3);
    type_name(&mut harness, "Model name", "Invoice");
    harness
        .get_by_role_and_label(Role::TextInput, "Fillable attributes")
        .focus();
    harness.run_steps(2);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.key_press(Key::Backspace);
    harness.run_steps(3);
    chord(&mut harness, Key::Enter);
    wait_for(&mut harness, "Created 2 of 3 files in Billing.");
    audit(&harness, "Generate, result with a file left");
    harness
        .get_by_role_and_label(Role::Button, "Replace InvoicePolicy.php")
        .click();
    harness.run_steps(3);
    audit(&harness, "Generate, Replace dialog");
    press(&mut harness, Key::Escape);
    // S9.4: the new module form, with its Replace dialog.
    chord(&mut harness, Key::N);
    type_name(&mut harness, "Module names", "Loyalty");
    harness
        .get_by_role_and_label(Role::RadioButton, "API")
        .click();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::CheckBox, "Replace the module if it exists")
        .click();
    harness.run_steps(3);
    audit(&harness, "Generate, new module form");
    chord(&mut harness, Key::Enter);
    audit(&harness, "Generate, Replace module dialog");
}

#[test]
fn the_generate_tab_of_a_13_4_app() {
    let desk = runner_desk(&fake(""), &[]);
    common::package_version(&desk.path().join("shop-api"), "v13.4.0");
    let mut harness = desk.start();
    // S9.1: the button that reads the generators, then its "Read again".
    select(&mut harness, "Billing");
    press(&mut harness, Key::Num3);
    audit(&harness, "Generate, picker before the read");
    harness
        .get_by_role_and_label(Role::Button, "Read the generators of shop-api")
        .click();
    wait_for(&mut harness, "Read from shop-api at ");
    harness.get_by_role_and_label(Role::Button, "Read again");
    audit(&harness, "Generate, picker after the read");
    // S9.3: the dry run finds the policy of Invoice, so "exists" shows.
    select(&mut harness, "Model");
    type_name(&mut harness, "Model name", "Invoice");
    for flag in ["Migration", "Policy"] {
        harness.get_by_role_and_label(Role::CheckBox, flag).click();
        harness.run_steps(3);
    }
    harness
        .get_by_role_and_label(Role::Button, "List files")
        .click();
    wait_for(&mut harness, "3 files, 1 exists already");
    audit(
        &harness,
        "Generate, model form with its files and an existing file",
    );
    // S9.4: the new module form with its file list.
    chord(&mut harness, Key::N);
    type_name(&mut harness, "Module names", "Loyalty");
    harness
        .get_by_role_and_label(Role::RadioButton, "API")
        .click();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::Button, "List files")
        .click();
    wait_for(&mut harness, "Modules/Loyalty does not exist yet");
    audit(&harness, "Generate, new module form with its files");
}
