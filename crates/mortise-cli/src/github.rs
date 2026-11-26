//! What a GitHub Actions run says about the commit it reads: its environment, and the event
//! file of a pull request.

use serde_json::Value;

use crate::strings;

/// The facts of a GitHub Actions run.
pub(crate) struct GitHub {
    /// `GITHUB_SHA`. On a pull request, the merge commit.
    pub(crate) commit: Option<String>,
    /// `GITHUB_REPOSITORY`, `owner/name`.
    pub(crate) repository: Option<String>,
    /// The branch of `GITHUB_REF`, when it names a branch.
    pub(crate) branch: Option<String>,
    pub(crate) run_id: Option<String>,
    pub(crate) run_attempt: Option<u32>,
    pub(crate) run_url: Option<String>,
    /// Set on `pull_request` and `pull_request_target` events.
    pub(crate) pull_request: Option<PullRequest>,
    /// Why the event file of a pull request run gives no pull request. The flags can still
    /// give it.
    pub(crate) event_error: Option<String>,
}

/// A pull request, from the event file.
pub(crate) struct PullRequest {
    pub(crate) number: u64,
    pub(crate) head_commit: Option<String>,
    pub(crate) head_branch: Option<String>,
    pub(crate) base_commit: Option<String>,
    pub(crate) base_branch: Option<String>,
    /// A `pull_request` run from another repository. GitHub gives it no secrets.
    pub(crate) fork: bool,
}

/// A variable of the environment, when it is set and not empty.
pub(crate) fn var(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

/// The run this is, on GitHub Actions. `None` elsewhere.
pub(crate) fn read() -> Option<GitHub> {
    if var("GITHUB_ACTIONS").as_deref() != Some("true") {
        return None;
    }
    let repository = var("GITHUB_REPOSITORY");
    let run_id = var("GITHUB_RUN_ID");
    let run_url = match (var("GITHUB_SERVER_URL"), &repository, &run_id) {
        (Some(server), Some(repository), Some(id)) => Some(format!(
            "{}/{repository}/actions/runs/{id}",
            server.trim_end_matches('/')
        )),
        _ => None,
    };
    let (pull_request, event_error) = match var("GITHUB_EVENT_NAME").as_deref() {
        Some(event @ ("pull_request" | "pull_request_target")) => {
            match pull_request(event == "pull_request") {
                Ok(pull) => (Some(pull), None),
                Err(error) => (None, Some(error)),
            }
        }
        _ => (None, None),
    };
    Some(GitHub {
        commit: var("GITHUB_SHA"),
        repository,
        branch: var("GITHUB_REF").and_then(|r| r.strip_prefix("refs/heads/").map(str::to_owned)),
        run_id,
        run_attempt: var("GITHUB_RUN_ATTEMPT").and_then(|n| n.parse().ok()),
        run_url,
        pull_request,
        event_error,
    })
}

/// The pull request of the event file. `can_be_fork` is false for `pull_request_target`,
/// which runs with the secrets of the base repository.
fn pull_request(can_be_fork: bool) -> Result<PullRequest, String> {
    let path = var("GITHUB_EVENT_PATH").ok_or_else(|| strings::NO_EVENT_FILE.to_owned())?;
    let event: Value = std::fs::read_to_string(&path)
        .map_err(|error| error.to_string())
        .and_then(|text| serde_json::from_str(&text).map_err(|error| error.to_string()))
        .map_err(|error| strings::unreadable_event(&path, &error))?;
    let pull = &event["pull_request"];
    let number = event["number"]
        .as_u64()
        .or_else(|| pull["number"].as_u64())
        .ok_or_else(|| strings::unreadable_event(&path, strings::NO_NUMBER))?;
    let text = |value: &Value| value.as_str().map(str::to_owned);
    // A head repository that was deleted is `null`, and is not the base either.
    let head_repository = &pull["head"]["repo"]["full_name"];
    Ok(PullRequest {
        number,
        head_commit: text(&pull["head"]["sha"]),
        head_branch: text(&pull["head"]["ref"]),
        base_commit: text(&pull["base"]["sha"]),
        base_branch: text(&pull["base"]["ref"]),
        fork: can_be_fork
            && (head_repository.is_null() || *head_repository != pull["base"]["repo"]["full_name"]),
    })
}
