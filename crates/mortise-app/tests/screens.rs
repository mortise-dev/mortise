//! Renders screens to `target/screens`, to be looked at beside the drawings of the
//! specification. Run with `cargo test -p mortise-app --test screens -- --ignored`.

mod common;

use std::path::Path;

use common::{Desk, wait_until};
use eframe::egui::{self, Key, Modifiers, Role, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use mortise_app::app::MortiseApp;
use mortise_app::chrome::Chrome;

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
        harness.key_press(Key::Num3);
        save(&format!("s7.4-generate-{suffix}"), &mut harness);
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
