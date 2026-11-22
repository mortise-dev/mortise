//! What the interface tests share: fixture apps in a temporary folder and the app run headless.
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Theme};
use egui_kittest::Harness;
use mortise_app::app::{Config, MortiseApp};
use mortise_app::chrome::Chrome;
use mortise_core::{AppEntry, Settings};

pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

pub fn copy_dir(from: &Path, to: &Path) {
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

/// A temporary folder that holds copies of fixture apps and Mortise's settings file.
pub struct Desk {
    dir: tempfile::TempDir,
    /// The folders the folder dialog will give, in order.
    pub picks: Rc<RefCell<VecDeque<PathBuf>>>,
    pub theme: Theme,
    pub chrome: Chrome,
    pub size: egui::Vec2,
    pub scale: f32,
    pub os: egui::os::OperatingSystem,
}

impl Desk {
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
            picks: Rc::default(),
            theme: Theme::Light,
            chrome: Chrome::Custom {
                controls_left: false,
            },
            size: egui::vec2(1280.0, 800.0),
            scale: 1.0,
            os: egui::os::OperatingSystem::Nix,
        }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn settings_path(&self) -> PathBuf {
        self.dir.path().join("config").join("settings.json")
    }

    /// Copies the shop-api fixture into a folder with the given name.
    pub fn app(&self, folder: &str) -> PathBuf {
        let root = self.dir.path().join(folder);
        copy_dir(&fixture("shop-api"), &root);
        root
    }

    /// Writes a settings file that knows these apps, with the first one open.
    pub fn knows(&self, apps: &[(&Path, &str)]) {
        let settings = Settings {
            apps: apps
                .iter()
                .map(|(root, monogram)| AppEntry {
                    root: root.to_path_buf(),
                    name: root.file_name().unwrap().to_string_lossy().into_owned(),
                    monogram: (*monogram).to_owned(),
                    codes: BTreeMap::new(),
                })
                .collect(),
            current: Some(0),
        };
        settings.save(&self.settings_path()).unwrap();
    }

    /// Starts Mortise headless and waits until every app it knows is read.
    pub fn start(&self) -> Harness<'static, MortiseApp> {
        self.start_with(None)
    }

    /// Starts Mortise with an app folder on the command line.
    pub fn start_with(&self, folder: Option<PathBuf>) -> Harness<'static, MortiseApp> {
        let picks = Rc::clone(&self.picks);
        let config = Config {
            chrome: self.chrome,
            settings_path: Some(self.settings_path()),
            folder,
            pick_folder: Box::new(move || picks.borrow_mut().pop_front()),
        };
        let mut harness = Harness::builder()
            .with_size(self.size)
            .with_pixels_per_point(self.scale)
            .with_theme(self.theme)
            .with_os(self.os)
            .build_eframe(move |cc| MortiseApp::new(cc, config));
        wait_until(&mut harness, "every app is read", |app| app.is_read());
        harness
    }

    pub fn saved(&self) -> Settings {
        Settings::load(&self.settings_path()).unwrap()
    }
}

/// A desk with shop-api known and open, started.
pub fn shop_api() -> (Desk, Harness<'static, MortiseApp>) {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    let harness = desk.start();
    (desk, harness)
}

/// Runs frames until `done` holds for the app, for at most 10 s. Work on other
/// threads (reading an app, the file watcher) needs real time to pass.
pub fn wait_until(
    harness: &mut Harness<'static, MortiseApp>,
    what: &str,
    done: impl Fn(&MortiseApp) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        harness.step();
        if done(harness.state()) {
            // A few more frames, so that what the app took in is on screen.
            harness.run_steps(3);
            return;
        }
        assert!(Instant::now() < deadline, "waited 10 s for: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn turn_off(root: &Path, module: &str) {
    let statuses = root.join("modules_statuses.json");
    let json = std::fs::read_to_string(&statuses).unwrap();
    let json = json.replace(
        &format!("\"{module}\": true"),
        &format!("\"{module}\": false"),
    );
    std::fs::write(statuses, json).unwrap();
}
