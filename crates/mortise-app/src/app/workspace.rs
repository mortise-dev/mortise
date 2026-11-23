//! The workspace: a header with the title and the five tabs, and the body of the open tab.

use eframe::egui::{Align2, Color32, Key, Modifiers, Rect, Role, Ui, WidgetInfo, pos2, vec2};
use mortise_core::{Action, AppSnapshot, Rules};

use crate::actions::view::{self as actions_view, ActionsInputs, ActionsState};
use crate::app::strings;
use crate::boundary::surface::SurfaceInputs;
use crate::boundary::view::{self as boundary, BoundaryState};
use crate::findings::view as findings_view;
use crate::generate::view::{self as generate_view, GenerateInputs, GenerateState};
use crate::index::view::Selection;
use crate::metrics::view::{self as metrics_view, MetricsView};
use crate::summary::view::{self as summary, SummaryState};
use crate::ui::draw::{self, Button, ButtonKind, Icon, StampState};
use crate::ui::theme::{Face, Palette, size, ty};

/// What a tab asks the app to do. Every tab returns it, not only Boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkAction {
    /// Select a module in the index.
    Select(String),
    /// Select a module and show its link with another.
    ShowLink { module: String, other: String },
    /// Change the rules of the app.
    SetRules(Rules),
    /// Copy the path of a file.
    CopyPath(String),
    /// Open the Findings list of the app.
    ShowFindings,
    /// Open a file or a folder in the editor, at a line.
    OpenInEditor { path: std::path::PathBuf, line: u32 },
    /// Something about the runtime of the app: another environment, a new try.
    Runtime(crate::runtime::view::RuntimeAction),
    /// Copy a text and say so.
    Copy { text: String, said: &'static str },
    /// Open the Actions tab of the selected module on this action. Nothing runs.
    OpenAction(mortise_core::Action),
    /// Open the Actions tab of a module on this action, from the Findings list. Nothing runs.
    OpenModuleAction {
        module: String,
        action: mortise_core::Action,
    },
    /// Start an action of the Actions tab, asking first where the rules say so.
    Act(crate::actions::view::Act),
    /// Read the migration state of a target with these artisan words, the line on
    /// screen (design D7).
    ReadMigrations {
        target: mortise_core::Target,
        args: Vec<String>,
    },
    /// Start a generator of the Generate tab, or ask first.
    Generate(crate::generate::view::GenerateAsk),
}

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
            Tab::Summary => crate::strings::SUMMARY,
            Tab::Boundary => crate::strings::BOUNDARY,
            Tab::Generate => crate::strings::GENERATE,
            Tab::Actions => crate::strings::ACTIONS,
            Tab::Metrics => crate::strings::METRICS,
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
    /// What the Metrics tab shows. `None` unless the Metrics tab was open when the frame began.
    pub metrics: Option<MetricsView<'a>>,
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
) -> Option<WorkAction> {
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
    let findings_title = crate::strings::findings_in(state.name);
    let mut title_x = head.left() + 16.0;
    // The new module form names the app it creates a module in (S9.4).
    let generate_title = (*tab == Tab::Generate)
        .then(|| generate_view::title(state.generate, module, state.name))
        .flatten();
    let title = if let Some(generate_title) = &generate_title {
        generate_title.as_str()
    } else if findings {
        // Where the list belongs, then its own name.
        let crumb = format!("{} /", crate::strings::ALL_MODULES);
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
        module.map_or(crate::strings::ALL_MODULES, |module| module.name.as_str())
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
    // The folder of the module after its code, drawn once the right of the header
    // is known.
    let mut folder_at = None;
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
        folder_at = Some(pos2(stamp.right() + 10.0, title_rect.bottom() - 2.0));
    }
    // The left edge of the controls at the right of the header.
    let mut controls_left = head.right() - 16.0;

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
        let label = draw::label(
            ui,
            pos2(cap.left() - 6.0, mid),
            Align2::RIGHT_CENTER,
            strings::RUN_WITH,
            ty::SECONDARY,
            pal.ink_2,
        );
        controls_left = label.left();
    } else if let Some(module) = module.filter(|_| *tab != Tab::Generate) {
        let said = strings::open_module_in_editor(&module.name);
        let open = Button {
            label: crate::strings::OPEN_IN_EDITOR,
            small: true,
            said: Some(&said),
            ..Default::default()
        };
        let tests = Button {
            label: crate::strings::action_button(Action::RunTests),
            small: true,
            icon: Some(Icon::Test),
            ..Default::default()
        };
        // "Migrate (1)" once the migration state is read (S4.3, design D7).
        let read = app
            .package_data
            .migrations_for("")
            .and_then(|migrations| migrations.get(&module.name));
        let migrate_label = match read {
            Some(state) => crate::strings::migrate_counted(state.waiting.len()),
            None => crate::strings::action_button(Action::Migrate).to_owned(),
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
        controls_left = tests_x;
        let buttons = [
            (tests, tests_x, Some(Action::RunTests)),
            (migrate, migrate_x, Some(Action::Migrate)),
            (open, open_x, None),
        ];
        for (button, x, action) in buttons {
            if button.show(ui, pal, pos2(x, y), state.keyboard).clicked() {
                header_action = Some(match action {
                    Some(action) => WorkAction::OpenAction(action),
                    None => WorkAction::OpenInEditor {
                        path: module.dir.clone(),
                        line: 1,
                    },
                });
            }
        }
    }

    if let (Some(at), Some(module)) = (folder_at, module) {
        // A long folder ends with an ellipsis short of the controls; its name stays whole.
        let folder = app.path_shown(&module.dir);
        let room = (controls_left - 16.0 - at.x).max(24.0);
        let galley = draw::galley_elided(ui, &folder, ty::PATH, pal.ink_2, room);
        let rect = draw::paint_galley(ui, at, Align2::LEFT_BOTTOM, galley);
        draw::announce(ui, rect, Role::Label, &folder);
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
    let in_body = match *tab {
        Tab::Metrics => match &state.metrics {
            Some(view) => metrics_view::metrics(ui, pal, body, app, view, state.keyboard),
            // The tab was chosen in this frame: it draws in the next one.
            None => {
                ui.ctx().request_repaint();
                None
            }
        },
        Tab::Generate => {
            let WorkState {
                generate,
                generate_inputs,
                ..
            } = state;
            generate_view::generate(ui, pal, body, app, module, generate, &generate_inputs)
        }
        Tab::Actions => {
            let WorkState {
                actions,
                actions_inputs,
                ..
            } = state;
            actions_view::actions(ui, pal, body, app, module, actions, &actions_inputs)
        }
        Tab::Boundary => {
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
        Tab::Summary => summary::summary(ui, pal, body, app, state.name, module, state.summary),
    };
    in_body.or(header_action)
}

/// The id of a tab, to give it focus.
pub fn tab_id(ui: &Ui, tab: Tab) -> eframe::egui::Id {
    ui.scope_id().with(("tab", tab.digit()))
}
