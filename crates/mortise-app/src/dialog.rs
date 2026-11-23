//! The dialog (C14): a confirmation before something is lost or before others are
//! affected. Forms live in the workspace, not here.

use eframe::egui::{self, Align2, Rect, Role, Sense, Shadow, Ui, vec2};

use crate::draw::{self, Button, ButtonKind};
use crate::theme::{Palette, radius, size, ty};

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
    /// The button that goes ahead is the filled red one: something is lost.
    pub danger: bool,
    /// A quiet button on the left, such as "Dry run".
    pub left: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Confirm,
    Cancel,
    /// The button on the left.
    Left,
}

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
        }
    }

    /// Paints the dialog over a scrim, 440 px wide. It holds focus until it closes.
    /// `opened` is true in the frame it opens: a confirmation that loses nothing
    /// starts on the button that goes ahead, else on the quiet choice.
    pub fn show(&self, ui: &Ui, pal: &Palette, opened: bool, keyboard: bool) -> Option<Answer> {
        let id = egui::Id::unique("dialog");
        let area = egui::Modal::default_area(id)
            .anchor(Align2::CENTER_TOP, vec2(0.0, size::TITLEBAR_H + 96.0));
        let mut answer = None;
        let modal = egui::Modal::new(id)
            .frame(egui::Frame::NONE)
            .backdrop_color(pal.scrim)
            .area(area)
            .show(ui.ctx(), |ui| {
                let text_w = size::DIALOG_W - 34.0;
                let body = draw::galley_wrapped(ui, &self.body, ty::BODY, pal.ink_2, text_w);
                let foot_h = 10.0 + size::CONTROL_H + 10.0;
                let height =
                    1.0 + 14.0 + ty::OVERLAY_TITLE.line + 8.0 + body.size().y + 12.0 + foot_h + 1.0;
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
                    egui::pos2(rect.left() + 17.0, title.bottom() + 8.0),
                    Align2::LEFT_TOP,
                    body,
                );
                draw::announce(ui, body_rect, Role::Label, &self.body);

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
                let response = confirm.show(ui, pal, egui::pos2(confirm_x, y), keyboard);
                if opened {
                    match self.start_on_cancel {
                        true => quiet.request_focus(),
                        false => response.request_focus(),
                    }
                }
                if response.clicked() {
                    answer = Some(Answer::Confirm);
                }
            });
        if modal.should_close() {
            answer = Some(Answer::Cancel);
        }
        answer
    }
}
