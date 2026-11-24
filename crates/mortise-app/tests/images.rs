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

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use common::Desk;
use eframe::egui::{Key, Modifiers, Role, Theme};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::{Harness, SnapshotOptions, SnapshotResults};
use mortise_app::app::MortiseApp;
use mortise_core::history::{Ended, Entry, History};
use mortise_core::testing::{Reply, StubActuator};
use mortise_core::{CommandLine, RunnerKind};

const THEMES: [(Theme, &str); 2] = [(Theme::Light, "light"), (Theme::Dark, "dark")];

/// A box of the window: its top left and bottom right.
type Part = ((f64, f64), (f64, f64));

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
        part: ((f64, f64), (f64, f64)),
    ) {
        self.take_part_masked(harness, screen, part, &[]);
    }

    /// As [`Shots::take_part`], with the boxes in `masks` painted over in the colour
    /// 4 px inside the top left corner of the part: a clock or a path cut out.
    fn take_part_masked(
        &mut self,
        harness: &mut Harness<'static, MortiseApp>,
        screen: &str,
        (min, max): ((f64, f64), (f64, f64)),
        masks: &[Part],
    ) {
        harness.run_steps(3);
        let options = SnapshotOptions::new().max_failed_pixels(40);
        let name = format!("{screen}-{}", self.theme);
        let mut window = harness.render().expect("the window is drawn");
        let [x, y] = [min.0, min.1].map(|at| at.round() as u32);
        let fill = *window.get_pixel(x + 4, y + 12);
        // A long line reaches past the window.
        let (right, bottom) = (window.width(), window.height());
        for (from, to) in masks {
            for my in from.1.floor() as u32..(to.1.ceil() as u32).min(bottom) {
                for mx in from.0.floor() as u32..(to.0.ceil() as u32).min(right) {
                    window.put_pixel(mx, my, fill);
                }
            }
        }
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
        // S1.4 a: a folder that is no Laravel app. S1.4 b: an app without the package.
        // Each is chosen first, then the panel is read again for the next folder.
        // The root of the disk has no artisan file, and its path is the same on every run.
        let notes = std::path::PathBuf::from("/");
        let bare = desk.app("bare-app");
        std::fs::write(
            bare.join("composer.lock"),
            r#"{"packages": [{"name": "laravel/framework", "version": "v13.2.0"}]}"#,
        )
        .unwrap();
        desk.picks.borrow_mut().push_back(notes);
        desk.picks.borrow_mut().push_back(bare);
        desk.picks.borrow_mut().push_back(root.clone());
        let mut harness = desk.start();
        chord(&mut harness, Key::O);
        let (folder, folder_end) = bounds(&harness, Role::TextInput, "App folder");
        let (_, cancel) = bounds(&harness, Role::Button, "Cancel");
        let top_left = (folder.0 - 9.0 - 12.0, folder_end.1 + 8.0);
        let message = harness
            .get_by_label_contains("No Laravel app in")
            .accesskit_node()
            .bounding_box()
            .expect("a drawn message");
        let right = message.x1.max(folder_end.0) + 12.0;
        shots.take_part(
            &mut harness,
            "s1.4a-not-laravel",
            (top_left, (right, cancel.1 + 12.0)),
        );
        chord(&mut harness, Key::O);
        let (_, check) = bounds(&harness, Role::Button, "Check again");
        shots.take_part(
            &mut harness,
            "s1.4b-package-missing",
            (top_left, (right, check.1 + 12.0)),
        );
        chord(&mut harness, Key::O);
        let (folder, folder_end) = bounds(&harness, Role::TextInput, "App folder");
        let (_, field) = bounds(&harness, Role::TextInput, "Run PHP with");
        let (_, test) = bounds(&harness, Role::Button, "Test how PHP runs");
        // The text of the folder field is inset by 9 px; the column starts at its edge.
        let top_left = (folder.0 - 9.0 - 12.0, folder_end.1 + 8.0);
        // The line that Test runs is under the field (deviation S35).
        let herd = CommandLine::artisan(&RunnerKind::Herd, &["--version"]).unwrap();
        let (_, runs) = bounds(
            &harness,
            Role::Label,
            &format!("Test runs {}", herd.display()),
        );
        // S1.3: the "Run PHP with" part, with the proposal of a folder that points to no runner.
        shots.take_part(
            &mut harness,
            "s1.3-run-php-with",
            (top_left, (test.0 + 12.0, field.1.max(runs.1) + 12.0)),
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
        // S13.1: the index as a strip.
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

        // A neighbour whose name does not fit shows its stamp, not a cut name.
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        let long = root.join("Modules/NotificationPreferencesAndDigests");
        common::copy_dir(&root.join("Modules/Notifications"), &long);
        common::rename_in_files(&long, "Notifications", "NotificationPreferencesAndDigests");
        desk.knows(&[(&root, "SA")]);
        let mut harness = desk.start();
        harness
            .get_by_role_and_label(Role::ListBoxOption, "Checkout")
            .click();
        press(&mut harness, Key::Num2);
        let (used_by, _) = bounds(&harness, Role::Label, "Used by, 3");
        let (files, _) = bounds(&harness, Role::Heading, "Checkout uses Billing");
        let part = (
            (used_by.0 - 33.0, used_by.1 - 17.0),
            (1264.0, files.1 - 12.0),
        );
        shots.take_part(&mut harness, "s5.1-long-neighbour", part);
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
fn runtime_per_module() {
    use mortise_core::testing::{Reply, StubActuator};
    // S3.1, the sixth drawing: each module reports its own indicator, and Billing is down.
    let stub = StubActuator::healthy();
    stub.set(
        "/actuator/health",
        Reply::status(
            503,
            r#"{"status":"DOWN","components":{"ping":{"status":"UP"},"db":{"status":"UP"},
            "module:catalog:search":{"status":"UP"},"module:checkout:payments":{"status":"UP"},
            "module:users:directory":{"status":"UP"},"module:shipping:carrier":{"status":"UP"},
            "module:inventory:stock":{"status":"UP"},"module:notifications:mail":{"status":"UP"},
            "module:billing:gateway":{"status":"DOWN","details":{"error":"Connection refused."}}}}"#,
        ),
    );
    // The index column only: the Summary says when the app was checked.
    let index = |width: f64| ((52.0, 34.0), (52.0 + width, 800.0));
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        for rows in [
            mortise_core::IndexRows::TwoLines,
            mortise_core::IndexRows::OneLine,
        ] {
            let mut desk = Desk::new();
            desk.theme = theme;
            let root = desk.app("shop-api");
            desk.knows(&[(&root, "SA")]);
            desk.connect(&[common::environment("local", &stub)]);
            let mut settings = desk.saved();
            settings.general.index_rows = rows;
            settings.save(&desk.settings_path()).unwrap();
            let mut harness = desk.start();
            common::wait_for_answer(&mut harness);
            if rows == mortise_core::IndexRows::OneLine {
                shots.take_part(&mut harness, "s3.1-index-runtime-one-line", index(264.0));
            } else {
                shots.take_part(&mut harness, "s3.1-index-runtime", index(264.0));
                // S13.1: the strip, wider by the runtime mark and its gap.
                chord(&mut harness, Key::B);
                shots.take_part(&mut harness, "s13.1-index-strip-runtime", index(96.0));
            }
        }
    }
}

#[test]
fn palette_and_settings() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        // S12.2 a: a migrate 2 minutes ago and a model before it, in "Recent".
        common::ran(
            &desk,
            &root,
            &[
                ("module:make-model Invoice Billing", 5),
                ("module:migrate Checkout", 2),
            ],
        );
        let mut harness = desk.start();
        // S12.1: the palette, empty with Checkout opened last, then with a query on
        // "All modules". Keys, not clicks, so that no pointer shows.
        chord(&mut harness, Key::K);
        harness
            .get_by_role_and_label(Role::TextInput, "Search or run a command")
            .type_text("checkout");
        press(&mut harness, Key::Enter);
        chord(&mut harness, Key::K);
        shots.take(&mut harness, "s12.2a-palette-empty");
        press(&mut harness, Key::Escape);
        for key in [Key::K; 3] {
            press(&mut harness, key);
        }
        chord(&mut harness, Key::K);
        harness
            .get_by_role_and_label(Role::TextInput, "Search or run a command")
            .type_text("check");
        shots.take(&mut harness, "s12.1-palette-query");
        // S12.2 b: a query that finds nothing.
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness
            .get_by_role_and_label(Role::TextInput, "Search or run a command")
            .type_text("zzq");
        shots.take(&mut harness, "s12.2b-palette-none");
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

#[test]
fn the_runtime_box_of_a_module() {
    // Billing enabled and missing on disk, as the modules component of 13.4 says it.
    let missing = r#"{"status":"DOWN","components":{"ping":{"status":"UP"},
        "modules":{"status":"DOWN","details":{"modules":{"Billing":{"enabled":true,
        "error":"Enabled in the status store, not found on disk."}}}}}}"#;
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // S4.3: the runtime box of Billing, up, then down with the app's sentence.
        for (screen, health) in [
            ("s4.3-module-runtime", None),
            ("s4.3-module-runtime-error", Some(missing)),
        ] {
            let stub = common::shop_api_actuator();
            if let Some(health) = health {
                stub.set("/actuator/health", Reply::status(503, health));
            }
            let mut desk = Desk::new();
            desk.theme = theme;
            let root = desk.app("shop-api");
            desk.knows(&[(&root, "SA")]);
            desk.connect(&[common::environment("local", &stub)]);
            let mut harness = desk.start();
            common::wait_for_answer(&mut harness);
            select(&mut harness, "Billing");
            // The box, from its heading to the box of scheduled tasks under it. A
            // heading is 18 px under the top of its box, and boxes are 12 px apart.
            let mid = |(min, max): Part| (min.1 + max.1) / 2.0;
            let heading = bounds(&harness, Role::Heading, "Runtime on local");
            let next = bounds(&harness, Role::Heading, "Scheduled tasks");
            // The time of the answer is cut out, in the colour outside the box.
            let checked = label_bounds(&harness, "checked ");
            let part = (
                (heading.0.0 - 13.0 - 8.0, mid(heading) - 18.0 - 8.0),
                (checked.1.0 + 13.0 + 8.0, mid(next) - 18.0 - 12.0 + 8.0),
            );
            shots.take_part_masked(&mut harness, screen, part, &[checked]);
        }
    }
}

// --- The Actions tab (S10.1 to S10.3) ---

/// The pointer rests where it lights nothing up.
fn rest(harness: &mut Harness<'static, MortiseApp>) {
    harness.hover_at(eframe::egui::pos2(1200.0, 780.0));
    harness.run_steps(3);
}

fn select(harness: &mut Harness<'static, MortiseApp>, row: &str) {
    harness
        .get_by_role_and_label(Role::ListBoxOption, row)
        .click();
    rest(harness);
}

#[test]
fn actions() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // Through Herd, so that the line reads as in the drawings, and nothing runs.
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        let mut settings = desk.saved();
        settings.apps[0].php.runner = Some(RunnerKind::Herd);
        settings.save(&desk.settings_path()).unwrap();
        let mut harness = desk.start();

        // S10.1: Checkout, with Migrate selected.
        select(&mut harness, "Checkout");
        harness.get_by_role_and_label(Role::Tab, "Actions").click();
        rest(&mut harness);
        shots.take(&mut harness, "s10.1-actions");
        // S10.2 a, c, d and e: the other panels of a module that is on.
        for (row, screen) in [
            ("Turn off", "s10.2a-turn-off"),
            ("Roll back", "s10.2c-roll-back"),
            ("Seed", "s10.2d-seed"),
            ("Run tests", "s10.2e-run-tests"),
        ] {
            select(&mut harness, row);
            shots.take(&mut harness, screen);
        }
        // S10.3: Roll back asks first, with focus on Cancel.
        select(&mut harness, "Roll back");
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
        harness.run_steps(3);
        shots.take(&mut harness, "s10.3-roll-back");
        press(&mut harness, Key::Escape);
        // S10.2 f: "All modules".
        select(&mut harness, "All modules");
        shots.take(&mut harness, "s10.2f-all-modules");
        // S10.2 b: Turn on, once Checkout is off.
        common::turn_off(&root, "Checkout");
        // Read the app again now, rather than wait for the watcher: the git work of
        // the test beside this one can hold its events back.
        chord(&mut harness, Key::R);
        common::wait_until(&mut harness, "Checkout is off", |app| {
            app.app().is_some_and(|app| {
                app.module("Checkout")
                    .is_some_and(|module| module.state == mortise_core::ModuleState::Off)
            })
        });
        select(&mut harness, "Checkout");
        select(&mut harness, "Turn on");
        shots.take(&mut harness, "s10.2b-turn-on");
    }
}

// --- The Actions tab of an app with package data (13.4) ---

/// Takes the window with the time of the state read cut out, where the screen shows one.
fn read_time(shots: &mut Shots, harness: &mut Harness<'static, MortiseApp>, screen: &str) {
    harness.run_steps(3);
    let masks: Vec<Part> = harness
        .query_by_label_contains("Read at ")
        .map(|node| {
            let rect = node.accesskit_node().bounding_box().expect("a drawn label");
            ((rect.x0, rect.y0), (rect.x1, rect.y1))
        })
        .into_iter()
        .collect();
    shots.take_part_masked(harness, screen, WINDOW, &masks);
}

#[test]
fn actions_with_package_data() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // The fake artisan in the app folder, so that the lines read the same on every machine.
        let desk = drawer_desk(theme, 1280.0, "", &[]);
        let root = desk.path().join("shop-api");
        common::package_version(&root, "v13.4.0");
        // The migration that ran in the recording creates a table.
        let migrations = root.join("Modules/Checkout/database/migrations");
        std::fs::create_dir_all(&migrations).unwrap();
        std::fs::write(
            migrations.join("2026_11_23_110149_create_orders_table.php"),
            "<?php\nreturn new class extends Migration {\n    public function up(): void\n    {\n        \
             Schema::create('orders', function (Blueprint $table) {\n            $table->id();\n        \
             });\n    }\n};\n",
        )
        .unwrap();
        let mut harness = desk.start();

        // S10.1: Checkout, with the state read.
        select(&mut harness, "Checkout");
        harness.get_by_role_and_label(Role::Tab, "Actions").click();
        rest(&mut harness);
        click(&mut harness, Role::Button, "Read migration state");
        wait_for(&mut harness, "2026_11_23_120000_create_order_items_table");
        read_time(&mut shots, &mut harness, "s10.1-actions-state");
        // S10.2 c: Roll back lists its last batch.
        select(&mut harness, "Roll back");
        read_time(&mut shots, &mut harness, "s10.2c-roll-back-state");
        // S10.3: the dialog lists the migration.
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
        harness.run_steps(3);
        // The dialog covers the time.
        shots.take(&mut harness, "s10.3-roll-back-state");
        press(&mut harness, Key::Escape);
        // S10.4 and S10.5 a: reset names its table.
        select(&mut harness, "Destructive");
        select(&mut harness, "Reset migrations");
        read_time(&mut shots, &mut harness, "s10.4-reset-state");
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
        harness.run_steps(3);
        // The dialog alone: the time of the read shows beside it.
        shots.take_part(
            &mut harness,
            "s10.5a-reset-state",
            ((400.0, 120.0), (880.0, 350.0)),
        );
        press(&mut harness, Key::Escape);
        // S10.2 f: "All modules", once every module is read.
        select(&mut harness, "All modules");
        click(&mut harness, Role::Button, "Read migration state");
        wait_for(&mut harness, "3 migrations in 2 modules");
        read_time(&mut shots, &mut harness, "s10.2f-all-modules-state");
        // S4.3: the Summary of Checkout.
        select(&mut harness, "Checkout");
        press(&mut harness, Key::Num1);
        read_time(&mut shots, &mut harness, "s4.3-summary-module-state");

        // S10.2 e: Run tests, with its options.
        press(&mut harness, Key::Num4);
        select(&mut harness, "Run tests");
        fill(&mut harness, "Filter, a test name or pattern", "Order");
        click(&mut harness, Role::CheckBox, "Run in parallel");
        read_time(&mut shots, &mut harness, "s10.2e-run-tests-13.4");

        // S6.4: Check translations, then Findings with the warning and the To do group.
        select(&mut harness, "Check translations");
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
        harness.run_steps(3);
        common::wait_until(&mut harness, "the report of Checkout", |app| {
            app.app()
                .is_some_and(|app| app.package_data.translations.is_some())
        });
        wait_for(&mut harness, ": Exit code 1");
        press(&mut harness, Key::F);
        // The window above the runner drawer: its output and time change from run to run.
        shots.take_part(
            &mut harness,
            "s6.4-findings-package-data",
            ((0.0, 0.0), (1280.0, 600.0)),
        );
    }
}

// --- The destructive actions (S10.4, S10.5) ---

#[test]
fn destructive_actions() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // Through Herd, so that the line reads as in the drawings, and nothing runs.
        let mut desk = Desk::new();
        desk.theme = theme;
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        let mut settings = desk.saved();
        settings.apps[0].php.runner = Some(RunnerKind::Herd);
        settings.save(&desk.settings_path()).unwrap();
        // A repository with two changed files in Checkout, for the line of delete.
        common::with_git_history(&root);
        for file in ["routes/web.php", "config/config.php"] {
            let path = root.join("Modules/Checkout").join(file);
            let mut text = std::fs::read_to_string(&path).unwrap();
            text.push_str("\n// not committed\n");
            std::fs::write(path, text).unwrap();
        }
        let mut harness = desk.start();

        // S10.4: the group open on Checkout, with Reset migrations selected.
        select(&mut harness, "Checkout");
        harness.get_by_role_and_label(Role::Tab, "Actions").click();
        rest(&mut harness);
        select(&mut harness, "Destructive");
        select(&mut harness, "Reset migrations");
        shots.take(&mut harness, "s10.4-reset");
        // S10.4 a and b.
        select(&mut harness, "Refresh migrations");
        shots.take(&mut harness, "s10.4a-refresh");
        select(&mut harness, "Delete module");
        shots.take(&mut harness, "s10.4b-delete");

        // S10.5 d: delete asks for the name, before and after it is typed.
        chord(&mut harness, Key::Enter);
        shots.take(&mut harness, "s10.5d-delete");
        type_name(&mut harness, "Type Checkout to confirm", "Checkout");
        shots.take(&mut harness, "s10.5d-delete-typed");
        press(&mut harness, Key::Escape);
        // S10.5 a and b.
        select(&mut harness, "Reset migrations");
        chord(&mut harness, Key::Enter);
        shots.take(&mut harness, "s10.5a-reset");
        press(&mut harness, Key::Escape);
        select(&mut harness, "Refresh migrations");
        chord(&mut harness, Key::Enter);
        shots.take(&mut harness, "s10.5b-refresh");
        press(&mut harness, Key::Escape);

        // S10.4 c and S10.5 c: under "All modules".
        select(&mut harness, "All modules");
        select(&mut harness, "Destructive");
        select(&mut harness, "Drop all tables and migrate");
        shots.take(&mut harness, "s10.4c-fresh");
        chord(&mut harness, Key::Enter);
        shots.take(&mut harness, "s10.5c-fresh");
        type_name(&mut harness, "Type shop-api to confirm", "shop-api");
        shots.take(&mut harness, "s10.5c-fresh-typed");
        press(&mut harness, Key::Escape);

        // S10.4 d: the app reports production.
        std::fs::write(root.join(".env"), "APP_ENV=production\n").unwrap();
        // Read the app again now, rather than wait for the watcher.
        chord(&mut harness, Key::R);
        common::wait_until(&mut harness, "the app reports production", |app| {
            app.app()
                .is_some_and(|app| app.app_env.as_deref() == Some("production"))
        });
        select(&mut harness, "Checkout");
        select(&mut harness, "Destructive");
        select(&mut harness, "Reset migrations");
        shots.take(&mut harness, "s10.4d-production");
    }
}

/// Types a name into the field of a dialog, with the pointer at rest after.
fn type_name(harness: &mut Harness<'static, MortiseApp>, field: &str, text: &str) {
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .focus();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .type_text(text);
    rest(harness);
}

// --- The Generate tab (S9.1 to S9.6) ---

/// Puts `text` in a field of the workspace, in place of what it holds.
fn fill(harness: &mut Harness<'static, MortiseApp>, field: &str, text: &str) {
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .focus();
    harness.run_steps(2);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.key_press(Key::Backspace);
    if !text.is_empty() {
        harness
            .get_by_role_and_label(Role::TextInput, field)
            .type_text(text);
    }
    harness.run_steps(2);
}

fn click(harness: &mut Harness<'static, MortiseApp>, role: Role, label: &str) {
    harness.get_by_role_and_label(role, label).click();
    rest(harness);
}

/// The time of the run in the bar of the drawer, up to "Run again", whatever its width.
fn time_of_run(harness: &Harness<'static, MortiseApp>) -> Vec<Part> {
    let (min, max) = label_bounds(harness, ": Done in ");
    let ((again, _), _) = bounds(harness, Role::Button, "Run again");
    vec![(min, (again - 4.0, max.1))]
}

/// The whole window.
const WINDOW: Part = ((0.0, 0.0), (1280.0, 800.0));

#[test]
fn generate() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // The fake artisan in the app folder, so that the lines read the same on every machine.
        let desk = drawer_desk(theme, 1280.0, "", &[]);
        let mut harness = desk.start();

        // S9.1: the picker of Billing, with Model the current row.
        select(&mut harness, "Billing");
        harness.get_by_role_and_label(Role::Tab, "Generate").click();
        rest(&mut harness);
        shots.take(&mut harness, "s9.1-picker");

        // S9.2: the model form, without the file list (deviation S44).
        click(&mut harness, Role::ListBoxOption, "Model");
        fill(&mut harness, "Model name", "PaymentIntent");
        fill(
            &mut harness,
            "Fillable attributes",
            "order_id,amount,currency,status",
        );
        click(&mut harness, Role::CheckBox, "Migration");
        click(&mut harness, Role::CheckBox, "Policy");
        shots.take(&mut harness, "s9.2-model-form");

        // S9.5: created, with the toast. The time of the run is cut out.
        click(&mut harness, Role::Button, "Create files");
        wait_for(&mut harness, "Created 3 files in Billing.");
        let masks = time_of_run(&harness);
        shots.take_part_masked(&mut harness, "s9.5-created", WINDOW, &masks);

        // S9.6: the policy existed and was left, then Replace asks with its line.
        click(&mut harness, Role::Button, "Generate another");
        fill(&mut harness, "Model name", "Invoice");
        fill(&mut harness, "Fillable attributes", "");
        click(&mut harness, Role::Button, "Create files");
        wait_for(&mut harness, "Created 2 of 3 files in Billing.");
        let masks = time_of_run(&harness);
        shots.take_part_masked(&mut harness, "s9.6-left", WINDOW, &masks);
        click(&mut harness, Role::Button, "Replace InvoicePolicy.php");
        shots.take_part_masked(&mut harness, "s9.6-replace", WINDOW, &masks);
        press(&mut harness, Key::Escape);

        // S9.4: the new module form, from Ctrl N.
        chord(&mut harness, Key::N);
        fill(&mut harness, "Module names", "Loyalty");
        click(&mut harness, Role::RadioButton, "API");
        shots.take_part_masked(&mut harness, "s9.4-new-module", WINDOW, &masks);
    }
}

/// The Generate tab of a 13.4 app: the generators read from the app and the files
/// that a dry run lists (S9.1 to S9.4, deviations S44 and S55 to S57).
#[test]
fn generate_with_package_data() {
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        let desk = drawer_desk(theme, 1280.0, "", &[]);
        common::package_version(&desk.path().join("shop-api"), "v13.4.0");
        let mut harness = desk.start();

        // S9.1: the picker after "Read the generators". The time of the read is cut out.
        select(&mut harness, "Billing");
        harness.get_by_role_and_label(Role::Tab, "Generate").click();
        rest(&mut harness);
        click(
            &mut harness,
            Role::Button,
            "Read the generators of shop-api",
        );
        wait_for(&mut harness, "Read from shop-api at ");
        let masks = varying(&harness, &["Read from shop-api at "]);
        shots.take_part_masked(&mut harness, "s9.1-picker-read", WINDOW, &masks);

        // S9.2: the model form with the three files of the dry run.
        click(&mut harness, Role::ListBoxOption, "Model");
        fill(&mut harness, "Model name", "PaymentIntent");
        fill(
            &mut harness,
            "Fillable attributes",
            "order_id,amount,currency,status",
        );
        click(&mut harness, Role::CheckBox, "Migration");
        click(&mut harness, Role::CheckBox, "Policy");
        click(&mut harness, Role::Button, "List files");
        wait_for(&mut harness, "3 files, none exist yet");
        shots.take(&mut harness, "s9.2-model-files");

        // S9.3: the dry run finds the policy of Invoice.
        fill(&mut harness, "Model name", "Invoice");
        fill(&mut harness, "Fillable attributes", "");
        click(&mut harness, Role::Button, "List files");
        wait_for(&mut harness, "3 files, 1 exists already");
        shots.take(&mut harness, "s9.3-file-exists");

        // S9.4: the new module form with the files of the module.
        chord(&mut harness, Key::N);
        fill(&mut harness, "Module names", "Loyalty");
        click(&mut harness, Role::RadioButton, "API");
        click(&mut harness, Role::Button, "List files");
        wait_for(&mut harness, "Modules/Loyalty does not exist yet");
        shots.take(&mut harness, "s9.4-new-module-files");
    }
}

// --- The runner drawer (S11.1) ---

/// A run of the history: its artisan words, how it ended, its local clock time today
/// and its output.
type Seeded<'a> = (&'a [&'a str], Ended, (i8, i8), Vec<String>);

/// shop-api run through the fake artisan, with `words` before the artisan
/// arguments, and these runs in its history, the latest first.
fn drawer_desk(theme: Theme, width: f32, words: &str, runs: &[Seeded<'_>]) -> Desk {
    let mut desk = Desk::new();
    desk.theme = theme;
    desk.size.x = width;
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let mut settings = desk.saved();
    // A copy of the fake artisan in the app folder, so that the $ line reads the
    // same on every machine. macOS starts a relative program from the folder it runs in.
    let fake = env!("CARGO_BIN_EXE_app-fake-artisan");
    std::fs::copy(fake, root.join("fake-artisan")).unwrap();
    settings.apps[0].php.runner = Some(RunnerKind::Custom {
        line: format!("./fake-artisan{words}"),
    });
    settings.save(&desk.settings_path()).unwrap();
    // The file of the migration that fails, so that the drawer offers to open it.
    let migrations = root.join("Modules/Shipping/database/migrations");
    std::fs::create_dir_all(&migrations).unwrap();
    std::fs::write(
        migrations.join("2026_11_22_140500_add_tracking_code_to_shipments_table.php"),
        "<?php\n",
    )
    .unwrap();
    // The clock times are local times of today, so the rows read the same anywhere.
    let history = History::open(&desk.data_dir(), &root);
    for (args, ended, (hour, minute), output) in runs.iter().rev() {
        let at = jiff::Zoned::now()
            .with()
            .hour(*hour)
            .minute(*minute)
            .second(0)
            .subsec_nanosecond(0)
            .build()
            .unwrap();
        let started_ms = at.timestamp().as_millisecond();
        let entry = Entry {
            id: started_ms as u64,
            args: args.iter().map(|word| word.to_string()).collect(),
            label: args.join(" "),
            started_ms,
            ended: ended.clone(),
            cut: false,
        };
        history.record(entry, output).unwrap();
    }
    desk
}

/// The output of a recording of `fixtures/output`, a line each, with its codes.
fn recording(name: &str) -> Vec<String> {
    let path = common::fixture("output").join(format!("{name}.json"));
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let stdout = json["stdout"].as_str().unwrap();
    stdout.lines().map(str::to_owned).collect()
}

/// Waits until a label holds `text`: a run needs real time to pass.
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
    // The pointer rests where it lights nothing up.
    harness.hover_at(eframe::egui::pos2(700.0, 20.0));
    harness.run_steps(3);
}

/// The box of the first label that holds `text`.
fn label_bounds(harness: &Harness<'static, MortiseApp>, text: &str) -> Part {
    let node = harness.query_by_label_contains(text).expect(text);
    let rect = node.accesskit_node().bounding_box().expect("a drawn label");
    ((rect.x0, rect.y0), (rect.x1, rect.y1))
}

/// The drawer, from its handle to the bottom of the window.
fn drawer_part(harness: &Harness<'static, MortiseApp>, window_h: f64) -> Part {
    let (min, max) = bounds(harness, Role::Splitter, "Height of the runner drawer");
    (min, (max.0, window_h))
}

/// What changes from run to run: a time depends on how fast it ran.
fn varying(harness: &Harness<'static, MortiseApp>, times: &[&str]) -> Vec<Part> {
    times
        .iter()
        .map(|text| label_bounds(harness, text))
        .collect()
}

#[test]
fn runner_drawer() {
    let failed = || Ended::Failed {
        code: Some(1),
        ms: 412,
    };
    let done = |ms: u64| Ended::Done { ms };
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        let window_h = 800.0;

        // f. Nothing has run yet.
        let desk = drawer_desk(theme, 1280.0, "", &[]);
        let mut harness = desk.start();
        harness.key_press_modifiers(Modifiers::CTRL, Key::Backtick);
        harness.hover_at(eframe::egui::pos2(700.0, 20.0));
        harness.run_steps(3);
        let part = drawer_part(&harness, window_h);
        shots.take_part(&mut harness, "s11.1f-empty", part);

        // d. The history, with the output of the latest run on the right.
        let runs: [Seeded<'_>; 5] = [
            (
                &["module:migrate", "Blog"],
                done(186),
                (14, 5),
                recording("module-migrate"),
            ),
            (
                &["module:make-controller", "PostController", "Blog"],
                failed(),
                (14, 3),
                recording("module-make-controller-exists"),
            ),
            (
                &["module:make", "Blog"],
                done(4200),
                (13, 58),
                recording("module-make"),
            ),
            (
                &["module:migrate", "Shipping"],
                failed(),
                (13, 51),
                recording("module-migrate-failing-migration"),
            ),
            (
                &["module:seed", "Blog"],
                done(388),
                (13, 40),
                recording("module-seed"),
            ),
        ];
        let desk = drawer_desk(theme, 1280.0, "", &runs);
        let mut harness = desk.start();
        harness.key_press_modifiers(Modifiers::CTRL, Key::Backtick);
        harness.hover_at(eframe::egui::pos2(700.0, 20.0));
        harness.run_steps(3);
        let part = drawer_part(&harness, window_h);
        // The line that Run again would run stands above the output of the selected run.
        shots.take_part(&mut harness, "s11.1d-history", part);

        // b. Finished: the latest run again.
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
        wait_for(&mut harness, ": Done in ");
        let masks = varying(&harness, &[": Done in "]);
        shots.take_part_masked(&mut harness, "s11.1b-done", part, &masks);

        // e. Collapsed to one line.
        harness.key_press_modifiers(Modifiers::CTRL, Key::Backtick);
        harness.run_steps(3);
        harness.get_by_role_and_label(Role::Button, "Show");
        let line = ((part.0.0, window_h - 28.0), (part.1.0, window_h));
        let masks = vec![label_bounds(&harness, ": Done in ")];
        shots.take_part_masked(&mut harness, "s11.1e-collapsed", line, &masks);

        // c. Failed: the fifth run of the history again, under the run of b.
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::H);
        harness.run_steps(3);
        for _ in 0..4 {
            harness.key_press(Key::ArrowDown);
            harness.run_steps(2);
        }
        harness.key_press(Key::Enter);
        wait_for(&mut harness, "Shipping: Exit code 1");
        let time = exit_time(&harness);
        let masks = varying(&harness, &[&time]);
        shots.take_part_masked(&mut harness, "s11.1c-failed", part, &masks);

        // a. Running: a migration that holds after its output. The elapsed time is cut out.
        let desk = drawer_desk(
            theme,
            1280.0,
            " hold",
            &[(&["module:migrate", "--all"], done(186), (14, 5), Vec::new())],
        );
        let mut harness = desk.start();
        // The first press opens the drawer on the line it runs; the second runs it.
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
        harness.run_steps(3);
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
        wait_for(&mut harness, "Running Migration Shop Module");
        wait_for(&mut harness, "create_items_table");
        let masks = varying(&harness, &["Running, "]);
        let part = drawer_part(&harness, window_h);
        shots.take_part_masked(&mut harness, "s11.1a-running", part, &masks);
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Period);
        harness.run_steps(3);
        harness
            .get_by_role_and_label(Role::Button, "Stop command")
            .click();
        wait_for(&mut harness, "Stopped after");

        // g. Failed, at the narrowest drawer: the narrowest window, the index a strip.
        let desk = drawer_desk(
            theme,
            mortise_app::ui::theme::size::WINDOW_MIN_W,
            "",
            &[(
                &["module:migrate", "Shipping"],
                failed(),
                (14, 5),
                Vec::new(),
            )],
        );
        let mut harness = desk.start();
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
        harness.run_steps(3);
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::R);
        wait_for(&mut harness, "Shipping: Exit code 1");
        let time = exit_time(&harness);
        let masks = varying(&harness, &[&time]);
        let part = drawer_part(&harness, window_h);
        shots.take_part_masked(&mut harness, "s11.1g-failed-narrow", part, &masks);
    }
}

/// The time beside an exit code: `412 ms`, as this run took. A label keeps its text as its value.
fn exit_time(harness: &Harness<'static, MortiseApp>) -> String {
    harness
        .query_all_by_role(Role::Label)
        .filter_map(|node| node.accesskit_node().value())
        .find(|label| {
            label
                .strip_suffix(" ms")
                .or_else(|| label.strip_suffix(" s"))
                .is_some_and(|number| number.parse::<f64>().is_ok())
        })
        .expect("the time of the run")
}

// --- The Metrics tab (S7.1, S7.2 and the plain table of milestone 20) ---

/// The answer of `prometheus` when the counters stand at `n`: requests of Checkout and
/// Billing, and jobs of Billing. A request has buckets from 50 ms to 1 s, and some of
/// Checkout end in 500.
fn prometheus(n: u32) -> String {
    let request = |status: &str, route: &str, count: u32| {
        let labels = format!(
            r#"http_request_method="GET",http_response_status_code="{status}",http_route="{route}""#
        );
        let under = |share: u32| count * share / 100;
        let mut text = String::new();
        for (bound, share) in [
            ("0.05", 40),
            ("0.1", 70),
            ("0.25", 90),
            ("0.5", 97),
            ("1", 99),
        ] {
            text += &format!(
                "http_server_request_duration_seconds_bucket{{{labels},le=\"{bound}\"}} {}\n",
                under(share)
            );
        }
        text + &format!(
            "http_server_request_duration_seconds_bucket{{{labels},le=\"+Inf\"}} {count}\n\
             http_server_request_duration_seconds_sum{{{labels}}} {}\n\
             http_server_request_duration_seconds_count{{{labels}}} {count}\n",
            f64::from(count) * 0.12
        )
    };
    let job = |name: &str, count: u32| {
        let labels = format!(r#"laravel_job_name="Modules\\Billing\\Jobs\\{name}""#);
        format!(
            "messaging_process_duration_seconds_bucket{{{labels},le=\"+Inf\"}} {count}\n\
             messaging_process_duration_seconds_sum{{{labels}}} {}\n\
             messaging_process_duration_seconds_count{{{labels}}} {count}\n",
            f64::from(count) * 0.1
        )
    };
    format!(
        "# TYPE http_server_request_duration_seconds histogram\n{}{}{}{}\
         # TYPE messaging_process_duration_seconds histogram\n{}{}",
        request("200", "/api/checkout/things/0", 100 + 30 * n),
        request("500", "/api/checkout/things/0", 3 * n),
        request("200", "/api/checkout/things/1", 40 + 12 * n),
        request("200", "/billing/things/2", 20 + 6 * n),
        job("SendInvoice", 10 + 3 * n),
        job("ChargeInvoice", 5 + 2 * n),
    )
}

/// The counters after `reads` reads: read `k` adds `1 + (k % 240) / 60`, so the rate
/// rises and falls in waves of 1 hour.
fn counters_after(reads: u32) -> u32 {
    let (whole, rest) = (reads / 240, reads % 240);
    let (quarters, left) = (rest / 60, rest % 60);
    // A wave of 240 reads adds 240 + 60 * (0 + 1 + 2 + 3).
    whole * 420 + rest + 30 * quarters * quarters.saturating_sub(1) + quarters * left
}

/// shop-api connected to the stub as "local", in a theme.
fn connected(theme: Theme, stub: &StubActuator) -> (Desk, Harness<'static, MortiseApp>) {
    let mut desk = Desk::new();
    desk.theme = theme;
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(&[common::environment("local", stub)]);
    let mut harness = desk.start();
    common::wait_for_answer(&mut harness);
    (desk, harness)
}

#[test]
fn the_metrics_tab() {
    // One set of results for the test: both themes are compared when it ends.
    let mut shots = Shots::new();
    for (theme, name) in THEMES {
        shots.theme = name;
        // S7.1: no environment.
        let (_desk, mut harness) = shop_api(theme);
        press(&mut harness, Key::Num5);
        shots.take(&mut harness, "s7.1-metrics-not-connected");

        // S7.2: the links of the environment lack `prometheus`.
        let stub = common::shop_api_actuator();
        let (_desk, mut harness) = connected(theme, &stub);
        press(&mut harness, Key::Num5);
        wait_for(&mut harness, "Metrics are not exposed on local");
        rest(&mut harness);
        shots.take(&mut harness, "s7.2-metrics-not-exposed");

        // Reads 15 s apart on a clock the test moves, with the counters growing by steps.
        let stub = common::shop_api_actuator();
        stub.expose(&["health", "info", "mappings", "scheduledtasks", "prometheus"]);
        stub.set("/actuator/prometheus", Reply::ok(&prometheus(0)));
        let now = Arc::new(AtomicU64::new(0));
        let clock = Arc::clone(&now);
        let mut desk = Desk::new();
        desk.theme = theme;
        desk.metrics_clock = Some(Arc::new(move || {
            Duration::from_secs(clock.load(Ordering::SeqCst))
        }));
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        desk.connect(&[common::environment("local", &stub)]);
        let mut harness = desk.start();
        common::wait_for_answer(&mut harness);
        press(&mut harness, Key::Num5);
        wait_for(&mut harness, "A rate needs two reads");
        let mut counted = 0;
        for read in 0..40_u32 {
            counted += 1 + read * 7 % 9;
            stub.set("/actuator/prometheus", Reply::ok(&prometheus(counted)));
            now.store(15 * (u64::from(read) + 1), Ordering::SeqCst);
            harness.key_press_modifiers(Modifiers::COMMAND, Key::R);
            let deadline = std::time::Instant::now() + Duration::from_secs(20);
            while stub
                .asked()
                .iter()
                .filter(|path| path.ends_with("/prometheus"))
                .count()
                < read as usize + 2
            {
                assert!(std::time::Instant::now() < deadline, "read {read}");
                std::thread::sleep(Duration::from_millis(20));
                harness.run_steps(2);
            }
        }
        rest(&mut harness);
        shots.take(&mut harness, "s7.3-metrics-all");

        // One module of the index: its routes, and its name over the chart.
        select(&mut harness, "Billing");
        shots.take(&mut harness, "s7.3-metrics-module");
    }

    // A day: 15 s per read, 5,700 reads, as fast as the stub answers. The clock of the reads
    // counts the requests, so a read is 15 s after the one before. The clock is read before
    // the request, so the read of number `DAY + 1` is the last one with new counters. Later
    // reads have its time and are not kept. The fill takes about 10 s in a debug build.
    // 5,700 reads, not 5,760: the chart of 5,760 ends in a read that shows 0 now (see the report).
    const DAY: u64 = 5_700;
    for (theme, name) in THEMES {
        shots.theme = name;
        let stub = common::shop_api_actuator();
        stub.expose(&["health", "info", "mappings", "scheduledtasks", "prometheus"]);
        // The reads the stub answered, which is what the clock counts.
        let asked = Arc::new(AtomicU64::new(0));
        let served = Arc::clone(&asked);
        stub.serve("/actuator/prometheus", move |_| {
            let read = served.fetch_add(1, Ordering::SeqCst) + 1;
            Reply::ok(&prometheus(counters_after(read.min(DAY + 1) as u32)))
        });
        let mut desk = Desk::new();
        desk.theme = theme;
        desk.pace = mortise_core::Pace {
            metrics: Duration::from_millis(1),
            ..mortise_core::Pace::default()
        };
        let counter = Arc::clone(&asked);
        desk.metrics_clock = Some(Arc::new(move || {
            Duration::from_secs(15 * counter.load(Ordering::SeqCst).min(DAY))
        }));
        let root = desk.app("shop-api");
        desk.knows(&[(&root, "SA")]);
        desk.connect(&[common::environment("local", &stub)]);
        let mut harness = desk.start();
        common::wait_for_answer(&mut harness);
        press(&mut harness, Key::Num5);
        let began = std::time::Instant::now();
        while asked.load(Ordering::SeqCst) < DAY + 2 {
            assert!(began.elapsed() < Duration::from_secs(120), "a day of reads");
            harness.run_steps(2);
        }
        eprintln!("a day of reads took {:?}", began.elapsed());
        harness
            .get_by_role_and_label(Role::RadioButton, "24 hours")
            .click();
        rest(&mut harness);
        shots.take(&mut harness, "s7.3-metrics-day");
    }
}
