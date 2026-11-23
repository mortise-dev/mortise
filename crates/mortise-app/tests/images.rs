//! Image tests: the screens of phase 1 in both themes, compared with the images in
//! `tests/snapshots`. Only the screens whose content is fixed are here: nothing
//! that shows a temporary folder, a clock or a port. A screen that shows one is cut
//! to the part that does not.
//!
//! The images are made on macOS, and other graphics drivers round edges in their
//! own way, so the tests run on macOS only. After a change that is meant to be
//! seen, make the images again with `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app
//! --test images` and look at what changed.
#![cfg(target_os = "macos")]

mod common;

use common::Desk;
use eframe::egui::{Key, Modifiers, Role, Theme};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::{Harness, SnapshotOptions, SnapshotResults};
use mortise_app::app::MortiseApp;

const THEMES: [(Theme, &str); 2] = [(Theme::Light, "light"), (Theme::Dark, "dark")];

/// The images of one test. They are compared when the test ends.
struct Shots {
    results: SnapshotResults,
    theme: &'static str,
}

impl Shots {
    fn new() -> Self {
        Self {
            results: SnapshotResults::new(),
            theme: "",
        }
    }

    fn take(&mut self, harness: &mut Harness<'static, MortiseApp>, screen: &str) {
        harness.run_steps(3);
        // A few pixels may differ where an edge is rounded.
        let options = SnapshotOptions::new().max_failed_pixels(40);
        let name = format!("{screen}-{}", self.theme);
        self.results
            .add(harness.try_snapshot_options(name, &options));
    }

    /// Takes the part of the window between `min` and `max`. The harness draws at
    /// scale 1, so a point is a pixel.
    fn take_part(
        &mut self,
        harness: &mut Harness<'static, MortiseApp>,
        screen: &str,
        (min, max): ((f64, f64), (f64, f64)),
    ) {
        harness.run_steps(3);
        let options = SnapshotOptions::new().max_failed_pixels(40);
        let name = format!("{screen}-{}", self.theme);
        let window = harness.render().expect("the window is drawn");
        let [x, y] = [min.0, min.1].map(|at| at.round() as u32);
        let [width, height] = [max.0 - min.0, max.1 - min.1].map(|size| size.round() as u32);
        let part = image::imageops::crop_imm(&window, x, y, width, height).to_image();
        self.results.add(egui_kittest::try_image_snapshot_options(
            &part, name, &options,
        ));
    }
}

/// The box of the control with this role and name: its top left and bottom right.
fn bounds(
    harness: &Harness<'static, MortiseApp>,
    role: Role,
    label: &str,
) -> ((f64, f64), (f64, f64)) {
    let node = harness.get_by_role_and_label(role, label);
    let rect = node
        .accesskit_node()
        .bounding_box()
        .expect("a drawn control");
    ((rect.x0, rect.y0), (rect.x1, rect.y1))
}

fn press(harness: &mut Harness<'static, MortiseApp>, key: Key) {
    harness.key_press(key);
    harness.run_steps(3);
}

fn chord(harness: &mut Harness<'static, MortiseApp>, key: Key) {
    harness.key_press_modifiers(Modifiers::COMMAND, key);
    harness.run_steps(3);
}

/// shop-api, known and open, in a theme.
fn shop_api(theme: Theme) -> (Desk, Harness<'static, MortiseApp>) {
    let mut desk = Desk::new();
    desk.theme = theme;
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let harness = desk.start();
    (desk, harness)
}

#[test]
fn start() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // S1.1: the first run. S1.2: the add panel before a folder is chosen.
        let mut desk = Desk::new();
        desk.theme = theme;
        let mut harness = desk.start();
        shots.take(&mut harness, "s1.1-first-run");
        harness
            .get_by_role_and_label(Role::Button, "Add app folder")
            .click();
        shots.take(&mut harness, "s1.2-add-app");
    }
}

#[test]
fn run_php_with_in_the_add_panel() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // booking-engine without the actuator, so that the buttons follow the field,
        // as in S1.4 c. The folder field shows a temporary folder: the images start under it.
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("booking-engine");
        let lock = std::fs::read_to_string(root.join("composer.lock")).unwrap();
        std::fs::write(
            root.join("composer.lock"),
            lock.replace("laravel-actuator", "laravel-other"),
        )
        .unwrap();
        desk.picks.borrow_mut().push_back(root);
        let mut harness = desk.start();
        chord(&mut harness, Key::O);
        let (folder, folder_end) = bounds(&harness, Role::TextInput, "App folder");
        let (_, field) = bounds(&harness, Role::TextInput, "Run PHP with");
        let (_, test) = bounds(&harness, Role::Button, "Test how PHP runs");
        // The text of the folder field is inset by 9 px; the column starts at its edge.
        let top_left = (folder.0 - 9.0 - 12.0, folder_end.1 + 8.0);
        // S1.3: the "Run PHP with" part, with the proposal of a folder that points to no runner.
        shots.take_part(
            &mut harness,
            "s1.3-run-php-with",
            (top_left, (test.0 + 12.0, field.1 + 12.0)),
        );

        // S1.4 c: a program that is not there. The app can still be added, read only.
        harness
            .get_by_role_and_label(Role::RadioButton, "Custom")
            .click();
        harness.run_steps(2);
        harness
            .get_by_role_and_label(Role::TextInput, "Run PHP with")
            .focus();
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness
            .get_by_role_and_label(Role::TextInput, "Run PHP with")
            .type_text("php-none artisan");
        harness.run();
        harness
            .get_by_role_and_label(Role::Button, "Test how PHP runs")
            .click();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while harness.query_by_label_contains("did not answer").is_none() {
            assert!(std::time::Instant::now() < deadline, "the test did not end");
            std::thread::sleep(std::time::Duration::from_millis(20));
            harness.run_steps(2);
        }
        // The pointer leaves the Test button, so that the image shows it at rest.
        harness.hover_at(eframe::egui::pos2(1200.0, 780.0));
        harness.run_steps(3);
        let (_, add) = bounds(&harness, Role::Button, "Add booking-engine, read only");
        shots.take_part(
            &mut harness,
            "s1.4c-php-not-found",
            (top_left, (test.0 + 12.0, add.1 + 12.0)),
        );
    }
}

#[test]
fn summary_and_findings() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // S3.1 and S4.2: the index and the Summary of all modules. The runtime box
        // is in state b of S6.5: no environment yet.
        let (_desk, mut harness) = shop_api(theme);
        shots.take(&mut harness, "s4.2-summary-all");
        // S6.4: the Findings list.
        press(&mut harness, Key::F);
        shots.take(&mut harness, "s6.4-findings");
        press(&mut harness, Key::Escape);
        // S4.3: the Summary of one module.
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Checkout")
            .click();
        shots.take(&mut harness, "s4.3-summary-module");
        // S7.4: a tab of a later phase.
        press(&mut harness, Key::Num3);
        shots.take(&mut harness, "s7.4-generate");
        // S13.1: the index as a strip.
        press(&mut harness, Key::Num1);
        chord(&mut harness, Key::B);
        shots.take(&mut harness, "s13.1-index-strip");
    }
}

#[test]
fn boundary_and_the_public_surface() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        let (_desk, mut harness) = shop_api(theme);
        // S5.3: the app graph and the rules.
        press(&mut harness, Key::Num2);
        shots.take(&mut harness, "s5.3-boundary-all");
        // S5.5 and S5.7: the public surface of Billing, before and after a scan.
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Billing")
            .click();
        harness.run_steps(2);
        press(&mut harness, Key::P);
        shots.take(&mut harness, "s5.5-surface-not-scanned");
        chord(&mut harness, Key::Enter);
        harness.run_steps(6);
        shots.take(&mut harness, "s5.7-surface-proposal");
        // S5.1: one module, with the files of the link that breaks a rule. Esc gives
        // the keys back to the index, and J moves to Checkout.
        press(&mut harness, Key::P);
        press(&mut harness, Key::Escape);
        press(&mut harness, Key::J);
        press(&mut harness, Key::J);
        shots.take(&mut harness, "s5.1-boundary-module");
    }
}

#[test]
fn many_modules() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // S5.4: above 12 modules the graph gives way to a table of links.
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = common::app_with_modules(&desk, "platform", 30);
        desk.knows(&[(&root, "PL")]);
        let mut harness = desk.start();
        shots.take(&mut harness, "s3.1-index-30-modules");
        press(&mut harness, Key::Num2);
        shots.take(&mut harness, "s5.4-links-table");
    }
}

#[test]
fn palette_and_settings() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        let (_desk, mut harness) = shop_api(theme);
        // S12.1: the palette, empty and with a query.
        chord(&mut harness, Key::K);
        shots.take(&mut harness, "s12.1-palette");
        harness
            .get_by_role_and_label(Role::TextInput, "Search or run a command")
            .type_text("check");
        shots.take(&mut harness, "s12.1-palette-query");
        press(&mut harness, Key::Escape);
        // S8.3 and S8.4: General and Keyboard. Apps shows the folder, which is a
        // temporary one here.
        chord(&mut harness, Key::Comma);
        harness
            .get_by_role_and_label(Role::ListBoxOption, "General")
            .click();
        shots.take(&mut harness, "s8.3-settings-general");
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Keyboard")
            .click();
        shots.take(&mut harness, "s8.4-settings-keyboard");
        // S8.2: PHP, after a test of a program that is not there. The working folder
        // is empty, so no temporary folder shows.
        harness
            .get_by_role_and_label(Role::ListBoxOption, "PHP")
            .click();
        harness.run_steps(2);
        harness
            .get_by_role_and_label(Role::RadioButton, "Custom")
            .click();
        harness.run_steps(2);
        harness
            .get_by_role_and_label(Role::TextInput, "Run PHP with")
            .focus();
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness
            .get_by_role_and_label(Role::TextInput, "Run PHP with")
            .type_text("php-none artisan");
        harness.run();
        harness
            .get_by_role_and_label(Role::Button, "Test how PHP runs")
            .click();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while harness.query_by_label_contains("did not answer").is_none() {
            assert!(std::time::Instant::now() < deadline, "the test did not end");
            std::thread::sleep(std::time::Duration::from_millis(20));
            harness.run_steps(2);
        }
        // The pointer leaves the Test button, so that the image shows it at rest.
        harness.hover_at(eframe::egui::pos2(1200.0, 780.0));
        harness.run();
        shots.take(&mut harness, "s8.2-settings-php");
    }
}

#[test]
fn an_app_without_the_actuator() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // S6.5 a: the runtime box says how to install the package.
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
        shots.take(&mut harness, "s6.5a-not-installed");
    }
}
