//! `mortise check` on the fixture shop-api: its exit codes and its three formats.
//! The rules themselves are tested in `mortise-core`.

mod common;

use std::process::{Command, Output};

use common::fixture;

/// A copy of shop-api with this `mortise.json`.
fn copy_with(rules: &str) -> tempfile::TempDir {
    let app = common::copy_of_fixture();
    std::fs::write(app.path().join("mortise.json"), rules).unwrap();
    app
}

fn check(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mortise"))
        .arg("check")
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn text_gives_one_line_per_problem_and_a_problem_fails() {
    let output = check(&[fixture().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "\
Modules/Checkout/app/Listeners/RequestShipment.php:6: problem: Checkout and Shipping use each other. 2 files in Checkout, 4 files in Shipping.
Modules/Checkout/app/Http/Controllers/OrderController.php:5: problem: Checkout reads Modules\\Billing\\Models\\Invoice directly. Billing offers Contracts and Events.
Modules/Checkout/app/Listeners/CreateInvoiceOnPaid.php:5: problem: Checkout reads Modules\\Billing\\Models\\Invoice directly. Billing offers Contracts and Events.
Modules/Checkout/app/Services/InvoiceSync.php:5: problem: Checkout reads Modules\\Billing\\Models\\Invoice directly. Billing offers Contracts and Events.
Modules/Notifications: warning: Notifications is 39% covered. The limit is 50%.
2 problems, 1 warning. 7 modules checked against 2 rules and 4 checks.
"
    );
}

#[test]
fn github_marks_each_file_that_reads_an_internal_class() {
    let output = check(&[fixture().to_str().unwrap(), "--format", "github"]);
    assert_eq!(output.status.code(), Some(1));
    let out = stdout(&output);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[..5],
        [
            "::error file=Modules/Checkout/app/Listeners/RequestShipment.php,line=6,title=No circular use::Checkout and Shipping use each other. 2 files in Checkout, 4 files in Shipping.",
            r"::error file=Modules/Checkout/app/Http/Controllers/OrderController.php,line=5,title=Public surface only::Checkout reads Modules\Billing\Models\Invoice directly. Billing offers Contracts and Events.",
            r"::error file=Modules/Checkout/app/Listeners/CreateInvoiceOnPaid.php,line=5,title=Public surface only::Checkout reads Modules\Billing\Models\Invoice directly. Billing offers Contracts and Events.",
            r"::error file=Modules/Checkout/app/Services/InvoiceSync.php,line=5,title=Public surface only::Checkout reads Modules\Billing\Models\Invoice directly. Billing offers Contracts and Events.",
            // A module folder is not a file, and `%` is escaped.
            "::warning title=Coverage limit::Notifications is 39%25 covered. The limit is 50%25.",
        ]
    );
}

#[test]
fn json_is_the_findings_as_serde_writes_them() {
    let output = check(&["--format=json", fixture().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let printed: mortise_core::Findings = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(
        printed,
        mortise_core::read_app(&fixture()).unwrap().findings
    );
}

#[test]
fn without_a_problem_the_check_passes_unless_warnings_fail_it() {
    let app = copy_with(
        r#"{"rules": {"no-circular-use": false, "public-surface-only": {"enabled": false}}}"#,
    );
    let path = app.path().to_str().unwrap();

    let output = check(&[path]);
    assert_eq!(output.status.code(), Some(0), "{}", stdout(&output));
    assert_eq!(
        check(&[path, "--fail-on", "warnings"]).status.code(),
        Some(1)
    );
    // Under a limit of 30 Notifications is no longer a warning.
    let output = check(&[path, "--fail-on", "warnings", "--coverage-limit", "30"]);
    assert_eq!(output.status.code(), Some(0), "{}", stdout(&output));
}

#[test]
fn with_ci_fail_off_a_problem_is_reported_and_the_check_passes() {
    let app = copy_with(r#"{"ci": {"fail": false}}"#);
    let output = check(&[app.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("2 problems, 1 warning."));
    assert!(stderr(&output).contains("mortise.json turns off"));
}

#[test]
fn a_use_outside_the_list_marks_each_file_in_every_format() {
    let layered = fixture().with_file_name("layered");
    let path = layered.to_str().unwrap();
    let output = check(&[path]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "\
Modules/Audit/app/Listeners/RecordCharge.php:5: problem: Audit uses Billing. mortise.json lets Audit use no module.
Modules/Audit/app/Services/ChargeTrail.php:6: problem: Audit uses Billing. mortise.json lets Audit use no module.
mortise.json: warning: mortise.json names Reports in allowed-use. The app has no module of that name.
1 problem, 1 warning. 5 modules checked against 3 rules and 3 checks.
"
    );

    let output = check(&[path, "--format", "github"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output).lines().take(3).collect::<Vec<_>>(),
        [
            "::error file=Modules/Audit/app/Listeners/RecordCharge.php,line=5,title=Allowed use::Audit uses Billing. mortise.json lets Audit use no module.",
            "::error file=Modules/Audit/app/Services/ChargeTrail.php,line=6,title=Allowed use::Audit uses Billing. mortise.json lets Audit use no module.",
            "::warning file=mortise.json,title=Allowed use::mortise.json names Reports in allowed-use. The app has no module of that name.",
        ]
    );

    let output = check(&[path, "--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    let printed: mortise_core::Findings = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(printed, mortise_core::read_app(&layered).unwrap().findings);

    // With ci.fail off the same break passes.
    let app = tempfile::tempdir().unwrap();
    common::copy_dir(&layered, app.path());
    let rules = app.path().join("mortise.json");
    let json = std::fs::read_to_string(&rules).unwrap();
    std::fs::write(&rules, json.replacen('{', r#"{"ci": {"fail": false},"#, 1)).unwrap();
    let output = check(&[app.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(stdout(&output).contains("1 problem, 1 warning."));
}

#[test]
fn a_folder_that_is_not_an_app_exits_3() {
    let folder = tempfile::tempdir().unwrap();
    let output = check(&[folder.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(3));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("is not a Laravel app"));
}

#[test]
fn a_wrong_command_line_exits_2() {
    let output = check(&[fixture().to_str().unwrap(), "--format", "sarif"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("--format takes text, json or github, not sarif."));

    // A flag left without its value does not take the next flag as its value.
    let output = check(&["--coverage", "--format", "github"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains("--coverage needs a value, and --format looks like an option.")
    );

    // A report given with --coverage must be a clover report: no other report is used instead.
    let folder = tempfile::tempdir().unwrap();
    let report = folder.path().join("clover.xml");
    std::fs::write(&report, "<phpunit/>").unwrap();
    let args = [fixture(), "--coverage".into(), report];
    let args: Vec<&str> = args.iter().map(|arg| arg.to_str().unwrap()).collect();
    let output = check(&args);
    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("it has no coverage element"));
}

#[test]
fn content_of_the_app_cannot_start_a_line_of_its_own() {
    let app = copy_with("{}");
    std::fs::write(
        app.path().join("modules_statuses.json"),
        r#"{"Billing": true, "Old\n::error::injected": true}"#,
    )
    .unwrap();
    for (format, escaped) in [
        ("text", r"Old\n::error::injected"),
        ("github", "Old%0A::error::injected"),
    ] {
        let output = check(&[app.path().to_str().unwrap(), "--format", format]);
        let out = stdout(&output);
        assert!(out.contains(escaped), "{out}");
        assert!(
            !out.lines()
                .any(|line| line.starts_with("::error::injected")),
            "{out}"
        );
    }
}
