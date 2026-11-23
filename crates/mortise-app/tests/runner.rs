//! The runner drawer (S11.1) against the fake artisan: a run, a stop, a failure,
//! the history, and a command that does not start.

mod common;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use common::Desk;
use eframe::egui::{Key, Modifiers, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::MortiseApp;
use mortise_core::RunnerKind;
use mortise_core::history::{Ended, Entry, History};

/// shop-api, known and open, run through `line`, with these runs in its history,
/// the latest first.
fn desk(line: &str, runs: &[&[&str]]) -> (Desk, PathBuf) {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut settings = desk.saved();
    settings.apps[0].php.runner = Some(RunnerKind::Custom {
        line: line.to_owned(),
    });
    settings.save(&desk.settings_path()).unwrap();
    seed(&desk, &root, runs);
    (desk, root)
}

/// The fake artisan, with these words before the artisan arguments.
fn fake(words: &str) -> String {
    format!("\"{}\"{words}", env!("CARGO_BIN_EXE_app-fake-artisan"))
}

/// Writes these runs into the history of the app, the latest first, an hour ago.
fn seed(desk: &Desk, root: &Path, runs: &[&[&str]]) {
    let history = History::open(&desk.data_dir(), root);
    let hour_ago = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
        - 3_600_000;
    for (age, args) in runs.iter().enumerate().rev() {
        let started_ms = hour_ago - age as i64 * 60_000;
        let entry = Entry {
            id: started_ms as u64,
            args: args.iter().map(|word| word.to_string()).collect(),
            label: args.join(" "),
            started_ms,
            ended: Ended::Done { ms: 186 },
            cut: false,
        };
        history.record(entry, &["  seeded".to_owned()]).unwrap();
    }
}

fn chord(harness: &mut Harness<'static, MortiseApp>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run_steps(3);
}

/// How many runs the history keeps, after a run of the fake would have ended.
fn kept_after_a_while(
    harness: &mut Harness<'static, MortiseApp>,
    desk: &Desk,
    root: &Path,
) -> usize {
    std::thread::sleep(Duration::from_millis(300));
    harness.run_steps(3);
    assert!(harness.query_by_label_contains("Running, ").is_none());
    History::open(&desk.data_dir(), root).entries().len()
}

/// Cmd Shift R, or Ctrl Shift R: run the last command again.
fn run_again(harness: &mut Harness<'static, MortiseApp>) {
    chord(harness, Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
}

fn toggle_drawer(harness: &mut Harness<'static, MortiseApp>) {
    chord(harness, Modifiers::CTRL, Key::Backtick);
}

/// Waits until a label holds `text`: a run needs real time to pass.
fn wait_for(harness: &mut Harness<'static, MortiseApp>, text: &str) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while harness.query_by_label_contains(text).is_none() {
        assert!(Instant::now() < deadline, "waited 20 s for: {text}");
        std::thread::sleep(Duration::from_millis(20));
        harness.run_steps(2);
    }
    // The output sticks to its bottom one frame after the last lines came.
    harness.run_steps(3);
}

fn top_of(harness: &Harness<'static, MortiseApp>, role: Role, label: &str) -> f64 {
    let node = harness.get_by_role_and_label(role, label);
    node.accesskit_node().bounding_box().unwrap().y0
}

#[test]
fn a_run_shows_its_output_and_its_time_and_the_drawer_hides() {
    let (desk, root) = desk(&fake(""), &[&["module:migrate", "Blog"]]);
    let mut harness = desk.start();
    // Until it is first shown, the drawer takes no room.
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Runner")
            .is_none()
    );

    // Run again with the drawer hidden opens it on the history, where the line it
    // would run shows, and runs nothing. A second press runs it.
    run_again(&mut harness);
    harness.get_by_role_and_label(Role::Heading, "Runner");
    let line = "module:migrate --no-interaction --ansi Blog";
    harness.query_by_label_contains(line).unwrap();
    assert_eq!(kept_after_a_while(&mut harness, &desk, &root), 1);
    run_again(&mut harness);
    wait_for(&mut harness, "module:migrate Blog: Done in ");
    harness
        .query_by_label_contains("Running Migration Blog Module")
        .unwrap();
    // The header shows the label; the $ line shows the line that runs (S36).
    harness.get_by_label("module:migrate Blog");
    harness.query_by_label_contains(line).unwrap();
    // The run is kept in the history, with its output.
    let entries = History::open(&desk.data_dir(), &root).entries();
    assert_eq!(entries.len(), 2);
    assert!(matches!(entries[0].ended, Ended::Done { .. }));

    // Up on the focused handle makes the drawer 24 px higher.
    let before = top_of(&harness, Role::Splitter, "Height of the runner drawer");
    harness
        .get_by_role_and_label(Role::Splitter, "Height of the runner drawer")
        .focus();
    harness.run_steps(2);
    harness.key_press(Key::ArrowUp);
    harness.run_steps(3);
    let after = top_of(&harness, Role::Splitter, "Height of the runner drawer");
    assert_eq!(before - after, 24.0);

    // Ctrl ` collapses it to its line with Show, and opens it again.
    toggle_drawer(&mut harness);
    harness.get_by_role_and_label(Role::Button, "Show");
    assert!(
        harness
            .query_by_role_and_label(Role::Splitter, "Height of the runner drawer")
            .is_none()
    );
    toggle_drawer(&mut harness);
    harness.get_by_role_and_label(Role::Button, "Hide");

    // Collapsed, Run again opens the drawer on the history and runs nothing.
    toggle_drawer(&mut harness);
    run_again(&mut harness);
    harness.get_by_role_and_label(Role::Button, "Hide");
    harness
        .query_by_label_contains(" today in shop-api")
        .unwrap();
    assert_eq!(kept_after_a_while(&mut harness, &desk, &root), 2);
}

#[test]
fn stop_asks_first_for_a_migration_and_run_again_waits() {
    let (desk, _root) = desk(&fake(" hold"), &[&["module:migrate", "--all"]]);
    let mut harness = desk.start();
    run_again(&mut harness);
    run_again(&mut harness);
    // The output streams before the command ends.
    wait_for(&mut harness, "Running Migration Shop Module");
    harness.query_by_label_contains("Running, ").unwrap();
    harness.get_by_role_and_label(Role::Button, "Stop");

    // An app that runs a command cannot be removed until it ends.
    chord(&mut harness, Modifiers::COMMAND, Key::Comma);
    let remove = harness.get_by_role_and_label(Role::Button, "Remove shop-api from Mortise");
    assert!(remove.accesskit_node().is_disabled());
    remove.hover();
    harness.run_steps(12);
    harness.get_by_label("module:migrate --all is running");
    chord(&mut harness, Modifiers::NONE, Key::Escape);

    // While it runs, Run again in the history is off and says why.
    chord(&mut harness, Modifiers::COMMAND | Modifiers::SHIFT, Key::H);
    let again = harness.get_by_role_and_label(Role::Button, "Run again");
    assert!(again.accesskit_node().is_disabled());
    again.hover();
    harness.run_steps(12);
    harness.get_by_label("module:migrate --all is running");
    chord(&mut harness, Modifiers::COMMAND | Modifiers::SHIFT, Key::H);

    // A migration asks first; Keep running has focus.
    chord(&mut harness, Modifiers::COMMAND, Key::Period);
    harness.get_by_role_and_label(Role::Heading, "Stop module:migrate --all?");
    harness
        .get_by_label("A migration that stops half way can leave the database between two states.");
    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Keep running")
            .is_focused()
    );
    harness
        .get_by_role_and_label(Role::Button, "Stop command")
        .click();
    wait_for(&mut harness, "module:migrate --all: Stopped after ");
}

#[test]
fn a_failure_says_its_exit_code_and_one_sentence() {
    let (desk, _root) = desk(&fake(""), &[&["module:migrate", "NoSuchModule"]]);
    let mut harness = desk.start();
    run_again(&mut harness);
    run_again(&mut harness);
    wait_for(&mut harness, "module:migrate NoSuchModule: Exit code 1");
    harness
        .query_by_label_contains("Application::handleCommand")
        .unwrap();
    harness.get_by_label("module:migrate NoSuchModule stopped with exit code 1.");
    harness.get_by_role_and_label(Role::Button, "Copy output of module:migrate NoSuchModule");
}

#[test]
fn the_history_moves_with_up_and_down_and_enter_runs_the_selected_command() {
    let runs: &[&[&str]] = &[
        &["module:make", "Blog"],
        &["module:seed", "Blog"],
        &["module:make-model", "Post", "Blog"],
    ];
    let (desk, _root) = desk(&fake(""), runs);
    let mut harness = desk.start();
    chord(&mut harness, Modifiers::COMMAND | Modifiers::SHIFT, Key::H);
    harness.get_by_label("3 runs today in shop-api");
    // The list takes focus on its selected row, the latest run.
    let selected = |harness: &Harness<'static, MortiseApp>| {
        let row = harness.query_by(|node| node.is_focused()).unwrap();
        assert_eq!(row.accesskit_node().role(), Role::ListBoxOption);
        row.accesskit_node().label().unwrap()
    };
    assert!(selected(&harness).ends_with(" module:make Blog, 186 ms"));
    // Down, Down, Up selects the second run.
    for key in [Key::ArrowDown, Key::ArrowDown, Key::ArrowUp] {
        harness.key_press(key);
        harness.run_steps(3);
    }
    assert!(selected(&harness).contains("module:seed Blog"));

    harness.key_press(Key::Enter);
    harness.run_steps(3);
    wait_for(&mut harness, "module:seed Blog: Done in ");
    harness
        .query_by_label_contains("Module [Blog] seeded.")
        .unwrap();
}

#[test]
fn a_command_that_does_not_start_offers_the_settings_of_php() {
    let (desk, root) = desk("mortise-no-such-php artisan", &[]);
    let mut harness = desk.start();
    toggle_drawer(&mut harness);
    harness.get_by_label("No command has run yet. Output of generators and actions appears here.");
    drop(harness);

    seed(&desk, &root, &[&["module:make", "Blog"]]);
    let mut harness = desk.start();
    run_again(&mut harness);
    run_again(&mut harness);
    harness.get_by_label("$ mortise-no-such-php artisan module:make --no-interaction --ansi Blog");
    harness.get_by_label(
        "The command did not start: \"mortise-no-such-php: command not found\". Choose how PHP \
         runs for shop-api in Settings.",
    );
    // A run that did not start is not kept.
    assert_eq!(History::open(&desk.data_dir(), &root).entries().len(), 1);

    harness
        .get_by_role_and_label(Role::Button, "Open settings, how PHP runs for shop-api")
        .click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "PHP");
    assert_eq!(
        harness
            .get_by_role_and_label(Role::TextInput, "Run PHP with")
            .value()
            .as_deref(),
        Some("mortise-no-such-php artisan")
    );
}
