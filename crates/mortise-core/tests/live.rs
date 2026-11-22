//! An app that is kept fresh while its files change.
#![cfg(feature = "watch")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use mortise_core::{AppSnapshot, LiveApp, LiveState, ModuleState, ReadError, ReadOptions};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
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

struct Opened {
    live: LiveApp,
    changed: Receiver<()>,
    _dir: tempfile::TempDir,
    root: PathBuf,
}

fn open(codes: BTreeMap<String, String>) -> Opened {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("shop-api");
    copy_dir(&fixture("shop-api"), &root);
    let (tell, changed) = channel();
    let live = LiveApp::open(root.clone(), codes, ReadOptions::default(), move || {
        let _ = tell.send(());
    });
    Opened {
        live,
        changed,
        _dir: dir,
        root,
    }
}

impl Opened {
    /// Turns the watcher on and waits until it is in place.
    fn start_watching(&self) {
        self.live.watch(true);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !self.live.is_watching() {
            assert!(Instant::now() < deadline, "waited 10 s for the watcher");
            std::thread::sleep(Duration::from_millis(10));
        }
        // The operating system needs a moment before it reports changes.
        std::thread::sleep(Duration::from_millis(300));
    }

    /// Waits until the app is read and `wanted` holds for the snapshot.
    fn wait_for(
        &self,
        what: &str,
        wanted: impl Fn(&AppSnapshot) -> bool,
    ) -> std::sync::Arc<AppSnapshot> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let LiveState::Ready(app) = self.live.state()
                && wanted(&app)
            {
                return app;
            }
            let left = deadline.saturating_duration_since(Instant::now());
            assert!(!left.is_zero(), "waited 10 s for: {what}");
            let _ = self.changed.recv_timeout(left);
        }
    }
}

#[test]
fn the_first_read_arrives_and_tells() {
    let opened = open(BTreeMap::new());
    let app = opened.wait_for("the first read", |_| true);
    assert_eq!(app.modules.len(), 7);
}

#[test]
fn a_folder_that_is_not_an_app_fails_and_tells() {
    let dir = tempfile::tempdir().unwrap();
    let (tell, changed) = channel();
    let live = LiveApp::open(
        dir.path().to_owned(),
        BTreeMap::new(),
        ReadOptions::default(),
        move || {
            let _ = tell.send(());
        },
    );
    changed.recv_timeout(Duration::from_secs(10)).unwrap();
    let LiveState::Failed(error) = live.state() else {
        panic!("expected a failure");
    };
    assert!(matches!(*error, ReadError::NotLaravel { .. }));
}

#[test]
fn refresh_reads_again_without_a_watcher() {
    let opened = open(BTreeMap::new());
    opened.wait_for("the first read", |_| true);

    std::fs::remove_dir_all(opened.root.join("Modules/Users")).unwrap();
    opened.live.refresh();
    opened.wait_for("six modules", |app| app.modules.len() == 6);
}

#[test]
fn a_change_of_the_statuses_file_is_seen() {
    let opened = open(BTreeMap::new());
    opened.wait_for("the first read", |_| true);
    opened.start_watching();

    let statuses = opened.root.join("modules_statuses.json");
    let json = std::fs::read_to_string(&statuses).unwrap();
    std::fs::write(
        &statuses,
        json.replace("\"Checkout\": true", "\"Checkout\": false"),
    )
    .unwrap();

    let app = opened.wait_for("Checkout turned off", |app| app.modules_on() == 6);
    let checkout = app
        .modules
        .iter()
        .find(|module| module.name == "Checkout")
        .unwrap();
    assert_eq!(checkout.state, ModuleState::Off);
}

#[test]
fn a_new_file_and_a_new_module_are_seen() {
    let opened = open(BTreeMap::new());
    opened.wait_for("the first read", |_| true);
    opened.start_watching();

    let before = opened.wait_for("the first read", |_| true).modules[0].clone();
    let model = opened.root.join("Modules/Billing/app/Models/Receipt.php");
    std::fs::write(model, "<?php\n\nclass Receipt {}\n").unwrap();
    let app = opened.wait_for("one more file in Billing", |app| {
        app.modules[0].files == before.files + 1
    });
    assert_eq!(app.modules[0].lines, before.lines + 3);

    let loyalty = opened.root.join("Modules/Loyalty");
    std::fs::create_dir(&loyalty).unwrap();
    std::fs::write(loyalty.join("module.json"), r#"{"name": "Loyalty"}"#).unwrap();
    let app = opened.wait_for("the module Loyalty", |app| app.modules.len() == 8);
    // A module that is new and has no entry in the statuses file is off.
    let loyalty = app
        .modules
        .iter()
        .find(|module| module.name == "Loyalty")
        .unwrap();
    assert_eq!(loyalty.state, ModuleState::Off);
}

#[test]
fn a_code_is_kept_when_modules_come_and_go() {
    let codes = BTreeMap::from([("Checkout".to_owned(), "COT".to_owned())]);
    let opened = open(codes);
    let app = opened.wait_for("the first read", |_| true);
    let code = |app: &AppSnapshot, name: &str| {
        app.modules
            .iter()
            .find(|module| module.name == name)
            .map(|module| module.code.clone())
    };
    assert_eq!(code(&app, "Checkout").as_deref(), Some("COT"));
    assert_eq!(code(&app, "Catalog").as_deref(), Some("CTL"));

    // A module whose proposal would be CTL arrives before Catalog in the order of names.
    let cattle = opened.root.join("Modules/Cattle");
    std::fs::create_dir(&cattle).unwrap();
    std::fs::write(cattle.join("module.json"), r#"{"name": "Cattle"}"#).unwrap();
    opened.live.refresh();
    let app = opened.wait_for("the module Cattle", |app| app.modules.len() == 8);
    assert_eq!(code(&app, "Catalog").as_deref(), Some("CTL"));
    assert_ne!(code(&app, "Cattle").as_deref(), Some("CTL"));
}

#[test]
fn a_new_coverage_report_is_seen() {
    let opened = open(BTreeMap::new());
    let app = opened.wait_for("the first read", |_| true);
    assert_eq!(app.coverage().unwrap().percent(), Some(68));
    opened.start_watching();

    // The tests ran again and cover every statement of Users.
    let report = opened.root.join("build/coverage/clover.xml");
    let xml = std::fs::read_to_string(&report).unwrap();
    let xml = xml.replace(
        r#"statements="25" coveredstatements="22""#,
        r#"statements="25" coveredstatements="25""#,
    );
    std::fs::write(&report, xml).unwrap();

    let app = opened.wait_for("Users fully covered", |app| {
        app.modules
            .iter()
            .any(|module| module.name == "Users" && module.coverage.unwrap().percent() == Some(100))
    });
    assert_eq!(app.coverage().unwrap().percent(), Some(70));
}

#[test]
fn other_options_are_read_with_at_once() {
    let opened = open(BTreeMap::new());
    let app = opened.wait_for("the first read", |_| true);
    assert_eq!(app.findings.warnings(), 1);

    // A lower limit: Notifications, at 39%, is no longer under it.
    opened.live.set_options(ReadOptions {
        coverage_limit: 30,
        coverage_report: None,
    });
    let app = opened.wait_for("the lower limit", |app| app.coverage_limit == 30);
    assert_eq!(app.findings.warnings(), 0);

    // A report the user names is read instead of the one Mortise finds.
    std::fs::write(
        opened.root.join("other.xml"),
        r#"<coverage><project><file name="/x/Modules/Users/app/A.php"><metrics statements="10" coveredstatements="1"/></file></project></coverage>"#,
    )
    .unwrap();
    opened.live.set_options(ReadOptions {
        coverage_limit: 30,
        coverage_report: Some("other.xml".into()),
    });
    let app = opened.wait_for("the named report", |app| {
        app.coverage_report
            .as_ref()
            .is_some_and(|report| report.path.ends_with("other.xml"))
    });
    assert_eq!(app.coverage().unwrap().percent(), Some(10));
    assert_eq!(app.findings.warnings(), 1);
}

#[test]
fn a_read_says_how_far_it_is() {
    let opened = open(BTreeMap::new());
    let app = opened.wait_for("the first read", |_| true);
    // When the read is over every PHP file of the modules is read.
    let (read, total) = opened.live.progress();
    assert_eq!(read, total);
    let php: u64 = app.modules.iter().map(|module| module.files).sum();
    assert!(total > 0 && total as u64 <= php, "{total} of {php}");

    // A second read counts again from the start, also for files it does not read again.
    opened.live.refresh();
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(opened.live.progress(), (total, total));
}
