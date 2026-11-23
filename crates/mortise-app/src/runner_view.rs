//! The runner drawer (S11.1): the command that runs or ran last in the app, its
//! output with its colours, its status, and the history of the app's runs.
//!
//! The drawer is dark in both themes, like a terminal. It takes the bottom of the
//! workspace column. Nothing here starts a process: the app does, through
//! `mortise_core::Runs`, and hands the run to [`AppRuns`].

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::text::LayoutJob;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, Galley, Id, Key, Modifiers, Rect, Response,
    Role, ScrollArea, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::ansi::{self, Span, Style};
use mortise_core::history::{Ended, Entry, History};
use mortise_core::{
    AppSnapshot, CommandLine, FailedMigration, PhpSettings, Run, RunEnd, RunEvent, RunnerKind,
    Runs, StartError, Stream,
};

use crate::draw::{self, Icon, ListNav};
use crate::strings;
use crate::theme::{Face, Palette, Type, radius, size, stroke, ty};

/// The top edge of the drawer that sets its height (C15).
const HANDLE_H: f32 = 7.0;
/// What Up and Down on the handle change.
const HANDLE_STEP: f32 = 24.0;
/// The bar: 24 px, so each of its controls is a target of 24 px.
const BAR_H: f32 = 24.0;
/// The padding of the drawer: 8 px at the top, 16 px at the sides, 12 px at the bottom.
const PAD_TOP: f32 = 4.0;
const PAD_X: f32 = 16.0;
const PAD_BOTTOM: f32 = 12.0;
/// The list of the history (`.dhist`).
const HISTORY_LIST_W: f32 = 420.0;
const HISTORY_ROW_H: f32 = 24.0;
/// Under this width the bar keeps the status and Hide; the keys still reach the rest (S11.1 g).
const ROOMY_W: f32 = 900.0;
/// The fill of a hovered button and of the selected history row: rgba(147,160,176,.18).
const HOVER: Color32 = Color32::from_rgba_premultiplied(26, 29, 32, 46);
/// The rule above the sentence of a failure: rgba(147,160,176,.35).
const RULE: Color32 = Color32::from_rgba_premultiplied(51, 56, 62, 89);

/// The bar text: sans 12 px.
const BAR: Type = Type {
    size: 12.0,
    line: 16.0,
    face: Face::Sans400,
    spacing: 0.0,
};
/// The status word: 11.5 px, 600.
const STATUS: Type = ty::LABEL;
/// The label in the bar: mono at 0.95 of the bar size.
const BAR_MONO: Type = Type {
    size: 11.4,
    line: 16.0,
    face: Face::Mono400,
    spacing: 0.0,
};
/// The sentence under a failure: sans 12.5 px.
const SENTENCE: Type = ty::BODY;

/// The id of the handle, for F6.
pub fn handle_id() -> Id {
    Id::unique("mortise-runner-handle")
}

/// How much room the drawer takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shown {
    /// Not shown yet in this window: it takes no room.
    Never,
    /// One line of 28 px.
    Collapsed,
    Open,
}

/// What the window keeps of the drawer, for every app.
#[derive(Debug)]
pub struct RunnerView {
    pub shown: Shown,
    /// The open height, handle included. `None` until it is first opened: then it
    /// follows the height of the window. It is kept in memory only.
    height: Option<f32>,
    /// The greatest open height of the last frame.
    max_h: f32,
    pub history_open: bool,
    /// The selected row of the history.
    pub selected: usize,
    /// The history list takes focus in the next frame.
    focus_list: bool,
}

impl Default for RunnerView {
    fn default() -> Self {
        Self {
            shown: Shown::Never,
            height: None,
            max_h: size::DRAWER_H_MIN,
            history_open: false,
            selected: 0,
            focus_list: false,
        }
    }
}

impl RunnerView {
    /// Ctrl `: open a drawer that is not, collapse an open one.
    pub fn toggle(&mut self) {
        self.shown = match self.shown {
            Shown::Open => Shown::Collapsed,
            Shown::Never | Shown::Collapsed => Shown::Open,
        };
    }

    /// Shows the history, or hides it when it is up. The list takes focus.
    pub fn toggle_history(&mut self) {
        self.shown = Shown::Open;
        self.history_open = !self.history_open;
        self.selected = 0;
        self.focus_list = self.history_open;
    }

    /// The height the drawer takes at the bottom of a workspace column `column_h`
    /// high, in a window `window_h` high (section 5.14): at least 120 px and at most
    /// half of what is left under the workspace header; 200 px when it first opens,
    /// or 160 px under 700 px of window height.
    pub fn taken(&mut self, window_h: f32, column_h: f32) -> f32 {
        self.max_h = size::DRAWER_H_MIN.max((column_h - size::WORKSPACE_HEAD_H) / 2.0);
        match self.shown {
            Shown::Never => 0.0,
            Shown::Collapsed => size::DRAWER_COLLAPSED_H,
            Shown::Open => {
                let first = if window_h < 700.0 {
                    160.0
                } else {
                    size::DRAWER_H_DEFAULT
                };
                let height = *self.height.get_or_insert(first);
                self.clamp(height)
            }
        }
    }

    fn clamp(&self, height: f32) -> f32 {
        height.clamp(size::DRAWER_H_MIN, self.max_h).round()
    }
}

/// An entry of the history with its local clock time, read once.
#[derive(Debug, Clone)]
struct Listed {
    entry: Entry,
    /// `14:05`
    clock: String,
    today: bool,
}

/// What a run shown in the drawer is now.
#[derive(Debug)]
enum State {
    Running(Run),
    Ended(RunEnd),
    NotStarted(StartError),
}

/// The run of the app that the drawer shows: the one that runs, or the last one.
#[derive(Debug)]
struct Current {
    label: String,
    args: Vec<String>,
    /// The `$` line: the line that runs (deviation S36). For a command that did not
    /// start, the line Run again would run. None when the runner gives no line.
    line: Option<String>,
    output: Output,
    /// The lines of standard output alone. They are in `output` too.
    stdout: Vec<String>,
    /// The last non-empty line of standard output was cut at the line limit.
    stdout_cut: bool,
    /// The output passed its limit and the later lines were dropped.
    cut: bool,
    state: State,
    /// Started through Sail or Docker: a stop reaches `docker` only.
    container: bool,
    started_ms: i64,
    /// The id the run was kept under in the history, once it ended.
    recorded: Option<u64>,
    /// The migration at which a `module:migrate` run failed, read once when it ended.
    failed: Option<FailedMigration>,
    /// The run failed because the app prohibits the command (design D4).
    prohibited: bool,
}

/// The line that `args` would run through `runner` now, as the drawer shows it.
pub fn line_now(runner: Option<&RunnerKind>, args: &[String]) -> Option<String> {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    CommandLine::artisan(runner?, &words).map(|line| line.display())
}

/// How a run ended, with its output.
#[derive(Debug, Clone, Copy)]
pub struct RunOver<'a> {
    pub end: RunEnd,
    /// The output, both streams mixed, as the drawer shows it.
    pub lines: &'a [String],
    /// The lines of standard output alone.
    pub stdout: &'a [String],
    /// The document on standard output was cut, or dropped at the output limit.
    pub stdout_cut: bool,
}

/// The runs of one app: the one on screen, and the history kept on disk.
#[derive(Debug, Default)]
pub struct AppRuns {
    history: Option<History>,
    entries: Vec<Listed>,
    current: Option<Current>,
    /// The output of the history row on screen: its id, its lines and their spans.
    shown_output: Option<(u64, Output)>,
}

impl AppRuns {
    /// The runs of the app in `app`. Without `data_dir` no history is kept.
    pub fn open(data_dir: Option<&Path>, app: &Path) -> Self {
        let mut runs = Self {
            history: data_dir.map(|dir| History::open(dir, app)),
            ..Default::default()
        };
        runs.read_history();
        runs
    }

    fn read_history(&mut self) {
        let Some(history) = &self.history else {
            return;
        };
        let zone = jiff::tz::TimeZone::system();
        let today = jiff::Timestamp::now().to_zoned(zone.clone()).date();
        self.entries = history
            .entries()
            .into_iter()
            .map(|entry| {
                let local = jiff::Timestamp::from_millisecond(entry.started_ms)
                    .unwrap_or(jiff::Timestamp::UNIX_EPOCH)
                    .to_zoned(zone.clone());
                Listed {
                    clock: local.strftime("%H:%M").to_string(),
                    today: local.date() == today,
                    entry,
                }
            })
            .collect();
        self.shown_output = None;
    }

    /// The runs kept in the history, the latest first, as read at the last record.
    pub fn history(&self) -> Vec<Entry> {
        self.entries
            .iter()
            .map(|listed| listed.entry.clone())
            .collect()
    }

    /// Starts `args` through the runner of `php`, for at most its time limit. A start
    /// that fails is shown as such.
    pub fn start(&mut self, runs: &Runs, app: &Path, php: &PhpSettings, args: Vec<String>) {
        let container = matches!(
            php.runner,
            Some(RunnerKind::Sail | RunnerKind::Docker { .. })
        );
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        let label = words.join(" ");
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_millis() as i64);
        let current = match runs.start(app, php, &args, php.time_limit()) {
            Ok(run) => Current {
                label: run.label.clone(),
                args,
                line: Some(run.line.display()),
                output: Output::default(),
                stdout: Vec::new(),
                stdout_cut: false,
                cut: false,
                container,
                started_ms: run
                    .started_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(now_ms, |since| since.as_millis() as i64),
                state: State::Running(run),
                recorded: None,
                failed: None,
                prohibited: false,
            },
            Err(error) => Current {
                label,
                line: line_now(php.runner.as_ref(), &args),
                args,
                output: Output::default(),
                stdout: Vec::new(),
                stdout_cut: false,
                cut: false,
                container,
                started_ms: now_ms,
                state: State::NotStarted(error),
                recorded: None,
                failed: None,
                prohibited: false,
            },
        };
        self.current = Some(current);
    }

    /// Takes in what the run sent. A run that ended is kept in the history.
    /// Returns true while it runs.
    pub fn poll(&mut self) -> bool {
        match &self.current {
            Some(Current {
                state: State::Running(run),
                ..
            }) => {
                let events = run.events();
                self.take(events)
            }
            _ => false,
        }
    }

    /// Waits until the run ended or `deadline` passed, as [`AppRuns::poll`] takes
    /// in what it sent. A run that ended is kept in the history.
    pub fn wait(&mut self, deadline: Instant) {
        if let Some(Current {
            state: State::Running(run),
            ..
        }) = &self.current
        {
            let events = run.wait(deadline);
            self.take(events);
        }
    }

    /// Takes in `events` of the run that runs. Returns true while it runs.
    fn take(&mut self, events: Vec<RunEvent>) -> bool {
        let Some(current) = &mut self.current else {
            return false;
        };
        let mut ended = None;
        for event in events {
            match event {
                RunEvent::Line { stream, text, cut } => {
                    if stream == Stream::Out {
                        // An empty line after a cut one leaves the mark.
                        current.stdout_cut = cut || (current.stdout_cut && text.trim().is_empty());
                        current.stdout.push(text.clone());
                    }
                    current.output.push(text);
                }
                RunEvent::Cut => current.cut = true,
                RunEvent::Ended(end) => ended = Some(end),
            }
        }
        let Some(end) = ended else {
            return true;
        };
        let failed = matches!(end, RunEnd::Exited { code, .. } if code != Some(0));
        if failed
            && current
                .args
                .first()
                .is_some_and(|word| word == "module:migrate")
        {
            current.failed = mortise_core::failed_migration(&current.output.lines);
        }
        current.prohibited = failed && mortise_core::prohibited(&current.output.lines);
        current.state = State::Ended(end);
        self.record();
        false
    }

    /// Keeps the run that just ended in the history, under an id no other run has.
    fn record(&mut self) {
        let (Some(history), Some(current)) = (&self.history, &self.current) else {
            return;
        };
        let State::Ended(end) = current.state else {
            return;
        };
        let ms = |elapsed: Duration| elapsed.as_millis() as u64;
        let ended = match end {
            RunEnd::Exited {
                code: Some(0),
                elapsed,
            } => Ended::Done { ms: ms(elapsed) },
            RunEnd::Exited { code, elapsed } => Ended::Failed {
                code,
                ms: ms(elapsed),
            },
            RunEnd::Stopped { elapsed, limit } => Ended::Stopped {
                ms: ms(elapsed),
                limit: limit.is_some(),
            },
        };
        let mut id = current.started_ms.max(0) as u64;
        while self.entries.iter().any(|listed| listed.entry.id == id) {
            id += 1;
        }
        let entry = Entry {
            id,
            args: current.args.clone(),
            label: current.label.clone(),
            started_ms: current.started_ms,
            ended,
            cut: current.cut,
        };
        // A history that cannot be written loses this run only.
        if history.record(entry, &current.output.lines).is_ok()
            && let Some(current) = &mut self.current
        {
            current.recorded = Some(id);
        }
        self.read_history();
    }

    /// How the run on screen ended, with its output, once it has ended.
    pub fn ended(&self) -> Option<RunOver<'_>> {
        match &self.current {
            Some(Current {
                state: State::Ended(end),
                output,
                stdout,
                stdout_cut,
                cut,
                ..
            }) => Some(RunOver {
                end: *end,
                lines: &output.lines,
                stdout,
                stdout_cut: *stdout_cut || *cut,
            }),
            _ => None,
        }
    }

    /// The label of the command that runs, if one does.
    pub fn running(&self) -> Option<&str> {
        match &self.current {
            Some(Current {
                state: State::Running(_),
                label,
                ..
            }) => Some(label),
            _ => None,
        }
    }

    /// Asks the run to end. Returns the label when it is a command whose stop asks first.
    pub fn stop_asks_first(&self) -> Option<String> {
        match &self.current {
            Some(
                current @ Current {
                    state: State::Running(_),
                    ..
                },
            ) if mortise_core::stop_asks_first(&current.args) => Some(current.label.clone()),
            _ => None,
        }
    }

    pub fn stop(&self) {
        if let Some(Current {
            state: State::Running(run),
            ..
        }) = &self.current
        {
            run.stop();
        }
    }

    /// What Run again starts: the artisan words of the selected row of the history,
    /// or of the run on screen. A command runs only when the exact line it runs
    /// through `runner` is on screen: the `$` line of the run, or the `$` line above
    /// the output of the selected row. Otherwise the drawer opens where that line
    /// shows, and nothing runs: on the history with the run selected, or for a
    /// command that did not start, on its line now.
    pub fn again(
        &mut self,
        view: &mut RunnerView,
        runner: Option<&RunnerKind>,
    ) -> Option<Vec<String>> {
        let open = view.shown == Shown::Open;
        view.shown = Shown::Open;
        if self.history_shown(view) {
            let args = &self.entries.get(view.selected)?.entry.args;
            let shown = open && line_now(runner, args).is_some();
            return shown.then(|| args.clone());
        }
        let current = self.current.as_mut()?;
        let now = line_now(runner, &current.args);
        if open && now.is_some() && now == current.line {
            return Some(current.args.clone());
        }
        match current.recorded {
            Some(id) => {
                view.history_open = true;
                view.selected = self
                    .entries
                    .iter()
                    .position(|listed| listed.entry.id == id)
                    .unwrap_or(0);
            }
            // Nothing ran: the line it shows is the line Run again runs.
            None if matches!(current.state, State::NotStarted(_)) => current.line = now,
            None => {}
        }
        None
    }

    /// The output on screen as plain text, a line each.
    pub fn output_text(&mut self, view: &RunnerView) -> Option<String> {
        let lines: &[String] = if self.history_shown(view) {
            &self.selected_output(view.selected)?.lines
        } else {
            &self.current.as_ref()?.output.lines
        };
        let text: Vec<String> = lines.iter().map(|line| ansi::plain(line)).collect();
        Some(text.join("\n"))
    }

    /// The history is on screen: asked for, or the app has no run in this window yet.
    fn history_shown(&self, view: &RunnerView) -> bool {
        !self.entries.is_empty() && (view.history_open || self.current.is_none())
    }

    /// The output of a row of the history, read from disk once.
    fn selected_output(&mut self, row: usize) -> Option<&Output> {
        let id = self.entries.get(row)?.entry.id;
        if self
            .shown_output
            .as_ref()
            .is_none_or(|(shown, _)| *shown != id)
        {
            let lines = self
                .history
                .as_ref()
                .map(|history| history.output(id))
                .unwrap_or_default();
            self.shown_output = Some((id, Output::of(lines)));
        }
        self.shown_output.as_ref().map(|(_, output)| output)
    }
}

/// Lines as written, with their codes, and their spans.
#[derive(Debug, Default)]
struct Output {
    lines: Vec<String>,
    spans: Vec<Vec<Span>>,
}

impl Output {
    fn of(lines: Vec<String>) -> Self {
        let spans = lines.iter().map(|line| ansi::spans(line)).collect();
        Self { lines, spans }
    }

    fn push(&mut self, line: String) {
        self.spans.push(ansi::spans(&line));
        self.lines.push(line);
    }
}

// ---------------------------------------------------------------------------
// Drawing

/// What the drawer asked for in a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawerAction {
    Stop,
    /// Run the selected or shown command again, through the current runner of the
    /// app, when its line is on screen (see [`AppRuns::again`]).
    RunAgain,
    CopyOutput,
    ToggleHistory,
    Hide,
    Show,
    /// Settings, on how PHP runs for the app.
    OpenSettings,
    /// Open the file of the migration that failed, in the editor.
    OpenMigration(PathBuf),
    /// Open the Migrate panel of this module, or of "All modules" for `None`, with
    /// the dry run checked and the database of the failed run. Nothing runs
    /// (deviation S38).
    DryRun {
        module: Option<String>,
        database: Option<String>,
    },
}

/// What the drawer shows besides the runs.
pub struct Inputs<'a> {
    /// The display name of the app.
    pub app: &'a str,
    /// How PHP runs for the app now: History shows the line Run again would run.
    pub runner: Option<&'a RunnerKind>,
    /// The app as read, to find the file of a migration that failed.
    pub snapshot: &'a AppSnapshot,
    /// The label of the command or read that runs in the app, if one does. Run
    /// again is off while it runs.
    pub busy: Option<&'a str>,
    /// True while the keyboard is in use: focus rings are drawn.
    pub keyboard: bool,
    pub mac: bool,
}

/// Paints the drawer into `rect`, the bottom of the workspace column.
pub fn drawer(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    view: &mut RunnerView,
    runs: &mut AppRuns,
    inputs: &Inputs<'_>,
) -> Vec<DrawerAction> {
    let mut actions = Vec::new();
    match view.shown {
        Shown::Never => {}
        Shown::Collapsed => collapsed(ui, pal, rect, runs, inputs, &mut actions),
        Shown::Open => open(ui, pal, rect, view, runs, inputs, &mut actions),
    }
    actions
}

/// The mark before a status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Run,
    Stop,
    Good,
    Bad,
}

/// The status of a run on the right of the bar: `Done in 186 ms`, `Exit code 1` and `412 ms`.
struct Status {
    mark: Mark,
    text: String,
    color: Color32,
    /// The time beside an exit code.
    time: Option<String>,
    /// A text as wide as `text` can be, whose room `text` keeps, so the mark does
    /// not move as the time changes.
    room: String,
    /// The run is over: the status is announced.
    ended: bool,
}

fn status_of(current: &Current, pal: &Palette) -> Option<Status> {
    let status = match &current.state {
        State::Running(run) => Status {
            mark: Mark::Run,
            text: strings::running_for(run.elapsed()),
            color: pal.code_fg,
            time: None,
            room: strings::running_for(Duration::from_millis(88_800)),
            ended: false,
        },
        State::Ended(RunEnd::Exited {
            code: Some(0),
            elapsed,
        }) => Status {
            mark: Mark::Good,
            text: strings::done_in(*elapsed),
            color: pal.code_ok,
            time: None,
            room: strings::done_in(Duration::from_millis(888)),
            ended: true,
        },
        State::Ended(RunEnd::Exited { code, elapsed }) => Status {
            mark: Mark::Bad,
            text: strings::exit_code(*code),
            color: pal.code_err,
            time: Some(strings::run_time(*elapsed)),
            room: String::new(),
            ended: true,
        },
        State::Ended(RunEnd::Stopped { elapsed, .. }) => Status {
            mark: Mark::Stop,
            text: strings::stopped_after(*elapsed),
            color: pal.code_fg,
            time: None,
            room: strings::stopped_after(Duration::from_millis(88_800)),
            ended: true,
        },
        State::NotStarted(_) => return None,
    };
    Some(status)
}

fn status_width(ui: &Ui, status: &Status) -> f32 {
    let width = |text: &str| {
        draw::galley(ui, text, STATUS, Color32::PLACEHOLDER)
            .size()
            .x
    };
    let text = width(&status.text).max(width(&status.room));
    // The time keeps the room of "999 ms", so the status does not move as times differ.
    let time = status.time.as_ref().map_or(0.0, |time| {
        let width = |text: &str| {
            draw::galley(ui, text, BAR_MONO, Color32::PLACEHOLDER)
                .size()
                .x
        };
        8.0 + width(time).max(width("999 ms"))
    });
    (size::STATE_MARK + 5.0 + text + time).ceil()
}

fn paint_mark(ui: &Ui, pal: &Palette, rect: Rect, mark: Mark, color: Color32) {
    match mark {
        Mark::Good => draw::mark_good(ui, rect, color),
        Mark::Bad => draw::mark_bad(ui, rect, color, pal.code_bg),
        Mark::Run => draw::icon(ui, rect, Icon::Run, color),
        Mark::Stop => draw::icon(ui, rect, Icon::Stop, color),
    }
}

/// Paints a status with its left at `x`. The end of a run is announced once, through
/// a live region (reference, section 9: the end of a command, not every line).
fn paint_status(ui: &Ui, pal: &Palette, x: f32, mid: f32, status: &Status, label: &str) {
    let mark = Rect::from_min_size(
        pos2(x, (mid - size::STATE_MARK / 2.0).round()),
        Vec2::splat(size::STATE_MARK),
    );
    paint_mark(ui, pal, mark, status.mark, status.color);
    let text = draw::paint_text(
        ui,
        pos2(mark.right() + 5.0, mid),
        Align2::LEFT_CENTER,
        &status.text,
        STATUS,
        status.color,
    );
    if status.ended {
        let said = strings::run_ended(label, &status.text);
        let announced = draw::announce(ui, text, Role::Label, &said);
        ui.ctx().accesskit_node_builder(announced.id, |node| {
            node.set_live(egui::accesskit::Live::Polite);
        });
    } else {
        draw::announce(ui, text, Role::Label, &status.text);
    }
    if let Some(time) = &status.time {
        draw::label(
            ui,
            pos2(text.right() + 8.0, mid),
            Align2::LEFT_CENTER,
            time,
            BAR_MONO,
            pal.code_dim,
        );
    }
}

/// A quiet button of the drawer: its label and its keys, code colours on the code fill.
struct BarButton {
    label: &'static str,
    keys: Option<String>,
    action: DrawerAction,
    /// History while it is up.
    pressed: bool,
    /// Off, with this tooltip.
    off: Option<String>,
    /// The name a screen reader says, where the label is not enough.
    said: Option<String>,
    /// Drawn in the text colour, as a step under a failure is.
    bright: bool,
    /// A flag after the label, in mono: `--pretend`.
    flag: Option<&'static str>,
}

impl BarButton {
    fn new(label: &'static str, action: DrawerAction) -> Self {
        Self {
            label,
            keys: None,
            action,
            pressed: false,
            off: None,
            said: None,
            bright: false,
            flag: None,
        }
    }

    fn keys(mut self, keys: impl Into<String>) -> Self {
        self.keys = Some(keys.into());
        self
    }

    fn label_type(&self) -> Type {
        if self.pressed {
            BAR.with_face(Face::Sans600)
        } else {
            BAR
        }
    }

    fn width(&self, ui: &Ui) -> f32 {
        let label = draw::galley(ui, self.label, self.label_type(), Color32::PLACEHOLDER)
            .size()
            .x;
        let keys = self
            .keys
            .as_ref()
            .map_or(0.0, |keys| 6.0 + draw::keycap_width(ui, keys));
        let flag = self.flag.map_or(0.0, |flag| {
            let galley = draw::galley(ui, flag, BAR_MONO, Color32::PLACEHOLDER);
            4.0 + galley.size().x
        });
        (4.0 + label + flag + keys + 4.0).ceil()
    }

    /// Paints the button with its left at `x`, centred on `mid`.
    fn show(&self, ui: &Ui, pal: &Palette, x: f32, mid: f32, keyboard: bool) -> Response {
        let rect = Rect::from_min_size(
            pos2(x, (mid - BAR_H / 2.0).round()),
            vec2(self.width(ui), BAR_H),
        );
        let said = self.said.as_deref().unwrap_or(self.label);
        let enabled = self.off.is_none();
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui.interact(rect, Id::unique(("mortise-runner-button", said)), sense);
        response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, said));
        if enabled && response.hovered() {
            ui.painter().rect_filled(rect, radius::CONTROL, HOVER);
        }
        let color = match (enabled, self.pressed || self.bright || response.hovered()) {
            (false, _) => pal.code_dim.gamma_multiply(0.6),
            (true, true) => pal.code_fg,
            (true, false) => pal.code_dim,
        };
        let label = draw::paint_text(
            ui,
            pos2(rect.left() + 4.0, mid),
            Align2::LEFT_CENTER,
            self.label,
            self.label_type(),
            color,
        );
        let label = match self.flag {
            Some(flag) => {
                let at = pos2(label.right() + 4.0, mid);
                draw::paint_text(ui, at, Align2::LEFT_CENTER, flag, BAR_MONO, color)
            }
            None => label,
        };
        if let Some(keys) = &self.keys {
            code_keycap(ui, pal, pos2(label.right() + 6.0, mid), keys);
        }
        if keyboard && response.has_focus() {
            code_focus_ring(ui, pal, rect);
        }
        match &self.off {
            Some(tip) => draw::tooltip(response, pal, tip),
            None => response,
        }
    }
}

/// A key hint on the code fill: a code-dim outline, no fill.
fn code_keycap(ui: &Ui, pal: &Palette, left_center: egui::Pos2, keys: &str) -> Rect {
    let rect =
        Align2::LEFT_CENTER.anchor_size(left_center, vec2(draw::keycap_width(ui, keys), 16.0));
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(radius::STAMP),
        Stroke::new(stroke::HAIRLINE, pal.code_dim),
        StrokeKind::Inside,
    );
    draw::paint_text(
        ui,
        rect.center(),
        Align2::CENTER_CENTER,
        keys,
        ty::KEYCAP,
        pal.code_dim,
    );
    rect
}

/// The focus ring on the code fill, in the code text colour: ink would not show.
fn code_focus_ring(ui: &Ui, pal: &Palette, rect: Rect) {
    draw::ring_is_drawn(ui);
    ui.painter().rect_stroke(
        rect.expand(stroke::FOCUS_OFFSET),
        CornerRadius::same(radius::CONTROL + stroke::FOCUS_OFFSET as u8),
        Stroke::new(stroke::FOCUS_RING, pal.code_fg),
        StrokeKind::Outside,
    );
}

/// Paints buttons from `x` to the right, 8 px apart, and adds what was clicked.
fn show_buttons(
    ui: &Ui,
    pal: &Palette,
    buttons: Vec<BarButton>,
    mut x: f32,
    mid: f32,
    keyboard: bool,
    actions: &mut Vec<DrawerAction>,
) {
    for button in buttons {
        let response = button.show(ui, pal, x, mid, keyboard);
        x = response.rect.right() + 8.0;
        if response.clicked() {
            actions.push(button.action);
        }
    }
}

fn buttons_width(ui: &Ui, buttons: &[BarButton]) -> f32 {
    let widths: f32 = buttons.iter().map(|button| button.width(ui)).sum();
    widths + 8.0 * buttons.len().saturating_sub(1) as f32
}

/// "Runner" and the label, elided to what fits in `width`. Returns where they end.
fn paint_title(ui: &Ui, pal: &Palette, x: f32, mid: f32, label: &str, width: f32) -> f32 {
    let runner = draw::paint_text(
        ui,
        pos2(x, mid),
        Align2::LEFT_CENTER,
        strings::RUNNER,
        BAR.with_face(Face::Sans700),
        pal.code_fg,
    );
    draw::announce(ui, runner, Role::Heading, strings::RUNNER);
    if label.is_empty() {
        return runner.right();
    }
    let room = (width - (runner.right() - x) - 8.0).max(0.0);
    let galley = draw::galley_elided(ui, label, BAR_MONO, pal.code_dim, room);
    let at = draw::paint_galley(
        ui,
        pos2(runner.right() + 8.0, mid),
        Align2::LEFT_CENTER,
        galley,
    );
    draw::announce(ui, at, Role::Label, label);
    at.right()
}

/// e. The drawer as one line of 28 px: the last status and Show.
fn collapsed(
    ui: &Ui,
    pal: &Palette,
    rect: Rect,
    runs: &AppRuns,
    inputs: &Inputs<'_>,
    actions: &mut Vec<DrawerAction>,
) {
    ui.painter().rect_filled(rect, 0, pal.code_bg);
    draw::hline(ui, rect.left()..rect.right(), rect.top(), pal.line_2);
    let mid = rect.center().y + 0.5;
    let show =
        vec![BarButton::new(strings::SHOW, DrawerAction::Show).keys(strings::TOGGLE_RUNNER_KEYS)];
    let show_x = rect.right() - PAD_X - buttons_width(ui, &show);
    let current = runs.current.as_ref();
    let status = current.and_then(|current| status_of(current, pal));
    let status_w = status
        .as_ref()
        .map_or(0.0, |status| 8.0 + status_width(ui, status));
    let label = current.map_or("", |current| current.label.as_str());
    let left = rect.left() + PAD_X;
    let end = paint_title(ui, pal, left, mid, label, show_x - 16.0 - status_w - left);
    if let Some(status) = &status {
        paint_status(ui, pal, end + 8.0, mid, status, label);
    }
    show_buttons(ui, pal, show, show_x, mid, inputs.keyboard, actions);
}

/// C15: the top 7 px of the drawer. Drag it, or focus it and use Up, Down, Home and
/// End. A double click collapses the drawer.
fn handle(ui: &Ui, pal: &Palette, rect: Rect, view: &mut RunnerView, bottom: f32, keyboard: bool) {
    let response = ui.interact(rect, handle_id(), Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(Role::Splitter, true, strings::DRAWER_HEIGHT));
    let pressed = response.dragged() || response.is_pointer_button_down_on();
    if response.hovered() || pressed {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }
    if response.dragged()
        && let Some(pointer) = response.interact_pointer_pos()
    {
        view.height = Some(view.clamp(bottom - pointer.y));
        ui.ctx().request_repaint();
    }
    if response.double_clicked() {
        view.shown = Shown::Collapsed;
    }
    if response.has_focus() {
        // Nothing is reachable only by dragging: the keys move the edge too.
        draw::keep_arrows(ui, handle_id(), true, false);
        let height = bottom - rect.top();
        let (up, down, home, end) = ui.input_mut(|input| {
            let mut pressed = |key| input.consume_key(Modifiers::NONE, key);
            (
                pressed(Key::ArrowUp),
                pressed(Key::ArrowDown),
                pressed(Key::Home),
                pressed(Key::End),
            )
        });
        let target = if up {
            Some(height + HANDLE_STEP)
        } else if down {
            Some(height - HANDLE_STEP)
        } else if home {
            Some(view.max_h)
        } else if end {
            Some(size::DRAWER_H_MIN)
        } else {
            None
        };
        if let Some(target) = target {
            view.height = Some(view.clamp(target));
            ui.ctx().request_repaint();
        }
    }

    ui.painter().rect_filled(rect, 0, pal.surface);
    let hot = response.hovered() || pressed;
    let edge = if hot { pal.ink } else { pal.line_2 };
    let edge_h = if pressed { 2.0 } else { 1.0 };
    ui.painter().rect_filled(
        Rect::from_min_size(rect.min, vec2(rect.width(), edge_h)),
        0,
        edge,
    );
    let grip_h = if hot { 2.0 } else { 1.0 };
    let grip = Rect::from_min_size(
        pos2((rect.center().x - 14.0).round(), (rect.center().y).floor()),
        vec2(28.0, grip_h),
    );
    ui.painter().rect_filled(grip, 0, edge);
    if keyboard && response.has_focus() {
        draw::focus_ring(ui, pal, rect.shrink2(vec2(0.0, 1.0)), 0);
    }
}

/// The drawer at its height: the handle, the bar and the body.
fn open(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    view: &mut RunnerView,
    runs: &mut AppRuns,
    inputs: &Inputs<'_>,
    actions: &mut Vec<DrawerAction>,
) {
    let handle_rect = Rect::from_min_size(rect.min, vec2(rect.width(), HANDLE_H));
    handle(ui, pal, handle_rect, view, rect.bottom(), inputs.keyboard);
    let fill = Rect::from_min_max(pos2(rect.left(), handle_rect.bottom()), rect.max);
    ui.painter().rect_filled(fill, 0, pal.code_bg);

    let mid = handle_rect.bottom() + PAD_TOP + BAR_H / 2.0;
    let left = rect.left() + PAD_X;
    let right = rect.right() - PAD_X;
    let body = Rect::from_min_max(
        pos2(left, mid + BAR_H / 2.0 + 5.0),
        pos2(right, rect.bottom() - PAD_BOTTOM),
    );
    let roomy = rect.width() >= ROOMY_W;
    let busy = inputs.busy.map(strings::is_running);
    let hide =
        || BarButton::new(strings::HIDE, DrawerAction::Hide).keys(strings::TOGGLE_RUNNER_KEYS);
    let run_again = |keys: Option<&str>| BarButton {
        keys: keys.map(str::to_owned),
        off: busy.clone(),
        ..BarButton::new(strings::RUN_AGAIN, DrawerAction::RunAgain)
    };
    let history_button = |pressed: bool| BarButton {
        pressed,
        ..BarButton::new(strings::HISTORY, DrawerAction::ToggleHistory)
    };
    let copy = || BarButton::new(strings::COPY_OUTPUT, DrawerAction::CopyOutput);

    let history = runs.history_shown(view);
    let (label, status, buttons) = if history {
        let today = runs.entries.iter().filter(|listed| listed.today).count();
        let buttons = vec![
            run_again(Some("Enter")),
            copy(),
            history_button(true),
            hide(),
        ];
        (strings::runs_today(today, inputs.app), None, buttons)
    } else {
        match &runs.current {
            None => (String::new(), None, vec![hide()]),
            Some(current) => {
                let buttons = match &current.state {
                    State::Running(_) => vec![
                        BarButton::new(strings::STOP, DrawerAction::Stop)
                            .keys(strings::stop_keys(inputs.mac)),
                        hide(),
                    ],
                    _ if !roomy => vec![hide()],
                    State::NotStarted(_) => vec![run_again(None), history_button(false), hide()],
                    State::Ended(_) => {
                        vec![run_again(None), copy(), history_button(false), hide()]
                    }
                };
                (current.label.clone(), status_of(current, pal), buttons)
            }
        }
    };
    let buttons_x = right - buttons_width(ui, &buttons);
    let status_w = status
        .as_ref()
        .map_or(0.0, |status| status_width(ui, status) + 10.0);
    paint_title(
        ui,
        pal,
        left,
        mid,
        &label,
        buttons_x - status_w - 16.0 - left,
    );
    if let Some(status) = &status {
        paint_status(ui, pal, buttons_x - status_w, mid, status, &label);
    }
    let mut clicked = Vec::new();
    show_buttons(
        ui,
        pal,
        buttons,
        buttons_x,
        mid,
        inputs.keyboard,
        &mut clicked,
    );
    actions.extend(clicked);

    if history {
        history_body(ui, pal, body, view, runs, busy.is_some(), inputs, actions);
        return;
    }
    let Some(current) = &runs.current else {
        // f. Nothing has run yet.
        let text = Type {
            size: 12.0,
            ..ty::SECONDARY
        };
        draw::paragraph(
            ui,
            body.min,
            strings::NOTHING_RAN,
            text,
            pal.code_dim,
            body.width(),
        );
        return;
    };
    match &current.state {
        State::NotStarted(error) => {
            let text = strings::did_not_start(error, inputs.app);
            let width = body.width().min(sentence_width());
            // The line that Run again starts, when the runner gives one.
            let mut top = body.min;
            if let Some(line) = &current.line {
                let first = format!("$ {line}");
                let at = pos2(top.x, top.y + ty::RUNNER.line / 2.0);
                let rect = draw::label(
                    ui,
                    at,
                    Align2::LEFT_CENTER,
                    &first,
                    ty::RUNNER,
                    pal.code_dim,
                );
                top.y = rect.top() + ty::RUNNER.line + 4.0;
            }
            let rect = draw::paragraph(ui, top, &text, SENTENCE, pal.code_fg, width);
            let button = BarButton {
                said: Some(strings::open_php_settings(inputs.app)),
                bright: true,
                ..BarButton::new(strings::OPEN_SETTINGS, DrawerAction::OpenSettings)
            };
            let mid = rect.bottom() + 8.0 + BAR_H / 2.0;
            let x = body.left() - 4.0;
            show_buttons(ui, pal, vec![button], x, mid, inputs.keyboard, actions);
        }
        State::Running(_) | State::Ended(_) => {
            current_body(ui, pal, body, current, inputs, actions);
        }
    }
}

/// The sentences of Mortise in the drawer are at most 86 characters wide.
fn sentence_width() -> f32 {
    86.0 * 0.535 * SENTENCE.size
}

/// The output of the run on screen, the notes of Mortise after it, and for a
/// failure the sentence under a rule with its steps.
fn current_body(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    current: &Current,
    inputs: &Inputs<'_>,
    actions: &mut Vec<DrawerAction>,
) {
    let mut notes = Vec::new();
    if current.cut {
        notes.push(strings::OUTPUT_CUT.to_owned());
    }
    let (failed, limit, stopped) = match current.state {
        State::Ended(RunEnd::Exited { code, .. }) if code != Some(0) => (Some(code), None, false),
        State::Ended(RunEnd::Stopped { limit, .. }) => (None, limit, true),
        _ => (None, None, false),
    };
    if let Some(limit) = limit {
        notes.push(strings::stopped_at_limit(limit));
    }
    if stopped && current.container {
        notes.push(strings::CONTAINER_MAY_RUN.to_owned());
    }

    // The sentence of a failure follows the output; a long output scrolls above it.
    let width = body.width().min(sentence_width());
    let sentence = failed.map(|code| {
        // The refusal of the app wins over the migration that failed (design D4).
        let text = match &current.failed {
            _ if current.prohibited => strings::PROHIBITED.to_owned(),
            Some(at) => strings::failed_at(&current.label, code, &at.migration),
            None => strings::stopped_with(&current.label, code),
        };
        let galley = draw::galley_wrapped(ui, &text, SENTENCE, pal.code_fg, width);
        (text, galley)
    });
    let block_h = sentence.as_ref().map_or(0.0, |(_, galley)| {
        10.0 + 1.0 + 8.0 + galley.size().y + 4.0 + BAR_H
    });
    let first = current.line.as_ref().map(|line| format!("$ {line}"));
    let rows = usize::from(first.is_some()) + current.output.lines.len() + notes.len();
    let rows_h = rows as f32 * ty::RUNNER.line;
    let room = (body.height() - block_h).max(ty::RUNNER.line);
    let output_rect = Rect::from_min_size(body.min, vec2(body.width(), rows_h.min(room)));
    output_rows(
        ui,
        pal,
        output_rect,
        "runner-output",
        first.as_deref(),
        &current.output,
        &notes,
        true,
    );

    if let Some((text, galley)) = sentence {
        let rule_y = output_rect.bottom() + 10.0;
        let rule = Rect::from_min_size(pos2(body.left(), rule_y), vec2(width, 1.0));
        ui.painter().rect_filled(rule, 0, RULE);
        let at = draw::paint_galley(
            ui,
            pos2(body.left(), rule_y + 1.0 + 8.0),
            Align2::LEFT_TOP,
            galley,
        );
        draw::announce(ui, at, Role::Label, &text);
        let copy = BarButton {
            said: Some(strings::copy_output_of(&current.label)),
            bright: true,
            ..BarButton::new(strings::COPY_OUTPUT, DrawerAction::CopyOutput)
        };
        let steps = match &current.failed {
            Some(at) if !current.prohibited => {
                let mut steps = migration_steps(ui, current, at, inputs.snapshot, actions);
                steps.push(copy);
                steps
            }
            _ => vec![copy],
        };
        let mid = at.bottom() + 4.0 + BAR_H / 2.0;
        show_buttons(
            ui,
            pal,
            steps,
            body.left() - 4.0,
            mid,
            inputs.keyboard,
            actions,
        );
    }
}

/// The steps under a migration that failed (S11.1 c): "Open migration file" when
/// the file is in the module's migrations folder, and "Dry run". Enter opens the
/// file while the handle of the drawer holds focus.
fn migration_steps(
    ui: &Ui,
    current: &Current,
    failed: &FailedMigration,
    app: &AppSnapshot,
    actions: &mut Vec<DrawerAction>,
) -> Vec<BarButton> {
    let file = failed
        .module
        .clone()
        .or_else(|| module_named(&current.args))
        .as_deref()
        .and_then(|name| app.module(name))
        .and_then(|module| mortise_core::migration_file(&module.dir, &failed.migration));
    let mut steps = Vec::new();
    if let Some(path) = file {
        let handle_focused = ui.memory(|memory| memory.has_focus(handle_id()));
        if handle_focused && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter)) {
            actions.push(DrawerAction::OpenMigration(path.clone()));
        }
        steps.push(BarButton {
            pressed: true,
            ..BarButton::new(strings::OPEN_MIGRATION, DrawerAction::OpenMigration(path))
                .keys("Enter")
        });
    }
    steps.push(BarButton {
        said: Some(strings::dry_run_of(&current.label)),
        bright: true,
        flag: Some("--pretend"),
        ..BarButton::new(strings::DRY_RUN, dry_run(&current.args, failed))
    });
    steps
}

/// The options of artisan words, before `--`, and the word after it.
fn split_options(args: &[String]) -> (&[String], Option<&String>) {
    match args.iter().position(|word| word == "--") {
        Some(at) => (&args[..at], args.get(at + 1)),
        None => (args, None),
    }
}

/// The module that artisan words name: the second word, or the word after `--`.
pub(crate) fn module_named(args: &[String]) -> Option<String> {
    let (options, after) = split_options(args);
    after
        .or(options.get(1).filter(|word| !word.starts_with('-')))
        .cloned()
}

/// "Dry run" under a failed migration: the module, or "All modules" for `--all`,
/// and the database of the run that failed.
fn dry_run(args: &[String], failed: &FailedMigration) -> DrawerAction {
    let (options, _) = split_options(args);
    let module = if options.iter().any(|word| word == "--all") {
        None
    } else {
        failed.module.clone().or_else(|| module_named(args))
    };
    let database = options
        .iter()
        .find_map(|word| word.strip_prefix("--database="))
        .map(str::to_owned);
    DrawerAction::DryRun { module, database }
}

/// The colour of a style of the output.
fn style_color(style: Style, pal: &Palette) -> Color32 {
    match style {
        Style::Text => pal.code_fg,
        Style::Dim => pal.code_dim,
        Style::Success => pal.code_ok,
        Style::Error => pal.code_err,
    }
}

/// The text as drawn. The bundled mono font has no U+21C2, which Laravel prints
/// in `--pretend` and `module:lang` output, so it is drawn as U+2193. Copied
/// text and the history keep the real character.
fn drawable(text: &str) -> String {
    text.replace('\u{21c2}', "\u{2193}")
}

/// One line of output in its styles, on one line however long.
fn line_galley(ui: &Ui, spans: &[Span], pal: &Palette) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    for span in spans {
        job.append(
            &drawable(&span.text),
            0.0,
            ty::RUNNER.format(style_color(span.style, pal)),
        );
    }
    if job.sections.is_empty() {
        job.append("", 0.0, ty::RUNNER.format(pal.code_fg));
    }
    job.wrap.max_width = f32::INFINITY;
    ui.painter().layout_job(job)
}

/// Lines of output in `rect`, as rows of the runner line height: only the rows in
/// view are laid out, so a long output stays quick. `first` is a line before the
/// output, `notes` lines after it, both dim.
#[allow(clippy::too_many_arguments)]
fn output_rows(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    salt: impl std::hash::Hash + std::fmt::Debug,
    first: Option<&str>,
    output: &Output,
    notes: &[String],
    stick: bool,
) {
    let skip = usize::from(first.is_some());
    let rows = skip + output.lines.len() + notes.len();
    let dim = |text: &str| {
        vec![Span {
            style: Style::Dim,
            text: text.to_owned(),
        }]
    };
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ScrollArea::both()
            .id_salt(salt)
            .auto_shrink(false)
            .stick_to_bottom(stick)
            .show_rows(ui, ty::RUNNER.line, rows, |ui, range| {
                for row in range {
                    let (spans, plain) = if row < skip {
                        let line = first.unwrap_or_default();
                        (dim(line), line.to_owned())
                    } else if row - skip < output.lines.len() {
                        let at = row - skip;
                        (output.spans[at].clone(), ansi::plain(&output.lines[at]))
                    } else {
                        let note = &notes[row - skip - output.lines.len()];
                        (dim(note), note.clone())
                    };
                    let galley = line_galley(ui, &spans, pal);
                    let (at, _) = ui.allocate_exact_size(
                        vec2(galley.size().x.max(1.0), ty::RUNNER.line),
                        Sense::hover(),
                    );
                    let top = at.center().y - galley.size().y / 2.0;
                    ui.painter()
                        .galley(pos2(at.left(), top), galley, Color32::PLACEHOLDER);
                    if !plain.trim().is_empty() {
                        draw::announce(ui, at, Role::Label, &plain);
                    }
                }
            });
    });
}

/// d. The runs of the app on the left, the output of the selected one on the right.
#[allow(clippy::too_many_arguments)]
fn history_body(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    view: &mut RunnerView,
    runs: &mut AppRuns,
    busy: bool,
    inputs: &Inputs<'_>,
    actions: &mut Vec<DrawerAction>,
) {
    let keyboard = inputs.keyboard;
    let list_w = HISTORY_LIST_W.min((body.width() - 14.0) / 2.0).floor();
    let list = Rect::from_min_size(body.min, vec2(list_w, body.height()));
    let output_rect = Rect::from_min_max(pos2(list.right() + 14.0, body.top()), body.max);
    let entries = &runs.entries;
    ui.scope_builder(UiBuilder::new().max_rect(list), |ui| {
        let nav = ListNav::over(ui, "runner-history", entries.len(), &mut view.selected);
        if std::mem::take(&mut view.focus_list) {
            nav.focus(ui);
        }
        // Enter runs the selected command again, through the current runner.
        if nav.has_focus(ui)
            && !busy
            && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter))
        {
            actions.push(DrawerAction::RunAgain);
        }
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ScrollArea::vertical()
            .id_salt("runner-history-list")
            .auto_shrink(false)
            .show_rows(ui, HISTORY_ROW_H, entries.len(), |ui, range| {
                for row in range {
                    let rect = ui
                        .allocate_exact_size(
                            vec2(ui.available_width(), HISTORY_ROW_H),
                            Sense::hover(),
                        )
                        .0;
                    let selected = row == view.selected;
                    let listed = &entries[row];
                    let (mark, result) = result_of(&listed.entry.ended);
                    let said = format!("{} {}, {result}", listed.clock, listed.entry.label);
                    let response = ui.interact(rect, nav.id(row), nav.sense(row));
                    response.widget_info(|| {
                        WidgetInfo::selected(Role::ListBoxOption, true, selected, &said)
                    });
                    if response.clicked() {
                        view.selected = row;
                    }
                    nav.after(ui, row, &response);
                    history_row(ui, pal, rect, listed, mark, &result, selected, &response);
                    if keyboard && response.has_focus() {
                        code_focus_ring(ui, pal, rect.shrink(1.0));
                    }
                }
            });
    });
    // The line that Enter and Run again start, above the output of the selected run.
    let selected = view.selected;
    let Some(listed) = runs.entries.get(selected) else {
        return;
    };
    let id = listed.entry.id;
    let first = line_now(inputs.runner, &listed.entry.args).map(|line| format!("$ {line}"));
    if let Some(output) = runs.selected_output(selected) {
        output_rows(
            ui,
            pal,
            output_rect,
            ("runner-history-output", id),
            first.as_deref(),
            output,
            &[],
            false,
        );
    }
}

/// Paints a row of the history: the clock time and the label, then the result.
#[allow(clippy::too_many_arguments)]
fn history_row(
    ui: &Ui,
    pal: &Palette,
    rect: Rect,
    listed: &Listed,
    mark: Mark,
    result: &str,
    selected: bool,
    response: &Response,
) {
    if selected || response.hovered() {
        ui.painter().rect_filled(rect, 0, HOVER);
    }
    let color = if selected { pal.code_fg } else { pal.code_dim };
    let result_color = match mark {
        Mark::Good => pal.code_ok,
        Mark::Bad => pal.code_err,
        Mark::Run | Mark::Stop => color,
    };
    let mid = rect.center().y;
    let result_w = draw::galley(ui, result, STATUS, Color32::PLACEHOLDER)
        .size()
        .x;
    let result_x = rect.right() - 6.0 - result_w;
    let mark_rect = Rect::from_min_size(
        pos2(
            result_x - 5.0 - size::STATE_MARK,
            (mid - size::STATE_MARK / 2.0).round(),
        ),
        Vec2::splat(size::STATE_MARK),
    );
    paint_mark(ui, pal, mark_rect, mark, result_color);
    draw::paint_text(
        ui,
        pos2(result_x, mid),
        Align2::LEFT_CENTER,
        result,
        STATUS,
        result_color,
    );
    let text = format!("{}  {}", listed.clock, listed.entry.label);
    let room = mark_rect.left() - 8.0 - (rect.left() + 6.0);
    let galley = draw::galley_elided(ui, &text, BAR_MONO, color, room);
    draw::paint_galley(
        ui,
        pos2(rect.left() + 6.0, mid),
        Align2::LEFT_CENTER,
        galley,
    );
}

/// The mark and the words of a row of the history: `186 ms`, or `Exit code 1`.
fn result_of(ended: &Ended) -> (Mark, String) {
    let time = |ms: u64| Duration::from_millis(ms);
    match ended {
        Ended::Done { ms } => (Mark::Good, strings::run_time(time(*ms))),
        Ended::Failed { code, .. } => (Mark::Bad, strings::exit_code(*code)),
        Ended::Stopped { ms, .. } => (Mark::Stop, strings::stopped_after(time(*ms))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_harpoon_is_drawn_as_an_arrow() {
        assert_eq!(drawable("\u{21c2} create table"), "\u{2193} create table");
        assert_eq!(drawable("plain"), "plain");
    }

    #[test]
    fn a_dry_run_keeps_the_module_and_the_database_of_the_failed_run() {
        let words =
            |words: &[&str]| -> Vec<String> { words.iter().map(|word| word.to_string()).collect() };
        let failed = |module: Option<&str>| FailedMigration {
            module: module.map(str::to_owned),
            migration: "2026_11_22_140500_add_tracking_code_to_shipments_table".to_owned(),
        };
        assert_eq!(
            dry_run(
                &words(&["module:migrate", "Shipping", "--seed", "--database=testing"]),
                &failed(None)
            ),
            DrawerAction::DryRun {
                module: Some("Shipping".to_owned()),
                database: Some("testing".to_owned()),
            }
        );
        assert_eq!(
            dry_run(
                &words(&["module:migrate", "--all"]),
                &failed(Some("Shipping"))
            ),
            DrawerAction::DryRun {
                module: None,
                database: None,
            }
        );
        assert_eq!(
            dry_run(&words(&["module:migrate", "--", "-Odd"]), &failed(None)),
            DrawerAction::DryRun {
                module: Some("-Odd".to_owned()),
                database: None,
            }
        );
    }
}
