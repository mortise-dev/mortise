//! Interface text of the Summary tab (section 10 of the specification).

use mortise_core::{AppSnapshot, LayoutKey};

pub const MODULES_ON: &str = "Modules on";
pub const MODULE_STATE: &str = "Module state";
/// Not in the spec: the state of a module when the app does not use the file activator.
pub const STATE_UNKNOWN: &str = "Unknown";

/// `7 of 7`
pub fn of(on: usize, total: usize) -> String {
    format!("{on} of {total}")
}

/// Not in the spec: a note for settings of `config/modules.php` that need PHP to evaluate.
pub fn assumed(app: &AppSnapshot, name: &str) -> Option<String> {
    let keys: Vec<&str> = app
        .layout
        .assumed
        .iter()
        .map(|key| key.config_key())
        .collect();
    let listed = match keys[..] {
        [] => return None,
        [one] => one.to_owned(),
        [ref rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    };
    let verb = if keys.len() == 1 { "needs" } else { "need" };
    let follows = if app.layout.assumed.contains(&LayoutKey::ModulesFolder) {
        format!(
            " If the modules of {name} are not in {}, this list is wrong.",
            app.modules_dir_shown()
        )
    } else {
        String::new()
    };
    Some(format!(
        "{listed} in config/modules.php {verb} PHP to work out. Mortise uses the default of \
         the package instead.{follows}"
    ))
}

/// Not in the spec: why the on or off state is unknown.
pub fn state_not_in_a_file(app: &AppSnapshot, name: &str) -> String {
    format!(
        "{name} keeps the state of its modules with the activator \"{}\", not in a file. \
         Mortise reads files only, so it cannot tell which modules are on.",
        app.layout.activator
    )
}

pub const LINES_OF_PHP: &str = "Lines of PHP";
pub const TEST_COVERAGE: &str = "Test coverage";
/// The word for a coverage that cannot be worked out: there is no report.
pub const UNKNOWN: &str = "unknown";
pub const COVERAGE_BY_MODULE: &str = "Coverage by module";
pub const BY_MODULE: &str = "By module";
pub const FILES_BY_KIND: &str = "Files by kind";
pub const GIT_CHURN: &str = "Git churn";
pub const COL_LINES: &str = "Lines";
pub const COL_TESTS: &str = "Tests";
pub const COL_COVERAGE: &str = "Coverage";
pub const COL_COMMITS: &str = "Commits, 30 days";

/// `54%`
pub fn percent(percent: u8) -> String {
    format!("{percent}%")
}

/// `sorted by name`
pub fn sorted_by(column: &str, ascending: bool) -> String {
    let by = if column == crate::strings::COL_MODULE {
        "name".to_owned()
    } else {
        column.to_lowercase()
    };
    if ascending {
        format!("sorted by {by}")
    } else {
        format!("sorted by {by}, highest first")
    }
}

/// `14 commits in 30 days, last 2 days ago`
pub fn churn(commits: u32, last: Option<i64>, now: i64) -> String {
    let count = match commits {
        0 => "no commits in 30 days".to_owned(),
        1 => "1 commit in 30 days".to_owned(),
        n => format!("{n} commits in 30 days"),
    };
    match last {
        Some(last) => format!("{count}, last {}", crate::strings::ago(now, last)),
        // Not in the spec: a module that no commit ever touched.
        None => "never committed".to_owned(),
    }
}

/// `From build/coverage/clover.xml, written 2 days ago`
pub fn coverage_from(path: &str, written: Option<i64>, now: i64) -> String {
    match written {
        Some(written) => format!("From {path}, written {}", crate::strings::ago(now, written)),
        None => format!("From {path}"),
    }
}

pub const DEPENDENCIES: &str = "Dependencies";
pub const USED_BY: &str = "Used by";
pub const USES: &str = "Uses";
pub const COL_USED_BY: &str = "Used by";

/// `4 out, 2 in`
pub fn out_in(out: usize, into: usize) -> String {
    format!("{out} out, {into} in")
}
pub const COL_FINDINGS: &str = "Findings";
pub const LOW_COVERAGE: &str = "Low coverage";
pub const WARNING: &str = "Warning";

/// `Show all 3 findings`
pub fn show_all_findings(n: usize) -> String {
    if n == 1 {
        "Show the finding".to_owned()
    } else {
        format!("Show all {n} findings")
    }
}

/// The class without the namespace all modules share: `Billing\Models\Invoice`.
fn short_class<'a>(class: &'a str, app: &AppSnapshot) -> &'a str {
    class
        .strip_prefix(app.layout.namespace.as_str())
        .and_then(|rest| rest.strip_prefix('\\'))
        .unwrap_or(class)
}

/// The findings of one module as clauses of one sentence. `own` is true on the
/// module's own Summary: `2 files here, 4 files there` and the full class name.
fn clauses(module: &str, app: &AppSnapshot, name: &str, own: bool) -> Vec<String> {
    use mortise_core::Detail;
    app.findings
        .of(module)
        .map(|found| match &found.detail {
            Detail::CircularUse { files: in_each } => {
                let total: usize = in_each.iter().map(|(_, n)| n).sum();
                let others: Vec<String> = in_each
                    .iter()
                    .filter(|(name, _)| name != module)
                    .map(|(name, _)| name.clone())
                    .collect();
                let here = in_each
                    .iter()
                    .find(|(name, _)| name == module)
                    .map_or(0, |(_, n)| *n);
                match (own, others.as_slice()) {
                    (true, [other]) => format!(
                        "{module} and {other} use each other ({} here, {} there)",
                        crate::strings::files(here as u64),
                        crate::strings::files((total - here) as u64)
                    ),
                    (false, [other]) => {
                        format!(
                            "it and {other} use each other ({})",
                            crate::strings::files(total as u64)
                        )
                    }
                    (_, others) => format!(
                        "it is in a circle with {} ({})",
                        crate::strings::list_of(others),
                        crate::strings::files(total as u64)
                    ),
                }
            }
            Detail::ReadsInternals {
                of,
                files: n,
                classes,
            } => {
                let what = match classes.as_slice() {
                    [class] if own => class.clone(),
                    [class] => short_class(class, app).to_owned(),
                    classes => format!("{} classes of {of}", classes.len()),
                };
                let verb = if *n == 1 { "reads" } else { "read" };
                if own {
                    format!(
                        "{} {verb} {what} directly",
                        crate::strings::files(*n as u64)
                    )
                } else {
                    format!("{n} of its files {verb} {what} directly")
                }
            }
            // Not in the spec: the sentence without the module in front.
            Detail::MissingTranslations {
                locale,
                keys,
                files,
                from,
            } => crate::strings::missing_translations(locale, *keys, *files, from.as_deref()),
            _ => {
                let sentence = crate::strings::finding(found, app, name);
                let mut letters = sentence.trim_end_matches('.').chars();
                // A clause in the middle of a sentence starts small, unless it starts with a path.
                match letters.next() {
                    Some(first) if !sentence.starts_with(module) && !sentence.contains('/') => {
                        first.to_lowercase().chain(letters).collect()
                    }
                    _ => sentence.trim_end_matches('.').to_owned(),
                }
            }
        })
        .collect()
}

fn sentence_of(clauses: &[String]) -> String {
    let joined = match clauses {
        [] => return String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    };
    let mut letters = joined.chars();
    let first: String = letters
        .next()
        .map(|first| first.to_uppercase().collect())
        .unwrap_or_default();
    format!("{first}{}.", letters.as_str())
}

/// The alert at the top of the Summary of "All modules", with its severity: the
/// module with the most problems, in bold, and what is wrong with it.
/// `Checkout` and ` has 2 problems. It and Shipping use each other (6 files), and 3 of its
/// files read Billing\Models\Invoice directly.`
/// Without a module that has a problem, the first finding, with its own severity.
pub fn app_alert(
    app: &AppSnapshot,
    name: &str,
) -> Option<(mortise_core::Severity, String, String)> {
    use mortise_core::{FindingState, Severity};
    let worst = app
        .modules
        .iter()
        .filter_map(|module| match app.findings.state_of(&module.name) {
            FindingState::Problems(n) => Some((n, &module.name)),
            _ => None,
        })
        .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(a.1)));
    let Some((count, module)) = worst else {
        let first = app.findings.list.first()?;
        return Some((
            first.severity,
            String::new(),
            crate::strings::finding(first, app, name),
        ));
    };
    let rest = format!(
        " has {}. {}",
        crate::strings::problems(count),
        sentence_of(&clauses(module, app, name, false))
    );
    Some((Severity::Problem, module.clone(), rest))
}

/// The alert at the top of the Summary of one module, with its severity.
/// `2 problems.` and ` Checkout and Shipping use each other (2 files here, 4 files there), and
/// 3 files read Modules\Billing\Models\Invoice directly.`
pub fn module_alert(
    app: &AppSnapshot,
    name: &str,
    module: &str,
) -> Option<(mortise_core::Severity, String, String)> {
    use mortise_core::{FindingState, Severity};
    match app.findings.state_of(module) {
        FindingState::Problems(n) => {
            let head = format!("{}.", crate::strings::problems(n));
            let rest = format!(" {}", sentence_of(&clauses(module, app, name, true)));
            Some((Severity::Problem, crate::strings::capitalised(&head), rest))
        }
        FindingState::Warning => {
            let sentences: Vec<String> = app
                .findings
                .of(module)
                .map(|found| crate::strings::finding(found, app, name))
                .collect();
            Some((Severity::Warning, String::new(), sentences.join(" ")))
        }
        FindingState::Good => None,
    }
}

/// Not in the spec: the line in the Summary of a module that is in a circle another
/// module carries.
pub fn part_of_circle_counted_on(module: &str) -> String {
    format!("Part of a circle counted on {module}")
}
pub const ROUTES_BY_MODULE: &str = "Routes by module";

/// The short form, in the place of routes or scheduled tasks.
pub fn endpoint_not_exposed(endpoint: &str, environment: &str) -> String {
    format!(
        "{endpoint} is not exposed on {environment}. Add it to exposure.include in \
         config/actuator.php."
    )
}

/// `5 tasks, read from local`
pub fn tasks_read_from(n: usize, environment: &str) -> String {
    match n {
        1 => format!("1 task, read from {environment}"),
        n => format!("{n} tasks, read from {environment}"),
    }
}

/// `1 task`
pub fn tasks_count(n: usize) -> String {
    match n {
        1 => "1 task".to_owned(),
        n => format!("{n} tasks"),
    }
}

/// `18 (12 API, 6 web)`
pub fn routes_of_module(total: usize, api: usize, web: usize) -> String {
    let mut parts = Vec::new();
    if api > 0 {
        parts.push(format!("{api} API"));
    }
    if web > 0 {
        parts.push(format!("{web} web"));
    }
    let other = total - api - web;
    if other > 0 && !parts.is_empty() {
        parts.push(format!("{other} other"));
    }
    if parts.is_empty() {
        total.to_string()
    } else {
        format!("{total} ({})", parts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn clauses_make_one_sentence() {
        assert_eq!(sentence_of(&[]), "");
        assert_eq!(
            sentence_of(&["it and Shipping use each other (6 files)".into()]),
            "It and Shipping use each other (6 files)."
        );
        assert_eq!(
            sentence_of(&[
                "it and Shipping use each other (6 files)".into(),
                "3 of its files read X directly".into()
            ]),
            "It and Shipping use each other (6 files), and 3 of its files read X directly."
        );
    }

    #[test]
    fn churn_says_how_many_and_when() {
        let day = 86_400;
        assert_eq!(
            churn(14, Some(100 * day - 2 * day), 100 * day),
            "14 commits in 30 days, last 2 days ago"
        );
        assert_eq!(
            churn(1, Some(100 * day - 3_600), 100 * day),
            "1 commit in 30 days, last 1 hour ago"
        );
        assert_eq!(
            churn(0, Some(10 * day), 100 * day),
            "no commits in 30 days, last 3 months ago"
        );
        assert_eq!(churn(0, None, 100 * day), "never committed");
    }

    #[test]
    fn a_problem_of_the_app_itself_is_alerted_as_a_problem() {
        use mortise_core::{Check, Detail, Finding, Severity};
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shop-api");
        let mut app = mortise_core::read_app(&fixture).unwrap();
        // No module has a finding; the statuses file lists a module that has no folder.
        app.findings.list = vec![Finding {
            severity: Severity::Problem,
            check: Check::NoStaleStatus,
            module: None,
            detail: Detail::StaleStatus {
                entry: "Legacy".to_owned(),
                file: "modules_statuses.json".to_owned(),
            },
        }];
        assert_eq!(
            app_alert(&app, &app.name),
            Some((
                Severity::Problem,
                String::new(),
                "modules_statuses.json lists Legacy, which has no folder.".to_owned()
            ))
        );
    }
}
