//! "Test" of how PHP runs, and runs of a command, against the fake artisan of
//! `mortise_core::testing`.
#![cfg(feature = "run")]

use std::path::Path;
use std::time::{Duration, Instant};

use mortise_core::{
    Failure, PhpSettings, Run, RunEnd, RunEvent, RunnerKind, Runs, StartError, TEST_LIMIT,
    TestOutcome, test_php,
};

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
fn answered_with_a_bounded_line_when_flooded() {
    let started = Instant::now();
    let outcome = test(&fake("flood"), TEST_LIMIT);
    let TestOutcome::Answered(line) = outcome else {
        panic!("{outcome:?}");
    };
    assert!(
        line.len() <= 64 * 1024 && line.starts_with('x'),
        "{}",
        line.len()
    );
    assert!(started.elapsed() < Duration::from_secs(10));
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
fn not_found_no_command_and_no_folder() {
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
    let gone = Path::new(env!("CARGO_MANIFEST_DIR")).join("no-such-folder");
    assert_eq!(
        test_php(&fake(""), &gone, TEST_LIMIT),
        TestOutcome::Failed(Failure::NoFolder { folder: gone })
    );
}

#[test]
fn timed_out_at_the_limit() {
    // `orphan-wait` ends in time only when its child died too and the pipes closed.
    for case in ["wait", "orphan-wait"] {
        let started = Instant::now();
        let outcome = test(&fake(case), Duration::from_millis(200));
        assert!(
            matches!(outcome, TestOutcome::Failed(Failure::TimedOut { .. })),
            "{case}: {outcome:?}"
        );
        assert!(started.elapsed() < Duration::from_secs(2), "{case}");
    }
}

#[test]
fn the_fake_replays_a_recording() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_fake-artisan"))
        .args(["module:make", "--no-interaction", "--ansi", "Blog"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(!output.stdout.is_empty());
}

/// The settings of an app whose runner is the fake with the words `before`.
fn php(before: &str) -> PhpSettings {
    PhpSettings {
        runner: Some(fake(before)),
        ..PhpSettings::default()
    }
}

fn app() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn words(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| word.to_string()).collect()
}

/// Starts `args` in the app through the fake with the words `before`.
fn start(before: &str, args: &[&str], limit: Duration) -> Run {
    Runs::new()
        .start(app(), &php(before), &words(args), limit)
        .unwrap()
}

/// The events of `run` up to its end, which must come `within` that time.
fn until_end(run: &Run, within: Duration) -> (Vec<RunEvent>, RunEnd) {
    let deadline = Instant::now() + within;
    let mut events = Vec::new();
    loop {
        for event in run.events() {
            if let RunEvent::Ended(end) = event {
                return (events, end);
            }
            events.push(event);
        }
        assert!(Instant::now() < deadline, "no end in {within:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Waits for the first line of `run`, which must come within 10 s, and gives it.
fn first_line(run: &Run) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        for event in run.events() {
            if let RunEvent::Line { text, .. } = event {
                return text;
            }
        }
        assert!(Instant::now() < deadline, "no line");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Asserts that the process `pid`, which the fake printed, is gone within 2 s.
#[cfg(unix)]
fn gone(pid: &str) {
    use rustix::process::{Pid, test_kill_process};
    let pid = Pid::from_raw(pid.trim().parse().unwrap()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while test_kill_process(pid).is_ok() {
        assert!(Instant::now() < deadline, "the child {pid:?} still runs");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_run_streams_its_lines_and_stops() {
    let run = start("hold", &["module:make", "Blog"], TEST_LIMIT);
    first_line(&run);
    run.stop();
    run.stop();
    let (_, end) = until_end(&run, Duration::from_secs(5));
    assert!(
        matches!(end, RunEnd::Stopped { limit: None, .. }),
        "{end:?}"
    );
    run.stop();
}

#[cfg(unix)]
#[test]
fn a_stop_kills_a_program_that_ignores_sigterm() {
    // The first line comes after the fake ignores SIGTERM.
    let run = start("ignore-term hold", &["module:make", "Blog"], TEST_LIMIT);
    first_line(&run);
    let asked = Instant::now();
    run.stop();
    let (_, end) = until_end(&run, Duration::from_secs(6));
    assert!(matches!(end, RunEnd::Stopped { .. }), "{end:?}");
    assert!(asked.elapsed() >= Duration::from_millis(2500));
}

#[test]
fn a_run_stops_at_its_limit_with_its_tree() {
    let limit = Duration::from_millis(300);
    let run = start("", &["wait"], limit);
    let (_, end) = until_end(&run, Duration::from_secs(5));
    assert!(
        matches!(end, RunEnd::Stopped { limit: Some(at), .. } if at == limit),
        "{end:?}"
    );
    // The child of the program ends with it, also one that ignores SIGTERM and
    // closed its pipes.
    for case in ["orphan-wait", "orphan-quiet"] {
        let run = start("", &[case], Duration::from_secs(2));
        let child = first_line(&run);
        let (_, end) = until_end(&run, Duration::from_secs(10));
        assert!(matches!(end, RunEnd::Stopped { .. }), "{case}: {end:?}");
        #[cfg(unix)]
        gone(&child);
        #[cfg(not(unix))]
        let _ = child;
    }
}

#[cfg(unix)]
#[test]
fn a_run_ends_soon_after_a_program_that_leaves_a_child() {
    let run = start("", &["orphan"], TEST_LIMIT);
    let child = first_line(&run);
    // The readers would be left behind after 1.25 s; an end before means the child died.
    let (_, end) = until_end(&run, Duration::from_secs(1));
    assert!(
        matches!(end, RunEnd::Exited { code: Some(0), .. }),
        "{end:?}"
    );
    gone(&child);
}

#[test]
fn a_question_ends_with_its_input() {
    let run = start("", &["ask"], TEST_LIMIT);
    let (_, end) = until_end(&run, Duration::from_secs(2));
    assert!(
        matches!(end, RunEnd::Exited { code: Some(1), .. }),
        "{end:?}"
    );
}

#[test]
fn a_flood_is_cut_once() {
    let run = start("", &["flood"], TEST_LIMIT);
    let (events, end) = until_end(&run, Duration::from_secs(20));
    let cuts = events.iter().filter(|event| **event == RunEvent::Cut);
    assert_eq!(cuts.count(), 1);
    assert_eq!(events.last(), Some(&RunEvent::Cut));
    assert!(
        matches!(end, RunEnd::Exited { code: Some(0), .. }),
        "{end:?}"
    );
}

#[test]
fn one_run_per_app() {
    let runs = Runs::new();
    let other = app().join("src");
    let first = runs
        .start(app(), &php(""), &words(&["wait"]), TEST_LIMIT)
        .unwrap();
    assert_eq!(runs.running(app()).as_deref(), Some("wait"));
    let busy = runs.start(
        app(),
        &php(""),
        &words(&["module:make", "Blog"]),
        TEST_LIMIT,
    );
    assert_eq!(
        busy.unwrap_err(),
        StartError::Busy {
            command: "wait".into()
        }
    );
    let second = runs
        .start(&other, &php(""), &words(&["wait"]), TEST_LIMIT)
        .unwrap();
    for run in [&first, &second] {
        run.stop();
        until_end(run, Duration::from_secs(5));
    }
    assert_eq!(runs.running(app()), None);
}
