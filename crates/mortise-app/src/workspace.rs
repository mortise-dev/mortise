//! The workspace: a header with the title and the five tabs, and the body of the open tab.

use eframe::egui::{Align2, Color32, Pos2, Rect, Role, Sense, Ui, Vec2, WidgetInfo, pos2, vec2};
use mortise_core::AppSnapshot;

use crate::app::Selection;
use crate::draw::{self, Icon, StampState};
use crate::strings;
use crate::summary::{self, SummaryState};
use crate::theme::{Face, Palette, size, ty};

/// The five tabs. They are in every build, in this order, with the keys 1 to 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Summary,
    Boundary,
    Generate,
    Actions,
    Metrics,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Summary,
        Tab::Boundary,
        Tab::Generate,
        Tab::Actions,
        Tab::Metrics,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Summary => "Summary",
            Tab::Boundary => "Boundary",
            Tab::Generate => "Generate",
            Tab::Actions => "Actions",
            Tab::Metrics => "Metrics",
        }
    }

    /// The digit that opens the tab.
    pub fn digit(self) -> &'static str {
        match self {
            Tab::Summary => "1",
            Tab::Boundary => "2",
            Tab::Generate => "3",
            Tab::Actions => "4",
            Tab::Metrics => "5",
        }
    }

    /// A tab that cannot do its work yet explains itself (S7.4): icon, title, paragraph.
    fn explanation(self) -> Option<(Icon, &'static str, &'static str)> {
        match self {
            Tab::Summary => None,
            // Not in the spec, and only until milestone 4 builds the tab.
            Tab::Boundary => Some((
                Icon::Boundary,
                "Boundary is not built yet",
                "Boundary shows which module uses which, and where a rule is broken. \
                 It arrives with milestone 4 of this version.",
            )),
            Tab::Generate => Some((
                Icon::Generate,
                "Generate is not in this version",
                "Generate creates modules and files through the package's own commands, with a \
                 preview before anything is written. It needs PHP and a newer \
                 nwidart/laravel-modules, and arrives in a later release of Mortise.",
            )),
            // The spec gives the text of Generate and says Actions and Metrics use the
            // same block. These two are written to match it.
            Tab::Actions => Some((
                Icon::Run,
                "Actions are not in this version",
                "Actions run the package's commands for a module: turn it on or off, migrate, \
                 roll back, seed and run its tests, with the output on screen. They need PHP \
                 and a newer nwidart/laravel-modules, and arrive in a later release of Mortise.",
            )),
            Tab::Metrics => Some((
                Icon::Metrics,
                "Metrics are not in this version",
                "Metrics show requests, response time, errors, jobs and database time per \
                 module, read from the running app. They need a metrics endpoint that \
                 nwidart/laravel-actuator does not have yet, and arrive in a later release \
                 of Mortise.",
            )),
        }
    }
}

/// Paints the workspace into `rect`.
/// Returns the name of a module that was chosen in the workspace, to select it in the index.
pub fn workspace(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    app: &AppSnapshot,
    selection: &Selection,
    tab: &mut Tab,
    summary: SummaryState<'_>,
) -> Option<String> {
    ui.painter().rect_filled(rect, 0, pal.surface);
    let module = match selection {
        Selection::All => None,
        Selection::Module(name) => app.modules.iter().find(|module| &module.name == name),
    };

    // Header strip: the title, then the tabs, joined to the body by the selected tab.
    let head = Rect::from_min_size(rect.min, vec2(rect.width(), size::WORKSPACE_HEAD_H));
    ui.painter().rect_filled(head, 0, pal.panel);
    draw::hline(
        ui,
        head.left()..head.right(),
        head.bottom() - 1.0,
        pal.line_2,
    );

    let title = module.map_or(strings::ALL_MODULES, |module| module.name.as_str());
    let title_rect = draw::paint_text(
        ui,
        pos2(head.left() + 16.0, head.top() + 12.0),
        Align2::LEFT_TOP,
        title,
        ty::TITLE,
        pal.ink,
    );
    draw::announce(ui, title_rect, Role::Heading, title);
    if let Some(module) = module {
        let stamp_min = pos2(
            title_rect.right() + 10.0,
            (title_rect.center().y - size::STAMP_H / 2.0 + 1.0).round(),
        );
        let stamp = draw::stamp(ui, pal, stamp_min, &module.code, StampState::default());
        let named = draw::announce(ui, stamp, Role::Label, &module.code);
        draw::tooltip(
            named,
            pal,
            &strings::code_is_for(&module.code, &module.name),
        );
        draw::label(
            ui,
            pos2(stamp.right() + 10.0, title_rect.bottom() - 2.0),
            Align2::LEFT_BOTTOM,
            &app.path_shown(&module.dir),
            ty::PATH,
            pal.ink_2,
        );
    }

    let mut x = head.left() + 16.0;
    for candidate in Tab::ALL {
        let label_type = if candidate == *tab {
            ty::TAB.with_face(Face::Sans600)
        } else {
            ty::TAB
        };
        let label_w = draw::galley(ui, candidate.label(), label_type, Color32::PLACEHOLDER)
            .size()
            .x;
        let digit_w = draw::galley(ui, candidate.digit(), ty::KEYCAP, Color32::PLACEHOLDER)
            .size()
            .x;
        let width = (1.0 + 12.0 + label_w + 6.0 + digit_w + 12.0 + 1.0).ceil();
        let tab_rect = Rect::from_min_size(
            pos2(x, head.bottom() - size::TAB_H),
            vec2(width, size::TAB_H),
        );
        x += width;

        let response = ui.interact(
            tab_rect,
            ui.scope_id().with(("tab", candidate.digit())),
            Sense::click(),
        );
        response.widget_info(|| {
            WidgetInfo::selected(Role::Tab, true, candidate == *tab, candidate.label())
        });
        if response.clicked() {
            *tab = candidate;
        }

        let selected = candidate == *tab;
        if selected {
            // The surface colour runs through the header's bottom line into the body.
            ui.painter().rect_filled(tab_rect, 0, pal.surface);
            draw::hline(
                ui,
                tab_rect.left()..tab_rect.right(),
                tab_rect.top(),
                pal.line_2,
            );
            draw::vline(
                ui,
                tab_rect.left(),
                tab_rect.top()..tab_rect.bottom(),
                pal.line_2,
            );
            draw::vline(
                ui,
                tab_rect.right() - 1.0,
                tab_rect.top()..tab_rect.bottom(),
                pal.line_2,
            );
        }
        let color = if selected || response.hovered() {
            pal.ink
        } else {
            pal.ink_2
        };
        let label_type = if selected {
            ty::TAB.with_face(Face::Sans600)
        } else {
            ty::TAB
        };
        let label = draw::paint_text(
            ui,
            pos2(tab_rect.left() + 13.0, tab_rect.top() + 7.0),
            Align2::LEFT_TOP,
            candidate.label(),
            label_type,
            color,
        );
        draw::paint_text(
            ui,
            pos2(label.right() + 6.0, label.center().y + 1.0),
            Align2::LEFT_CENTER,
            candidate.digit(),
            ty::KEYCAP,
            pal.ink_3,
        );
    }

    let body = Rect::from_min_max(pos2(rect.left(), head.bottom()), rect.max);
    match tab.explanation() {
        Some((icon, title, text)) => {
            empty_state(ui, pal, body, EmptyMark::Icon(icon), title, &[text], 0.0);
            None
        }
        None => summary::summary(ui, pal, body, app, module, summary),
    }
}

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
        .map(|text| {
            (
                draw::galley_wrapped(ui, text, ty::BODY, pal.ink_2, text_w),
                *text,
            )
        })
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
        EmptyMark::Icon(icon) => draw::icon(ui, mark_rect, icon, pal.ink),
        EmptyMark::Brand => draw::brand_mark(ui, pal, mark_rect.expand(3.5)),
    }
    y += mark_h;
    let title_rect = draw::paint_text(
        ui,
        pos2(left, y),
        Align2::LEFT_TOP,
        title,
        title_type,
        pal.ink,
    );
    draw::announce(ui, title_rect, Role::Heading, title);
    y += title_type.line;
    for (galley, text) in paragraphs {
        y += 12.0;
        let rect = draw::paint_galley(ui, pos2(left, y), Align2::LEFT_TOP, galley);
        draw::announce(ui, rect, Role::Label, text);
        y = rect.bottom();
    }
    pos2(left, y + 12.0)
}
