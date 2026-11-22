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

/// Waits until `done` holds for the app, for at most 10 s. Work on other threads
/// (reading an app, the file watcher) needs real time to pass, but few frames:
/// each frame moves the clock of the interface on, and a toast leaves after 4 s of it.
pub fn wait_until(
    harness: &mut Harness<'static, MortiseApp>,
    what: &str,
    done: impl Fn(&MortiseApp) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    harness.step();
    let mut waited = 0;
    while !done(harness.state()) {
        assert!(Instant::now() < deadline, "waited 10 s for: {what}");
        std::thread::sleep(Duration::from_millis(10));
        waited += 1;
        if waited % 25 == 0 {
            harness.step();
        }
    }
    // A few frames, so that what the app took in is on screen.
    harness.run_steps(3);
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

/// Makes the app a git repository with a few commits in the last 30 days.
pub fn with_git_history(root: &Path) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let git = |days_ago: i64, args: &[&str]| {
        let date = format!("{} +0000", now - days_ago * 86_400);
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args([
                "-c",
                "user.name=Mortise Test",
                "-c",
                "user.email=test@mortise.invalid",
            ])
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_AUTHOR_DATE", &date)
            .env("GIT_COMMITTER_DATE", &date)
            .output()
            .expect("git is installed");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(70, &["init", "-q", "-b", "main"]);
    git(70, &["add", "-A"]);
    git(70, &["commit", "-q", "-m", "first"]);
    for (days_ago, files) in [
        (
            21,
            vec![
                "Modules/Checkout/routes/web.php",
                "Modules/Shipping/routes/web.php",
            ],
        ),
        (9, vec!["Modules/Checkout/routes/api.php"]),
        (
            2,
            vec![
                "Modules/Checkout/config/config.php",
                "Modules/Billing/routes/web.php",
            ],
        ),
    ] {
        for file in files {
            let path = root.join(file);
            let mut content = std::fs::read_to_string(&path).unwrap();
            content.push_str("\n// changed\n");
            std::fs::write(path, content).unwrap();
        }
        git(days_ago, &["add", "-A"]);
        git(days_ago, &["commit", "-q", "-m", "change"]);
    }
}

/// Copies the shop-api fixture and gives it `count` modules, by copying Catalog
/// under other names.
pub fn app_with_modules(desk: &Desk, folder: &str, count: usize) -> PathBuf {
    let root = desk.app(folder);
    let names = [
        "Accounts",
        "Addresses",
        "Audit",
        "Carts",
        "Coupons",
        "Currencies",
        "Customers",
        "Discounts",
        "Exports",
        "Fulfilment",
        "GiftCards",
        "Imports",
        "Invoices",
        "Labels",
        "Loyalty",
        "Orders",
        "Payments",
        "Pricing",
        "Products",
        "Promotions",
        "Refunds",
        "Reports",
        "Returns",
        "Reviews",
        "Search",
        "Subscriptions",
        "Taxes",
        "Vouchers",
        "Warehouses",
        "Wishlists",
    ];
    let mut statuses: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("modules_statuses.json")).unwrap())
            .unwrap();
    for i in 7..count {
        let name = match names.get(i - 7) {
            Some(name) => (*name).to_owned(),
            None => format!("Extra{i}"),
        };
        let dir = root.join("Modules").join(&name);
        copy_dir(&root.join("Modules/Catalog"), &dir);
        rename_in_files(&dir, "Catalog", &name);
        statuses[&name] = serde_json::Value::Bool(i % 9 != 0);
    }
    std::fs::write(root.join("modules_statuses.json"), statuses.to_string()).unwrap();
    root
}

/// Replaces a module name in every file under a folder, so that a copied module
/// has its own name and namespace.
fn rename_in_files(dir: &Path, from: &str, to: &str) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rename_in_files(&path, from, to);
        } else if let Ok(text) = std::fs::read_to_string(&path) {
            let text = text
                .replace(from, to)
                .replace(&from.to_lowercase(), &to.to_lowercase());
            std::fs::write(&path, text).unwrap();
        }
    }
}
