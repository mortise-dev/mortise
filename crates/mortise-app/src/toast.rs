//! The toast (C13): a short message after something happened out of sight.
//!
//! One at a time, at the bottom right of the workspace. A done toast leaves after
//! 4 s; a failed toast stays until it is closed.

use eframe::egui::{
    Align2, Color32, Rect, Role, Sense, Shadow, Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};

use crate::draw::{self, Button, Icon};
use crate::strings;
use crate::theme::{Palette, radius, size, ty};

const SLIDE_IN: f64 = 0.120;
const DWELL: f64 = 4.0;
const FADE_OUT: f64 = 0.160;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Done,
    Failed,
}

/// What the button of a toast asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastAction {
    /// Select the app at this place on the rail and open the runner drawer.
    ShowOutput(usize),
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    /// A small button under the text, such as "Show output".
    pub action: Option<(String, ToastAction)>,
    /// When it was first painted, in seconds of egui's clock.
    shown: Option<f64>,
}

impl Toast {
    pub fn done(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ToastKind::Done,
            action: None,
            shown: None,
        }
    }

    pub fn failed(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ToastKind::Failed,
            action: None,
            shown: None,
        }
    }

    /// A toast with a button under its text.
    pub fn with_action(mut self, label: impl Into<String>, action: ToastAction) -> Self {
        self.action = Some((label.into(), action));
        self
    }

    /// Paints the toast at the bottom right of `area`. Returns false once it has
    /// left or was closed, and what its button asked for.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        pal: &Palette,
        area: Rect,
        reduced_motion: bool,
    ) -> (bool, Option<ToastAction>) {
        let now = ui.input(|input| input.time);
        let age = now - *self.shown.get_or_insert(now);

        // In: 8 px up over 120 ms, ease out. Out: fades over 160 ms.
        // With reduced motion the toast appears and disappears; the dwell stays.
        let slide = if age < SLIDE_IN && !reduced_motion {
            let left = 1.0 - (age / SLIDE_IN) as f32;
            8.0 * left * left
        } else {
            0.0
        };
        let fade = if reduced_motion { 0.0 } else { FADE_OUT };
        let opacity = match self.kind {
            ToastKind::Done if age > DWELL + fade => return (false, None),
            ToastKind::Done if age > DWELL => 1.0 - ((age - DWELL) / FADE_OUT) as f32,
            _ => 1.0,
        };
        match self.kind {
            _ if (age < SLIDE_IN && !reduced_motion) || opacity < 1.0 => ui.ctx().request_repaint(),
            ToastKind::Done => {
                let wait = std::time::Duration::from_secs_f64(DWELL - age);
                ui.ctx().request_repaint_after(wait);
            }
            ToastKind::Failed => {}
        }

        let text_w =
            size::TOAST_W - (1.0 + 12.0 + size::STATE_MARK + 9.0) - (9.0 + 12.0 + 10.0 + 1.0);
        let text = draw::galley_wrapped(ui, &self.text, ty::BODY, pal.ink, text_w);
        let button = self.action.as_ref().map(|(label, _)| Button {
            label,
            small: true,
            ..Default::default()
        });
        let button_h = button.as_ref().map_or(0.0, |button| 6.0 + button.height());
        let height = (text.size().y + button_h + 18.0 + 2.0).ceil();
        let min = pos2(
            area.right() - 16.0 - size::TOAST_W,
            area.bottom() - 16.0 - height + slide,
        );
        let rect = Rect::from_min_size(min, vec2(size::TOAST_W, height));

        let mut open = true;
        let mut asked = None;
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.set_opacity(opacity);

            let shadow = Shadow {
                offset: [0, 6],
                blur: 16,
                spread: 0,
                color: pal.shadow,
            };
            ui.painter().add(shadow.as_shape(rect, radius::OVERLAY));
            draw::outlined(ui, rect, radius::OVERLAY, pal.surface, pal.ink);

            let mark = Rect::from_min_size(
                rect.min + vec2(13.0, 10.0 + 3.0),
                Vec2::splat(size::STATE_MARK),
            );
            match self.kind {
                ToastKind::Done => draw::mark_good(ui, mark, pal.good),
                ToastKind::Failed => draw::mark_bad(ui, mark, pal.bad, pal.surface),
            }
            let text_rect = draw::paint_galley(
                ui,
                pos2(mark.right() + 9.0, rect.top() + 10.0),
                Align2::LEFT_TOP,
                text,
            );
            // The message is announced when it appears, not read again while it stays.
            let announced = draw::announce(ui, text_rect, Role::Label, &self.text);
            ui.ctx().accesskit_node_builder(announced.id, |node| {
                node.set_live(eframe::egui::accesskit::Live::Polite);
            });
            if let (Some(button), Some((_, action))) = (&button, &self.action) {
                let at = pos2(text_rect.left(), text_rect.bottom() + 6.0);
                if button.show(ui, pal, at, false).clicked() {
                    asked = Some(*action);
                }
            }

            let close = Rect::from_min_size(
                pos2(rect.right() - 11.0 - 12.0, rect.top() + 13.0),
                Vec2::splat(12.0),
            );
            let target = close.expand(6.0);
            let response = ui.interact(target, ui.scope_id().with("toast-close"), Sense::click());
            response
                .widget_info(|| WidgetInfo::labeled(Role::Button, true, strings::CLOSE_MESSAGE));
            let color: Color32 = if response.hovered() {
                pal.ink
            } else {
                pal.ink_2
            };
            draw::icon(ui, close, Icon::Close, color);
            if response.clicked() {
                open = false;
            }
        });
        (open, asked)
    }
}
