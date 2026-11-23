//! Interface text of the runner drawer and the PHP setting of an app (section 10 of the specification).

pub const RUN_PHP_WITH: &str = "Run PHP with";
/// Not in the spec: the name a screen reader says for the Test of PHP. The label is
/// "Test", as for an environment, and the add panel shows both.
pub const TEST_PHP: &str = "Test how PHP runs";
/// Not in the spec: the words before the line that Test runs, under the field.
/// A command runs only after its exact line is on screen.
pub const TEST_RUNS: &str = "Test runs";
/// Not in the spec: the tooltip of Sail on Windows (design D6).
pub const SAIL_NOT_ON_WINDOWS: &str = "Sail runs in WSL, not on Windows itself";
pub const ANSWERED: &str = "Answered";
/// Not in the spec: the line under the field while the test runs.
pub const TESTING_PHP: &str = "Running artisan --version";
pub const FOUND_HERD: &str = "Found: this folder is a Herd site.";
/// Not in the spec, written like the Herd hint.
pub const FOUND_SAIL: &str = "Found: vendor/bin/sail and a compose file.";
/// Not in the spec, written like the Herd hint.
pub const FOUND_COMPOSE: &str = "Found: a compose file.";
pub const FOUND_NO_RUNNER: &str = "Not found: Herd, Sail, a compose file.";

/// The hint of "Run PHP with" after detection: what the folder points to.
pub fn php_found(found: Option<mortise_core::Detected>) -> &'static str {
    use mortise_core::Detected;
    match found {
        Some(Detected::Herd) => FOUND_HERD,
        Some(Detected::Sail) => FOUND_SAIL,
        Some(Detected::Docker) => FOUND_COMPOSE,
        None => FOUND_NO_RUNNER,
    }
}

/// Who did not answer, for `x.nophp`: Herd, Sail or Docker, and for Custom the
/// program of the line. Not in the spec: "The command" for a Custom line without one.
pub fn runner_name(runner: &mortise_core::RunnerKind) -> String {
    use mortise_core::RunnerKind;
    match runner {
        RunnerKind::Custom { .. } => runner
            .prefix()
            .into_iter()
            .next()
            .unwrap_or_else(|| "The command".to_owned()),
        other => crate::strings::RUNNERS[crate::strings::runner_place(other)].to_owned(),
    }
}

/// `Test runs herd php artisan --version --no-interaction --ansi`, as a screen
/// reader says it. Not in the spec.
pub fn test_runs(line: &str) -> String {
    format!("{TEST_RUNS} {line}")
}

/// `x.nophp`, under the field of the add panel and of Settings.
pub fn php_failed(runner: &mortise_core::RunnerKind, failure: &mortise_core::Failure) -> String {
    format!(
        "{} did not answer: \"{}\". You can add the app now and read it. Generating and \
         running need PHP: choose another way, or enter the command.",
        runner_name(runner),
        crate::strings::php_reason(failure)
    )
}

pub const RUNNER: &str = "Runner";
pub const STOP: &str = "Stop";
pub const HIDE: &str = "Hide";
pub const SHOW: &str = "Show";
pub const RUN_AGAIN: &str = "Run again";
pub const COPY_OUTPUT: &str = "Copy output";
pub const HISTORY: &str = "History";
pub const NOTHING_RAN: &str =
    "No command has run yet. Output of generators and actions appears here.";
/// Not in the spec: the name of the handle at the top of the drawer.
pub const DRAWER_HEIGHT: &str = "Height of the runner drawer";
/// Not in the spec: a run whose output passed the limit of the history.
pub const OUTPUT_CUT: &str = "Output past 1 MB is not kept.";
/// Not in the spec: a stop through Sail or Docker reaches `docker` only.
pub const CONTAINER_MAY_RUN: &str = "PHP in the container may still run.";
/// The stop dialog of S10.3.
pub const STOP_BODY: &str =
    "A migration that stops half way can leave the database between two states.";
pub const KEEP_RUNNING: &str = "Keep running";
pub const STOP_COMMAND: &str = "Stop command";

/// The keys of Stop: `Cmd .` on macOS, `Ctrl .` elsewhere.
pub fn stop_keys(mac: bool) -> String {
    format!("{} .", crate::strings::command_key(mac))
}

/// A time as the drawer writes it: `186 ms` under a second, else `2.4 s`.
pub fn run_time(elapsed: std::time::Duration) -> String {
    let ms = elapsed.as_millis();
    if ms < 1000 {
        format!("{ms} ms")
    } else {
        format!("{:.1} s", elapsed.as_secs_f64())
    }
}

/// `Running, 2.4 s`. Seconds with one decimal.
pub fn running_for(elapsed: std::time::Duration) -> String {
    format!("Running, {:.1} s", elapsed.as_secs_f64())
}

/// `Done in 186 ms`
pub fn done_in(elapsed: std::time::Duration) -> String {
    format!("Done in {}", run_time(elapsed))
}

/// `Exit code 1`. Not in the spec: a program a signal ended.
pub fn exit_code(code: Option<i32>) -> String {
    match code {
        Some(code) => format!("Exit code {code}"),
        None => "Ended by a signal".to_owned(),
    }
}

/// `Stopped after 3.1 s`
pub fn stopped_after(elapsed: std::time::Duration) -> String {
    format!("Stopped after {}", run_time(elapsed))
}

/// Not in the spec: the line under a run that the time limit stopped.
pub fn stopped_at_limit(limit: std::time::Duration) -> String {
    let minutes = (limit.as_secs() / 60).clamp(1, u64::from(u16::MAX)) as u16;
    format!(
        "Stopped at the time limit of {}.",
        crate::strings::minutes(minutes)
    )
}

/// What the live region says when a run ends: `module:migrate Blog: Done in 186 ms`.
pub fn run_ended(label: &str, status: &str) -> String {
    format!("{label}: {status}")
}

/// `Stop module:migrate Checkout?`
pub fn stop_title(label: &str) -> String {
    format!("Stop {label}?")
}

/// `5 runs today in shop-api`
pub fn runs_today(n: usize, app: &str) -> String {
    match n {
        1 => format!("1 run today in {app}"),
        n => format!("{n} runs today in {app}"),
    }
}

/// `The command did not start: "herd: command not found". Choose how PHP runs for shop-api in Settings.`
pub fn did_not_start(error: &mortise_core::StartError, app: &str) -> String {
    format!(
        "The command did not start: \"{}\". Choose how PHP runs for {app} in Settings.",
        crate::strings::start_reason(error)
    )
}

/// Not in the spec: the name of "Open settings" in the drawer, which opens the PHP
/// section, apart from the "Open settings" of the runtime box.
pub fn open_php_settings(app: &str) -> String {
    format!("Open settings, how PHP runs for {app}")
}

/// Not in the spec: the name of the step "Copy output" under a failure, apart from
/// the button of the bar.
pub fn copy_output_of(label: &str) -> String {
    format!("Copy output of {label}")
}

/// `module:migrate Checkout stopped with exit code 1 at 2026_11_20_...` (S11.1 c).
/// A program a signal ended keeps the sentence of [`stopped_with`].
pub fn failed_at(label: &str, code: Option<i32>, migration: &str) -> String {
    match code {
        Some(code) => format!(
            "{label} stopped with exit code {code} at {migration}. The error from the database \
             is quoted above. Fix the migration, or try a dry run to read the SQL."
        ),
        None => crate::strings::stopped_with(label, code),
    }
}

pub const OPEN_MIGRATION: &str = "Open migration file";

/// Not in the spec: Run again of a line Mortise asked about before it ran.
pub fn run_again_title(label: &str) -> String {
    format!("Run {label} again?")
}

/// Not in the spec.
pub const RUN_AGAIN_BODY: &str =
    "Mortise asked before this command ran last time. It runs the same line again.";

/// Not in the spec: the name of the step "Dry run" in the drawer, apart from the
/// "Dry run" of the Actions tab.
pub fn dry_run_of(label: &str) -> String {
    format!("Dry run of {label}")
}

/// The drawer, under a run the app prohibits (design D4).
pub const PROHIBITED: &str = "The app prohibits this command, so the package refused to run it. Its message is quoted above.";

#[cfg(test)]
mod runner_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn times_are_written_as_in_the_drawings() {
        assert_eq!(done_in(Duration::from_millis(186)), "Done in 186 ms");
        assert_eq!(
            stopped_after(Duration::from_millis(3140)),
            "Stopped after 3.1 s"
        );
        assert_eq!(running_for(Duration::from_millis(2420)), "Running, 2.4 s");
        assert_eq!(running_for(Duration::from_millis(300)), "Running, 0.3 s");
    }
}
