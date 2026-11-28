//! Environments and their credentials: in Settings and in the add panel.

mod common;

use common::{Desk, environment, shop_api_actuator, wait_for_answer, wait_until};
use eframe::egui::{Key, Modifiers, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use mortise_app::app::MortiseApp;
use mortise_core::{Answer, Credentials, Environment};

const SECRET: &str = "s3cret-token";

fn open_settings(harness: &mut Harness<'static, MortiseApp>) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run_steps(3);
}

/// Types into a field and leaves it, so that what was typed applies.
fn fill(harness: &mut Harness<'static, MortiseApp>, role: Role, label: &str, text: &str) {
    harness.get_by_role_and_label(role, label).focus();
    harness.run_steps(2);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_role_and_label(role, label).type_text(text);
    harness.run_steps(2);
    harness
        .get_by_role_and_label(Role::TextInput, "Name")
        .focus();
    harness.run_steps(3);
}

/// Waits for the answer of a test that a button started.
fn wait_for_test(harness: &mut Harness<'static, MortiseApp>, label: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        harness.run_steps(2);
        if harness.query_by_label(label).is_some() {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "waited 10 s for: {label}"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// The text of the settings file, to look for a secret in it.
fn settings_text(desk: &Desk) -> String {
    std::fs::read_to_string(desk.settings_path()).unwrap()
}

#[test]
fn a_token_is_kept_in_the_store_and_never_shown_or_written() {
    let stub = shop_api_actuator();
    stub.require_header("authorization", &format!("Bearer {SECRET}"));
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(&[environment("local", &stub)]);
    let mut harness = desk.start();
    wait_for_answer(&mut harness);
    harness.get_by_label("Not authorized");

    open_settings(&mut harness);
    harness.get_by_role_and_label(Role::Heading, "Environments of shop-api");
    harness.get_by_role_and_label(Role::Row, "Environment local");
    harness.get_by_label(
        "Mortise sends no credentials. Right for a local app whose actuator is open.",
    );

    // The kind, then the token. While it is typed it is a password field.
    harness
        .get_by_role_and_label(Role::RadioButton, "Bearer token")
        .click();
    harness.run_steps(3);
    harness.get_by_label("sent as Authorization: Bearer");
    harness.get_by_label(
        "Kept in the credential store of this computer, not in mortise.json and not in the app.",
    );
    assert_eq!(
        desk.saved().apps[0].environments[0].credentials,
        Credentials::Bearer
    );
    assert!(!harness.state().has_secret("local"));
    fill(&mut harness, Role::PasswordInput, "Token", SECRET);

    // After the field loses focus the secret is kept and is not drawn again.
    assert!(harness.state().has_secret("local"));
    harness.get_by_label("Saved. Not shown again.");
    harness.get_by_label("Bearer token, saved");
    assert!(
        harness
            .query_by_role_and_label(Role::PasswordInput, "Token")
            .is_none()
    );
    assert!(harness.query_by_label_contains(SECRET).is_none());
    // The settings file holds the kind and nothing of the secret.
    let text = settings_text(&desk);
    assert!(text.contains(r#""kind": "bearer""#));
    assert!(!text.contains(SECRET));

    // The app is asked with the token from now on.
    wait_until(&mut harness, "Up with the token", |app| {
        matches!(app.runtime().answer.as_deref(), Some(Answer::Answered(_)))
    });
    // Esc leaves the Name field, the next Esc closes Settings.
    harness.key_press(Key::Escape);
    harness.run_steps(3);
    harness.key_press(Key::Escape);
    harness.run_steps(3);
    assert_eq!(harness.get_all_by_label("Up on local").count(), 2);
}

#[test]
fn replace_and_remove_change_the_kept_secret() {
    let stub = shop_api_actuator();
    stub.require_header("x-actuator-key", SECRET);
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(&[Environment {
        credentials: Credentials::Header {
            name: "X-Actuator-Key".into(),
        },
        ..environment("local", &stub)
    }]);
    let mut harness = desk.start();
    open_settings(&mut harness);

    // A custom header: the name is readable, the value is a secret.
    harness.get_by_role_and_label(Role::TextInput, "Header name");
    fill(&mut harness, Role::PasswordInput, "Header value", "wrong");
    harness.get_by_label("Saved. Not shown again.");
    harness.get_by_role_and_label(Role::Button, "Test").click();
    wait_for_test(
        &mut harness,
        &format!("{}/actuator answered 401.", stub.base_url),
    );
    harness.get_by_label("Last answer Not authorized");

    // Replace empties the field for a new value.
    harness
        .get_by_role_and_label(Role::Button, "Replace")
        .click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::PasswordInput, "Header value");
    fill(&mut harness, Role::PasswordInput, "Header value", SECRET);
    harness.get_by_role_and_label(Role::Button, "Test").click();
    wait_for_test(
        &mut harness,
        "Exposed: health, info, mappings, scheduledtasks.",
    );
    harness.get_by_label("Last answer Up");

    // Remove forgets it.
    harness
        .get_by_role_and_label(Role::Button, "Remove")
        .click();
    harness.run_steps(3);
    assert!(!harness.state().has_secret("local"));
    harness.get_by_role_and_label(Role::PasswordInput, "Header value");
    assert!(!settings_text(&desk).contains(SECRET));
}

#[test]
fn http_basic_keeps_the_user_name_readable() {
    let stub = shop_api_actuator();
    // base64 of "mortise:s3cret-token"
    stub.require_header("authorization", "Basic bW9ydGlzZTpzM2NyZXQtdG9rZW4=");
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(&[environment("local", &stub)]);
    let mut harness = desk.start();
    open_settings(&mut harness);

    harness
        .get_by_role_and_label(Role::RadioButton, "HTTP basic")
        .click();
    harness.run_steps(3);
    fill(&mut harness, Role::TextInput, "User name", "mortise");
    fill(&mut harness, Role::PasswordInput, "Password", SECRET);
    assert_eq!(
        desk.saved().apps[0].environments[0].credentials,
        Credentials::Basic {
            user: "mortise".into()
        }
    );
    let text = settings_text(&desk);
    assert!(text.contains("mortise"));
    assert!(!text.contains(SECRET));
    harness.get_by_role_and_label(Role::Button, "Test").click();
    wait_for_test(
        &mut harness,
        "Exposed: health, info, mappings, scheduledtasks.",
    );

    // Another kind has no use for the password: it is forgotten.
    harness
        .get_by_role_and_label(Role::RadioButton, "None")
        .click();
    harness.run_steps(3);
    assert!(!harness.state().has_secret("local"));
}

#[test]
fn environments_are_added_renamed_and_removed() {
    let stub = shop_api_actuator();
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(&[Environment {
        credentials: Credentials::Bearer,
        ..environment("local", &stub)
    }]);
    let mut harness = desk.start();
    open_settings(&mut harness);
    fill(&mut harness, Role::PasswordInput, "Token", SECRET);
    assert!(harness.state().has_secret("local"));

    // A second environment gets the next name and is selected, with an empty form.
    harness
        .get_by_role_and_label(Role::Button, "Add environment")
        .click();
    harness.run_steps(4);
    harness.get_by_role_and_label(Role::Row, "Environment staging");
    fill(
        &mut harness,
        Role::TextInput,
        "Base URL",
        "https://staging.shop-api.example",
    );
    let saved = desk.saved();
    assert_eq!(saved.apps[0].environments.len(), 2);
    assert_eq!(
        saved.apps[0].environments[1].base_url,
        "https://staging.shop-api.example"
    );
    // The chosen environment stays the first one.
    assert_eq!(saved.apps[0].environment.as_deref(), Some("local"));

    // A name that another environment has is not taken.
    fill(&mut harness, Role::TextInput, "Environment name", "local");
    assert_eq!(desk.saved().apps[0].environments[1].name, "staging");

    // A rename moves the secret and the choice with the name.
    harness
        .get_by_role_and_label(Role::Row, "Environment local")
        .click();
    harness.run_steps(3);
    fill(&mut harness, Role::TextInput, "Environment name", "herd");
    let saved = desk.saved();
    assert_eq!(saved.apps[0].environments[0].name, "herd");
    assert_eq!(saved.apps[0].environment.as_deref(), Some("herd"));
    assert!(harness.state().has_secret("herd"));
    assert!(!harness.state().has_secret("local"));

    // Removing the environment forgets its secret, and the other one is chosen.
    harness
        .get_by_role_and_label(Role::Button, "Remove environment")
        .click();
    harness.run_steps(4);
    let saved = desk.saved();
    assert_eq!(saved.apps[0].environments.len(), 1);
    assert_eq!(saved.apps[0].environment.as_deref(), Some("staging"));
    assert!(!harness.state().has_secret("herd"));
}

#[test]
fn removing_an_app_forgets_its_saved_credentials() {
    let stub = shop_api_actuator();
    let desk = Desk::new();
    let shop = desk.app("shop-api");
    let crm = desk.app("crm");
    desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
    desk.connect(&[Environment {
        credentials: Credentials::Bearer,
        ..environment("local", &stub)
    }]);
    let mut harness = desk.start();
    open_settings(&mut harness);
    fill(&mut harness, Role::PasswordInput, "Token", SECRET);
    assert!(harness.state().has_secret("local"));

    harness
        .get_by_role_and_label(Role::Button, "Remove shop-api from Mortise")
        .click();
    harness.run_steps(3);
    harness.get_by_label(
        "The folder and its files stay as they are. Mortise forgets the app, its environments \
         and its saved credentials.",
    );
    harness
        .get_by_role_and_label(Role::Button, "Remove app")
        .click();
    harness.run_steps(4);
    assert_eq!(harness.state().app_names(), ["crm"]);

    assert!(!settings_text(&desk).contains(SECRET));

    // The same folder added again starts without a secret.
    desk.picks.borrow_mut().push_back(shop);
    harness.key_press(Key::Escape);
    harness.run_steps(3);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::Button, "Add shop-api")
        .click();
    harness.run_steps(3);
    wait_until(&mut harness, "the app is read", |app| app.is_read());
    assert_eq!(harness.state().app_names(), ["crm", "shop-api"]);
    assert_eq!(harness.state().current(), Some(1));
    assert!(!harness.state().has_secret("local"));
}

#[test]
fn the_add_panel_connects_the_app_with_the_url_from_env() {
    let stub = shop_api_actuator();
    stub.require_header("authorization", &format!("Bearer {SECRET}"));
    let desk = Desk::new();
    let root = desk.app("booking-engine");
    std::fs::write(
        root.join(".env"),
        format!(
            "APP_NAME=Booking\nAPP_ENV=local\nAPP_URL={}\nDB_PASSWORD=never-read\n",
            stub.base_url
        ),
    )
    .unwrap();
    desk.picks.borrow_mut().push_back(root);
    let mut harness = desk.start();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run_steps(3);

    // The section is there because the lock file has the actuator. The URL is proposed.
    harness.get_by_role_and_label(Role::Heading, "Runtime, optional");
    harness.get_by_label("from APP_URL in .env");
    harness.get_by_label("what the app's middleware asks for");

    // Without credentials the app refuses; Test says so before the app is added.
    harness.get_by_role_and_label(Role::Button, "Test").click();
    wait_for_test(
        &mut harness,
        &format!("{}/actuator answered 401.", stub.base_url),
    );

    harness
        .get_by_role_and_label(Role::RadioButton, "Bearer token")
        .click();
    harness.run_steps(3);
    harness
        .get_by_role_and_label(Role::PasswordInput, "Token")
        .focus();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(Role::PasswordInput, "Token")
        .type_text(SECRET);
    harness.run_steps(2);
    harness.get_by_role_and_label(Role::Button, "Test").click();
    wait_for_test(
        &mut harness,
        "Exposed: health, info, mappings, scheduledtasks.",
    );
    assert!(harness.query_by_label_contains(SECRET).is_none());

    // Adding the app keeps the environment in the settings and the token in the store.
    harness
        .get_by_role_and_label(Role::Button, "Add booking-engine")
        .click();
    harness.run_steps(3);
    wait_until(&mut harness, "the app is read", |app| app.is_read());
    wait_for_answer(&mut harness);
    assert!(harness.state().has_secret("local"));
    let saved = desk.saved();
    assert_eq!(
        saved.apps[0].environments,
        [Environment {
            name: "local".into(),
            base_url: stub.base_url.clone(),
            path: "actuator".into(),
            credentials: Credentials::Bearer,
        }]
    );
    assert!(!settings_text(&desk).contains(SECRET));
    assert_eq!(harness.get_all_by_label("Up on local").count(), 2);
    // Nothing but the actuator was asked, and never env.
    assert!(
        stub.asked()
            .iter()
            .all(|path| path.starts_with("/actuator") && !path.contains("env"))
    );
}

#[test]
fn the_add_panel_adds_an_app_without_runtime() {
    // Without APP_URL the base URL is empty, and the app is added without an environment.
    let desk = Desk::new();
    let root = desk.app("booking-engine");
    desk.picks.borrow_mut().push_back(root);
    let mut harness = desk.start();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Runtime, optional");
    assert!(harness.query_by_label("from APP_URL in .env").is_none());
    harness
        .get_by_role_and_label(Role::Button, "Add booking-engine")
        .click();
    harness.run_steps(3);
    wait_until(&mut harness, "the app is read", |app| app.is_read());
    assert!(desk.saved().apps[0].environments.is_empty());
    harness.get_by_label_contains("No environment is connected.");

    // An app without the actuator has no runtime section at all.
    let desk = Desk::new();
    let root = desk.app("plain");
    let lock = std::fs::read_to_string(root.join("composer.lock")).unwrap();
    std::fs::write(
        root.join("composer.lock"),
        lock.replace("laravel-actuator", "laravel-other"),
    )
    .unwrap();
    desk.picks.borrow_mut().push_back(root);
    let mut harness = desk.start();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run_steps(3);
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Runtime, optional")
            .is_none()
    );
}
