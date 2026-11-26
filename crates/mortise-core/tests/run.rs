//! "Test" of how PHP runs, and runs of a command, against the fake artisan of
//! `mortise_core::testing`.
#![cfg(feature = "run")]

use std::path::Path;
use std::time::{Duration, Instant};

use mortise_core::{
    DataError, Failure, MigrationStatus, PhpSettings, Read, Run, RunEnd, RunEvent, RunnerKind,
    Runs, STOP_WAIT, StartError, TEST_LIMIT, TestOutcome, document, test_php,
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

    // A recording with a `root` prints the paths of the folder it runs in.
    let folder = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_fake-artisan"))
        .args(["module:make", "Loyalty", "--api", "--json"])
        .current_dir(folder.path())
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let path = json["files"][0]["path"].as_str().unwrap();
    let name = folder.path().file_name().unwrap().to_str().unwrap();
    assert!(
        path.contains(&format!("/{name}/Modules/Loyalty/")) && !path.contains('\\'),
        "{path}"
    );
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

/// Whether Windows lists a process with the id `pid`.
#[cfg(windows)]
fn listed(pid: &str) -> bool {
    let list = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    assert!(list.status.success(), "{list:?}");
    String::from_utf8_lossy(&list.stdout).contains(&format!("\"{pid}\""))
}

/// Asserts that the process `pid`, which the fake printed, is gone within 5 s.
#[cfg(windows)]
fn gone(pid: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while listed(pid.trim()) {
        assert!(Instant::now() < deadline, "the child {pid} still runs");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn a_run_streams_its_lines_and_stops() {
    let run = start("hold", &["module:make", "Blog"], TEST_LIMIT);
    first_line(&run);
    run.stop();
    run.stop();
    let end = stopped_within_the_wait(&run);
    assert!(
        matches!(end, RunEnd::Stopped { limit: None, .. }),
        "{end:?}"
    );
    run.stop();

    // The program ended and left a child that writes and ignores SIGTERM: a stop
    // while its lines are read ends the run as stopped, on Unix with the child.
    let run = start("", &["orphan-ticks"], TEST_LIMIT);
    let child = loop {
        let line = first_line(&run);
        if line.parse::<u32>().is_ok() {
            break line;
        }
    };
    std::thread::sleep(Duration::from_millis(500));
    run.stop();
    let end = stopped_within_the_wait(&run);
    assert!(
        matches!(end, RunEnd::Stopped { limit: None, .. }),
        "{end:?}"
    );
    #[cfg(unix)]
    gone(&child);
    #[cfg(not(unix))]
    let _ = child;
}

/// The end of `run`, which a wait of [`STOP_WAIT`] after its stop must give.
fn stopped_within_the_wait(run: &Run) -> RunEnd {
    let asked = Instant::now();
    let events = run.wait(asked + STOP_WAIT);
    assert!(asked.elapsed() < STOP_WAIT, "{events:?}");
    match events.last() {
        Some(RunEvent::Ended(end)) => *end,
        last => panic!("no end, the last event is {last:?}"),
    }
}

#[cfg(unix)]
#[test]
fn a_stop_kills_a_program_that_ignores_sigterm() {
    // The first line comes after the fake ignores SIGTERM.
    let run = start("ignore-term hold", &["module:make", "Blog"], TEST_LIMIT);
    first_line(&run);
    let asked = Instant::now();
    run.stop();
    let end = stopped_within_the_wait(&run);
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
        // On Windows `taskkill /T` walks the tree from the program, which still runs.
        gone(&child);
    }
}

/// A batch file, as `herd.bat` is: std starts it through `cmd.exe`, refuses an
/// argument it cannot escape (S52), and a stop ends the program under it (S53).
#[cfg(windows)]
#[test]
fn a_batch_file_refuses_a_line_break_and_stops_with_its_tree() {
    let folder = std::env::temp_dir().join(format!("mortise-batch-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let batch = folder.join("herd.bat");
    let fake = env!("CARGO_BIN_EXE_fake-artisan");
    std::fs::write(&batch, format!("@\"{fake}\" %*\r\n")).unwrap();
    let php = PhpSettings {
        runner: Some(RunnerKind::Custom {
            line: format!("\"{}\"", batch.display()),
        }),
        ..PhpSettings::default()
    };

    let refused = Runs::new().start(app(), &php, &words(&["a\nb"]), TEST_LIMIT);
    let Err(StartError::Failed(Failure::NotStarted { message, .. })) = refused else {
        panic!("{refused:?}");
    };
    assert_eq!(message, "batch file arguments are invalid");

    // `cmd.exe` runs the fake, which starts a child, prints its id and waits.
    let run = Runs::new()
        .start(app(), &php, &words(&["orphan-wait"]), TEST_LIMIT)
        .unwrap();
    let child = first_line(&run);
    assert!(listed(child.trim()), "{child}");
    run.stop();
    let end = stopped_within_the_wait(&run);
    assert!(
        matches!(end, RunEnd::Stopped { limit: None, .. }),
        "{end:?}"
    );
    gone(&child);
    let _ = std::fs::remove_dir_all(&folder);
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

    // Input given to the run is the answer.
    let run = Runs::new()
        .start_with_input(
            app(),
            &php(""),
            &words(&["ask"]),
            TEST_LIMIT,
            Some(b"Billing\n".to_vec()),
        )
        .unwrap();
    let (_, end) = until_end(&run, Duration::from_secs(2));
    assert!(
        matches!(end, RunEnd::Exited { code: Some(0), .. }),
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

#[test]
fn a_read_keeps_its_streams_apart_and_holds_the_app() {
    // The fake ends only once the gate file exists.
    let desk = tempfile::tempdir().unwrap();
    let gate = desk.path().join("gate");
    let runs = Runs::new();
    let mut read = Read::start(
        &runs,
        app(),
        &php(&format!("gate \"{}\"", gate.display())),
        &words(&["module:migrate-status", "Checkout", "--json"]),
    )
    .unwrap();
    // A read and a run share the lock of the app.
    assert!(matches!(
        runs.start(app(), &php(""), &words(&["wait"]), TEST_LIMIT),
        Err(StartError::Busy { .. })
    ));
    std::fs::write(&gate, "").unwrap();
    let end = read
        .wait(Instant::now() + Duration::from_secs(10))
        .expect("the read ended");
    assert!(matches!(end.end, RunEnd::Exited { code: Some(0), .. }));
    assert!(end.stderr.is_empty());
    let status: MigrationStatus = end.document().unwrap();
    assert_eq!(status.modules[0].name, "Checkout");

    // A read that fails says why on its last line of standard error.
    let mut read = Read::start(&runs, app(), &php(""), &words(&["fail"])).unwrap();
    let end = read
        .wait(Instant::now() + Duration::from_secs(10))
        .expect("the read ended");
    assert!(end.stdout.is_empty());
    assert_eq!(
        end.error_line().as_deref(),
        Some(r#"service "app" is not running"#)
    );

    // A line cut at 64 KiB is not read, though its start parses as a document.
    let mut read = Read::start(&runs, app(), &php(""), &words(&["long"])).unwrap();
    let end = read
        .wait(Instant::now() + Duration::from_secs(10))
        .expect("the read ended");
    assert!(document::<serde_json::Value>(&end.stdout).is_ok());
    assert_eq!(end.document::<serde_json::Value>(), Err(DataError::Cut));
}
