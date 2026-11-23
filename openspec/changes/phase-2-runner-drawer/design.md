# Design

## Context

See `proposal.md` for why. Milestone 10 (`openspec/changes/archive/2026-11-23-phase-2-how-php-runs/design.md`)
decided the process model of this milestone in its D10. This design builds it, and changes it only
where a decision below says so and why.

Today `run/process.rs` has `command()` and `capture()`. `capture` runs a program to its end, reads both
pipes on two threads into one channel, and at its limit calls `Child::kill` on the launcher only. It
never joins its reader threads. The fake artisan (`src/bin/fake-artisan.rs`, `testing::fake_artisan_main`)
knows `--version`, `fail`, `flood` and `wait`.

Constraints: `unsafe_code = "forbid"`; `mortise-core` has no interface dependency; Rust 1.99.0;
macOS, Windows and Linux.

Marks: (crate) read in the crate source of `Cargo.lock` or the registry on this Mac. (std) read in the
std source of the toolchain. (assumed) not run; a task checks it or the report says "not run".

## Goals / Non-Goals

**Goals**: a run streams its lines, stops its whole process tree, ends at its limit, and leaves no
thread behind. One run per app. Output in four styles. A history of 50 runs per app. The drawer of
S11.1 a to g with its keys. Recorded package output for the fake artisan.

**Non-Goals**: the actions and generators that start runs (milestones 12 and 14). The sentence of
S11.1 c that names a migration, and its "Open migration file" and "Dry run" steps (milestone 12 reads
the output). The failure toast of C13 (milestone 12). The real-PHP CI job (milestone 16).

## Decisions

### D1. Public surface (feature `run`)

```rust
// run/process.rs
pub enum Stream { Out, Err }
pub struct Runs { /* Arc<Mutex<BTreeMap<PathBuf, String>>>: app folder to the running label */ }
impl Runs {
    pub fn new() -> Self;                                    // also Default
    /// Starts `args` through the runner of `php` in `php.folder(app)`, with `php.time_limit()`.
    pub fn start(&self, app: &Path, php: &PhpSettings, args: &[String]) -> Result<Run, StartError>;
    /// The label of the command that runs in `app`, if one does.
    pub fn running(&self, app: &Path) -> Option<String>;
}
pub enum StartError {
    Busy { command: String },        // the label of the running command
    NoRunner,                        // `php.runner` is None
    Failed(Failure),                 // NoCommand, NoFolder, NotFound, NotStarted of milestone 10
}
pub struct Run {
    pub line: CommandLine,           // what runs; `display()` is the `$` line
    pub label: String,               // the artisan words, without --no-interaction and --ansi
    pub args: Vec<String>,           // as given to `start`, for history and Run again
    pub started_at: SystemTime,      // for history
    /* started: Instant, events: Receiver<RunEvent>, stop: Arc<AtomicBool> */
}
impl Run {
    pub fn stop(&self);              // safe to call twice or after the end
    pub fn elapsed(&self) -> Duration;
    pub fn events(&self) -> Vec<RunEvent>;   // drains what arrived, never blocks
}
pub enum RunEvent { Line { stream: Stream, text: String }, Cut, Ended(RunEnd) }
pub enum RunEnd {
    Exited { code: Option<i32>, elapsed: Duration },     // code None: ended by a signal Mortise did not send
    Stopped { elapsed: Duration, limit: Option<Duration> }, // limit Some: the time limit stopped it
}
pub const OUTPUT_LIMIT: usize = 1_000_000;   // bytes of text kept per run, newlines counted
```

`label` is the `args` joined with one space, each word quoted by the quoting of `CommandLine::display`
for the platform. It is the text of `Busy`, the header of the drawer and the history row.

`Failure` stays the type of milestone 10. `StartError::Failed` uses only its four start variants. The
interface writes each as the text in quotes of "The command did not start" with the functions that
already write `Failure` in `strings.rs`. `NoRunner` reads "no way to run PHP is chosen" (not in the
spec).

- Alternative: `start` takes a `CommandLine`. Rejected: history and Run again need the artisan words
  and the current runner, and `Busy` needs the label. `start` builds the line, so the flags of D7 of
  milestone 10 are always there.

### D2. Three threads per run, and no thread left behind (carried-over finding 2)

`start` prepares the command as `capture` does (the private `prepare` of milestone 10), adds
`process_group(0)` on Unix (std, safe), spawns it, and starts:

- two reader threads, the `read_lines` loop of milestone 10, one per pipe, sending into one channel;
- one owner thread that holds the `Child`.

The owner loops every 50 ms: it forwards lines from the readers to the run's channel, checks the stop
flag and the limit, and calls `try_wait`. It keeps a count of forwarded bytes; past `OUTPUT_LIMIT` it
sends `Cut` once and drops later lines, while the readers keep reading so the program never blocks.
The per-line cut of 64 KiB stays. The 64 KiB per stream of `capture` stays a `capture` rule.

The end, in this order:

1. The leader has exited (by itself or after a stop).
2. The owner waits up to 250 ms for both readers to finish (`JoinHandle::is_finished`).
3. Still open: on Unix it sends SIGKILL to the group, which ends any process left in it, and the pipes
   close. On Windows see D4.
4. It joins both readers and forwards their last lines.
5. It removes the app from `Runs`, then sends `Ended`. So when the interface sees `Ended`, a new start
   in that app is accepted, and no thread of the run is alive.

A process that left the group (`setsid`) can keep a pipe open past step 3. The owner then waits at most
1 s more, detaches the readers, and ends. Known limit, said in a comment.

`capture` becomes a caller of the same internal process type: spawn, read, stop the tree at the limit
(D3), join the readers. So the test of milestone 10 fixes carried-over finding 1 too.

### D3. Stop and the time limit (carried-over finding 1)

As D10 of milestone 10:

- **macOS, Linux**: `rustix::process::kill_process_group(Pid::from_child(&child), Signal::TERM)`
  (crate: `rustix` 1.1.6 `src/process/kill.rs`, safe). Wait up to 3 s for the leader. Then
  `Signal::KILL` to the group. `rustix` becomes a direct dependency, `cfg(unix)`, with the feature
  `process`, behind the feature `run`.
- **Windows**: `taskkill /PID <pid> /T /F` through `process::command`, waited for, then `Child::kill`
  when `taskkill` fails. `/T` ends the children, so `php.exe` under `herd.bat` ends too.
- The time limit stops the same way and ends as `Stopped { limit: Some(_) }`. The interface adds the
  line "Stopped at the time limit of 10 minutes." (not in the spec).

An error of `kill_process_group` with `ESRCH` means the group is gone; that is not an error.

### D4. Windows and a child that outlives a finished command

`taskkill /T` walks the tree from a living process. Once the leader has exited by itself, Windows has
no way to name its children without a job object, and a job object needs `unsafe` or a new crate that
wraps it. So on Windows step 3 of D2 does nothing: the readers end when the child closes the pipe, and
the owner detaches them after 1 s, as for `setsid`. Stop and the time limit end the tree while the
leader still lives, so carried-over finding 1 is fixed on Windows. This is a known limit, recorded in
the report, not in `docs/deviations.md`: the spec says nothing about it.

- Alternative: the `win32job` crate. Rejected for now: it pulls the `windows` crate and a licence row
  for one case that artisan commands do not reach. It can be added later without a change of the API.

### D5. One run per app

`Runs::start` takes the lock, refuses with `Busy` when the app is in the map, inserts the label, then
spawns. A spawn error removes the entry before it returns. The owner removes it at the end (D2 step 5).
The key is the app folder as given, the folder of `AppEntry`. The test of milestone 10 does not take
the lock.

### D6. ANSI (`run/ansi.rs`)

```rust
pub enum Style { Text, Dim, Success, Error }
pub struct Span { pub style: Style, pub text: String }
pub fn spans(line: &str) -> Vec<Span>;   // a style carries over from one span to the next
pub fn plain(line: &str) -> String;      // the text without any escape sequence
```

`plain` moves here from `run/mod.rs`, and `test_php` calls it.

SGR (`ESC [ n ; n m`), read left to right; the last code that sets a style wins:

| Codes | Style |
|---|---|
| 31, 91, 41, 101 | Error |
| 32, 92, 42, 102 | Success |
| 2, 33, 93, 43, 103, 90, 37 when faint | Dim |
| 0, empty, 22, 39, 49 | back to Text |
| any other, and `38;5;n`, `38;2;r;g;b` and their `48` forms (with their arguments skipped) | no change |

Yellow is dim because S11.1 a draws `RUNNING`, which Laravel writes in yellow, as dim. 256-colour and
true-colour codes keep the style; their arguments are not read as codes. Every other escape sequence is
removed: CSI that does not end in `m`, OSC (`ESC ]` up to BEL or `ESC \`, so hyperlinks keep their
text), and a lone `ESC` with the next byte. A `\r` inside a line keeps only the text after the last one.
Spans of the same style next to each other are merged; empty spans are dropped.

### D7. History (`run/history.rs`)

```rust
pub const HISTORY_LIMIT: usize = 50;
pub struct History { /* the folder of one app */ }
#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub id: u64,                 // the start time in ms since 1970; one more when taken
    pub args: Vec<String>,
    pub label: String,
    pub started_ms: i64,
    pub ended: Ended,
    pub cut: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Ended { Done { ms: u64 }, Failed { code: Option<i32>, ms: u64 }, Stopped { ms: u64, limit: bool } }
impl History {
    pub fn open(data_dir: &Path, app: &Path) -> Self;
    pub fn entries(&self) -> Vec<Entry>;                 // the latest first; missing or invalid: empty
    pub fn output(&self, id: u64) -> Vec<String>;        // the lines as written, with their codes
    pub fn record(&self, entry: Entry, lines: &[String]) -> std::io::Result<()>;
}
pub fn data_dir() -> Option<PathBuf>;   // MORTISE_DATA_DIR, else the data folder of ProjectDirs "Mortise"
```

Layout: `<data_dir>/runs/<key>/runs.json` (the entries, the latest first, and the app folder for a
reader) and `<key>/<id>.log` (the lines, `\n` after each). `<key>` is 16 hex digits of FNV-1a 64 of the
app folder's path as a lossy string: stable across releases, no new crate. `record` writes the log,
then `runs.json` through a temporary file in the same folder and a rename, as settings do. It keeps
`HISTORY_LIMIT` entries and removes the logs of those it drops, and any `.log` no entry names. It keeps
at most `OUTPUT_LIMIT` bytes of lines and sets `cut` when it drops some. A run that did not start is
not recorded.

`data_dir()` follows `MORTISE_CONFIG_DIR` of settings: a new `MORTISE_DATA_DIR` for work on Mortise
itself; `AGENTS.md` names it. The app gets `Config::data_dir: Option<PathBuf>`; `None` keeps no history,
as `settings_path: None` keeps no settings. The tests give a temp folder.

The app records the run when it sees `Ended`, with the raw lines it kept. The core does not record by
itself: the interface owns what a "run" in the history is (milestone 12 may label "tests of Checkout").

### D8. The fake artisan and the recordings

Recordings: one JSON file per command in `fixtures/output/`, named after the command
(`module-make.json`, `module-make-model.json`, ...):

```json
{
  "command": "php artisan module:make Blog --no-interaction --ansi",
  "args": ["module:make", "Blog"],
  "made_with": "nwidart/laravel-modules 13.3.0, laravel/framework 13.x.y, PHP 8.4.z, macOS",
  "exit": 0,
  "stdout": "…with \u001b codes…",
  "stderr": ""
}
```

JSON keeps every byte, the escape codes included, and Git and Windows line endings cannot change it.
`args` are the artisan words without `--no-interaction` and `--ansi`; the paths in the output are
those of the scratch app and stay as recorded. The set covers at least `module:make`, one
`module:make-*` generator, a generator that finds an existing file, `module:migrate`, `module:seed`, a
failing command and `--help`. The app is made in a scratch folder outside the repository and is not
kept.

The fake reads the arguments after its own program. First it takes any of these words, in any order:

- `hold`: after the output, wait 60 s before it exits.
- `ignore-term`: on Unix, ignore SIGTERM. It replaces itself, through the safe
  `CommandExt::exec` (std), with `/bin/sh -c "trap '' TERM; exec \"$0\" \"$@\"" <itself> <rest>`;
  an ignored signal stays ignored across `exec`. On Windows the word does nothing. This is the only
  place a shell runs, in a test helper, with no user text.

Then the first remaining word picks the case. `--version`, `fail`, `flood` and `wait` stay as in
milestone 10. New:

- `ask`: prints `Name of the new module?` on stdout, reads one line from stdin. At the end of input it
  prints `Aborted.` on stderr and exits 1.
- `orphan`: starts a copy of itself with `wait` that inherits both pipes, prints its pid, and exits 0.
- `orphan-wait`: as `orphan`, then waits 60 s itself.
- any other word: the recording in `fixtures/output/` whose `args` equal the remaining words without
  `--no-interaction` and `--ansi`. It prints `stdout`, then `stderr`, and exits with `exit`. No match:
  `Command "<word>" is not defined.` on stderr, exit 1.

The folder is `concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/output")`, read at run time, so a
new recording needs no code change. A test starts it with a Custom runner: `"<fake>" hold` gives the
words before the artisan arguments.

### D9. The drawer (`runner_view.rs`)

State, in the app:

- `RunnerView` (one for the window): `shown: Shown` (`Never`, `Collapsed`, `Open`), `height: f32`
  (200, or 160 under 700 px of window height, in memory only), `history_open: bool`,
  `selected: usize`.
- `AppRuns` (one per open app): the current `Run` if any, its `lines: Vec<String>` and their spans,
  the last ended run with its `RunEnd` or the `StartError` of the last start, and the history entries,
  read once when the app opens and again after each `record`.

`Never` draws nothing: the read-only build of the architecture table ("Runner drawer: Hidden") and an
app that has not run anything keep all the room. A start, Ctrl `, Cmd J or Ctrl J, or the palette
shows it. From then on Ctrl ` toggles `Open` and `Collapsed`; Hide collapses.

Layout: the drawer takes the bottom of the workspace column. The handle is the top 7 px (C15). Open
height is clamped to `[DRAWER_H_MIN, max(DRAWER_H_MIN, (workspace height) / 2)]`. Collapsed is
`DRAWER_COLLAPSED_H`. Colours are the `code_*` tokens in both themes.

The bar (S11.1): "Runner" in `code_fg`, then the label in mono, elided to the left part that fits.
Right side by state, each a ghost button with its keycap:

| State | Right side |
|---|---|
| a. Running | run icon and "Running, 2.4 s" (one decimal, seconds), Stop `Ctrl .`, Hide ``Ctrl ` `` |
| b. Done | good mark and "Done in 186 ms" (ms under 1 s, else one decimal), Run again, Copy output, History, Hide |
| c. Failed | bad mark and "Exit code 1", "412 ms", Run again, Copy output, History, Hide |
| Stopped | "Stopped after 3.1 s", Run again, Copy output, History, Hide |
| d. History | "5 runs today in shop-api", Run again `Enter`, Copy output, History (pressed), Hide |
| e. Collapsed | the status of the last run, Show ``Ctrl ` `` |
| f. Empty | Hide; the body reads the empty text |

On macOS the keycaps read `Cmd .` and keep ``Ctrl ` ``, as section 7 says.

Body: the first line `$ ` and `run.line.display()` in dim, then each line as spans in `code_fg`,
`code_dim`, `code_ok`, `code_err`, in a scroll area that sticks to the bottom while the run goes on.
A cut adds "Output past 1 MB is not kept." in dim (not in the spec). A failure adds a rule and the
sentence "<label> stopped with exit code <n>." (not in the spec; the sentence of S11.1 c that names the
migration comes with milestone 12), then the step "Copy output". A run that did not start shows the
text of S11.1 for it and the button "Open settings", which opens Settings on the PHP section with that
app selected. A stop through Sail or Docker adds "PHP in the container may still run." (not in the
spec).

History (d): a list of 420 px on the left, the latest first, each row the clock time (`HH:MM`, local
time through `jiff`), the label and the result mark with "186 ms" or "Exit code 1". The right shows
`History::output` of the selected row as spans. Up and Down move, Enter runs the selected `args` again
through the current runner of the app. "5 runs today" counts entries whose local date is today.

Keys, in the key handler of `app.rs`, where they are not typing:

| Key | macOS | Windows, Linux |
|---|---|---|
| Show or hide | Ctrl `, Cmd J | Ctrl `, Ctrl J |
| Stop | Cmd . | Ctrl . |
| Run the last command again | Cmd Shift R | Ctrl Shift R |
| Show the history | Cmd Shift H | Ctrl Shift H |
| Copy the output | Cmd Shift O | Ctrl Shift O |
| On the focused handle | Up, Down 24 px; Home, End; double click collapses | same |

F6 adds the drawer as the fourth region after the workspace when it is open.

Stop asks first when the first artisan word is `module:migrate`, `module:migrate-rollback`,
`module:migrate-refresh`, `module:migrate-reset`, `module:migrate-fresh` or `module:seed`: the dialog
of S10.3, "Stop <label>?", "A migration that stops half way can leave the database between two
states.", "Keep running" (focused) and "Stop command". A pure function in the core,
`stop_asks_first(args: &[String]) -> bool`, holds the list; milestone 12 extends it with the rest of
the confirmation table.

While a run goes on the app asks for a repaint every 100 ms, so lines and the elapsed time show without
a callback from the core. When a run ends the drawer announces "<label>: Done in 186 ms" or the exit
code through a live region (reference: "announces the end of a command, not every line").

While an app runs a command, Run again for that app is off with the tooltip "<label> is running".
Milestone 12 uses `Runs::running` for its buttons the same way.

### D10. The header line and the `$` line

S11.1 draws the header as `php artisan module:migrate --all` and the `$` line the same. The header shows
the label only, as S11.1 g does at every width; the `$` line shows the line that runs, with the
runner's prefix and `--no-interaction --ansi`. Deviation S36 records it, as S32 did for `-T`.

### D11. Tests

Core, `tests/run.rs` (the existing file, which reaches the fake through `CARGO_BIN_EXE_fake-artisan`):

- a run streams a line of a `hold` recording before it ends, then is stopped: `Stopped { limit: None }`;
- `ignore-term` on Unix: stopped after about 3 s, under 6 s;
- the limit: `wait` with a limit of 300 ms ends as `Stopped { limit: Some(_) }`; the same with
  `orphan-wait` ends within 2 s, which holds only when the orphan died and the pipes closed;
- `orphan` on Unix: `Ended` within 1 s of the exit;
- `ask` ends with exit code 1 in under 2 s;
- `flood` sends `Cut` once and ends with code 0;
- `Busy` names the label; another app starts at the same time;
- `test_php` at its limit with `orphan-wait` returns `TimedOut` within 2 s (finding 1 through `capture`).

Unit tests: the ANSI table of D6 in `ansi.rs`; the history limit, the damaged file and the cut in
`history.rs` with a temp folder; `stop_asks_first`.

Interface, `tests/runner.rs` with `egui_kittest` against the fake: run (Run again from a seeded history,
output and "Done in"), stop (`hold`, Ctrl ., "Stopped after"), fail (a failing recording, "Exit code 1"
and the sentence), history (Up, Down, Enter), did not start (missing program, "Open settings" opens the
PHP section). Cargo sets `CARGO_BIN_EXE_<name>` only for the tests of the package that owns the
binary. So `mortise-app` gets the same three-line binary as the core: `[[bin]] fake-artisan` with
`required-features = ["testing"]`, a feature `testing = ["mortise-core/testing"]`, and a dev-dependency
on itself with `testing`, as `mortise-core` has. A release build leaves the feature off, so the binary
is not built, and cargo-packager names its one binary (`binaries = [{ path = "mortise", main = true }]`).
Rejected: a nested `cargo build` from the tests (a second target folder and a second build of the
core), and a binary found from `current_exe()` (not built by `cargo test -p mortise-app`).

Images: S11.1 a to g in light and dark, with the drawer cut out of the window. The data comes from a
seeded history and the fake, with the elapsed time of a cut out of the running image.

## Risks / Trade-offs

- [Lines of stdout and stderr can swap places] → They are read on two threads. If a recording shows it
  in the drawer, deviation S37 records it (plan section 7).
- [Stop through Sail or Docker reaches `docker` only] → The drawer says so (D9).
- [A process that leaves the group, or a child on Windows after a finished command] → The owner
  detaches the readers after 1 s (D2, D4). Known limit.
- [Closing Mortise while a command runs] → The command goes on; nothing stops it. Known limit, said in
  the report.
- [Group kill after the leader exited] → The group id could in theory be reused within 250 ms. The
  kill is sent only while a reader still holds a pipe, which means a member of the group still lives.
- [The fake reaches `fixtures/` through `CARGO_MANIFEST_DIR`] → Tests only; the binary is not part of a
  build without `testing`.
- [`jiff` is a new dependency of `mortise-app`] → It gives the local time zone on the three platforms
  without `unsafe` in Mortise. `std` has no local time. Licence MIT or Unlicense; the script lists it.

## Migration Plan

New files only in Mortise's data folder. No change to `settings.json`. Rollback is a revert; the
`runs` folder is then ignored.

Order of work: the ANSI parser, the history and the recordings, the fake artisan, the process model
(needs the fake for its tests), then the drawer.

## Open Questions

- Should the drawer height survive a restart? Now it does not. Saving it adds one optional field to
  `general` in `settings.json`.
- Should closing Mortise stop a running command? Now it does not.
