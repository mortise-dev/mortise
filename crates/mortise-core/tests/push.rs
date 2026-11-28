//! The push document of shop-api, whose shape is a contract with Mortise Cloud (its request R8).
//!
//! The schema is `website/public/schema/push-document-1.json`, the file the docs site publishes at its
//! `$id`, with the same members as the cloud's copy. `push/example-shop-api.json` is a copy of `mortise-cloud/docs/spec/contract` at `fedac7e`.
//! `push/shop-api.json` is the document this Mortise writes. After a change to a type in the document, write it again with
//! `UPDATE_SNAPSHOTS=1 cargo test -p mortise-core --test push` and review the change: a member that
//! is removed, renamed or changes its type needs a new `SCHEMA_VERSION`.

use std::path::{Path, PathBuf};

use mortise_core::Detail;

use mortise_core::{
    PushBackfill, PushCi, PushContext, PushGit, commit_time, push_document, read_app,
};
use serde_json::{Value, json};

/// The public URL of schema version 1. A new version gets a new file and URL; this one stays.
const SCHEMA_ID: &str = "https://mortise-dev.github.io/mortise/schema/push-document-1.json";

fn here(file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/push")
        .join(file)
}

fn json(file: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(here(file)).unwrap()).unwrap()
}

/// The envelope of the cloud's example, from a shallow clone.
fn context() -> PushContext {
    let sha = |s: &str| Some(s.to_owned());
    PushContext {
        generated_at: "2026-11-24T14:03:11Z".to_owned(),
        tool_version: "0.2.0".to_owned(),
        app_path: ".".to_owned(),
        git: PushGit {
            repository: sha("acme/shop-api"),
            commit: "9b2e4c71d0a85f3e6c1b7a24d8e0f5a3c9d61e2b".to_owned(),
            head_commit: sha("c41f0d9a7e3b52816d4c0a9f1e7b3d5a2c8e6f40"),
            branch: sha("split-invoices"),
            base_branch: sha("main"),
            base_commit: sha("e07a3d5c19b84f2a6d0c7e1b9f3a5d8c2e4b6a10"),
            pull_request: Some(42),
            shallow: true,
            committed_at: sha("2026-11-24T14:01:52Z"),
        },
        ci: PushCi {
            provider: "github-actions".to_owned(),
            run_id: sha("12093847561"),
            run_attempt: Some(1),
            run_url: sha("https://github.com/acme/shop-api/actions/runs/12093847561"),
        },
        environment: None,
        backfill: None,
    }
}

/// Every member of `expected` is in `actual` with the same value, except the members named
/// in `skip`. `actual` may have more members: version 1 allows an optional member anywhere.
fn holds(actual: &Value, expected: &Value, skip: &[&str], at: &str) {
    match (actual, expected) {
        (Value::Object(actual), Value::Object(expected)) => {
            for (name, value) in expected
                .iter()
                .filter(|(name, _)| !skip.contains(&name.as_str()))
            {
                let Some(found) = actual.get(name) else {
                    panic!("{at}/{name} is missing");
                };
                holds(found, value, skip, &format!("{at}/{name}"));
            }
        }
        (Value::Array(actual), Value::Array(expected)) => {
            assert_eq!(actual.len(), expected.len(), "{at} has another length");
            for (index, (found, value)) in actual.iter().zip(expected).enumerate() {
                holds(found, value, skip, &format!("{at}/{index}"));
            }
        }
        _ => assert_eq!(actual, expected, "{at} differs"),
    }
}

/// Whether `value` is valid against the cloud's schema, with what is wrong when it is not.
fn validate(value: &Value) -> Result<(), String> {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../website/public/schema/push-document-1.json");
    let schema_json: Value = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let id = schema_json["$id"].as_str().unwrap().to_owned();
    assert_eq!(id, SCHEMA_ID);
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    compiler.enable_format_assertions();
    compiler.add_resource(&id, schema_json).unwrap();
    let schema = compiler.compile(&id, &mut schemas).unwrap();
    schemas
        .validate(value, schema)
        .map_err(|error| format!("{error:#}"))
}

/// Panics when `value` is not valid against the cloud's schema.
fn assert_valid(value: &Value) {
    if let Err(error) = validate(value) {
        panic!("not valid against the schema: {error}");
    }
}

#[test]
fn the_document_of_shop_api_keeps_its_shape() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shop-api");
    let mut document = push_document(read_app(&fixture).unwrap(), context());
    // When the checkout wrote the report: the time the cloud's example was made.
    document.snapshot.coverage_report.as_mut().unwrap().written = Some(1_795_369_951);
    let mut written = serde_json::to_string_pretty(&document).unwrap();
    written.push('\n');

    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(here("shop-api.json"), &written).unwrap();
    }
    let committed = std::fs::read_to_string(here("shop-api.json")).unwrap();
    assert_eq!(
        written,
        committed.replace("\r\n", "\n"),
        "the push document changed"
    );

    let value: Value = serde_json::from_str(&written).unwrap();
    assert_valid(&value);

    // A document of Mortise 1.0.0 has neither member and stays valid.
    let mut older = value.clone();
    older["git"].as_object_mut().unwrap().remove("committed_at");
    older.as_object_mut().unwrap().remove("backfill");
    assert_valid(&older);

    // A document of a run that sends past commits names a branch, a commit time and no deploy.
    let mut past = document.clone();
    past.backfill = Some(PushBackfill {
        position: 3,
        of: 13,
    });
    let past = |change: &dyn Fn(&mut mortise_core::PushDocument)| {
        let mut past = past.clone();
        change(&mut past);
        validate(&serde_json::to_value(&past).unwrap())
    };
    assert!(past(&|_| {}).is_err(), "a pull request");
    assert_eq!(past(&|past| past.git.pull_request = None), Ok(()));
    let without_time = past(&|past| {
        past.git.pull_request = None;
        past.git.committed_at = None;
    });
    assert!(without_time.is_err(), "no commit time");
    let deploy = past(&|past| {
        past.git.pull_request = None;
        past.environment = Some("production".to_owned());
    });
    assert!(deploy.is_err(), "a deploy");

    // The example was read from a full clone, so its churn is there and ours is not.
    holds(
        &value,
        &json("example-shop-api.json"),
        &["churn", "shallow"],
        "",
    );
}

#[test]
fn the_rule_allowed_use_needs_no_new_schema_version() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/layered");
    let document = push_document(read_app(&fixture).unwrap(), context());
    assert_eq!(document.schema_version, 1);
    let value = serde_json::to_value(&document).unwrap();
    assert_valid(&value);
    let snapshot = &value["snapshot"];
    assert_eq!(
        snapshot["rules"]["allowed_use"]["modules"]["Audit"],
        json!([])
    );
    assert_eq!(
        snapshot["findings"]["list"][0],
        json!({
            "severity": "Problem",
            "check": "AllowedUse",
            "module": "Audit",
            "detail": { "OutsideAllowedUse": { "of": "Billing", "files": 2 } }
        })
    );
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
fn no_absolute_path_leaves_when_the_modules_live_outside_the_app() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shop-api");
    let folder = tempfile::tempdir().unwrap();
    let (app, outside) = (folder.path().join("app"), folder.path().join("outside"));
    copy_dir(&fixture, &app);
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::rename(app.join("Modules"), outside.join("Modules")).unwrap();
    // A broken module and a stale status, so findings name files outside the app.
    std::fs::create_dir_all(outside.join("Modules/Broken")).unwrap();
    std::fs::write(outside.join("Modules/Broken/module.json"), "{").unwrap();
    std::fs::write(
        outside.join("modules_statuses.json"),
        r#"{"Billing": true, "Legacy": true}"#,
    )
    .unwrap();
    let absolute = |path: &Path| path.to_string_lossy().replace('\\', "/");
    let config = app.join("config/modules.php");
    let php = std::fs::read_to_string(&config)
        .unwrap()
        .replace(
            "base_path('Modules')",
            &format!("'{}'", absolute(&outside.join("Modules"))),
        )
        .replace(
            "base_path('modules_statuses.json')",
            &format!("'{}'", absolute(&outside.join("modules_statuses.json"))),
        );
    std::fs::write(&config, php).unwrap();

    let document = push_document(read_app(&app).unwrap(), context());
    let json = serde_json::to_string(&document).unwrap();
    // The temporary folder has a name of its own; no member may hold it.
    let name = folder.path().file_name().unwrap().to_string_lossy();
    assert!(!json.contains(name.as_ref()), "{json}");

    let snapshot = &document.snapshot;
    assert_eq!(snapshot.layout.modules_dir, Path::new("../outside/Modules"));
    let reference = &snapshot.links[0].refs[0];
    assert!(
        reference.file.starts_with("../outside/Modules/"),
        "{}",
        reference.file
    );
    let files: Vec<&str> = snapshot
        .findings
        .list
        .iter()
        .filter_map(|finding| match &finding.detail {
            Detail::InvalidManifest { file, .. } | Detail::StaleStatus { file, .. } => {
                Some(file.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        files,
        [
            "../outside/Modules/Broken/module.json",
            "../outside/modules_statuses.json"
        ]
    );
}

/// Runs git in `folder`, with `time` as the committer date in seconds and without the
/// settings of this computer. Gives what git prints.
fn git(folder: &Path, time: i64, args: &[&str]) -> String {
    let date = format!("{time} +0100");
    let output = std::process::Command::new("git")
        .current_dir(folder)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@mortise.invalid",
        ])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_AUTHOR_DATE", "1700000000 +0000")
        .env("GIT_COMMITTER_DATE", &date)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[test]
fn the_commit_time_is_the_committer_date_of_the_named_commit_in_the_checkout() {
    let folder = tempfile::tempdir().unwrap();
    let origin = folder.path().join("origin");
    std::fs::create_dir(&origin).unwrap();
    let commit = |time: i64| {
        std::fs::write(origin.join("file"), time.to_string()).unwrap();
        git(&origin, time, &["add", "file"]);
        git(&origin, time, &["commit", "-q", "-m", "change"]);
        git(&origin, time, &["rev-parse", "HEAD"])
    };
    let (january, february, march) = (1_800_000_000, 1_802_000_000, 1_804_000_000);
    git(&origin, january, &["init", "-q"]);
    // A clone with a filter may ask this repository for any commit by its SHA.
    git(
        &origin,
        january,
        &["config", "uploadpack.allowFilter", "true"],
    );
    git(
        &origin,
        january,
        &["config", "uploadpack.allowAnySHA1InWant", "true"],
    );
    let first = commit(january);
    let second = commit(february);

    // Not the author date, and the date of the commit that is named, not of `HEAD`.
    assert_eq!(commit_time(&origin, &first), Some(january));
    assert_eq!(commit_time(&origin, &second), Some(february));

    // A partial clone can download a commit it does not hold. The read does not.
    let filter = ["clone", "-q", "--no-local", "--filter=blob:none"];
    git(
        folder.path(),
        january,
        &[&filter[..], &["origin", "clone"]].concat(),
    );
    let third = commit(march);
    assert_eq!(commit_time(&folder.path().join("clone"), &third), None);

    // Another commit stands in for the second one: the date stays the second one's.
    git(&origin, january, &["replace", &second, &first]);
    assert_eq!(commit_time(&origin, &second), Some(february));

    // A file with the name of the revision does not hide the commit.
    std::fs::write(origin.join(format!("{second}^{{commit}}")), "").unwrap();
    assert_eq!(commit_time(&origin, &second), Some(february));
}
