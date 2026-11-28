//! `mortise backfill` against a stub of the cloud on this computer: which commits it
//! sends, what each document says, and its exit codes. Reading a past commit is tested in
//! `mortise-core` (`tests/git.rs`). No test reaches the network.

mod common;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Output;

use common::{TOKEN, copy_dir, fixture, git, git_at, git_says, mortise, requests, stderr, stub};
use serde_json::Value;

const DAY: i64 = 86_400;

/// An answer of the stub: status, extra header lines and body.
type Answer = (u16, &'static str, &'static str);

/// Monday of this week, 00:00 UTC.
fn monday() -> i64 {
    let now = jiff::Timestamp::now().as_second();
    let today = now.div_euclid(DAY);
    // 1 January 1970 was a Thursday.
    (today - (today + 3).rem_euclid(7)) * DAY
}

/// A repository with shop-api in `apps/shop-api`, and these commits on `main`, oldest
/// first. All but the last are older than 30 days.
struct History {
    folder: tempfile::TempDir,
    /// 10 weeks ago: a README only.
    no_app: String,
    /// 8 weeks ago, Tuesday and Thursday: the app, then a change to Checkout.
    week_8: String,
    /// 7 weeks ago: a change to Billing.
    week_7: String,
    /// 6 weeks ago: a change to Billing.
    week_6: String,
    /// This week: a change to Checkout. `HEAD`.
    head: String,
}

impl History {
    fn repository(&self) -> PathBuf {
        self.folder.path().join("origin")
    }

    fn app(&self) -> PathBuf {
        self.repository().join("apps/shop-api")
    }
}

fn history() -> History {
    let folder = tempfile::tempdir().unwrap();
    let repository = folder.path().join("origin");
    let app = repository.join("apps/shop-api");
    std::fs::create_dir_all(&repository).unwrap();
    git(&repository, &["init", "-q", "-b", "main"]);
    let commit = |time: i64, file: &str| {
        let path = repository.join(file);
        let mut content = std::fs::read_to_string(&path).unwrap_or_default();
        content.push_str("\n// changed\n");
        std::fs::write(path, content).unwrap();
        git(&repository, &["add", "-A"]);
        git_at(&repository, Some(time), &["commit", "-q", "-m", "change"]);
        git_says(&repository, &["rev-parse", "HEAD"])
    };
    let monday = monday();
    let no_app = commit(monday - 70 * DAY + DAY, "README.md");
    // A commit with a date before any period stands between commits in it.
    commit(monday - 300 * DAY, "README.md");
    copy_dir(&fixture(), &app);
    commit(monday - 56 * DAY + DAY, "README.md");
    let order = "apps/shop-api/Modules/Checkout/app/Models/Order.php";
    let invoice = "apps/shop-api/Modules/Billing/app/Models/Invoice.php";
    History {
        no_app,
        week_8: commit(monday - 56 * DAY + 3 * DAY, order),
        week_7: commit(monday - 49 * DAY + 2 * DAY, invoice),
        week_6: commit(monday - 42 * DAY + DAY, invoice),
        head: commit(monday, order),
        folder,
    }
}

fn backfill(args: &[&str], vars: &[(&str, &str)]) -> Output {
    mortise("backfill", args, vars)
}

fn stdout(output: &Output) -> String {
    common::stdout(output).replace("\r\n", "\n")
}

/// The document of a request.
fn document(body: &[u8]) -> Value {
    let mut json = String::new();
    flate2::read::GzDecoder::new(body)
        .read_to_string(&mut json)
        .unwrap();
    serde_json::from_str(&json).unwrap()
}

fn day(time: i64) -> String {
    jiff::Timestamp::from_second(time)
        .unwrap()
        .strftime("%-d %b %Y")
        .to_string()
}

fn utc(time: i64) -> String {
    jiff::Timestamp::from_second(time)
        .unwrap()
        .strftime("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

/// The answer of the cloud that asks for the period from `since` on.
fn asks(since: i64, have: &[&str]) -> &'static str {
    let answer = serde_json::json!({
        "needed": true, "since": utc(since), "every": "week", "have": have,
    });
    Box::leak(answer.to_string().into_boxed_str())
}

fn clean(checkout: &Path) -> bool {
    git_says(checkout, &["status", "--porcelain"]).is_empty()
}

#[test]
fn a_clone_without_file_contents_sends_the_last_commit_of_each_week_newest_first() {
    let history = history();
    let origin = history.repository();
    git(&origin, &["config", "uploadpack.allowFilter", "true"]);
    git(
        &origin,
        &["config", "uploadpack.allowAnySHA1InWant", "true"],
    );
    let clone = history.folder.path().join("clone");
    git(
        history.folder.path(),
        &[
            "clone",
            "-q",
            "--no-local",
            "--filter=blob:none",
            origin.to_str().unwrap(),
            "clone",
        ],
    );
    let app = clone.join("apps/shop-api");
    let app = app.to_str().unwrap();
    let monday = monday();

    // The cloud has the commit of 7 weeks ago. A rate limit comes between two documents.
    let accepted = r#"{"id": "snp_1", "status": "queued", "warnings": ["Old plan."]}"#;
    let (url, taken) = stub(vec![
        (200, "", asks(monday - 80 * DAY, &[&history.week_7])),
        (202, "", accepted),
        (429, "Retry-After: 0\r\n", "{}"),
        (202, "", accepted),
    ]);
    let output = backfill(
        &[app, "--repository", "acme/shop-api"],
        &[("MORTISE_TOKEN", TOKEN), ("MORTISE_CLOUD_URL", &url)],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let week_8 = monday - 56 * DAY + 3 * DAY;
    assert_eq!(
        stdout(&output),
        format!("Sent 2 past commits, back to {}.\n", day(week_8))
    );
    // The commit before the app is left out, and the run goes on.
    let left_out = format!(
        "mortise: commit {} is left out: it has no folder apps/shop-api.",
        &history.no_app[..7]
    );
    assert!(stderr(&output).contains(&left_out), "{}", stderr(&output));
    assert_eq!(stderr(&output).matches("Old plan.").count(), 1);
    assert!(clean(&clone), "the working tree changed");

    let [asked, first, limited, second] = requests(&taken).try_into().ok().unwrap();
    assert_eq!(asked.line, "GET /api/v1/backfill HTTP/1.1");
    assert_eq!(
        asked.header("authorization"),
        Some(format!("Bearer {TOKEN}").as_str())
    );
    assert_eq!(first.line, "POST /api/v1/snapshots HTTP/1.1");
    // The document that met the rate limit is sent again as it was.
    assert_eq!(limited.body, second.body);

    let first = document(&first.body);
    assert_eq!(first["app_path"], "apps/shop-api");
    assert_eq!(first["environment"], Value::Null);
    assert_eq!(
        first["git"],
        serde_json::json!({
            "repository": "acme/shop-api",
            "commit": history.week_6,
            "head_commit": null,
            "branch": "main",
            "base_branch": null,
            "base_commit": null,
            "pull_request": null,
            "shallow": false,
            "committed_at": utc(monday - 42 * DAY + DAY),
        })
    );
    // Neither the commit that is checked out nor the one the cloud has is sent, and the
    // commit that cannot be read is not counted.
    assert_eq!(
        first["backfill"],
        serde_json::json!({ "position": 1, "of": 2 })
    );
    let second = document(&second.body);
    assert_eq!(second["git"]["commit"], history.week_8);
    assert_eq!(
        second["backfill"],
        serde_json::json!({ "position": 2, "of": 2 })
    );
    assert_eq!(second["snapshot"]["name"], "shop-api");
    // The commits of the 30 days before the commit, which are all older than 30 days now.
    let modules = first["snapshot"]["modules"].as_array().unwrap();
    let billing = modules.iter().find(|module| module["name"] == "Billing");
    assert_eq!(billing.unwrap()["churn"]["commits"], 3);

    // A dry run with a number of weeks asks nothing and reads nothing.
    let output = backfill(&[app, "--dry-run", "--weeks", "30"], &[]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let line = |commit: &str, time: i64| format!("{commit} {}\n", utc(time));
    assert_eq!(
        stdout(&output),
        [
            line(&history.week_6, monday - 42 * DAY + DAY),
            line(&history.week_7, monday - 49 * DAY + 2 * DAY),
            line(&history.week_8, week_8),
            line(&history.no_app, monday - 70 * DAY + DAY),
        ]
        .concat()
    );
    // The number of weeks lowers what the cloud asks for.
    let (url, _) = stub(vec![(200, "", asks(monday - 80 * DAY, &[]))]);
    let output = backfill(
        &[app, "--weeks", "1"],
        &[("MORTISE_TOKEN", TOKEN), ("MORTISE_CLOUD_URL", &url)],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), "No past commits to send.\n");
}

#[test]
fn a_refusal_stops_the_run_and_only_a_wrong_token_or_document_fails_the_build() {
    let history = history();
    let app = history.app();
    let app = app.to_str().unwrap();
    let asks = asks(monday() - 60 * DAY, &[]);
    let done = "Sent 1 past commit, back to";
    let wait = "Retry-After: 0\r\n";
    let limited = (429, wait, "{}");
    let accepted = (202, "", "{}");
    let token = r#"{"title": "The token is not valid"}"#;
    // The answers of the cloud, the exit code, standard output, and standard error.
    let cases: Vec<(Vec<Answer>, i32, &str, &str)> = vec![
        (
            vec![(200, "", r#"{"needed": false}"#)],
            0,
            "History is already filled.\n",
            "",
        ),
        (vec![(401, "", token)], 4, "", "The token is not valid."),
        (
            vec![(402, "", r#"{"title": "No history on this plan"}"#)],
            0,
            "",
            "takes no past commits for this app (HTTP 402)",
        ),
        (
            vec![(200, "", r#"{"needed": true, "every": "day"}"#)],
            0,
            "",
            "Update Mortise to fill the history.",
        ),
        (
            vec![(200, "", asks), accepted, (402, "", "{}")],
            0,
            done,
            "The backfill stops here; the build goes on.",
        ),
        // Three waits in a row for one document.
        (
            vec![
                (200, "", asks),
                accepted,
                limited,
                limited,
                limited,
                limited,
            ],
            0,
            done,
            "after 4 tries: HTTP 429",
        ),
        (
            vec![(200, "", asks), accepted, (401, "", token)],
            4,
            done,
            "refused the document of commit",
        ),
    ];
    for (answers, code, said, warned) in cases {
        let statuses: Vec<u16> = answers.iter().map(|(status, _, _)| *status).collect();
        let (url, taken) = stub(answers);
        let output = backfill(
            &[app],
            &[("MORTISE_TOKEN", TOKEN), ("MORTISE_CLOUD_URL", &url)],
        );
        let shown = format!("{statuses:?}: {}", stderr(&output));
        assert_eq!(output.status.code(), Some(code), "{shown}");
        assert!(stdout(&output).starts_with(said), "{shown}");
        assert!(stderr(&output).contains(warned), "{shown}");
        assert_eq!(requests(&taken).len(), statuses.len(), "{shown}");
    }

    // A shallow clone has no past commit: nothing is asked, with or without a token.
    let origin = history.repository();
    git(
        history.folder.path(),
        &[
            "clone",
            "-q",
            "--no-local",
            "--depth",
            "1",
            origin.to_str().unwrap(),
            "shallow",
        ],
    );
    let shallow = history.folder.path().join("shallow/apps/shop-api");
    let output = backfill(&[shallow.to_str().unwrap()], &[]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        stderr(&output).trim_end(),
        "mortise: backfill needs the full history. Set `fetch-depth: 0` on \
         `actions/checkout`, then run the workflow again."
    );
    // Without a token, only a dry run with a number of weeks goes on.
    let output = backfill(&[app, "--dry-run"], &[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("--dry-run with --weeks to see the commits"));
    assert!(history.head.len() == 40 && clean(&origin));
}
