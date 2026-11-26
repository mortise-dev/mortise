//! Interface text of the Findings list (section 10 of the specification).

use mortise_core::AppSnapshot;

pub const COL_FINDING: &str = "Finding";
pub const COL_CHECK: &str = "Check";
pub const COL_ENTER_OPENS: &str = "Enter opens";
pub const PROBLEMS: &str = "Problems";
pub const WARNINGS: &str = "Warnings";
pub const CHECKS_WITHOUT_A_FINDING: &str = "Checks without a finding";
pub const TODO_NOT_COUNTED: &str = "To do, not counted as findings";

/// `Checkout has 1 migration waiting.`: a row of the To do group of Findings.
pub fn migrations_waiting(module: &str, n: usize) -> String {
    if n == 1 {
        format!("{module} has 1 migration waiting.")
    } else {
        format!("{module} has {n} migrations waiting.")
    }
}

/// `4 of 7 checks`
pub fn of_checks(quiet: usize, total: usize) -> String {
    format!("{quiet} of {total} checks")
}

/// `No findings. 7 modules checked against 2 rules and 4 checks.`
pub fn no_findings(modules_count: usize, rules: usize, checks: usize) -> String {
    let rules = if rules == 1 {
        "1 rule".to_owned()
    } else {
        format!("{rules} rules")
    };
    let checks = if checks == 1 {
        "1 check".to_owned()
    } else {
        format!("{checks} checks")
    };
    format!(
        "No findings. {} checked against {rules} and {checks}.",
        crate::strings::modules(modules_count)
    )
}

/// The name of a check in the Findings list: `Rule: no circular use`.
pub fn check_name(check: mortise_core::Check, app: &AppSnapshot) -> String {
    use mortise_core::Check;
    match check {
        Check::NoCircularUse => "Rule: no circular use".to_owned(),
        Check::PublicSurfaceOnly => format!(
            "Rule: {} only",
            crate::strings::list_of(&app.rules.public_folders)
        ),
        // Not in the spec: the drawing has findings of the first two rules.
        Check::AllowedUse => "Rule: allowed use".to_owned(),
        Check::CoverageLimit => "Coverage limit".to_owned(),
        Check::TranslationsComplete => "Translations".to_owned(),
        // Not in the spec: the drawing has no finding of these three.
        Check::ValidManifest => "Valid module.json".to_owned(),
        Check::NamespaceMatchesFolder => "Namespace".to_owned(),
        Check::NoStaleStatus => "Statuses file".to_owned(),
    }
}

/// A check that found nothing, as a sentence: `Every module.json is valid JSON`.
pub fn check_passed(check: mortise_core::Check, app: &AppSnapshot) -> String {
    use mortise_core::Check;
    match check {
        // Not in the spec: the drawing has findings for both rules.
        Check::NoCircularUse => crate::strings::RULE_NO_CIRCULAR.to_owned(),
        Check::PublicSurfaceOnly => crate::strings::RULE_PUBLIC_SURFACE.to_owned(),
        Check::AllowedUse => crate::strings::RULE_ALLOWED_USE.to_owned(),
        Check::ValidManifest => "Every module.json is valid JSON".to_owned(),
        Check::NamespaceMatchesFolder => "Declared namespace matches the folder".to_owned(),
        Check::NoStaleStatus => {
            format!(
                "No entry in {} without a module",
                app.path_shown(&app.layout.statuses_file)
            )
        }
        // Not in the spec.
        Check::CoverageLimit => format!("Every module is covered {}% or more", app.coverage_limit),
        Check::TranslationsComplete => "Translations are complete".to_owned(),
    }
}

/// The command a check reads from, after its sentence: `module:lang`.
pub fn check_source(check: mortise_core::Check) -> Option<&'static str> {
    match check {
        mortise_core::Check::TranslationsComplete => Some("module:lang"),
        _ => None,
    }
}

/// What a check looked at: `7 modules`, `7 entries`.
pub fn checked(result: &mortise_core::CheckResult) -> String {
    match result.check {
        mortise_core::Check::NoStaleStatus if result.checked == 1 => "1 entry".to_owned(),
        mortise_core::Check::NoStaleStatus => format!("{} entries", result.checked),
        // `5 modules with lang files`
        mortise_core::Check::TranslationsComplete => {
            format!(
                "{} with lang files",
                crate::strings::modules(result.checked)
            )
        }
        _ => crate::strings::modules(result.checked),
    }
}
