//! Text of the command line tool (section 10 of the specification). The specification
//! draws no command line, so none of this text is in the spec. The sentences of the
//! findings follow those of the Findings list.

use mortise_core::{Check, Counted, MAX_TEXT, PushLimit, Severity};

pub const HELP: &str = "\
Mortise checks the boundary rules of a Laravel app that uses nwidart/laravel-modules,
and sends what it read to Mortise Cloud.

Usage: mortise check [PATH] [OPTIONS]
       mortise push [PATH] [OPTIONS]

PATH is the app folder. It defaults to the current folder.

Options of check:
  --coverage <clover.xml>     The coverage report. Wins over mortise.json
  --coverage-limit <0-100>    A module covered less is a warning. Wins over mortise.json
  --format text|json|github   How the findings are printed. Default: text
  --fail-on warnings          Warnings fail the check too

Options of push:
  --coverage <clover.xml>     The coverage report. Wins over mortise.json
  --environment <name>        This run deploys the commit to this environment
  --dry-run                   Print the document and send nothing. Needs no token
  --output <file>             Write the document to this file
  --commit <sha>              The commit that was read. Default: HEAD, or GITHUB_SHA
  --branch <name>             Default: the branch of the run, or of HEAD
  --pull-request <number>     The pull request. Default: the event of the run
  --base-branch <name>        The base branch of the pull request
  --base-commit <sha>         The tip of the base branch
  --repository <owner/name>   Default: GITHUB_REPOSITORY

  -h, --help                  Print this help
  -V, --version               Print the version

push reads the token from MORTISE_TOKEN and the host from MORTISE_CLOUD_URL
(default https://mortisecloud.com). It never prints the token.

Exit codes:
  0  check: no finding fails the check, or mortise.json sets \"ci\": { \"fail\": false }
     push: sent; or not sent because the cloud is down or for a plan reason, with a warning
  1  check: a finding fails the check
  2  The command line is wrong, the report of --coverage cannot be read, or push has no token
  3  The folder cannot be read as an app
  4  push: the cloud refused the document, or it is over a limit of the cloud
  5  The report or the document could not be written
";

pub const NO_COMMAND: &str = "No command was given.";

/// Said when `mortise.json` sets `ci.fail` to false and a finding would fail the check.
pub const CI_FAIL_OFF: &str =
    "mortise.json turns off \"Fail CI when a rule breaks\", so the check passes.";

/// `mortise: --format takes text, json or github, not xml. Run mortise --help for the options.`
pub fn usage_error(message: &str) -> String {
    format!("mortise: {message} Run mortise --help for the options.")
}

pub fn unknown_command(command: &str) -> String {
    format!("{command} is not a command. The commands are check and push.")
}

pub fn unknown_flag(command: &str, flag: &str) -> String {
    format!("{flag} is not an option of {command}.")
}

pub fn extra_argument(command: &str, argument: &str) -> String {
    format!("{command} takes one folder. {argument} is one too many.")
}

pub fn missing_value(flag: &str) -> String {
    format!("{flag} needs a value.")
}

/// `--output needs a value, and --dry-run looks like an option. ...`
pub fn flag_as_value(flag: &str, value: &str) -> String {
    format!(
        "{flag} needs a value, and {value} looks like an option. \
         For a value that starts with -, write {flag}={value}."
    )
}

pub fn takes_no_value(flag: &str) -> String {
    format!("{flag} takes no value.")
}

pub fn wrong_limit(value: &str) -> String {
    format!("--coverage-limit takes a number from 0 to 100, not {value}.")
}

pub fn wrong_format(value: &str) -> String {
    format!("--format takes text, json or github, not {value}.")
}

pub fn wrong_fail_on(value: &str) -> String {
    format!("--fail-on takes warnings, not {value}.")
}

pub fn wrong_sha(flag: &str, value: &str) -> String {
    format!("{flag} takes a full commit SHA of 40 lower-case hexadecimal characters, not {value}.")
}

pub fn wrong_environment(value: &str) -> String {
    format!(
        "--environment takes up to 32 lower-case letters, digits and hyphens, such as production, not {value}."
    )
}

pub fn wrong_pull_request(value: &str) -> String {
    format!("--pull-request takes a number from 1, not {value}.")
}

pub fn wrong_repository(value: &str) -> String {
    format!("--repository takes owner/name, such as acme/shop-api, not {value}.")
}

/// `mortise: cannot read the coverage report /app/cov.xml: it has no coverage element. ...`
pub fn unreadable_report(error: &str) -> String {
    format!("mortise: {error}. Give --coverage the clover report that the tests wrote.")
}

pub fn not_written(error: &str) -> String {
    format!(
        "mortise: the report could not be written: {error}. Check the disk or the output, then run it again."
    )
}

pub fn coverage_ignored(message: &str) -> String {
    format!(
        "mortise: the coverage block of mortise.json is ignored: {message}. The default coverage applies."
    )
}

/// `mortise: /tmp is not a Laravel app. Run mortise check in the folder of an app ...`
pub fn not_an_app(command: &str, error: &str) -> String {
    format!(
        "mortise: {error}. Run mortise {command} in the folder of an app that uses \
         nwidart/laravel-modules, or give that folder."
    )
}

pub fn invalid_rules_file(message: &str) -> String {
    format!("mortise: mortise.json cannot be read: {message}. The default rules apply.")
}

/// Stands for the token where text from the cloud names it.
pub const HIDDEN_TOKEN: &str = "[MORTISE_TOKEN]";

pub const NO_TOKEN: &str = "mortise: MORTISE_TOKEN is not set, so nothing can be sent. \
     Make a token for the app in Mortise Cloud and give it to this run as MORTISE_TOKEN, \
     or use --dry-run to see the document.";

pub const WRONG_TOKEN: &str = "mortise: MORTISE_TOKEN holds a space or a control character, \
     which a token cannot have. Copy the token again from Mortise Cloud.";

/// On a pull request from a fork, GitHub gives the run no secrets.
pub const FORK_WITHOUT_TOKEN: &str = "mortise: this pull request comes from a fork, and GitHub \
     gives such a run no secrets, so MORTISE_TOKEN is not set. Nothing was sent; the build goes on.";

pub const NO_EVENT_FILE: &str = "mortise: GITHUB_EVENT_PATH is not set on a pull request run, \
     so the pull request is unknown. Give --pull-request, --base-branch and --base-commit.";

/// Why an event file has no pull request.
pub const NO_NUMBER: &str = "it has no pull request number";

pub fn unreadable_event(path: &str, error: &str) -> String {
    format!(
        "mortise: the event file {path} cannot be read: {error}. \
         Give --pull-request, --base-branch and --base-commit."
    )
}

pub const COMMIT_UNKNOWN: &str = "mortise: the commit that was read is unknown: the app is not in \
     a git repository with a commit. Give --commit.";

/// The job checked out another commit than the one the run is for.
pub fn head_not_github_sha(head: &str, sha: &str) -> String {
    format!(
        "mortise: the checkout is at {head}, not at GITHUB_SHA {sha}. The document names {head}, \
         the commit that was read. Give --commit to name another."
    )
}

pub const PULL_REQUEST_NEEDS_BASE: &str =
    "mortise: a pull request needs its base. Give --base-branch and --base-commit too.";

pub fn wrong_cloud_url(url: &str) -> String {
    format!(
        "mortise: MORTISE_CLOUD_URL is {url}, which is not an https address. \
         The token is sent in clear only to this computer. Give an https address."
    )
}

pub fn document_not_written(error: &str) -> String {
    format!(
        "mortise: the document could not be written: {error}. Check the disk or the output, then run it again."
    )
}

/// `mortise: the document has 2,100 modules. Mortise Cloud takes at most 2,000. Nothing was sent.`
pub fn over_limit(limit: &PushLimit) -> String {
    let head = match limit {
        PushLimit::TooMany { what, count, limit } => {
            let what = match what {
                Counted::Modules => "modules",
                Counted::Links => "links between modules",
                Counted::References => "references between modules",
                Counted::Findings => "findings",
                Counted::DocumentBytes => "bytes of JSON",
                Counted::BodyBytes => "bytes after gzip",
            };
            format!(
                "the document has {} {what}. Mortise Cloud takes at most {}",
                thousands(*count),
                thousands(*limit)
            )
        }
        PushLimit::TooLong { text } => {
            let start: String = text.chars().take(60).collect();
            format!(
                "the document names {start}..., which is longer than {} characters. \
                 Mortise Cloud takes no longer path or class name",
                thousands(MAX_TEXT)
            )
        }
    };
    format!("mortise: {head}. Nothing was sent.")
}

/// `Sent to Mortise Cloud: https://mortisecloud.com/orgs/acme/apps/shop-api/snapshots/snp_01`
pub fn sent(url: &str) -> String {
    format!("Sent to Mortise Cloud: {url}")
}

pub const SENT_WITHOUT_URL: &str = "Sent to Mortise Cloud.";

pub fn cloud_warning(warning: &str) -> String {
    format!("mortise: Mortise Cloud says: {warning}")
}

/// A billing state: the build goes on.
pub fn not_taken(status: u16, problem: &str) -> String {
    format!(
        "mortise: Mortise Cloud did not take the document (HTTP {status}): {problem} The build goes on."
    )
}

/// An outage: the build goes on.
pub fn unreachable(reason: &str) -> String {
    format!(
        "mortise: Mortise Cloud could not take the document after 4 tries: {reason}. \
         Nothing was sent; the build goes on. The next run sends its own commit."
    )
}

pub fn refused(status: u16, problem: &str) -> String {
    format!(
        "mortise: Mortise Cloud refused the document (HTTP {status}): {problem} \
         Check MORTISE_TOKEN and MORTISE_CLOUD_URL, or run mortise push --dry-run to see the document."
    )
}

/// `  /git/commit: Must be a full commit SHA of 40 hexadecimal characters.`
pub fn refused_member(pointer: &str, message: &str) -> String {
    format!("  {pointer}: {message}")
}

/// `2000000` as `2,000,000`.
fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

pub fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Problem => "problem",
        Severity::Warning => "warning",
    }
}

/// The title of a line in the CI log.
pub fn check(check: Check) -> &'static str {
    match check {
        Check::NoCircularUse => "No circular use",
        Check::PublicSurfaceOnly => "Public surface only",
        Check::ValidManifest => "Valid module.json",
        Check::NamespaceMatchesFolder => "Namespace",
        Check::NoStaleStatus => "Statuses file",
        Check::CoverageLimit => "Coverage limit",
        Check::TranslationsComplete => "Translations",
    }
}

/// `2 problems, 1 warning. 7 modules checked against 2 rules and 4 checks.`
pub fn summary(
    problems: usize,
    warnings: usize,
    modules: usize,
    rules: usize,
    checks: usize,
) -> String {
    let checked = format!(
        "{} checked against {} and {}.",
        count(modules, "module", "modules"),
        count(rules, "rule", "rules"),
        count(checks, "check", "checks")
    );
    if problems + warnings == 0 {
        format!("No findings. {checked}")
    } else {
        format!(
            "{}, {}. {checked}",
            count(problems, "problem", "problems"),
            count(warnings, "warning", "warnings")
        )
    }
}

/// `Checkout and Shipping use each other. 2 files in Checkout, 4 files in Shipping.`
pub fn circle(files: &[(String, usize)]) -> String {
    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    let head = match names.as_slice() {
        [a, b] => format!("{a} and {b} use each other"),
        names => format!("{} use each other in a circle", list_of(names)),
    };
    let counts: Vec<String> = files
        .iter()
        .map(|(name, n)| format!("{} in {name}", count(*n, "file", "files")))
        .collect();
    format!("{head}. {}.", counts.join(", "))
}

/// `Checkout reads Modules\Billing\Models\Invoice directly. Billing offers Contracts and Events.`
/// For a module with an accepted public surface: `... Billing offers its accepted public surface.`
pub fn reads_internals(
    module: &str,
    of: &str,
    classes: &[&str],
    public: &[String],
    accepted: bool,
) -> String {
    let offers = if accepted {
        "its accepted public surface".to_owned()
    } else {
        let public: Vec<&str> = public.iter().map(String::as_str).collect();
        list_of(&public)
    };
    format!(
        "{module} reads {} directly. {of} offers {offers}.",
        list_of(classes)
    )
}

pub fn invalid_manifest(module: &str, message: &str) -> String {
    format!("The module.json of {module} is not valid JSON: {message}.")
}

pub fn no_manifest_name(module: &str) -> String {
    format!("The module.json of {module} has no name. The package needs one.")
}

/// `Blog declares the namespace Modules\Blogs. Its folder is Modules/Blog.`
pub fn namespace_mismatch(module: &str, namespace: &str, folder: &str) -> String {
    format!("{module} declares the namespace {namespace}. Its folder is {folder}.")
}

/// `modules_statuses.json lists Legacy, which has no folder.`
pub fn stale_status(entry: &str, file: &str) -> String {
    format!("{file} lists {entry}, which has no folder.")
}

/// `Notifications is 39% covered. The limit is 50%.`
pub fn low_coverage(module: &str, percent: u8, limit: u8) -> String {
    format!("{module} is {percent}% covered. The limit is {limit}%.")
}

/// `Checkout lacks 2 keys and 1 file in fr.`
pub fn missing_translations(module: &str, locale: &str, keys: usize, files: usize) -> String {
    let what: Vec<String> = [(keys, "key", "keys"), (files, "file", "files")]
        .into_iter()
        .filter(|(n, _, _)| *n > 0)
        .map(|(n, one, many)| count(n, one, many))
        .collect();
    let what: Vec<&str> = what.iter().map(String::as_str).collect();
    format!("{module} lacks {} in {locale}.", list_of(&what))
}

fn count(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

fn list_of(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}
