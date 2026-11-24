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

use mortise_core::Severity;

use crate::ui::theme::{Face, Palette, Type, radius, size, stroke, ty};

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
    ring_is_drawn(ui);
    ui.painter().rect_stroke(
        rect.expand(stroke::FOCUS_OFFSET),
        CornerRadius::same(radius + stroke::FOCUS_OFFSET as u8),
        Stroke::new(stroke::FOCUS_RING, pal.ink),
        StrokeKind::Outside,
    );
}

/// Tells the frame that the focused control drew its own ring.
pub fn ring_is_drawn(ui: &Ui) {
    ui.ctx()
        .data_mut(|data| data.insert_temp(ring_drawn_id(), true));
}

fn ring_drawn_id() -> eframe::egui::Id {
    eframe::egui::Id::unique("mortise-focus-ring-drawn")
}

/// Draws the focus ring around the focused control when that control did not
/// draw one itself, so that focus is visible on every focusable thing. Call at
/// the end of a frame, while the keyboard is in use.
pub fn focus_ring_for_the_rest(ui: &Ui, pal: &Palette) {
    let ctx = ui.ctx();
    let drawn = ctx
        .data_mut(|data| data.remove_temp::<bool>(ring_drawn_id()))
        .unwrap_or(false);
    if drawn || ctx.text_edit_focused() {
        return;
    }
    let Some(focused) = ctx.memory(|memory| memory.focused()) else {
        return;
    };
    let Some(response) = ctx.read_response(focused) else {
        return;
    };
    // Not for a control that is scrolled out of sight.
    if response.interact_rect.width() < response.rect.width() - 1.0
        || response.interact_rect.height() < response.rect.height() - 1.0
    {
        return;
    }
    let layer = eframe::egui::LayerId::new(eframe::egui::Order::Tooltip, ring_drawn_id());
    ctx.layer_painter(layer).rect_stroke(
        response.rect.expand(stroke::FOCUS_OFFSET),
        CornerRadius::same(stroke::FOCUS_OFFSET as u8),
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
    Folder,
    App,
    List,
    Lock,
    Plus,
    Stop,
    /// A migration: a box, then an arrow out of it.
    Migrate,
    /// Tests: two ticks and two lines.
    Test,
    /// A closed group: it opens to the right.
    ChevronRight,
    /// An open group.
    ChevronDown,
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
        const FOLDER: &[Path] = &[closed(&[
            [1.75, 3.75],
            [6.25, 3.75],
            [7.75, 5.5],
            [14.25, 5.5],
            [14.25, 12.75],
            [1.75, 12.75],
        ])];
        const APP_BOX: [[f32; 2]; 4] = square(2.25, 3.25, 11.5, 9.5);
        const APP: &[Path] = &[closed(&APP_BOX), open(&[[2.25, 6.25], [13.75, 6.25]])];
        const LIST: &[Path] = &[
            open(&[[2.5, 4.0], [13.5, 4.0]]),
            open(&[[2.5, 8.0], [13.5, 8.0]]),
            open(&[[2.5, 12.0], [9.5, 12.0]]),
        ];
        const LOCK_BODY: [[f32; 2]; 4] = square(4.25, 7.25, 7.5, 6.0);
        const LOCK: &[Path] = &[
            closed(&LOCK_BODY),
            open(&[[5.75, 7.25], [5.75, 4.25], [10.25, 4.25], [10.25, 7.25]]),
        ];
        const PLUS: &[Path] = &[
            open(&[[8.0, 3.0], [8.0, 13.0]]),
            open(&[[3.0, 8.0], [13.0, 8.0]]),
        ];
        const STOP_BOX: [[f32; 2]; 4] = square(4.25, 4.25, 7.5, 7.5);
        const STOP: &[Path] = &[closed(&STOP_BOX)];
        const CLOSE: &[Path] = &[
            open(&[[3.5, 3.5], [12.5, 12.5]]),
            open(&[[12.5, 3.5], [3.5, 12.5]]),
        ];
        const MIGRATE_BOX: [[f32; 2]; 4] = square(1.75, 3.75, 6.5, 8.5);
        const MIGRATE: &[Path] = &[
            closed(&MIGRATE_BOX),
            open(&[[1.75, 8.0], [8.25, 8.0]]),
            open(&[[10.25, 8.0], [14.75, 8.0]]),
            open(&[[12.5, 5.5], [15.0, 8.0], [12.5, 10.5]]),
        ];
        const TEST: &[Path] = &[
            open(&[[2.25, 4.75], [3.9, 6.4], [6.5, 3.2]]),
            open(&[[8.75, 4.9], [14.25, 4.9]]),
            open(&[[2.25, 10.75], [3.9, 12.4], [6.5, 9.2]]),
            open(&[[8.75, 10.9], [14.25, 10.9]]),
        ];

        const CHEVRON_RIGHT: &[Path] = &[open(&[[6.0, 3.5], [10.5, 8.0], [6.0, 12.5]])];
        const CHEVRON_DOWN: &[Path] = &[open(&[[3.5, 6.0], [8.0, 10.5], [12.5, 6.0]])];

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
            Icon::Folder => FOLDER,
            Icon::App => APP,
            Icon::List => LIST,
            Icon::Lock => LOCK,
            Icon::Plus => PLUS,
            Icon::Stop => STOP,
            Icon::Migrate => MIGRATE,
            Icon::Test => TEST,
            Icon::ChevronRight => CHEVRON_RIGHT,
            Icon::ChevronDown => CHEVRON_DOWN,
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

// ---------------------------------------------------------------------------
// More state marks (C9)

/// The mark of a good state: a check. 12 px.
pub fn mark_good(ui: &Ui, rect: Rect, color: Color32) {
    let scale = rect.width() / size::STATE_MARK;
    let at = |x: f32, y: f32| rect.min + vec2(x, y) * scale;
    let points = vec![at(1.8, 6.4), at(4.7, 9.3), at(10.2, 2.9)];
    ui.painter()
        .add(Shape::line(points, Stroke::new(1.9 * scale, color)));
}

/// The mark of a problem: a block with a cross cut out of it. 12 px.
/// `behind` is the fill the mark sits on, which shows through the cross.
pub fn mark_bad(ui: &Ui, rect: Rect, color: Color32, behind: Color32) {
    let scale = rect.width() / size::STATE_MARK;
    let at = |x: f32, y: f32| rect.min + vec2(x, y) * scale;
    ui.painter()
        .rect_filled(Rect::from_min_max(at(1.0, 1.0), at(11.0, 11.0)), 0, color);
    let cut = Stroke::new(1.27 * scale, behind);
    ui.painter()
        .line_segment([at(3.75, 3.75), at(8.25, 8.25)], cut);
    ui.painter()
        .line_segment([at(8.25, 3.75), at(3.75, 8.25)], cut);
}

/// The mark of a warning: a wedge. 12 px.
pub fn mark_warn(ui: &Ui, rect: Rect, color: Color32) {
    let scale = rect.width() / size::STATE_MARK;
    let at = |x: f32, y: f32| rect.min + vec2(x, y) * scale;
    let points = vec![at(6.0, 1.0), at(11.4, 10.8), at(0.6, 10.8)];
    ui.painter()
        .add(Shape::convex_polygon(points, color, Stroke::NONE));
}

// ---------------------------------------------------------------------------
// Runtime marks (C9). What the running app reports is drawn in strokes, never as
// a solid silhouette: a runtime mark cannot be taken for a finding mark.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMark {
    /// Up: an arrow up.
    Up,
    /// Down: an arrow down.
    Down,
    /// Out of service: two bars.
    Bars,
    /// Unknown, no answer, not authorized: three dashes.
    Dashes,
}

/// Paints a runtime mark into `rect`. 12 px.
pub fn runtime_mark(ui: &Ui, rect: Rect, mark: RuntimeMark, color: Color32) {
    let scale = rect.width() / size::STATE_MARK;
    let at = |x: f32, y: f32| rect.min + vec2(x, y) * scale;
    let painter = ui.painter();
    let line = |width: f32| Stroke::new(width * scale, color);
    match mark {
        RuntimeMark::Up => {
            painter.line_segment([at(6.0, 11.0), at(6.0, 2.4)], line(1.7));
            painter.add(Shape::line(
                vec![at(2.4, 6.0), at(6.0, 2.4), at(9.6, 6.0)],
                line(1.7),
            ));
        }
        RuntimeMark::Down => {
            painter.line_segment([at(6.0, 1.0), at(6.0, 9.6)], line(1.7));
            painter.add(Shape::line(
                vec![at(2.4, 6.0), at(6.0, 9.6), at(9.6, 6.0)],
                line(1.7),
            ));
        }
        RuntimeMark::Bars => {
            painter.line_segment([at(3.6, 1.5), at(3.6, 10.5)], line(2.0));
            painter.line_segment([at(8.4, 1.5), at(8.4, 10.5)], line(2.0));
        }
        RuntimeMark::Dashes => {
            for (from, to) in [(1.0, 3.2), (4.9, 7.1), (8.8, 11.0)] {
                painter.line_segment([at(from, 6.0), at(to, 6.0)], line(1.8));
            }
        }
    }
}

/// A runtime mark and its word, with the right end of the word at `right_center`.
/// Returns the rect of both.
pub fn runtime_state(
    ui: &Ui,
    right_center: Pos2,
    mark: RuntimeMark,
    word: &str,
    color: Color32,
) -> Rect {
    let state_type = Type {
        size: 11.5,
        ..ty::LABEL
    };
    let text = label(
        ui,
        right_center,
        Align2::RIGHT_CENTER,
        word,
        state_type,
        color,
    );
    let at = Rect::from_min_size(
        pos2(
            text.left() - 5.0 - size::STATE_MARK,
            right_center.y - size::STATE_MARK / 2.0,
        ),
        Vec2::splat(size::STATE_MARK),
    );
    runtime_mark(ui, at, mark, color);
    at.union(text)
}

// ---------------------------------------------------------------------------
// Code block: text to copy or to read as the app wrote it

/// The height a code block takes for `text` in `width`.
pub fn code_block_height(ui: &Ui, text: &str, width: f32) -> f32 {
    galley_wrapped(ui, text, ty::CODE, Color32::PLACEHOLDER, width - 24.0)
        .size()
        .y
        + 20.0
}

/// Paints a code block with its top left at `min`: code fill, mono text that wraps.
/// Returns its rect.
pub fn code_block(ui: &Ui, pal: &Palette, min: Pos2, width: f32, text: &str) -> Rect {
    let galley = galley_wrapped(ui, text, ty::CODE, pal.code_fg, width - 24.0);
    let rect = Rect::from_min_size(min, vec2(width, galley.size().y + 20.0));
    ui.painter().rect_filled(rect, radius::CONTROL, pal.code_bg);
    let text_rect = paint_galley(ui, rect.min + vec2(12.0, 10.0), Align2::LEFT_TOP, galley);
    announce(ui, text_rect, Role::Label, text);
    rect
}

/// Several runs of text in one wrapped paragraph, each in its own type and colour.
pub fn rich_galley(ui: &Ui, parts: &[(&str, Type, Color32)], width: f32) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    for (text, ty, color) in parts {
        job.append(text, 0.0, ty.format(*color));
    }
    job.wrap.max_width = width;
    ui.painter().layout_job(job)
}

/// The pointer target of a control: at least 24 by 24 px, also where the control is
/// drawn smaller. The target extends beyond the drawing on both sides.
pub fn hit_target(drawn: Rect) -> Rect {
    let grow = |side: f32| ((size::HIT_MIN - side) / 2.0).max(0.0);
    drawn.expand2(vec2(grow(drawn.width()), grow(drawn.height())))
}

// ---------------------------------------------------------------------------
// Buttons (C1)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonKind {
    /// One per view: ink fill.
    Primary,
    /// Surface fill, line-2 outline.
    #[default]
    Default,
    /// No outline, for the quiet choice.
    Ghost,
    /// A destructive action that asks first: bad outline and bad text.
    Danger,
    /// The button that confirms a destructive action, in a dialog only: bad fill.
    DangerFill,
}

/// A button: 26 px high, or 22 px when small.
#[derive(Debug, Clone, Copy, Default)]
pub struct Button<'a> {
    pub label: &'a str,
    pub kind: ButtonKind,
    pub small: bool,
    /// The shortcut, shown in a key hint after the label.
    pub keys: Option<&'a str>,
    pub icon: Option<Icon>,
    pub disabled: bool,
    /// The name a screen reader says, where the label alone does not tell this
    /// button from another on the same screen.
    pub said: Option<&'a str>,
}

impl Button<'_> {
    fn label_type(&self) -> Type {
        if self.small {
            Type {
                size: 11.5,
                ..ty::CONTROL
            }
        } else {
            ty::CONTROL
        }
    }

    fn padding(&self) -> f32 {
        if self.small { 8.0 } else { 10.0 }
    }

    pub fn height(&self) -> f32 {
        if self.small {
            size::CONTROL_SM_H
        } else {
            size::CONTROL_H
        }
    }

    pub fn width(&self, ui: &Ui) -> f32 {
        let label = galley(ui, self.label, self.label_type(), Color32::PLACEHOLDER)
            .size()
            .x;
        let icon = if self.icon.is_some() {
            size::ICON_IN_CONTROL + 6.0
        } else {
            0.0
        };
        let keys = self
            .keys
            .map_or(0.0, |keys| 6.0 + 2.0 + keycap_width(ui, keys));
        (self.padding() * 2.0 + icon + label + keys).ceil()
    }

    /// The id of the button in `ui`: the same in every frame it is drawn there.
    pub fn id(&self, ui: &Ui) -> eframe::egui::Id {
        ui.scope_id().with(("button", self.label, self.said))
    }

    /// Paints the button with its top left at `min`. `keyboard` is true while the
    /// keyboard is in use: the focus ring shows only then.
    pub fn show(&self, ui: &Ui, pal: &Palette, min: Pos2, keyboard: bool) -> Response {
        let rect = Rect::from_min_size(min, vec2(self.width(ui), self.height()));
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        // A small button is drawn 22 px high; its target is 24 px.
        let response = ui.interact(hit_target(rect), self.id(ui), sense);
        let said = self.said.unwrap_or(self.label);
        response.widget_info(|| WidgetInfo::labeled(Role::Button, !self.disabled, said));

        let pressed = response.is_pointer_button_down_on();
        let hovered = response.hovered();
        let (fill, outline, text) = match (self.kind, self.disabled) {
            (ButtonKind::Primary, true) => (pal.line, pal.line, pal.ink_2),
            (_, true) => (Color32::TRANSPARENT, pal.line, pal.ink_3),
            (ButtonKind::Danger, false) => {
                let fill = if pressed || hovered {
                    pal.bad_soft
                } else {
                    pal.surface
                };
                (fill, pal.bad, pal.bad)
            }
            (ButtonKind::DangerFill, false) => (pal.bad, pal.bad, pal.bad_fg),
            (ButtonKind::Primary, false) => {
                let fill = if pressed {
                    pal.key_press
                } else if hovered {
                    pal.key_hover
                } else {
                    pal.key
                };
                (fill, fill, pal.key_fg)
            }
            (kind, false) => {
                let ghost = kind == ButtonKind::Ghost;
                let fill = if pressed {
                    pal.line
                } else if hovered {
                    pal.well
                } else if ghost {
                    Color32::TRANSPARENT
                } else {
                    pal.surface
                };
                (
                    fill,
                    if ghost {
                        Color32::TRANSPARENT
                    } else {
                        pal.line_2
                    },
                    pal.ink,
                )
            }
        };
        outlined(ui, rect, radius::CONTROL, fill, outline);
        // The filled red button shows hover and press as a ring of its text colour.
        if self.kind == ButtonKind::DangerFill && !self.disabled && (hovered || pressed) {
            let width = if pressed { 2.0 } else { 1.0 };
            ui.painter().rect_stroke(
                rect.shrink(1.0),
                CornerRadius::same(radius::CONTROL - 1),
                Stroke::new(width, pal.bad_fg),
                StrokeKind::Inside,
            );
        }

        let mid = rect.center().y;
        let mut x = rect.left() + self.padding();
        if let Some(glyph) = self.icon {
            let side = size::ICON_IN_CONTROL;
            icon(
                ui,
                Rect::from_min_size(pos2(x, mid - side / 2.0), Vec2::splat(side)),
                glyph,
                text,
            );
            x += side + 6.0;
        }
        paint_text(
            ui,
            pos2(x, mid),
            Align2::LEFT_CENTER,
            self.label,
            self.label_type(),
            text,
        );
        if let Some(keys) = self.keys {
            keycap(ui, pal, pos2(rect.right() - self.padding(), mid), keys);
        }
        if keyboard && response.has_focus() {
            focus_ring(ui, pal, rect, radius::CONTROL);
        }
        response
    }
}

// ---------------------------------------------------------------------------
// Boxes with lines of a name and a value

/// The outline of a box and its heading line: a title at the left, a note at the right.
/// A note with no room beside the title goes under it, which makes the head
/// `box_head_extra` higher. Returns the y where the content starts.
pub fn box_head(ui: &Ui, pal: &Palette, rect: Rect, title: &str, note: &str) -> f32 {
    outlined(ui, rect, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
    let mid = rect.top() + 1.0 + 8.0 + 9.0;
    let title_rect = paint_text(
        ui,
        pos2(rect.left() + 13.0, mid),
        Align2::LEFT_CENTER,
        title,
        BOX_TITLE,
        pal.ink,
    );
    announce(ui, title_rect, Role::Heading, title);
    let mut line = rect.top() + 1.0 + 8.0 + 18.0 + 7.0;
    if let Some(below) = note_below(ui, rect.width(), title, note, pal.ink_2) {
        let min = pos2(rect.left() + 13.0, rect.top() + 1.0 + 8.0 + 18.0 + 2.0);
        let note_rect = paint_galley(ui, min, Align2::LEFT_TOP, below);
        announce(ui, note_rect, Role::Label, note);
        line = note_rect.bottom() + 7.0;
    } else if !note.is_empty() {
        label(
            ui,
            pos2(rect.right() - 13.0, mid),
            Align2::RIGHT_CENTER,
            note,
            ty::SECONDARY,
            pal.ink_2,
        );
    }
    hline(ui, rect.left() + 13.0..rect.right() - 13.0, line, pal.line);
    line + 1.0 + 6.0
}

/// How much higher `box_head` is than one line, for a box `width` wide.
pub fn box_head_extra(ui: &Ui, width: f32, title: &str, note: &str) -> f32 {
    note_below(ui, width, title, note, Color32::PLACEHOLDER)
        .map_or(0.0, |below| below.size().y + 2.0)
}

const BOX_TITLE: Type = ty::BODY.with_face(crate::ui::theme::Face::Sans700);

/// The note of a box head, wrapped to the box, when it does not fit beside the title.
fn note_below(ui: &Ui, width: f32, title: &str, note: &str, color: Color32) -> Option<Arc<Galley>> {
    if note.is_empty() {
        return None;
    }
    let title_w = galley(ui, title, BOX_TITLE, Color32::PLACEHOLDER).size().x;
    let note_w = galley(ui, note, ty::SECONDARY, Color32::PLACEHOLDER)
        .size()
        .x;
    (title_w + 16.0 + note_w > width - 26.0)
        .then(|| galley_wrapped(ui, note, ty::SECONDARY, color, width - 26.0))
}

/// One line of a box: a name at the left, a value at the right, 27 px high, with a
/// hairline under it unless it is the last. Returns the y of the next line.
pub fn box_line(
    ui: &Ui,
    pal: &Palette,
    rect: Rect,
    y: f32,
    name: (&str, Type),
    value: &[(&str, Type, Color32)],
    last: bool,
) -> f32 {
    let mid = y + size::LIST_LINE_H / 2.0;
    label(
        ui,
        pos2(rect.left() + 13.0, mid),
        Align2::LEFT_CENTER,
        name.0,
        name.1,
        pal.ink,
    );
    // The parts of the value are laid out from the right edge.
    let mut right = rect.right() - 13.0;
    for (text, ty, color) in value.iter().rev() {
        let part = label(
            ui,
            pos2(right, mid),
            Align2::RIGHT_CENTER,
            text,
            *ty,
            *color,
        );
        right = part.left();
    }
    if !last {
        hline(
            ui,
            rect.left() + 13.0..rect.right() - 13.0,
            y + size::LIST_LINE_H - 1.0,
            pal.line,
        );
    }
    y + size::LIST_LINE_H
}

// ---------------------------------------------------------------------------
// Progress (C17)

/// Paints a progress bar with its top left at `min`: a label, the share in per
/// cent, and a 6 px track with an ink fill. Returns its bottom edge.
pub fn progress(ui: &Ui, pal: &Palette, min: Pos2, width: f32, label: &str, share: f32) -> f32 {
    let share = share.clamp(0.0, 1.0);
    crate::ui::draw::label(ui, min, Align2::LEFT_TOP, label, ty::SECONDARY, pal.ink);
    let percent = format!("{}%", (share * 100.0).round() as u32);
    let percent_type = Type {
        size: 11.0,
        ..ty::CODE
    };
    let at = pos2(min.x + width, min.y);
    paint_text(ui, at, Align2::RIGHT_TOP, &percent, percent_type, pal.ink_2);
    let track = Rect::from_min_size(
        pos2(min.x, min.y + ty::SECONDARY.line + 6.0),
        vec2(width, size::BAR_H),
    );
    ui.painter().rect_filled(track, 0, pal.well);
    let fill = Rect::from_min_size(track.min, vec2((width * share).round(), size::BAR_H));
    ui.painter().rect_filled(fill, 0, pal.ink);
    let bar = ui.interact(
        track,
        ui.scope_id().with(("progress", label)),
        Sense::hover(),
    );
    bar.widget_info(|| WidgetInfo::labeled(Role::ProgressIndicator, true, label));
    track.bottom()
}

// ---------------------------------------------------------------------------
// Alert (C19)

/// An alert (C19) at the top of the view it concerns. A problem has the bad-soft
/// fill, the bad outline and the problem mark; a warning has the surface fill, the
/// warn outline and the wedge. It may carry one small button on the right.
/// Returns its bottom edge and whether the button was pressed.
#[expect(clippy::too_many_arguments)]
pub(crate) fn alert_box(
    ui: &Ui,
    pal: &Palette,
    min: eframe::egui::Pos2,
    width: f32,
    severity: Severity,
    bold: &str,
    rest: &str,
    button: Option<(&str, &str)>,
    paint: bool,
) -> (f32, bool) {
    let button = button.map(|(label, keys)| Button {
        label,
        small: true,
        keys: Some(keys),
        ..Default::default()
    });
    let button_w = button
        .as_ref()
        .map_or(0.0, |button| button.width(ui) + 12.0);
    let text_x = min.x + 13.0 + size::STATE_MARK + 9.0;
    let text_w = min.x + width - 13.0 - button_w - text_x;
    let parts = [
        (bold, ty::BODY.with_face(Face::Sans700), pal.ink),
        (rest, ty::BODY, pal.ink),
    ];
    let galley = rich_galley(ui, &parts, text_w);
    let rect = Rect::from_min_size(min, vec2(width, (galley.size().y + 20.0).max(40.0)));
    if !paint {
        return (rect.bottom(), false);
    }
    let (fill, outline) = match severity {
        Severity::Problem => (pal.bad_soft, pal.bad),
        Severity::Warning => (pal.surface, pal.warn),
    };
    outlined(ui, rect, radius::CONTROL, fill, outline);
    let mark = Rect::from_min_size(rect.min + vec2(13.0, 13.0), Vec2::splat(size::STATE_MARK));
    match severity {
        Severity::Problem => mark_bad(ui, mark, pal.bad, fill),
        Severity::Warning => mark_warn(ui, mark, pal.warn),
    }
    let text = paint_galley(
        ui,
        pos2(text_x, rect.top() + 10.0),
        Align2::LEFT_TOP,
        galley,
    );
    announce(ui, text, Role::Label, &format!("{bold}{rest}"));
    let clicked = button.is_some_and(|button| {
        let at = pos2(rect.right() - 13.0 - button.width(ui), rect.top() + 9.0);
        button.show(ui, pal, at, false).clicked()
    });
    (rect.bottom(), clicked)
}

// ---------------------------------------------------------------------------
// Empty state (C18)

/// What stands above the title of an empty-state block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyMark {
    None,
    Icon(Icon),
    /// The Mortise mark, on the first run.
    Brand,
}

/// The empty-state block (C18): left aligned, a mark, a title, a paragraph of at most
/// 46 characters a line, then room of `action_h` for the action. Centred in the
/// height of `body`. Returns where the action starts.
pub fn empty_state(
    ui: &Ui,
    pal: &Palette,
    body: Rect,
    mark: EmptyMark,
    title: &str,
    paragraphs: &[&str],
    action_h: f32,
) -> Pos2 {
    // 46 times the width of a zero in the body type.
    let text_w = 46.0 * 0.535 * ty::BODY.size;
    let title_type = ty::TITLE;

    let paragraphs: Vec<_> = paragraphs
        .iter()
        .map(|text| (galley_wrapped(ui, text, ty::BODY, pal.ink_2, text_w), *text))
        .collect();
    let mark_h = if mark == EmptyMark::None {
        0.0
    } else {
        28.0 + 12.0
    };
    let action = if action_h > 0.0 { 12.0 + action_h } else { 0.0 };
    let height = mark_h
        + title_type.line
        + paragraphs
            .iter()
            .map(|(galley, _)| 12.0 + galley.size().y)
            .sum::<f32>()
        + action;

    let left = body.left() + 28.0;
    let mut y = (body.center().y - height / 2.0)
        .round()
        .max(body.top() + 26.0);
    let mark_rect = Rect::from_min_size(pos2(left, y), Vec2::splat(28.0));
    match mark {
        EmptyMark::None => {}
        EmptyMark::Icon(icon) => self::icon(ui, mark_rect, icon, pal.ink),
        EmptyMark::Brand => brand_mark(ui, pal, mark_rect.expand(3.5)),
    }
    y += mark_h;
    let title_rect = paint_text(
        ui,
        pos2(left, y),
        Align2::LEFT_TOP,
        title,
        title_type,
        pal.ink,
    );
    announce(ui, title_rect, Role::Heading, title);
    y += title_type.line;
    for (galley, text) in paragraphs {
        y += 12.0;
        let rect = paint_galley(ui, pos2(left, y), Align2::LEFT_TOP, galley);
        announce(ui, rect, Role::Label, text);
        y = rect.bottom();
    }
    pos2(left, y + 12.0)
}

// ---------------------------------------------------------------------------
// One stop in the Tab order (K1): a list, a table, the tab row and a segmented
// control each take one Tab press. Arrows, or J and K, move inside them.

/// The sense of a part of a one-stop control: every part can be clicked, and only
/// the current one can hold focus.
pub fn one_stop(current: bool) -> Sense {
    if current {
        Sense::click()
    } else {
        Sense::CLICK
    }
}

/// Keeps the arrow keys with a focused control, so that egui does not use them
/// to move focus to another one.
pub fn keep_arrows(ui: &Ui, id: eframe::egui::Id, vertical: bool, horizontal: bool) {
    let filter = eframe::egui::EventFilter {
        vertical_arrows: vertical,
        horizontal_arrows: horizontal,
        ..Default::default()
    };
    ui.memory_mut(|memory| memory.set_focus_lock_filter(id, filter));
}

fn pending_focus_id() -> eframe::egui::Id {
    eframe::egui::Id::unique("mortise-focus-in-the-next-frame")
}

/// Gives focus to a control in the next frame. For a control that cannot hold
/// focus yet in this one: the part of a one-stop control that is about to be chosen.
pub fn focus_soon(ui: &Ui, id: eframe::egui::Id) {
    ui.ctx()
        .data_mut(|data| data.insert_temp(pending_focus_id(), Some(id)));
    ui.ctx().request_repaint();
}

/// Gives focus to the control [`focus_soon`] named in the last frame. Call at the
/// start of a frame.
pub fn focus_what_was_asked(ctx: &eframe::egui::Context) {
    let pending = ctx
        .data_mut(|data| data.remove_temp::<Option<eframe::egui::Id>>(pending_focus_id()))
        .flatten();
    if let Some(id) = pending {
        ctx.memory_mut(|memory| memory.request_focus(id));
    }
}

fn list_focus_id() -> eframe::egui::Id {
    eframe::egui::Id::unique("mortise-list-has-focus")
}

/// True when a row of a list or table had focus in the last frame. Asking forgets it.
/// While it is true, J, K and the arrows belong to that list, not to the module index.
pub fn take_list_focus(ctx: &eframe::egui::Context) -> bool {
    ctx.data_mut(|data| data.remove_temp::<bool>(list_focus_id()))
        .unwrap_or(false)
}

/// The rows of a list or table as one stop: one row is the current one and can
/// hold focus. J, K, Down, Up, Home and End move it while it has focus; Enter and
/// Space press it, as they press any focused control.
pub struct ListNav {
    salt: eframe::egui::Id,
    current: usize,
    /// The current row moved by a key in this frame: scroll it into view.
    pub moved: bool,
}

impl ListNav {
    /// `name` tells this list from the others on screen. `len` is how many rows it has.
    /// The list keeps its current row itself.
    pub fn new(ui: &Ui, name: impl std::hash::Hash + std::fmt::Debug, len: usize) -> Self {
        let salt = ui.scope_id().with(("list", &name));
        let mut current = ui.data(|data| data.get_temp::<usize>(salt)).unwrap_or(0);
        let nav = Self::over(ui, name, len, &mut current);
        ui.data_mut(|data| data.insert_temp(salt, current));
        nav
    }

    /// As [`ListNav::new`], for a list whose selected row the caller keeps: that row
    /// is the current one, and the keys move it.
    pub fn over(
        ui: &Ui,
        name: impl std::hash::Hash + std::fmt::Debug,
        len: usize,
        selected: &mut usize,
    ) -> Self {
        use eframe::egui::{Key, Modifiers};
        let salt = ui.scope_id().with(("list", name));
        let last = len.saturating_sub(1);
        let mut current = (*selected).min(last);
        let mut moved = false;
        if len > 0 && ui.memory(|memory| memory.has_focus(salt.with(current))) {
            ui.ctx()
                .data_mut(|data| data.insert_temp(list_focus_id(), true));
            keep_arrows(ui, salt.with(current), true, false);
            let target = ui.input_mut(|input| {
                let mut pressed = |key| input.consume_key(Modifiers::NONE, key);
                let mut target = current;
                if pressed(Key::J) || pressed(Key::ArrowDown) {
                    target = (current + 1).min(last);
                }
                if pressed(Key::K) || pressed(Key::ArrowUp) {
                    target = current.saturating_sub(1);
                }
                if pressed(Key::Home) {
                    target = 0;
                }
                if pressed(Key::End) {
                    target = last;
                }
                target
            });
            if target != current {
                current = target;
                moved = true;
                ui.memory_mut(|memory| memory.request_focus(salt.with(current)));
            }
        }
        *selected = current;
        Self {
            salt,
            current,
            moved,
        }
    }

    /// Gives focus to the current row.
    pub fn focus(&self, ui: &Ui) {
        ui.memory_mut(|memory| memory.request_focus(self.salt.with(self.current)));
    }

    /// True when the current row holds focus.
    pub fn has_focus(&self, ui: &Ui) -> bool {
        ui.memory(|memory| memory.has_focus(self.salt.with(self.current)))
    }

    /// The id of a row, for its interaction.
    pub fn id(&self, row: usize) -> eframe::egui::Id {
        self.salt.with(row)
    }

    /// The sense of a row: every row can be clicked, the current one can hold focus.
    pub fn sense(&self, row: usize) -> Sense {
        one_stop(row == self.current)
    }

    /// Call with the response of a row. A row that was clicked becomes the current
    /// one; the current row is scrolled into view when a key moved it.
    pub fn after(&self, ui: &Ui, row: usize, response: &Response) {
        if response.clicked() && row != self.current {
            ui.data_mut(|data| data.insert_temp(self.salt, row));
        }
        if self.moved && row == self.current {
            response.scroll_to_me(None);
        }
    }
}

// ---------------------------------------------------------------------------
// Tooltip (C12)

/// Gives a control a tooltip: key fill, key-fg text, at most 240 px wide, no shadow.
/// A tooltip names; it does not instruct.
pub fn tooltip(response: Response, pal: &Palette, text: &str) -> Response {
    response.on_hover_ui(|ui| {
        ui.set_max_width(size::TOOLTIP_MAX_W);
        let format = ty::SECONDARY.format(pal.key_fg);
        ui.label(LayoutJob::single_section(text.to_owned(), format));
    })
}

/// A tooltip that also shows the shortcut of the control: a shortcut is shown where
/// its action is.
pub fn tooltip_keys(response: Response, pal: &Palette, text: &str, keys: &str) -> Response {
    response.on_hover_ui(|ui| {
        ui.set_max_width(size::TOOLTIP_MAX_W);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.label(LayoutJob::single_section(
                text.to_owned(),
                ty::SECONDARY.format(pal.key_fg),
            ));
            // The key hint, on the key fill of the tooltip.
            let cap = galley(ui, keys, ty::KEYCAP, pal.key_fg);
            let (rect, _) =
                ui.allocate_exact_size(vec2(cap.size().x.ceil() + 8.0, 16.0), Sense::hover());
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(radius::STAMP),
                Stroke::new(stroke::HAIRLINE, pal.key_fg),
                StrokeKind::Inside,
            );
            paint_galley(ui, rect.center(), Align2::CENTER_CENTER, cap);
            announce(ui, rect, Role::Label, keys);
        });
    })
}

// ---------------------------------------------------------------------------
// Segmented control (C5)

/// Paints a segmented control with its top left at `min`: a 26 px track with
/// segments of 20 px. The selected segment is a surface piece with an ink outline.
/// Returns its rect and the segment that was clicked.
pub fn segmented(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    name: &str,
    labels: &[&str],
    selected: usize,
    keyboard: bool,
) -> (Rect, Option<usize>) {
    segmented_with(ui, pal, min, name, labels, selected, None, keyboard)
}

/// The width of the track of a segmented control with `labels`, before it is painted.
pub fn segmented_width(ui: &Ui, labels: &[&str]) -> f32 {
    track_width(&segment_widths(ui, labels))
}

/// The text of a segment.
const SEGMENT_LABEL: Type = Type {
    size: 12.0,
    line: 12.0,
    ..ty::CONTROL
};

/// The width of each segment: its label and 10 px on each side.
fn segment_widths(ui: &Ui, labels: &[&str]) -> Vec<f32> {
    labels
        .iter()
        .map(|label| {
            let text = galley(ui, label, SEGMENT_LABEL, Color32::PLACEHOLDER);
            (text.size().x + 20.0).ceil()
        })
        .collect()
}

/// The track holds the segments, 2 px apart, with 3 px of padding at each end.
fn track_width(widths: &[f32]) -> f32 {
    3.0 + widths.iter().sum::<f32>() + 2.0 * (widths.len() as f32 - 1.0) + 3.0
}

/// A segmented control in which one segment can be off: `off` is its place and its
/// tooltip. An off segment cannot be clicked, and Left and Right pass over it.
#[expect(clippy::too_many_arguments)]
pub fn segmented_with(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    name: &str,
    labels: &[&str],
    selected: usize,
    off: Option<(usize, &str)>,
    keyboard: bool,
) -> (Rect, Option<usize>) {
    let is_off = |i: usize| off.is_some_and(|(place, _)| place == i);
    let widths = segment_widths(ui, labels);
    let width = track_width(&widths);
    let track = Rect::from_min_size(min, vec2(width, size::CONTROL_H));
    outlined(ui, track, radius::CONTROL, pal.well, pal.line_2);

    let mut clicked = None;
    let mut x = track.left() + 3.0;
    for (i, label) in labels.iter().enumerate() {
        let segment = Rect::from_min_size(pos2(x, track.top() + 3.0), vec2(widths[i], 20.0));
        x += widths[i] + 2.0;
        let id = ui.scope_id().with(("segment", name, i));
        // A segment is drawn 20 px high; its target is as high as the track.
        let target = segment.expand2(vec2(0.0, 3.0));
        let sense = if is_off(i) {
            Sense::hover()
        } else {
            one_stop(i == selected)
        };
        let mut response = ui.interact(target, id, sense);
        response.widget_info(|| {
            WidgetInfo::selected(Role::RadioButton, !is_off(i), i == selected, *label)
        });
        if let Some((place, tip)) = off
            && place == i
        {
            response = tooltip(response, pal, tip);
        }
        if response.clicked() {
            clicked = Some(i);
        }
        // Left and Right change the choice while the control has focus.
        if i == selected && response.has_focus() {
            use eframe::egui::{Key, Modifiers};
            keep_arrows(ui, id, false, true);
            let (left, right) = ui.input_mut(|input| {
                (
                    input.consume_key(Modifiers::NONE, Key::ArrowLeft),
                    input.consume_key(Modifiers::NONE, Key::ArrowRight),
                )
            });
            let step = |from: usize, back: bool| {
                if back {
                    from.checked_sub(1)
                } else {
                    Some(from + 1).filter(|next| *next < labels.len())
                }
            };
            let mut target = match (left, right) {
                (true, _) => step(i, true),
                (_, true) => step(i, false),
                _ => None,
            };
            if let Some(over) = target.filter(|place| is_off(*place)) {
                target = step(over, left);
            }
            if let Some(target) = target {
                clicked = Some(target);
                focus_soon(ui, ui.scope_id().with(("segment", name, target)));
            }
        }
        let color = if i == selected {
            outlined(ui, segment, radius::STAMP, pal.surface, pal.ink);
            if is_off(i) { pal.ink_3 } else { pal.ink }
        } else if is_off(i) {
            pal.ink_3
        } else if response.hovered() {
            pal.ink
        } else {
            pal.ink_2
        };
        paint_text(
            ui,
            segment.center(),
            Align2::CENTER_CENTER,
            label,
            SEGMENT_LABEL,
            color,
        );
        if keyboard && response.has_focus() {
            focus_ring(ui, pal, segment, radius::STAMP);
        }
    }
    (track, clicked)
}

// ---------------------------------------------------------------------------
// Checkbox (C4)

/// Paints a checkbox row with its top left at `min`: a 14 px box and a label, 24 px
/// high. The whole row is the target. Returns its response; `clicked` switches it.
pub fn checkbox(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    label: &str,
    checked: bool,
    disabled: bool,
    keyboard: bool,
) -> Response {
    checkbox_in(
        ui,
        pal,
        min,
        f32::INFINITY,
        label,
        checked,
        disabled,
        keyboard,
    )
}

/// A checkbox row whose label wraps at `width`. The row grows by the lines the label
/// adds: see `checkbox_extra`.
#[expect(clippy::too_many_arguments)]
pub fn checkbox_in(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    label: &str,
    checked: bool,
    disabled: bool,
    keyboard: bool,
) -> Response {
    let color = if disabled { pal.ink_3 } else { pal.ink };
    let text = galley_wrapped(ui, label, ty::BODY, color, width);
    // The box and the first line of the label are centred in the first 24 px.
    let first_mid = min.y + size::HIT_MIN / 2.0;
    let line_h = text.size().y / text.rows.len().max(1) as f32;
    let text_top = first_mid - line_h / 2.0;
    let row = Rect::from_min_size(
        min,
        vec2(
            size::CHECKBOX + 8.0 + text.size().x,
            size::HIT_MIN + text.size().y - line_h,
        ),
    );
    let sense = if disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let response = ui.interact(row, ui.scope_id().with(("checkbox", label)), sense);
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, !disabled, checked, label));

    let mark = Rect::from_min_size(
        pos2(row.left(), (first_mid - size::CHECKBOX / 2.0).round()),
        Vec2::splat(size::CHECKBOX),
    );
    checkbox_box(ui, pal, mark, checked, disabled, response.hovered());
    paint_galley(
        ui,
        pos2(mark.right() + 8.0, text_top),
        Align2::LEFT_TOP,
        text,
    );
    if keyboard && response.has_focus() {
        focus_ring(ui, pal, row, radius::STAMP);
    }
    response
}

/// How much higher than 24 px `checkbox_in` draws a row whose label wraps at `width`.
pub fn checkbox_extra(ui: &Ui, width: f32, label: &str) -> f32 {
    let text = galley_wrapped(ui, label, ty::BODY, Color32::PLACEHOLDER, width);
    text.size().y - text.size().y / text.rows.len().max(1) as f32
}

/// Paints the 14 px box of a checkbox into `mark`, with its tick when checked.
pub fn checkbox_box(
    ui: &Ui,
    pal: &Palette,
    mark: Rect,
    checked: bool,
    disabled: bool,
    hovered: bool,
) {
    let (fill, outline, tick) = match (checked, disabled) {
        (true, true) => (pal.line, pal.line, pal.ink_2),
        (false, true) => (pal.well, pal.line, pal.ink_2),
        (true, false) => (pal.key, pal.key, pal.key_fg),
        (false, false) => (
            pal.surface,
            if hovered { pal.ink } else { pal.line_2 },
            pal.key_fg,
        ),
    };
    ui.painter().rect(
        mark,
        CornerRadius::same(radius::STAMP),
        fill,
        Stroke::new(stroke::CHECKBOX, outline),
        StrokeKind::Inside,
    );
    if checked {
        // The tick, on a 10 px grid inside the box.
        let at = |x: f32, y: f32| mark.min + vec2(2.0 + x, 2.0 + y);
        let points = vec![at(1.6, 5.2), at(4.0, 7.6), at(8.4, 2.6)];
        ui.painter()
            .add(Shape::line(points, Stroke::new(1.8, tick)));
    }
}

// ---------------------------------------------------------------------------
// Text field (C2)

/// Paints a text field into `rect`: 28 px high, an outline that is ink with focus
/// and bad with an error. Paths, names and commands are set in Inconsolata.
pub fn text_field(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    name: &str,
    text: &mut String,
    mono: bool,
    keyboard: bool,
) -> Response {
    field(ui, pal, rect, name, text, "", mono, false, keyboard)
}

/// A text field with a placeholder that shows while it is empty.
#[expect(clippy::too_many_arguments)]
pub fn text_field_with_hint(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    name: &str,
    text: &mut String,
    hint: &str,
    mono: bool,
    keyboard: bool,
) -> Response {
    field(ui, pal, rect, name, text, hint, mono, false, keyboard)
}

/// A text field for a secret: what is typed is drawn as dots, and there is no way
/// to reveal it.
pub fn secret_field(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    name: &str,
    text: &mut String,
    keyboard: bool,
) -> Response {
    field(ui, pal, rect, name, text, "", true, true, keyboard)
}

#[expect(clippy::too_many_arguments)]
fn field(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    name: &str,
    text: &mut String,
    hint: &str,
    mono: bool,
    secret: bool,
    keyboard: bool,
) -> Response {
    let id = ui.scope_id().with(("field", name));
    let focused = ui.memory(|memory| memory.has_focus(id));
    let outline = if focused {
        pal.ink
    } else if ui.rect_contains_pointer(rect) {
        pal.ink_2
    } else {
        pal.line_2
    };
    outlined(ui, rect, radius::CONTROL, pal.surface, outline);
    let font = if mono { ty::CODE } else { ty::BODY };
    let font = eframe::egui::FontId::new(font.size, font.face.family());
    let placeholder = eframe::egui::RichText::new(hint)
        .color(pal.ink_3)
        .font(font.clone());
    let edit = eframe::egui::TextEdit::singleline(text)
        .id(id)
        .password(secret)
        .hint_text(placeholder)
        .frame(eframe::egui::Frame::NONE)
        .margin(eframe::egui::Margin::ZERO)
        .font(font)
        .text_color(pal.ink)
        .desired_width(rect.width() - 18.0)
        .align(Align2::LEFT_CENTER);
    let response = ui.put(rect.shrink2(vec2(9.0, 1.0)), edit);
    let role = if secret {
        Role::PasswordInput
    } else {
        Role::TextInput
    };
    response.widget_info(|| WidgetInfo::labeled(role, true, name));
    if keyboard && focused {
        focus_ring(ui, pal, rect, radius::CONTROL);
    }
    response
}
