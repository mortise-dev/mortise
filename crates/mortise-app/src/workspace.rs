//! The workspace: a header with the title and the five tabs, and the body of the open tab.

use eframe::egui::{
    Align2, Color32, Key, Modifiers, Pos2, Rect, Role, Ui, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{Action, AppSnapshot};

use crate::actions_view::{self, ActionsInputs, ActionsState};
use crate::app::Selection;
use crate::boundary::{self, BoundaryAction, BoundaryState};
use crate::draw::{self, Button, ButtonKind, Icon, StampState};
use crate::findings_view;
use crate::generate_view::{self, GenerateInputs, GenerateState};
use crate::strings;
use crate::summary::{self, SummaryState};
use crate::surface_view::SurfaceInputs;
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
            Tab::Boundary => None,
            Tab::Generate => None,
            Tab::Actions => None,
            // The spec gives the text of Generate and says Metrics uses the same block.
            // This one is written to match it.
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
/// What the tabs of the workspace keep between frames.
pub struct WorkState<'a> {
    /// The display name of the app.
    pub name: &'a str,
    pub summary: SummaryState<'a>,
    pub boundary: &'a mut BoundaryState,
    /// The Findings list is open: a second view of the Summary of "All modules".
    pub findings_open: &'a mut bool,
    /// The row of the Findings list that is selected.
    pub finding_row: &'a mut usize,
    /// True while the keyboard is in use: focus rings are drawn.
    pub keyboard: bool,
    /// What a scan of the public surface takes from the running app.
    pub surface: SurfaceInputs<'a>,
    pub mac: bool,
    /// What the Actions tab keeps, and what it needs.
    pub actions: &'a mut ActionsState,
    pub actions_inputs: ActionsInputs<'a>,
    /// What the Generate tab keeps, and what it needs.
    pub generate: &'a mut GenerateState,
    pub generate_inputs: GenerateInputs<'a>,
}

/// Returns what was asked for in the workspace, such as a module to select in the index.
pub fn workspace(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    app: &AppSnapshot,
    selection: &Selection,
    tab: &mut Tab,
    state: WorkState<'_>,
) -> Option<BoundaryAction> {
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

    // The Findings list takes the place of the Summary of "All modules" until Esc.
    let findings = *state.findings_open && module.is_none() && *tab == Tab::Summary;
    let findings_title = strings::findings_in(state.name);
    let mut title_x = head.left() + 16.0;
    // The Generate tab names what it generates in (S9.1, S9.4).
    let generate_title = generate_view::title(state.generate, module, state.name);
    let generating = *tab == Tab::Generate && !findings;
    let title = if generating {
        generate_title.as_str()
    } else if findings {
        // Where the list belongs, then its own name.
        let crumb = format!("{} /", strings::ALL_MODULES);
        let at = pos2(title_x, head.top() + 12.0 + ty::TITLE.line - 3.0);
        let crumb_rect = draw::label(
            ui,
            at,
            Align2::LEFT_BOTTOM,
            &crumb,
            ty::SECONDARY,
            pal.ink_2,
        );
        title_x = crumb_rect.right() + 10.0;
        let back = Button {
            label: strings::BACK_TO_SUMMARY,
            kind: ButtonKind::Ghost,
            small: true,
            keys: Some("Esc"),
            ..Default::default()
        };
        let at = pos2(head.right() - 16.0 - back.width(ui), head.top() + 12.0);
        if back.show(ui, pal, at, state.keyboard).clicked() {
            *state.findings_open = false;
        }
        findings_title.as_str()
    } else {
        module.map_or(strings::ALL_MODULES, |module| module.name.as_str())
    };
    let title_rect = draw::paint_text(
        ui,
        pos2(title_x, head.top() + 12.0),
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
        if !generating {
            draw::label(
                ui,
                pos2(stamp.right() + 10.0, title_rect.bottom() - 2.0),
                Align2::LEFT_BOTTOM,
                &app.path_shown(&module.dir),
                ty::PATH,
                pal.ink_2,
            );
        }
    }

    // The actions of the header come before the tabs in the Tab order, so that the
    // first control after the tab row is the first control of the tab. The Actions
    // tab shows the key that runs instead (S10.1).
    let mut header_action = None;
    let run_with = match *tab {
        Tab::Actions => true,
        // The picker has no key that runs; a form does (S9.1, S9.2).
        Tab::Generate => state.generate.shows_form(),
        _ => false,
    };
    if run_with {
        let keys = actions_view::run_keys(state.mac);
        let mid = head.top() + 12.0 + ty::TITLE.line / 2.0;
        let cap = draw::keycap(ui, pal, pos2(head.right() - 16.0, mid), &keys);
        draw::label(
            ui,
            pos2(cap.left() - 6.0, mid),
            Align2::RIGHT_CENTER,
            strings::RUN_WITH,
            ty::SECONDARY,
            pal.ink_2,
        );
    } else if let Some(module) = module.filter(|_| *tab != Tab::Generate) {
        let said = strings::open_module_in_editor(&module.name);
        let open = Button {
            label: "Open in editor",
            small: true,
            said: Some(&said),
            ..Default::default()
        };
        let tests = Button {
            label: strings::action_button(Action::RunTests),
            small: true,
            icon: Some(Icon::Test),
            ..Default::default()
        };
        // "Migrate (1)" once the migration state is read (S4.3, design D7).
        let migrate_label = match app.package_data.migrations.get(&module.name) {
            Some(state) => strings::migrate_counted(state.waiting.len()),
            None => strings::action_button(Action::Migrate).to_owned(),
        };
        let migrate = Button {
            label: &migrate_label,
            kind: ButtonKind::Primary,
            small: true,
            icon: Some(Icon::Migrate),
            ..Default::default()
        };
        let y = head.top() + 12.0;
        let open_x = head.right() - 16.0 - open.width(ui);
        let migrate_x = open_x - 8.0 - migrate.width(ui);
        let tests_x = migrate_x - 8.0 - tests.width(ui);
        let buttons = [
            (tests, tests_x, Some(Action::RunTests)),
            (migrate, migrate_x, Some(Action::Migrate)),
            (open, open_x, None),
        ];
        for (button, x, action) in buttons {
            if button.show(ui, pal, pos2(x, y), state.keyboard).clicked() {
                header_action = Some(match action {
                    Some(action) => BoundaryAction::OpenAction(action),
                    None => BoundaryAction::OpenInEditor {
                        path: module.dir.clone(),
                        line: 1,
                    },
                });
            }
        }
    }

    let mut moved_to = None;
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

        // The tab row is one stop: the selected tab holds focus.
        let id = tab_id(ui, candidate);
        let response = ui.interact(tab_rect, id, draw::one_stop(candidate == *tab));
        response.widget_info(|| {
            WidgetInfo::selected(Role::Tab, true, candidate == *tab, candidate.label())
        });
        if response.clicked() {
            *tab = candidate;
        }
        // Left and Right change the tab while the row has focus.
        if candidate == *tab && response.has_focus() {
            draw::keep_arrows(ui, id, false, true);
            let at = Tab::ALL
                .iter()
                .position(|other| *other == candidate)
                .unwrap_or(0);
            let (left, right) = ui.input_mut(|input| {
                (
                    input.consume_key(Modifiers::NONE, Key::ArrowLeft),
                    input.consume_key(Modifiers::NONE, Key::ArrowRight),
                )
            });
            let target = match (left, right) {
                (true, _) if at > 0 => Some(Tab::ALL[at - 1]),
                (_, true) if at + 1 < Tab::ALL.len() => Some(Tab::ALL[at + 1]),
                _ => None,
            };
            if let Some(target) = target {
                moved_to = Some(target);
                draw::focus_soon(ui, tab_id(ui, target));
            }
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

    if let Some(target) = moved_to {
        *tab = target;
    }

    let body = Rect::from_min_max(pos2(rect.left(), head.bottom()), rect.max);
    if findings {
        return findings_view::findings(ui, pal, body, app, state.name, state.finding_row);
    }
    let in_body = match (*tab, tab.explanation()) {
        (_, Some((icon, title, text))) => {
            empty_state(ui, pal, body, EmptyMark::Icon(icon), title, &[text], 0.0);
            None
        }
        (Tab::Generate, None) => {
            let WorkState {
                generate,
                generate_inputs,
                ..
            } = state;
            generate_view::generate(ui, pal, body, app, module, generate, &generate_inputs)
        }
        (Tab::Actions, None) => {
            let WorkState {
                actions,
                actions_inputs,
                ..
            } = state;
            actions_view::actions(ui, pal, body, app, module, actions, &actions_inputs)
        }
        (Tab::Boundary, None) => {
            let WorkState {
                name,
                boundary,
                surface,
                keyboard,
                mac,
                ..
            } = state;
            boundary::boundary(
                ui, pal, body, app, name, module, boundary, &surface, keyboard, mac,
            )
        }
        (_, None) => summary::summary(ui, pal, body, app, state.name, module, state.summary),
    };
    in_body.or(header_action)
}

/// The id of a tab, to give it focus.
pub fn tab_id(ui: &Ui, tab: Tab) -> eframe::egui::Id {
    ui.scope_id().with(("tab", tab.digit()))
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
