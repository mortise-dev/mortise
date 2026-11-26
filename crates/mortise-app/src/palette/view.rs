//! The command palette (S12.1, S12.2): search over modules, findings, apps and
//! Mortise's own commands. Nothing is run to fill it: it shows what is in memory.

use eframe::egui::{
    self, Align2, Color32, FontId, Key, Rect, RichText, Role, ScrollArea, Sense, Shadow, Ui, Vec2,
    WidgetInfo, pos2, vec2,
};
use mortise_core::{
    Action, AppSnapshot, Catalog, Detail, FindingState, MODULE_MAKE, ModuleState, Ran, Target,
    ThemeChoice, fuzzy, history,
};

use crate::palette::strings;
use crate::ui::draw::{self, Icon, StampState};
use crate::ui::theme::{Face, Palette as Colors, Type, radius, size, ty};

/// At most this many rows of a group are shown.
const GROUP_LIMIT: usize = 4;
/// At most this many actions and generators that ran are in "Recent".
const RECENT_RUNS: usize = 3;
/// The list scrolls after this many rows.
const VISIBLE_ROWS: f32 = 12.0;

/// What the palette keeps while it is open.
#[derive(Debug, Default)]
pub struct PaletteState {
    pub query: String,
    /// The row that Enter runs.
    pub row: usize,
    /// The query the row belongs to: another query starts at the first row again.
    row_query: String,
}

/// What a row of the palette does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Open a module of the app in scope.
    Module(String),
    /// The public surface of a module of the app in scope. The scan is not started.
    Surface(String),
    /// The Summary of "All modules" of the app in scope.
    AllModules,
    /// The Findings list of the app in scope.
    Findings,
    /// Open a finding where it lives: the module, and the other module of its link.
    Finding {
        module: String,
        other: Option<String>,
    },
    /// Another app, by its place on the rail.
    App(usize),
    AddApp,
    Settings,
    /// Settings, on the PHP section.
    PhpSettings,
    /// Show or hide the runner drawer.
    Runner,
    Theme(ThemeChoice),
    /// Open the Actions tab on an action. `None` is "All modules".
    Action {
        module: Option<String>,
        action: Action,
    },
    /// Open the form of a generator in a module, or the new module form with `None`.
    /// Nothing runs.
    Generate {
        module: Option<String>,
        generator: String,
    },
}

/// One row: what it says, what stands at its right end, what it does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub group: String,
    pub label: String,
    /// The code of a module, for its stamp.
    pub code: Option<String>,
    /// A finding state after the label: its word, and whether it is a problem.
    pub state: Option<(String, bool)>,
    /// What Enter does, in a word, the shortcut of the command, or when it ran.
    pub hint: String,
    /// The hint is a shortcut, drawn as keys.
    pub keys: bool,
    /// The artisan words of an action, after the label, in dim mono.
    pub detail: Option<String>,
    pub command: Command,
}

/// What the palette searches.
pub struct Source<'a> {
    /// The app in scope: its place on the rail and its snapshot, once it is read.
    pub scope: Option<(usize, &'a AppSnapshot)>,
    /// Every app: its name and how many problems it has.
    pub apps: &'a [(String, usize)],
    /// The modules opened last in the app in scope, the latest first.
    pub recent: &'a [String],
    /// The runs of the app in scope, the latest first.
    pub history: &'a [history::Entry],
    /// The generators of the app in scope: read from it, or shipped for its version.
    pub catalog: &'a Catalog,
    /// Now, in seconds since 1970, for the age of a run.
    pub now: i64,
    /// The module selected in the index of the app in scope. `None` is "All modules".
    pub selected: Option<&'a str>,
    pub mac: bool,
}

/// The rows for a query, in groups: modules of the app in scope, findings, other
/// apps, Mortise's commands. A group with no match is left out.
pub fn entries(query: &str, source: &Source<'_>) -> Vec<Entry> {
    let query = query.trim();
    let command = crate::strings::command_key(source.mac);
    let option = if source.mac {
        strings::KEY_OPTION
    } else {
        strings::KEY_ALT
    };
    let mut entries = Vec::new();
    // The best matches of a group, at most four, in the order of their score.
    let mut add = |group: &str, mut found: Vec<(u32, Entry)>| {
        found.sort_by_key(|(score, _)| *score);
        entries.extend(found.into_iter().take(GROUP_LIMIT).map(|(_, entry)| Entry {
            group: group.to_owned(),
            ..entry
        }));
    };
    let entry = |label: String, hint: &str, command: Command| Entry {
        group: String::new(),
        label,
        code: None,
        state: None,
        hint: hint.to_owned(),
        keys: hint.contains(' ') || hint.len() == 1,
        detail: None,
        command,
    };

    if let Some((index, app)) = source.scope {
        // The display name of the app, from the names of the rail.
        let app_name = source
            .apps
            .get(index)
            .map_or(app.name.as_str(), |(name, _)| name);
        let module_entry = |name: &str| {
            let module = app.module(name)?;
            let state = match (module.state, app.findings.state_of(name)) {
                (ModuleState::Off, _) => Some((crate::strings::OFF.to_owned(), false)),
                (_, FindingState::Problems(n)) => Some((crate::strings::problems(n), true)),
                _ => None,
            };
            Some(Entry {
                code: Some(module.code.clone()),
                state,
                ..entry(
                    module.name.clone(),
                    strings::OPEN,
                    Command::Module(module.name.clone()),
                )
            })
        };

        if query.is_empty() {
            // The recent module, the actions and generators that ran last, then the
            // other recent modules (S12.2 a). A row of a run opens its panel or its form.
            let mut modules = source.recent.iter().filter_map(|name| module_entry(name));
            let runs = mortise_core::recent_runs(source.history, source.catalog, app, RECENT_RUNS);
            let runs = runs.into_iter().map(|run| {
                let age = strings::ran_ago(source.now, run.started_ms.div_euclid(1000));
                let mut row = match run.ran {
                    Ran::Action { action, module } => {
                        let label = strings::action_label(action, module.as_deref());
                        let detail = action_words(action, module.as_deref());
                        let mut row = entry(label, &age, Command::Action { module, action });
                        row.detail = detail;
                        row
                    }
                    Ran::Generator { name, module } => {
                        let label = match &module {
                            Some(module) => strings::generate_row(&name, module),
                            None => crate::strings::generator_label(&name),
                        };
                        let command = Command::Generate {
                            module,
                            generator: name.clone(),
                        };
                        let mut row = entry(label, strings::FORM, command);
                        row.detail = Some(name);
                        row
                    }
                };
                row.keys = false;
                row
            });
            let recent = modules.next().into_iter().chain(runs).chain(modules);
            add(
                strings::GROUP_RECENT,
                recent
                    .enumerate()
                    .map(|(i, entry)| (i as u32, entry))
                    .collect(),
            );

            let mut findings = entry(
                crate::strings::findings_in(app_name),
                "F",
                Command::Findings,
            );
            findings.state = match (app.findings.problems(), app.findings.warnings()) {
                (0, 0) => None,
                (0, warnings) => Some((crate::strings::warnings(warnings), false)),
                (problems, _) => Some((crate::strings::problems(problems), true)),
            };
            let settings = entry(
                crate::strings::SETTINGS.to_owned(),
                &format!("{command} ,"),
                Command::Settings,
            );
            let all = entry(
                crate::strings::ALL_MODULES.to_owned(),
                crate::strings::SUMMARY,
                Command::AllModules,
            );
            add(
                strings::GROUP_GO_TO,
                vec![(0, all), (1, findings), (2, settings)],
            );
        } else {
            let modules = app.modules.iter().filter_map(|module| {
                let score = fuzzy(query, &format!("{} {}", module.name, module.code))?;
                Some((score, module_entry(&module.name)?))
            });
            add(&strings::modules_in(app_name), modules.collect());

            // The actions of the best module match, else of the selected module. The
            // word of a destructive action is not part of the module's name (design D8).
            let named = named_destructive(query);
            let module_query = query
                .split_whitespace()
                .filter(|word| !DESTRUCTIVE_WORDS.contains(&word.to_lowercase().as_str()))
                .collect::<Vec<_>>()
                .join(" ");
            let matched = app
                .modules
                .iter()
                .filter(|_| !module_query.is_empty())
                .filter_map(|module| {
                    Some((
                        fuzzy(&module_query, &format!("{} {}", module.name, module.code))?,
                        module,
                    ))
                })
                .min_by_key(|(score, _)| *score)
                .map(|(_, module)| module);
            let module = matched.or_else(|| source.selected.and_then(|name| app.module(name)));
            let (group, actions): (String, Vec<(Option<&str>, Action)>) = match module {
                Some(module) => {
                    let on = !matches!(module.state, ModuleState::Off);
                    let name = Some(module.name.as_str());
                    let actions = Action::for_module(on).into_iter();
                    (
                        strings::actions_for(&module.name),
                        actions.map(|action| (name, action)).collect(),
                    )
                }
                None => (
                    strings::ACTIONS_FOR_ALL.to_owned(),
                    Action::for_all().into_iter().map(|a| (None, a)).collect(),
                ),
            };
            // A destructive action only by its own word: reset, refresh and delete with
            // the module, fresh with "All modules" (design D8).
            let target = module.map(|module| module.name.as_str());
            let destructive = target
                .map(|name| Action::destructive(&Target::Module(name.to_owned())))
                .into_iter()
                .flatten()
                .map(|action| (target, *action))
                .chain(std::iter::once((None, Action::FreshMigrate)))
                .filter(|(_, action)| named.contains(action));
            // They come first, so that the limit of the group never leaves them out.
            let actions: Vec<(Option<&str>, Action)> = destructive.chain(actions).collect();
            let found = actions.into_iter().filter_map(|(module, action)| {
                let label = strings::action_label(action, module);
                let words = action_words(action, module);
                // Named by its own word, a destructive action comes first.
                if action.is_destructive() {
                    let command = Command::Action {
                        module: module.map(str::to_owned),
                        action,
                    };
                    let mut entry = entry(label, strings::ACTION, command);
                    entry.detail = words;
                    return Some((0, entry));
                }
                // An action named by the query comes before one that only has the module.
                let own = crate::strings::action_button(action);
                let score = match fuzzy(query, own) {
                    Some(score) => score,
                    None => {
                        1000 + fuzzy(
                            query,
                            &format!("{label} {}", words.as_deref().unwrap_or("")),
                        )?
                    }
                };
                let command = Command::Action {
                    module: module.map(str::to_owned),
                    action,
                };
                let mut entry = entry(label, strings::ACTION, command);
                entry.detail = words;
                Some((score, entry))
            });
            add(&group, found.collect());

            // The generators of the module the query names, else of the selected
            // module, matched by the rest of the query. A row opens the form.
            let found = generator_entries(query, app, source.catalog, source.selected)
                .into_iter()
                .map(|(score, label, detail, command)| {
                    let mut entry = entry(label, strings::FORM, command);
                    entry.detail = Some(detail);
                    (score, entry)
                });
            add(strings::GENERATORS, found.collect());

            let findings = app.findings.list.iter().filter_map(|finding| {
                let module = finding.module.clone()?;
                // A translation warning opens Check translations of its module.
                if let Detail::MissingTranslations { .. } = finding.detail {
                    let label = crate::strings::finding(finding, app, app_name);
                    let score = fuzzy(query, &label)?;
                    let command = Command::Action {
                        module: Some(module),
                        action: Action::CheckTranslations,
                    };
                    return Some((score, entry(label, crate::strings::ACTIONS, command)));
                }
                let (label, other) = match &finding.detail {
                    Detail::CircularUse { files } => {
                        let names: Vec<String> =
                            files.iter().map(|(name, _)| name.clone()).collect();
                        let other = app.circle_neighbour(&module).map(str::to_owned);
                        let label = match names.as_slice() {
                            [a, b] => crate::strings::use_each_other(a, b),
                            names => crate::strings::use_each_other_in_a_circle(names),
                        };
                        (label, other)
                    }
                    Detail::ReadsInternals { of, files, .. } => (
                        strings::reads_internals(*files as u64, &module, of),
                        Some(of.clone()),
                    ),
                    _ => (crate::strings::finding(finding, app, app_name), None),
                };
                let score = fuzzy(query, &label)?;
                let place = if other.is_some() {
                    crate::strings::BOUNDARY
                } else {
                    crate::strings::SUMMARY
                };
                Some((
                    score,
                    entry(label, place, Command::Finding { module, other }),
                ))
            });
            add(crate::strings::FINDINGS, findings.collect());

            // The public surface of a module. Opening the view does not start the scan.
            let surfaces = app.modules.iter().filter_map(|module| {
                let label = strings::scan_the_surface_of(&module.name);
                let score = fuzzy(query, &format!("{label} {}", module.code))?;
                let command = Command::Surface(module.name.clone());
                let mut entry = entry(label, crate::strings::BOUNDARY, command);
                entry.code = Some(module.code.clone());
                Some((score, entry))
            });
            add(crate::strings::PUBLIC_SURFACE, surfaces.collect());
        }
    }

    // The other apps, each with the shortcut that selects it.
    let scope = source.scope.map(|(index, _)| index);
    let apps = source
        .apps
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != scope)
        .filter_map(|(index, (name, problems))| {
            let score = if query.is_empty() {
                index as u32
            } else {
                fuzzy(query, name)?
            };
            let hint = if index < 9 {
                format!("{command} {option} {}", index + 1)
            } else {
                String::new()
            };
            let mut entry = entry(name.clone(), &hint, Command::App(index));
            entry.state = (*problems > 0).then(|| (crate::strings::problems(*problems), true));
            Some((score, entry))
        });
    let mut apps: Vec<(u32, Entry)> = apps.collect();
    let add_app = entry(
        crate::strings::ADD_APP.to_owned(),
        &format!("{command} O"),
        Command::AddApp,
    );
    if query.is_empty() {
        apps.truncate(GROUP_LIMIT - 1);
        apps.push((u32::MAX, add_app));
        add(strings::GROUP_APPS, apps);
    } else {
        add(strings::GROUP_APPS, apps);
        // Mortise's own commands. The theme has no shortcut, so it is found here.
        let commands = [
            (
                crate::strings::SETTINGS,
                format!("{command} ,"),
                Command::Settings,
            ),
            (strings::SETTINGS_PHP, String::new(), Command::PhpSettings),
            (
                crate::strings::ADD_APP,
                format!("{command} O"),
                Command::AddApp,
            ),
            (
                strings::SHOW_OR_HIDE_RUNNER,
                crate::strings::TOGGLE_RUNNER_KEYS.to_owned(),
                Command::Runner,
            ),
            (
                strings::THEME_SYSTEM,
                String::new(),
                Command::Theme(ThemeChoice::System),
            ),
            (
                strings::THEME_LIGHT,
                String::new(),
                Command::Theme(ThemeChoice::Light),
            ),
            (
                strings::THEME_DARK,
                String::new(),
                Command::Theme(ThemeChoice::Dark),
            ),
        ];
        let found = commands.into_iter().filter_map(|(label, hint, command)| {
            Some((
                fuzzy(query, label)?,
                entry(label.to_owned(), &hint, command),
            ))
        });
        add(crate::app::strings::BRAND, found.collect());
    }
    entries
}

/// The words that name the destructive actions, in the order of `named_destructive`.
const DESTRUCTIVE_WORDS: [&str; 4] = ["reset", "refresh", "fresh", "delete"];

/// The destructive actions whose own word is a word of `query`, in lower case.
/// Words, not parts of words: "refresh" does not name fresh (design D8).
fn named_destructive(query: &str) -> Vec<Action> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let actions = [
        Action::ResetMigrations,
        Action::RefreshMigrations,
        Action::FreshMigrate,
        Action::DeleteModule,
    ];
    DESTRUCTIVE_WORDS
        .into_iter()
        .zip(actions)
        .filter(|(word, _)| words.iter().any(|typed| typed == word))
        .map(|(_, action)| action)
        .collect()
}

/// The generator rows for a query: score, label, artisan name and command. A word
/// that matches a module names the module; the other words match the label and
/// the artisan name of a generator. Without a module only New module is offered.
fn generator_entries(
    query: &str,
    app: &AppSnapshot,
    catalog: &Catalog,
    selected: Option<&str>,
) -> Vec<(u32, String, String, Command)> {
    let module_of = |word: &str| {
        app.modules
            .iter()
            .filter_map(|module| {
                Some((
                    fuzzy(word, &format!("{} {}", module.name, module.code))?,
                    module,
                ))
            })
            .min_by_key(|(score, _)| *score)
    };
    let mut named = None;
    let mut rest = Vec::new();
    for word in query.split_whitespace() {
        match module_of(word) {
            Some((score, module)) if named.is_none_or(|(best, _)| score < best) => {
                named = Some((score, module));
            }
            Some(_) => {}
            None => rest.push(word),
        }
    }
    let rest = rest.join(" ");
    if rest.is_empty() {
        return Vec::new();
    }
    let module = named
        .map(|(_, module)| module.name.as_str())
        .or(selected)
        .and_then(|name| app.module(name).map(|module| module.name.clone()));
    catalog
        .grouped()
        .into_iter()
        .flat_map(|(_, generators)| generators)
        .filter_map(|generator| {
            let label = crate::strings::generator_label(&generator.name);
            let score = fuzzy(&rest, &format!("{label} {}", generator.name))?;
            let new_module = generator.name == MODULE_MAKE;
            let (label, module) = match (&module, new_module) {
                (_, true) => (label, None),
                (Some(module), false) => (
                    strings::generate_row(&generator.name, module),
                    Some(module.clone()),
                ),
                (None, false) => return None,
            };
            let command = Command::Generate {
                module,
                generator: generator.name.clone(),
            };
            Some((score, label, generator.name.clone(), command))
        })
        .collect()
}

/// The artisan words of an action on a module, or on all modules with `None`.
fn action_words(action: Action, module: Option<&str>) -> Option<String> {
    let command = action.artisan()?;
    Some(format!("{command} {}", module.unwrap_or("--all")))
}

/// What the palette asks for in a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    Close,
    /// Search in the next app on the rail.
    NextApp,
    Run(Command),
}

/// Paints the palette over a scrim: 620 px wide, 58 px from the top of the window.
/// It holds focus until it closes.
pub fn palette(
    ui: &Ui,
    pal: &Colors,
    state: &mut PaletteState,
    source: &Source<'_>,
    scope_name: &str,
) -> Option<PaletteAction> {
    if state.query != state.row_query {
        state.row_query = state.query.clone();
        state.row = 0;
    }
    let rows = entries(&state.query, source);
    let last = rows.len().saturating_sub(1);
    let mut action = None;

    // Up and Down move, Enter runs, Tab searches in the next app.
    let pressed = |key: Key| ui.input(|input| input.key_pressed(key));
    if pressed(Key::ArrowDown) {
        state.row = if state.row >= last { 0 } else { state.row + 1 };
    }
    if pressed(Key::ArrowUp) {
        state.row = if state.row == 0 { last } else { state.row - 1 };
    }
    state.row = state.row.min(last);
    if pressed(Key::Enter)
        && let Some(entry) = rows.get(state.row)
    {
        action = Some(PaletteAction::Run(entry.command.clone()));
    }
    if pressed(Key::Tab) {
        action = Some(PaletteAction::NextApp);
    }

    let id = egui::Id::unique("palette");
    let area = egui::Modal::default_area(id).anchor(Align2::CENTER_TOP, vec2(0.0, 58.0));
    let modal = egui::Modal::new(id)
        .frame(egui::Frame::NONE)
        .backdrop_color(pal.scrim)
        .area(area)
        .show(ui.ctx(), |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            // The height: the query row, the groups and rows, the footer.
            const GROUP_H: f32 = 28.0;
            let groups = rows
                .iter()
                .map(|entry| &entry.group)
                .collect::<std::collections::BTreeSet<_>>()
                .len();
            let list_h = if rows.is_empty() {
                64.0
            } else {
                (groups as f32 * GROUP_H + rows.len() as f32 * size::PALETTE_ROW_H + 6.0)
                    .min(VISIBLE_ROWS * size::PALETTE_ROW_H + 4.0 * GROUP_H)
            };
            let foot_h = 33.0;
            let height = 1.0 + size::PALETTE_INPUT_H + list_h + foot_h + 1.0;
            let (rect, _) = ui.allocate_exact_size(vec2(size::PALETTE_W, height), Sense::hover());

            let shadow = Shadow {
                offset: [0, 14],
                blur: 34,
                spread: 0,
                color: pal.shadow,
            };
            ui.painter().add(shadow.as_shape(rect, radius::OVERLAY));
            draw::outlined(ui, rect, radius::OVERLAY, pal.surface, pal.ink);

            // The query.
            let input = Rect::from_min_size(
                rect.min + vec2(1.0, 1.0),
                vec2(rect.width() - 2.0, size::PALETTE_INPUT_H),
            );
            draw::hline(
                ui,
                input.left()..input.right(),
                input.bottom() - 1.0,
                pal.line_2,
            );
            let glass = Rect::from_min_size(
                pos2(input.left() + 14.0, input.center().y - 8.0),
                Vec2::splat(16.0),
            );
            draw::icon(ui, glass, Icon::Search, pal.ink);
            let cap = draw::keycap(
                ui,
                pal,
                pos2(input.right() - 14.0, input.center().y),
                crate::strings::KEY_ESC,
            );
            let text_rect = Rect::from_min_max(
                pos2(glass.right() + 10.0, input.top()),
                pos2(cap.left() - 10.0, input.bottom() - 1.0),
            );
            let font = FontId::new(ty::PALETTE_INPUT.size, ty::PALETTE_INPUT.face.family());
            let hint = RichText::new(strings::SEARCH_HINT)
                .color(pal.ink_3)
                .font(font.clone());
            let edit = egui::TextEdit::singleline(&mut state.query)
                .id(ui.scope_id().with("palette-query"))
                .frame(egui::Frame::NONE)
                .margin(egui::Margin::ZERO)
                .font(font)
                .text_color(pal.ink)
                .desired_width(text_rect.width())
                .align(Align2::LEFT_CENTER)
                .hint_text(hint);
            let response = ui.put(text_rect, edit);
            response
                .widget_info(|| WidgetInfo::labeled(Role::TextInput, true, crate::strings::SEARCH));
            // The palette holds focus: Tab, Enter and the arrows do not leave the field.
            // Focus is asked only when the field lost it. Each request also restarts the
            // input method, and a restart in every frame loses keys under fcitx5 on Wayland.
            if !response.has_focus() {
                response.request_focus();
            }

            // The rows, in their groups.
            let list = Rect::from_min_size(
                pos2(rect.left() + 1.0, input.bottom()),
                vec2(rect.width() - 2.0, list_h),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(list), |ui| {
                ScrollArea::vertical()
                    .id_salt("palette-rows")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        if rows.is_empty() {
                            let text = strings::nothing_matches(state.query.trim());
                            let (slot, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), list_h),
                                Sense::hover(),
                            );
                            draw::paragraph(
                                ui,
                                slot.min + vec2(14.0, 14.0),
                                &text,
                                ty::BODY,
                                pal.ink_2,
                                slot.width() - 28.0,
                            );
                            return;
                        }
                        let mut group = "";
                        for (index, entry) in rows.iter().enumerate() {
                            if entry.group != group {
                                group = &entry.group;
                                let (slot, _) = ui.allocate_exact_size(
                                    vec2(ui.available_width(), GROUP_H),
                                    Sense::hover(),
                                );
                                draw::label(
                                    ui,
                                    pos2(slot.left() + 14.0, slot.top() + 9.0),
                                    Align2::LEFT_TOP,
                                    group,
                                    ty::LABEL,
                                    pal.ink_2,
                                );
                            }
                            let (row, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), size::PALETTE_ROW_H),
                                Sense::hover(),
                            );
                            let current = index == state.row;
                            if palette_row(ui, pal, row, entry, index, current) {
                                action = Some(PaletteAction::Run(entry.command.clone()));
                            }
                            if current && (pressed(Key::ArrowDown) || pressed(Key::ArrowUp)) {
                                ui.scroll_to_rect(row.expand2(vec2(0.0, GROUP_H)), None);
                            }
                        }
                    });
            });

            // Footer: the keys, and the app that is searched.
            let foot = Rect::from_min_max(
                pos2(rect.left() + 1.0, rect.bottom() - 1.0 - foot_h),
                pos2(rect.right() - 1.0, rect.bottom() - 1.0),
            );
            ui.painter().rect_filled(foot, 0, pal.panel);
            draw::hline(ui, foot.left()..foot.right(), foot.top(), pal.line);
            let keys = strings::FOOTER_KEYS;
            draw::label(
                ui,
                pos2(foot.left() + 14.0, foot.center().y),
                Align2::LEFT_CENTER,
                keys,
                ty::SECONDARY,
                pal.ink_2,
            );
            draw::label(
                ui,
                pos2(foot.right() - 14.0, foot.center().y),
                Align2::RIGHT_CENTER,
                scope_name,
                ty::SECONDARY,
                pal.ink_2,
            );
        });
    if modal.should_close() {
        action = Some(PaletteAction::Close);
    }
    action
}

/// One row: 32 px high. The current row has the well fill, a line above and below,
/// and the peg. Returns true when it was clicked.
fn palette_row(
    ui: &Ui,
    pal: &Colors,
    row: Rect,
    entry: &Entry,
    index: usize,
    current: bool,
) -> bool {
    let response = ui.interact(
        row,
        ui.scope_id().with(("palette-row", index)),
        Sense::click(),
    );
    response.widget_info(|| WidgetInfo::selected(Role::ListBoxOption, true, current, &entry.label));
    let behind = if current {
        ui.painter().rect_filled(row, 0, pal.well);
        draw::hline(ui, row.left()..row.right(), row.top(), pal.line_2);
        draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line_2);
        draw::peg(ui, pal, row);
        pal.well
    } else if response.hovered() {
        ui.painter().rect_filled(row, 0, pal.well);
        pal.well
    } else {
        pal.surface
    };

    let mid = row.center().y;
    // The hint, with Enter after it on the current row. Keys are drawn as keycaps
    // on the other rows.
    let mut right = row.right() - 14.0;
    if current {
        right = draw::keycap(ui, pal, pos2(right, mid), crate::strings::KEY_ENTER).left() - 6.0;
    }
    let chord = entry.keys;
    // A current row with no hint of its own says what Enter does.
    let hint = if current && entry.hint.is_empty() {
        strings::OPEN
    } else {
        entry.hint.as_str()
    };
    if !hint.is_empty() {
        right = if chord && !current {
            draw::keycap(ui, pal, pos2(right, mid), hint).left()
        } else {
            draw::paint_text(
                ui,
                pos2(right, mid),
                Align2::RIGHT_CENTER,
                hint,
                ty::SECONDARY,
                pal.ink_2,
            )
            .left()
        };
    }

    // The stamp and the state keep their size. The label, then the detail, take
    // the room left before the hint, with a gap; past it they end in an ellipsis.
    let end = right - 12.0;
    let state = entry.state.as_ref().map(|(word, problem)| {
        let color: Color32 = if *problem { pal.bad } else { pal.ink_2 };
        let face = if *problem {
            Face::Sans600
        } else {
            Face::Sans400
        };
        let mark = if *problem {
            size::STATE_MARK + 5.0
        } else {
            0.0
        };
        let text = draw::galley(
            ui,
            word,
            Type {
                face,
                ..ty::SECONDARY
            },
            color,
        );
        (12.0 + mark + text.size().x, *problem, color, text)
    });
    let state_w = state.as_ref().map_or(0.0, |(width, ..)| *width);
    let stamp_w = entry.code.as_ref().map_or(0.0, |_| 7.0 + size::STAMP_W);

    let label_type = if entry.code.is_some() {
        ty::BODY.with_face(Face::Sans600)
    } else {
        ty::BODY
    };
    let left = row.left() + 14.0;
    let label = draw::galley_elided(
        ui,
        &entry.label,
        label_type,
        pal.ink,
        (end - left - stamp_w - state_w).max(0.0),
    );
    let mut x = draw::paint_galley(ui, pos2(left, mid), Align2::LEFT_CENTER, label).right();
    if let Some(code) = &entry.code {
        let stamp_min = pos2(x + 7.0, (mid - size::STAMP_H / 2.0).round());
        x = draw::stamp(ui, pal, stamp_min, code, StampState::default()).right();
    }
    let room = end - state_w - (x + 12.0);
    if let Some(detail) = &entry.detail
        && room > 0.0
    {
        let detail = draw::galley_elided(ui, detail, ty::CODE, pal.ink_3, room);
        x = draw::paint_galley(ui, pos2(x + 12.0, mid), Align2::LEFT_CENTER, detail).right();
    }
    if let Some((_, problem, color, text)) = state {
        let mut at = x + 12.0;
        if problem {
            let mark = Rect::from_min_size(pos2(at, mid - 6.0), Vec2::splat(size::STATE_MARK));
            draw::mark_bad(ui, mark, color, behind);
            at = mark.right() + 5.0;
        }
        draw::paint_galley(ui, pos2(at, mid), Align2::LEFT_CENTER, text);
    }
    response.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(entries: &[Entry]) -> Vec<(&str, &str)> {
        entries
            .iter()
            .map(|entry| (entry.group.as_str(), entry.label.as_str()))
            .collect()
    }

    #[test]
    fn without_an_app_the_palette_offers_to_add_one_and_the_settings() {
        let source = Source {
            scope: None,
            apps: &[],
            recent: &[],
            history: &[],
            catalog: Catalog::v13_3(),
            now: 0,
            selected: None,
            mac: false,
        };
        assert_eq!(labels(&entries("", &source)), [("Apps", "Add app")]);
        assert_eq!(
            labels(&entries("set", &source)),
            [("Mortise", "Settings"), ("Mortise", "Settings: PHP")]
        );
        assert!(entries("zzq", &source).is_empty());
    }

    #[test]
    fn other_apps_are_found_with_their_shortcut() {
        let apps = [
            ("shop-api".to_owned(), 2),
            ("crm".to_owned(), 0),
            ("landing-pages".to_owned(), 1),
        ];
        let source = Source {
            scope: None,
            apps: &apps,
            recent: &[],
            history: &[],
            catalog: Catalog::v13_3(),
            now: 0,
            selected: None,
            mac: true,
        };
        let found = entries("land", &source);
        assert_eq!(labels(&found), [("Apps", "landing-pages")]);
        assert_eq!(found[0].hint, "Cmd Option 3");
        assert_eq!(found[0].state, Some(("1 problem".to_owned(), true)));
        assert_eq!(found[0].command, Command::App(2));

        let all = entries("", &source);
        assert_eq!(
            labels(&all),
            [
                ("Apps", "shop-api"),
                ("Apps", "crm"),
                ("Apps", "landing-pages"),
                ("Apps", "Add app")
            ]
        );
    }

    #[test]
    fn the_theme_is_a_command_of_mortise() {
        let source = Source {
            scope: None,
            apps: &[],
            recent: &[],
            history: &[],
            catalog: Catalog::v13_3(),
            now: 0,
            selected: None,
            mac: false,
        };
        let found = entries("dark", &source);
        assert_eq!(labels(&found), [("Mortise", strings::THEME_DARK)]);
        assert_eq!(found[0].command, Command::Theme(ThemeChoice::Dark));
    }
}
