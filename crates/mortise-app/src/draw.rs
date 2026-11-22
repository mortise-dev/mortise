//! Painting: text, lines, icons and the small components of section 4.
//!
//! Mortise paints everything itself from flat fills, 1 px strokes and rounded
//! rectangles. Nothing here keeps state.

use std::sync::Arc;

use eframe::egui::text::LayoutJob;
use eframe::egui::{
    Align2, Color32, CornerRadius, Galley, Pos2, Rect, Response, Role, Sense, Shape, Stroke,
    StrokeKind, Ui, Vec2, WidgetInfo, pos2, vec2,
};

use crate::theme::{Palette, Type, radius, size, stroke, ty};

// ---------------------------------------------------------------------------
// Text

/// Text laid out on one line, however long.
pub fn galley(ui: &Ui, text: &str, ty: Type, color: Color32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(text.to_owned(), ty.format(color));
    job.wrap.max_width = f32::INFINITY;
    ui.painter().layout_job(job)
}

/// Text laid out on one line and cut with an ellipsis when it is wider than `width`.
pub fn galley_elided(ui: &Ui, text: &str, ty: Type, color: Color32, width: f32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(text.to_owned(), ty.format(color));
    job.wrap.max_width = width;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    ui.painter().layout_job(job)
}

/// Text wrapped at `width`.
pub fn galley_wrapped(ui: &Ui, text: &str, ty: Type, color: Color32, width: f32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(text.to_owned(), ty.format(color));
    job.wrap.max_width = width;
    ui.painter().layout_job(job)
}

/// Paints a galley with `anchor` of its box at `pos`. Returns the box.
pub fn paint_galley(ui: &Ui, pos: Pos2, anchor: Align2, galley: Arc<Galley>) -> Rect {
    let rect = anchor.anchor_size(pos, galley.size());
    ui.painter().galley(rect.min, galley, Color32::PLACEHOLDER);
    rect
}

/// Paints one line of text. For text inside a control that carries its own name.
pub fn paint_text(
    ui: &Ui,
    pos: Pos2,
    anchor: Align2,
    text: &str,
    ty: Type,
    color: Color32,
) -> Rect {
    paint_galley(ui, pos, anchor, galley(ui, text, ty, color))
}

/// Paints one line of text and tells the accessibility interface about it.
pub fn label(ui: &Ui, pos: Pos2, anchor: Align2, text: &str, ty: Type, color: Color32) -> Rect {
    let rect = paint_text(ui, pos, anchor, text, ty, color);
    announce(ui, rect, Role::Label, text);
    rect
}

/// Paints wrapped text with its top left at `pos` and tells the accessibility interface about it.
pub fn paragraph(ui: &Ui, pos: Pos2, text: &str, ty: Type, color: Color32, width: f32) -> Rect {
    let rect = paint_galley(
        ui,
        pos,
        Align2::LEFT_TOP,
        galley_wrapped(ui, text, ty, color, width),
    );
    announce(ui, rect, Role::Label, text);
    rect
}

/// Gives a painted, non-interactive thing a role and a name.
pub fn announce(ui: &Ui, rect: Rect, role: Role, name: &str) -> Response {
    let id = ui
        .scope_id()
        .with(("announce", name, rect.min.x as i32, rect.min.y as i32));
    let response = ui.interact(rect, id, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(role, true, name));
    response
}

// ---------------------------------------------------------------------------
// Lines and boxes

/// A horizontal 1 px line whose top edge is at `y`.
pub fn hline(ui: &Ui, x: std::ops::Range<f32>, y: f32, color: Color32) {
    let rect = Rect::from_min_max(pos2(x.start, y), pos2(x.end, y + stroke::HAIRLINE));
    ui.painter().rect_filled(rect, 0, color);
}

/// A vertical 1 px line whose left edge is at `x`.
pub fn vline(ui: &Ui, x: f32, y: std::ops::Range<f32>, color: Color32) {
    let rect = Rect::from_min_max(pos2(x, y.start), pos2(x + stroke::HAIRLINE, y.end));
    ui.painter().rect_filled(rect, 0, color);
}

/// A filled box with a 1 px outline drawn inside its edge.
pub fn outlined(ui: &Ui, rect: Rect, radius: u8, fill: Color32, outline: Color32) {
    ui.painter().rect(
        rect,
        CornerRadius::same(radius),
        fill,
        Stroke::new(stroke::HAIRLINE, outline),
        StrokeKind::Inside,
    );
}

/// The keyboard focus ring: 2 px of ink, 1 px outside the control, over everything else.
pub fn focus_ring(ui: &Ui, pal: &Palette, rect: Rect, radius: u8) {
    ui.painter().rect_stroke(
        rect.expand(stroke::FOCUS_OFFSET),
        CornerRadius::same(radius + stroke::FOCUS_OFFSET as u8),
        Stroke::new(stroke::FOCUS_RING, pal.ink),
        StrokeKind::Outside,
    );
}

/// The peg of a selected row: a 5 px square of ink, 4 px inside the row's edge.
pub fn peg(ui: &Ui, pal: &Palette, row: Rect) {
    let min = pos2(row.left() + 5.0, (row.center().y - size::PEG / 2.0).round());
    ui.painter()
        .rect_filled(Rect::from_min_size(min, Vec2::splat(size::PEG)), 0, pal.ink);
}

// ---------------------------------------------------------------------------
// Icons: strokes on a 16 px grid, square ends, mitred corners

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Search,
    Settings,
    Generate,
    Run,
    Boundary,
    Metrics,
    Minimise,
    Maximise,
    Restore,
    Close,
}

/// One stroke of an icon: points on the 16 px grid.
struct Path {
    points: &'static [[f32; 2]],
    closed: bool,
    /// Stroke width on the 16 px grid.
    width: f32,
    dashed: bool,
}

const fn open(points: &'static [[f32; 2]]) -> Path {
    Path {
        points,
        closed: false,
        width: stroke::ICON,
        dashed: false,
    }
}

const fn closed(points: &'static [[f32; 2]]) -> Path {
    Path {
        points,
        closed: true,
        width: stroke::ICON,
        dashed: false,
    }
}

const fn square(x: f32, y: f32, w: f32, h: f32) -> [[f32; 2]; 4] {
    [[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
}

impl Icon {
    /// The strokes, copied from the symbols of the specification's drawings.
    fn paths(self) -> &'static [Path] {
        const SEARCH_BOX: [[f32; 2]; 4] = square(2.75, 2.75, 7.5, 7.5);
        const KNOB_A: [[f32; 2]; 4] = square(4.75, 3.25, 3.5, 3.5);
        const KNOB_B: [[f32; 2]; 4] = square(7.75, 9.25, 3.5, 3.5);
        const SIDE_A: [[f32; 2]; 4] = square(1.75, 4.75, 4.0, 6.5);
        const SIDE_B: [[f32; 2]; 4] = square(10.25, 4.75, 4.0, 6.5);
        const MAX_BOX: [[f32; 2]; 4] = square(3.75, 3.75, 8.5, 8.5);
        const RESTORE_FRONT: [[f32; 2]; 4] = square(3.25, 5.25, 7.5, 7.5);

        const SEARCH: &[Path] = &[closed(&SEARCH_BOX), open(&[[10.25, 10.25], [14.0, 14.0]])];
        const SETTINGS: &[Path] = &[
            open(&[[1.5, 5.0], [4.75, 5.0]]),
            open(&[[8.25, 5.0], [14.5, 5.0]]),
            open(&[[1.5, 11.0], [7.75, 11.0]]),
            open(&[[11.25, 11.0], [14.5, 11.0]]),
            closed(&KNOB_A),
            closed(&KNOB_B),
        ];
        const GENERATE: &[Path] = &[
            open(&[[2.75, 6.0], [2.75, 2.75], [6.0, 2.75]]),
            open(&[[10.0, 2.75], [13.25, 2.75], [13.25, 6.0]]),
            open(&[[13.25, 10.0], [13.25, 13.25], [10.0, 13.25]]),
            open(&[[6.0, 13.25], [2.75, 13.25], [2.75, 10.0]]),
            open(&[[8.0, 5.25], [8.0, 10.75]]),
            open(&[[5.25, 8.0], [10.75, 8.0]]),
        ];
        const RUN: &[Path] = &[closed(&[[4.75, 3.1], [12.9, 8.0], [4.75, 12.9]])];
        const BOUNDARY: &[Path] = &[
            closed(&SIDE_A),
            closed(&SIDE_B),
            Path {
                dashed: true,
                ..open(&[[8.0, 1.5], [8.0, 14.5]])
            },
        ];
        const METRICS: &[Path] = &[
            Path {
                width: 2.4,
                ..open(&[[3.5, 13.5], [3.5, 8.5]])
            },
            Path {
                width: 2.4,
                ..open(&[[8.0, 13.5], [8.0, 2.5]])
            },
            Path {
                width: 2.4,
                ..open(&[[12.5, 13.5], [12.5, 6.0]])
            },
        ];
        const MINIMISE: &[Path] = &[open(&[[3.5, 8.0], [12.5, 8.0]])];
        const MAXIMISE: &[Path] = &[closed(&MAX_BOX)];
        // Not in the drawings: two offset squares, the usual sign for "restore".
        const RESTORE: &[Path] = &[
            closed(&RESTORE_FRONT),
            open(&[
                [5.25, 5.25],
                [5.25, 3.25],
                [12.75, 3.25],
                [12.75, 10.75],
                [10.75, 10.75],
            ]),
        ];
        const CLOSE: &[Path] = &[
            open(&[[3.5, 3.5], [12.5, 12.5]]),
            open(&[[12.5, 3.5], [3.5, 12.5]]),
        ];

        match self {
            Icon::Search => SEARCH,
            Icon::Settings => SETTINGS,
            Icon::Generate => GENERATE,
            Icon::Run => RUN,
            Icon::Boundary => BOUNDARY,
            Icon::Metrics => METRICS,
            Icon::Minimise => MINIMISE,
            Icon::Maximise => MAXIMISE,
            Icon::Restore => RESTORE,
            Icon::Close => CLOSE,
        }
    }
}

/// Paints an icon into `rect`, which is square. The stroke scales with the icon.
pub fn icon(ui: &Ui, rect: Rect, icon: Icon, color: Color32) {
    let scale = rect.width() / size::ICON;
    let at = |point: &[f32; 2]| rect.min + vec2(point[0], point[1]) * scale;
    for path in icon.paths() {
        let line = Stroke::new(path.width * scale, color);
        let points: Vec<Pos2> = path.points.iter().map(at).collect();
        if path.dashed {
            ui.painter()
                .extend(Shape::dashed_line(&points, line, 2.0 * scale, 1.6 * scale));
        } else if path.closed {
            ui.painter().add(Shape::closed_line(points, line));
        } else {
            ui.painter().add(Shape::line(points, line));
        }
    }
}

/// The Mortise mark: an ink piece and a lighter piece that fit together. 24 px grid.
pub fn brand_mark(ui: &Ui, pal: &Palette, rect: Rect) {
    let scale = rect.width() / 24.0;
    let piece = |x0: f32, y0: f32, x1: f32, y1: f32, color: Color32| {
        let piece = Rect::from_min_max(
            rect.min + vec2(x0, y0) * scale,
            rect.min + vec2(x1, y1) * scale,
        );
        ui.painter().rect_filled(piece, 0, color);
    };
    // M3 4h9v4H8v8h4v4H3z
    piece(3.0, 4.0, 12.0, 8.0, pal.ink);
    piece(3.0, 8.0, 8.0, 16.0, pal.ink);
    piece(3.0, 16.0, 12.0, 20.0, pal.ink);
    // M21 4v16h-8v-5H9V9h4V4z
    piece(13.0, 4.0, 21.0, 20.0, pal.line_2);
    piece(9.0, 9.0, 13.0, 15.0, pal.line_2);
}

// ---------------------------------------------------------------------------
// State marks (C9). Findings are solid silhouettes; "off" is the empty frame.

/// The mark of a module that is turned off: an empty frame with a dash. 12 px.
pub fn mark_off(ui: &Ui, rect: Rect, color: Color32) {
    let scale = rect.width() / size::STATE_MARK;
    let line = Stroke::new(1.5 * scale, color);
    let frame = Rect::from_min_size(
        rect.min + Vec2::splat(1.75) * scale,
        Vec2::splat(8.5) * scale,
    );
    ui.painter().rect_stroke(frame, 0, line, StrokeKind::Middle);
    ui.painter().line_segment(
        [
            rect.min + vec2(4.0, 6.0) * scale,
            rect.min + vec2(8.0, 6.0) * scale,
        ],
        line,
    );
}

// ---------------------------------------------------------------------------
// Module code stamp (C10)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StampState {
    /// In a selected row: ink letters and outline.
    pub selected: bool,
    /// A module that is turned off: dashed outline.
    pub off: bool,
}

/// Paints a code stamp, 34 by 18 px, with its top left at `min`.
pub fn stamp(ui: &Ui, pal: &Palette, min: Pos2, code: &str, state: StampState) -> Rect {
    let rect = Rect::from_min_size(min, vec2(size::STAMP_W, size::STAMP_H));
    let color = if state.selected { pal.ink } else { pal.ink_2 };
    let outline = if state.selected { pal.ink } else { pal.line_2 };

    if state.off {
        let inner = rect.shrink(0.5);
        let corners = [
            inner.left_top(),
            inner.right_top(),
            inner.right_bottom(),
            inner.left_bottom(),
        ];
        for (from, to) in corners.into_iter().zip(corners.into_iter().cycle().skip(1)) {
            let line = Stroke::new(stroke::HAIRLINE, outline);
            ui.painter()
                .extend(Shape::dashed_line(&[from, to], line, 3.0, 2.0));
        }
    } else {
        outlined(ui, rect, radius::STAMP, Color32::TRANSPARENT, outline);
    }
    paint_text(
        ui,
        rect.center(),
        Align2::CENTER_CENTER,
        code,
        ty::STAMP,
        color,
    );
    rect
}

// ---------------------------------------------------------------------------
// Key hint (C11)

/// The width a key hint takes: its text and 4 px on each side.
pub fn keycap_width(ui: &Ui, keys: &str) -> f32 {
    galley(ui, keys, ty::KEYCAP, Color32::PLACEHOLDER)
        .size()
        .x
        .ceil()
        + 8.0
}

/// Paints a key hint, 16 px high, with its right centre at `right_center`.
pub fn keycap(ui: &Ui, pal: &Palette, right_center: Pos2, keys: &str) -> Rect {
    let rect = Align2::RIGHT_CENTER.anchor_size(right_center, vec2(keycap_width(ui, keys), 16.0));
    outlined(ui, rect, radius::STAMP, pal.well, pal.line_2);
    paint_text(
        ui,
        rect.center(),
        Align2::CENTER_CENTER,
        keys,
        ty::KEYCAP,
        pal.ink_2,
    );
    rect
}
