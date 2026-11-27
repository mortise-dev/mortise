//! `mortise push` against a stub of the cloud on this computer: its exit codes, the GitHub
//! Actions environment, and what it sends. The document itself is tested in `mortise-core`.
//! No test reaches the network.

mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use common::fixture;
use serde_json::Value;

const TOKEN: &str = "mortise_01jd8k2x9q4m_secretsecretsecret";
const SHA: &str = "9b2e4c71d0a85f3e6c1b7a24d8e0f5a3c9d61e2b";

/// One request the stub took.
struct Request {
    line: String,
    /// Names in lower case.
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// A cloud that gives these answers, one per connection, in order: status, extra header
/// lines and body. Gives its base URL and the requests it takes.
fn stub(answers: Vec<(u16, &'static str, &'static str)>) -> (String, Receiver<Request>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let (sent, taken) = channel();
    std::thread::spawn(move || {
        for (status, headers, body) in answers {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let mut fields = Vec::new();
            loop {
                let mut field = String::new();
                reader.read_line(&mut field).unwrap();
                let Some((name, value)) = field.trim_end().split_once(':') else {
                    break;
                };
                fields.push((name.to_lowercase(), value.trim().to_owned()));
            }
            let length = fields
                .iter()
                .find(|(name, _)| name == "content-length")
                .map_or(0, |(_, value)| value.parse().unwrap());
            let mut content = vec![0; length];
            reader.read_exact(&mut content).unwrap();
            write!(
                stream,
                "HTTP/1.1 {status} Answer\r\nContent-Type: application/json\r\n{headers}\
                 Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            let _ = sent.send(Request {
                line: line.trim_end().to_owned(),
                headers: fields,
                body: content,
            });
        }
    });
    (url, taken)
}

/// Every request the stub took, waiting a little for each.
fn requests(taken: &Receiver<Request>) -> Vec<Request> {
    std::iter::from_fn(|| taken.recv_timeout(Duration::from_secs(2)).ok()).collect()
}

/// `mortise push` with these variables, and none of the variables of the CI or the proxy
/// of the computer that runs the tests. The cloud is a closed port unless `vars` names one.
fn push(args: &[&str], vars: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mortise"));
    command.arg("push").args(args);
    for (name, _) in std::env::vars() {
        let proxy = name.to_ascii_lowercase().ends_with("_proxy");
        if name.starts_with("GITHUB_") || name.starts_with("MORTISE_") || proxy {
            command.env_remove(name);
        }
    }
    command.env("MORTISE_CLOUD_URL", "http://127.0.0.1:9");
    command.envs(vars.iter().copied()).output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

/// A pull request from a fork, on GitHub Actions.
fn github_pull_request() -> Vec<(&'static str, String)> {
    let event =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/events/pull_request_from_fork.json");
    [
        ("GITHUB_ACTIONS", "true"),
        ("GITHUB_EVENT_NAME", "pull_request"),
        ("GITHUB_SHA", SHA),
        ("GITHUB_REF", "refs/pull/42/merge"),
        ("GITHUB_REPOSITORY", "acme/shop-api"),
        ("GITHUB_RUN_ID", "12093847561"),
        ("GITHUB_RUN_ATTEMPT", "2"),
        ("GITHUB_SERVER_URL", "https://github.com"),
    ]
    .into_iter()
    .map(|(name, value)| (name, value.to_owned()))
    .chain([("GITHUB_EVENT_PATH", event.to_str().unwrap().to_owned())])
    .collect()
}

fn vars<'a>(owned: &'a [(&'static str, String)]) -> Vec<(&'static str, &'a str)> {
    owned
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect()
}

#[test]
fn a_dry_run_on_a_pull_request_fills_the_envelope_and_sends_nothing() {
    let github = github_pull_request();
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("push.json");
    let output = push(
        &[
            "--dry-run",
            fixture().to_str().unwrap(),
            "--output",
            file.to_str().unwrap(),
        ],
        &vars(&github),
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    // The document goes to the file, not to standard output.
    assert_eq!(stdout(&output), "");
    let document: Value = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    assert_eq!(document["app_path"], "fixtures/shop-api");
    // The checkout of the tests is not at GITHUB_SHA: the commit read wins, with a warning.
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(fixture())
        .output()
        .unwrap();
    let head = String::from_utf8(head.stdout).unwrap().trim().to_owned();
    assert!(
        stderr(&output).contains(&format!(
            "the checkout is at {head}, not at GITHUB_SHA {SHA}"
        )),
        "{}",
        stderr(&output)
    );
    assert_eq!(
        document["git"],
        serde_json::json!({
            "repository": "acme/shop-api",
            "commit": head,
            "head_commit": "c41f0d9a7e3b52816d4c0a9f1e7b3d5a2c8e6f40",
            "branch": "split-invoices",
            "base_branch": "main",
            "base_commit": "e07a3d5c19b84f2a6d0c7e1b9f3a5d8c2e4b6a10",
            "pull_request": 42,
            "shallow": document["git"]["shallow"],
        })
    );
    assert_eq!(
        document["ci"],
        serde_json::json!({
            "provider": "github-actions",
            "run_id": "12093847561",
            "run_attempt": 2,
            "run_url": "https://github.com/acme/shop-api/actions/runs/12093847561",
        })
    );

    // Without the event file, the flags give the pull request; without them it is an error.
    let without_event: Vec<_> = vars(&github)
        .into_iter()
        .filter(|(name, _)| *name != "GITHUB_EVENT_PATH")
        .collect();
    let path = fixture();
    let path = path.to_str().unwrap();
    let flags = [
        "--pull-request",
        "7",
        "--base-branch",
        "main",
        "--base-commit",
        SHA,
    ];
    let output = push(&[&["--dry-run", path][..], &flags].concat(), &without_event);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let document: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(document["git"]["pull_request"], 7);
    let output = push(&["--dry-run", path], &without_event);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("GITHUB_EVENT_PATH is not set"));
}

#[test]
fn without_a_token_a_fork_passes_and_any_other_run_fails() {
    let github = github_pull_request();
    let path = fixture();
    let output = push(&[path.to_str().unwrap()], &vars(&github));
    assert_eq!(output.status.code(), Some(0));
    assert!(
        stderr(&output).contains("comes from a fork"),
        "{}",
        stderr(&output)
    );

    let mut on_push = vars(&github);
    on_push.push(("GITHUB_EVENT_NAME", "push"));
    let output = push(&[path.to_str().unwrap()], &on_push);
    assert_eq!(output.status.code(), Some(2));
    let message = stderr(&output);
    assert!(message.contains("MORTISE_TOKEN is not set"));
    assert!(message.contains("https://mortisecloud.com/push?utm_source=mortise-cli"));
}

#[test]
fn an_accepted_document_prints_the_snapshot_url() {
    let (url, taken) = stub(vec![(
        202,
        "",
        r#"{"id": "snp_1", "status": "queued", "duplicate": false, "replaced": false,
            "warnings": ["Deploy markers come with Team.", "mortise_01jd8k2x9q4m_secretsecretsecret is old."],
            "url": "https://cloud.test/snapshots/snp_1"}"#,
    )]);
    let output = push(
        &[
            fixture().to_str().unwrap(),
            "--commit",
            SHA,
            "--environment",
            "production",
        ],
        &[
            ("MORTISE_TOKEN", TOKEN),
            ("MORTISE_CLOUD_URL", &url),
            // A proxy would see the token of a plain http request: none is used for it.
            ("HTTP_PROXY", "http://127.0.0.1:9"),
            ("ALL_PROXY", "http://127.0.0.1:9"),
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Sent to Mortise Cloud: https://cloud.test/snapshots/snp_1\n"
    );
    assert!(stderr(&output).contains("Deploy markers come with Team."));
    // A warning that echoes the token shows a mark instead.
    assert!(stderr(&output).contains("Mortise Cloud says: [MORTISE_TOKEN] is old."));
    assert!(!stderr(&output).contains("secret") && !stdout(&output).contains("secret"));

    let [request] = requests(&taken).try_into().ok().unwrap();
    assert_eq!(request.line, "POST /api/v1/snapshots HTTP/1.1");
    assert_eq!(
        request.header("authorization"),
        Some(format!("Bearer {TOKEN}").as_str())
    );
    assert_eq!(request.header("content-type"), Some("application/json"));
    assert_eq!(request.header("content-encoding"), Some("gzip"));
    let mut json = String::new();
    flate2::read::GzDecoder::new(request.body.as_slice())
        .read_to_string(&mut json)
        .unwrap();
    let document: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(document["environment"], "production");
    assert_eq!(document["ci"]["provider"], "other");
}

#[test]
fn a_refused_document_fails_and_a_plan_limit_does_not() {
    for (status, body, code, said) in [
        (
            401,
            r#"{"title": "The token is not valid", "code": "invalid_token"}"#,
            4,
            "The token is not valid.",
        ),
        (
            422,
            r#"{"title": "The push document does not match the schema", "detail": "1 member is not valid.",
                "errors": [{"pointer": "/git/commit", "message": "Must be a full commit SHA."},
                           {"pointer": "/token", "message": "secretsecretsecret was sent."}]}"#,
            4,
            "  /git/commit: Must be a full commit SHA.",
        ),
        (
            402,
            r#"{"title": "Deploys come with Team"}"#,
            0,
            "The build goes on.",
        ),
    ] {
        let (url, taken) = stub(vec![(status, "", body)]);
        let output = push(
            &[fixture().to_str().unwrap()],
            &[("MORTISE_TOKEN", TOKEN), ("MORTISE_CLOUD_URL", &url)],
        );
        assert_eq!(
            output.status.code(),
            Some(code),
            "{status}: {}",
            stderr(&output)
        );
        assert!(
            stderr(&output).contains(said),
            "{status}: {}",
            stderr(&output)
        );
        assert!(!stderr(&output).contains("secret"), "{}", stderr(&output));
        assert_eq!(requests(&taken).len(), 1, "{status} is not tried again");
    }
}

#[test]
fn a_rate_limit_and_an_outage_are_tried_again_then_the_build_goes_on() {
    let wait = "Retry-After: 0\r\n";
    let (url, taken) = stub(vec![
        (429, wait, "{}"),
        (503, wait, "{}"),
        (502, wait, ""),
        (500, wait, ""),
    ]);
    let output = push(
        &[fixture().to_str().unwrap()],
        &[("MORTISE_TOKEN", TOKEN), ("MORTISE_CLOUD_URL", &url)],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("after 4 tries: HTTP 500"),
        "{}",
        stderr(&output)
    );
    assert_eq!(requests(&taken).len(), 4);
}

#[test]
fn a_document_over_a_limit_is_not_sent() {
    let app = common::copy_of_fixture();
    let class = format!("Modules\\Billing\\Models\\{}", "Long".repeat(275));
    std::fs::write(
        app.path().join("Modules/Checkout/app/Services/Long.php"),
        format!("<?php\n\nnamespace Modules\\Checkout\\Services;\n\nuse {class};\n"),
    )
    .unwrap();
    // The copy is in no repository, so the commit is given.
    let output = push(
        &[app.path().to_str().unwrap(), "--commit", SHA],
        &[("MORTISE_TOKEN", TOKEN)],
    );
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(stderr(&output).contains("longer than 1,024 characters"));
}
