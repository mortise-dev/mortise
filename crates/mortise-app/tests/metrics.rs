//! The Metrics tab: what the environment answered, and the table of requests and jobs per
//! minute for each module. The figures themselves are the work of `mortise-core`.

mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use common::{Desk, connected, environment, fixture, shop_api, wait_for_answer};
use eframe::egui::{Key, Modifiers, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use mortise_app::app::MortiseApp;
use mortise_core::Pace;
use mortise_core::testing::{Reply, StubActuator};

const ONE_READ: &str = "A rate needs two reads. The second read comes 15 s after the first.";

fn open_metrics(harness: &mut Harness<'static, MortiseApp>) {
    harness.key_press(Key::Num5);
    harness.run_steps(3);
}

/// Waits until a label is on screen, for at most 10 s. A read of metrics happens on the
/// worker thread, so real time has to pass between frames.
fn wait_for(harness: &mut Harness<'static, MortiseApp>, label: &str) {
    wait_while(harness, label, |harness| {
        harness.query_by_label(label).is_none()
    });
}

fn wait_while(
    harness: &mut Harness<'static, MortiseApp>,
    what: &str,
    waiting: impl Fn(&Harness<'static, MortiseApp>) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        harness.step();
        if !waiting(harness) {
            break;
        }
        assert!(Instant::now() < deadline, "waited 10 s for: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
    harness.run_steps(2);
}

#[test]
fn without_an_environment_the_tab_offers_to_connect_one() {
    let (_desk, mut harness) = shop_api();
    open_metrics(&mut harness);

    harness.get_by_role_and_label(Role::Heading, "Metrics are not connected");
    harness.get_by_label(
        "Metrics come from the actuator of the running app. Connect an environment for shop-api \
         to read requests, response time and errors per module.",
    );
    harness
        .get_by_role_and_label(Role::Button, "Connect an environment")
        .click();
    harness.run_steps(4);
    harness.get_by_role_and_label(Role::Heading, "Environments of shop-api");
}

#[test]
fn without_prometheus_in_the_links_the_tab_says_how_to_expose_it() {
    let stub = StubActuator::recorded();
    let links = stub.reply("/actuator").unwrap();
    stub.expose(&["health", "info", "mappings", "scheduledtasks"]);
    let mut desk = Desk::new();
    // Metrics are read every 100 ms, so a read after the tab closed would show at once.
    desk.pace = Pace {
        metrics: Duration::from_millis(100),
        ..Pace::default()
    };
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(&[environment("local", &stub)]);
    let mut harness = desk.start();
    wait_for_answer(&mut harness);
    open_metrics(&mut harness);

    harness.get_by_role_and_label(Role::Heading, "Metrics are not exposed on local");
    harness
        .get_by_label("Add prometheus to exposure.include in config/actuator.php, next to health.");
    harness
        .get_by_role_and_label(Role::Button, "Open settings")
        .click();
    harness.run_steps(4);
    harness.get_by_role_and_label(Role::Heading, "Environments of shop-api");
    harness.key_press(Key::Escape);
    harness.run_steps(4);

    // The app's team adds the endpoint; "Try again" asks for the links again.
    stub.set("/actuator", links);
    harness
        .get_by_role_and_label(Role::Button, "Try again")
        .click();
    wait_for(&mut harness, "Requests/min");
    harness.get_by_role_and_label(Role::ColumnHeader, "Jobs/min");

    // Settings take the place of the tab: no read of `prometheus` starts after that.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    let prometheus = || {
        stub.asked()
            .iter()
            .filter(|path| *path == "/actuator/prometheus")
            .count()
    };
    let draw_for = |harness: &mut Harness<'static, MortiseApp>, time: Duration| {
        let until = Instant::now() + time;
        while Instant::now() < until {
            harness.step();
            std::thread::sleep(Duration::from_millis(20));
        }
    };
    // A read on its way ends.
    draw_for(&mut harness, Duration::from_millis(300));
    let asked = prometheus();
    draw_for(&mut harness, Duration::from_millis(600));
    assert_eq!(prometheus(), asked);
}

#[test]
fn a_503_quotes_the_app_and_an_empty_answer_says_metrics_are_off() {
    let stub = StubActuator::recorded();
    let line = std::fs::read_to_string(fixture("actuator/prometheus-503.txt")).unwrap();
    stub.set("/actuator/prometheus", Reply::status(503, &line));
    let (_desk, mut harness) = connected(&stub);
    open_metrics(&mut harness);

    wait_for(&mut harness, "Metrics cannot be read on local");
    harness.get_by_label(
        "local answered 503 for /actuator/prometheus: \"The metrics store cannot be read.\" \
         Check the metrics store of the app, then try again.",
    );

    stub.set("/actuator/prometheus", Reply::ok(""));
    harness
        .get_by_role_and_label(Role::Button, "Try again")
        .click();
    wait_for(&mut harness, "Metrics are off on local");
    harness.get_by_label(
        "The app answers with no metrics. Set METRICS_ENABLED=true in its .env to turn them on.",
    );

    // A sign-in page served with 200 is not metrics.
    stub.set(
        "/actuator/prometheus",
        Reply::ok("<!doctype html>\n<title>Sign in</title>\n"),
    );
    harness
        .get_by_role_and_label(Role::Button, "Try again")
        .click();
    wait_for(&mut harness, "The answer of local is not metrics");
    harness.get_by_label(
        "local answered prometheus with text that holds no metrics. Check that the address \
         points to the actuator of the app.",
    );
    harness.get_by_role_and_label(Role::Button, "Open settings");

    // 11,000 jobs of 3 numbers each are over the 32,768 a read keeps.
    let mut jobs = String::from("# TYPE messaging_process_duration_seconds histogram\n");
    for job in 0..11_000 {
        jobs.push_str(&format!(
            "messaging_process_duration_seconds_count{{laravel_job_name=\"J{job}\"}} 1\n"
        ));
    }
    stub.set("/actuator/prometheus", Reply::ok(&jobs));
    harness
        .get_by_role_and_label(Role::Button, "Try again")
        .click();
    wait_for(&mut harness, "The metrics of local have too many series");
    harness.get_by_label(
        "Mortise keeps at most 32,768 figures per read. Export fewer series from the app.",
    );
}

/// Every file under `dir` with its content.
fn files(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.insert(path.clone(), std::fs::read(&path).unwrap());
        }
    }
    found
}

#[test]
fn two_reads_give_a_figure_for_every_module_and_write_nothing() {
    let stub = StubActuator::recorded();
    let mut desk = Desk::new();
    // The clock of the reads moves when the test moves it.
    let now = Arc::new(AtomicU64::new(0));
    let clock = Arc::clone(&now);
    desk.metrics_clock = Some(Arc::new(move || {
        Duration::from_secs(clock.load(Ordering::SeqCst))
    }));
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(&[environment("local", &stub), environment("staging", &stub)]);
    let mut harness = desk.start();
    wait_for_answer(&mut harness);
    let config = desk.settings_path().parent().unwrap().to_path_buf();
    let before = (files(&config), files(&desk.data_dir()));
    open_metrics(&mut harness);

    // One read: every module is listed, with no figure yet.
    wait_for(&mut harness, ONE_READ);
    let modules = harness.state().app().unwrap().modules.len();
    let rows = modules + 1;
    for module in ["Billing", "Catalog", "Notifications", "Not in a module"] {
        harness.get_by_role_and_label(Role::Cell, module);
    }
    for header in [
        "Module",
        "Requests/min",
        "95% under",
        "Errors",
        "Jobs/min",
        "Database time",
        "Last hour",
    ] {
        harness.get_by_role_and_label(Role::ColumnHeader, header);
    }
    let count = |harness: &Harness<'static, MortiseApp>, text: &str| {
        harness.get_all_by_role_and_label(Role::Cell, text).count()
    };
    assert_eq!(count(&harness, "\u{2013}"), 5 * rows);

    // The second read, 15 s later, asked for at once.
    now.store(15, Ordering::SeqCst);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::R);
    wait_while(&mut harness, "the second read", |harness| {
        harness.query_by_label(ONE_READ).is_some()
    });
    // Nothing was asked for: requests and jobs are 0, the 95% figure and errors have no request.
    assert_eq!(count(&harness, "0"), 2 * rows);
    assert_eq!(count(&harness, "\u{2013}"), 2 * rows);
    // No series has the module label.
    assert_eq!(count(&harness, "unknown"), rows);
    harness
        .get_all_by_role_and_label(Role::Cell, "unknown")
        .next()
        .unwrap()
        .hover();
    harness.run_steps(12);
    harness.get_by_label("Needs a newer nwidart/laravel-modules");

    // "last read N s ago" counts up by the clock of the reads, with no read.
    harness.get_by_label("read every 15 s, last read 0 s ago");
    now.store(20, Ordering::SeqCst);
    harness.run_steps(2);
    harness.get_by_label("read every 15 s, last read 5 s ago");

    // Another range keeps the reads.
    for range in ["15 min", "1 hour"] {
        harness
            .get_by_role_and_label(Role::RadioButton, range)
            .click();
        harness.run_steps(3);
        assert_eq!(count(&harness, "0"), 2 * rows);
    }
    harness.get_by_role_and_label(Role::Image, "Requests per minute, all modules, last hour");

    // A row clicked shows in the chart, and J moves to the next row.
    harness.get_by_role_and_label(Role::Row, "Catalog").click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Requests per minute, Catalog");
    harness.key_press(Key::J);
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Requests per minute, Checkout");

    // A module selected in the index lists its routes.
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Billing")
        .click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Requests per minute, Billing");
    harness.get_by_role_and_label(Role::ColumnHeader, "Route");
    harness.get_by_role_and_label(Role::Cell, "GET /billing/invoices/{invoice}");

    // Reading metrics writes nothing: not the settings, not the history.
    assert_eq!((files(&config), files(&desk.data_dir())), before);

    // Another environment starts with no read.
    harness
        .get_by_role_and_label(Role::RadioButton, "staging")
        .click();
    wait_for(&mut harness, ONE_READ);
}
