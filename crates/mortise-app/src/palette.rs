//! The command palette (S12.1, S12.2): search over modules, findings, apps and
//! Mortise's own commands. Nothing is run to fill it: it shows what is in memory.

use eframe::egui::{
    self, Align2, Color32, FontId, Key, Rect, RichText, Role, ScrollArea, Sense, Shadow, Ui, Vec2,
    WidgetInfo, pos2, vec2,
};
use mortise_core::{AppSnapshot, Detail, FindingState, ModuleState, ThemeChoice, fuzzy};

use crate::draw::{self, Icon, StampState};
use crate::strings;
use crate::theme::{Face, Palette as Colors, Type, radius, size, ty};

/// At most this many rows of a group are shown.
const GROUP_LIMIT: usize = 4;
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
    Theme(ThemeChoice),
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
    /// What Enter does, in a word, or the shortcut of the command.
    pub hint: String,
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
    pub mac: bool,
}

/// The rows for a query, in groups: modules of the app in scope, findings, other
/// apps, Mortise's commands. A group with no match is left out.
pub fn entries(query: &str, source: &Source<'_>) -> Vec<Entry> {
    let query = query.trim();
    let command = strings::command_key(source.mac);
    let option = if source.mac { "Option" } else { "Alt" };
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
        command,
    };

    if let Some((_, app)) = source.scope {
        let module_entry = |name: &str| {
            let module = app.module(name)?;
            let state = match (module.state, app.findings.state_of(name)) {
                (ModuleState::Off, _) => Some((strings::OFF.to_owned(), false)),
                (_, FindingState::Problems(n)) => Some((strings::problems(n), true)),
                _ => None,
            };
            Some(Entry {
                code: Some(module.code.clone()),
                state,
                ..entry(
                    module.name.clone(),
                    "Open",
                    Command::Module(module.name.clone()),
                )
            })
        };

        if query.is_empty() {
            let recent = source.recent.iter().filter_map(|name| module_entry(name));
            add(
                "Recent",
                recent
                    .enumerate()
                    .map(|(i, entry)| (i as u32, entry))
                    .collect(),
            );

            let mut findings = entry(strings::findings_in(&app.name), "F", Command::Findings);
            findings.state = match (app.findings.problems(), app.findings.warnings()) {
                (0, 0) => None,
                (0, warnings) => Some((strings::warnings(warnings), false)),
                (problems, _) => Some((strings::problems(problems), true)),
            };
            let settings = entry(
                "Settings".to_owned(),
                &format!("{command} ,"),
                Command::Settings,
            );
            let all = entry(
                strings::ALL_MODULES.to_owned(),
                strings::SUMMARY,
                Command::AllModules,
            );
            add("Go to", vec![(0, all), (1, findings), (2, settings)]);
        } else {
            let modules = app.modules.iter().filter_map(|module| {
                let score = fuzzy(query, &format!("{} {}", module.name, module.code))?;
                Some((score, module_entry(&module.name)?))
            });
            add(&format!("Modules in {}", app.name), modules.collect());

            let findings = app.findings.list.iter().filter_map(|finding| {
                let module = finding.module.clone()?;
                let (label, other) = match &finding.detail {
                    Detail::CircularUse { files } => {
                        let names: Vec<String> =
                            files.iter().map(|(name, _)| name.clone()).collect();
                        let other = names.iter().find(|name| **name != module).cloned();
                        let label = match names.as_slice() {
                            [a, b] => strings::use_each_other(a, b),
                            names => strings::use_each_other_in_a_circle(names),
                        };
                        (label, other)
                    }
                    Detail::ReadsInternals { of, files, .. } => (
                        format!(
                            "{} in {module} read {of} internals",
                            strings::files(*files as u64)
                        ),
                        Some(of.clone()),
                    ),
                    _ => (strings::finding(finding, app), None),
                };
                let score = fuzzy(query, &label)?;
                let place = if other.is_some() {
                    strings::BOUNDARY
                } else {
                    strings::SUMMARY
                };
                Some((
                    score,
                    entry(label, place, Command::Finding { module, other }),
                ))
            });
            add(strings::FINDINGS, findings.collect());

            // The public surface of a module. Opening the view does not start the scan.
            let surfaces = app.modules.iter().filter_map(|module| {
                let label = strings::scan_the_surface_of(&module.name);
                let score = fuzzy(query, &format!("{label} {}", module.code))?;
                let command = Command::Surface(module.name.clone());
                let mut entry = entry(label, strings::BOUNDARY, command);
                entry.code = Some(module.code.clone());
                Some((score, entry))
            });
            add(strings::PUBLIC_SURFACE, surfaces.collect());
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
            entry.state = (*problems > 0).then(|| (strings::problems(*problems), true));
            Some((score, entry))
        });
    let mut apps: Vec<(u32, Entry)> = apps.collect();
    let add_app = entry(
        strings::ADD_APP.to_owned(),
        &format!("{command} O"),
        Command::AddApp,
    );
    if query.is_empty() {
        apps.truncate(GROUP_LIMIT - 1);
        apps.push((u32::MAX, add_app));
        add("Apps", apps);
    } else {
        add("Apps", apps);
        // Mortise's own commands. The theme has no shortcut, so it is found here.
        let commands = [
            ("Settings", format!("{command} ,"), Command::Settings),
            (strings::ADD_APP, format!("{command} O"), Command::AddApp),
            (
                "Theme: follow the system",
                String::new(),
                Command::Theme(ThemeChoice::System),
            ),
            (
                "Theme: light",
                String::new(),
                Command::Theme(ThemeChoice::Light),
            ),
            (
                "Theme: dark",
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
        add("Mortise", found.collect());
    }
    entries
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
    let modal = egui::Modal::new(id).frame(egui::Frame::NONE).backdrop_color(pal.scrim).area(area).show(
        ui.ctx(),
        |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            // The height: the query row, the groups and rows, the footer.
            const GROUP_H: f32 = 28.0;
            let groups = rows.iter().map(|entry| &entry.group).collect::<std::collections::BTreeSet<_>>().len();
            let list_h = if rows.is_empty() {
                64.0
            } else {
                (groups as f32 * GROUP_H + rows.len() as f32 * size::PALETTE_ROW_H + 6.0)
                    .min(VISIBLE_ROWS * size::PALETTE_ROW_H + 4.0 * GROUP_H)
            };
            let foot_h = 33.0;
            let height = 1.0 + size::PALETTE_INPUT_H + list_h + foot_h + 1.0;
            let (rect, _) = ui.allocate_exact_size(vec2(size::PALETTE_W, height), Sense::hover());

            let shadow = Shadow { offset: [0, 14], blur: 34, spread: 0, color: pal.shadow };
            ui.painter().add(shadow.as_shape(rect, radius::OVERLAY));
            draw::outlined(ui, rect, radius::OVERLAY, pal.surface, pal.ink);

            // The query.
            let input = Rect::from_min_size(rect.min + vec2(1.0, 1.0), vec2(rect.width() - 2.0, size::PALETTE_INPUT_H));
            draw::hline(ui, input.left()..input.right(), input.bottom() - 1.0, pal.line_2);
            let glass = Rect::from_min_size(pos2(input.left() + 14.0, input.center().y - 8.0), Vec2::splat(16.0));
            draw::icon(ui, glass, Icon::Search, pal.ink);
            let cap = draw::keycap(ui, pal, pos2(input.right() - 14.0, input.center().y), "Esc");
            let text_rect = Rect::from_min_max(pos2(glass.right() + 10.0, input.top()), pos2(cap.left() - 10.0, input.bottom() - 1.0));
            let font = FontId::new(ty::PALETTE_INPUT.size, ty::PALETTE_INPUT.face.family());
            let hint = RichText::new("Search modules, findings and apps").color(pal.ink_3).font(font.clone());
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
            response.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, strings::SEARCH));
            // The palette holds focus: Tab, Enter and the arrows do not leave the field.
            response.request_focus();

            // The rows, in their groups.
            let list = Rect::from_min_size(pos2(rect.left() + 1.0, input.bottom()), vec2(rect.width() - 2.0, list_h));
            ui.scope_builder(egui::UiBuilder::new().max_rect(list), |ui| {
                ScrollArea::vertical().id_salt("palette-rows").auto_shrink(false).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    if rows.is_empty() {
                        let text = format!(
                            "Nothing matches \"{}\". Try a module name, the name of an app, or a word of a finding.",
                            state.query.trim()
                        );
                        let (slot, _) = ui.allocate_exact_size(vec2(ui.available_width(), list_h), Sense::hover());
                        draw::paragraph(ui, slot.min + vec2(14.0, 14.0), &text, ty::BODY, pal.ink_2, slot.width() - 28.0);
                        return;
                    }
                    let mut group = "";
                    for (index, entry) in rows.iter().enumerate() {
                        if entry.group != group {
                            group = &entry.group;
                            let (slot, _) = ui.allocate_exact_size(vec2(ui.available_width(), GROUP_H), Sense::hover());
                            draw::label(ui, pos2(slot.left() + 14.0, slot.top() + 9.0), Align2::LEFT_TOP, group, ty::LABEL, pal.ink_2);
                        }
                        let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), size::PALETTE_ROW_H), Sense::hover());
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
            let foot = Rect::from_min_max(pos2(rect.left() + 1.0, rect.bottom() - 1.0 - foot_h), pos2(rect.right() - 1.0, rect.bottom() - 1.0));
            ui.painter().rect_filled(foot, 0, pal.panel);
            draw::hline(ui, foot.left()..foot.right(), foot.top(), pal.line);
            let keys = "Up and Down to move, Enter to run, Tab to switch app";
            draw::label(ui, pos2(foot.left() + 14.0, foot.center().y), Align2::LEFT_CENTER, keys, ty::SECONDARY, pal.ink_2);
            draw::label(ui, pos2(foot.right() - 14.0, foot.center().y), Align2::RIGHT_CENTER, scope_name, ty::SECONDARY, pal.ink_2);
        },
    );
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
    // What Enter does on the current row; elsewhere the place or the shortcut.
    let mut right = row.right() - 14.0;
    if current {
        right = draw::keycap(ui, pal, pos2(right, mid), "Enter").left() - 6.0;
    }
    let chord = entry.hint.contains(' ') || entry.hint.len() == 1;
    let hint = if current && !chord {
        "Open"
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

    let label_type = if entry.code.is_some() {
        ty::BODY.with_face(Face::Sans600)
    } else {
        ty::BODY
    };
    let label = draw::galley_elided(
        ui,
        &entry.label,
        label_type,
        pal.ink,
        right - 12.0 - (row.left() + 14.0),
    );
    let mut x =
        draw::paint_galley(ui, pos2(row.left() + 14.0, mid), Align2::LEFT_CENTER, label).right();
    if let Some(code) = &entry.code {
        let stamp_min = pos2(x + 7.0, (mid - size::STAMP_H / 2.0).round());
        x = draw::stamp(ui, pal, stamp_min, code, StampState::default()).right();
    }
    if let Some((word, problem)) = &entry.state {
        let color: Color32 = if *problem { pal.bad } else { pal.ink_2 };
        let mut at = x + 12.0;
        if *problem {
            let mark = Rect::from_min_size(pos2(at, mid - 6.0), Vec2::splat(size::STATE_MARK));
            draw::mark_bad(ui, mark, color, behind);
            at = mark.right() + 5.0;
        }
        let state_type = Type {
            face: if *problem {
                Face::Sans600
            } else {
                Face::Sans400
            },
            ..ty::SECONDARY
        };
        draw::paint_text(
            ui,
            pos2(at, mid),
            Align2::LEFT_CENTER,
            word,
            state_type,
            color,
        );
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
            mac: false,
        };
        assert_eq!(labels(&entries("", &source)), [("Apps", "Add app")]);
        assert_eq!(labels(&entries("set", &source)), [("Mortise", "Settings")]);
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
            mac: false,
        };
        let found = entries("dark", &source);
        assert_eq!(labels(&found), [("Mortise", "Theme: dark")]);
        assert_eq!(found[0].command, Command::Theme(ThemeChoice::Dark));
    }
}
