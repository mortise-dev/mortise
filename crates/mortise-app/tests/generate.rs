//! The Generate tab (S9.1 to S9.6) against the fake artisan: the picker, the line
//! of a form, the name checks, the file list of a 13.4 app, the result read from
//! the output or the answer, Replace and its dialog, and the new module form.

mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use common::Desk;
use eframe::egui::accesskit::Toggled;
use eframe::egui::{Key, Modifiers, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::{MortiseApp, Selection};
use mortise_app::workspace::Tab;
use mortise_core::RunnerKind;
use mortise_core::history::History;

/// shop-api, known and open, run through the fake artisan.
fn desk() -> (Desk, PathBuf) {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut settings = desk.saved();
    settings.apps[0].php.runner = Some(RunnerKind::Custom {
        line: format!("\"{}\"", env!("CARGO_BIN_EXE_app-fake-artisan")),
    });
    settings.save(&desk.settings_path()).unwrap();
    (desk, root)
}

/// Selects a module in the index and opens the Generate tab.
fn generate_in(harness: &mut Harness<'static, MortiseApp>, module: &str) {
    harness
        .get_by_role_and_label(Role::ListBoxOption, module)
        .click();
    harness.run_steps(3);
    harness.key_press(Key::Num3);
    harness.run_steps(3);
}

fn click(harness: &mut Harness<'static, MortiseApp>, role: Role, label: &str) {
    harness.get_by_role_and_label(role, label).click();
    harness.run_steps(3);
}

fn type_into(harness: &mut Harness<'static, MortiseApp>, field: &str, text: &str) {
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .focus();
    harness.run_steps(2);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .type_text(text);
    harness.run_steps(3);
}

/// Ctrl Enter, or Cmd Enter: run the form.
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

/// The artisan words of the runs kept in the history, the latest first.
fn ran(harness: &mut Harness<'static, MortiseApp>, desk: &Desk, root: &Path) -> Vec<Vec<String>> {
    std::thread::sleep(Duration::from_millis(300));
    harness.run_steps(3);
    let entries = History::open(&desk.data_dir(), root).entries();
    entries.into_iter().map(|entry| entry.args).collect()
}

/// Every file under `dir` with its content.
fn contents(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(contents(&path));
        } else {
            let content = std::fs::read(&path).unwrap();
            files.insert(path, content);
        }
    }
    files
}

/// The model form of Billing in a 13.4 app, filled with `name`, Migration and Policy.
fn model_form_13_4(name: &str) -> (Desk, PathBuf, Harness<'static, MortiseApp>) {
    let (desk, root) = desk();
    common::package_version(&root, "v13.4.0");
    let mut harness = desk.start();
    generate_in(&mut harness, "Billing");
    click(&mut harness, Role::ListBoxOption, "Model");
    type_into(&mut harness, "Model name", name);
    for flag in ["Migration", "Policy"] {
        click(&mut harness, Role::CheckBox, flag);
    }
    (desk, root, harness)
}

fn disabled(harness: &Harness<'static, MortiseApp>, button: &str) -> bool {
    harness
        .get_by_role_and_label(Role::Button, button)
        .accesskit_node()
        .is_disabled()
}

#[test]
fn the_picker_counts_the_generators_and_its_filter_opens_a_form() {
    let (desk, root) = desk();
    let mut harness = desk.start();
    generate_in(&mut harness, "Billing");
    harness.get_by_label("41 generators for Billing, and New module. Enter opens the form.");
    harness.get_by_role_and_label(Role::Heading, "Generate in Billing");
    assert!(
        harness
            .query_by_label("for nwidart/laravel-modules 13.3")
            .is_none()
    );
    // Model is the current row; "Other" is left out while it is empty.
    let model = harness.get_by_role_and_label(Role::ListBoxOption, "Model");
    assert_eq!(model.accesskit_node().toggled(), Some(Toggled::True));
    assert!(harness.query_by_label("Other").is_none());

    // G focuses the filter; it filters by label and artisan name, and Enter opens.
    harness.key_press(Key::G);
    harness.run_steps(3);
    let filter = harness.get_by_role_and_label(Role::TextInput, "Find a generator");
    assert!(filter.is_focused());
    filter.type_text("polic");
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::ListBoxOption, "Policy");
    for gone in ["Model", "Inertia component", "New module"] {
        assert!(
            harness
                .query_by_role_and_label(Role::ListBoxOption, gone)
                .is_none(),
            "{gone} is listed"
        );
    }
    harness.key_press(Key::Enter);
    harness.run_steps(3);
    assert!(
        harness
            .get_by_role_and_label(Role::TextInput, "Policy name")
            .is_focused()
    );
    harness
        .query_by_label_contains("module:make-policy --no-interaction --ansi Billing")
        .unwrap();

    // "All modules": the picker lists New module only.
    click(&mut harness, Role::Button, "Change generator");
    click(&mut harness, Role::ListBoxOption, "All modules");
    harness.get_by_label("Select a module in the index to generate files in it.");
    assert!(
        harness
            .query_by_role_and_label(Role::ListBoxOption, "Model")
            .is_none()
    );

    // Another version of the package: the picker says which one it describes.
    common::package_version(&root, "v13.2.0");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::R);
    common::wait_until(&mut harness, "the package is 13.2.0", |app| {
        app.app().is_some_and(|app| app.package.full() == "13.2.0")
    });
    harness.get_by_label("for nwidart/laravel-modules 13.3");
}

#[test]
fn a_13_4_app_reads_its_generators_with_the_line_on_screen() {
    let (desk, root) = desk();
    common::package_version(&root, "v13.4.0");
    let mut harness = desk.start();
    generate_in(&mut harness, "Billing");
    // The list shipped for 13.4, not marked, and the button with the line it runs.
    harness.get_by_label("42 generators for Billing, and New module. Enter opens the form.");
    assert!(
        harness
            .query_by_label_contains("for nwidart/laravel-modules")
            .is_none()
    );
    harness
        .query_by_label_contains("module:generators --no-interaction --ansi --json")
        .unwrap();

    // The read runs the recording; the picker then lists what the app gave.
    click(
        &mut harness,
        Role::Button,
        "Read the generators of shop-api",
    );
    wait_for(&mut harness, "Read from shop-api at ");
    harness.get_by_role_and_label(Role::Button, "Read again");
    harness.get_by_label("42 generators for Billing, and New module. Enter opens the form.");
    // A read stays out of the history.
    assert!(ran(&mut harness, &desk, &root).is_empty());
}

#[test]
fn the_model_form_shows_its_line_runs_it_and_shows_the_result() {
    let (desk, root) = desk();
    let mut harness = desk.start();
    generate_in(&mut harness, "Billing");
    click(&mut harness, Role::ListBoxOption, "Model");
    harness.get_by_label("The files are listed after the run.");

    // A name that is not a class name: its error, and the button is off.
    type_into(&mut harness, "Model name", "payment intent");
    harness.get_by_label(
        "A model name starts with a letter and has no spaces. Write PaymentIntent, not \
         \"payment intent\".",
    );
    assert!(disabled(&harness, "Create files"));
    type_into(&mut harness, "Model name", "PaymentIntent");
    assert!(!disabled(&harness, "Create files"));
    assert!(
        harness
            .query_by_label_contains("starts with a letter")
            .is_none()
    );

    // A checkbox changes the line; "All of these" gives --all for the seven.
    for flag in ["Migration", "Policy"] {
        click(&mut harness, Role::CheckBox, flag);
    }
    harness
        .query_by_label_contains("--ansi PaymentIntent Billing --migration --policy")
        .unwrap();
    click(&mut harness, Role::CheckBox, "All of these");
    harness
        .query_by_label_contains("--ansi PaymentIntent Billing --all")
        .unwrap();
    assert!(harness.query_by_label_contains("--migration").is_none());
    click(&mut harness, Role::CheckBox, "All of these");
    for flag in ["Migration", "Policy"] {
        click(&mut harness, Role::CheckBox, flag);
    }
    type_into(
        &mut harness,
        "Fillable attributes",
        "order_id,amount,currency,status",
    );
    // A value is one word of the line, quoted for the shell where it needs it.
    harness
        .query_by_label_contains("--fillable=order_id,amount,currency,status")
        .unwrap();

    // Create files runs the line; the result replaces it, with a toast.
    run(&mut harness);
    wait_for(&mut harness, "Created 3 files in Billing.");
    harness.get_by_role_and_label(Role::Heading, "Created in Billing");
    for path in [
        "app/Models/PaymentIntent.php",
        "app/Policies/PaymentIntentPolicy.php",
    ] {
        harness.get_by_role_and_label(Role::Button, &format!("Open {path}"));
    }
    harness.get_by_role_and_label(Role::Button, "Open PaymentIntent.php");
    assert_eq!(
        ran(&mut harness, &desk, &root)[0],
        [
            "module:make-model",
            "PaymentIntent",
            "Billing",
            "--migration",
            "--policy",
            "--fillable=order_id,amount,currency,status"
        ]
    );
    // Generate another: the form again, the name cleared, the rest kept.
    click(&mut harness, Role::Button, "Generate another");
    harness
        .query_by_label_contains("--ansi Billing --migration --policy")
        .unwrap();
    assert!(disabled(&harness, "Create files"));
}

#[test]
fn a_13_4_form_lists_its_files_by_a_dry_run_and_reads_the_answer_of_its_run() {
    let (desk, root, mut harness) = model_form_13_4("PaymentIntent");
    type_into(
        &mut harness,
        "Fillable attributes",
        "order_id,amount,currency,status",
    );
    // The line runs with --json; "List files" shows the line of its dry run.
    harness
        .query_by_label_contains(
            "Billing --migration --policy '--fillable=order_id,amount,currency,status' --json",
        )
        .unwrap();
    harness
        .query_by_label_contains("currency,status' --dry-run --json")
        .unwrap();
    assert!(!disabled(&harness, "Create files"));

    // The dry run lists the files by folder and counts them; it writes nothing and
    // is neither in the drawer nor in the history.
    let before = contents(&root);
    click(&mut harness, Role::Button, "List files");
    wait_for(&mut harness, "3 files, none exist yet");
    harness.get_by_label("Modules/Billing/");
    for path in [
        "app/Models/PaymentIntent.php",
        "app/Policies/PaymentIntentPolicy.php",
        "database/migrations/2026_11_23_110209_create_payment_intents_table.php",
    ] {
        harness.get_by_label(path);
    }
    assert!(!disabled(&harness, "Create 3 files"));
    assert!(harness.query_by_label("List files").is_none());
    assert!(ran(&mut harness, &desk, &root).is_empty());
    assert!(harness.query_by_label("Run again").is_none());
    assert!(contents(&root) == before, "the dry run changed the app");

    // Another name drops the list, also when the name comes back.
    type_into(&mut harness, "Model name", "Payment");
    harness.get_by_role_and_label(Role::Button, "List files");
    assert!(harness.query_by_label("Create 3 files").is_none());
    type_into(&mut harness, "Model name", "PaymentIntent");
    harness.get_by_role_and_label(Role::Button, "Create files");
    // A dry run starts only from a line that was whole on screen (deviation S37).
    let fillable = "order_id,amount,currency,status";
    type_into(&mut harness, "Fillable attributes", &fillable.repeat(200));
    assert!(disabled(&harness, "List files"));
    type_into(&mut harness, "Fillable attributes", fillable);

    // The run carries --json and its result is read from the answer.
    run(&mut harness);
    wait_for(&mut harness, "Created 3 files in Billing.");
    harness.get_by_role_and_label(Role::Heading, "Created in Billing");
    harness.get_by_role_and_label(Role::Button, "Open app/Policies/PaymentIntentPolicy.php");
    assert!(
        harness
            .query_by_label_contains("Read from the output")
            .is_none()
    );
    assert_eq!(
        ran(&mut harness, &desk, &root)[0]
            .last()
            .map(String::as_str),
        Some("--json")
    );
}

#[test]
fn a_dry_run_that_finds_the_policy_takes_it_out_of_the_command() {
    let (_desk, _root, mut harness) = model_form_13_4("Invoice");
    click(&mut harness, Role::Button, "List files");
    // The dry run exits 1 with its whole answer (S9.3).
    wait_for(&mut harness, "3 files, 1 exists already");
    let policy = harness.get_by_role_and_label(Role::CheckBox, "Policy");
    assert_eq!(policy.accesskit_node().toggled(), Some(Toggled::False));
    harness.get_by_label("Policy exists");
    harness.get_by_label("app/Policies/InvoicePolicy.php, exists");
    harness.get_by_label(
        "app/Policies/InvoicePolicy.php exists. A generator does not write over a file, so the \
         policy was taken out of the command. Create it afterwards with \"Replace\" if you want \
         the generated one.",
    );
    harness
        .query_by_label_contains("Invoice Billing --migration --json")
        .unwrap();
    assert!(harness.query_by_label_contains("--policy").is_none());
    assert!(!disabled(&harness, "Create the other 2 files"));
    // Another name drops the list, and the flag it took out comes back.
    type_into(&mut harness, "Model name", "Receipt");
    let policy = harness.get_by_role_and_label(Role::CheckBox, "Policy");
    assert_eq!(policy.accesskit_node().toggled(), Some(Toggled::True));
    harness
        .query_by_label_contains("Receipt Billing --migration --policy --json")
        .unwrap();
    type_into(&mut harness, "Model name", "Invoice");
    click(&mut harness, Role::Button, "List files");
    wait_for(&mut harness, "3 files, 1 exists already");

    // The result lists the policy as left, and Replace writes it with the words of
    // the form before the flag went.
    run(&mut harness);
    wait_for(&mut harness, "Created 2 of 3 files in Billing.");
    harness.get_by_label("app/Policies/InvoicePolicy.php, exists, left as it is");
    // Each left row has its own Replace; the one under the alert is the first's.
    click(
        &mut harness,
        Role::Button,
        "Replace app/Policies/InvoicePolicy.php",
    );
    harness
        .query_by_label_contains(
            "module:make-policy --no-interaction --ansi InvoicePolicy Billing --force --json",
        )
        .unwrap();
}

#[test]
fn a_dry_run_that_finds_the_factory_keeps_it_in_the_command() {
    let (_desk, _root, mut harness) = model_form_13_4("Invoice");
    click(&mut harness, Role::CheckBox, "Factory");
    click(&mut harness, Role::Button, "List files");
    wait_for(&mut harness, "4 files, 2 exist already");
    // The button takes the line once a frame drew it whole.
    harness.run_steps(2);
    // The model's stub uses the factory: its flag stays, the policy's goes, and the
    // button counts the files the run writes.
    let factory = harness.get_by_role_and_label(Role::CheckBox, "Factory");
    assert_eq!(factory.accesskit_node().toggled(), Some(Toggled::True));
    harness.get_by_label("Factory exists");
    harness.get_by_label(
        "database/factories/InvoiceFactory.php exists. The model uses its factory, so the \
         factory stays in the command. The run leaves the file as it is.",
    );
    harness
        .query_by_label_contains("Invoice Billing --factory --migration --json")
        .unwrap();
    assert!(!disabled(&harness, "Create the other 2 files"));

    // The list belongs to the runner and the working folder it ran with: another
    // working folder drops it, and the flag it took out comes back.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::TextInput, "Search or run a command")
        .type_text("php");
    harness.run_steps(3);
    click(&mut harness, Role::ListBoxOption, "Settings: PHP");
    type_into(&mut harness, "Working folder", ".");
    harness.key_press(Key::Escape);
    harness.run_steps(3);
    harness.key_press(Key::Escape);
    harness.run_steps(3);
    assert!(harness.query_by_label("4 files, 2 exist already").is_none());
    harness.get_by_role_and_label(Role::Button, "List files");
    let policy = harness.get_by_role_and_label(Role::CheckBox, "Policy");
    assert_eq!(policy.accesskit_node().toggled(), Some(Toggled::True));

    // A run that fails before it answers, with no file read: the form says why.
    run(&mut harness);
    wait_for(
        &mut harness,
        "Read from the output: the package's answer was ",
    );
}

#[test]
fn a_policy_that_exists_is_left_and_replace_asks_then_runs_force() {
    let (desk, root) = desk();
    // The fake ends once the gate file exists, so that the replacement holds the app.
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
    std::fs::write(&gate, "").unwrap();
    let mut harness = desk.start();
    generate_in(&mut harness, "Billing");
    click(&mut harness, Role::ListBoxOption, "Model");
    type_into(&mut harness, "Model name", "Invoice");
    for flag in ["Migration", "Policy"] {
        click(&mut harness, Role::CheckBox, flag);
    }
    run(&mut harness);
    // The run exits 0, but the result reads the lines: the policy was left.
    wait_for(&mut harness, "Created 2 of 3 files in Billing.");
    harness.get_by_label(
        "Created 2 of 3 files in Billing. InvoicePolicy.php already exists and was left as it is.",
    );
    harness.get_by_label("app/Policies/InvoicePolicy.php, exists, left as it is");
    // No toast while the result and the drawer show it.
    assert!(harness.query_by_label("Show output").is_none());

    click(&mut harness, Role::Button, "Replace InvoicePolicy.php");
    harness.get_by_role_and_label(Role::Heading, "Replace InvoicePolicy.php?");
    harness.get_by_label(
        "The file exists. The generator writes a new one over it, and Mortise cannot bring the \
         old one back.",
    );
    harness
        .query_by_label_contains(
            "module:make-policy --no-interaction --ansi InvoicePolicy Billing --force",
        )
        .unwrap();
    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Cancel")
            .is_focused()
    );
    assert_eq!(ran(&mut harness, &desk, &root).len(), 1);
    std::fs::remove_file(&gate).unwrap();
    click(&mut harness, Role::Button, "Replace file");
    // While the replacement runs, no Replace of the result starts another.
    assert!(disabled(&harness, "Replace InvoicePolicy.php"));
    assert!(disabled(&harness, "Replace app/Policies/InvoicePolicy.php"));
    std::fs::write(&gate, "").unwrap();
    wait_for(&mut harness, "Replaced 1 file in Billing.");
    // The row of the policy changes in the result; the other rows stay.
    harness.get_by_role_and_label(Role::Heading, "Created in Billing");
    harness.get_by_label("app/Policies/InvoicePolicy.php, replaced");
    harness.get_by_label("app/Models/Invoice.php");
    assert_eq!(
        ran(&mut harness, &desk, &root)[0],
        ["module:make-policy", "InvoicePolicy", "Billing", "--force"]
    );
}

#[test]
fn ctrl_n_opens_the_new_module_form_and_replace_asks_first() {
    let (desk, root) = desk();
    // A 13.4 app: the form lists its files and the run carries --json.
    common::package_version(&root, "v13.4.0");
    // The fake ends once the gate file exists, so that the module appears during the run.
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
    let mut harness = desk.start();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run_steps(3);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.run_steps(3);
    assert_eq!(harness.state().selection(), Selection::All);
    assert_eq!(harness.state().tab(), Tab::Generate);
    harness.get_by_role_and_label(Role::Heading, "New module in shop-api");
    assert!(
        harness
            .get_by_role_and_label(Role::TextInput, "Module names")
            .is_focused()
    );

    // The package's sentence for a name it refuses.
    type_into(&mut harness, "Module names", "2fa");
    harness.get_by_label(
        "The module name [2fa] is not valid. Use letters, digits, \"-\", \"_\" or spaces, \
         starting with a letter.",
    );
    assert!(disabled(&harness, "Create module"));

    // The dry run lists the files of the module that does not exist yet (S9.4).
    type_into(&mut harness, "Module names", "Loyalty");
    click(&mut harness, Role::RadioButton, "API");
    std::fs::write(&gate, "").unwrap();
    click(&mut harness, Role::Button, "List files");
    wait_for(&mut harness, "Modules/Loyalty does not exist yet");
    harness.get_by_label("app/Providers/LoyaltyServiceProvider.php");
    harness.get_by_role_and_label(Role::Button, "Create module");
    std::fs::remove_file(&gate).unwrap();

    // Replace asks first, with the line, and nothing runs until "Replace module".
    click(
        &mut harness,
        Role::CheckBox,
        "Replace the module if it exists",
    );
    run(&mut harness);
    harness.get_by_role_and_label(Role::Heading, "Replace the module Loyalty?");
    harness.get_by_label_contains("module:make --force deletes its folder, then generates");
    // The dialog shows the line of the form.
    let line = "module:make --no-interaction --ansi Loyalty --api --force --json";
    assert_eq!(harness.query_all_by_label_contains(line).count(), 2);
    click(&mut harness, Role::Button, "Cancel");
    assert!(ran(&mut harness, &desk, &root).is_empty());

    // Created: the module is selected and its Summary opens.
    click(
        &mut harness,
        Role::CheckBox,
        "Replace the module if it exists",
    );
    run(&mut harness);
    // The module is written while the command runs, as the package does it.
    let loyalty = root.join("Modules/Loyalty");
    common::copy_dir(&root.join("Modules/Catalog"), &loyalty);
    common::rename_in_files(&loyalty, "Catalog", "Loyalty");
    std::fs::write(&gate, "").unwrap();
    wait_for(&mut harness, "Created the module Loyalty.");
    common::wait_until(&mut harness, "Loyalty is selected", |app| {
        app.selection() == Selection::Module("Loyalty".into())
    });
    assert_eq!(harness.state().tab(), Tab::Summary);

    // The footer of the index opens the same form.
    click(&mut harness, Role::Button, "New module");
    assert_eq!(harness.state().selection(), Selection::All);
    harness.get_by_role_and_label(Role::TextInput, "Module names");
}
