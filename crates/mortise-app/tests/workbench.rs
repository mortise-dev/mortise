//! The workbench, run headless: what is on screen, and what the keys and the pointer do.

use std::path::{Path, PathBuf};

use eframe::egui::{self, Key, Role, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use mortise_app::app::{MortiseApp, Selection};
use mortise_app::chrome::Chrome;
use mortise_app::workspace::Tab;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

fn workbench(folder: Option<&Path>, theme: Theme, chrome: Chrome) -> Harness<'static, MortiseApp> {
    let folder = folder.map(Path::to_owned);
    Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .with_pixels_per_point(1.0)
        .with_theme(theme)
        .with_os(egui::os::OperatingSystem::Nix)
        .build_eframe(move |cc| MortiseApp::new(cc, chrome, folder.as_deref()))
}

fn shop_api() -> Harness<'static, MortiseApp> {
    workbench(
        Some(&fixture("shop-api")),
        Theme::Light,
        Chrome::Custom {
            controls_left: false,
        },
    )
}

/// A copy of shop-api with Notifications turned off.
struct Copy {
    _dir: tempfile::TempDir,
    app: PathBuf,
}

impl Copy {
    fn path(&self) -> &Path {
        &self.app
    }
}

fn shop_api_with_a_module_off() -> Copy {
    let dir = tempfile::tempdir().unwrap();
    // The folder keeps its name, because the name of the app is the name of its folder.
    let app = dir.path().join("shop-api");
    copy_dir(&fixture("shop-api"), &app);
    let statuses = app.join("modules_statuses.json");
    let json = std::fs::read_to_string(&statuses).unwrap();
    std::fs::write(
        statuses,
        json.replace("\"Notifications\": true", "\"Notifications\": false"),
    )
    .unwrap();
    Copy { _dir: dir, app }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn shows_the_app_and_its_modules() {
    let harness = shop_api();

    harness.get_by_role_and_label(Role::Heading, "shop-api");
    harness.get_by_label("7 of 7 on");
    harness.get_by_label("Laravel 13.2, package 13.3.0");
    harness.get_by_label("Sorted: by name");

    // "All modules" first, then the modules by name.
    let rows: Vec<_> = harness
        .get_all_by_role(Role::ListBoxOption)
        .filter_map(|row| row.accesskit_node().label())
        .collect();
    assert_eq!(
        rows,
        [
            "shop-api", // the tile on the rail
            "All modules",
            "Billing",
            "Catalog",
            "Checkout",
            "Inventory",
            "Notifications",
            "Shipping",
            "Users"
        ]
    );

    // The home screen: "All modules", Summary.
    assert_eq!(harness.state().selection(), &Selection::All);
    assert_eq!(harness.state().tab(), Tab::Summary);
    harness.get_by_role_and_label(Role::Heading, "All modules");
    harness.get_by_label("Modules on");
    harness.get_by_label("7 of 7");
}

#[test]
fn j_and_k_move_through_the_modules_and_the_workspace_follows() {
    let mut harness = shop_api();

    harness.key_press(Key::J);
    harness.run();
    assert_eq!(
        harness.state().selection(),
        &Selection::Module("Billing".into())
    );
    harness.get_by_role_and_label(Role::Heading, "Billing");
    harness.get_by_label("Modules/Billing");
    harness.get_by_label("Module state");
    harness.get_by_label("On");

    harness.key_press(Key::ArrowDown);
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(
        harness.state().selection(),
        &Selection::Module("Checkout".into())
    );

    harness.key_press(Key::K);
    harness.run();
    assert_eq!(
        harness.state().selection(),
        &Selection::Module("Catalog".into())
    );

    harness.key_press(Key::End);
    harness.run();
    assert_eq!(
        harness.state().selection(),
        &Selection::Module("Users".into())
    );
    // The last row stays the last row.
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(
        harness.state().selection(),
        &Selection::Module("Users".into())
    );

    harness.key_press(Key::Home);
    harness.run();
    assert_eq!(harness.state().selection(), &Selection::All);
    harness.key_press(Key::ArrowUp);
    harness.run();
    assert_eq!(harness.state().selection(), &Selection::All);
}

#[test]
fn the_digits_open_the_tabs_and_the_tab_is_kept_while_moving() {
    let mut harness = shop_api();

    for (key, tab) in [
        (Key::Num2, Tab::Boundary),
        (Key::Num3, Tab::Generate),
        (Key::Num4, Tab::Actions),
        (Key::Num5, Tab::Metrics),
        (Key::Num1, Tab::Summary),
    ] {
        harness.key_press(key);
        harness.run();
        assert_eq!(harness.state().tab(), tab);
    }

    harness.key_press(Key::Num3);
    harness.key_press(Key::J);
    harness.run();
    assert_eq!(harness.state().tab(), Tab::Generate);
    assert_eq!(
        harness.state().selection(),
        &Selection::Module("Billing".into())
    );
}

#[test]
fn a_tab_of_a_later_phase_explains_itself() {
    let mut harness = shop_api();
    for (tab, title) in [
        ("Generate", "Generate is not in this version"),
        ("Actions", "Actions are not in this version"),
        ("Metrics", "Metrics are not in this version"),
    ] {
        harness.get_by_role_and_label(Role::Tab, tab).click();
        harness.run();
        harness.get_by_role_and_label(Role::Heading, title);
    }
    // Every tab is always there, and none is disabled.
    assert_eq!(harness.get_all_by_role(Role::Tab).count(), 5);
}

#[test]
fn a_click_selects_a_module() {
    let mut harness = shop_api();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Shipping")
        .click();
    harness.run();
    assert_eq!(
        harness.state().selection(),
        &Selection::Module("Shipping".into())
    );
    harness.get_by_role_and_label(Role::Heading, "Shipping");
}

#[test]
fn a_module_that_is_turned_off_says_so() {
    let app = shop_api_with_a_module_off();
    let mut harness = workbench(Some(app.path()), Theme::Light, Chrome::Mac);

    harness.get_by_label("6 of 7 on");
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Notifications")
        .click();
    harness.run();
    harness.get_by_label("Module state");
    harness.get_by_label("Off");
}

#[test]
fn without_a_folder_it_says_how_to_start() {
    let harness = workbench(None, Theme::Light, Chrome::Mac);
    harness.get_by_role_and_label(Role::Heading, "No apps yet");
    assert_eq!(harness.query_all_by_role(Role::ListBoxOption).count(), 0);
}

#[test]
fn a_folder_that_is_not_an_app_says_why() {
    let dir = tempfile::tempdir().unwrap();
    let harness = workbench(Some(dir.path()), Theme::Light, Chrome::Mac);
    harness.get_by_role_and_label(Role::Heading, "This folder was not added");
    harness.get_by_label_contains("The folder has no artisan file");
}

#[test]
fn the_window_controls_are_drawn_only_where_mortise_draws_the_frame() {
    let custom = shop_api();
    for name in ["Minimise", "Maximise", "Close"] {
        custom.get_by_role_and_label(Role::Button, name);
    }
    let mac = workbench(Some(&fixture("shop-api")), Theme::Light, Chrome::Mac);
    assert_eq!(mac.query_all_by_role(Role::Button).count(), 0);
}

/// Renders the screens of this milestone to `target/screens`, to be looked at beside
/// the drawings of the specification. Run with `-- --ignored screens`.
#[test]
#[ignore = "writes PNG files for a person to look at"]
fn screens() {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screens");
    std::fs::create_dir_all(&out).unwrap();
    let off = shop_api_with_a_module_off();
    let save = |name: &str, harness: &mut Harness<'_, MortiseApp>| {
        harness.run();
        harness
            .render()
            .unwrap()
            .save(out.join(format!("{name}.png")))
            .unwrap();
    };

    for (theme, suffix) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
        let mut harness = workbench(Some(&fixture("shop-api")), theme, Chrome::Mac);
        save(&format!("s3.1-all-mac-{suffix}"), &mut harness);

        let mut harness = workbench(
            Some(off.path()),
            theme,
            Chrome::Custom {
                controls_left: false,
            },
        );
        harness.key_press(Key::J);
        harness.key_press(Key::J);
        harness.key_press(Key::J);
        save(&format!("s3.1-module-custom-{suffix}"), &mut harness);

        harness.key_press(Key::Num3);
        save(&format!("s7.4-generate-{suffix}"), &mut harness);
    }

    let mut harness = workbench(
        Some(off.path()),
        Theme::Light,
        Chrome::Custom {
            controls_left: true,
        },
    );
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Notifications")
        .click();
    save("s3.1-off-selected-custom-left-light", &mut harness);

    let mut harness = workbench(None, Theme::Light, Chrome::System);
    save("no-app-system-light", &mut harness);

    let mut harness = Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .with_pixels_per_point(2.0)
        .with_theme(Theme::Light)
        .build_eframe(|cc| MortiseApp::new(cc, Chrome::Mac, Some(&fixture("shop-api"))));
    harness.key_press(Key::J);
    save("s3.1-module-mac-light-2x", &mut harness);
}
