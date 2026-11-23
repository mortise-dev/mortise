//! "Test" of how PHP runs, against the fake artisan of `mortise_core::testing`.
#![cfg(feature = "run")]

use std::path::Path;
use std::time::{Duration, Instant};

use mortise_core::{Failure, RunnerKind, TEST_LIMIT, TestOutcome, test_php};

/// A Custom runner whose first artisan argument is `case`, or `--version` with no case.
fn fake(case: &str) -> RunnerKind {
    RunnerKind::Custom {
        line: format!("\"{}\" {case}", env!("CARGO_BIN_EXE_fake-artisan")),
    }
}

fn test(runner: &RunnerKind, limit: Duration) -> TestOutcome {
    test_php(runner, Path::new(env!("CARGO_MANIFEST_DIR")), limit)
}

#[test]
fn answered_without_colour_codes() {
    assert_eq!(
        test(&fake(""), TEST_LIMIT),
        TestOutcome::Answered("Laravel Framework 13.2.0".into())
    );
}

#[test]
fn exited_with_the_line_of_stderr() {
    assert_eq!(
        test(&fake("fail"), TEST_LIMIT),
        TestOutcome::Failed(Failure::Exited {
            code: Some(1),
            line: r#"service "app" is not running"#.into(),
        })
    );
}

#[test]
fn not_found_and_no_command() {
    let missing = RunnerKind::Custom {
        line: "mortise-no-such-php artisan".into(),
    };
    assert_eq!(
        test(&missing, TEST_LIMIT),
        TestOutcome::Failed(Failure::NotFound {
            program: "mortise-no-such-php".into(),
        })
    );
    let empty = RunnerKind::Custom { line: " ".into() };
    assert_eq!(
        test(&empty, TEST_LIMIT),
        TestOutcome::Failed(Failure::NoCommand)
    );
}

#[test]
fn timed_out_at_the_limit() {
    let started = Instant::now();
    let outcome = test(&fake("wait"), Duration::from_millis(200));
    assert!(
        matches!(outcome, TestOutcome::Failed(Failure::TimedOut { .. })),
        "{outcome:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(10));
}
