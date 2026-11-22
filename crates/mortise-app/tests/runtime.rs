//! Runtime: what the running app reports through the actuator, and how Mortise connects to it.

mod common;

use common::{Desk, connected, environment, shop_api_actuator, wait_for_answer, wait_until};
use eframe::egui::{Key, Modifiers, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use mortise_app::app::MortiseApp;
use mortise_core::testing::{Reply, StubActuator};
use mortise_core::{Answer, Credentials, Environment, Status};

/// A desk with shop-api known and these environments, started. It does not wait
/// for an answer.
fn with_environments(environments: &[Environment]) -> (Desk, Harness<'static, MortiseApp>) {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    desk.knows(&[(&root, "SA")]);
    desk.connect(environments);
    let harness = desk.start();
    (desk, harness)
}

fn status(harness: &Harness<'static, MortiseApp>) -> Option<Status> {
    match harness.state().runtime().answer.as_deref() {
        Some(Answer::Answered(runtime)) => Some(runtime.status),
        _ => None,
    }
}

#[test]
fn the_summary_shows_what_the_running_app_reports() {
    let stub = shop_api_actuator();
    let (_desk, harness) = connected(&stub);

    // The index header and the stat cell: a word, and where it was asked.
    assert_eq!(harness.get_all_by_label("Up on local").count(), 2);

    // The runtime box: the status, each component with what it says, then info.
    harness.get_by_role_and_label(Role::Heading, "Runtime on local");
    harness.get_by_label("Status");
    for component in ["ping", "readinessState", "diskSpace", "db", "cache"] {
        harness.get_by_label(component);
    }
    harness.get_by_label("198.6 GB free of 460.4 GB");
    harness.get_by_label("mysql");
    harness.get_by_label("redis");
    harness.get_by_label("local, Laravel 13.2.0");
    harness.get_by_label("feature/tracking-code");
    harness.get_by_label("49689af");
    harness.get_by_label("8.4.27, fpm-fcgi");
    harness.get_by_label("38 MB now, 41 MB peak, limit 512 MB");
    // One "Up" for the status and one for each component.
    assert_eq!(harness.get_all_by_label("Up").count(), 6);

    // Routes by module, counted by the namespace of the action.
    harness.get_by_role_and_label(Role::Heading, "Routes by module");
    harness.get_by_label("106 routes, read from local");
    harness.get_by_label("Not in a module");
    harness.get_by_label("22");
    harness.get_by_label("12");

    // Scheduled tasks. Only a task whose target is a class has a module.
    harness.get_by_role_and_label(Role::Heading, "Scheduled tasks");
    harness.get_by_label("5 tasks, read from local");
    harness.get_by_label("php artisan checkout:expire-carts");
    harness.get_by_label(r"Modules\Inventory\Jobs\SyncStock");
    harness.get_by_label("every 10 s");
    harness.get_by_label("no fixed time");
    assert_eq!(harness.get_all_by_label("tomorrow 06:00 UTC").count(), 2);
    assert_eq!(harness.get_all_by_label("none").count(), 4);
    harness.get_by_role_and_label(Role::Cell, "INV");

    // Mortise never asks for env, and asks for nothing else than these.
    for path in stub.asked() {
        assert!(
            [
                "/actuator",
                "/actuator/health",
                "/actuator/info",
                "/actuator/mappings",
                "/actuator/scheduledtasks"
            ]
            .contains(&path.as_str()),
            "asked for {path}"
        );
    }
}

#[test]
fn a_module_shows_its_routes_and_its_tasks() {
    let stub = shop_api_actuator();
    let (_desk, mut harness) = connected(&stub);

    harness
        .get_by_role_and_label(Role::ListBoxOption, "Checkout")
        .click();
    harness.run_steps(3);
    harness.get_by_label("18 (12 API, 6 web)");
    harness.get_by_role_and_label(Role::Heading, "Routes");
    harness.get_by_label("18 routes, read from local");
    harness.get_by_label("api/checkout/things/0");
    harness.get_by_label("checkout.things.0");
    // The action without its namespace.
    harness.get_by_label("ThingController@show0");
    assert_eq!(harness.get_all_by_label("api, auth:sanctum").count(), 12);
    assert_eq!(harness.get_all_by_label("web, auth").count(), 6);
    // No task of the app is a class of Checkout.
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Scheduled tasks")
            .is_none()
    );
    // The runtime box belongs to the app, not to a module.
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Runtime on local")
            .is_none()
    );

    harness
        .get_by_role_and_label(Role::ListBoxOption, "Inventory")
        .click();
    harness.run_steps(3);
    harness.get_by_role_and_label(Role::Heading, "Scheduled tasks");
    harness.get_by_label("1 task");
    harness.get_by_label(r"Modules\Inventory\Jobs\SyncStock");
    harness.get_by_label("9 (6 API, 3 web)");
}

#[test]
fn an_app_without_the_actuator_says_how_to_install_it() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    let lock = std::fs::read_to_string(root.join("composer.lock")).unwrap();
    std::fs::write(
        root.join("composer.lock"),
        lock.replace("laravel-actuator", "laravel-other"),
    )
    .unwrap();
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();

    harness.get_by_role_and_label(Role::Heading, "Runtime");
    harness.get_by_label(
        "Runtime status needs nwidart/laravel-actuator. shop-api has no entry for it in \
         composer.lock. Run composer require nwidart/laravel-actuator in the app.",
    );
    harness.get_by_label("composer require nwidart/laravel-actuator");
    // No line in the index header, no cell in the stats, no routes and no tasks.
    assert!(harness.query_by_label_contains("on local").is_none());
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Routes by module")
            .is_none()
    );

    harness
        .get_by_role_and_label(Role::Button, "Copy command")
        .click();
    harness.run_steps(3);
    harness.get_by_label("Copied the command.");
}

#[test]
fn an_app_without_an_environment_offers_to_add_one() {
    let desk = Desk::new();
    let root = desk.app("shop-api");
    std::fs::write(
        root.join(".env"),
        "APP_NAME=Shop\nAPP_URL=https://shop-api.test\nDB_PASSWORD=never-read\n",
    )
    .unwrap();
    desk.knows(&[(&root, "SA")]);
    let mut harness = desk.start();

    harness.get_by_label(
        "No environment is connected. Add the URL of a running shop-api to read its runtime status.",
    );
    assert!(harness.query_by_label_contains("on local").is_none());

    // The button opens Settings on a new environment, with the URL from .env.
    harness
        .get_by_role_and_label(Role::Button, "Add environment")
        .click();
    harness.run_steps(4);
    harness.get_by_role_and_label(Role::Heading, "Environments of shop-api");
    harness.get_by_role_and_label(Role::Row, "Environment local");
    let saved = desk.saved();
    assert_eq!(
        saved.apps[0].environments,
        [Environment {
            name: "local".into(),
            base_url: "https://shop-api.test".into(),
            path: "actuator".into(),
            credentials: Credentials::None,
        }]
    );
    assert_eq!(saved.apps[0].environment.as_deref(), Some("local"));
}

#[test]
fn an_app_that_cannot_be_reached_says_no_answer() {
    // A port nothing listens on.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let local = Environment {
        name: "local".into(),
        base_url: base_url.clone(),
        path: "actuator".into(),
        credentials: Credentials::None,
    };
    let (_desk, mut harness) = with_environments(&[local]);
    wait_for_answer(&mut harness);

    // "No answer" is Mortise's word: in the box, the stat cell and the index header.
    assert_eq!(harness.get_all_by_label("No answer on local").count(), 2);
    harness.get_by_label("No answer");
    harness.get_by_label(&format!(
        "Cannot reach local. No answer from {base_url}/actuator: connection refused. Start the \
         app, or correct the URL in Settings."
    ));
    // One box takes the place of the three.
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Routes by module")
            .is_none()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Scheduled tasks")
            .is_none()
    );

    harness.get_by_role_and_label(Role::Button, "Try again");
    harness
        .get_by_role_and_label(Role::Button, "Open settings")
        .click();
    harness.run_steps(4);
    harness.get_by_role_and_label(Role::Heading, "Environments of shop-api");
}

#[test]
fn try_again_asks_at_once() {
    // No actuator at the address yet.
    let stub = StubActuator::start();
    let (_desk, mut harness) = with_environments(&[environment("local", &stub)]);
    wait_for_answer(&mut harness);
    harness.get_by_label_contains("local answered 404 for /actuator.");
    assert_eq!(harness.get_all_by_label("No answer on local").count(), 2);

    // It appears: "Try again" does not wait for the next regular check.
    let healthy = shop_api_actuator();
    for path in [
        "/actuator",
        "/actuator/health",
        "/actuator/info",
        "/actuator/mappings",
        "/actuator/scheduledtasks",
    ] {
        stub.set(path, healthy.reply(path).unwrap());
    }
    // A 404 has no "Try again": the address is wrong. Ctrl R reads the app again,
    // from disk and from the actuator.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::R);
    wait_until(&mut harness, "Up", |app| {
        matches!(app.runtime().answer.as_deref(), Some(Answer::Answered(_)))
    });
    assert_eq!(harness.get_all_by_label("Up on local").count(), 2);
    harness.get_by_label("106 routes, read from local");
}

#[test]
fn endpoints_that_are_not_exposed_are_named_with_the_line_to_add() {
    let stub = shop_api_actuator();
    stub.expose(&["health", "info"]);
    let (_desk, mut harness) = connected(&stub);

    harness.get_by_label(
        "mappings and scheduledtasks are not exposed on local. Only health is exposed by \
         default. Add them to exposure.include in config/actuator.php.",
    );
    harness.get_by_label_contains("'include' => ['health', 'info', 'mappings', 'scheduledtasks'],");
    // Routes and scheduled tasks say so in the place of their content.
    harness.get_by_label(
        "mappings is not exposed on local. Add it to exposure.include in config/actuator.php.",
    );
    harness.get_by_label(
        "scheduledtasks is not exposed on local. Add it to exposure.include in config/actuator.php.",
    );
    // What is exposed is shown.
    harness.get_by_label("local, Laravel 13.2.0");
    assert!(!stub.asked().iter().any(|path| path.ends_with("mappings")));

    harness
        .get_by_role_and_label(Role::Button, "Copy line")
        .click();
    harness.run_steps(3);
    harness.get_by_label("Copied the line.");

    // The app exposes them: "Try again" reads them.
    stub.expose(&["health", "info", "mappings", "scheduledtasks"]);
    harness
        .get_by_role_and_label(Role::Button, "Try again")
        .click();
    wait_until(
        &mut harness,
        "the routes",
        |app| matches!(app.runtime().answer.as_deref(), Some(Answer::Answered(runtime)) if runtime.routes.is_some()),
    );
    harness.get_by_label("106 routes, read from local");
}

#[test]
fn refused_credentials_are_not_authorized() {
    let stub = shop_api_actuator();
    stub.require_header("authorization", "Bearer right");
    let (_desk, harness) = connected(&stub);

    harness.get_by_label("Not authorized");
    // The index header has one word for an app that gives nothing.
    harness.get_by_label("No answer on local");
    harness.get_by_label("Not authorized on local");
    harness.get_by_label(&format!(
        "local refused the credentials. {}/actuator answered 401. Replace the credentials for \
         local in Settings.",
        stub.base_url
    ));
    harness.get_by_role_and_label(Role::Button, "Open settings");
}

#[test]
fn hidden_components_name_the_setting_that_hides_them() {
    let stub = shop_api_actuator();
    stub.set("/actuator/health", Reply::ok(r#"{"status":"UP"}"#));
    let (_desk, harness) = connected(&stub);
    harness.get_by_label_contains(
        "Components are hidden. The app shows them only when health.show_details allows it",
    );
    assert!(harness.query_by_label("ping").is_none());
}

#[test]
fn an_app_that_is_down_quotes_the_failing_component() {
    let stub = shop_api_actuator();
    stub.set(
        "/actuator/health",
        Reply::status(
            503,
            r#"{"status":"DOWN","components":{"ping":{"status":"UP"},"readinessState":{"status":"UP"},
            "db":{"status":"DOWN","details":{"error":"Illuminate\\Database\\QueryException: SQLSTATE[HY000] [2002] Connection refused."}},
            "cache":{"status":"UP","details":{"store":"redis"}}}}"#,
        ),
    );
    let (_desk, harness) = connected(&stub);

    assert_eq!(harness.get_all_by_label("Down on local").count(), 2);
    // The status and the component.
    assert_eq!(harness.get_all_by_label("Down").count(), 2);
    harness.get_by_label("db");
    // The detail is the app's own text, and nothing is added to it.
    harness.get_by_label(
        r"Illuminate\Database\QueryException: SQLSTATE[HY000] [2002] Connection refused.",
    );
    // The components that are up share one line.
    harness.get_by_label("ping, readinessState, cache");
    harness.get_by_label("3 up");
    // A 503 is an answer: routes and tasks are still shown.
    harness.get_by_label("106 routes, read from local");
}

#[test]
fn an_app_in_maintenance_is_out_of_service() {
    let stub = shop_api_actuator();
    stub.set(
        "/actuator/health",
        Reply::status(
            503,
            r#"{"status":"OUT_OF_SERVICE","components":{"ping":{"status":"UP"},"readinessState":{"status":"OUT_OF_SERVICE"}}}"#,
        ),
    );
    let (_desk, harness) = connected(&stub);
    assert_eq!(
        harness.get_all_by_label("Out of service on local").count(),
        2
    );
    assert_eq!(harness.get_all_by_label("Out of service").count(), 2);
    harness.get_by_label(
        "readinessState is out of service. The app reports this while it is in maintenance mode.",
    );
    harness.get_by_label("1 up");
}

#[test]
fn a_500_is_no_answer_with_the_sentence_about_the_configuration() {
    let stub = shop_api_actuator();
    stub.set(
        "/actuator/health",
        Reply::status(500, r#"{"message":"Server Error"}"#),
    );
    let (_desk, harness) = connected(&stub);
    assert_eq!(harness.get_all_by_label("No answer on local").count(), 2);
    harness.get_by_label(
        "local answered 500 for /actuator/health. The actuator configuration has an error. Read \
         the app log.",
    );
}

#[test]
fn the_switch_in_the_runtime_box_chooses_the_environment() {
    let local = shop_api_actuator();
    let staging = shop_api_actuator();
    staging.set(
        "/actuator/health",
        Reply::status(503, r#"{"status":"DOWN"}"#),
    );
    let (desk, mut harness) = with_environments(&[
        environment("local", &local),
        environment("staging", &staging),
    ]);
    wait_for_answer(&mut harness);
    assert_eq!(harness.get_all_by_label("Up on local").count(), 2);
    let before = staging.asked().len();
    // Only the chosen environment is asked.
    assert_eq!(before, 0);

    harness
        .get_by_role_and_label(Role::RadioButton, "staging")
        .click();
    harness.run_steps(3);
    wait_until(
        &mut harness,
        "staging answers",
        |app| matches!(app.runtime().answer.as_deref(), Some(Answer::Answered(runtime)) if runtime.status == Status::Down),
    );
    harness.get_by_role_and_label(Role::Heading, "Runtime on staging");
    assert_eq!(harness.get_all_by_label("Down on staging").count(), 2);
    // The choice is kept.
    assert_eq!(desk.saved().apps[0].environment.as_deref(), Some("staging"));
    assert_eq!(status(&harness), Some(Status::Down));
}

#[test]
fn only_the_app_on_screen_is_asked() {
    let shop = shop_api_actuator();
    let crm = shop_api_actuator();
    let desk = Desk::new();
    let shop_root = desk.app("shop-api");
    let crm_root = desk.app("crm");
    desk.knows(&[(&shop_root, "SA"), (&crm_root, "CR")]);
    let mut settings = desk.saved();
    settings.apps[0].environments = vec![environment("local", &shop)];
    settings.apps[1].environments = vec![environment("local", &crm)];
    settings.save(&desk.settings_path()).unwrap();
    let mut harness = desk.start();
    wait_for_answer(&mut harness);
    assert!(crm.asked().is_empty());

    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::Num2);
    harness.run_steps(3);
    wait_for_answer(&mut harness);
    assert!(!crm.asked().is_empty());
}
