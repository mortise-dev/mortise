//! The Findings list (S6.4): every finding of the app, grouped by kind, and the
//! checks that found nothing. It is a second view of the Summary of "All modules".

use eframe::egui::{
    Align2, Color32, Rect, Role, ScrollArea, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{AppSnapshot, Check, Detail, Finding, Severity};

use crate::boundary::BoundaryAction;
use crate::draw::{self, StampState};
use crate::strings;
use crate::theme::{Face, Palette, Type, radius, size, ty};

/// Where Enter on a finding lands.
fn opens(finding: &Finding, app: &AppSnapshot) -> (&'static str, Option<BoundaryAction>) {
    let module = finding.module.clone();
    match (&finding.detail, module) {
        (Detail::CircularUse { .. }, Some(module)) => {
            let other = app.circle_neighbour(&module).map(str::to_owned);
            (
                strings::BOUNDARY,
                other.map(|other| BoundaryAction::ShowLink { module, other }),
            )
        }
        (Detail::ReadsInternals { of, .. }, Some(module)) => (
            strings::BOUNDARY,
            Some(BoundaryAction::ShowLink {
                module,
                other: of.clone(),
            }),
        ),
        (_, Some(module)) => (strings::SUMMARY, Some(BoundaryAction::Select(module))),
        // A finding of the app itself lives on this list.
        (_, None) => ("", None),
    }
}

/// Paints the list into `body`, which scrolls. `selected` is the row that has the peg.
pub fn findings(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    app: &AppSnapshot,
    name: &str,
    selected: &mut usize,
) -> Option<BoundaryAction> {
    let mut action = None;
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        ScrollArea::vertical()
            .id_salt("findings")
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let origin = ui.cursor().min;
                let width = ui.available_width();
                let left = origin.x + 16.0;
                let inner = width - 32.0;
                let mut y = origin.y + 14.0;

                if app.findings.list.is_empty() {
                    let (rules, checks) = app.findings.rules_and_checks();
                    let text = strings::no_findings(app.modules.len(), rules, checks);
                    y = draw::paragraph(ui, pos2(left, y), &text, ty::BODY, pal.ink, inner)
                        .bottom()
                        + 12.0;
                } else {
                    y = list(
                        ui,
                        pal,
                        pos2(left, y),
                        inner,
                        app,
                        name,
                        selected,
                        &mut action,
                    ) + 12.0;
                }
                y = quiet_checks(ui, pal, pos2(left, y), inner, app);
                ui.allocate_rect(
                    Rect::from_min_max(origin, pos2(origin.x + width, y + 16.0)),
                    Sense::hover(),
                );
            });
    });
    action
}

/// The findings, problems first, each with its module, its check and where it opens.
#[expect(clippy::too_many_arguments)]
fn list(
    ui: &Ui,
    pal: &Palette,
    min: eframe::egui::Pos2,
    width: f32,
    app: &AppSnapshot,
    name: &str,
    selected: &mut usize,
    action: &mut Option<BoundaryAction>,
) -> f32 {
    let module_x = min.x + 46.0;
    let text_x = min.x + 102.0;
    let check_x = (min.x + width * 0.655).round();
    let right = min.x + width - 14.0;
    let text_w = check_x - 16.0 - text_x;
    const GROUP_H: f32 = 30.0;

    // Lay the sentences out first: a row is as high as its sentence.
    let rows: Vec<_> = app
        .findings
        .list
        .iter()
        .map(|finding| {
            let sentence = strings::finding(finding, app, name);
            let galley = draw::galley_wrapped(ui, &sentence, ty::BODY, pal.ink, text_w);
            let height = (galley.size().y + 14.0).max(size::TABLE_ROW_H + 6.0);
            (finding, sentence, galley, height)
        })
        .collect();
    let groups = [Severity::Problem, Severity::Warning]
        .into_iter()
        .filter(|severity| {
            app.findings
                .list
                .iter()
                .any(|finding| finding.severity == *severity)
        })
        .count();
    let height = size::TABLE_HEAD_H
        + groups as f32 * GROUP_H
        + rows.iter().map(|row| row.3).sum::<f32>()
        + 1.0;
    let rect = Rect::from_min_size(min, vec2(width, height));
    draw::outlined(ui, rect, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);

    let head_mid = rect.top() + size::TABLE_HEAD_H / 2.0;
    for (x, align, label) in [
        (module_x, Align2::LEFT_CENTER, strings::COL_MODULE),
        (text_x, Align2::LEFT_CENTER, strings::COL_FINDING),
        (check_x, Align2::LEFT_CENTER, strings::COL_CHECK),
        (right, Align2::RIGHT_CENTER, strings::COL_ENTER_OPENS),
    ] {
        draw::label(ui, pos2(x, head_mid), align, label, ty::LABEL, pal.ink_2);
    }
    let mut y = rect.top() + size::TABLE_HEAD_H;
    draw::hline(
        ui,
        rect.left() + 1.0..rect.right() - 1.0,
        y - 1.0,
        pal.line_2,
    );

    let mut group = None;
    let last = rows.len().saturating_sub(1);
    // The list is one stop: J and K move through it, Enter opens a finding where it
    // lives. It takes focus when nothing else holds it, so the keys work at once.
    let nav = draw::ListNav::over(ui, "findings", rows.len(), selected);
    if !rows.is_empty() && ui.memory(|memory| memory.focused()).is_none() {
        nav.focus(ui);
    }
    for (index, (finding, sentence, galley, row_h)) in rows.into_iter().enumerate() {
        if group != Some(finding.severity) {
            group = Some(finding.severity);
            let (name, count) = match finding.severity {
                Severity::Problem => (strings::PROBLEMS, app.findings.problems()),
                Severity::Warning => (strings::WARNINGS, app.findings.warnings()),
            };
            let mid = y + GROUP_H / 2.0;
            draw::label(
                ui,
                pos2(rect.left() + 14.0, mid),
                Align2::LEFT_CENTER,
                name,
                ty::LABEL,
                pal.ink_2,
            );
            draw::label(
                ui,
                pos2(right, mid),
                Align2::RIGHT_CENTER,
                &count.to_string(),
                ty::SECONDARY,
                pal.ink_2,
            );
            y += GROUP_H;
            draw::hline(ui, rect.left() + 1.0..rect.right() - 1.0, y - 1.0, pal.line);
        }

        let row = Rect::from_min_max(
            pos2(rect.left() + 1.0, y),
            pos2(rect.right() - 1.0, y + row_h),
        );
        let (place, opens) = opens(finding, app);
        let is_selected = index == (*selected).min(last);
        let response = ui.interact(row, nav.id(index), nav.sense(index));
        nav.after(ui, index, &response);
        response.widget_info(|| WidgetInfo::selected(Role::Row, true, is_selected, &sentence));
        let behind = if is_selected {
            // A selected list row: well fill, a line above and below, and the peg.
            ui.painter().rect_filled(row, 0, pal.well);
            draw::hline(ui, row.left()..row.right(), row.top() - 1.0, pal.line_2);
            draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line_2);
            draw::peg(ui, pal, row);
            pal.well
        } else {
            if response.hovered() {
                ui.painter().rect_filled(row, 0, pal.well);
            }
            if index < last {
                draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line);
            }
            if response.hovered() {
                pal.well
            } else {
                pal.surface
            }
        };
        if response.clicked() {
            *selected = index;
            *action = opens;
        }

        let top_mid = row.top() + (size::TABLE_ROW_H + 6.0) / 2.0;
        let mark = Rect::from_min_size(
            pos2(row.left() + 14.0, top_mid - 6.0),
            Vec2::splat(size::STATE_MARK),
        );
        match finding.severity {
            Severity::Problem => draw::mark_bad(ui, mark, pal.bad, behind),
            Severity::Warning => draw::mark_warn(ui, mark, pal.warn),
        }
        if let Some(module) = finding.module.as_deref().and_then(|name| app.module(name)) {
            let stamp_min = pos2(module_x, (top_mid - size::STAMP_H / 2.0).round());
            draw::stamp(ui, pal, stamp_min, &module.code, StampState::default());
        }
        draw::paint_galley(ui, pos2(text_x, row.top() + 7.0), Align2::LEFT_TOP, galley);
        let check_name = strings::check_name(finding.check, app);
        let check =
            draw::galley_elided(ui, &check_name, ty::BODY, pal.ink_2, right - 80.0 - check_x);
        let check_rect = draw::paint_galley(ui, pos2(check_x, top_mid), Align2::LEFT_CENTER, check);
        draw::announce(ui, check_rect, Role::Label, &check_name);
        draw::paint_text(
            ui,
            pos2(right, top_mid),
            Align2::RIGHT_CENTER,
            place,
            ty::BODY,
            pal.ink_2,
        );
        y += row_h;
    }
    rect.bottom()
}

/// The checks that ran and found nothing, each with what it looked at.
fn quiet_checks(
    ui: &Ui,
    pal: &Palette,
    min: eframe::egui::Pos2,
    width: f32,
    app: &AppSnapshot,
) -> f32 {
    let quiet: Vec<_> = app
        .findings
        .checks
        .iter()
        .filter(|result| result.findings == 0)
        .collect();
    if quiet.is_empty() {
        return min.y - 12.0;
    }
    let rect = Rect::from_min_size(
        min,
        vec2(width, 41.0 + quiet.len() as f32 * size::LIST_LINE_H + 8.0),
    );
    let note = strings::of_checks(quiet.len(), app.findings.checks.len());
    let mut y = draw::box_head(ui, pal, rect, strings::CHECKS_WITHOUT_A_FINDING, &note);
    let _: Check = quiet[0].check;
    let value_type = Type {
        face: Face::Sans600,
        ..ty::SECONDARY
    };
    for (i, result) in quiet.iter().enumerate() {
        let mid = y + size::LIST_LINE_H / 2.0;
        let sentence = strings::check_passed(result.check, app);
        draw::label(
            ui,
            pos2(rect.left() + 13.0, mid),
            Align2::LEFT_CENTER,
            &sentence,
            ty::BODY,
            pal.ink,
        );
        let checked = strings::checked(result);
        let text = draw::label(
            ui,
            pos2(rect.right() - 13.0, mid),
            Align2::RIGHT_CENTER,
            &checked,
            value_type,
            pal.good,
        );
        let mark = Rect::from_min_size(
            pos2(text.left() - 5.0 - 12.0, mid - 6.0),
            Vec2::splat(size::STATE_MARK),
        );
        draw::mark_good(ui, mark, pal.good);
        if i + 1 < quiet.len() {
            draw::hline(
                ui,
                rect.left() + 13.0..rect.right() - 13.0,
                y + size::LIST_LINE_H - 1.0,
                pal.line,
            );
        }
        y += size::LIST_LINE_H;
    }
    rect.bottom()
}
