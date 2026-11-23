//! Apps: the first run, adding an app and what can go wrong, the rail, and live refresh.

mod common;

use common::{Desk, shop_api, wait_until};
use eframe::egui::{Key, Modifiers, Role, vec2};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::theme::size;
use mortise_core::{RunnerKind, Settings};

const COMMAND_ALT: Modifiers = Modifiers {
    alt: true,
    command: true,
    ..Modifiers::NONE
};

#[test]
fn the_first_run_has_no_apps_and_one_action() {
    let desk = Desk::new();
    let mut harness = desk.start();

    harness.get_by_role_and_label(Role::Heading, "No apps yet");
    harness.get_by_label("No app");
    assert_eq!(harness.query_all_by_role(Role::ListBoxOption).count(), 0);

    harness
        .get_by_role_and_label(Role::Button, "Add app folder")
        .click();
    harness.run();
    assert!(harness.state().is_adding());
    harness.get_by_role_and_label(Role::Heading, "Add app");
    harness.get_by_label("the folder that holds artisan");
    harness.get_by_role_and_label(Role::Button, "Choose folder");
    // Focus is in the path field.
    assert!(harness.get_by_role(Role::TextInput).is_focused());
}

#[test]
fn the_add_tile_opens_the_panel_and_esc_closes_it() {
    let (_desk, mut harness) = shop_api();
    harness
        .get_by_role_and_label(Role::Button, "Add app")
        .click();
    harness.run();
    assert!(harness.state().is_adding());
    harness.get_by_role_and_label(Role::Heading, "Add app");

    harness.key_press(Key::Escape);
    harness.run();
    assert!(!harness.state().is_adding());
    harness.get_by_role_and_label(Role::Heading, "All modules");
}

#[test]
fn adding_an_app_with_the_keyboard_takes_three_actions() {
    let desk = Desk::new();
    let root = desk.app("booking-engine");
    desk.picks.borrow_mut().push_back(root.clone());
    let mut harness = desk.start();

    // 1. Ctrl O opens the folder dialog at once. 2. The folder is chosen there.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run();
    assert!(harness.state().is_adding());
    harness.get_by_role_and_label(Role::Heading, "Found in this folder");
    for found in [
        "13.2",
        " composer.lock",
        "13.3.0",
        "7 in ",
        "Modules/",
        ", 7 on",
        "1.0.0",
        "booking-engine",
        "BE",
    ] {
        harness.get_by_label(found);
    }
    // At 960 by 700 the note of the box goes under its title, and the box grows: its
    // last line, Git, ends 8 px above its bottom edge, 12 px above the next box.
    harness.set_size(vec2(960.0, 700.0));
    harness.run();
    let git = harness.get_by_label("Git").rect();
    let next = harness
        .get_by_role_and_label(Role::Heading, "Shown in the rail as")
        .rect();
    let line_bottom = git.center().y + size::LIST_LINE_H / 2.0;
    assert!(line_bottom + 8.0 + 12.0 <= next.top(), "{git:?} {next:?}");
    harness.set_size(vec2(1280.0, 800.0));
    harness.run();
    // Focus lands on the button that adds.
    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Add booking-engine")
            .is_focused()
    );

    // 3. Enter adds the app.
    harness.key_press(Key::Enter);
    wait_until(&mut harness, "the new app is read", |app| {
        app.app().is_some()
    });

    assert!(!harness.state().is_adding());
    assert_eq!(harness.state().app_names(), ["booking-engine"]);
    assert_eq!(harness.state().current(), Some(0));
    harness.get_by_label("Added booking-engine. 7 modules read.");
    harness.get_by_role_and_label(Role::Heading, "booking-engine");
    harness.get_by_role_and_label(Role::ListBoxOption, "All modules");

    // The app is kept for the next run, with its monogram and the codes of its modules.
    let saved = desk.saved();
    assert_eq!(saved.apps.len(), 1);
    assert_eq!(saved.apps[0].root, root);
    assert_eq!(saved.apps[0].monogram, "BE");
    assert_eq!(saved.apps[0].codes["Checkout"], "CHK");
    assert_eq!(saved.current, Some(0));
}

#[test]
fn ctrl_enter_adds_too_and_the_toast_leaves_on_esc() {
    let desk = Desk::new();
    desk.picks.borrow_mut().push_back(desk.app("crm"));
    let mut harness = desk.start();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    wait_until(&mut harness, "the new app is read", |app| {
        app.app().is_some()
    });
    harness.get_by_label("Added crm. 7 modules read.");

    harness.key_press(Key::Escape);
    harness.run_steps(3);
    assert!(
        harness
            .query_by_label("Added crm. 7 modules read.")
            .is_none()
    );
}

#[test]
fn a_path_can_be_typed_into_the_field() {
    let desk = Desk::new();
    let root = desk.app("crm");
    let mut harness = desk.start();

    harness
        .get_by_role_and_label(Role::Button, "Add app")
        .click();
    harness.run();
    harness
        .get_by_role(Role::TextInput)
        .type_text(root.to_str().unwrap());
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Add crm");
    harness.get_by_role_and_label(Role::Heading, "Found in this folder");

    // Another path drops what was found for crm: Add acts on the folder the field shows.
    let pages = desk.app("landing-pages");
    harness
        .get_by_role_and_label(Role::TextInput, "App folder")
        .focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "App folder")
        .type_text(pages.to_str().unwrap());
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Add crm")
            .is_none()
    );
    harness.key_press(Key::Enter);
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Add landing-pages")
        .click();
    wait_until(&mut harness, "the new app is read", |app| {
        app.app().is_some()
    });
    assert_eq!(desk.saved().apps[0].root, pages);
}

#[test]
fn a_folder_that_is_not_a_laravel_app_cannot_be_added() {
    let desk = Desk::new();
    let notes = desk.path().join("notes");
    std::fs::create_dir(&notes).unwrap();
    desk.picks.borrow_mut().push_back(notes);
    let mut harness = desk.start();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run();
    harness.get_by_label_contains("The folder has no artisan file and no composer.json");
    harness.get_by_label_contains("Choose the folder that holds artisan.");
    // The add button is there, disabled, beside the add tile of the rail.
    assert_eq!(
        harness
            .get_all_by_role_and_label(Role::Button, "Add app")
            .count(),
        2
    );
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.run();
    assert!(harness.state().is_adding());
    assert!(harness.state().app_names().is_empty());

    harness
        .get_by_role_and_label(Role::Button, "Cancel")
        .click();
    harness.run();
    assert!(!harness.state().is_adding());
}

#[test]
fn an_app_whose_php_does_not_answer_is_added_read_only() {
    let desk = Desk::new();
    desk.picks
        .borrow_mut()
        .push_back(desk.app("booking-engine"));
    let mut harness = desk.start();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run();
    // Nothing in the folder points to a runner: Herd is shown, and the hint says so.
    harness.get_by_label("Not found: Herd, Sail, a compose file.");
    let command = harness.get_by_role_and_label(Role::TextInput, "Run PHP with");
    assert_eq!(
        command.accesskit_node().value(),
        Some(mortise_core::RunnerKind::Herd.line())
    );

    harness
        .get_by_role_and_label(Role::RadioButton, "Custom")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Run PHP with")
        .focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, "Run PHP with")
        .type_text("mortise-no-such-php artisan");
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Test how PHP runs")
        .click();
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

    harness.get_by_role_and_label(Role::Button, "Add booking-engine, read only");

    // The result belongs to the runner it tested: with another the app is not read only.
    harness
        .get_by_role_and_label(Role::RadioButton, "Docker")
        .click();
    harness.run();
    assert!(harness.query_by_label_contains(failed).is_none());

    // The app is added with the runner the field shows.
    harness
        .get_by_role_and_label(Role::Button, "Add booking-engine")
        .click();
    wait_until(&mut harness, "the new app is read", |app| {
        app.app().is_some()
    });
    assert_eq!(
        desk.saved().apps[0].php.runner,
        Some(RunnerKind::Docker {
            service: "app".to_owned()
        })
    );
    // Settings shows the test of the add panel as the last test of the app.
    // The toast of the new app still moves, so a fixed number of steps.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run_steps(6);
    harness
        .get_by_role_and_label(Role::ListBoxOption, "PHP")
        .click();
    harness.run_steps(6);
    harness.get_by_label("No answer: mortise-no-such-php: command not found");
}

#[test]
fn an_app_without_the_package_gets_the_command_and_can_be_checked_again() {
    let desk = Desk::new();
    let root = desk.app("booking-engine");
    let lock = root.join("composer.lock");
    let with_package = std::fs::read_to_string(&lock).unwrap();
    std::fs::write(
        &lock,
        r#"{"packages": [{"name": "laravel/framework", "version": "v13.2.0"}]}"#,
    )
    .unwrap();
    desk.picks.borrow_mut().push_back(root);
    let mut harness = desk.start();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run();
    harness.get_by_label_contains("booking-engine does not use nwidart/laravel-modules.");
    harness.get_by_label("composer require nwidart/laravel-modules");

    harness
        .get_by_role_and_label(Role::Button, "Copy command")
        .click();
    harness.run_steps(3);
    harness.get_by_label("Copied the command.");

    // The package is installed, and the folder is read again.
    std::fs::write(&lock, with_package).unwrap();
    harness
        .get_by_role_and_label(Role::Button, "Check again")
        .click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Button, "Add booking-engine");
}

#[test]
fn an_app_without_a_lock_file_says_what_to_run() {
    let desk = Desk::new();
    let root = desk.app("booking-engine");
    std::fs::remove_file(root.join("composer.lock")).unwrap();
    desk.picks.borrow_mut().push_back(root);
    let mut harness = desk.start();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run();
    harness
        .get_by_label_contains("Mortise reads versions from it. Run composer install in the app");
}

#[test]
fn an_app_that_is_already_there_is_opened_not_added() {
    let desk = Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
    desk.picks.borrow_mut().push_back(crm);
    let mut harness = desk.start();
    assert_eq!(harness.state().current(), Some(0));

    // From Settings: the app that opens takes the place of Settings.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Back to shop-api");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run();
    harness.get_by_label("crm is already in Mortise. Press Enter to open it.");

    harness.key_press(Key::Enter);
    wait_until(&mut harness, "crm is on screen", |app| {
        app.current() == Some(1)
    });
    assert!(!harness.state().is_adding());
    assert!(harness.query_by_label_contains("Back to").is_none());
    assert_eq!(harness.state().app_names(), ["shop-api", "crm"]);
}

#[test]
fn a_folder_on_the_command_line_opens_the_panel_or_the_app() {
    let desk = Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    desk.knows(&[(&shop, "SA"), (&crm, "CR")]);

    // An app Mortise knows is opened.
    let harness = desk.start_with(Some(crm));
    assert!(!harness.state().is_adding());
    assert_eq!(harness.state().current(), Some(1));

    // A new folder is shown in the add panel first.
    let harness = desk.start_with(Some(desk.app("landing-pages")));
    assert!(harness.state().is_adding());
    harness.get_by_role_and_label(Role::Button, "Add landing-pages");

    // At 150% text size in a low window the panel scrolls to the control that gets
    // focus: Tab from the path field reaches Add, and Add is in the window.
    let mut settings = desk.saved();
    settings.general.text_size = 150;
    settings.save(&desk.settings_path()).unwrap();
    let mut desk = desk;
    desk.size = vec2(900.0, 360.0);
    let mut harness = desk.start_with(Some(desk.app("booking-engine")));
    let window = harness.ctx.content_rect();
    harness
        .get_by_role_and_label(Role::TextInput, "App folder")
        .focus();
    harness.run_steps(3);
    let add = harness.get_by_role_and_label(Role::Button, "Add booking-engine");
    assert!(
        !window.contains_rect(add.rect()),
        "the button is below the fold"
    );
    for _ in 0..40 {
        if harness
            .get_by_role_and_label(Role::Button, "Add booking-engine")
            .is_focused()
        {
            break;
        }
        harness.key_press(Key::Tab);
        harness.run_steps(3);
    }
    let add = harness.get_by_role_and_label(Role::Button, "Add booking-engine");
    assert!(add.is_focused());
    assert!(window.contains_rect(add.rect()));

    // A name longer than the panel is wide is cut in the label: Cancel stays in the panel.
    let long = "booking-engine-for-the-regional-warehouses-of-the-northern-and-western-partners";
    let harness = desk.start_with(Some(desk.app(long)));
    let label = format!("Add {long}");
    let add = harness.get_by_role_and_label(Role::Button, &label).rect();
    let cancel = harness.get_by_role_and_label(Role::Button, "Cancel").rect();
    assert!(add.right() < cancel.left());
    assert!(cancel.right() <= window.right(), "{cancel:?}");
}

#[test]
fn the_rail_selects_apps_by_click_and_by_key() {
    let desk = Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    let pages = desk.app("landing-pages");
    // Another folder with the same name.
    let other_crm = desk.app("elsewhere/crm");
    desk.knows(&[
        (&shop, "SA"),
        (&crm, "CR"),
        (&pages, "LP"),
        (&other_crm, "C2"),
    ]);
    let mut harness = desk.start();

    // The title bar names the app on screen.
    assert_eq!(harness.state().current(), Some(0));
    assert_eq!(
        harness.get_all_by_label("shop-api").count(),
        3,
        "tile, title bar, index head"
    );

    harness.key_press_modifiers(COMMAND_ALT, Key::Num3);
    wait_until(&mut harness, "landing-pages is on screen", |app| {
        app.current() == Some(2)
    });
    harness.get_by_role_and_label(Role::Heading, "landing-pages");

    // Two apps with the same name are two tiles, each of its own app.
    let tiles: Vec<_> = harness
        .get_all_by_role_and_label(Role::ListBoxOption, "crm")
        .collect();
    assert_eq!(tiles.len(), 2);
    tiles[1].click();
    wait_until(&mut harness, "the other crm is on screen", |app| {
        app.current() == Some(3)
    });
    harness
        .get_all_by_role_and_label(Role::ListBoxOption, "crm")
        .next()
        .unwrap()
        .click();
    wait_until(&mut harness, "crm is on screen", |app| {
        app.current() == Some(1)
    });
    harness.get_by_role_and_label(Role::Heading, "crm");

    // There is no ninth app: nothing happens.
    harness.key_press_modifiers(COMMAND_ALT, Key::Num9);
    harness.run();
    assert_eq!(harness.state().current(), Some(1));

    // The app on screen is the one opened at the next start.
    assert_eq!(desk.saved().current, Some(1));

    // The selected module and tab are kept per app.
    harness.key_press(Key::J);
    harness.key_press(Key::Num2);
    harness.run();
    harness.key_press_modifiers(COMMAND_ALT, Key::Num1);
    wait_until(&mut harness, "shop-api is on screen", |app| {
        app.current() == Some(0)
    });
    harness.get_by_role_and_label(Role::Heading, "All modules");
    harness.key_press_modifiers(COMMAND_ALT, Key::Num2);
    wait_until(&mut harness, "crm is on screen", |app| {
        app.current() == Some(1)
    });
    harness.get_by_role_and_label(Role::Heading, "Billing");
}

#[test]
fn a_change_on_disk_shows_without_asking() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();
    harness.get_by_label("7 of 7 on");
    wait_until(&mut harness, "the watcher is in place", |app| {
        app.is_watching()
    });
    // The operating system needs a moment before it reports changes.
    std::thread::sleep(std::time::Duration::from_millis(300));

    common::turn_off(&root, "Checkout");
    wait_until(&mut harness, "Checkout is off", |app| {
        app.app().is_some_and(|app| app.modules_on() == 6)
    });
    harness.get_by_label("6 of 7 on");
}

#[test]
fn a_module_that_is_removed_while_selected_gives_way_to_all_modules() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();

    harness.key_press(Key::End);
    harness.run();
    harness.get_by_role_and_label(Role::Heading, "Users");

    std::fs::remove_dir_all(root.join("Modules/Users")).unwrap();
    // Ctrl R reads the app again, with or without the watcher.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::R);
    wait_until(&mut harness, "six modules", |app| {
        app.app().is_some_and(|app| app.modules.len() == 6)
    });
    harness.get_by_role_and_label(Role::Heading, "All modules");
    let rows = harness
        .get_all_by_role(Role::ListBoxOption)
        .filter_map(|row| row.accesskit_node().label())
        .filter(|label| label == "Users")
        .count();
    assert_eq!(rows, 0);
}

#[test]
fn settings_that_cannot_be_read_are_kept_aside() {
    let desk = Desk::new();
    let path = desk.settings_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "{ this is not JSON").unwrap();

    let mut harness = desk.start();
    harness.run_steps(2);
    harness.get_by_role_and_label(Role::Heading, "No apps yet");
    harness.get_by_label_contains("Mortise could not read its settings");
    let kept = path.with_extension("invalid.json");
    assert_eq!(std::fs::read_to_string(kept).unwrap(), "{ this is not JSON");
    // A failed toast stays until it is closed.
    harness
        .get_by_role_and_label(Role::Button, "Close the message")
        .click();
    harness.run_steps(2);
    assert!(
        harness
            .query_by_label_contains("Mortise could not read its settings")
            .is_none()
    );
    assert_eq!(Settings::load(&path).unwrap(), Settings::default());
}

#[test]
fn a_large_folder_is_read_without_stopping_the_window_and_says_how_far_it_is() {
    let mut desk = Desk::new();
    // The window does not wait for the folder at all.
    desk.quick_read = std::time::Duration::ZERO;
    let root = desk.app("platform");
    // Enough files that the read takes longer than a moment.
    common::pad_module(&root, "Billing", 6000);
    desk.picks.borrow_mut().push_back(root);
    let mut harness = desk.start();

    let started = std::time::Instant::now();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run_steps(2);
    // The window answers while the folder is read: the panel is there, and nothing
    // can be added yet.
    assert!(harness.state().is_adding());
    harness.get_by_role_and_label(Role::Heading, "Add app");
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Add platform")
            .is_none()
    );

    // The bar is drawn only after 300 ms.
    let mut bar_seen_at = None;
    let deadline = started + std::time::Duration::from_secs(30);
    while harness
        .query_by_role_and_label(Role::Button, "Add platform")
        .is_none()
    {
        assert!(
            std::time::Instant::now() < deadline,
            "the folder was not read in 30 s"
        );
        // The text beside the bar is a node of its own with the same words, so ask the bar.
        if bar_seen_at.is_none()
            && let Some(bar) = harness.query_by_role(Role::ProgressIndicator)
        {
            let label = bar.accesskit_node().label().unwrap_or_default();
            assert!(label.ends_with(" of 6,077 files"), "{label}");
            bar_seen_at = Some(started.elapsed());
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        harness.step();
    }
    println!(
        "read in {:?}, bar first seen at {bar_seen_at:?}",
        started.elapsed()
    );
    if let Some(at) = bar_seen_at {
        assert!(at >= std::time::Duration::from_millis(300), "{at:?}");
    }
    harness.get_by_label("nwidart/laravel-modules");
}
