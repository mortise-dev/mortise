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
use mortise_core::testing::{Reply, StubActuator};
use mortise_core::{AppEntry, Credentials, Environment, Secrets, Settings};

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
    /// How long the window waits for a folder before it goes on without the result.
    /// Long, so that a test sees the folder in the frame it chose it in.
    pub quick_read: Duration,
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
            quick_read: Duration::from_secs(30),
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
                    problems_first: false,
                    coverage_limit: 50,
                    coverage_report: None,
                    environments: Vec::new(),
                    environment: None,
                })
                .collect(),
            current: Some(0),
            general: mortise_core::General::default(),
        };
        settings.save(&self.settings_path()).unwrap();
    }

    /// Starts Mortise headless and waits until every app it knows is read.
    pub fn start(&self) -> Harness<'static, MortiseApp> {
        self.start_with(None)
    }

    /// Starts Mortise with an app folder on the command line.
    pub fn start_with(&self, folder: Option<PathBuf>) -> Harness<'static, MortiseApp> {
        // The theme of a test is a choice in Settings, as it is for a user: left to
        // "System", a headless run has no system to follow.
        // A settings file that cannot be read is left for the app to deal with.
        if let Ok(mut settings) = Settings::load(&self.settings_path())
            && settings.general.theme == mortise_core::ThemeChoice::System
        {
            settings.general.theme = match self.theme {
                Theme::Light => mortise_core::ThemeChoice::Light,
                Theme::Dark => mortise_core::ThemeChoice::Dark,
            };
            settings.save(&self.settings_path()).unwrap();
        }
        let picks = Rc::clone(&self.picks);
        let config = Config {
            chrome: self.chrome,
            settings_path: Some(self.settings_path()),
            folder,
            pick_folder: Box::new(move || picks.borrow_mut().pop_front()),
            // Never the credential store of the computer the tests run on.
            secrets: Secrets::memory(),
            quick_read: self.quick_read,
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

    /// Gives the first app Mortise knows these environments, with the first one chosen.
    pub fn connect(&self, environments: &[Environment]) {
        let mut settings = self.saved();
        settings.apps[0].environments = environments.to_vec();
        settings.apps[0].environment = environments.first().map(|e| e.name.clone());
        settings.save(&self.settings_path()).unwrap();
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

/// An environment without credentials at the address of a stub.
pub fn environment(name: &str, stub: &StubActuator) -> Environment {
    Environment {
        name: name.to_owned(),
        base_url: stub.base_url.clone(),
        path: "actuator".to_owned(),
        credentials: Credentials::None,
    }
}

/// A desk with shop-api known, open and connected to a stub as "local", started
/// and waited for until the stub answered.
pub fn connected(stub: &StubActuator) -> (Desk, Harness<'static, MortiseApp>) {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(&[environment("local", stub)]);
    let mut harness = desk.start();
    wait_for_answer(&mut harness);
    (desk, harness)
}

pub fn wait_for_answer(harness: &mut Harness<'static, MortiseApp>) {
    wait_until(harness, "the environment answers", |app| {
        let state = app.runtime();
        state.answer.is_some() && !state.asking
    });
}

/// Seconds since 1970 as ISO 8601 in UTC, as the actuator writes a next run.
pub fn iso(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let (hour, minute) = (
        seconds.rem_euclid(86_400) / 3600,
        seconds.rem_euclid(3600) / 60,
    );
    // Days since 1970 to a date of the calendar.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:00+00:00")
}

/// A stub that answers as the shop-api of the specification's drawings: everything
/// exposed and up, 106 routes of which 12 are in no module, and 5 scheduled tasks.
pub fn shop_api_actuator() -> StubActuator {
    let stub = StubActuator::healthy();
    let mut routes = Vec::new();
    let mut add = |uri: String, name: Option<String>, action: String, middleware: &str| {
        let name = name.map_or("null".to_owned(), |name| format!("\"{name}\""));
        routes.push(format!(
            r#"{{"methods":["GET","HEAD"],"uri":"{uri}","name":{name},"action":"{}","middleware":[{middleware}],"domain":null}}"#,
            action.replace('\\', "\\\\")
        ));
    };
    let modules = [
        ("Catalog", 22),
        ("Checkout", 18),
        ("Users", 16),
        ("Billing", 14),
        ("Shipping", 11),
        ("Inventory", 9),
        ("Notifications", 4),
    ];
    for (module, count) in modules {
        let lower = module.to_lowercase();
        for i in 0..count {
            // Two thirds of the routes of a module are API routes.
            let (prefix, middleware) = if i % 3 == 2 {
                ("", r#""web","auth""#)
            } else {
                ("api/", r#""api","auth:sanctum""#)
            };
            add(
                format!("{prefix}{lower}/things/{i}"),
                Some(format!("{lower}.things.{i}")),
                format!(r"Modules\{module}\Http\Controllers\ThingController@show{i}"),
                middleware,
            );
        }
    }
    for i in 0..10 {
        add(format!("up/{i}"), None, "Closure".to_owned(), "");
    }
    add(
        "actuator".to_owned(),
        Some("actuator".to_owned()),
        r"Nwidart\Actuator\Http\Controllers\DiscoveryController".to_owned(),
        "",
    );
    add(
        "actuator/health/{path?}".to_owned(),
        Some("actuator.health".to_owned()),
        r"Nwidart\Actuator\Http\Controllers\HealthController".to_owned(),
        "",
    );
    stub.set(
        "/actuator/mappings",
        Reply::ok(&format!(r#"{{"routes":[{}]}}"#, routes.join(","))),
    );

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let today = now - now.rem_euclid(86_400);
    // Later today when there is a later today, and tomorrow at six.
    let soon = iso((now + 600).min(today + 86_399));
    let tomorrow = iso(today + 86_400 + 6 * 3600);
    let cron = |target: &str, expression: &str, next: &str| {
        format!(
            r#"{{"runnable":{{"target":"{target}"}},"expression":"{expression}","timezone":"UTC","nextExecution":{{"time":"{next}"}}}}"#
        )
    };
    let tasks = [
        cron("php artisan checkout:expire-carts", "*/5 * * * *", &soon),
        cron("php artisan billing:send-invoices", "0 6 * * *", &tomorrow),
        cron(r"Modules\\Inventory\\Jobs\\SyncStock", "0 * * * *", &soon),
        cron("php artisan queue:prune-batches", "0 3 * * *", &tomorrow),
    ];
    stub.set(
        "/actuator/scheduledtasks",
        Reply::ok(&format!(
            r#"{{"cron":[{}],"fixedDelay":[],"fixedRate":[{{"runnable":{{"target":"Poll shipment tracking"}},"initialDelay":0,"interval":10000}}],"custom":[]}}"#,
            tasks.join(",")
        )),
    );
    stub
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

/// Gives a module more PHP files, so that reading it takes more than one frame.
pub fn pad_module(root: &Path, module: &str, files: usize) {
    let dir = root.join("Modules").join(module).join("app/Services");
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..files {
        let source = format!(
            "<?php\n\nnamespace Modules\\{module}\\Services;\n\nclass Service{i}\n{{\n    public function run(): int\n    {{\n        return {i};\n    }}\n}}\n"
        );
        std::fs::write(dir.join(format!("Service{i}.php")), source).unwrap();
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
