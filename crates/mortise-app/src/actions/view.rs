//! The Actions tab (S10.1, S10.2): the actions of a module, or of "All modules", on
//! the left, and the selected one on the right with its options, the exact line its
//! run button starts, and its buttons.
//!
//! Nothing here starts a process. The tab hands the app an [`Act`] with the words
//! on screen; the app asks first where the rules say so, then starts them.

use eframe::egui::{
    Align2, Color32, Key, KeyboardShortcut, Modifiers, Pos2, Rect, Role, ScrollArea, Sense, Ui,
    UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{
    Action, AppSnapshot, Ask, Context, LayoutKey, Module, ModuleMigrations, ModuleState, OptionSet,
    Options, Plan, RunnerKind, Severity, Target, gives_data, migrate_status_args, plan,
};

use crate::actions::strings;
use crate::app::workspace::WorkAction;
use crate::runner::view::line_now;
use crate::ui::draw::alert_box;
use crate::ui::draw::{self, Button, ButtonKind, Icon, ListNav, StampState};
use crate::ui::theme::{Face, Palette, Type, radius, size, ty};

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
    /// The words of the read of the migration state whose line was whole on screen
    /// in the last frame. "Read migration state" runs only these (design D7).
    read_shown: Option<Vec<String>>,
    /// The group "Destructive" is open (S10.4).
    destructive_open: bool,
    /// The row of the group is the current row of the list. The panel keeps the
    /// action that was selected before.
    on_group: bool,
}

impl Default for ActionsState {
    fn default() -> Self {
        Self {
            target: None,
            action: Action::Migrate,
            options: Options::default(),
            last_shown: None,
            read_shown: None,
            destructive_open: false,
            on_group: false,
        }
    }
}

impl ActionsState {
    /// The tab of `target` with `action` selected and no option set. The group
    /// "Destructive" is open when `action` is in it (design D5).
    pub fn on(target: Target, action: Action) -> Self {
        Self {
            target: Some(target),
            action,
            options: Options::default(),
            last_shown: None,
            read_shown: None,
            destructive_open: action.is_destructive(),
            on_group: false,
        }
    }

    /// True when the tab is on `action` of `target`.
    pub fn shows(&self, target: &Target, action: Action) -> bool {
        self.target.as_ref() == Some(target) && self.action == action
    }

    /// Called every frame for the app on screen: another module, or "All modules",
    /// starts the options again on Migrate (design D4). `shown` is false while the
    /// Actions tab is not on screen, and then no line counts as shown and the group
    /// "Destructive" closes (design D5).
    pub fn follow(&mut self, target: Target, shown: bool) {
        if self.target.as_ref() != Some(&target) {
            *self = Self::on(target, Action::Migrate);
        }
        if !shown {
            self.last_shown = None;
            self.read_shown = None;
            self.on_group = false;
            self.close_group();
        }
    }

    /// Closes the group "Destructive". An action of it that was selected goes back to Migrate.
    fn close_group(&mut self) {
        self.destructive_open = false;
        if self.action.is_destructive() {
            self.action = Action::Migrate;
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
    /// The target of the read of the migration state that runs, if one does (design D7).
    pub reading_state: Option<&'a Target>,
    /// Why the last read of the migration state gave nothing.
    pub state_error: Option<&'a StateError>,
}

/// Why a read of the migration state gave nothing (design D2, D7). The state box of
/// the same target and connection says so.
pub struct StateError {
    pub target: Target,
    /// The connection of the read: its `--database`, empty for the default.
    pub connection: String,
    pub reason: String,
}

/// The connection `action` runs on with `options`: its `--database`, or empty for
/// the default connection when it takes none or none is set.
pub fn connection<'a>(action: Action, target: &Target, options: &'a Options) -> &'a str {
    if action.options(target, true).database {
        options.database.trim()
    } else {
        ""
    }
}

/// The migration state of the module, or of each module that is on for "All
/// modules", on `connection`, from the reads of this session. `None` when it was
/// not read for that connection, and for "All modules" until the state covers every
/// module that is on: a count of some would leave out what `--all` runs in the others.
fn migration_state<'a>(
    app: &'a AppSnapshot,
    module: Option<&'a Module>,
    connection: &str,
) -> Option<Vec<(&'a Module, &'a ModuleMigrations)>> {
    let migrations = app.package_data.migrations_for(connection)?;
    if module.is_none()
        && !app
            .package_data
            .migrations_of_every(connection, &modules_on(app))
    {
        return None;
    }
    let read: Vec<(&Module, &ModuleMigrations)> = match module {
        Some(module) => migrations
            .get(&module.name)
            .map(|state| vec![(module, state)])
            .unwrap_or_default(),
        None => app
            .modules
            .iter()
            .filter(|module| module.state == ModuleState::On)
            .filter_map(|module| migrations.get(&module.name).map(|state| (module, state)))
            .collect(),
    };
    (!read.is_empty()).then_some(read)
}

/// The names of the modules that are on.
fn modules_on(app: &AppSnapshot) -> Vec<&str> {
    app.modules
        .iter()
        .filter(|module| module.state == ModuleState::On)
        .map(|module| module.name.as_str())
        .collect()
}

/// The migrations waiting in a state from [`migration_state`].
fn waiting_count(read: &[(&Module, &ModuleMigrations)]) -> usize {
    read.iter().map(|(_, state)| state.waiting.len()).sum()
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

/// True when Mortise could not read where the app keeps its modules, and assumes
/// the package default.
fn modules_folder_assumed(app: &AppSnapshot) -> bool {
    app.layout.assumed.contains(&LayoutKey::ModulesFolder)
}

/// Why Mortise does not run `action` in `app`, whose display name is `name`, or
/// `None` when it runs it. The line can then only be copied (design D5).
pub fn refusal(app: &AppSnapshot, name: &str, action: Action) -> Option<String> {
    let context = Context {
        production: production(app),
        users_on: &[],
        modules_folder_assumed: modules_folder_assumed(app),
        package_data: mortise_core::gives_data(&app.package),
    };
    mortise_core::refuses(action, &context).then(|| match context.production {
        true => strings::production_refused(name),
        false => strings::modules_folder_assumed(name),
    })
}

/// True when the app reports the environment production, exactly (design D2).
pub fn production(app: &AppSnapshot) -> bool {
    app.app_env.as_deref() == Some("production")
}

/// The keys that run the selected action.
pub fn run_keys(mac: bool) -> String {
    format!("{} Enter", crate::strings::command_key(mac))
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
) -> Option<WorkAction> {
    let target = module.map_or(Target::All, |module| Target::Module(module.name.clone()));
    // Another module, or another app: the options start again on Migrate (D4).
    state.follow(target.clone(), true);
    let everyday: Vec<Action> = match module {
        Some(module) => Action::for_module(module.state != ModuleState::Off).to_vec(),
        None => Action::for_all().to_vec(),
    };
    let destructive = Action::destructive(&target);
    // A module turned on or off while its tab is open keeps the row of that action.
    let listed = everyday.contains(&state.action)
        || (state.destructive_open && destructive.contains(&state.action));
    if !listed {
        state.action = match state.action {
            Action::TurnOff => Action::TurnOn,
            Action::TurnOn => Action::TurnOff,
            _ => Action::Migrate,
        };
    }
    // The everyday actions, the row of the group, then the group's actions while it is open.
    let mut rows: Vec<Row> = everyday.into_iter().map(Row::Action).collect();
    rows.push(Row::Group);
    if state.destructive_open {
        rows.extend(destructive.iter().copied().map(Row::Action));
    }

    let run_key = !inputs.held
        && ui.input_mut(|input| {
            input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Enter))
        });
    // Ctrl Shift C copies the line on screen (S7), as the Generate form does.
    let copy_key = !inputs.held
        && ui.input_mut(|input| {
            input.consume_shortcut(&KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::SHIFT,
                Key::C,
            ))
        });

    let mut asked = None;
    let salt = module.map_or("all", |module| module.name.as_str());
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        // A tall panel scrolls, for a low window or an open drawer.
        ScrollArea::vertical()
            .id_salt(("actions", salt))
            .auto_shrink(false)
            .show(ui, |ui| {
                let origin = ui.cursor().min;
                let width = ui.available_width();
                let left = origin.x + 16.0;
                let top = origin.y + 14.0;
                let list_bottom =
                    action_list(ui, pal, pos2(left, top), app, module, &rows, state, inputs);
                let panel_x = left + LIST_W + 14.0;
                let panel_w = (origin.x + width - 16.0 - panel_x).max(200.0);
                let (bottom, panel_asked) = panel(
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
                    copy_key,
                );
                asked = panel_asked;
                // Tell the scroll area how high the content is.
                ui.allocate_rect(
                    Rect::from_min_max(
                        origin,
                        pos2(origin.x + width, list_bottom.max(bottom) + 16.0),
                    ),
                    Sense::hover(),
                );
            });
    });
    asked
}

/// A row of the list of actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Action(Action),
    /// The row of the group "Destructive", which opens and closes it.
    Group,
}

/// The detail on the right of a row of the list (design D4). The migration state,
/// once read, gives Migrate, Roll back, Reset and Refresh their counts (design D7).
enum Detail {
    None,
    On,
    Off,
    Text(String),
}

fn detail(action: Action, app: &AppSnapshot, module: Option<&Module>, connection: &str) -> Detail {
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
    let read = migration_state(app, module, connection);
    let ran = || {
        read.as_ref()
            .map(|read| read.iter().map(|(_, s)| s.ran.len()).sum())
    };
    match (action, module.map(|module| module.state)) {
        (Action::TurnOff, Some(ModuleState::On)) => Detail::On,
        (Action::TurnOn, Some(ModuleState::Off)) => Detail::Off,
        (Action::Migrate, _) => Detail::Text(match &read {
            Some(read) => crate::strings::waiting(waiting_count(read)),
            None => crate::strings::files(u64::from(total(&kind("Migrations")))),
        }),
        (Action::RollBack, _) => match read.as_ref().map(|read| read[0].1.last_batch().len()) {
            Some(n) if n > 0 => Detail::Text(strings::last_batch(n)),
            _ => Detail::None,
        },
        (Action::Seed, _) => Detail::Text(strings::seeders(total(&kind("Seeders")))),
        (Action::RunTests, _) => Detail::Text(crate::strings::tests(tests_counted(
            app,
            module,
            mortise_core::gives_data(&app.package),
        ))),
        // Without the migration state reset and refresh have none (deviation S42).
        (Action::ResetMigrations | Action::RefreshMigrations, _) => {
            ran().map_or(Detail::None, |ran| Detail::Text(crate::strings::ran(ran)))
        }
        (Action::DeleteModule, _) => {
            Detail::Text(crate::strings::files(module.map_or(0, |m| m.files)))
        }
        _ => Detail::None,
    }
}

/// The list of actions: one stop in the Tab order, J and K or the arrows move it.
/// The row of the group is a stop too; Enter or a click opens or closes the group.
/// Returns the bottom of the list.
#[expect(clippy::too_many_arguments)]
fn action_list(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    app: &AppSnapshot,
    module: Option<&Module>,
    rows: &[Row],
    state: &mut ActionsState,
    inputs: &ActionsInputs<'_>,
) -> f32 {
    let all = module.is_none();
    let group = rows.iter().position(|row| *row == Row::Group).unwrap_or(0);
    let mut selected = if state.on_group {
        group
    } else {
        rows.iter()
            .position(|row| *row == Row::Action(state.action))
            .unwrap_or(0)
    };
    let nav = ListNav::over(ui, "actions", rows.len(), &mut selected);
    match rows[selected] {
        Row::Action(action) => {
            state.action = action;
            state.on_group = false;
        }
        Row::Group => state.on_group = true,
    }

    // For a module, the open group ends with the line about dropping all tables.
    let note = (!all && state.destructive_open).then(|| {
        let width = LIST_W - 2.0 - 25.0 - 10.0;
        draw::galley_wrapped(ui, strings::FRESH_IS_UNDER_ALL, NOTE, pal.ink_2, width)
    });
    let note_h = note
        .as_ref()
        .map_or(0.0, |galley| 6.0 + galley.size().y + 7.0);
    let height = rows.len() as f32 * (ROW_H + 1.0) + 1.0 + note_h;
    let outline = Rect::from_min_size(min, vec2(LIST_W, height));
    draw::outlined(ui, outline, radius::CONTROL, pal.surface, pal.line_2);
    let mut toggled = false;
    for (index, row) in rows.iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(min.x + 1.0, min.y + 1.0 + index as f32 * (ROW_H + 1.0)),
            vec2(LIST_W - 2.0, ROW_H),
        );
        let on = index == selected;
        let in_group = matches!(row, Row::Action(action) if action.is_destructive());
        let label = match row {
            Row::Action(action) => crate::strings::action_row(*action, all),
            Row::Group => strings::DESTRUCTIVE,
        };
        let response = ui.interact(rect, nav.id(index), nav.sense(index));
        response.widget_info(|| WidgetInfo::selected(Role::ListBoxOption, true, on, label));
        if *row == Row::Group {
            let open = state.destructive_open;
            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_expanded(open));
        }
        if response.clicked() {
            match row {
                Row::Action(action) => {
                    state.action = *action;
                    state.on_group = false;
                }
                Row::Group => toggled = true,
            }
        }
        nav.after(ui, index, &response);

        let last = index + 1 == rows.len() && note.is_none();
        if *row == Row::Group && !on {
            ui.painter().rect_filled(rect, 0, pal.panel);
        }
        if on {
            ui.painter().rect_filled(rect, 0, pal.well);
            draw::hline(ui, rect.left()..rect.right(), rect.top() - 1.0, pal.line_2);
            draw::hline(ui, rect.left()..rect.right(), rect.bottom(), pal.line_2);
            // The peg of a row of the group stands under its chevron (S10.4).
            let peg_row = if in_group {
                Rect::from_min_max(pos2(rect.left() + 10.0, rect.top()), rect.max)
            } else {
                rect
            };
            draw::peg(ui, pal, peg_row);
        } else {
            if response.hovered() {
                ui.painter().rect_filled(rect, 0, pal.well);
            }
            if *row == Row::Group {
                draw::hline(ui, rect.left()..rect.right(), rect.top() - 1.0, pal.line_2);
            }
            if !last {
                draw::hline(ui, rect.left()..rect.right(), rect.bottom(), pal.line);
            }
        }
        let mid = rect.center().y;
        let mut text_x = rect.left() + 13.0;
        if in_group {
            text_x = rect.left() + 25.0;
        }
        let label_type = if *row == Row::Group {
            ty::BODY.with_face(Face::Sans600)
        } else {
            ty::BODY
        };
        if *row == Row::Group {
            let chevron = if state.destructive_open {
                Icon::ChevronDown
            } else {
                Icon::ChevronRight
            };
            let side = 11.0;
            let at = Rect::from_min_size(pos2(text_x, mid - side / 2.0), Vec2::splat(side));
            draw::icon(ui, at, chevron, pal.ink);
            text_x += side + 6.0;
        }
        draw::paint_text(
            ui,
            pos2(text_x, mid),
            Align2::LEFT_CENTER,
            label,
            label_type,
            pal.ink,
        );
        let right = rect.right() - 10.0;
        let detail = match row {
            Row::Action(action) => {
                let target = module.map_or(Target::All, |m| Target::Module(m.name.clone()));
                let connection = connection(*action, &target, &state.options);
                detail(*action, app, module, connection)
            }
            Row::Group if state.destructive_open => Detail::Text(strings::GROUP_OPEN.to_owned()),
            Row::Group => Detail::Text(strings::GROUP_CLOSED.to_owned()),
        };
        match detail {
            Detail::None => {}
            Detail::On => {
                let text = draw::paint_text(
                    ui,
                    pos2(right, mid),
                    Align2::RIGHT_CENTER,
                    crate::strings::ON,
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
                    crate::strings::OFF,
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
    if let Some(galley) = note {
        let top = min.y + 1.0 + rows.len() as f32 * (ROW_H + 1.0);
        let said = galley.text().to_owned();
        let at = draw::paint_galley(
            ui,
            pos2(min.x + 1.0 + 25.0, top + 6.0),
            Align2::LEFT_TOP,
            galley,
        );
        draw::announce(ui, at, Role::Label, &said);
    }
    // Enter or a click on the row of the group opens or closes it.
    if toggled {
        state.on_group = true;
        if state.destructive_open {
            state.close_group();
        } else {
            state.destructive_open = true;
        }
        ui.ctx().request_repaint();
    }
    outline.bottom()
}

/// The dim line under the group of a module.
const NOTE: Type = Type {
    line: 16.0,
    ..ty::SECONDARY
};

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

/// The panel of the selected action. Returns its bottom and what its buttons asked for.
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
    copy_key: bool,
) -> (f32, Option<WorkAction>) {
    let action = state.action;
    let destructive = action.is_destructive();
    let name = module.map(|module| module.name.as_str());
    let users = name.map(|name| users_on(app, name)).unwrap_or_default();
    // A 13.4 app gives the data of the package: Run tests has its command (design D8).
    let package_data = mortise_core::gives_data(&app.package);
    let context = Context {
        production: production(app),
        users_on: &users,
        modules_folder_assumed: modules_folder_assumed(app),
        package_data,
    };
    // The line the run button starts, as the options stood at the end of the last frame.
    // It is planned again below, after the options have changed in this frame.
    let refused = matches!(
        plan(action, target, &state.options, &context),
        Plan::Refused { .. }
    );

    // The title and the sentence.
    let title = if destructive {
        strings::destructive_title(action, name.unwrap_or_default(), inputs.app)
    } else {
        strings::action_title(action, name)
    };
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
        tests_text(ui, pal, text_at, width, app, module, package_data)
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
    // Production: a destructive action only shows its line (S10.4 d).
    let production_text = refused.then(|| refusal(app, inputs.app, action)).flatten();
    if let Some(text) = &production_text {
        let at = pos2(min.x, y + 12.0);
        y = alert_box(ui, pal, at, width, Severity::Warning, "", text, None, true).0;
    }
    // The state of the connection the line runs on (design D7).
    let connection = connection(action, target, &state.options).to_owned();
    let read = migration_state(app, module, &connection);
    if destructive {
        let rows = lost_rows(action, app, module, read.as_deref());
        y = lost_box(ui, pal, pos2(min.x, y + 12.0), width, &rows);
    }
    // The migration state and its read, on a 13.4 app (design D7).
    let mut state_asked = None;
    let mut next_read_shown = None;
    let last_read = state.read_shown.take();
    if gives_data(&app.package) {
        let at = pos2(min.x, y + 12.0);
        let shown = StateShown {
            app,
            module,
            target,
            action,
            read: read.as_deref(),
            last_read: last_read.as_deref(),
            connection: &connection,
        };
        if let Some((bottom, box_asked, read_shown)) = state_box(ui, pal, at, width, &shown, inputs)
        {
            y = bottom;
            state_asked = box_asked;
            next_read_shown = read_shown;
        }
    }
    state.read_shown = next_read_shown;
    y = options(
        ui,
        pal,
        pos2(min.x, y),
        width,
        action,
        action.options(target, package_data),
        name,
        &mut state.options,
        inputs.keyboard,
    );
    let hint = Type {
        line: 16.0,
        ..ty::SECONDARY
    };
    if action == Action::RollBack {
        let at = pos2(min.x, y + 4.0);
        y = draw::paragraph(ui, at, strings::ROLL_BACK_HINT, hint, pal.ink_2, width).bottom();
    }
    // Delete: what stays, and the modules that will break (S10.4 b).
    if action == Action::DeleteModule
        && let Some(name) = name
    {
        let mut text = strings::stays(name);
        if !users.is_empty() {
            let names: Vec<&str> = users.iter().map(String::as_str).collect();
            text = format!("{text} {}", strings::will_break(&names, name));
        }
        let at = pos2(min.x, y + 6.0);
        y = draw::paragraph(ui, at, &text, hint, pal.ink_2, width).bottom();
    }

    // The line the run button starts, after the options have changed in this frame.
    let planned = plan(action, target, &state.options, &context);
    let shown = match &planned {
        Plan::Run { args } | Plan::Ask { args, .. } | Plan::Refused { args } => Some(args),
        Plan::NoCommand => None,
    };
    let shown_args = shown.cloned();
    // The button and Ctrl Enter run only the line that was whole on screen in the
    // last frame.
    let whole = state.last_shown == shown_args;
    let line = shown.map(|args| {
        line_now(inputs.runner, args).unwrap_or_else(|| format!("php artisan {}", args.join(" ")))
    });
    let label_at = pos2(min.x, y + 12.0);
    let label = draw::paint_text(
        ui,
        label_at,
        Align2::LEFT_TOP,
        crate::strings::COMMAND,
        ty::LABEL,
        pal.ink,
    );
    draw::announce(ui, label, Role::Label, crate::strings::COMMAND);
    let code_at = pos2(min.x, label.top() + ty::LABEL.line + 4.0);
    let code = match &line {
        Some(line) => draw::code_block(ui, pal, code_at, width, line),
        None => tests_line(ui, pal, code_at, width, name),
    };

    // The buttons: run, a dry run for Roll back and Reset, and copy.
    let refused = matches!(planned, Plan::Refused { .. });
    let off = if line.is_none() {
        Some(strings::NO_TESTS_COMMAND.to_owned())
    } else if refused {
        refusal(app, inputs.app, action)
    } else if let Some(label) = inputs.busy {
        Some(crate::strings::is_running(label))
    } else if line_now(inputs.runner, &[]).is_none() {
        Some(crate::strings::choose_php(inputs.app))
    } else if !whole {
        Some(crate::strings::SCROLL_TO_COMMAND.to_owned())
    } else {
        None
    };
    let keys = run_keys(inputs.mac);
    // Migrate counts the waiting migrations once the state is read (S10.1).
    let run_label = match (action, &read) {
        (Action::Migrate, Some(read)) => crate::strings::migrate_counted(waiting_count(read)),
        _ => crate::strings::action_button(action).to_owned(),
    };
    let run = Button {
        label: &run_label,
        // Roll back and the four of the group lose data: the red outline (S10.4).
        kind: if action == Action::RollBack || destructive {
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
    let mut asked = state_asked;
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
            WorkAction::Act(Act {
                action,
                target: target.clone(),
                args,
                ask,
            })
        });
    }
    // A line cut off by the scroll area is not on screen, so Ctrl Enter does not take
    // it. Its text sits 12 px and 10 px inside the code block.
    let text = code.shrink2(vec2(12.0, 10.0));
    state.last_shown = shown_args.filter(|_| ui.clip_rect().contains_rect(text));
    // Dry run checks the option and runs nothing (deviation S38). In production
    // reset runs nothing at all (S10.4 d).
    if matches!(action, Action::RollBack | Action::ResetMigrations) {
        let dry = Button {
            label: crate::strings::DRY_RUN,
            disabled: refused,
            ..Default::default()
        };
        let response = dry.show(ui, pal, pos2(x, y), inputs.keyboard);
        x = response.rect.right() + 8.0;
        if response.clicked() {
            state.options.pretend = true;
        }
    }
    let copy = Button {
        label: crate::strings::COPY_COMMAND,
        disabled: line.is_none(),
        ..Default::default()
    };
    let response = copy.show(ui, pal, pos2(x, y), inputs.keyboard);
    let bottom = response.rect.bottom();
    match line {
        Some(line) if response.clicked() || copy_key => {
            asked = Some(WorkAction::Copy {
                text: line,
                said: crate::strings::COPIED_COMMAND,
            });
        }
        Some(_) => {}
        None => {
            draw::tooltip(response, pal, strings::NO_TESTS_COMMAND);
        }
    }
    (bottom, asked)
}

/// What the state box of a panel shows (design D7).
#[derive(Clone, Copy)]
struct StateShown<'a> {
    app: &'a AppSnapshot,
    module: Option<&'a Module>,
    target: &'a Target,
    action: Action,
    /// The state of the target, from [`migration_state`].
    read: Option<&'a [(&'a Module, &'a ModuleMigrations)]>,
    /// The words of the read whose line was whole on screen in the last frame.
    last_read: Option<&'a [String]>,
    /// The connection of the panel's line: the read takes its `--database`.
    connection: &'a str,
}

/// A row of the state box.
enum StateRow<'a> {
    /// A migration, with its module under "All modules".
    Migration {
        module: Option<&'a Module>,
        name: &'a str,
    },
    /// A sentence in the secondary colour.
    Text(String),
}

/// The type of the names of migrations: mono, on the line of the body.
const MIGRATION_TYPE: Type = Type {
    size: 11.5,
    line: ty::BODY.line,
    ..ty::CODE
};

/// The local time of a read, `14:05`, as the drawer shows the time of a run.
fn clock(ms: i64) -> String {
    jiff::Timestamp::from_millisecond(ms)
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH)
        .to_zoned(jiff::tz::TimeZone::system())
        .strftime("%H:%M")
        .to_string()
}

/// The state box of Migrate, Roll back, Reset and Refresh (S10.1 to S10.4), with
/// the button that reads the state and the line it runs (design D7). The button
/// runs only a line that was whole on screen in the last frame. Returns the bottom,
/// what was asked for and the words of the line when it is whole on screen in this
/// frame, or `None` when the panel has no box.
fn state_box(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    shown: &StateShown<'_>,
    inputs: &ActionsInputs<'_>,
) -> Option<(f32, Option<WorkAction>, Option<Vec<String>>)> {
    let StateShown {
        app,
        module,
        target,
        action,
        read,
        last_read,
        connection,
    } = *shown;
    let title = match action {
        Action::Migrate => strings::WAITING,
        Action::RollBack => strings::LAST_BATCH,
        Action::ResetMigrations | Action::RefreshMigrations => strings::RAN,
        _ => return None,
    };
    let args = migrate_status_args(target, connection);
    let line =
        line_now(inputs.runner, &args).unwrap_or_else(|| format!("php artisan {}", args.join(" ")));

    let all = module.is_none();
    let reading = inputs
        .reading_state
        .is_some_and(|reading| reading == target || *reading == Target::All);
    let mut note = String::new();
    let mut rows: Vec<StateRow<'_>> = Vec::new();
    if reading {
        rows.push(StateRow::Text(strings::STATE_READING.to_owned()));
    } else if let Some(read) = read {
        let mut names: Vec<StateRow<'_>> = Vec::new();
        let mut modules = 0;
        for (module, state) in read {
            let module = all.then_some(*module);
            let before = names.len();
            match action {
                Action::Migrate => names.extend(
                    state
                        .waiting
                        .iter()
                        .map(|name| StateRow::Migration { module, name }),
                ),
                Action::RollBack => {
                    names.extend(
                        state
                            .last_batch()
                            .into_iter()
                            .map(|ran| StateRow::Migration {
                                module,
                                name: &ran.name,
                            }),
                    );
                }
                _ => names.extend(state.ran.iter().map(|ran| StateRow::Migration {
                    module,
                    name: &ran.name,
                })),
            }
            if names.len() > before {
                modules += 1;
            }
        }
        if names.is_empty() {
            let none = match action {
                Action::Migrate => crate::strings::NOTHING_WAITING,
                _ => strings::NOTHING_RAN_YET,
            };
            rows.push(StateRow::Text(none.to_owned()));
        } else {
            note = if all {
                strings::migrations_in_modules(names.len(), modules)
            } else {
                strings::migrations(names.len())
            };
            rows.extend(names);
        }
        if app.package_data.migration_table == Some(false) {
            rows.push(StateRow::Text(strings::NO_MIGRATIONS_TABLE.to_owned()));
        }
    } else if app.package_data.migration_table.is_some()
        && app.package_data.migrations_for(connection).is_none()
    {
        // A state of another connection says nothing about this one.
        rows.push(StateRow::Text(strings::NOT_READ_FOR_CONNECTION.to_owned()));
    } else if module.is_none()
        && app
            .package_data
            .migrations_for(connection)
            .is_some_and(|migrations| {
                modules_on(app)
                    .iter()
                    .any(|name| migrations.contains_key(*name))
            })
    {
        // Some modules were read, not all: "All modules" offers the read of them all.
        rows.push(StateRow::Text(
            strings::NOT_READ_FOR_EVERY_MODULE.to_owned(),
        ));
    } else {
        rows.push(StateRow::Text(strings::NOT_READ_YET.to_owned()));
    }
    // A read that gave nothing says why and keeps the state it had (design D2).
    if let Some(failed) = inputs
        .state_error
        .filter(|failed| failed.target == *target && failed.connection == connection)
    {
        rows.push(StateRow::Text(strings::migration_state_failed(
            &failed.reason,
        )));
    }

    // The sentences wrap; a migration is one line.
    let text_w = width - 26.0;
    let galleys: Vec<Option<std::sync::Arc<eframe::egui::Galley>>> = rows
        .iter()
        .map(|row| match row {
            StateRow::Text(text) => {
                Some(draw::galley_wrapped(ui, text, ty::BODY, pal.ink_2, text_w))
            }
            StateRow::Migration { .. } => None,
        })
        .collect();
    let heights: Vec<f32> = galleys
        .iter()
        .map(|galley| {
            galley.as_ref().map_or(size::LIST_LINE_H, |galley| {
                (galley.size().y + 10.0).max(size::LIST_LINE_H)
            })
        })
        .collect();
    let head = 41.0 + draw::box_head_extra(ui, width, title, &note);
    let height = head + heights.iter().sum::<f32>() + 8.0;
    let rect = Rect::from_min_size(min, vec2(width, height));
    let mut y = draw::box_head(ui, pal, rect, title, &note);
    let count = rows.len();
    for (index, ((row, galley), height)) in rows.iter().zip(galleys).zip(&heights).enumerate() {
        let mid = y + height / 2.0;
        let left = rect.left() + 13.0;
        match (row, galley) {
            (StateRow::Text(text), Some(galley)) => {
                let at = draw::paint_galley(ui, pos2(left, mid), Align2::LEFT_CENTER, galley);
                draw::announce(ui, at, Role::Label, text);
            }
            (StateRow::Migration { module, name }, _) => {
                let mut x = left;
                if let Some(module) = module {
                    let stamp_at = pos2(x, (mid - size::STAMP_H / 2.0).round());
                    x = draw::stamp(ui, pal, stamp_at, &module.code, StampState::default()).right()
                        + 7.0;
                }
                let mut job = eframe::egui::text::LayoutJob::single_section(
                    (*name).to_owned(),
                    MIGRATION_TYPE.format(pal.ink),
                );
                job.wrap = eframe::egui::text::TextWrapping::truncate_at_width(
                    (rect.right() - 13.0 - x).max(40.0),
                );
                let galley = ui.painter().layout_job(job);
                let at = draw::paint_galley(ui, pos2(x, mid), Align2::LEFT_CENTER, galley);
                let said = match module {
                    Some(module) => format!("{}: {name}", module.name),
                    None => (*name).to_owned(),
                };
                draw::announce(ui, at, Role::Label, &said);
            }
            (StateRow::Text(_), None) => {}
        }
        if index + 1 < count {
            draw::hline(
                ui,
                rect.left() + 13.0..rect.right() - 13.0,
                y + height - 1.0,
                pal.line,
            );
        }
        y += height;
    }

    // The button that reads, and when the state was read. It runs only the line
    // that was whole on screen in the last frame, as the run button does (S37).
    let off = match inputs.busy {
        Some(label) => Some(crate::strings::is_running(label)),
        None if line_now(inputs.runner, &[]).is_none() => {
            Some(crate::strings::choose_php(inputs.app))
        }
        None if last_read != Some(&args[..]) => Some(crate::strings::SCROLL_TO_COMMAND.to_owned()),
        None => None,
    };
    let button = Button {
        label: if read.is_some() {
            strings::STATE_READ_AGAIN
        } else {
            strings::READ_MIGRATION_STATE
        },
        small: true,
        disabled: off.is_some(),
        ..Default::default()
    };
    let response = button.show(ui, pal, pos2(min.x, rect.bottom() + 8.0), inputs.keyboard);
    let button_rect = response.rect;
    let mut asked = None;
    if response.clicked() && off.is_none() {
        asked = Some(WorkAction::ReadMigrations {
            target: target.clone(),
            args: args.clone(),
        });
    }
    if let Some(tip) = &off {
        draw::tooltip(response, pal, tip);
    }
    if let Some(read_ms) = read.and_then(|read| read.iter().map(|(_, s)| s.read_ms).max()) {
        let text = strings::state_read_at(&clock(read_ms));
        draw::label(
            ui,
            pos2(button_rect.right() + 8.0, button_rect.center().y),
            Align2::LEFT_CENTER,
            &text,
            ty::SECONDARY,
            pal.ink_2,
        );
    }
    let code = draw::code_block(
        ui,
        pal,
        pos2(min.x, button_rect.bottom() + 8.0),
        width,
        &line,
    );
    // Its text sits 12 px and 10 px inside the code block.
    let whole = ui.clip_rect().contains_rect(code.shrink2(vec2(12.0, 10.0)));
    Some((code.bottom(), asked, whole.then_some(args)))
}

/// A row of "What is lost": its name, in parts that are set in mono or not, and its
/// value on the right.
struct Lost {
    name: Vec<(String, bool)>,
    value: String,
}

impl Lost {
    fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: vec![(name.into(), false)],
            value: value.into(),
        }
    }

    /// A name with a path in mono after its first words.
    fn with_path(words: &str, path: String, value: impl Into<String>) -> Self {
        Self {
            name: vec![(words.to_owned(), false), (path, true)],
            value: value.into(),
        }
    }
}

/// The tables a row of "What is lost" names: the first five in mono, then how many
/// more there are (S10.4).
const TABLES_NAMED: usize = 5;

/// The rows of "What is lost" of a destructive action (design D5). With the
/// migration state reset and refresh count the migrations and name the tables they
/// create (design D7); without it they do not (deviation S42).
fn lost_rows(
    action: Action,
    app: &AppSnapshot,
    module: Option<&Module>,
    read: Option<&[(&Module, &ModuleMigrations)]>,
) -> Vec<Lost> {
    let name = module.map_or("", |module| module.name.as_str());
    let state = read.and_then(|read| read.first()).map(|(_, state)| *state);
    let tables = state.map_or(&[][..], |state| state.tables.as_slice());
    match (action, module) {
        (Action::ResetMigrations, _) => {
            let first = match state {
                Some(state) => strings::reset_loses_counted(name, state.ran.len()),
                None => strings::reset_loses(name),
            };
            let mut rows = vec![Lost::new(first, strings::WITH_ALL_ROWS)];
            if !tables.is_empty() {
                rows.push(tables_row(tables));
            }
            rows
        }
        (Action::RefreshMigrations, _) => {
            let value = match tables.len() {
                0 => strings::COME_BACK_EMPTY.to_owned(),
                n => strings::tables_come_back_empty(n),
            };
            vec![Lost::new(strings::refresh_loses(name), value)]
        }
        (Action::FreshMigrate, _) => vec![
            Lost::new(strings::FRESH_LOSES_TABLES, strings::WITH_ALL_ROWS),
            Lost::new(strings::VIEWS_IF_CHOSEN, ""),
            Lost::new(strings::TYPES_IF_CHOSEN, strings::POSTGRES_ONLY),
        ],
        (Action::DeleteModule, Some(module)) => {
            let mut rows = vec![Lost::with_path(
                strings::THE_FOLDER,
                app.path_shown(&module.dir),
                crate::strings::files_and_lines(module.files, module.lines),
            )];
            // Left out when git has nothing to lose; said when git cannot tell.
            match module.uncommitted {
                Some(0) => {}
                Some(n) => rows.push(Lost::new(
                    strings::UNCOMMITTED_CHANGES,
                    crate::strings::files(u64::from(n)),
                )),
                None => rows.push(Lost::new(strings::CANNOT_TELL_GIT, "")),
            }
            if let Some(file) = statuses_file(app) {
                rows.push(Lost::with_path(strings::ITS_LINE_IN, file, ""));
            }
            rows
        }
        _ => Vec::new(),
    }
}

/// The row of the tables of reset: `checkouts, carts, cart_items, orders, order_items
/// and 2 more` with `7 tables` (S10.4).
fn tables_row(tables: &[String]) -> Lost {
    let mut name = Vec::new();
    for (index, table) in tables.iter().take(TABLES_NAMED).enumerate() {
        if index > 0 {
            name.push((", ".to_owned(), false));
        }
        name.push((table.clone(), true));
    }
    if tables.len() > TABLES_NAMED {
        name.push((strings::and_more(tables.len() - TABLES_NAMED), false));
    }
    Lost {
        name,
        value: strings::tables(tables.len()),
    }
}

/// The file that keeps which modules are on, as the user writes it, when the app
/// keeps it in a file. The package's delete removes the module's line from it.
pub fn statuses_file(app: &AppSnapshot) -> Option<String> {
    (app.layout.activator == "file").then(|| app.path_shown(&app.layout.statuses_file))
}

/// The box "What is lost", with "cannot be undone" on the right (S10.4). Returns its bottom.
fn lost_box(ui: &Ui, pal: &Palette, min: Pos2, width: f32, rows: &[Lost]) -> f32 {
    let height = 41.0 + rows.len() as f32 * size::LIST_LINE_H + 8.0;
    let rect = Rect::from_min_size(min, vec2(width, height));
    let mut y = draw::box_head(ui, pal, rect, strings::WHAT_IS_LOST, "");
    let mid = rect.top() + 1.0 + 8.0 + 9.0;
    let text = draw::paint_text(
        ui,
        pos2(rect.right() - 13.0, mid),
        Align2::RIGHT_CENTER,
        strings::CANNOT_BE_UNDONE,
        ty::LABEL,
        pal.bad,
    );
    draw::announce(ui, text, Role::Label, strings::CANNOT_BE_UNDONE);
    draw::mark_bad(ui, mark_rect(text.left() - 5.0, mid), pal.bad, pal.surface);
    // A path in a row shares the line of the row, so that both stand on one baseline.
    let mono = Type {
        size: 11.5,
        line: ty::BODY.line,
        ..ty::CODE
    };
    for (index, row) in rows.iter().enumerate() {
        let mid = y + size::LIST_LINE_H / 2.0;
        let value = draw::paint_text(
            ui,
            pos2(rect.right() - 13.0, mid),
            Align2::RIGHT_CENTER,
            &row.value,
            ty::BODY,
            pal.ink_2,
        );
        let parts: Vec<(&str, Type, Color32)> = row
            .name
            .iter()
            .map(|(text, is_mono)| {
                let part_type = if *is_mono { mono } else { ty::BODY };
                (text.as_str(), part_type, pal.ink)
            })
            .collect();
        let room = (value.left() - 12.0 - (rect.left() + 13.0)).max(40.0);
        let mut galley = draw::rich_galley(ui, &parts, f32::INFINITY);
        // A name too long for its row ends in an ellipsis, on one line.
        if galley.size().x > room {
            let mut job = eframe::egui::text::LayoutJob::default();
            for (text, part_type, color) in &parts {
                job.append(text, 0.0, part_type.format(*color));
            }
            job.wrap = eframe::egui::text::TextWrapping::truncate_at_width(room);
            galley = ui.painter().layout_job(job);
        }
        let said = row
            .name
            .iter()
            .map(|(text, _)| text.as_str())
            .collect::<String>();
        let at = draw::paint_galley(
            ui,
            pos2(rect.left() + 13.0, mid),
            Align2::LEFT_CENTER,
            galley,
        );
        let said = if row.value.is_empty() {
            said
        } else {
            format!("{said}, {}", row.value)
        };
        draw::announce(ui, at, Role::Label, &said);
        if index + 1 < rows.len() {
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

/// The tests Run tests runs: the module's, or under "All modules" those of the
/// modules that are on. On a 13.4 app `module:test --all` runs every module, on or
/// off, so it counts them all (design D8). The row and the panel give the same count.
fn tests_counted(app: &AppSnapshot, module: Option<&Module>, package_data: bool) -> u32 {
    match module {
        Some(module) => module.tests,
        None => app
            .modules
            .iter()
            .filter(|module| package_data || module.state == ModuleState::On)
            .map(|module| module.tests)
            .sum(),
    }
}

/// The sentence of Run tests: the count, the folder in mono, then the rest (S10.2 e).
/// On a 13.4 app (`package_data`) the sentence about the placeholder is left out,
/// and "All modules" counts every module, as `module:test --all` takes them all.
fn tests_text(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    app: &AppSnapshot,
    module: Option<&Module>,
    package_data: bool,
) -> f32 {
    let count = crate::strings::tests(tests_counted(app, module, package_data));
    let folder = module.map(|module| app.path_shown(&module.dir.join("tests")));
    let mono = Type {
        size: 11.5,
        ..ty::CODE
    };
    let mut parts = vec![(count.as_str(), ty::BODY, pal.ink_2)];
    let after = match (&folder, package_data) {
        (Some(folder), _) => {
            parts.push((" in ", ty::BODY, pal.ink_2));
            parts.push((folder.as_str(), mono, pal.ink_2));
            if package_data {
                ".".to_owned()
            } else {
                format!(". {}", strings::TESTS_TEXT)
            }
        }
        (None, true) => format!(" {}", strings::TESTS_IN_EVERY_MODULE),
        (None, false) => format!(" {} {}", strings::TESTS_IN_MODULES_ON, strings::TESTS_TEXT),
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
        (set.drop_views, strings::ALSO_DROP_VIEWS, "--drop-views"),
        (set.drop_types, strings::ALSO_DROP_TYPES, "--drop-types"),
        (set.parallel, strings::RUN_IN_PARALLEL, "--parallel"),
    ];
    if checks.iter().any(|(taken, _, _)| *taken) {
        y += 12.0;
        for (taken, label, flag) in checks {
            if !taken {
                continue;
            }
            // The package seeds after a pretend migrate: a dry run cannot seed.
            let off = flag == "--seed" && set.pretend && options.pretend;
            let value = match flag {
                "--pretend" => &mut options.pretend,
                "--drop-views" => &mut options.drop_views,
                "--drop-types" => &mut options.drop_types,
                "--parallel" => &mut options.parallel,
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
            flag: false,
        };
        let bottom = if set.class {
            let class = Field {
                label: strings::SEEDER_CLASS,
                hint: strings::class_hint(module.is_none()),
                placeholder: module.map(strings::module_seeder).unwrap_or_default(),
                mono: true,
                flag: false,
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
    // Run tests on 13.4: the filter, with its option in dim mono (design D8).
    if set.filter {
        y += 12.0;
        let filter = Field {
            label: strings::TESTS_FILTER,
            hint: "--filter",
            placeholder: String::new(),
            mono: true,
            flag: true,
        };
        let field_w = width.min(FIELD_MAX_W);
        y = filter.show(
            ui,
            pal,
            pos2(top.x, y),
            field_w,
            &mut options.filter,
            keyboard,
        );
    }
    y
}

/// A text field with its label and hint above it.
struct Field {
    label: &'static str,
    hint: &'static str,
    placeholder: String,
    mono: bool,
    /// The hint is the option itself, in dim mono as beside a checkbox.
    flag: bool,
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
        let (at, align, hint_type) = if self.flag {
            let at = pos2(label.right() + 5.0, label.center().y);
            (at, Align2::LEFT_CENTER, ty::PATH)
        } else {
            let at = pos2(label.right() + 4.0, label.top());
            (at, Align2::LEFT_TOP, ty::SECONDARY)
        };
        draw::label(ui, at, align, self.hint, hint_type, pal.ink_2);
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
