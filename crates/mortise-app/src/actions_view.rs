//! The Actions tab (S10.1, S10.2): the actions of a module, or of "All modules", on
//! the left, and the selected one on the right with its options, the exact line its
//! run button starts, and its buttons.
//!
//! Nothing here starts a process. The tab hands the app an [`Act`] with the words
//! on screen; the app asks first where the rules say so, then starts them.

use eframe::egui::{
    Align2, Key, KeyboardShortcut, Modifiers, Pos2, Rect, Role, Ui, UiBuilder, Vec2, WidgetInfo,
    pos2, vec2,
};
use mortise_core::{
    Action, AppSnapshot, Ask, Context, Module, ModuleState, OptionSet, Options, Plan, RunnerKind,
    Severity, Target, plan,
};

use crate::boundary::BoundaryAction;
use crate::draw::{self, Button, ButtonKind, Icon, ListNav};
use crate::runner_view::line_now;
use crate::strings;
use crate::summary::alert_box;
use crate::theme::{Palette, Type, radius, size, ty};

/// The list of actions is 250 px wide (S10.1).
const LIST_W: f32 = 250.0;
/// A row of the list: 30 px, and a hairline under it.
const ROW_H: f32 = 30.0;
/// The largest width of a single text field.
const FIELD_MAX_W: f32 = 260.0;

/// What the Actions tab keeps for an open app, in memory: the selected action and
/// the options, and the module they belong to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionsState {
    /// The module or "All modules" the selection and the options belong to. `None`
    /// until the tab is first drawn.
    target: Option<Target>,
    action: Action,
    pub options: Options,
    /// The words the panel showed in the last frame it was drawn. Ctrl Enter runs
    /// only these, so a key never runs a line that was not on screen.
    last_shown: Option<Vec<String>>,
}

impl Default for ActionsState {
    fn default() -> Self {
        Self {
            target: None,
            action: Action::Migrate,
            options: Options::default(),
            last_shown: None,
        }
    }
}

impl ActionsState {
    /// The tab of `target` with `action` selected and no option set.
    pub fn on(target: Target, action: Action) -> Self {
        Self {
            target: Some(target),
            action,
            options: Options::default(),
            last_shown: None,
        }
    }

    /// Called every frame for the app on screen: another module, or "All modules",
    /// starts the options again on Migrate (design D4). `shown` is false while the
    /// Actions tab is not on screen, and then no line counts as shown.
    pub fn follow(&mut self, target: Target, shown: bool) {
        if self.target.as_ref() != Some(&target) {
            *self = Self::on(target, Action::Migrate);
        }
        if !shown {
            self.last_shown = None;
        }
    }
}

/// An action to start, with the words its line on screen shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    pub action: Action,
    pub target: Target,
    /// The artisan words, exactly as the line on screen shows them.
    pub args: Vec<String>,
    /// The question Mortise asks before it starts them, if any.
    pub ask: Option<Ask>,
}

/// What the tab needs besides the app and its own state.
pub struct ActionsInputs<'a> {
    /// The display name of the app.
    pub app: &'a str,
    /// How PHP runs for the app. Without it nothing can run.
    pub runner: Option<&'a RunnerKind>,
    /// The label of the command that runs in the app, if one does.
    pub busy: Option<&'a str>,
    /// A dialog or the palette holds the keys.
    pub held: bool,
    pub keyboard: bool,
    pub mac: bool,
}

/// The modules that are on and use `name`, by name.
pub fn users_on(app: &AppSnapshot, name: &str) -> Vec<String> {
    let mut users: Vec<String> = app
        .used_by(name)
        .filter(|link| {
            app.module(&link.from)
                .is_some_and(|module| module.state == ModuleState::On)
        })
        .map(|link| link.from.clone())
        .collect();
    users.dedup();
    users
}

/// True when the app reports the environment production, exactly (design D2).
pub fn production(app: &AppSnapshot) -> bool {
    app.app_env.as_deref() == Some("production")
}

/// The keys that run the selected action.
pub fn run_keys(mac: bool) -> String {
    format!("{} Enter", strings::command_key(mac))
}

/// Paints the Actions tab into `body`. Returns what was asked for: an action to
/// start, or a line to copy.
pub fn actions(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    app: &AppSnapshot,
    module: Option<&Module>,
    state: &mut ActionsState,
    inputs: &ActionsInputs<'_>,
) -> Option<BoundaryAction> {
    let target = module.map_or(Target::All, |module| Target::Module(module.name.clone()));
    // Another module, or another app: the options start again on Migrate (D4).
    state.follow(target.clone(), true);
    let list: Vec<Action> = match module {
        Some(module) => Action::for_module(module.state != ModuleState::Off).to_vec(),
        None => Action::for_all().to_vec(),
    };
    // A module turned on or off while its tab is open keeps the row of that action.
    if !list.contains(&state.action) {
        state.action = match state.action {
            Action::TurnOff => Action::TurnOn,
            Action::TurnOn => Action::TurnOff,
            _ => Action::Migrate,
        };
    }

    let run_key = !inputs.held
        && ui.input_mut(|input| {
            input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Enter))
        });

    let mut asked = None;
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        let left = body.left() + 16.0;
        let top = body.top() + 14.0;
        action_list(ui, pal, pos2(left, top), app, module, &list, state, inputs);
        let panel_x = left + LIST_W + 14.0;
        let panel_w = (body.right() - 16.0 - panel_x).max(200.0);
        asked = panel(
            ui,
            pal,
            pos2(panel_x, top),
            panel_w,
            app,
            module,
            &target,
            state,
            inputs,
            run_key,
        );
    });
    asked
}

/// The detail on the right of a row of the list (design D4, without the migration state).
enum Detail {
    None,
    On,
    Off,
    Text(String),
}

fn detail(action: Action, app: &AppSnapshot, module: Option<&Module>) -> Detail {
    // A count over the module, or over the modules that are on.
    let total = |count: &dyn Fn(&Module) -> u32| -> u32 {
        match module {
            Some(module) => count(module),
            None => app
                .modules
                .iter()
                .filter(|module| module.state == ModuleState::On)
                .map(count)
                .sum(),
        }
    };
    let kind = |name: &'static str| {
        move |module: &Module| {
            module
                .kinds
                .iter()
                .find(|kind| kind.kind == name)
                .map_or(0, |kind| kind.files)
        }
    };
    match (action, module.map(|module| module.state)) {
        (Action::TurnOff, Some(ModuleState::On)) => Detail::On,
        (Action::TurnOn, Some(ModuleState::Off)) => Detail::Off,
        (Action::Migrate, _) => Detail::Text(strings::files(u64::from(total(&kind("Migrations"))))),
        (Action::Seed, _) => Detail::Text(strings::seeders(total(&kind("Seeders")))),
        (Action::RunTests, _) => Detail::Text(strings::tests(total(&|module| module.tests))),
        _ => Detail::None,
    }
}

/// The list of actions: one stop in the Tab order, J and K or the arrows move it.
#[expect(clippy::too_many_arguments)]
fn action_list(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    app: &AppSnapshot,
    module: Option<&Module>,
    list: &[Action],
    state: &mut ActionsState,
    inputs: &ActionsInputs<'_>,
) {
    let all = module.is_none();
    let mut selected = list
        .iter()
        .position(|action| *action == state.action)
        .unwrap_or(0);
    let nav = ListNav::over(ui, "actions", list.len(), &mut selected);
    state.action = list[selected];

    let height = list.len() as f32 * (ROW_H + 1.0) + 1.0;
    let outline = Rect::from_min_size(min, vec2(LIST_W, height));
    draw::outlined(ui, outline, radius::CONTROL, pal.surface, pal.line_2);
    for (row, action) in list.iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(min.x + 1.0, min.y + 1.0 + row as f32 * (ROW_H + 1.0)),
            vec2(LIST_W - 2.0, ROW_H),
        );
        let on = row == selected;
        let label = strings::action_row(*action, all);
        let response = ui.interact(rect, nav.id(row), nav.sense(row));
        response.widget_info(|| WidgetInfo::selected(Role::ListBoxOption, true, on, label));
        if response.clicked() {
            state.action = *action;
        }
        nav.after(ui, row, &response);

        if on {
            ui.painter().rect_filled(rect, 0, pal.well);
            draw::hline(ui, rect.left()..rect.right(), rect.top() - 1.0, pal.line_2);
            draw::hline(ui, rect.left()..rect.right(), rect.bottom(), pal.line_2);
            draw::peg(ui, pal, rect);
        } else {
            if response.hovered() {
                ui.painter().rect_filled(rect, 0, pal.well);
            }
            if row + 1 < list.len() {
                draw::hline(ui, rect.left()..rect.right(), rect.bottom(), pal.line);
            }
        }
        let mid = rect.center().y;
        draw::paint_text(
            ui,
            pos2(rect.left() + 13.0, mid),
            Align2::LEFT_CENTER,
            label,
            ty::BODY,
            pal.ink,
        );
        let right = rect.right() - 10.0;
        match detail(*action, app, module) {
            Detail::None => {}
            Detail::On => {
                let text = draw::paint_text(
                    ui,
                    pos2(right, mid),
                    Align2::RIGHT_CENTER,
                    strings::ON,
                    ty::LABEL,
                    pal.good,
                );
                let mark = mark_rect(text.left() - 5.0, mid);
                draw::mark_good(ui, mark, pal.good);
            }
            Detail::Off => {
                let text = draw::paint_text(
                    ui,
                    pos2(right, mid),
                    Align2::RIGHT_CENTER,
                    strings::OFF,
                    ty::LABEL,
                    pal.ink_2,
                );
                let mark = mark_rect(text.left() - 5.0, mid);
                draw::mark_off(ui, mark, pal.off);
            }
            Detail::Text(text) => {
                draw::paint_text(
                    ui,
                    pos2(right, mid),
                    Align2::RIGHT_CENTER,
                    &text,
                    ty::BODY,
                    pal.ink_2,
                );
            }
        }
        if inputs.keyboard && response.has_focus() {
            draw::focus_ring(ui, pal, rect.shrink(1.0), radius::STAMP);
        }
    }
}

/// A state mark of 12 px whose right edge is at `right`.
fn mark_rect(right: f32, mid: f32) -> Rect {
    Rect::from_min_size(
        pos2(
            right - size::STATE_MARK,
            (mid - size::STATE_MARK / 2.0).round(),
        ),
        Vec2::splat(size::STATE_MARK),
    )
}

/// The panel of the selected action. Returns what its buttons asked for.
#[expect(clippy::too_many_arguments)]
fn panel(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    app: &AppSnapshot,
    module: Option<&Module>,
    target: &Target,
    state: &mut ActionsState,
    inputs: &ActionsInputs<'_>,
    run_key: bool,
) -> Option<BoundaryAction> {
    let action = state.action;
    let name = module.map(|module| module.name.as_str());
    let users = name.map(|name| users_on(app, name)).unwrap_or_default();
    let context = Context {
        production: production(app),
        users_on: &users,
    };

    // The title and the sentence.
    let title = strings::action_title(action, name);
    let title_rect = draw::paint_text(
        ui,
        min,
        Align2::LEFT_TOP,
        &title,
        ty::OVERLAY_TITLE,
        pal.ink,
    );
    draw::announce(ui, title_rect, Role::Heading, &title);
    let text_at = pos2(min.x, title_rect.top() + ty::OVERLAY_TITLE.line + 2.0);
    let mut y = if action == Action::RunTests {
        tests_text(ui, pal, text_at, width, app, module)
    } else {
        let text = strings::action_text(action, name);
        draw::paragraph(ui, text_at, &text, ty::BODY, pal.ink_2, width).bottom()
    };

    // What the action asks of the user before it runs, and its options.
    if action == Action::TurnOff
        && let Some(name) = name
        && !users.is_empty()
    {
        let names: Vec<&str> = users.iter().map(String::as_str).collect();
        let text = strings::turn_off_alert(&names, name);
        let at = pos2(min.x, y + 12.0);
        y = alert_box(ui, pal, at, width, Severity::Warning, "", &text, None, true).0;
    }
    y = options(
        ui,
        pal,
        pos2(min.x, y),
        width,
        action,
        action.options(target),
        name,
        &mut state.options,
        inputs.keyboard,
    );
    if action == Action::RollBack {
        let at = pos2(min.x, y + 4.0);
        let hint = Type {
            line: 16.0,
            ..ty::SECONDARY
        };
        y = draw::paragraph(ui, at, strings::ROLL_BACK_HINT, hint, pal.ink_2, width).bottom();
    }

    // The line the run button starts, after the options have changed in this frame.
    let planned = plan(action, target, &state.options, &context);
    let shown = match &planned {
        Plan::Run { args } | Plan::Ask { args, .. } | Plan::Refused { args } => Some(args),
        Plan::NoCommand => None,
    };
    let shown_args = shown.cloned();
    // Ctrl Enter runs only the line that was on screen in the last frame.
    let run_key = run_key && state.last_shown == shown_args;
    let line = shown.map(|args| {
        line_now(inputs.runner, args).unwrap_or_else(|| format!("php artisan {}", args.join(" ")))
    });
    let label_at = pos2(min.x, y + 12.0);
    let label = draw::paint_text(
        ui,
        label_at,
        Align2::LEFT_TOP,
        strings::COMMAND,
        ty::LABEL,
        pal.ink,
    );
    draw::announce(ui, label, Role::Label, strings::COMMAND);
    let code_at = pos2(min.x, label.top() + ty::LABEL.line + 4.0);
    let code = match &line {
        Some(line) => draw::code_block(ui, pal, code_at, width, line),
        None => tests_line(ui, pal, code_at, width, name),
    };

    // The buttons: run, a dry run for Roll back, and copy.
    let off = if line.is_none() {
        Some(strings::NO_TESTS_COMMAND.to_owned())
    } else if let Some(label) = inputs.busy {
        Some(strings::is_running(label))
    } else if line_now(inputs.runner, &[]).is_none() {
        Some(strings::choose_php(inputs.app))
    } else {
        None
    };
    let keys = run_keys(inputs.mac);
    let run = Button {
        label: strings::action_button(action),
        kind: if action == Action::RollBack {
            ButtonKind::Danger
        } else {
            ButtonKind::Primary
        },
        keys: Some(&keys),
        icon: match action {
            Action::Migrate => Some(Icon::Migrate),
            Action::RunTests => Some(Icon::Test),
            _ => None,
        },
        disabled: off.is_some(),
        ..Default::default()
    };
    let mut x = min.x;
    let y = code.bottom() + 12.0;
    let response = run.show(ui, pal, pos2(x, y), inputs.keyboard);
    x = response.rect.right() + 8.0;
    let mut asked = None;
    let pressed = response.clicked() || run_key;
    if let Some(tip) = &off {
        draw::tooltip(response, pal, tip);
    }
    if pressed && off.is_none() {
        asked = match planned {
            Plan::Run { args } => Some((args, None)),
            Plan::Ask { ask, args } => Some((args, Some(ask))),
            Plan::Refused { .. } | Plan::NoCommand => None,
        }
        .map(|(args, ask)| {
            BoundaryAction::Act(Act {
                action,
                target: target.clone(),
                args,
                ask,
            })
        });
    }
    state.last_shown = shown_args;
    // Dry run checks the option and runs nothing (deviation S38).
    if action == Action::RollBack {
        let dry = Button {
            label: strings::DRY_RUN,
            ..Default::default()
        };
        let response = dry.show(ui, pal, pos2(x, y), inputs.keyboard);
        x = response.rect.right() + 8.0;
        if response.clicked() {
            state.options.pretend = true;
        }
    }
    let copy = Button {
        label: strings::COPY_COMMAND,
        disabled: line.is_none(),
        ..Default::default()
    };
    let response = copy.show(ui, pal, pos2(x, y), inputs.keyboard);
    match line {
        Some(line) if response.clicked() => {
            asked = Some(BoundaryAction::Copy {
                text: line,
                said: strings::COPIED_COMMAND,
            });
        }
        Some(_) => {}
        None => {
            draw::tooltip(response, pal, strings::NO_TESTS_COMMAND);
        }
    }
    asked
}

/// The sentence of Run tests: the count, the folder in mono, then the rest (S10.2 e).
fn tests_text(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    app: &AppSnapshot,
    module: Option<&Module>,
) -> f32 {
    let (count, folder) = match module {
        Some(module) => (
            strings::tests(module.tests),
            Some(app.path_shown(&module.dir.join("tests"))),
        ),
        None => {
            let on = app
                .modules
                .iter()
                .filter(|module| module.state == ModuleState::On);
            (strings::tests(on.map(|module| module.tests).sum()), None)
        }
    };
    let mono = Type {
        size: 11.5,
        ..ty::CODE
    };
    let mut parts = vec![(count.as_str(), ty::BODY, pal.ink_2)];
    let after = match &folder {
        Some(folder) => {
            parts.push((" in ", ty::BODY, pal.ink_2));
            parts.push((folder.as_str(), mono, pal.ink_2));
            format!(". {}", strings::TESTS_TEXT)
        }
        None => format!(" {} {}", strings::TESTS_IN_MODULES_ON, strings::TESTS_TEXT),
    };
    parts.push((after.as_str(), ty::BODY, pal.ink_2));
    let galley = draw::rich_galley(ui, &parts, width);
    let said = galley.text().to_owned();
    let rect = draw::paint_galley(ui, min, Align2::LEFT_TOP, galley);
    draw::announce(ui, rect, Role::Label, &said);
    rect.bottom()
}

/// The line of Run tests with its placeholder in dim (S10.2 e, deviation S39).
fn tests_line(ui: &Ui, pal: &Palette, min: Pos2, width: f32, module: Option<&str>) -> Rect {
    let object = module.unwrap_or("--all");
    let after = format!(" {object}");
    let parts = [
        ("php artisan ", ty::CODE, pal.code_fg),
        (strings::TESTS_PLACEHOLDER, ty::CODE, pal.code_dim),
        (after.as_str(), ty::CODE, pal.code_fg),
    ];
    let galley = draw::rich_galley(ui, &parts, width - 24.0);
    let rect = Rect::from_min_size(min, vec2(width, galley.size().y + 20.0));
    ui.painter().rect_filled(rect, radius::CONTROL, pal.code_bg);
    let said = galley.text().to_owned();
    let text = draw::paint_galley(ui, rect.min + vec2(12.0, 10.0), Align2::LEFT_TOP, galley);
    draw::announce(ui, text, Role::Label, &said);
    rect
}

/// The options an action takes: the checkboxes, then the text fields. Returns the
/// bottom of the last one, or `top` when the action takes none.
#[expect(clippy::too_many_arguments)]
fn options(
    ui: &mut Ui,
    pal: &Palette,
    top: Pos2,
    width: f32,
    action: Action,
    set: OptionSet,
    module: Option<&str>,
    options: &mut Options,
    keyboard: bool,
) -> f32 {
    let mut y = top.y;
    let checks = [
        (set.pretend, strings::DRY_RUN_OPTION, "--pretend"),
        (set.seed, strings::SEED_AFTERWARDS, "--seed"),
    ];
    if set.pretend || set.seed {
        y += 12.0;
        for (taken, label, flag) in checks {
            if !taken {
                continue;
            }
            // The package seeds after a pretend migrate: a dry run cannot seed.
            let off = flag == "--seed" && options.pretend;
            let value = match flag {
                "--pretend" => &mut options.pretend,
                _ => &mut options.seed,
            };
            let at = pos2(top.x, y);
            let response = draw::checkbox(ui, pal, at, label, *value && !off, off, keyboard);
            if response.clicked() {
                *value = !*value;
            }
            draw::label(
                ui,
                pos2(response.rect.right() + 5.0, response.rect.center().y),
                Align2::LEFT_CENTER,
                flag,
                ty::PATH,
                pal.ink_2,
            );
            y = response.rect.bottom();
        }
    }
    if set.class || set.database {
        y += 12.0;
        let database = Field {
            label: strings::DATABASE_CONNECTION,
            hint: strings::database_hint(action),
            placeholder: strings::DEFAULT_CONNECTION.to_owned(),
            mono: false,
        };
        let bottom = if set.class {
            let class = Field {
                label: strings::SEEDER_CLASS,
                hint: strings::class_hint(module.is_none()),
                placeholder: module.map(strings::module_seeder).unwrap_or_default(),
                mono: true,
            };
            let half = ((width - 10.0) / 2.0).floor();
            let left = class.show(ui, pal, pos2(top.x, y), half, &mut options.class, keyboard);
            let at = pos2(top.x + half + 10.0, y);
            let right = if set.database {
                database.show(ui, pal, at, half, &mut options.database, keyboard)
            } else {
                left
            };
            left.max(right)
        } else {
            let field_w = width.min(FIELD_MAX_W);
            database.show(
                ui,
                pal,
                pos2(top.x, y),
                field_w,
                &mut options.database,
                keyboard,
            )
        };
        y = bottom;
    }
    y
}

/// A text field with its label and hint above it.
struct Field {
    label: &'static str,
    hint: &'static str,
    placeholder: String,
    mono: bool,
}

impl Field {
    /// Paints the field with its top left at `min`. Returns its bottom.
    fn show(
        &self,
        ui: &mut Ui,
        pal: &Palette,
        min: Pos2,
        width: f32,
        text: &mut String,
        keyboard: bool,
    ) -> f32 {
        let label = draw::paint_text(ui, min, Align2::LEFT_TOP, self.label, ty::LABEL, pal.ink);
        draw::label(
            ui,
            pos2(label.right() + 4.0, label.top()),
            Align2::LEFT_TOP,
            self.hint,
            ty::SECONDARY,
            pal.ink_2,
        );
        let rect = Rect::from_min_size(
            pos2(min.x, label.top() + ty::LABEL.line + 4.0),
            vec2(width, size::FIELD_H),
        );
        draw::text_field_with_hint(
            ui,
            pal,
            rect,
            self.label,
            text,
            &self.placeholder,
            self.mono,
            keyboard,
        );
        rect.bottom()
    }
}
