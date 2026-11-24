//! The actuator client against a real app with laravel-modules, laravel-actuator and
//! laravel-metrics, through the calls the worker of the app makes.
//!
//! The tests are ignored: they need the app that `tools/real-actuator-app.sh` builds, served three
//! times as the job "Real actuator" in `.github/workflows/ci.yml` does:
//!
//! - `MORTISE_REAL_ACTUATOR`: the actuator of the app as built, `http://127.0.0.1:8931/actuator`.
//! - `MORTISE_REAL_ACTUATOR_EMPTY`: the same app served with `METRICS_ENABLED=false`.
//! - `MORTISE_REAL_ACTUATOR_503`: the same app served with `METRICS_CACHE_STORE=metrics_broken`.
//! - `MORTISE_REAL_APP`: the folder of the app.
//!
//! A test whose variable is not set says so on stderr and passes. Run them with
//! `cargo test -p mortise-core --test real_actuator -- --ignored --test-threads=1`.
//! The test of the figures makes requests and runs the queue of the app between two reads.
#![cfg(feature = "actuator")]

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use mortise_core::{
    Answer, AppSnapshot, Credentials, Environment, LiveMetrics, NoMetrics, Range, RowOf, Runtime,
    Scope, Status, ask, ask_metrics, by_module, read_app,
};

/// The value of `name`, or `None` with a line on stderr when it is not set.
fn var(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => {
            eprintln!(
                "{name} is not set, so the test does not run. \
                 The job \"Real actuator\" in ci.yml shows how to serve the app."
            );
            None
        }
    }
}

/// The environment whose actuator is at `url`, as a person sets it in the app.
fn environment(url: &str) -> Environment {
    Environment {
        name: "local".into(),
        base_url: url.into(),
        path: String::new(),
        credentials: Credentials::None,
    }
}

/// The app read from its files, and what its actuator answers.
fn app_and_runtime(url: &str, folder: &str) -> (AppSnapshot, Runtime) {
    let app = read_app(&PathBuf::from(folder)).expect("the app can be read");
    match ask(&environment(url), &None) {
        Answer::Answered(runtime) => (app, *runtime),
        other => panic!("{url} did not answer: {other:?}"),
    }
}

/// Asks for `url` with a plain HTTP/1.0 `GET`, so the body is not chunked. Gives the status and
/// the body. This makes traffic and checks what the client read; the reads go through the client.
fn get(url: &str) -> (u16, String) {
    let rest = url.strip_prefix("http://").expect("an http:// address");
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    let mut stream = TcpStream::connect(host).expect("the server takes a connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    // One write: the PHP server refuses a request line that comes in pieces.
    let request =
        format!("GET /{path} HTTP/1.0\r\nHost: {host}\r\nAccept: application/json\r\n\r\n");
    stream.write_all(request.as_bytes()).unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).unwrap();
    let (head, body) = reply.split_once("\r\n\r\n").unwrap_or((&reply, ""));
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|status| status.parse().ok())
        .unwrap_or_else(|| panic!("no status in {head:?}"));
    (status, body.to_owned())
}

/// `http://127.0.0.1:8931` of `http://127.0.0.1:8931/actuator`.
fn origin(url: &str) -> &str {
    let after = url.find("://").map_or(0, |at| at + 3);
    url[after..].find('/').map_or(url, |at| &url[..after + at])
}

#[test]
#[ignore = "needs the served app of tools/real-actuator-app.sh"]
fn health_gives_each_module_its_indicator() {
    let (Some(url), Some(folder)) = (var("MORTISE_REAL_ACTUATOR"), var("MORTISE_REAL_APP")) else {
        return;
    };
    let (app, runtime) = app_and_runtime(&url, &folder);
    assert!(runtime.exposes("prometheus"), "{:?}", runtime.exposed);
    assert_eq!(runtime.not_exposed(), Vec::<&str>::new());
    assert_eq!(runtime.status, Status::Up);

    let components = runtime.components.expect("health shows its components");
    let split = by_module(&app, &components);
    for (module, key) in [("Blog", "storage"), ("Billing", "gateway")] {
        let found = split
            .of(module)
            .unwrap_or_else(|| panic!("{module} has no status: {split:?}"));
        assert_eq!(found.status, Status::Up, "{found:?}");
        assert_eq!(found.error, None);
        let names: Vec<&str> = found.indicators.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, [key]);
    }
    // The indicators of the modules leave the components of the app.
    let own: Vec<&str> = split.app.iter().map(|c| c.name.as_str()).collect();
    assert!(
        own.iter().all(|name| !name.starts_with("module:")),
        "{own:?}"
    );
    assert!(!own.is_empty());
    assert_eq!(split.elsewhere, []);
}

#[test]
#[ignore = "needs the served app of tools/real-actuator-app.sh"]
fn figures_place_requests_and_jobs_in_their_modules() {
    let (Some(url), Some(folder)) = (var("MORTISE_REAL_ACTUATOR"), var("MORTISE_REAL_APP")) else {
        return;
    };
    let (app, runtime) = app_and_runtime(&url, &folder);
    let environment = environment(&url);
    let now = Arc::new(AtomicU64::new(0));
    let clock = Arc::clone(&now);
    let metrics = LiveMetrics::with_clock(Arc::new(move || {
        Duration::from_secs(clock.load(Ordering::Relaxed))
    }));
    let mut sink = metrics.sink();

    // A figure is what the counters grew by between two reads.
    sink(ask_metrics(&environment, &None));
    assert_eq!(metrics.state().last, Some(Ok(())));
    for (path, status) in [
        ("/blog/posts/7", 200),
        ("/blog/boom", 500),
        ("/billing/invoices/7/charge", 200),
        ("/hello/mortise", 200),
    ] {
        assert_eq!(get(&format!("{}{path}", origin(&url))).0, status, "{path}");
    }
    let worked = Command::new("php")
        .args(["artisan", "queue:work", "--stop-when-empty"])
        .current_dir(&folder)
        .output()
        .expect("php runs");
    assert!(worked.status.success(), "{worked:?}");
    now.store(15, Ordering::Relaxed);
    sink(ask_metrics(&environment, &None));
    assert_eq!(metrics.state().last, Some(Ok(())));

    let figures = metrics.figures(
        &app,
        runtime.routes.as_deref(),
        Scope::All,
        Range::Hour,
        None,
    );
    assert_eq!(figures.covered, Some(Duration::from_secs(15)));
    let rates = |module: &str| {
        let of = RowOf::Module(module.to_owned());
        let row = figures.rows.iter().find(|row| row.of == of);
        row.unwrap_or_else(|| panic!("no row of {module}: {figures:?}"))
            .rates
            .clone()
    };
    // Per minute, over 15 s: one request is 4.
    let blog = rates("Blog");
    assert_eq!(blog.requests_per_minute, 8.0, "{blog:?}");
    assert_eq!(blog.errors, Some(0.5), "{blog:?}");
    assert_eq!(blog.jobs_per_minute, 0.0, "{blog:?}");
    let billing = rates("Billing");
    assert_eq!(billing.requests_per_minute, 4.0, "{billing:?}");
    assert_eq!(billing.jobs_per_minute, 4.0, "{billing:?}");
    let outside = figures.outside.expect("All has \"Not in a module\"").rates;
    assert_eq!(outside.requests_per_minute, 4.0, "{outside:?}");
}

#[test]
#[ignore = "needs the served app of tools/real-actuator-app.sh"]
fn an_empty_answer_reads_as_metrics_off() {
    let Some(url) = var("MORTISE_REAL_ACTUATOR_EMPTY") else {
        return;
    };
    assert_eq!(ask_metrics(&environment(&url), &None), Err(NoMetrics::Off));
}

#[test]
#[ignore = "needs the served app of tools/real-actuator-app.sh"]
fn a_store_that_cannot_be_read_reads_as_503_with_the_line_of_the_app() {
    let Some(url) = var("MORTISE_REAL_ACTUATOR_503") else {
        return;
    };
    let (status, body) = get(&format!("{url}/prometheus"));
    assert_eq!(status, 503);
    // The client keeps at most 200 characters of the line.
    let line: String = body
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(200)
        .collect();
    assert!(!line.is_empty());
    assert_eq!(
        ask_metrics(&environment(&url), &None),
        Err(NoMetrics::Unavailable {
            url: format!("{url}/prometheus"),
            line,
        })
    );
}
