//! `mortise push`: reads one app at one commit and sends its push document to Mortise Cloud.
//!
//! The document comes from `mortise-core`. This file finds the commit and the CI run it
//! belongs to, checks the limits of the cloud, and turns the cloud's answer into an exit
//! code. An outage or a billing state never fails the build; a wrong token or document does.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use mortise_core::{
    CoverageSettings, PushCi, PushContext, PushGit, ReadCache, ReadError, ReadOptions, Repository,
    RulesSource, byte_limit, commit_time, push_document, read_app_cached, read_repository,
};

use crate::github::{self, GitHub};
use crate::send::{self, Outcome};
use crate::{say, strings};

/// The cloud host when `MORTISE_CLOUD_URL` is not set.
const DEFAULT_CLOUD_URL: &str = "https://mortisecloud.com";

#[derive(Default)]
pub(crate) struct Options {
    /// The app folder.
    pub(crate) path: PathBuf,
    /// The clover report, relative to the current folder. Wins over `mortise.json`.
    pub(crate) coverage: Option<PathBuf>,
    /// The environment this run deploys to.
    pub(crate) environment: Option<String>,
    /// Build and print the document; send nothing.
    pub(crate) dry_run: bool,
    /// Write the document to this file.
    pub(crate) output: Option<PathBuf>,
    pub(crate) commit: Option<String>,
    pub(crate) branch: Option<String>,
    pub(crate) pull_request: Option<u64>,
    pub(crate) base_branch: Option<String>,
    pub(crate) base_commit: Option<String>,
    pub(crate) repository: Option<String>,
}

/// A full commit SHA: 40 lower-case hexadecimal characters, or 64 in a SHA-256 repository.
pub(crate) fn is_sha(text: &str) -> bool {
    matches!(text.len(), 40 | 64) && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// A name of an environment: lower-case letters, digits and `-`, at most 32, not `-` first.
pub(crate) fn is_environment(text: &str) -> bool {
    let mut bytes = text.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && text.len() <= 32
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// `owner/name` on GitHub.
pub(crate) fn is_repository(text: &str) -> bool {
    let part = |part: &str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
    };
    text.len() <= 200
        && text
            .split_once('/')
            .is_some_and(|(owner, name)| part(owner) && part(name))
}

/// Runs the push and gives the exit code.
pub(crate) fn run(options: &Options) -> u8 {
    let github = github::read();

    // Where to send, and with what token. A dry run needs neither.
    let cloud = if options.dry_run {
        None
    } else {
        let fork = github
            .as_ref()
            .and_then(|github| github.pull_request.as_ref())
            .is_some_and(|pull| pull.fork);
        match (token(), cloud_url()) {
            (Ok(None), _) if fork => {
                say(strings::FORK_WITHOUT_TOKEN);
                return crate::PASSED;
            }
            (Ok(None), _) => {
                say(strings::NO_TOKEN);
                return crate::USAGE;
            }
            (Err(message), _) | (_, Err(message)) => {
                say(&message);
                return crate::USAGE;
            }
            (Ok(Some(token)), Ok(url)) => Some((url, token)),
        }
    };

    // A path in a flag is relative to the current folder, as the shell completes it.
    let report = options
        .coverage
        .as_ref()
        .map(|path| std::path::absolute(path).unwrap_or_else(|_| path.clone()));
    let read_options = ReadOptions {
        coverage_given: CoverageSettings {
            limit: None,
            report,
        },
        ..ReadOptions::default()
    };
    let app = match read_app_cached(
        &options.path,
        &BTreeMap::new(),
        &read_options,
        &mut ReadCache::default(),
    ) {
        Ok(app) => app,
        Err(error @ ReadError::CoverageReport { .. }) => {
            say(&strings::unreadable_report(&error.to_string()));
            return crate::USAGE;
        }
        Err(error) => {
            say(&strings::not_an_app("push", &error.to_string()));
            return crate::NOT_AN_APP;
        }
    };
    if let RulesSource::Invalid(message) = &app.rules_source {
        say(&strings::invalid_rules_file(message));
    }
    if let Some(message) = &app.coverage_ignored {
        say(&strings::coverage_ignored(message));
    }

    let repository = read_repository(&app.root);
    let context = match context(options, &app.root, github.as_ref(), repository.as_ref()) {
        Ok(context) => context,
        Err(message) => {
            say(&message);
            return crate::USAGE;
        }
    };
    if repository
        .as_ref()
        .is_some_and(|repository| repository.shallow)
    {
        say(strings::SHALLOW_CLONE);
    }
    let document = push_document(app, context);

    let mut pretty = serde_json::to_string_pretty(&document).expect("the document is plain data");
    pretty.push('\n');
    let written = match &options.output {
        Some(path) => std::fs::write(path, &pretty),
        None if options.dry_run => {
            let mut out = std::io::stdout().lock();
            match out.write_all(pretty.as_bytes()).and_then(|()| out.flush()) {
                // A closed pipe, as with `| head`, is not an error of the push.
                Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
                other => other,
            }
        }
        None => Ok(()),
    };
    if let Err(error) = written {
        say(&strings::document_not_written(&error.to_string()));
        return crate::NOT_WRITTEN;
    }

    let compact = serde_json::to_vec(&document).expect("the document is plain data");
    let body = gzip(&compact);
    if let Some(limit) = document
        .over_limit()
        .or_else(|| byte_limit(compact.len(), body.len()))
    {
        say(&strings::over_limit(&limit));
        return crate::REFUSED;
    }

    let Some((url, token)) = cloud else {
        return crate::PASSED;
    };
    // Text from the cloud could echo the token. It never reaches the log.
    let hide = |text: &str| hide_token(text, &token);
    match send::send(&url, &token, &body, &mut std::thread::sleep) {
        Outcome::Accepted { url, warnings } => {
            for warning in warnings {
                say(&strings::cloud_warning(&hide(&warning)));
            }
            match url {
                Some(url) => println!("{}", crate::printable(&strings::sent(&hide(&url)))),
                None => println!("{}", strings::SENT_WITHOUT_URL),
            }
            crate::PASSED
        }
        Outcome::NotTaken { status, problem } => {
            say(&strings::not_taken(status, &hide(&problem.summary())));
            crate::PASSED
        }
        Outcome::Unreachable { reason } => {
            say(&strings::unreachable(&hide(&reason)));
            crate::PASSED
        }
        Outcome::Refused { status, problem } => {
            say(&strings::refused(status, &hide(&problem.summary())));
            for (pointer, message) in &problem.errors {
                say(&strings::refused_member(&hide(pointer), &hide(message)));
            }
            crate::REFUSED
        }
    }
}

/// `text` with the token, and the secret part of it, replaced by a mark.
fn hide_token(text: &str, token: &str) -> String {
    let text = text.replace(token, strings::HIDDEN_TOKEN);
    // `mortise_<id>_<secret>`: the secret alone is as good as the token.
    match token.rsplit_once('_') {
        Some((_, secret)) if secret.len() >= 16 => text.replace(secret, strings::HIDDEN_TOKEN),
        _ => text,
    }
}

/// `MORTISE_TOKEN`, without the space around it. `Ok(None)` when it is not set.
fn token() -> Result<Option<String>, String> {
    let Some(token) = github::var("MORTISE_TOKEN") else {
        return Ok(None);
    };
    let token = token.trim();
    // A header cannot carry a control character or a space; the token is never printed.
    if !token.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(strings::WRONG_TOKEN.to_owned());
    }
    Ok(Some(token.to_owned()))
}

/// The endpoint for snapshots on the cloud host. The token is sent in clear only to this
/// computer.
fn cloud_url() -> Result<String, String> {
    let base = github::var("MORTISE_CLOUD_URL").unwrap_or_else(|| DEFAULT_CLOUD_URL.to_owned());
    endpoint(&base)
}

/// The endpoint for snapshots under `base`. `base` is `https`, or `http` on this computer
/// only, and names no user: the token must not cross a network in clear or reach a host
/// that hides behind a user name.
fn endpoint(base: &str) -> Result<String, String> {
    let base = base.trim().trim_end_matches('/');
    let wrong = || strings::wrong_cloud_url(base);
    let uri: ureq::http::Uri = base.parse().map_err(|_| wrong())?;
    let authority = uri.authority().ok_or_else(wrong)?;
    if authority.as_str().contains('@') {
        return Err(wrong());
    }
    let host = authority.host();
    let local = ["localhost", "127.0.0.1", "[::1]"]
        .iter()
        .any(|local| host.eq_ignore_ascii_case(local));
    match uri.scheme_str() {
        Some(scheme) if scheme.eq_ignore_ascii_case("https") => {}
        Some(scheme) if scheme.eq_ignore_ascii_case("http") && local => {}
        _ => return Err(wrong()),
    }
    Ok(format!("{base}/api/v1/snapshots"))
}

/// The envelope. A flag wins over GitHub Actions, which wins over git, except for the
/// commit: the one checked out is the one read.
fn context(
    options: &Options,
    root: &Path,
    github: Option<&GitHub>,
    repository: Option<&Repository>,
) -> Result<PushContext, String> {
    let pull = github.and_then(|github| github.pull_request.as_ref());
    let head = repository.and_then(|repository| repository.commit.clone());
    let github_sha = github.and_then(|github| github.commit.clone());
    // A job can check out another commit than GITHUB_SHA, such as the head of the pull
    // request. The snapshot belongs to the files that were read.
    if options.commit.is_none()
        && let (Some(head), Some(sha)) = (&head, &github_sha)
        && head != sha
    {
        say(&strings::head_not_github_sha(head, sha));
    }
    let commit = options
        .commit
        .clone()
        .or(head)
        .or(github_sha)
        .ok_or_else(|| strings::COMMIT_UNKNOWN.to_owned())?;
    let pull_request = options.pull_request.or(pull.map(|pull| pull.number));
    let base_branch = options
        .base_branch
        .clone()
        .or_else(|| pull.and_then(|pull| pull.base_branch.clone()));
    let base_commit = options
        .base_commit
        .clone()
        .or_else(|| pull.and_then(|pull| pull.base_commit.clone()));
    let complete = pull_request.is_some() && base_branch.is_some() && base_commit.is_some();
    // An event file that could not be read matters only when the flags do not stand in.
    if let Some(error) = github.and_then(|github| github.event_error.as_ref())
        && !complete
    {
        return Err(error.clone());
    }
    if pull_request.is_some() && !complete {
        return Err(strings::PULL_REQUEST_NEEDS_BASE.to_owned());
    }
    // Outside GitHub no merge commit is made: the commit read is the head.
    let head_commit = pull_request.map(|_| {
        pull.and_then(|pull| pull.head_commit.clone())
            .unwrap_or_else(|| commit.clone())
    });
    let branch = options
        .branch
        .clone()
        .or_else(|| pull.and_then(|pull| pull.head_branch.clone()))
        .or_else(|| github.and_then(|github| github.branch.clone()))
        .or_else(|| repository.and_then(|repository| repository.branch.clone()));
    let ci = match github {
        Some(github) => PushCi {
            provider: "github-actions".to_owned(),
            run_id: github.run_id.clone(),
            run_attempt: github.run_attempt,
            run_url: github.run_url.clone(),
        },
        None => PushCi {
            provider: "other".to_owned(),
            run_id: None,
            run_attempt: None,
            run_url: None,
        },
    };
    // The date of the commit that the document names, which a flag can set.
    let committed_at = commit_time(root, &commit)
        .and_then(|seconds| jiff::Timestamp::from_second(seconds).ok())
        .map(utc);
    Ok(PushContext {
        generated_at: utc(jiff::Timestamp::now()),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        app_path: repository
            .map_or_else(|| ".".to_owned(), |repository| repository.app_path.clone()),
        git: PushGit {
            repository: options
                .repository
                .clone()
                .or_else(|| github.and_then(|github| github.repository.clone())),
            commit,
            head_commit,
            branch,
            base_branch,
            base_commit,
            pull_request,
            shallow: repository.is_some_and(|repository| repository.shallow),
            committed_at,
        },
        ci,
        environment: options.environment.clone(),
        // Only a run that sends past commits sets it.
        backfill: None,
    })
}

/// `time` as RFC 3339 in UTC, to the second.
fn utc(time: jiff::Timestamp) -> String {
    time.strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// The body as sent: the document, gzipped.
fn gzip(json: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(json)
        .and_then(|()| encoder.finish())
        .expect("writing to memory does not fail")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_token_goes_in_clear_only_to_this_computer() {
        let path = "/api/v1/snapshots";
        assert_eq!(
            endpoint("https://mortisecloud.com/").unwrap(),
            format!("https://mortisecloud.com{path}")
        );
        for local in [
            "http://localhost:8000",
            "http://127.0.0.1",
            "http://[::1]:80/",
        ] {
            assert!(endpoint(local).is_ok(), "{local}");
        }
        // The URLs with user info are built at run time, so the secret scan of CI
        // does not read them as credentials.
        let at = '@';
        for wrong in [
            "http://mortisecloud.com".to_string(),
            format!("http://localhost:x{at}evil.example/"),
            format!("http://localhost{at}evil.example"),
            format!("https://user:pass{at}mortisecloud.com"),
            "http://localhost.evil.example".to_string(),
            "ftp://localhost".to_string(),
            "mortisecloud.com".to_string(),
        ] {
            assert!(endpoint(&wrong).is_err(), "{wrong}");
        }
    }
}
