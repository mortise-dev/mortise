//! The chart and the sparklines of the Metrics tab (S7.3), drawn with the painter.
//!
//! All text comes from the caller. The chart follows the viewBox of S7.3: 132 px high, the plot
//! 40 px from the left and 10 px from the right, from 10.5 px to 118.5 px from the top.

use std::time::Duration;

use eframe::egui::{Align2, Color32, Pos2, Rect, Role, Shape, Stroke, Ui, pos2};

use mortise_core::Line;

use crate::metrics::strings;
use crate::ui::draw;
use crate::ui::theme::{Face, Palette, Type, ty};

/// What the chart draws. All text comes from the caller.
pub struct Plot<'a> {
    pub line: &'a Line,
    pub span: Duration,
    /// The accessible name, such as "Requests per minute, all modules, last hour".
    pub name: &'a str,
    /// The text summary, given as the accessible description.
    pub summary: &'a str,
    /// Under the two ends of the time axis.
    pub from: &'a str,
    pub to: &'a str,
}

/// The height of the chart in its box, as the viewBox of S7.3.
pub const CHART_H: f32 = 132.0;

/// The left edge of the plot, from the left of the chart.
const PLOT_LEFT: f32 = 40.0;
/// The right edge of the plot, from the right of the chart.
const PLOT_RIGHT: f32 = 10.0;
/// The top grid line, from the top of the chart. Its 1 px line starts half a pixel above.
const PLOT_TOP: f32 = 10.5;
/// The space between two grid lines.
const GRID_GAP: f32 = 36.0;
/// The number of steps from the bottom grid line to the top one.
const STEPS: u32 = 3;
/// The gap between a value label and the plot.
const LABEL_GAP: f32 = 6.0;
/// The centre of the time labels, from the top of the chart: the baseline 131 less half the x-height.
const TIME_Y: f32 = 127.5;
/// The width of the line of the chart.
const LINE_W: f32 = 1.5;
/// The width of the line of a sparkline.
const SPARK_W: f32 = 1.2;

/// The text of the axes: Inconsolata 400 at 10.5 px.
const AXIS: Type = ty::KEYCAP.with_face(Face::Mono400);

/// Draws the chart of one line in `rect`, which is `CHART_H` high.
pub fn chart(ui: &Ui, pal: &Palette, rect: Rect, plot: &Plot<'_>) {
    let left = rect.left() + PLOT_LEFT;
    let right = rect.right() - PLOT_RIGHT;
    let top = rect.top() + PLOT_TOP;
    let bottom = top + GRID_GAP * STEPS as f32;
    let step = step(highest(plot.line));

    for i in 0..=STEPS {
        let y = bottom - GRID_GAP * i as f32;
        draw::hline(ui, left..right, y - 0.5, pal.line);
        draw::paint_text(
            ui,
            pos2(left - LABEL_GAP, y),
            Align2::RIGHT_CENTER,
            &strings::per_minute(step * f64::from(i)),
            AXIS,
            pal.ink_2,
        );
    }
    let time_y = rect.top() + TIME_Y;
    draw::paint_text(
        ui,
        pos2(left, time_y),
        Align2::LEFT_CENTER,
        plot.from,
        AXIS,
        pal.ink_2,
    );
    draw::paint_text(
        ui,
        pos2(right, time_y),
        Align2::RIGHT_CENTER,
        plot.to,
        AXIS,
        pal.ink_2,
    );

    let area = Rect::from_min_max(pos2(left, top), pos2(right, bottom));
    let top_value = step * f64::from(STEPS);
    draw_line(
        ui, area, plot.line, plot.span, top_value, pal.ink, LINE_W, false,
    );

    let node = draw::announce(ui, rect, Role::Image, plot.name);
    ui.ctx().accesskit_node_builder(node.id, |builder| {
        builder.set_description(plot.summary);
    });
}

/// Draws the sparkline of a row in `rect`, 58 by 14 px, scaled to the row's own highest point.
/// Off screen it draws nothing and keeps its accessible node.
pub fn spark(ui: &Ui, pal: &Palette, rect: Rect, line: &Line, span: Duration, name: &str) {
    // Room for half the stroke on every side, as the paths of S7.3 keep inside the box.
    // A row scrolled out of view builds no path: a module can have thousands of routes.
    if ui.is_rect_visible(rect) {
        let area = rect.shrink2(eframe::egui::vec2(1.0, 1.0));
        draw_line(
            ui,
            area,
            line,
            span,
            highest(line),
            pal.ink_2,
            SPARK_W,
            true,
        );
    }
    draw::announce(ui, rect, Role::Image, name);
}

/// The highest point of a line, or zero.
fn highest(line: &Line) -> f64 {
    line.runs
        .iter()
        .flatten()
        .map(|point| point.per_minute)
        .fold(0.0, f64::max)
}

/// The step of the scale: the smallest of 1, 2 and 5 times a power of ten that is at least a third
/// of the highest point. With no point above 0 the step is 1. A third that is not a normal number
/// (zero, subnormal, infinite or not a number) also gives 1: a subnormal power of ten is 0.
fn step(highest: f64) -> f64 {
    let least = highest / f64::from(STEPS);
    if !least.is_normal() || least < 0.0 {
        return 1.0;
    }
    let mut power = 10f64.powf(least.log10().floor());
    loop {
        for factor in [1.0, 2.0, 5.0] {
            let step = factor * power;
            // A relative margin keeps 0.6 / 3 from reaching 0.5 through rounding.
            if step >= least * (1.0 - 1e-9) {
                return step;
            }
        }
        power *= 10.0;
    }
}

/// Draws each run of `line` in `area`: `ago` zero at the right edge, `span` at the left edge,
/// zero at the bottom and `top_value` at the top. A run of one point is a dot. With `thin`, points
/// in the same pixel column are drawn as their average, so a line of thousands of points in a
/// few dozen pixels stays a line and does not fill the box.
#[allow(clippy::too_many_arguments)]
fn draw_line(
    ui: &Ui,
    area: Rect,
    line: &Line,
    span: Duration,
    top_value: f64,
    color: Color32,
    width: f32,
    thin: bool,
) {
    let span = span.as_secs_f64();
    let place = |ago: Duration, value: f64| -> Pos2 {
        let back = if span > 0.0 {
            (ago.as_secs_f64() / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let up = if top_value > 0.0 {
            (value / top_value).clamp(0.0, 1.0)
        } else {
            0.0
        };
        pos2(
            area.right() - back as f32 * area.width(),
            area.bottom() - up as f32 * area.height(),
        )
    };
    for run in &line.runs {
        let mut points: Vec<Pos2> = run
            .iter()
            .map(|point| place(point.ago, point.per_minute))
            .collect();
        if thin {
            points = one_per_column(&points, ui.pixels_per_point());
        }
        match points.as_slice() {
            [] => {}
            [dot] => {
                ui.painter().circle_filled(*dot, width / 2.0, color);
            }
            _ => {
                ui.painter()
                    .add(Shape::line(points, Stroke::new(width, color)));
            }
        }
    }
}

/// The points of a run with those that share a pixel column replaced by their average. The points
/// come in order, so the ones that share a column are next to each other.
fn one_per_column(points: &[Pos2], pixels_per_point: f32) -> Vec<Pos2> {
    let column = |point: &Pos2| (point.x * pixels_per_point).floor();
    let mut thinned = Vec::new();
    let mut rest = points;
    while let Some(first) = rest.first() {
        let same = rest
            .iter()
            .take_while(|point| column(point) == column(first))
            .count();
        let (group, tail) = rest.split_at(same);
        let sum = group
            .iter()
            .fold(Pos2::ZERO, |sum, point| sum + point.to_vec2());
        thinned.push(sum / group.len() as f32);
        rest = tail;
    }
    thinned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_in_one_pixel_column_are_averaged() {
        let points = [
            pos2(10.1, 4.0),
            pos2(10.7, 8.0),
            pos2(11.2, 2.0),
            pos2(12.0, 6.0),
        ];
        // At scale 1 the first two share column 10, and the others stand alone.
        assert_eq!(
            one_per_column(&points, 1.0),
            [pos2(10.4, 6.0), pos2(11.2, 2.0), pos2(12.0, 6.0)]
        );
        // At scale 2 each of them has its own column.
        assert_eq!(one_per_column(&points, 2.0), points);
    }

    #[test]
    fn the_step_is_1_2_or_5_times_a_power_of_ten() {
        let cases = [
            (0.0, 1.0),
            (0.4, 0.2),
            (0.6, 0.2),
            (3.0, 1.0),
            (31.0, 20.0),
            (900.0, 500.0),
            (1_020.0, 500.0),
            (1_600.0, 1_000.0),
            (f64::MIN_POSITIVE / 4.0, 1.0),
        ];
        for (highest, expected) in cases {
            assert_eq!(step(highest), expected, "highest {highest}");
        }
        assert!(step(f64::MAX) >= f64::MAX / 3.0);
    }
}
