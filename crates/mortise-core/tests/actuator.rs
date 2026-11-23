//! The actuator client, against a stub that answers as `nwidart/laravel-actuator` 1.0.0 does.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use mortise_core::testing::{Reply, StubActuator};
use mortise_core::{
    Answer, AppSnapshot, Credentials, Environment, LiveRuntime, Pace, Runtime, RuntimeState,
    Status, ask, ask_status, by_module, module_of, read_app, routes_by_module, routes_of, tasks_of,
};

fn local(stub: &StubActuator) -> Environment {
    Environment {
        name: "local".into(),
        base_url: stub.base_url.clone(),
        path: "actuator".into(),
        credentials: Credentials::None,
    }
}

fn answered(answer: Answer) -> Runtime {
    match answer {
        Answer::Answered(runtime) => *runtime,
        other => panic!("expected an answer, got {other:?}"),
    }
}

#[test]
fn everything_an_open_app_exposes_is_read() {
    let stub = StubActuator::healthy();
    let runtime = answered(ask(&local(&stub), &None));

    assert_eq!(runtime.status, Status::Up);
    assert_eq!(
        runtime.exposed,
        ["health", "info", "mappings", "scheduledtasks"]
    );
    assert!(runtime.not_exposed().is_empty());

    // The components, in the order the app gives them, with their details.
    let components = runtime.components.unwrap();
    let names: Vec<&str> = components
        .iter()
        .map(|component| component.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "ping",
            "readinessState",
            "diskSpace",
            "db",
            "cache",
            "module:billing:gateway",
            "modules"
        ]
    );
    assert!(
        components
            .iter()
            .all(|component| component.status == Status::Up)
    );
    let disk = &components[2];
    assert_eq!(
        disk.details[0],
        ("total".to_owned(), "494384795648".to_owned())
    );
    assert_eq!(
        disk.details[1],
        ("free".to_owned(), "213212008448".to_owned())
    );
    assert_eq!(
        components[3].details[0],
        ("connection".to_owned(), "mysql".to_owned())
    );

    let info = runtime.info.unwrap();
    assert_eq!(info.environment.as_deref(), Some("local"));
    assert_eq!(info.laravel.as_deref(), Some("13.2.0"));
    assert_eq!(info.git_branch.as_deref(), Some("feature/tracking-code"));
    assert_eq!(info.git_commit.as_deref(), Some("49689af"));
    assert_eq!(info.php_version.as_deref(), Some("8.4.27"));
    assert_eq!(info.php_sapi.as_deref(), Some("fpm-fcgi"));
    assert_eq!(info.memory_usage, Some(39_845_888));
    assert_eq!(info.memory_limit, Some(536_870_912));

    let routes = runtime.routes.unwrap();
    assert_eq!(routes.len(), 5);
    assert_eq!(routes[0].methods, ["GET", "HEAD"]);
    assert_eq!(routes[0].uri, "api/checkout/orders/{order}");
    assert_eq!(routes[0].name.as_deref(), Some("api.checkout.orders.show"));
    assert_eq!(
        routes[0].action,
        r"Modules\Checkout\Http\Controllers\OrderController@show"
    );
    assert_eq!(routes[0].middleware, ["api", "auth:sanctum"]);
    assert_eq!(routes[3].name, None);
    assert_eq!(routes[3].action, "Closure");

    let tasks = runtime.tasks.unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].target, "php artisan billing:send-invoices");
    assert_eq!(tasks[0].expression.as_deref(), Some("0 6 * * *"));
    assert_eq!(tasks[0].timezone.as_deref(), Some("UTC"));
    assert_eq!(tasks[0].next.as_deref(), Some("2026-11-23T06:00:00+00:00"));
    assert_eq!(tasks[2].target, "Poll shipment tracking");
    assert_eq!(tasks[2].expression, None);
    assert_eq!(tasks[2].interval_ms, Some(10_000));

    // The env endpoint is never asked for, and nothing else is.
    assert_eq!(
        stub.asked(),
        [
            "/actuator",
            "/actuator/health",
            "/actuator/info",
            "/actuator/mappings",
            "/actuator/scheduledtasks"
        ]
    );
}

#[test]
fn env_is_not_asked_for_when_the_app_exposes_it() {
    let stub = StubActuator::healthy();
    stub.expose(&["health", "info", "env", "mappings", "scheduledtasks"]);
    stub.set(
        "/actuator/env",
        Reply::ok(r#"{"activeProfiles":["local"],"propertySources":[]}"#),
    );
    answered(ask(&local(&stub), &None));
    assert!(!stub.asked().iter().any(|path| path.contains("env")));
}

#[test]
fn an_endpoint_that_is_not_exposed_is_not_asked_for() {
    // The default of the package: only health.
    let stub = StubActuator::healthy();
    stub.expose(&["health"]);
    let runtime = answered(ask(&local(&stub), &None));
    assert_eq!(runtime.status, Status::Up);
    assert_eq!(
        runtime.not_exposed(),
        ["info", "mappings", "scheduledtasks"]
    );
    assert_eq!(runtime.info, None);
    assert_eq!(runtime.routes, None);
    assert_eq!(runtime.tasks, None);
    assert_eq!(stub.asked(), ["/actuator", "/actuator/health"]);
}

#[test]
fn hidden_components_are_none() {
    // show_details is "never" by default: health has a status and nothing else.
    let stub = StubActuator::healthy();
    stub.set("/actuator/health", Reply::ok(r#"{"status":"UP"}"#));
    let runtime = answered(ask(&local(&stub), &None));
    assert_eq!(runtime.status, Status::Up);
    assert_eq!(runtime.components, None);
}

#[test]
fn a_503_from_health_is_an_answer() {
    let stub = StubActuator::healthy();
    stub.set(
        "/actuator/health",
        Reply::status(
            503,
            r#"{"status":"DOWN","components":{
            "ping":{"status":"UP"},
            "db":{"status":"DOWN","details":{"error":"Illuminate\\Database\\QueryException: SQLSTATE[HY000] [2002] Connection refused"}}}}"#,
        ),
    );
    let runtime = answered(ask(&local(&stub), &None));
    assert_eq!(runtime.status, Status::Down);
    let components = runtime.components.unwrap();
    assert_eq!(components[1].status, Status::Down);
    // The detail is the app's own text.
    assert_eq!(
        components[1].details[0].1,
        r"Illuminate\Database\QueryException: SQLSTATE[HY000] [2002] Connection refused"
    );

    stub.set(
        "/actuator/health",
        Reply::status(
            503,
            r#"{"status":"OUT_OF_SERVICE","components":{"readinessState":{"status":"OUT_OF_SERVICE"}}}"#,
        ),
    );
    let runtime = answered(ask(&local(&stub), &None));
    assert_eq!(runtime.status, Status::OutOfService);
    assert_eq!(runtime.components.unwrap()[0].status, Status::OutOfService);

    stub.set(
        "/actuator/health",
        Reply::ok(r#"{"status":"SOMETHING_NEW"}"#),
    );
    assert_eq!(answered(ask(&local(&stub), &None)).status, Status::Unknown);
}

#[test]
fn no_answer_names_the_address_and_the_reason() {
    // A port nothing listens on.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let environment = Environment {
        name: "local".into(),
        base_url: base_url.clone(),
        path: "actuator".into(),
        credentials: Credentials::None,
    };
    assert_eq!(
        ask(&environment, &None),
        Answer::NoAnswer {
            url: format!("{base_url}/actuator"),
            message: "connection refused".into()
        }
    );
}

#[test]
fn a_host_that_does_not_exist_is_said_in_plain_words() {
    // `.invalid` never resolves.
    let environment = Environment {
        name: "local".into(),
        base_url: "http://shop-api.invalid".into(),
        path: "actuator".into(),
        credentials: Credentials::None,
    };
    assert_eq!(
        ask(&environment, &None),
        Answer::NoAnswer {
            url: "http://shop-api.invalid/actuator".into(),
            message: "the host was not found".into()
        }
    );
}

#[test]
fn a_500_and_a_404_are_not_statuses_of_the_app() {
    let stub = StubActuator::healthy();
    stub.set(
        "/actuator/health",
        Reply::status(500, r#"{"message":"Server Error"}"#),
    );
    assert_eq!(
        ask(&local(&stub), &None),
        Answer::Unexpected {
            url: format!("{}/actuator/health", stub.base_url),
            status: 500,
            location: None
        }
    );
    // A redirect names where it sends to, read against the address that was asked for.
    stub.set("/actuator/health", Reply::redirect("/actuator/health/"));
    assert_eq!(
        ask(&local(&stub), &None),
        Answer::Unexpected {
            url: format!("{}/actuator/health", stub.base_url),
            status: 302,
            location: Some(format!("{}/actuator/health/", stub.base_url))
        }
    );

    // No actuator at this path.
    let empty = StubActuator::start();
    assert_eq!(
        ask(&local(&empty), &None),
        Answer::Unexpected {
            url: format!("{}/actuator", empty.base_url),
            status: 404,
            location: None
        }
    );
}

#[test]
fn each_kind_of_credentials_is_sent_as_its_header() {
    let cases = [
        (Credentials::Bearer, "authorization", "Bearer s3cret"),
        // base64 of "mortise:s3cret"
        (
            Credentials::Basic {
                user: "mortise".into(),
            },
            "authorization",
            "Basic bW9ydGlzZTpzM2NyZXQ=",
        ),
        (
            Credentials::Header {
                name: "X-Actuator-Key".into(),
            },
            "x-actuator-key",
            "s3cret",
        ),
    ];
    for (credentials, header, value) in cases {
        let stub = StubActuator::healthy();
        stub.require_header(header, value);
        let environment = Environment {
            credentials,
            ..local(&stub)
        };

        // Without the secret, and with a wrong one, the app refuses.
        assert_eq!(
            ask(&local(&stub), &None),
            Answer::NotAuthorized {
                url: format!("{}/actuator", stub.base_url),
                status: 401
            }
        );
        assert!(matches!(
            ask(&environment, &Some("wrong".into())),
            Answer::NotAuthorized { status: 401, .. }
        ));
        // With it, the app answers.
        assert_eq!(
            answered(ask(&environment, &Some("s3cret".into()))).status,
            Status::Up
        );

        // A redirect is not followed, so the secret does not go to another host.
        let other = StubActuator::healthy();
        stub.set(
            "/actuator",
            Reply::redirect(&format!("{}/actuator/env", other.base_url)),
        );
        assert_eq!(
            ask(&environment, &Some("s3cret".into())),
            Answer::Unexpected {
                url: format!("{}/actuator", stub.base_url),
                status: 302,
                location: Some(format!("{}/actuator/env", other.base_url))
            }
        );
        assert!(other.asked().is_empty());
    }
}

#[test]
fn a_403_on_one_endpoint_is_not_authorized_with_its_address() {
    let stub = StubActuator::healthy();
    stub.set(
        "/actuator/info",
        Reply::status(403, r#"{"message":"This action is unauthorized."}"#),
    );
    assert_eq!(
        ask(&local(&stub), &None),
        Answer::NotAuthorized {
            url: format!("{}/actuator/info", stub.base_url),
            status: 403
        }
    );
}

#[test]
fn the_regular_check_asks_for_health_only() {
    let stub = StubActuator::healthy();
    let earlier = answered(ask(&local(&stub), &None));
    let before = stub.asked().len();

    stub.set(
        "/actuator/health",
        Reply::status(
            503,
            r#"{"status":"DOWN","components":{"db":{"status":"DOWN"}}}"#,
        ),
    );
    let now = answered(ask_status(&local(&stub), &None, &earlier));
    assert_eq!(stub.asked()[before..], ["/actuator/health"]);
    assert_eq!(now.status, Status::Down);
    assert_eq!(now.components.as_ref().unwrap().len(), 1);
    // The rest is what the last full answer had.
    assert_eq!(now.routes, earlier.routes);
    assert_eq!(now.info, earlier.info);
    assert_eq!(now.exposed, earlier.exposed);
}

#[test]
fn routes_and_tasks_are_given_to_the_module_of_their_class() {
    let app =
        read_app(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shop-api"))
            .unwrap();
    let stub = StubActuator::healthy();
    let runtime = answered(ask(&local(&stub), &None));
    let routes = runtime.routes.unwrap();
    let tasks = runtime.tasks.unwrap();

    assert_eq!(
        module_of(
            &app,
            r"Modules\Checkout\Http\Controllers\OrderController@show"
        )
        .map(|module| module.name.as_str()),
        Some("Checkout")
    );
    // A namespace that only starts the same is another thing.
    assert!(module_of(&app, r"Modules\CheckoutExtras\Thing").is_none());
    assert!(module_of(&app, "Closure").is_none());

    // Two routes in Checkout, one in Billing; the closure and the actuator's own
    // route are in no module.
    let (by_module, outside) = routes_by_module(&app, &routes);
    assert_eq!(
        by_module,
        [("Checkout".to_owned(), 2), ("Billing".to_owned(), 1)]
    );
    assert_eq!(outside, 2);
    assert_eq!(routes_of(&app, "Checkout", &routes).len(), 2);
    assert!(routes_of(&app, "Users", &routes).is_empty());

    // A task belongs to a module only when its target is a class of the module.
    let inventory = tasks_of(&app, "Inventory", &tasks);
    assert_eq!(inventory.len(), 1);
    assert_eq!(inventory[0].target, r"Modules\Inventory\Jobs\SyncStock");
    assert!(tasks_of(&app, "Billing", &tasks).is_empty());
}

/// The modules of the recorded app: Blog, Billing and Legacy, made from shop-api's Billing.
fn recorded_app() -> AppSnapshot {
    let mut app =
        read_app(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shop-api"))
            .unwrap();
    let billing = app
        .modules
        .iter()
        .find(|module| module.name == "Billing")
        .unwrap()
        .clone();
    let named = |name: &str| {
        let mut module = billing.clone();
        module.name = name.to_owned();
        module
    };
    app.modules = vec![named("Billing"), named("Blog"), named("Legacy")];
    app
}

/// The components of a health answer, read by the client.
fn components_of(health: &str) -> Vec<mortise_core::Component> {
    let stub = StubActuator::healthy();
    stub.set("/actuator/health", Reply::ok(health));
    answered(ask(&local(&stub), &None)).components.unwrap()
}

#[test]
fn the_recorded_health_gives_each_module_its_indicators() {
    let recorded = include_str!("../../../fixtures/actuator/health.json");
    let split = by_module(&recorded_app(), &components_of(recorded));

    // module:blog:storage belongs to Blog by its lower case name, under its key.
    let blog = split.of("Blog").unwrap();
    assert_eq!(blog.status, Status::Up);
    assert_eq!(blog.error, None);
    assert_eq!(blog.indicators.len(), 1);
    assert_eq!(blog.indicators[0].name, "storage");
    assert_eq!(
        blog.indicators[0].details,
        [("drafts".to_owned(), "3".to_owned())]
    );
    assert_eq!(split.of("Billing").unwrap().indicators[0].name, "gateway");
    // Legacy is turned off and registers no indicator: it has no status.
    assert!(split.of("Legacy").is_none());
    let order: Vec<&str> = split.modules.iter().map(|m| m.module.as_str()).collect();
    assert_eq!(order, ["Billing", "Blog"]);

    // The module components leave the app's own list.
    let own: Vec<&str> = split.app.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        own,
        [
            "ping",
            "readinessState",
            "diskSpace",
            "db",
            "migrations",
            "cache"
        ]
    );
}

#[test]
fn a_module_takes_its_worst_indicator_and_the_error_of_the_modules_component() {
    let health = r#"{"status":"DOWN","components":{
        "db":{"status":"UP"},
        "module:billing:gateway":{"status":"UNKNOWN"},
        "module:billing:queue":{"status":"UP"},
        "module:blog:storage":{"status":"UP"},
        "module:blog:search":{"status":"OUT_OF_SERVICE"},
        "module:legacy:x":{"status":"UP"},
        "module:ghost:x":{"status":"DOWN"},
        "modules":{"status":"DOWN","details":{"modules":{
            "Billing":{"enabled":true},
            "Legacy":{"enabled":true,"error":"Enabled in the status store, not found on disk."},
            "Ghost":{"enabled":true,"error":"Enabled in the status store, not found on disk."}}}}}}"#;
    let split = by_module(&recorded_app(), &components_of(health));

    // Down, Out of service, Up, Unknown: Unknown is the mildest.
    assert_eq!(split.of("Billing").unwrap().status, Status::Up);
    assert_eq!(split.of("Blog").unwrap().status, Status::OutOfService);
    // An error of the modules component makes the module Down, whatever its
    // indicators say, with the app's sentence.
    let legacy = split.of("Legacy").unwrap();
    assert_eq!(legacy.status, Status::Down);
    assert_eq!(
        legacy.error.as_deref(),
        Some("Enabled in the status store, not found on disk.")
    );

    // An indicator of no module stays the app's, under its full name. The modules
    // component stays with the error of the name that is no module only.
    let own: Vec<&str> = split.app.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(own, ["db", "module:ghost:x", "modules"]);
    assert_eq!(
        split.app[2].details,
        [(
            "Ghost".to_owned(),
            "Enabled in the status store, not found on disk.".to_owned()
        )]
    );
}

fn wait_for(
    runtime: &LiveRuntime,
    what: &str,
    done: impl Fn(&RuntimeState) -> bool,
) -> RuntimeState {
    let start = Instant::now();
    loop {
        let state = runtime.state();
        if done(&state) {
            return state;
        }
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "timed out waiting for {what}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn count(stub: &StubActuator, path: &str) -> usize {
    stub.asked().iter().filter(|asked| *asked == path).count()
}

#[test]
fn a_live_environment_asks_health_often_and_the_rest_seldom() {
    let stub = StubActuator::healthy();
    let changes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&changes);
    let pace = Pace {
        status: Duration::from_millis(60),
        everything: Duration::from_secs(3600),
        seen: Duration::from_secs(3600),
    };
    let runtime = LiveRuntime::open_with(local(&stub), None, pace, move || {
        counter.fetch_add(1, Ordering::Relaxed);
    });
    assert!(runtime.state().asking);

    let state = wait_for(&runtime, "the first answer", |state| state.answer.is_some());
    assert!(matches!(state.answer.as_deref(), Some(Answer::Answered(_))));
    assert!(state.checked.is_some());
    assert!(changes.load(Ordering::Relaxed) >= 2);

    // health is asked again and again; the rest only once.
    wait_for(&runtime, "three checks", |_| {
        count(&stub, "/actuator/health") >= 4
    });
    assert_eq!(count(&stub, "/actuator/mappings"), 1);
    assert_eq!(count(&stub, "/actuator"), 1);

    // The app goes down: the next check shows it, and the routes are still there.
    stub.set(
        "/actuator/health",
        Reply::status(503, r#"{"status":"DOWN"}"#),
    );
    let state = wait_for(
        &runtime,
        "Down",
        |state| matches!(state.answer.as_deref(), Some(Answer::Answered(runtime)) if runtime.status == Status::Down),
    );
    let Some(Answer::Answered(down)) = state.answer.as_deref() else {
        unreachable!()
    };
    assert_eq!(down.routes.as_ref().map(Vec::len), Some(5));

    // A refresh asks for everything.
    runtime.refresh();
    wait_for(&runtime, "everything again", |_| {
        count(&stub, "/actuator/mappings") == 2
    });
}

#[test]
fn a_live_environment_stops_asking_when_the_window_is_not_seen() {
    let stub = StubActuator::healthy();
    let pace = Pace {
        status: Duration::from_millis(40),
        everything: Duration::from_secs(3600),
        seen: Duration::from_millis(200),
    };
    let runtime = LiveRuntime::open_with(local(&stub), None, pace, || {});
    wait_for(&runtime, "the first answer", |state| state.answer.is_some());

    // Nobody draws: after a moment no request is made any more.
    std::thread::sleep(Duration::from_millis(500));
    let quiet = stub.asked().len();
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(stub.asked().len(), quiet);

    // The window is seen again: asking starts at once.
    runtime.seen();
    wait_for(&runtime, "a check after the window is seen", |_| {
        stub.asked().len() > quiet
    });
}

#[test]
fn a_live_environment_that_gave_no_answer_asks_for_everything_when_it_can() {
    let stub = StubActuator::start();
    let pace = Pace {
        status: Duration::from_millis(40),
        everything: Duration::from_secs(3600),
        seen: Duration::from_secs(3600),
    };
    let runtime = LiveRuntime::open_with(local(&stub), None, pace, || {});
    wait_for(&runtime, "404", |state| {
        matches!(
            state.answer.as_deref(),
            Some(Answer::Unexpected { status: 404, .. })
        )
    });

    // The actuator appears: the next check reads all of it.
    let healthy = StubActuator::healthy();
    for path in [
        "/actuator",
        "/actuator/health",
        "/actuator/info",
        "/actuator/mappings",
        "/actuator/scheduledtasks",
    ] {
        stub.set(path, healthy.reply(path).unwrap());
    }
    let state = wait_for(&runtime, "an answer", |state| {
        matches!(state.answer.as_deref(), Some(Answer::Answered(_)))
    });
    let Some(Answer::Answered(up)) = state.answer.as_deref() else {
        unreachable!()
    };
    assert_eq!(up.routes.as_ref().map(Vec::len), Some(5));
}

#[test]
#[ignore = "asks a site over HTTPS; run by hand, with MORTISE_HTTPS_URL for a site of your own"]
fn https_works_with_the_certificates_this_computer_trusts() {
    let environment = Environment {
        name: "public".into(),
        base_url: std::env::var("MORTISE_HTTPS_URL").unwrap_or("https://example.com".into()),
        path: "actuator".into(),
        credentials: Credentials::None,
    };
    // That the site answers at all, with or without an actuator, shows that TLS works.
    let answer = ask(&environment, &None);
    assert!(
        matches!(
            answer,
            Answer::Unexpected { .. } | Answer::Answered(_) | Answer::NotAuthorized { .. }
        ),
        "expected an HTTP status, got {answer:?}"
    );
}
