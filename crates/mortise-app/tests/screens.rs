//! Renders screens to `target/screens`, to be looked at beside the drawings of the
//! specification. Run with `cargo test -p mortise-app --test screens -- --ignored`.

mod common;

use std::path::Path;

use common::{Desk, wait_until};
use eframe::egui::{self, Key, Modifiers, Role, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use mortise_app::app::MortiseApp;
use mortise_app::app::chrome::Chrome;

fn save(name: &str, harness: &mut Harness<'static, MortiseApp>) {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screens");
    std::fs::create_dir_all(&out).unwrap();
    harness.run_steps(3);
    harness
        .render()
        .unwrap()
        .save(out.join(format!("{name}.png")))
        .unwrap();
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_1() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut desk = Desk::new();
        desk.theme = theme;
        desk.chrome = Chrome::Mac;
        let root = desk.app("shop-api");
        common::turn_off(&root, "Notifications");
        desk.knows(&[(&root, "SA")]);
        let mut harness = desk.start();
        save(&format!("s3.1-all-mac-{suffix}"), &mut harness);

        desk.chrome = Chrome::Custom {
            controls_left: false,
        };
        let mut harness = desk.start();
        harness.key_press(Key::J);
        harness.key_press(Key::J);
        harness.key_press(Key::J);
        save(&format!("s3.1-module-custom-{suffix}"), &mut harness);
        harness.key_press(Key::Num5);
        save(&format!("s7.4-metrics-{suffix}"), &mut harness);
    }
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_2() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        // S1.1, S1.2: the first run and the empty add panel.
        let mut desk = Desk::new();
        desk.theme = theme;
        let mut harness = desk.start();
        save(&format!("s1.1-first-run-{suffix}"), &mut harness);
        harness
            .get_by_role_and_label(Role::Button, "Add app folder")
            .click();
        harness.key_press(Key::Tab);
        harness.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
        save(&format!("s1.2-add-app-{suffix}"), &mut harness);

        // S1.3 and S2.1: three apps on the rail, a fourth found.
        let desk = {
            let mut desk = Desk::new();
            desk.theme = theme;
            desk
        };
        let apps = [
            desk.app("shop-api"),
            desk.app("crm"),
            desk.app("landing-pages"),
        ];
        desk.knows(&[(&apps[0], "SA"), (&apps[1], "CR"), (&apps[2], "LP")]);
        desk.picks
            .borrow_mut()
            .push_back(desk.app("booking-engine"));
        let mut harness = desk.start();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
        save(&format!("s1.3-found-{suffix}"), &mut harness);
        harness.key_press(Key::Enter);
        wait_until(&mut harness, "the new app is read", |app| {
            app.current() == Some(3) && app.app().is_some()
        });
        save(&format!("f1-added-toast-{suffix}"), &mut harness);
    }

    // S1.4: the failures.
    let desk = Desk::new();
    let notes = desk.path().join("notes");
    std::fs::create_dir(&notes).unwrap();
    let no_package = desk.app("booking-engine");
    std::fs::write(no_package.join("composer.lock"), r#"{"packages": []}"#).unwrap();
    let shop = desk.app("shop-api");
    desk.knows(&[(&shop, "SA")]);
    desk.picks.borrow_mut().extend([notes, no_package, shop]);
    let mut harness = desk.start();
    for name in [
        "s1.4a-not-laravel",
        "s1.4b-no-package",
        "s1.4-already-added",
    ] {
        harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
        save(&format!("{name}-light"), &mut harness);
    }

    // S2.1: 10 apps fit; 16 do not, at the minimum window height.
    for (count, size) in [
        (10, egui::vec2(1280.0, 800.0)),
        (16, egui::vec2(960.0, 600.0)),
    ] {
        let mut desk = Desk::new();
        desk.size = size;
        let names = [
            "shop-api",
            "crm",
            "landing-pages",
            "booking-engine",
            "ledger",
            "assets",
            "platform",
            "design-system",
            "blog",
            "partner-api",
            "search",
            "billing",
            "warehouse",
            "intranet",
            "status-page",
            "docs",
        ];
        let roots: Vec<_> = names[..count].iter().map(|name| desk.app(name)).collect();
        let monograms: Vec<String> = names[..count]
            .iter()
            .map(|name| name.chars().take(2).collect::<String>().to_uppercase())
            .collect();
        let known: Vec<(&Path, &str)> = roots
            .iter()
            .zip(&monograms)
            .map(|(root, monogram)| (root.as_path(), monogram.as_str()))
            .collect();
        desk.knows(&known);
        let mut harness = desk.start();
        save(&format!("s2.1-{count}-apps-light"), &mut harness);
    }
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_3() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        common::turn_off(&root, "Notifications");
        common::with_git_history(&root);
        desk.knows(&[(&root, "SA")]);
        let mut harness = desk.start();
        // S4.2: the Summary of "All modules".
        save(&format!("s4.2-all-{suffix}"), &mut harness);
        // S4.3: the Summary of Checkout.
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Checkout")
            .click();
        save(&format!("s4.3-checkout-{suffix}"), &mut harness);
    }

    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();
    // S3.1: filtered, and a filter with no match.
    harness.key_press(Key::Slash);
    harness.run_steps(2);
    harness
        .get_by_role_and_label(Role::TextInput, "Filter modules")
        .type_text("ch");
    save("s3.1-filtered-light", &mut harness);
    harness
        .get_by_role_and_label(Role::TextInput, "Filter modules")
        .type_text("x");
    save("s3.1-no-match-light", &mut harness);
    harness.key_press(Key::Escape);
    harness.key_press(Key::Escape);
    // S13.1 a: the index as a strip.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::B);
    harness.key_press(Key::J);
    harness.key_press(Key::J);
    harness.key_press(Key::J);
    save("s13.1-strip-light", &mut harness);

    // S3.1: 30 modules, one line a row, and the table scrolls.
    let mut desk = Desk::new();
    desk.size = egui::vec2(1100.0, 700.0);
    let root = common::app_with_modules(&desk, "platform", 30);
    desk.knows(&[(&root, "PL")]);
    let mut harness = desk.start();
    harness.key_press(Key::J);
    save("s3.1-30-modules-light", &mut harness);
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_4() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        let mut harness = desk.start();
        // S5.3: the Boundary of "All modules".
        harness.key_press(Key::Num2);
        save(&format!("s5.3-all-{suffix}"), &mut harness);
        // S5.1: the Boundary of Checkout, on its link with Billing.
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Checkout")
            .click();
        save(&format!("s5.1-checkout-{suffix}"), &mut harness);
        // S5.2: the circle with Shipping.
        harness
            .get_all_by_role_and_label(Role::ListBoxOption, "Shipping")
            .next()
            .unwrap()
            .click();
        save(&format!("s5.2-circle-{suffix}"), &mut harness);
        // S4.3: the Summary with its Dependencies.
        harness.key_press(Key::Num1);
        save(&format!("s4.3-dependencies-{suffix}"), &mut harness);
    }

    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    harness.run_steps(2);
    harness
        .get_by_role_and_label(Role::CheckBox, "No circular use")
        .click();
    save("c14-dialog-light", &mut harness);

    // S5.3: the graph of 12 modules, the most it draws, in a narrow workspace.
    let mut desk = Desk::new();
    desk.size = egui::vec2(1100.0, 700.0);
    let root = common::app_with_modules(&desk, "platform", 12);
    desk.knows(&[(&root, "PL")]);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    save("s5.3-12-modules-light", &mut harness);

    // S5.4: more than 12 modules.
    let desk = Desk::new();
    let root = common::app_with_modules(&desk, "platform", 16);
    desk.knows(&[(&root, "PL")]);
    let mut harness = desk.start();
    harness.key_press(Key::Num2);
    save("s5.4-table-light", &mut harness);
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_5() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut desk = Desk::new();
        desk.theme = theme;
        let shop = desk.app("shop-api");
        let crm = desk.app("crm");
        std::fs::write(
            crm.join("mortise.json"),
            r#"{"rules": {"no-circular-use": false, "public-surface-only": {"enabled": false}}}"#,
        )
        .unwrap();
        desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
        let mut harness = desk.start();
        // S4.1: the home screen, with the pip on the rail.
        harness.key_press(Key::S);
        save(&format!("s4.1-home-{suffix}"), &mut harness);
        // S6.4: the Findings list.
        harness.key_press(Key::F);
        save(&format!("s6.4-findings-{suffix}"), &mut harness);
        // S4.3: Checkout, with its alert.
        harness.key_press(Key::Escape);
        harness.key_press(Key::J);
        save(&format!("s4.3-alert-{suffix}"), &mut harness);
        harness.key_press(Key::J);
        save(&format!("s4.3-warning-{suffix}"), &mut harness);
    }

    // S3.1: many modules, problems first, with the group that needs attention.
    let mut desk = Desk::new();
    desk.size = egui::vec2(1100.0, 700.0);
    let root = common::app_with_modules(&desk, "platform", 30);
    desk.knows(&[(&root, "PL")]);
    let mut harness = desk.start();
    harness.key_press(Key::S);
    save("s3.1-needs-attention-light", &mut harness);
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_6_palette() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut desk = Desk::new();
        desk.theme = theme;
        let shop = desk.app("shop-api");
        let crm = desk.app("crm");
        desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
        let mut harness = desk.start();
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Checkout")
            .click();
        harness.run_steps(2);
        // S12.2 a: an empty query.
        harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
        save(&format!("s12.2-empty-{suffix}"), &mut harness);
        // S12.1: grouped results.
        harness
            .get_by_role_and_label(Role::TextInput, "Search or run a command")
            .type_text("check");
        save(&format!("s12.1-results-{suffix}"), &mut harness);
    }
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_6_settings() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut desk = Desk::new();
        desk.theme = theme;
        let shop = desk.app("shop-api");
        let crm = desk.app("crm");
        let pages = desk.app("landing-pages");
        desk.knows(&[(&shop, "SA"), (&crm, "CR"), (&pages, "LP")]);
        let mut harness = desk.start();
        // S8.1: Apps.
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
        save(&format!("s8.1-apps-{suffix}"), &mut harness);
        // S8.2: PHP, with no runner chosen yet.
        harness
            .get_by_role_and_label(Role::ListBoxOption, "PHP")
            .click();
        save(&format!("s8.2-php-{suffix}"), &mut harness);
        // S8.3: General.
        harness
            .get_by_role_and_label(Role::ListBoxOption, "General")
            .click();
        save(&format!("s8.3-general-{suffix}"), &mut harness);
        // S8.4: Keyboard.
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Keyboard")
            .click();
        save(&format!("s8.4-keyboard-{suffix}"), &mut harness);
    }

    // S13.1: the minimum window, with the strip and with the index opened by hand.
    let mut desk = Desk::new();
    desk.size = egui::vec2(960.0, 600.0);
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    save("s13.1a-960-strip-light", &mut harness);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::B);
    save("s13.1b-960-open-light", &mut harness);

    // Text size 150%.
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut settings = desk.saved();
    settings.general.text_size = 150;
    settings.save(&desk.settings_path()).unwrap();
    let mut harness = desk.start();
    save("text-size-150-light", &mut harness);
}

/// The runtime box in each of its states, by what the stub answers.
fn runtime_state(letter: char) -> Option<mortise_core::testing::StubActuator> {
    use mortise_core::testing::{Reply, StubActuator};
    let stub = common::shop_api_actuator();
    match letter {
        'c' => return None,
        'd' => {
            stub.expose(&["health"]);
            stub.set("/actuator/health", Reply::ok(r#"{"status":"UP"}"#));
        }
        'e' => stub.require_header("authorization", "Bearer right"),
        'f' => stub.set("/actuator/health", Reply::ok(r#"{"status":"UP"}"#)),
        'g' => stub.set(
            "/actuator/health",
            Reply::status(
                503,
                r#"{"status":"DOWN","components":{"ping":{"status":"UP"},"readinessState":{"status":"UP"},
                "diskSpace":{"status":"UP","details":{"total":494384795648,"free":213212008448}},
                "db":{"status":"DOWN","details":{"error":"Illuminate\\Database\\QueryException: SQLSTATE[HY000] [2002] Connection refused."}},
                "cache":{"status":"UP","details":{"store":"redis"}}}}"#,
            ),
        ),
        'h' => stub.set(
            "/actuator/health",
            Reply::status(
                503,
                r#"{"status":"OUT_OF_SERVICE","components":{"ping":{"status":"UP"},
                "readinessState":{"status":"OUT_OF_SERVICE"},
                "diskSpace":{"status":"UP","details":{"total":494384795648,"free":213212008448}},
                "db":{"status":"UP","details":{"connection":"mysql"}},
                "cache":{"status":"UP","details":{"store":"redis"}}}}"#,
            ),
        ),
        'i' => stub.set("/actuator/health", Reply::status(500, r#"{"message":"Server Error"}"#)),
        _ => return Some(StubActuator::healthy()),
    }
    Some(stub)
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_7_summary() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        // S4.2 with a running app, and S4.3 for two modules.
        let stub = common::shop_api_actuator();
        let staging = common::shop_api_actuator();
        let mut desk = Desk::new();
        desk.theme = theme;
        desk.size = egui::vec2(1280.0, 1100.0);
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        desk.connect(&[
            common::environment("local", &stub),
            common::environment("staging", &staging),
            common::environment("production", &staging),
        ]);
        let mut harness = desk.start();
        common::wait_for_answer(&mut harness);
        save(&format!("s4.2-runtime-{suffix}"), &mut harness);
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Checkout")
            .click();
        save(&format!("s4.3-runtime-checkout-{suffix}"), &mut harness);
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Inventory")
            .click();
        save(&format!("s4.3-runtime-inventory-{suffix}"), &mut harness);
    }
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_7_states() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        // a. The package is not installed.
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        let lock = std::fs::read_to_string(root.join("composer.lock")).unwrap();
        std::fs::write(
            root.join("composer.lock"),
            lock.replace("laravel-actuator", "laravel-other"),
        )
        .unwrap();
        desk.knows(&[(&root, "SA")]);
        let mut harness = desk.start();
        save(&format!("s6.5a-{suffix}"), &mut harness);

        // b. Installed, no environment yet.
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        let mut harness = desk.start();
        save(&format!("s6.5b-{suffix}"), &mut harness);

        for letter in ['c', 'd', 'e', 'f', 'g', 'h', 'i'] {
            let stub = runtime_state(letter);
            let mut desk = Desk::new();
            desk.theme = theme;
            let root = desk.app("shop-api");
            desk.knows(&[(&root, "SA")]);
            let environment = match &stub {
                Some(stub) => common::environment("local", stub),
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
            save(&format!("s6.5{letter}-{suffix}"), &mut harness);
        }
    }
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_7_settings() {
    use mortise_core::{Credentials, Environment};
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let local = common::shop_api_actuator();
        let staging = common::shop_api_actuator();
        staging.require_header("authorization", "Bearer s3cret-token");
        let production = common::shop_api_actuator();
        production.require_header("authorization", "Basic nothing");
        let mut desk = Desk::new();
        desk.theme = theme;
        let shop = desk.app("shop-api");
        let crm = desk.app("crm");
        desk.knows(&[(&shop, "SA"), (&crm, "CR")]);
        desk.connect(&[
            common::environment("local", &local),
            Environment {
                credentials: Credentials::Bearer,
                ..common::environment("staging", &staging)
            },
            Environment {
                credentials: Credentials::Basic {
                    user: "mortise".into(),
                },
                ..common::environment("production", &production)
            },
        ]);
        let mut harness = desk.start();
        common::wait_for_answer(&mut harness);

        // S8.1: the environments of the app, staging being edited.
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
        harness.run_steps(3);
        harness
            .get_by_role_and_label(Role::Row, "Environment staging")
            .click();
        harness.run_steps(3);
        // S8.5 b: the token as it is typed.
        harness
            .get_by_role_and_label(Role::PasswordInput, "Token")
            .focus();
        harness.run_steps(2);
        harness
            .get_by_role_and_label(Role::PasswordInput, "Token")
            .type_text("s3cret-token");
        save(&format!("s8.5b-typing-{suffix}"), &mut harness);
        // Leaving the field keeps the secret, and it is not shown again.
        harness
            .get_by_role_and_label(Role::TextInput, "Environment name")
            .focus();
        harness.run_steps(3);
        harness.get_by_role_and_label(Role::Button, "Test").click();
        wait_until(&mut harness, "the test of staging", |_| true);
        std::thread::sleep(std::time::Duration::from_millis(300));
        harness.run_steps(3);
        // Production: a test that is refused.
        harness
            .get_by_role_and_label(Role::Row, "Environment production")
            .click();
        harness.run_steps(3);
        harness.get_by_role_and_label(Role::Button, "Test").click();
        std::thread::sleep(std::time::Duration::from_millis(300));
        harness.run_steps(3);
        save(&format!("s8.5c-basic-{suffix}"), &mut harness);
        harness
            .get_by_role_and_label(Role::Row, "Environment staging")
            .click();
        save(&format!("s8.1-environments-{suffix}"), &mut harness);
        harness
            .get_by_role_and_label(Role::RadioButton, "Custom header")
            .click();
        save(&format!("s8.5d-header-{suffix}"), &mut harness);
        harness
            .get_by_role_and_label(Role::Row, "Environment local")
            .click();
        save(&format!("s8.5a-none-{suffix}"), &mut harness);

        // S1.3: the add panel with the runtime section.
        let stub = common::shop_api_actuator();
        stub.require_header("authorization", "Bearer s3cret-token");
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("booking-engine");
        std::fs::write(root.join(".env"), format!("APP_URL={}\n", stub.base_url)).unwrap();
        desk.picks.borrow_mut().push_back(root);
        let mut harness = desk.start();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
        harness.run_steps(3);
        save(&format!("s1.3-runtime-none-{suffix}"), &mut harness);
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
            .type_text("s3cret-token");
        harness.run_steps(2);
        harness.get_by_role_and_label(Role::Button, "Test").click();
        std::thread::sleep(std::time::Duration::from_millis(300));
        save(&format!("s1.3-runtime-{suffix}"), &mut harness);
    }
}

#[test]
#[ignore = "writes PNG files for a person to look at"]
fn milestone_8() {
    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let stub = common::shop_api_actuator();
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        desk.connect(&[common::environment("local", &stub)]);
        let mut harness = desk.start();
        common::wait_for_answer(&mut harness);
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Billing")
            .click();
        harness.run_steps(2);
        harness.key_press(Key::Num2);
        harness.run_steps(2);
        save(&format!("s5.1-links-control-{suffix}"), &mut harness);
        // S5.5: not scanned yet.
        harness.key_press(Key::P);
        save(&format!("s5.5-not-scanned-{suffix}"), &mut harness);
        // S5.7: the proposal.
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
        save(&format!("s5.7-proposal-{suffix}"), &mut harness);

        // S5.6: the scan as it reads, in a module of 96 files. One frame reads 8.
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        common::pad_module(&root, "Billing", 81);
        desk.knows(&[(&root, "SA")]);
        desk.connect(&[common::environment("local", &stub)]);
        let mut harness = desk.start();
        common::wait_for_answer(&mut harness);
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Billing")
            .click();
        harness.run_steps(2);
        harness.key_press(Key::Num2);
        harness.key_press(Key::P);
        harness.run_steps(2);
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
        harness.step();
        harness.step();
        harness.step();
        let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screens");
        harness
            .render()
            .unwrap()
            .save(out.join(format!("s5.6-scanning-{suffix}.png")))
            .unwrap();
    }

    // Without an environment the routes are not read.
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Billing")
        .click();
    harness.run_steps(2);
    harness.key_press(Key::Num2);
    harness.key_press(Key::P);
    harness.run_steps(2);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    save("s5.7-proposal-no-routes-light", &mut harness);
}
