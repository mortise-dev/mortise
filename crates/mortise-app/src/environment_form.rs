//! The form of one environment (S8.5): a name, a base URL, the actuator path and
//! one of four kinds of credentials. The same form is in Settings and in the add panel.
//!
//! A secret is drawn as dots while it is typed and is never drawn afterwards.

use std::sync::mpsc::{Receiver, channel};

use eframe::egui::{self, Align2, Pos2, Rect, Ui, Vec2, pos2, vec2};
use mortise_core::{Answer, Credentials, Environment, Secret, ask};

use crate::draw::{self, Button, Icon};
use crate::runtime_view::Brief;
use crate::strings;
use crate::theme::{Palette, radius, size, ty};

/// A test of the connection: one request for everything, on a worker thread.
pub enum Test {
    Running(Receiver<Answer>),
    Done(Answer),
}

pub struct EnvironmentForm {
    pub name: String,
    pub base_url: String,
    pub path: String,
    /// The kind of credentials, as a place in [`strings::CREDENTIAL_KINDS`].
    pub kind: usize,
    pub user: String,
    pub header: String,
    /// What is typed in the secret field. Drawn as dots.
    pub secret: String,
    /// A secret is kept for this environment: the field says so and shows nothing.
    pub saved: bool,
    /// "Replace" was pressed: the field is empty and waits for a new value.
    replacing: bool,
    /// The secret field takes focus in the next frame.
    focus_secret: bool,
    /// No credential store is running: the secret is kept until Mortise closes.
    pub until_close: bool,
    pub test: Option<Test>,
}

impl std::fmt::Debug for EnvironmentForm {
    /// The secret is left out: it is never written anywhere, not even to a log.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnvironmentForm")
            .field("environment", &self.environment())
            .field("saved", &self.saved)
            .finish_non_exhaustive()
    }
}

/// What happened in the form in this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormEvent {
    /// A field that is not a secret was left, or another kind was chosen.
    Changed,
    /// A secret was entered, to be kept.
    Secret(String),
    /// The kept secret is to be removed.
    RemoveSecret,
    /// Ask the environment now.
    Test,
}

#[derive(Debug, Clone, Copy)]
pub struct FormOptions<'a> {
    pub keyboard: bool,
    /// The base URL was proposed from `APP_URL`: the label says so.
    pub from_env: bool,
    /// In the add panel a secret stays in the field, as dots, until the app is added.
    pub keep_typed: bool,
    /// Tells the fields of this form from those of another on the same screen.
    pub salt: &'a str,
}

impl EnvironmentForm {
    /// The form of a new environment.
    pub fn new(name: &str, base_url: &str) -> Self {
        Self::of(
            &Environment {
                name: name.to_owned(),
                base_url: base_url.to_owned(),
                path: "actuator".to_owned(),
                credentials: Credentials::None,
            },
            false,
        )
    }

    /// The form of an environment Mortise knows. `saved` says that it has a secret.
    pub fn of(environment: &Environment, saved: bool) -> Self {
        let (kind, user, header) = match &environment.credentials {
            Credentials::None => (0, "", ""),
            Credentials::Bearer => (1, "", ""),
            Credentials::Basic { user } => (2, user.as_str(), ""),
            Credentials::Header { name } => (3, "", name.as_str()),
        };
        Self {
            name: environment.name.clone(),
            base_url: environment.base_url.clone(),
            path: environment.path.clone(),
            kind,
            user: user.to_owned(),
            header: header.to_owned(),
            secret: String::new(),
            saved,
            replacing: false,
            focus_secret: false,
            until_close: false,
            test: None,
        }
    }

    /// The environment as the fields have it.
    pub fn environment(&self) -> Environment {
        let credentials = match self.kind {
            1 => Credentials::Bearer,
            2 => Credentials::Basic {
                user: self.user.trim().to_owned(),
            },
            3 => Credentials::Header {
                name: self.header.trim().to_owned(),
            },
            _ => Credentials::None,
        };
        Environment {
            name: self.name.trim().to_owned(),
            base_url: self.base_url.trim().to_owned(),
            path: self.path.trim().trim_matches('/').to_owned(),
            credentials,
        }
    }

    /// Asks the environment of the fields, with this secret. The answer is shown
    /// under the form when it is there.
    pub fn start_test(&mut self, secret: Secret, ctx: &egui::Context) {
        let (sender, receiver) = channel();
        let environment = self.environment();
        let ctx = ctx.clone();
        std::thread::Builder::new()
            .name("mortise-test".to_owned())
            .spawn(move || {
                let _ = sender.send(ask(&environment, &secret));
                ctx.request_repaint();
            })
            .expect("a thread can be started");
        self.test = Some(Test::Running(receiver));
    }

    /// The answer of the last test, once it is there.
    pub fn tested(&mut self) -> Option<&Answer> {
        if let Some(Test::Running(receiver)) = &self.test
            && let Ok(answer) = receiver.try_recv()
        {
            self.test = Some(Test::Done(answer));
        }
        match &self.test {
            Some(Test::Done(answer)) => Some(answer),
            _ => None,
        }
    }
}

/// A label over a field. Returns the y of the field.
fn label(ui: &Ui, pal: &Palette, at: Pos2, text: &str, hint: &str) -> f32 {
    let rect = draw::label(ui, at, Align2::LEFT_TOP, text, ty::LABEL, pal.ink);
    if !hint.is_empty() {
        let at = pos2(rect.right() + 4.0, at.y);
        draw::label(ui, at, Align2::LEFT_TOP, hint, ty::SECONDARY, pal.ink_2);
    }
    at.y + ty::LABEL.line + 4.0
}

/// Paints the form with its top left at `min`. Returns its bottom edge and what happened.
pub fn environment_form(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    form: &mut EnvironmentForm,
    options: FormOptions<'_>,
) -> (f32, Vec<FormEvent>) {
    let mut events = Vec::new();
    let FormOptions {
        keyboard,
        from_env,
        keep_typed,
        salt,
    } = options;
    let field_name = |name: &str| format!("{salt}{name}");
    let left = min.x;
    let mut y = min.y;

    // Name, base URL and path, side by side.
    let side = 110.0_f32.min((width - 24.0) / 4.0);
    let url_w = width - 2.0 * side - 16.0;
    let fields_y = label(ui, pal, pos2(left, y), strings::NAME, "");
    let hint = if from_env { strings::BASE_URL_HINT } else { "" };
    label(ui, pal, pos2(left + side + 8.0, y), strings::BASE_URL, hint);
    label(
        ui,
        pal,
        pos2(left + side + 8.0 + url_w + 8.0, y),
        strings::ACTUATOR_PATH,
        "",
    );
    let name_rect = Rect::from_min_size(pos2(left, fields_y), vec2(side, size::FIELD_H));
    let url_rect = Rect::from_min_size(
        pos2(name_rect.right() + 8.0, fields_y),
        vec2(url_w, size::FIELD_H),
    );
    let path_rect = Rect::from_min_size(
        pos2(url_rect.right() + 8.0, fields_y),
        vec2(side, size::FIELD_H),
    );
    let mut left_a_field = false;
    left_a_field |= draw::text_field(
        ui,
        pal,
        name_rect,
        &field_name("Environment name"),
        &mut form.name,
        false,
        keyboard,
    )
    .lost_focus();
    left_a_field |= draw::text_field(
        ui,
        pal,
        url_rect,
        &field_name(strings::BASE_URL),
        &mut form.base_url,
        true,
        keyboard,
    )
    .lost_focus();
    left_a_field |= draw::text_field(
        ui,
        pal,
        path_rect,
        &field_name(strings::ACTUATOR_PATH),
        &mut form.path,
        true,
        keyboard,
    )
    .lost_focus();
    y = name_rect.bottom() + 12.0;

    // The kind of credentials: one control, and under it the fields of that kind.
    let hint = if keep_typed {
        strings::CREDENTIALS_HINT
    } else {
        ""
    };
    let kinds_y = label(ui, pal, pos2(left, y), strings::CREDENTIALS, hint);
    let (track, clicked) = draw::segmented(
        ui,
        pal,
        pos2(left, kinds_y),
        &field_name(strings::CREDENTIALS),
        &strings::CREDENTIAL_KINDS,
        form.kind,
        keyboard,
    );
    if let Some(kind) = clicked.filter(|kind| *kind != form.kind) {
        form.kind = kind;
        form.secret.clear();
        form.replacing = false;
        events.push(FormEvent::Changed);
        // A secret of another kind is of no use.
        if form.saved {
            form.saved = false;
            events.push(FormEvent::RemoveSecret);
        }
    }
    y = track.bottom() + 12.0;

    let test = Button {
        label: strings::TEST,
        ..Default::default()
    };
    let test_w = test.width(ui);
    let test_at;
    if form.kind == 0 {
        let text = draw::paragraph(
            ui,
            pos2(left, y - 4.0),
            strings::NO_CREDENTIALS,
            ty::SECONDARY,
            pal.ink_2,
            width,
        );
        y = text.bottom() + 10.0;
        test_at = pos2(left, y);
        y += size::CONTROL_H;
    } else {
        // A readable field for what is not a secret, then the secret.
        let fields_w = width - test_w - 8.0;
        let (plain, secret_label, secret_hint): (Option<(&str, &mut String)>, &str, &str) =
            match form.kind {
                1 => (None, strings::TOKEN, strings::TOKEN_HINT),
                2 => (
                    Some((strings::USER_NAME, &mut form.user)),
                    strings::PASSWORD,
                    "",
                ),
                _ => (
                    Some((strings::HEADER_NAME, &mut form.header)),
                    strings::HEADER_VALUE,
                    "",
                ),
            };
        let secret_x = match plain {
            Some((plain_label, text)) => {
                let half = ((fields_w - 8.0) / 2.0).floor();
                let field_y = label(ui, pal, pos2(left, y), plain_label, "");
                let rect = Rect::from_min_size(pos2(left, field_y), vec2(half, size::FIELD_H));
                left_a_field |= draw::text_field(
                    ui,
                    pal,
                    rect,
                    &field_name(plain_label),
                    text,
                    true,
                    keyboard,
                )
                .lost_focus();
                rect.right() + 8.0
            }
            None => left,
        };
        let field_y = label(ui, pal, pos2(secret_x, y), secret_label, secret_hint);
        let rect = Rect::from_min_max(
            pos2(secret_x, field_y),
            pos2(left + fields_w, field_y + size::FIELD_H),
        );
        if form.saved && !form.replacing {
            // Never drawn again, not even masked by length.
            draw::outlined(ui, rect, radius::CONTROL, pal.well, pal.line);
            let lock = Rect::from_min_size(
                pos2(rect.left() + 9.0, rect.center().y - 6.0),
                Vec2::splat(12.0),
            );
            draw::icon(ui, lock, Icon::Lock, pal.ink_2);
            draw::label(
                ui,
                pos2(lock.right() + 6.0, rect.center().y),
                Align2::LEFT_CENTER,
                strings::SAVED_NOT_SHOWN,
                ty::BODY,
                pal.ink_2,
            );
            let remove = Button {
                label: strings::REMOVE,
                small: true,
                ..Default::default()
            };
            let replace = Button {
                label: strings::REPLACE,
                small: true,
                ..Default::default()
            };
            let button_y = rect.center().y - remove.height() / 2.0;
            let remove_x = rect.right() - 3.0 - remove.width(ui);
            let replace_x = remove_x - 6.0 - replace.width(ui);
            if replace
                .show(ui, pal, pos2(replace_x, button_y), keyboard)
                .clicked()
            {
                form.replacing = true;
                form.focus_secret = true;
            }
            if remove
                .show(ui, pal, pos2(remove_x, button_y), keyboard)
                .clicked()
            {
                form.saved = false;
                events.push(FormEvent::RemoveSecret);
            }
        } else {
            let response = draw::secret_field(
                ui,
                pal,
                rect,
                &field_name(secret_label),
                &mut form.secret,
                keyboard,
            );
            if std::mem::take(&mut form.focus_secret) {
                response.request_focus();
            }
            if response.lost_focus() && !keep_typed {
                if !form.secret.is_empty() {
                    events.push(FormEvent::Secret(std::mem::take(&mut form.secret)));
                    form.saved = true;
                }
                form.replacing = false;
            }
        }
        test_at = pos2(
            left + fields_w + 8.0,
            field_y + (size::FIELD_H - size::CONTROL_H) / 2.0,
        );
        y = rect.bottom();
    }
    let running = matches!(form.test, Some(Test::Running(_)));
    let button = Button {
        disabled: running || form.base_url.trim().is_empty(),
        ..test
    };
    if button.show(ui, pal, test_at, keyboard).clicked() {
        events.push(FormEvent::Test);
    }
    // The answer of the test: a runtime mark and its word, then what it means. It
    // stands beside the button, or on a line of its own under the secret.
    if form.test.is_some() {
        let at = if form.kind == 0 {
            pos2(test_at.x + test_w + 10.0, test_at.y + size::CONTROL_H / 2.0)
        } else {
            y += 8.0 + 16.0;
            pos2(left, y - 8.0)
        };
        match form.tested() {
            Some(answer) => {
                let brief = Brief::of_answer(answer);
                let color = brief.tone.color(pal);
                let mark =
                    Rect::from_min_size(pos2(at.x, at.y - 6.0), Vec2::splat(size::STATE_MARK));
                draw::runtime_mark(ui, mark, brief.mark, color);
                let word_at = pos2(mark.right() + 5.0, at.y);
                let word = draw::label(
                    ui,
                    word_at,
                    Align2::LEFT_CENTER,
                    brief.word,
                    ty::LABEL,
                    color,
                );
                let text = match answer {
                    Answer::Answered(runtime) => strings::exposed(&runtime.exposed),
                    Answer::NoAnswer { url, message } => format!("{url}: {message}."),
                    Answer::NotAuthorized { url, status } | Answer::Unexpected { url, status } => {
                        format!("{url} answered {status}.")
                    }
                };
                let room = left + width - (word.right() + 8.0);
                let galley = draw::galley_elided(ui, &text, ty::SECONDARY, pal.ink_2, room);
                let text_at = pos2(word.right() + 8.0, at.y);
                let text_rect = draw::paint_galley(ui, text_at, Align2::LEFT_CENTER, galley);
                draw::announce(ui, text_rect, egui::Role::Label, &text);
            }
            None => {
                draw::label(
                    ui,
                    at,
                    Align2::LEFT_CENTER,
                    strings::ASKING,
                    ty::SECONDARY,
                    pal.ink_2,
                );
            }
        }
    }
    if form.kind != 0 {
        // Where the secret goes.
        y += 8.0;
        let lock = Rect::from_min_size(pos2(left, y + 1.5), Vec2::splat(12.0));
        draw::icon(ui, lock, Icon::Lock, pal.ink_2);
        let sentence = if form.until_close {
            strings::NO_STORE
        } else {
            strings::KEPT_IN_STORE
        };
        let color = if form.until_close {
            pal.warn
        } else {
            pal.ink_2
        };
        let text = draw::paragraph(
            ui,
            pos2(lock.right() + 6.0, y),
            sentence,
            ty::SECONDARY,
            color,
            width - 18.0,
        );
        y = text.bottom();
    }
    if left_a_field {
        events.push(FormEvent::Changed);
    }
    (y, events)
}
