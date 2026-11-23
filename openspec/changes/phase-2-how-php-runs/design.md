# Design

## Context

See `proposal.md` for why. This design covers milestone 10 of `docs/plan/phase-2.md` and fixes the
shape of milestone 11 (the runner drawer), so milestone 10 is not redone.

Today the core starts two programs: `git` (`git.rs`) and the editor (`editor.rs`). Each sets up its own
`Command`. Only `git.rs` hides the console window on Windows. `settings.json` keeps nothing about PHP.

Constraints:

- `unsafe_code = "forbid"` in the workspace. No route to a signal or a job object may need `unsafe`.
- `mortise-core` has no interface dependency.
- The toolchain is Rust 1.99.0 (`rust-toolchain.toml`).
- macOS, Windows and Linux.

Marks: (std) read in the std source of the 1.99.0 toolchain on this Mac. (crate) read in the crate
source or `Cargo.lock`. (assumed) from the tool's documentation as remembered, not run; a task checks
it or reports it as not run. Herd is not installed here. Nothing was run on Windows or Linux.

## Goals / Non-Goals

**Goals**: one place builds a command line as a list of words, never through a shell. Old
`settings.json` files load. Detection and the test need no Herd, Docker, PHP or real home folder. The
process model of milestone 11 is decided, with the safe route to stop a program.

**Non-Goals**: streaming output, history, ANSI styles and the drawer (milestone 11). Any artisan
command but `--version`. A YAML parser in the core. Sail on Windows through WSL.

## Decisions

### D1. Module tree and what is public

```
crates/mortise-core/src/run/
  mod.rs      RunnerKind, PhpSettings (always compiled)
              CommandLine, quoting, test(), TestOutcome, Failure (feature `run`)
  process.rs  command() (always compiled); capture() (feature `run`)
  detect.rs   detect(), herd_config_dir() (feature `run`)
  ansi.rs     milestone 11
  history.rs  milestone 11
crates/mortise-core/src/bin/fake-artisan.rs   feature `testing` (D12)
```

Public from `mortise_core` in milestone 10: `RunnerKind`, `PhpSettings` (no feature); `CommandLine`,
`detect`, `herd_config_dir`, `test_php`, `TestOutcome`, `Failure`, `TEST_LIMIT` (feature `run`).
Crate-private: `process::command`, `process::capture`, the quoting functions, `editor::split_words`,
the Herd file readers.

Milestone 11 adds, and milestone 10 does not: `Run`, `RunState`, `Runs` (the registry with the lock of
D10), `RunEvent`, `ansi::Span`, `History`. `process::capture` is written as a loop over a channel of
lines (D9), so milestone 11 lifts the loop into `Run` and keeps `capture` as a caller of it.

- Alternative: all of `run` behind the feature. Rejected: `settings.rs` is always compiled and holds the
  runner. A build without `run` would drop the PHP fields when it saves `settings.json`.

### D2. The `run` feature

`run = []` is a new default feature. It adds no dependency in milestone 10. `mod run;` in `lib.rs` has
no `cfg`. Inside it, the items of D1 marked `run` carry `#[cfg(feature = "run")]`.
`git.rs` and `editor.rs` call `run::process::command`, which is always compiled.
`cargo build -p mortise-core --no-default-features` must build without a warning; a task runs it.

### D3. `RunnerKind` and its form in `settings.json`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RunnerKind {
    Herd,
    Sail,
    Docker { service: String },
    Custom { line: String },
}
```

JSON: `{"kind":"herd"}`, `{"kind":"docker","service":"app"}`, `{"kind":"custom","line":"php artisan"}`.
This is the form of `Credentials` in `environment.rs`.

`RunnerKind::prefix(&self) -> Vec<String>` gives the words before the artisan arguments:

| Kind | macOS, Linux | Windows |
|---|---|---|
| Herd | `herd php artisan` | `herd.bat php artisan` |
| Sail | `./vendor/bin/sail artisan` | the same words; see D6 |
| Docker | `docker compose exec -T <service> php artisan` | the same |
| Custom | the words of the user's line | the same |

Custom uses the splitter of `editor_arguments`. That code moves into `editor::split_words(&str)`
(`pub(crate)`), and `editor_arguments` calls it. A part in double quotes is one word. A backslash is an
ordinary letter, so a Windows path needs no escape. An empty Custom line gives no words; D8 turns that
into a failure.

- Alternative: the `shlex` crate for Custom. Rejected: POSIX rules eat the backslashes of a Windows
  path, and the editor command already taught users one rule.

### D4. Herd on Windows is `herd.bat`

The program name is `herd.bat`, with its extension.

- (std) `resolve_exe` in `sys/process/windows.rs` adds `.exe` to a name without an extension. So
  `Command::new("herd")` never finds `herd.bat`.
- (std) A program that ends in `.bat` or `.cmd` is started as
  `cmd.exe /e:ON /v:OFF /d /c ""<script>" <args>"`. `make_bat_command_line` in `sys/args/windows.rs`
  escapes each argument for `cmd.exe`. It refuses `\r` and `\n` and anything it cannot escape with
  `InvalidInput`, "batch file arguments are invalid".
- (assumed) Herd for Windows installs `herd.bat` in `%USERPROFILE%\.config\herd\bin`.

Safety consequence: through Herd on Windows, the arguments pass `cmd.exe`. The guard is the escaping of
std, in place since Rust 1.77.2 (CVE-2024-24576). Mortise adds no escaping of its own and never builds
the `cmd` line itself. A refused argument shows as "did not start" with the message of std.
Milestone 14 also limits the letters of a name before it reaches a command line.

- Alternative: `cmd /c herd ...` built by Mortise. Rejected: Mortise would own the escaping.
- Alternative: find `php.exe` of Herd and call it. Rejected: it skips the PHP version Herd isolates per
  site, which is the point of `herd php`.

### D5. Docker passes `-T`

The prefix is `docker compose exec -T <service> php artisan`. Mortise gives the program no terminal.
Without `-T`, `exec` asks for one and can fail with "the input device is not a TTY" (assumed; behaviour
differs between Compose versions). `--ansi` keeps the colours. The spec shows the line without `-T`:
add a row to `docs/deviations.md`. The line on screen shows `-T`, because it is the line that runs.

Sail needs no flag: its script adds `-T` itself when standard input is not a terminal (assumed).

### D6. Sail on Windows

Sail is a bash script and runs in WSL only. Mortise on Windows does not detect Sail and disables the
Sail choice, with a tooltip (text not in the spec). A `settings.json` that already holds Sail keeps it;
the test then fails with the message of Windows. The route for a WSL user is Custom.

- Alternative: the prefix `wsl ./vendor/bin/sail artisan`. Rejected for now: nothing here can run it,
  and a wrong guess looks like a feature. It can be added later without a change of data.

### D7. `CommandLine`

```rust
pub struct CommandLine { pub program: String, pub args: Vec<String> }

impl CommandLine {
    /// `None` when the runner gives no program (an empty Custom line).
    pub fn artisan(runner: &RunnerKind, arguments: &[&str]) -> Option<Self>;
    pub fn display(&self) -> String;           // quoted for this platform
}
```

`artisan` joins the prefix and the arguments, and puts `--no-interaction --ansi` right after the first
artisan argument: `herd php artisan module:make --no-interaction --ansi Blog`, and
`herd php artisan --version --no-interaction --ansi`. With no argument the two flags come alone.
A flag that the prefix, or the arguments before a `--`, already hold is not added again.

- Why not at the end: an argument list can hold `--`. Words after `--` are not options, so flags at the
  end would be lost. Symfony Console reads options anywhere before `--`.

`display` quotes each word with one of two crate-private functions. Both are compiled and tested on
every platform; `display` picks by `cfg!(windows)`.

A word is written bare when it is not empty and holds only ASCII letters, digits and `_ - . / : = +`
(and `\` on Windows). Any other word is quoted:

- **macOS, Linux (POSIX `sh`)**: in single quotes. A `'` inside becomes `'\''`. An empty word is `''`.
  A program that holds `=` is quoted too: `sh` reads `a=b cmd` as an assignment.
- **Windows (PowerShell)**: in single quotes. A `'` inside becomes `''`. An empty word is `''`. When the
  program itself is quoted, the line starts with `& `.

Why PowerShell and not `cmd`: PowerShell is the default shell of Windows Terminal. A single-quoted
PowerShell string expands nothing. `cmd` has no quoting that stops `%NAME%`. Known limit, said in a
comment: Windows PowerShell 5.1 drops an empty argument and mangles a `"` inside an argument to a
program. PowerShell 7.3 and later pass both, except to a `.bat` or `.cmd`, which still gets its
arguments the legacy way. So an empty argument can vanish when the copied Herd line is pasted. The line
is for reading and copying. The process always gets the list.

Tests, one table per quoting function, in `run/mod.rs`:

| Word | POSIX | PowerShell |
|---|---|---|
| `module:make`, `--name=Blog` | bare | bare |
| `My Blog` | `'My Blog'` | `'My Blog'` |
| `it's` | `'it'\''s'` | `'it''s'` |
| `say "hi"` | `'say "hi"'` | `'say "hi"'` |
| (empty) | `''` | `''` |
| `$HOME` | `'$HOME'` | `'$HOME'` |
| `a\b` | `'a\b'` | `a\b` |
| `C:\Program Files\php\php.exe` as program | `'C:\Program Files\php\php.exe'` | `& 'C:\Program Files\php\php.exe'` |
| `Füße` | `'Füße'` | `'Füße'` |
| `a,b`, `@x`, `%PATH%`, `a;b`, `a*` | quoted | quoted |
| `bin/a=b` as program | `'bin/a=b'` | bare |

One more test: `--no-interaction --ansi` sit after the first argument, also when the list holds `--`.

### D8. Settings

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PhpSettings {
    /// `None`: not chosen yet. Detection proposes one; nothing runs.
    pub runner: Option<RunnerKind>,
    /// `None`: the app folder. A relative path starts at the app folder.
    pub working_folder: Option<PathBuf>,
    /// Minutes before a command is stopped. 10 by default (S8.2).
    pub time_limit_minutes: u16,
}
```

`AppEntry` gets one field: `#[serde(default)] pub php: PhpSettings`. An old file loads with
`runner: None`, the app folder and 10 minutes. `PhpSettings` has `folder(&self, root) -> PathBuf` and
`time_limit(&self) -> Duration`, which reads 0 as 1 minute. The Settings field accepts 1 to 120.

`runner` is an `Option`, not a default kind.

- Alternative: default `Herd`. Rejected: Mortise would hold a choice the user never made, and on Linux
  one that cannot work. `None` also tells "read only" (`b.addro`, `tip.nophp`) from "chosen".
- Alternative: three flat fields on `AppEntry`. Rejected: nine places build an `AppEntry`; one field
  with `Default` touches each once.

The last test result is not saved. The app keeps it in memory per app. After a restart the "Last test"
column of S8.2 shows "Not tested" (text not in the spec). Mortise does not run the test by itself,
because a command runs only when the user starts it.

Interface rule that follows from the data: the command field of S1.3 and S8.2 can be edited for every
kind. The kind follows the text. `RunnerKind::from_line(&str) -> RunnerKind` splits the text with
`split_words` and compares the words with the prefixes of D3. The Herd or Sail words give that kind.
`docker compose exec [-T] <service> php artisan` gives Docker with that service, which is how S8.2 says
the service is edited ("The service name is part of the command and is edited there"). Any other text
gives Custom. Picking a kind in the segmented control writes its prefix into the field. No Service field
is added, so this needs no deviation.

### D9. Starting a program: `run/process.rs`

```rust
/// A command with no standard input and, on Windows, no console window.
pub(crate) fn command(program: impl AsRef<OsStr>) -> Command;

#[cfg(feature = "run")]
pub(crate) struct Captured { pub code: Option<i32>, pub stdout: Vec<String>, pub stderr: Vec<String>, pub timed_out: bool }

#[cfg(feature = "run")]
pub(crate) fn capture(line: &CommandLine, folder: &Path, limit: Duration) -> std::io::Result<Captured>;
```

`command` sets `stdin(Stdio::null())` and, on Windows, `creation_flags(0x0800_0000)`
(`CREATE_NO_WINDOW`; `CommandExt::creation_flags` is safe std). `git.rs` drops its own copy.
`editor.rs` uses it too, so an editor that is a console program opens no window on Windows (not run).

`capture` and, later, a run prepare the command the same way, in one private function:

1. A relative program that holds a separator (`/`, or `\` on Windows) is joined to `folder`. std says
   a relative program with `current_dir` is platform specific, so Mortise makes it absolute. The line
   on screen keeps the word as typed. A bare name stays a lookup in `PATH`.
   On Windows a `.bat` or `.cmd` whose path holds `%` is refused with `InvalidInput`, because `cmd.exe`
   would expand it. It shows as "did not start".
2. `current_dir(folder)`.
3. `PATH` of the child is the `PATH` of Mortise plus the folders below, added at the end. std looks the
   program up in the child's `PATH`: on Windows in `resolve_exe` (std), on Unix by the documentation
   of `Command::new` (not read in the source). A desktop app on macOS starts with
   `/usr/bin:/bin:/usr/sbin:/sbin`, so `herd` and `docker` are not found without this.

   Added (assumed). macOS: `~/Library/Application Support/Herd/bin`, `/opt/homebrew/bin`,
   `/usr/local/bin`, `~/.docker/bin`. Linux: `/usr/local/bin`, `~/.local/bin`.
   Windows: `%USERPROFILE%\.config\herd\bin`.
   - Alternative: ask a login shell for its `PATH`. Rejected: it runs the user's shell files, and the
     risk list of the plan says no shell.
4. stdout and stderr are piped.

`capture` then starts two threads. Each reads one pipe by line (`BufRead::read_until(b'\n')`, lossy
UTF-8, `\r` trimmed) and sends `(Stream, String)` over one `mpsc` channel. A line is cut at 64 KiB,
and a stream keeps at most 64 KiB. The rest is read and dropped, so the program never blocks. The caller loops:
`recv_timeout(50 ms)`, then `try_wait`. It ends when the program has ended and both pipes closed, or
250 ms after the program ended (a grandchild can hold a pipe open), or at the limit. At the limit it
calls `Child::kill` and sets `timed_out`. It never joins a reader thread.

Milestone 10 needs no new dependency: `Child::kill` is enough for `--version`.

### D10. The process model of milestone 11 (decided now, built then)

- **Start**: the preparation of D9, plus `process_group(0)` on macOS and Linux (std, safe, stable since
  1.64). The program leads its own group, and its children join it.
- **Output**: the reader threads and the channel of D9. A third thread owns the `Child`, watches the
  limit and the stop flag, and sends `RunEvent::{Line, Ended}`. The interface drains the channel each
  frame. Lines of stdout and stderr can swap places (plan, section 7).
- **Stop on macOS and Linux**: SIGTERM to the group, wait up to 3 s, then SIGKILL to the group.
- **Stop on Windows**: `taskkill /PID <pid> /T /F` through `process::command`, then `Child::kill` if
  that fails. `/T` ends the children too. With Herd the child is `cmd.exe`, and `Child::kill` alone
  (`TerminateProcess`) leaves `php.exe` running. A job object needs `unsafe` calls. `taskkill` is in
  `System32` (assumed, not run).
- **Time limit**: `PhpSettings::time_limit`. At the limit the run stops the same way and ends as
  `Stopped`, with a line that names the limit.
- **Sail or Docker**: the signal reaches `docker` on this computer. The PHP process in the container
  may go on. The drawer says so.
- **One run at a time per app**: `Runs` in the core holds `Mutex<BTreeMap<PathBuf, Running>>`, keyed by
  the app folder. `Runs::start` refuses a second run with `Busy { command }`. The app owns one `Runs`.
  The test of milestone 10 does not take this lock; its button is off while the test runs.

The safe route to SIGTERM:

| Route | Cost | Verdict |
|---|---|---|
| `rustix` 1.1, feature `process` | Already in the tree at 1.1.6 through `tempfile` (crate). No new crate. `kill_process_group(Pid::from_child(&child), Signal::TERM)` is a safe function (crate). Apache-2.0 or MIT. Unix only, under `cfg(unix)` | **Chosen** |
| `nix` | A new crate with a safe `kill`. MIT. Adds a licence row and an audit surface for one call | Rejected |
| `/bin/kill -TERM -- -<pid>` | No dependency. One more process per stop, a fixed path, and an exit code instead of an error value | Rejected; the fallback if `rustix` ever changes |
| `libc::kill` | `unsafe` | Not allowed |

`rustix` becomes a direct, optional dependency of the feature `run` in milestone 11, for `cfg(unix)`
only. `licenses/THIRD-PARTY.md` already lists it; run the script again anyway.

Known limit: when Mortise itself is killed, a running command goes on. Recorded in milestone 11.

### D11. Detection: `run/detect.rs`

```rust
/// The runners the folder points to, the best first. Reads files only.
pub fn detect(root: &Path, herd_config: Option<&Path>) -> Vec<RunnerKind>;
/// The valet folder of Herd on this platform. `None` on Linux.
pub fn herd_config_dir() -> Option<PathBuf>;
```

The app calls `detect(root, herd_config_dir().as_deref())`. A test passes a folder in a temp dir. No
test reads the home folder.

| Runner | Found when | Checked |
|---|---|---|
| Herd | `root` is a site of Herd (below) | assumed |
| Sail | `vendor/bin/sail` exists and a compose file exists. Never on Windows (D6) | read in `laravel/sail` as remembered |
| Docker | a compose file exists and Sail is not found. The service is `app` | spec S8.2 |

Compose files: `compose.yaml`, `compose.yml`, `docker-compose.yaml`, `docker-compose.yml`.

Order: Herd, then Sail, then Docker. The first is proposed. The hint names it (S1.3: "Found: this folder
is a Herd site."). An empty list gives "Not found: Herd, Sail, a compose file." and `runner` stays `None`.

- Why Herd first: a Herd site is a choice made on this computer. A compose file in a Herd site is often
  only a database.
- Why Sail needs a compose file: the Laravel skeleton requires `laravel/sail` for development, so
  `vendor/bin/sail` is in most apps that never use it. `sail:install` writes the compose file.
- Why the service is always `app`: the core has no YAML parser, and a line scan breaks on flow style
  and anchors. The service in the command field can be edited, and the test names a wrong service in seconds
  (`service "app" is not running`). A parser can come later without a change of data.

Herd site. The valet folder (assumed, from the Herd and Valet documentation; not run) is
`~/Library/Application Support/Herd/config/valet` on macOS and `%USERPROFILE%\.config\herd\config\valet`
on Windows. Herd does not exist on Linux.

`root` is a site when one of these holds, after `std::fs::canonicalize` of both sides:

- **Parked**: the parent of `root` is one of the `paths` in `config.json` (read with `serde_json`).
- **Linked**: an entry in `Sites/` is a link whose target is `root`.

A file that is missing or not valid means "not a Herd site". Detection never fails and never starts a
program. Tests: a parked folder, a link (a symlink; on Windows a link needs a right the CI may lack, so
that case is `cfg(unix)`), Sail with and without a compose file, each compose file name, the order, an
empty folder, a broken `config.json`.

### D12. The test (10.7)

```rust
pub const TEST_LIMIT: Duration = Duration::from_secs(20);
pub fn test_php(runner: &RunnerKind, folder: &Path, limit: Duration) -> TestOutcome;

pub enum TestOutcome { Answered(String), Failed(Failure) }
pub enum Failure {
    NoCommand,                                   // an empty Custom line
    NoFolder { folder: PathBuf },                // the working folder is not a folder
    NotFound { program: String },
    NotStarted { program: String, message: String },
    Exited { code: Option<i32>, line: String },
    TimedOut { seconds: u64 },
}
```

`test_php` blocks. The app calls it on a worker thread and asks for a repaint when it returns, as the
actuator test does. It runs `CommandLine::artisan(runner, &["--version"])` through `capture`.

- Exit code 0: `Answered` with the first line of stdout that is not empty, without escape sequences.
  `--ansi` colours the version number, so a private `plain(&str)` drops `ESC [ ... letter` sequences.
  Milestone 11 moves it into `ansi.rs`.
- Any other end: `Exited` with the first line of stderr that is not empty, else of stdout, else empty.

`Failure` is data. The interface writes it, in `strings.rs`, as the text in quotes of `x.nophp`:
`herd: command not found` (`NotFound`, the spec's wording on every platform); `herd: ` and the system's
message (`NotStarted`, like `EditorError`); the line, or "exit code 1" when it is empty (`Exited`);
"no answer in 20 seconds" (`TimedOut`); "no command entered" (`NoCommand`); "the working folder does not exist" (`NoFolder`). The last four are not in
the spec. There is no `reason()` in the core: all interface text stays in `strings.rs`.

`NotFound` comes from a spawn error of kind `ErrorKind::NotFound`. The text of the operating system
differs per platform and language, so the interface does not show it for this case. Every other spawn
error keeps the system's text.

The sentence around it is `{Runner} did not answer: "{reason}". ...` (`x.nophp`) in S1.3, and
`No answer: {reason}` in the "Last test" column of S8.2. 20 seconds: a cold `docker compose exec` takes
a few seconds; a longer wait looks like a hang beside a button.

### D13. The fake artisan

Milestone 10 builds it, because it is how the test is checked in the core and in the app.

```toml
[[bin]]
name = "fake-artisan"
path = "src/bin/fake-artisan.rs"
required-features = ["testing"]
```

The binary is three lines: it calls `mortise_core::testing::fake_artisan_main()`. It looks at its first
argument only. `--version` prints `Laravel Framework ` and `13.2.0` in green SGR codes and exits 0.
`fail` prints `service "app" is not running` on stderr and exits 1. `wait` sleeps 60 s. `flood` prints
2 MiB of lines on stderr and a 2 MiB line with no newline on stdout, and exits 0. Anything else
prints `Command "<name>" is not defined.` on stderr and exits 1. A test picks the case with a word in
the Custom line: `"<path>" fail` makes `fail` the first argument.

How a test reaches it: Cargo sets `CARGO_BIN_EXE_fake-artisan` for the integration tests of
`mortise-core` (its dev-dependency on itself turns on `testing`). So the tests of `test_php` live in
`crates/mortise-core/tests/run.rs`. A test builds `Custom { line: format!("\"{}\" {case}", path) }`; the
double quotes keep a path with spaces as one word.

- Alternative: a helper that finds the binary from `current_exe()` for `mortise-app`. Rejected:
  `cargo test -p mortise-app` alone does not build the binary. The app needs no fake artisan: its one
  screen test of the wiring uses a Custom line with a program that does not exist, and shows the
  failure of S1.4 c. The answered state is drawn in the images from data.
- Alternative: a second binary in `mortise-app`. Rejected: cargo-packager could pick it up.
- Alternative: a shell script. Rejected by the plan: not the same on the three platforms.

Core tests in `tests/run.rs`: answered (with the colour codes gone), `Exited` with the stderr line,
`NotFound` for a program that is not there, `TimedOut` with a limit of 200 ms on `wait`, `NoCommand`.

## Risks / Trade-offs

- [The Herd folders and `herd.bat` are assumed] → Tasks check them on a Mac with Herd and on Windows,
  or the report says "not run". When detection finds nothing, the user picks a runner by hand.
- [`PATH` of a desktop app on macOS] → The added folders of D9. A tool elsewhere needs Custom with a
  full path; `command not found` points there.
- [`-T` differs from the spec] → One row in `docs/deviations.md`.
- [Arguments pass `cmd.exe` with Herd on Windows] → std escapes or refuses (D4).
- [The PowerShell line is wrong in PowerShell 5.1 for an empty word or a `"`] → A comment says so. The
  process is not affected.
- [Test kills only the launcher at its limit, and a descendant that holds the pipes keeps the reader
  threads alive] → Milestone 11 stops the process group (Unix) or the tree (`taskkill /T`, Windows) as
  D10 says, for the test too. Until then `--version` is the only command, and it ends by itself.

## Migration Plan

The `php` object in `settings.json` is optional, so an old file loads. An older Mortise that saves the
file drops `php` and nothing else. No app file changes. Rollback is a revert.

Order of work: `process::command` with `git.rs` and `editor.rs`; data and settings; `CommandLine`;
detection; the fake artisan and the test; the screens.

## Open Questions

- Should "Last test" of S8.2 survive a restart? Now it does not (D8). Saving it later adds one optional
  field and changes no approach.
- Should the Docker service come from the compose file? It needs a YAML parser or a line scan (D11).
  It can be added later; the data does not change.
- Should Sail run through WSL on Windows (D6)? It can be added later; the data does not change.
