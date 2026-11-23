//! "Run PHP with" (S1.3, S8.2): how PHP runs for one app, and its Test. The same
//! field is in Settings and in the add panel.
//!
//! The kind follows the text (design D8). The command field can be edited for every
//! kind: when it loses focus, [`RunnerKind::from_line`] reads the kind from it.
//! Picking a kind writes its words into the field.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};

use eframe::egui::{self, Align2, Pos2, Rect, Role, Ui, Vec2, pos2, vec2};
use mortise_core::{Detected, RunnerKind, TEST_LIMIT, TestOutcome, detect, herd_config_dir};

use crate::draw::{self, Button};
use crate::strings;
use crate::theme::{Palette, Type, size, ty};

/// A test that ended, and the runner it tested. Mortise keeps the last one of each
/// app in memory only (design D8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tested {
    pub runner: RunnerKind,
    pub outcome: TestOutcome,
}

/// `artisan --version` on a worker thread, or what came of it.
#[derive(Debug)]
enum Test {
    Running(RunnerKind, Receiver<TestOutcome>),
    Done(Tested),
}

/// What the field holds: the text, the kind the text gives, and the test.
#[derive(Debug)]
pub struct PhpForm {
    /// The text of the command field.
    pub line: String,
    /// The kind of [`PhpForm::line`], read when the field loses focus or a kind is picked.
    pub runner: RunnerKind,
    test: Option<Test>,
}

/// What happened in the field in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhpEvent {
    /// Another kind was picked, or the field was left with another kind in it.
    Changed,
    /// Test was pressed. The caller starts it with [`PhpForm::start_test`] and the
    /// folder the command starts in.
    Test,
}

#[derive(Debug, Clone, Copy)]
pub struct PhpOptions<'a> {
    pub keyboard: bool,
    /// The hint after the label, such as what detection found. Empty for none.
    pub hint: &'a str,
    /// Tells the fields of this form from those of another on the same screen.
    pub salt: &'a str,
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
            test: None,
        }
    }

    /// The form with the result of an earlier test shown under the field.
    pub fn with_last(mut self, last: Option<Tested>) -> Self {
        self.test = last.map(Test::Done);
        self
    }

    /// Runs `artisan --version` through the runner of the form in `folder`, on a
    /// worker thread. The window is drawn again when it ends.
    pub fn start_test(&mut self, folder: PathBuf, ctx: &egui::Context) {
        let (sender, receiver) = channel();
        let runner = self.runner.clone();
        let tested = runner.clone();
        let ctx = ctx.clone();
        std::thread::Builder::new()
            .name("mortise-test-php".to_owned())
            .spawn(move || {
                let _ = sender.send(mortise_core::test_php(&runner, &folder, TEST_LIMIT));
                ctx.request_repaint();
            })
            .expect("a thread can be started");
        self.test = Some(Test::Running(tested, receiver));
    }

    pub fn running(&self) -> bool {
        matches!(self.test, Some(Test::Running(..)))
    }

    /// Takes the outcome of a running test once it is there.
    pub fn poll(&mut self) {
        if let Some(Test::Running(runner, receiver)) = &self.test
            && let Ok(outcome) = receiver.try_recv()
        {
            let runner = runner.clone();
            self.test = Some(Test::Done(Tested { runner, outcome }));
        }
    }

    /// The last test that ended.
    pub fn last(&self) -> Option<&Tested> {
        match &self.test {
            Some(Test::Done(tested)) => Some(tested),
            _ => None,
        }
    }

    /// Picks the kind at `place` of [`strings::RUNNERS`] and writes its words into the field.
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
/// with Test beside it, and under it the result. Returns its bottom edge and what happened.
pub fn php_field(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    form: &mut PhpForm,
    options: PhpOptions<'_>,
) -> (f32, Vec<PhpEvent>) {
    let PhpOptions {
        keyboard,
        hint,
        salt,
    } = options;
    let mut events = Vec::new();
    let left = min.x;
    let name = format!("{salt}{}", strings::RUN_PHP_WITH);

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
    let place = strings::runner_place(&form.runner);
    let (track, clicked) = draw::segmented_with(
        ui,
        pal,
        pos2(left, y),
        &name,
        &strings::RUNNERS,
        place,
        off,
        keyboard,
    );
    if let Some(clicked) = clicked.filter(|clicked| *clicked != place) {
        form.pick(clicked);
        events.push(PhpEvent::Changed);
    }
    y = track.bottom() + 6.0;

    let test = Button {
        label: strings::TEST,
        said: Some(strings::TEST_PHP),
        disabled: form.running(),
        ..Default::default()
    };
    let field = Rect::from_min_size(
        pos2(left, y),
        vec2(width - test.width(ui) - 8.0, size::FIELD_H),
    );
    let left_field =
        draw::text_field(ui, pal, field, &name, &mut form.line, true, keyboard).lost_focus();
    let test_at = pos2(
        field.right() + 8.0,
        field.top() + (size::FIELD_H - size::CONTROL_H) / 2.0,
    );
    let pressed = test.show(ui, pal, test_at, keyboard).clicked();
    if left_field || pressed {
        let now = RunnerKind::from_line(&form.line);
        if now != form.runner {
            form.runner = now;
            events.push(PhpEvent::Changed);
        }
    }
    if pressed {
        events.push(PhpEvent::Test);
    }
    y = field.bottom();

    // The result: the first line of the answer, or why there was none.
    form.poll();
    match &form.test {
        None => {}
        Some(Test::Running(..)) => {
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
        }
        Some(Test::Done(tested)) => {
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
    }
    (y, events)
}
