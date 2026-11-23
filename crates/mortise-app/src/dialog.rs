//! The dialog (C14): a confirmation before something is lost or before others are
//! affected. Forms live in the workspace, not here.

use eframe::egui::{self, Align2, Key, Modifiers, Rect, Role, Sense, Shadow, Ui, vec2};
use mortise_core::Severity;

use crate::draw::{self, Button, ButtonKind};
use crate::summary::alert_box;
use crate::theme::{Palette, Type, radius, size, ty};

/// What a dialog asks.
#[derive(Debug, Clone)]
pub struct Dialog {
    pub title: String,
    pub body: String,
    /// The label of the button that goes ahead. An action keeps one name from the
    /// button to the confirmation to the message.
    pub confirm: String,
    /// The label of the quiet choice, "Cancel" unless the dialog names it.
    pub cancel: String,
    /// Focus starts on the quiet choice, as for a stop that can harm the database.
    pub start_on_cancel: bool,
    /// The button that goes ahead is the filled red one: something is lost. Enter
    /// never presses it.
    pub danger: bool,
    /// A quiet button on the left, such as "Dry run".
    pub left: Option<String>,
    /// A warning in an alert under the body, such as the files of a module that are
    /// not committed (S10.5 d).
    pub alert: Option<String>,
    /// A command line in the mono code style: the line the dialog runs (S10.5 c).
    pub code: Option<String>,
    /// Short lines under the body, such as "This command has no dry run."
    pub lines: Vec<String>,
    /// A name to type before the button that goes ahead turns on (S10.5 c and d).
    pub typed: Option<Typed>,
}

/// A name the user types to confirm, and what is typed so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typed {
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Confirm,
    Cancel,
    /// The button on the left.
    Left,
}

/// The type of the lines under the body.
const HINT: Type = Type {
    line: 16.0,
    ..ty::SECONDARY
};

impl Dialog {
    /// A dialog with "Cancel" as the quiet choice, which starts on the button that goes ahead.
    pub fn new(title: String, body: String, confirm: String) -> Self {
        Self {
            title,
            body,
            confirm,
            cancel: crate::strings::CANCEL.to_owned(),
            start_on_cancel: false,
            danger: false,
            left: None,
            alert: None,
            code: None,
            lines: Vec::new(),
            typed: None,
        }
    }

    /// True when the button that goes ahead can be pressed: the typed name, if the
    /// dialog asks for one, matches exactly.
    pub fn can_confirm(&self) -> bool {
        self.typed
            .as_ref()
            .is_none_or(|typed| mortise_core::typed_name_matches(&typed.text, &typed.name))
    }

    /// Paints the dialog over a scrim, 440 px wide. It holds focus until it closes.
    /// `opened` is true in the frame it opens: a confirmation that loses nothing
    /// starts on the button that goes ahead, else on the quiet choice.
    pub fn show(&mut self, ui: &Ui, pal: &Palette, opened: bool, keyboard: bool) -> Option<Answer> {
        let id = egui::Id::unique("dialog");
        let area = egui::Modal::default_area(id)
            .anchor(Align2::CENTER_TOP, vec2(0.0, size::TITLEBAR_H + 96.0));
        let mut answer = None;
        let can_confirm = self.can_confirm();
        let modal = egui::Modal::new(id)
            .frame(egui::Frame::NONE)
            .backdrop_color(pal.scrim)
            .area(area)
            .show(ui.ctx(), |ui| {
                let text_w = size::DIALOG_W - 34.0;
                let body = draw::galley_wrapped(ui, &self.body, ty::BODY, pal.ink_2, text_w);
                // The parts under the body, each 10 px below the one before.
                let alert_h = self.alert.as_ref().map_or(0.0, |alert| {
                    let at = egui::pos2(0.0, 0.0);
                    let (bottom, _) = alert_box(
                        ui,
                        pal,
                        at,
                        text_w,
                        Severity::Warning,
                        "",
                        alert,
                        None,
                        false,
                    );
                    10.0 + bottom
                });
                let code_h = self
                    .code
                    .as_ref()
                    .map_or(0.0, |code| 10.0 + draw::code_block_height(ui, code, text_w));
                let lines: Vec<_> = self
                    .lines
                    .iter()
                    .map(|line| draw::galley_wrapped(ui, line, HINT, pal.ink_2, text_w))
                    .collect();
                let lines_h: f32 = lines.iter().map(|galley| 8.0 + galley.size().y).sum();
                let typed_h = self
                    .typed
                    .as_ref()
                    .map_or(0.0, |_| 12.0 + ty::LABEL.line + 4.0 + size::FIELD_H);
                let foot_h = 10.0 + size::CONTROL_H + 10.0;
                let height = 1.0
                    + 14.0
                    + ty::OVERLAY_TITLE.line
                    + 8.0
                    + body.size().y
                    + alert_h
                    + code_h
                    + lines_h
                    + typed_h
                    + 12.0
                    + foot_h
                    + 1.0;
                let (rect, _) =
                    ui.allocate_exact_size(vec2(size::DIALOG_W, height), Sense::hover());

                let shadow = Shadow {
                    offset: [0, 14],
                    blur: 34,
                    spread: 0,
                    color: pal.shadow,
                };
                ui.painter().add(shadow.as_shape(rect, radius::OVERLAY));
                draw::outlined(ui, rect, radius::OVERLAY, pal.surface, pal.ink);

                let left = rect.left() + 17.0;
                let title = draw::paint_text(
                    ui,
                    rect.min + vec2(17.0, 15.0),
                    Align2::LEFT_TOP,
                    &self.title,
                    ty::OVERLAY_TITLE,
                    pal.ink,
                );
                draw::announce(ui, title, Role::Heading, &self.title);
                let body_rect = draw::paint_galley(
                    ui,
                    egui::pos2(left, title.bottom() + 8.0),
                    Align2::LEFT_TOP,
                    body,
                );
                draw::announce(ui, body_rect, Role::Label, &self.body);
                let mut y = body_rect.bottom();
                if let Some(alert) = &self.alert {
                    let at = egui::pos2(left, y + 10.0);
                    y = alert_box(
                        ui,
                        pal,
                        at,
                        text_w,
                        Severity::Warning,
                        "",
                        alert,
                        None,
                        true,
                    )
                    .0;
                }
                if let Some(code) = &self.code {
                    y = draw::code_block(ui, pal, egui::pos2(left, y + 10.0), text_w, code)
                        .bottom();
                }
                for (galley, text) in lines.into_iter().zip(&self.lines) {
                    let at =
                        draw::paint_galley(ui, egui::pos2(left, y + 8.0), Align2::LEFT_TOP, galley);
                    draw::announce(ui, at, Role::Label, text);
                    y = at.bottom();
                }
                if let Some(typed) = &mut self.typed {
                    let label = crate::strings::type_to_confirm(&typed.name);
                    let shown = draw::paint_text(
                        ui,
                        egui::pos2(left, y + 12.0),
                        Align2::LEFT_TOP,
                        &label,
                        ty::LABEL,
                        pal.ink,
                    );
                    let field = Rect::from_min_size(
                        egui::pos2(left, shown.top() + ty::LABEL.line + 4.0),
                        vec2(text_w, size::FIELD_H),
                    );
                    draw::text_field(ui, pal, field, &label, &mut typed.text, false, keyboard);
                }

                // Footer: the button on the left, the quiet choice, then the one that goes ahead.
                let foot = Rect::from_min_max(
                    egui::pos2(rect.left() + 1.0, rect.bottom() - 1.0 - foot_h),
                    egui::pos2(rect.right() - 1.0, rect.bottom() - 1.0),
                );
                ui.painter().rect_filled(foot, 0, pal.panel);
                draw::hline(ui, foot.left()..foot.right(), foot.top(), pal.line);
                let confirm = Button {
                    label: &self.confirm,
                    kind: if self.danger {
                        ButtonKind::DangerFill
                    } else {
                        ButtonKind::Primary
                    },
                    disabled: !can_confirm,
                    ..Default::default()
                };
                let cancel = Button {
                    label: &self.cancel,
                    keys: Some("Esc"),
                    ..Default::default()
                };
                let y = foot.top() + 10.0;
                if let Some(left) = &self.left {
                    let button = Button {
                        label: left,
                        ..Default::default()
                    };
                    let at = egui::pos2(foot.left() + 15.0, y);
                    if button.show(ui, pal, at, keyboard).clicked() {
                        answer = Some(Answer::Left);
                    }
                }
                let confirm_x = foot.right() - 16.0 - confirm.width(ui);
                let cancel_x = confirm_x - 8.0 - cancel.width(ui);
                let quiet = cancel.show(ui, pal, egui::pos2(cancel_x, y), keyboard);
                if quiet.clicked() {
                    answer = Some(Answer::Cancel);
                }
                // Enter never presses the red button, also when it has focus (S10.5).
                // Space and a click do.
                let confirm_id = confirm.id(ui);
                if self.danger && ui.memory(|memory| memory.has_focus(confirm_id)) {
                    ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter));
                }
                let response = confirm.show(ui, pal, egui::pos2(confirm_x, y), keyboard);
                if opened {
                    match self.start_on_cancel {
                        true => quiet.request_focus(),
                        false => response.request_focus(),
                    }
                }
                // The name is checked again with what the field holds after this frame's keys.
                if response.clicked() && can_confirm && self.can_confirm() {
                    answer = Some(Answer::Confirm);
                }
            });
        if modal.should_close() {
            answer = Some(Answer::Cancel);
        }
        answer
    }
}
