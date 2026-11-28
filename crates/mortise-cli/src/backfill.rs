//! `mortise backfill`: sends one push document for each of the past weeks of the default
//! branch, so that Mortise Cloud has a history on the first day.
//!
//! Mortise Cloud says which period to fill and which commits it has. Each commit is read
//! on its own, as `mortise push` reads the checkout: nothing is compared and nothing is
//! kept. A plan reason, a rate limit or an outage never fails the build.

use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::path::PathBuf;

use mortise_core::{
    PastCommit, PushBackfill, PushContext, PushGit, RulesSource, byte_limit, push_document,
    read_past_app, read_repository, weekly_commits,
};
use serde_json::Value;

use crate::push::{self, Cloud};
use crate::send::{self, Outcome};
use crate::{github, say, strings};

/// The path of the endpoint that says which past commits to send.
const BACKFILL: &str = "/api/v1/backfill";

#[derive(Default)]
pub(crate) struct Options {
    /// The app folder.
    pub(crate) path: PathBuf,
    /// Read no commit older than this many weeks.
    pub(crate) weeks: Option<u32>,
    /// Print the commits; read and send nothing.
    pub(crate) dry_run: bool,
    /// The default branch.
    pub(crate) branch: Option<String>,
    pub(crate) repository: Option<String>,
}

/// What Mortise Cloud asks for.
struct Asked {
    /// The start of the period, in seconds since 1970.
    since: i64,
    /// The commits it has.
    have: BTreeSet<String>,
    /// The default branch that is set on the app in the cloud, when it says so.
    default_branch: Option<String>,
}

/// The answer of the cloud to the question. `Ok(None)` when it needs nothing, `Err` when
/// this Mortise cannot read it.
fn asked(answer: &Value) -> Result<Option<Asked>, ()> {
    match answer["needed"].as_bool() {
        Some(true) => {}
        Some(false) => return Ok(None),
        None => return Err(()),
    }
    // One commit per week is the only pace this Mortise knows.
    if answer["every"] != "week" {
        return Err(());
    }
    let since: jiff::Timestamp = answer["since"]
        .as_str()
        .and_then(|since| since.parse().ok())
        .ok_or(())?;
    Ok(Some(Asked {
        // A commit has a whole second. One in the same second as a later cutoff is
        // before it.
        since: since.as_second() + i64::from(since.subsec_nanosecond() > 0),
        have: answer["have"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|commit| commit.as_str().map(str::to_ascii_lowercase))
            .collect(),
        default_branch: answer["default_branch"].as_str().map(str::to_owned),
    }))
}

/// Runs the backfill and gives the exit code.
pub(crate) fn run(options: &Options) -> u8 {
    let github = github::read();

    // A shallow clone has no past commit, with or without a token.
    let repository = read_repository(&options.path);
    let Some((repository, head)) = repository
        .as_ref()
        .and_then(|repository| Some((repository, repository.commit.as_ref()?)))
    else {
        say(&strings::backfill_without_git(
            &options.path.to_string_lossy(),
        ));
        return crate::USAGE;
    };
    if repository.shallow {
        say(strings::BACKFILL_SHALLOW);
        return crate::PASSED;
    }
    // A dry run with `--weeks` knows its period, so it needs no token.
    let cloud = match Cloud::read() {
        Ok(Some(cloud)) => Some(cloud),
        Ok(None) if options.dry_run && options.weeks.is_some() => None,
        Ok(None) if push::from_fork(github.as_ref()) => {
            say(strings::FORK_WITHOUT_TOKEN);
            return crate::PASSED;
        }
        Ok(None) => {
            say(&strings::no_token(strings::DRY_RUN_OF_BACKFILL));
            return crate::USAGE;
        }
        Err(message) => {
            say(&message);
            return crate::USAGE;
        }
    };

    // A document names the default branch. A dry run writes none.
    // The branch that is checked out is the one that is read. A job can check out another
    // one than its run is for. With a detached `HEAD`, the branch of the run stands in
    // only when the run is for the commit that is checked out.
    let branch = options
        .branch
        .clone()
        .or_else(|| repository.branch.clone())
        .or_else(|| {
            let github = github.as_ref()?;
            (github.commit.as_ref() == Some(head)).then(|| github.branch.clone())?
        });
    if branch.is_none() && !options.dry_run {
        say(strings::BACKFILL_BRANCH_UNKNOWN);
        return crate::USAGE;
    }

    // The period: what the cloud asks for, and never more than `--weeks`.
    let now = jiff::Timestamp::now();
    let mut since = options
        .weeks
        .map(|weeks| now.as_second() - i64::from(weeks) * 7 * 86_400);
    let mut have = BTreeSet::new();
    if let Some(cloud) = &cloud {
        let hide = |text: &str| cloud.hide(text);
        let answer = match cloud.send(BACKFILL, None) {
            Outcome::Accepted { answer } => answer,
            Outcome::NotTaken { status, problem } => {
                say(&strings::backfill_not_taken(
                    status,
                    &hide(&problem.summary()),
                ));
                return crate::PASSED;
            }
            Outcome::Unreachable { reason } => {
                say(&strings::backfill_not_asked(&hide(&reason)));
                return crate::PASSED;
            }
            Outcome::Refused { status, problem } => {
                say(&strings::backfill_refused(
                    status,
                    &hide(&problem.summary()),
                ));
                return crate::REFUSED;
            }
        };
        match asked(&answer) {
            Ok(Some(asked)) => {
                // The cloud takes past commits of its default branch only. Nothing was
                // read, so the run is not done either.
                if let (Some(branch), Some(default)) = (&branch, &asked.default_branch)
                    && branch != default
                {
                    say(&strings::backfill_other_branch(branch, &hide(default)));
                    return crate::PASSED;
                }
                since = Some(since.map_or(asked.since, |weeks| weeks.max(asked.since)));
                have = asked.have;
            }
            Ok(None) => {
                println!("{}", strings::BACKFILL_NOT_NEEDED);
                return crate::PASSED;
            }
            Err(()) => {
                say(strings::BACKFILL_ANSWER_NOT_READ);
                return crate::PASSED;
            }
        }
    }
    let since = since.expect("the cloud or --weeks gives the period");

    // The last commit of each week. `mortise push` sends the one that is checked out.
    // The walk starts at the commit that was checked out when the run began.
    let Some(commits) = weekly_commits(&options.path, head, since) else {
        say(&strings::backfill_without_git(
            &options.path.to_string_lossy(),
        ));
        return crate::USAGE;
    };
    let commits: Vec<PastCommit> = commits
        .into_iter()
        .filter(|commit| commit.commit != *head && !have.contains(&commit.commit))
        .collect();
    if commits.is_empty() {
        println!("{}", strings::BACKFILL_NO_COMMITS);
        return match &cloud {
            Some(cloud) if !options.dry_run => done(cloud, options),
            _ => crate::PASSED,
        };
    }
    if options.dry_run {
        let lines: String = commits
            .iter()
            .map(|commit| format!("{} {}\n", commit.commit, time(commit.committed_at)))
            .collect();
        let mut out = std::io::stdout().lock();
        return match out.write_all(lines.as_bytes()).and_then(|()| out.flush()) {
            Ok(()) => crate::PASSED,
            // A closed pipe, as with `| head`, is not an error.
            Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => crate::PASSED,
            Err(error) => {
                say(&strings::not_written(&error.to_string()));
                crate::NOT_WRITTEN
            }
        };
    }
    let cloud = cloud.expect("only a dry run goes on without a token");

    // Read every commit first: the place of a document in the run counts only the
    // commits that can be sent. A document waits as its gzipped JSON.
    let mut documents: Vec<(&PastCommit, Vec<u8>)> = Vec::new();
    for commit in &commits {
        let short = &commit.commit[..commit.commit.len().min(7)];
        let app = match read_past_app(&options.path, &repository.app_path, commit) {
            Ok(app) => app,
            Err(error) => {
                let reason = strings::past_error(&error, &repository.app_path);
                say(&strings::backfill_left_out(short, &reason));
                continue;
            }
        };
        if let RulesSource::Invalid(message) = &app.rules_source {
            say(&strings::backfill_invalid_rules(short, message));
        }
        let context = PushContext {
            generated_at: push::utc(jiff::Timestamp::now()),
            tool_version: env!("CARGO_PKG_VERSION").to_owned(),
            app_path: repository.app_path.clone(),
            git: PushGit {
                repository: options
                    .repository
                    .clone()
                    .or_else(|| github.as_ref().and_then(|github| github.repository.clone())),
                commit: commit.commit.clone(),
                head_commit: None,
                branch: branch.clone(),
                base_branch: None,
                base_commit: None,
                pull_request: None,
                shallow: false,
                committed_at: Some(time(commit.committed_at)),
            },
            ci: push::ci(github.as_ref()),
            environment: None,
            // `placed` writes the place, once the run knows how many it sends.
            backfill: Some(PushBackfill { position: 1, of: 1 }),
        };
        let document = push_document(app, context);
        if let Some(limit) = document.over_limit() {
            say(&strings::backfill_left_out(
                short,
                &strings::limit_passed(&limit),
            ));
            continue;
        }
        let json = serde_json::to_vec(&document).expect("the document is plain data");
        documents.push((commit, push::gzip(&json)));
    }
    // The size of a body is known only with its place in it. A body over a limit is left
    // out, and the others are placed again.
    let bodies = loop {
        let of = u32::try_from(documents.len()).unwrap_or(u32::MAX);
        let mut bodies = Vec::new();
        let over = (1..)
            .zip(&documents)
            .find_map(|(position, (commit, body))| {
                let (size, body) = placed(body, position, of);
                let limit = byte_limit(size, body.len());
                bodies.push(body);
                limit.map(|limit| (bodies.len() - 1, *commit, limit))
            });
        let Some((at, commit, limit)) = over else {
            break bodies;
        };
        let short = &commit.commit[..commit.commit.len().min(7)];
        say(&strings::backfill_left_out(
            short,
            &strings::limit_passed(&limit),
        ));
        documents.remove(at);
    };
    if documents.is_empty() {
        println!("{}", strings::BACKFILL_NONE_READ);
        return done(&cloud, options);
    }

    // The newest first: a run that stops early keeps the most useful part.
    let hide = |text: &str| cloud.hide(text);
    let mut warned = BTreeSet::new();
    let mut sent: Option<(usize, i64)> = None;
    let mut code = crate::PASSED;
    for (position, ((commit, _), body)) in (1usize..).zip(documents.iter().zip(&bodies)) {
        let short = &commit.commit[..commit.commit.len().min(7)];
        match cloud.send(push::SNAPSHOTS, Some(body)) {
            Outcome::Accepted { answer } => {
                for warning in send::warnings(&answer) {
                    // The same warning for every commit is said once.
                    if warned.insert(warning.clone()) {
                        say(&strings::cloud_warning(&hide(&warning)));
                    }
                }
                sent = Some((position, commit.committed_at));
            }
            Outcome::NotTaken { status, problem } => {
                say(&strings::backfill_stopped_not_taken(
                    short,
                    status,
                    &hide(&problem.summary()),
                ));
                break;
            }
            Outcome::Unreachable { reason } => {
                say(&strings::backfill_stopped_unreachable(
                    short,
                    &hide(&reason),
                ));
                break;
            }
            Outcome::Refused { status, problem } => {
                say(&strings::backfill_stopped_refused(
                    short,
                    status,
                    &hide(&problem.summary()),
                ));
                for (pointer, message) in &problem.errors {
                    say(&strings::refused_member(&hide(pointer), &hide(message)));
                }
                code = crate::REFUSED;
                break;
            }
        }
    }
    let summary = match sent {
        Some((count, oldest)) => {
            let day = jiff::Timestamp::from_second(oldest)
                .map(|oldest| oldest.strftime("%-d %b %Y").to_string())
                .unwrap_or_default();
            strings::backfill_sent(count, &day)
        }
        None => strings::BACKFILL_SENT_NONE.to_owned(),
    };
    println!("{summary}");
    code
}

/// Tells the cloud that a run it asked for is done with nothing to send, so that it
/// stops asking. Gives the exit code. A run with `--weeks` is a test run over a part of
/// the period: it says nothing, so the history does not count as filled.
fn done(cloud: &Cloud, options: &Options) -> u8 {
    if options.weeks.is_some() {
        return crate::PASSED;
    }
    let hide = |text: &str| cloud.hide(text);
    match cloud.send(BACKFILL, Some(&[])) {
        Outcome::Accepted { .. } => crate::PASSED,
        Outcome::NotTaken { status, problem } => {
            let reason = format!("HTTP {status}: {}", hide(&problem.summary()));
            say(&strings::backfill_not_told(
                reason.trim_end_matches([':', ' ', '.']),
            ));
            crate::PASSED
        }
        Outcome::Unreachable { reason } => {
            say(&strings::backfill_not_told(&hide(&reason)));
            crate::PASSED
        }
        // Only a wrong token is for the user to fix. Another refusal, such as from a
        // cloud that does not know this request yet, does not fail the build.
        Outcome::Refused {
            status: 401,
            problem,
        } => {
            say(&strings::backfill_refused(401, &hide(&problem.summary())));
            crate::REFUSED
        }
        Outcome::Refused { status, .. } => {
            say(&strings::backfill_not_told(&format!("HTTP {status}")));
            crate::PASSED
        }
    }
}

/// A committer date as RFC 3339 in UTC.
fn time(seconds: i64) -> String {
    jiff::Timestamp::from_second(seconds)
        .map(push::utc)
        .unwrap_or_default()
}

/// `body`, a gzipped push document, with its place in the run: the size of its JSON, and
/// the body as sent.
fn placed(body: &[u8], position: u32, of: u32) -> (usize, Vec<u8>) {
    let mut json = Vec::new();
    flate2::read::GzDecoder::new(body)
        .read_to_end(&mut json)
        .expect("the body was gzipped here");
    let mut document: Value = serde_json::from_slice(&json).expect("the body is JSON");
    document["backfill"] = serde_json::json!({ "position": position, "of": of });
    let json = serde_json::to_vec(&document).expect("the document is plain data");
    (json.len(), push::gzip(&json))
}
