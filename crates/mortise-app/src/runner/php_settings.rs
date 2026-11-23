//! "Run PHP with" (S1.3, S8.2): how PHP runs for one app, and its Test. The same
//! field is in Settings and in the add panel.
//!
//! The kind follows the text (design D8). The command field can be edited for every
//! kind: when it loses focus, [`RunnerKind::from_line`] reads the kind from it.
//! Picking a kind writes its words into the field.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};

use eframe::egui::{self, Align2, Pos2, Rect, Role, Ui, Vec2, pos2, vec2};
use mortise_core::{
    CommandLine, Detected, RunnerKind, TEST_LIMIT, TestOutcome, detect, herd_config_dir,
};

use crate::runner::strings;
use crate::ui::draw::{self, Button};
use crate::ui::theme::{Palette, Type, size, ty};

/// A test that ended, and the runner and folder it tested. Mortise keeps the last
/// one of each app in memory only (design D8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tested {
    pub runner: RunnerKind,
    pub folder: PathBuf,
    pub outcome: TestOutcome,
}

/// The test of how PHP runs for one app: the one on a worker thread, and the last
/// one that ended. It lives with the app, so closing Settings does not drop it.
#[derive(Debug, Default)]
pub struct PhpTest {
    running: Option<(RunnerKind, PathBuf, Receiver<TestOutcome>)>,
    last: Option<Tested>,
}

impl PhpTest {
    /// Runs `artisan --version` through `runner` in `folder`, on a worker thread.
    /// The window is drawn again when it ends.
    pub fn start(&mut self, runner: &RunnerKind, folder: PathBuf, ctx: &egui::Context) {
        let (sender, receiver) = channel();
        let worker = (runner.clone(), folder.clone(), ctx.clone());
        std::thread::Builder::new()
            .name("mortise-test-php".to_owned())
            .spawn(move || {
                let (runner, folder, ctx) = worker;
                let _ = sender.send(mortise_core::test_php(&runner, &folder, TEST_LIMIT));
                ctx.request_repaint();
            })
            .expect("a thread can be started");
        self.running = Some((runner.clone(), folder, receiver));
    }

    pub fn running(&self) -> bool {
        self.running.is_some()
    }

    /// Takes the outcome of a running test once it is there.
    pub fn poll(&mut self) {
        if let Some((_, _, receiver)) = &self.running
            && let Ok(outcome) = receiver.try_recv()
            && let Some((runner, folder, _)) = self.running.take()
        {
            self.last = Some(Tested {
                runner,
                folder,
                outcome,
            });
        }
    }

    /// The last test that ended.
    pub fn last(&self) -> Option<&Tested> {
        self.last.as_ref()
    }

    /// The last test that ended, when it tested `runner` in `folder`. A result is
    /// not shown under a runner or a folder it did not test.
    pub fn last_of(&self, runner: &RunnerKind, folder: &Path) -> Option<&Tested> {
        self.last
            .as_ref()
            .filter(|tested| tested.runner == *runner && tested.folder == folder)
    }
}

/// What the field holds: the text, and the kind the text gives.
#[derive(Debug)]
pub struct PhpForm {
    /// The text of the command field.
    pub line: String,
    /// The kind of [`PhpForm::line`], read when the field loses focus or a kind is picked.
    pub runner: RunnerKind,
}

/// What happened in the field in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhpEvent {
    /// Another kind was picked, or the field was left with another kind in it.
    Changed,
    /// Test was pressed. The caller starts it with [`PhpTest::start`], the runner
    /// of the form and the folder the command starts in.
    Test,
}

#[derive(Debug, Clone, Copy)]
pub struct PhpOptions<'a> {
    pub keyboard: bool,
    /// The hint after the label, such as what detection found. Empty for none.
    pub hint: &'a str,
    /// The folder a test starts in now. A result shows only for this folder.
    pub folder: &'a Path,
    /// Tells the controls of this form from those of another form, also of another
    /// app. The names that are read out stay the same.
    pub salt: egui::IdSalt,
}

/// The runner that detection proposes for a folder, and the hint that says why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    pub runner: RunnerKind,
    pub hint: &'static str,
}

/// Proposes a runner for the app in `root`. Reads files only. When nothing is found,
/// Herd is shown, as S1.4 c draws it.
pub fn propose(root: &Path) -> Proposal {
    let found = detect(root, herd_config_dir().as_deref()).first().copied();
    let runner = match found {
        Some(Detected::Sail) => RunnerKind::Sail,
        Some(Detected::Docker) => RunnerKind::Docker {
            service: "app".to_owned(),
        },
        Some(Detected::Herd) | None => RunnerKind::Herd,
    };
    Proposal {
        runner,
        hint: strings::php_found(found),
    }
}

impl PhpForm {
    pub fn new(runner: RunnerKind) -> Self {
        Self {
            line: runner.line(),
            runner,
        }
    }

    /// Reads the kind from the text, as when the field loses focus. True when the
    /// kind changed.
    pub fn commit(&mut self) -> bool {
        let now = RunnerKind::from_line(&self.line);
        let changed = now != self.runner;
        self.runner = now;
        changed
    }

    /// Picks the kind at `place` of [`crate::strings::RUNNERS`] and writes its words into the field.
    fn pick(&mut self, place: usize) {
        self.runner = match place {
            0 => RunnerKind::Herd,
            1 => RunnerKind::Sail,
            2 => RunnerKind::Docker {
                service: "app".to_owned(),
            },
            // Words that read as Custom, so that the kind stays when the field is left.
            _ => RunnerKind::Custom {
                line: "php artisan".to_owned(),
            },
        };
        self.line = self.runner.line();
    }
}

/// Paints the field with its top left at `min`: the label, the kinds, the command
/// with Test beside it, the line that Test runs, and under it the result. Returns its
/// bottom edge and what happened.
pub fn php_field(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    form: &mut PhpForm,
    test: &mut PhpTest,
    options: PhpOptions<'_>,
) -> (f32, Vec<PhpEvent>) {
    ui.push_id(options.salt, |ui| {
        paint(ui, pal, min, width, form, test, options)
    })
    .inner
}

fn paint(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    form: &mut PhpForm,
    test: &mut PhpTest,
    options: PhpOptions<'_>,
) -> (f32, Vec<PhpEvent>) {
    let PhpOptions {
        keyboard,
        hint,
        folder,
        ..
    } = options;
    let mut events = Vec::new();
    let left = min.x;
    let name = strings::RUN_PHP_WITH;

    let label = draw::label(
        ui,
        min,
        Align2::LEFT_TOP,
        strings::RUN_PHP_WITH,
        ty::LABEL,
        pal.ink,
    );
    if !hint.is_empty() {
        let at = pos2(label.right() + 4.0, min.y);
        draw::label(ui, at, Align2::LEFT_TOP, hint, ty::SECONDARY, pal.ink_2);
    }
    let mut y = min.y + ty::LABEL.line + 4.0;

    // Sail is a bash script: on Windows it runs in WSL only (design D6).
    let off = cfg!(windows).then_some((1, strings::SAIL_NOT_ON_WINDOWS));
    let place = crate::strings::runner_place(&form.runner);
    let (track, clicked) = draw::segmented_with(
        ui,
        pal,
        pos2(left, y),
        name,
        &crate::strings::RUNNERS,
        place,
        off,
        keyboard,
    );
    if let Some(clicked) = clicked.filter(|clicked| *clicked != place) {
        form.pick(clicked);
        events.push(PhpEvent::Changed);
    }
    y = track.bottom() + 6.0;

    test.poll();
    let button = Button {
        label: crate::strings::TEST,
        said: Some(strings::TEST_PHP),
        disabled: test.running(),
        ..Default::default()
    };
    let field = Rect::from_min_size(
        pos2(left, y),
        vec2(width - button.width(ui) - 8.0, size::FIELD_H),
    );
    let left_field =
        draw::text_field(ui, pal, field, name, &mut form.line, true, keyboard).lost_focus();
    let test_at = pos2(
        field.right() + 8.0,
        field.top() + (size::FIELD_H - size::CONTROL_H) / 2.0,
    );
    let pressed = button.show(ui, pal, test_at, keyboard).clicked();
    if (left_field || pressed) && form.commit() {
        events.push(PhpEvent::Changed);
    }
    if pressed {
        events.push(PhpEvent::Test);
    }
    y = field.bottom();

    // The line that Test runs, quoted as a shell of this platform reads it: a command
    // runs only after its exact line is on screen. The runner is read from the text
    // as Test reads it when pressed. An empty Custom line shows nothing, and Test
    // then says that no command was entered.
    let runner = RunnerKind::from_line(&form.line);
    if let Some(line) = CommandLine::artisan(&runner, &["--version"]) {
        y += 6.0;
        let line = line.display();
        let code = Type {
            size: 11.5,
            ..ty::CODE
        };
        let parts = [
            (strings::TEST_RUNS, ty::SECONDARY, pal.ink_2),
            (" ", ty::SECONDARY, pal.ink_2),
            (line.as_str(), code, pal.ink),
        ];
        let galley = draw::rich_galley(ui, &parts, width);
        let text = draw::paint_galley(ui, pos2(left, y), Align2::LEFT_TOP, galley);
        draw::announce(ui, text, Role::Label, &strings::test_runs(&line));
        y = text.bottom();
    }

    // The result: the first line of the answer, or why there was none. A result
    // shows only under the runner and the folder it tested.
    if test.running() {
        y += 6.0;
        let line = draw::label(
            ui,
            pos2(left, y),
            Align2::LEFT_TOP,
            strings::TESTING_PHP,
            ty::SECONDARY,
            pal.ink_2,
        );
        y = line.bottom();
    } else if let Some(tested) = test.last_of(&runner, folder) {
        y += 6.0;
        let mark = Rect::from_min_size(pos2(left, y + 2.0), Vec2::splat(size::STATE_MARK));
        match &tested.outcome {
            TestOutcome::Answered(answer) => {
                draw::mark_good(ui, mark, pal.good);
                let mid = mark.center().y;
                let word = draw::label(
                    ui,
                    pos2(mark.right() + 5.0, mid),
                    Align2::LEFT_CENTER,
                    strings::ANSWERED,
                    ty::LABEL,
                    pal.good,
                );
                let code = Type {
                    size: 11.5,
                    ..ty::CODE
                };
                let room = left + width - (word.right() + 8.0);
                let galley = draw::galley_elided(ui, answer, code, pal.ink_2, room);
                let at = pos2(word.right() + 8.0, mid);
                let text = draw::paint_galley(ui, at, Align2::LEFT_CENTER, galley);
                draw::announce(ui, text, Role::Label, answer);
                y = mark.bottom().max(word.bottom());
            }
            TestOutcome::Failed(failure) => {
                // The sentence at the field: what happened, then what to do.
                draw::mark_bad(ui, mark, pal.bad, pal.surface);
                let sentence = strings::php_failed(&tested.runner, failure);
                let sentence_type = Type {
                    line: 16.0,
                    ..ty::SECONDARY
                };
                let text = draw::paragraph(
                    ui,
                    pos2(mark.right() + 6.0, y),
                    &sentence,
                    sentence_type,
                    pal.bad,
                    width - size::STATE_MARK - 6.0,
                );
                y = text.bottom();
            }
        }
    }
    (y, events)
}
