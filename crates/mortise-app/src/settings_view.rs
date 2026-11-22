//! Settings (S8.1, S8.3, S8.4): they take the place of the index and the workspace.
//!
//! There is no Save button. A choice applies at once, and a field applies when it
//! loses focus.

use eframe::egui::{
    Align2, Color32, Rect, Role, ScrollArea, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{AppEntry, Editor, General, IndexRows, Motion, Settings, ThemeChoice};

use crate::draw::{self, Button, Icon};
use crate::strings;
use crate::theme::{Face, Palette, Type, radius, size, ty};

/// The sections of Settings. PHP joins them with the version that runs commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    #[default]
    Apps,
    General,
    Keyboard,
}

impl Section {
    pub const ALL: [Section; 3] = [Section::Apps, Section::General, Section::Keyboard];

    pub fn label(self) -> &'static str {
        match self {
            Section::Apps => "Apps",
            Section::General => "General",
            Section::Keyboard => "Keyboard",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Section::Apps => Icon::App,
            Section::General => Icon::Settings,
            Section::Keyboard => Icon::List,
        }
    }
}

/// What Settings keep while they are open.
#[derive(Debug, Default)]
pub struct SettingsState {
    pub section: Section,
    /// The app whose form is shown in the Apps section.
    pub app: usize,
    /// The text of the fields of that app, and the app they were filled from.
    fields: Fields,
    fields_of: Option<(usize, AppEntry)>,
    /// The text of the custom editor command.
    editor_command: String,
    editor_command_set: bool,
    /// The text of the shortcut filter.
    pub shortcut_filter: String,
}

#[derive(Debug, Default)]
struct Fields {
    name: String,
    monogram: String,
    report: String,
    limit: String,
}

impl SettingsState {
    pub fn open(section: Section, app: usize) -> Self {
        Self {
            section,
            app,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsAction {
    /// Back to the app.
    Close,
    /// The settings changed and must be kept. `app` names an app whose way of
    /// being read changed.
    Changed { reread: Option<usize> },
    /// Ask before an app is removed.
    RemoveApp(usize),
}

/// One shortcut of the Keyboard section: what it does, its keys as written for
/// Windows and Linux, and where it works.
const SHORTCUTS: &[(&str, &str, &str)] = &[
    ("Open the command palette", "Ctrl K", "Anywhere"),
    ("Collapse or open the module index", "Ctrl B", "Anywhere"),
    (
        "Select app 1 to 9 on the rail",
        "Ctrl Alt 1 to 9",
        "Anywhere",
    ),
    ("Add app", "Ctrl O", "Anywhere"),
    ("Settings", "Ctrl ,", "Anywhere"),
    ("Read the app again, from disk", "Ctrl R", "Anywhere"),
    ("Close the topmost thing", "Esc", "Anywhere"),
    (
        "Next or previous module",
        "J  K  or Down  Up",
        "Main window",
    ),
    ("First or last row of the index", "Home  End", "Main window"),
    ("Filter modules", "/  or Ctrl F", "Main window"),
    ("Change the order of the index", "S", "Main window"),
    (
        "Summary, Boundary, Generate, Actions, Metrics",
        "1  2  3  4  5",
        "Main window",
    ),
    ("Findings list of the app", "F", "Main window"),
    ("Open the selected module in the editor", "E", "Main window"),
    ("Show the shortcuts", "?", "Main window"),
    (
        "Move, run, close",
        "Up  Down  Enter  Esc",
        "Command palette",
    ),
    ("Search in the next app", "Tab", "Command palette"),
    ("Press the focused button, cancel", "Enter  Esc", "Dialog"),
];

/// Paints the list of sections into the index column. The rect includes the joint
/// at its right edge.
pub fn sections(
    ui: &Ui,
    pal: &Palette,
    rect: Rect,
    state: &mut SettingsState,
    back_to: &str,
    keyboard: bool,
) -> Option<SettingsAction> {
    ui.painter().rect_filled(rect, 0, pal.panel);
    draw::vline(
        ui,
        rect.right() - 1.0,
        rect.top()..rect.bottom(),
        pal.line_2,
    );
    let left = rect.left() + 12.0;
    let right = rect.right() - 1.0 - 12.0;

    let version = format!("Mortise {}", env!("CARGO_PKG_VERSION"));
    draw::label(
        ui,
        pos2(right, rect.top() + 30.0),
        Align2::RIGHT_BOTTOM,
        &version,
        ty::SECONDARY,
        pal.ink_2,
    );
    let title = draw::paint_text(
        ui,
        pos2(left, rect.top() + 12.0),
        Align2::LEFT_TOP,
        "Settings",
        ty::APP_NAME,
        pal.ink,
    );
    draw::announce(ui, title, Role::Heading, "Settings");
    draw::label(
        ui,
        pos2(left, rect.top() + 33.0),
        Align2::LEFT_TOP,
        "Kept on this computer",
        ty::SECONDARY,
        pal.ink_2,
    );
    let head_bottom = rect.top() + 12.0 + 20.0 + 1.0 + 16.0 + 13.0;
    draw::hline(
        ui,
        rect.left()..rect.right() - 1.0,
        head_bottom - 1.0,
        pal.line,
    );

    // One row per section, joined to the workspace like a module row.
    let mut y = head_bottom + 8.0;
    for section in Section::ALL {
        let row = Rect::from_min_max(pos2(rect.left() + 8.0, y), pos2(rect.right(), y + 32.0));
        let selected = state.section == section;
        let response = ui.interact(
            row,
            ui.scope_id().with(("settings-section", section.label())),
            Sense::click(),
        );
        response.widget_info(|| {
            WidgetInfo::selected(Role::ListBoxOption, true, selected, section.label())
        });
        if response.clicked() {
            state.section = section;
        }
        if selected {
            ui.painter().rect_filled(row, 0, pal.surface);
            draw::hline(ui, row.left()..row.right(), row.top(), pal.line_2);
            draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line_2);
            draw::vline(ui, row.left(), row.top()..row.bottom(), pal.line_2);
            draw::peg(ui, pal, row);
        } else if response.hovered() {
            ui.painter().rect_filled(
                Rect::from_min_max(row.min, pos2(row.right() - 1.0, row.bottom())),
                0,
                pal.well,
            );
        }
        let icon = Rect::from_min_size(
            pos2(row.left() + 24.0, row.center().y - 8.0),
            Vec2::splat(size::ICON),
        );
        draw::icon(
            ui,
            icon,
            section.icon(),
            if selected { pal.ink } else { pal.ink_2 },
        );
        draw::paint_text(
            ui,
            pos2(row.left() + 60.0, row.center().y),
            Align2::LEFT_CENTER,
            section.label(),
            ty::ROW_NAME,
            pal.ink,
        );
        if selected && keyboard && response.has_focus() {
            draw::focus_ring(ui, pal, row, 0);
        }
        y += 32.0;
    }

    // The way back, at the foot.
    let foot_top = rect.bottom() - 34.0;
    draw::hline(ui, rect.left()..rect.right() - 1.0, foot_top, pal.line);
    let back = format!("Back to {back_to}");
    let foot = Rect::from_min_max(
        pos2(rect.left(), foot_top + 1.0),
        pos2(rect.right() - 1.0, rect.bottom()),
    );
    let response = ui.interact(foot, ui.scope_id().with("settings-back"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, &back));
    if response.hovered() {
        ui.painter().rect_filled(foot, 0, pal.well);
    }
    let cap = draw::keycap(ui, pal, pos2(right, foot.center().y), "Esc");
    let text = draw::galley_elided(ui, &back, ty::SECONDARY, pal.ink_2, cap.left() - 8.0 - left);
    draw::paint_galley(ui, pos2(left, foot.center().y), Align2::LEFT_CENTER, text);
    response.clicked().then_some(SettingsAction::Close)
}

/// Paints the open section into the workspace.
#[expect(clippy::too_many_arguments)]
pub fn section(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    state: &mut SettingsState,
    settings: &mut Settings,
    versions: &[String],
    custom_frame: bool,
    keyboard: bool,
    mac: bool,
) -> Option<SettingsAction> {
    ui.painter().rect_filled(rect, 0, pal.surface);
    let head = Rect::from_min_size(rect.min, vec2(rect.width(), 46.0));
    ui.painter().rect_filled(head, 0, pal.panel);
    draw::hline(
        ui,
        head.left()..head.right(),
        head.bottom() - 1.0,
        pal.line_2,
    );
    let label = state.section.label();
    let title = draw::paint_text(
        ui,
        pos2(head.left() + 16.0, head.top() + 12.0),
        Align2::LEFT_TOP,
        label,
        ty::TITLE,
        pal.ink,
    );
    draw::announce(ui, title, Role::Heading, label);

    let body = Rect::from_min_max(pos2(rect.left(), head.bottom()), rect.max);
    let mut action = None;
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        ScrollArea::vertical()
            .id_salt(("settings", label))
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let origin = ui.cursor().min;
                let width = ui.available_width();
                let form = Form {
                    pal,
                    keyboard,
                    left: origin.x + 16.0,
                    width: width - 32.0,
                };
                let top = origin.y + 14.0;
                let bottom = match state.section {
                    Section::General => form.general(
                        ui,
                        top,
                        state,
                        &mut settings.general,
                        custom_frame,
                        &mut action,
                    ),
                    Section::Apps => form.apps(ui, top, state, settings, versions, &mut action),
                    Section::Keyboard => form.keyboard(ui, top, state, mac),
                };
                ui.allocate_rect(
                    Rect::from_min_max(origin, pos2(origin.x + width, bottom + 16.0)),
                    Sense::hover(),
                );
            });
    });
    action
}

struct Form<'a> {
    pal: &'a Palette,
    keyboard: bool,
    left: f32,
    width: f32,
}

impl Form<'_> {
    /// A label in weight 600 with a hint in weight 400 after it. Returns the y under it.
    fn label(&self, ui: &Ui, y: f32, label: &str, hint: &str) -> f32 {
        let rect = draw::label(
            ui,
            pos2(self.left, y),
            Align2::LEFT_TOP,
            label,
            ty::LABEL,
            self.pal.ink,
        );
        if !hint.is_empty() {
            draw::label(
                ui,
                pos2(rect.right() + 4.0, y),
                Align2::LEFT_TOP,
                hint,
                ty::SECONDARY,
                self.pal.ink_2,
            );
        }
        y + ty::LABEL.line + 4.0
    }

    /// A labelled choice of one of several. Returns the y under it and the choice that was clicked.
    fn choice(
        &self,
        ui: &Ui,
        y: f32,
        label: &str,
        hint: &str,
        options: &[&str],
        selected: usize,
    ) -> (f32, Option<usize>) {
        let y = self.label(ui, y, label, hint);
        let (track, clicked) = draw::segmented(
            ui,
            self.pal,
            pos2(self.left, y),
            label,
            options,
            selected,
            self.keyboard,
        );
        (track.bottom() + 12.0, clicked)
    }

    // --- General (S8.3) ---

    fn general(
        &self,
        ui: &mut Ui,
        mut y: f32,
        state: &mut SettingsState,
        general: &mut General,
        custom_frame: bool,
        action: &mut Option<SettingsAction>,
    ) -> f32 {
        let mut changed = false;

        let themes = [ThemeChoice::System, ThemeChoice::Light, ThemeChoice::Dark];
        let at = themes
            .iter()
            .position(|theme| *theme == general.theme)
            .unwrap_or(0);
        let hint = "System follows the operating system and changes with it";
        let (next, clicked) = self.choice(ui, y, "Theme", hint, &["System", "Light", "Dark"], at);
        if let Some(clicked) = clicked {
            general.theme = themes[clicked];
            changed = true;
        }
        y = next;

        let sizes = [100, 110, 125, 150];
        let at = sizes
            .iter()
            .position(|size| *size == general.text_size)
            .unwrap_or(0);
        let hint = "every size in the interface scales with it";
        let (next, clicked) = self.choice(
            ui,
            y,
            "Text size",
            hint,
            &["100%", "110%", "125%", "150%"],
            at,
        );
        if let Some(clicked) = clicked {
            general.text_size = sizes[clicked];
            changed = true;
        }
        y = next;

        let motions = [Motion::System, Motion::Reduced, Motion::Full];
        let at = motions
            .iter()
            .position(|motion| *motion == general.motion)
            .unwrap_or(0);
        let hint = "System follows the reduced motion setting";
        let (next, clicked) =
            self.choice(ui, y, "Motion", hint, &["System", "Reduced", "Full"], at);
        if let Some(clicked) = clicked {
            general.motion = motions[clicked];
            changed = true;
        }
        y = next;

        let rows = [
            IndexRows::Automatic,
            IndexRows::TwoLines,
            IndexRows::OneLine,
        ];
        let at = rows
            .iter()
            .position(|rows| *rows == general.index_rows)
            .unwrap_or(0);
        let hint = "Automatic is two lines up to 20 modules, one line above";
        let (next, clicked) = self.choice(
            ui,
            y,
            "Module index rows",
            hint,
            &["Automatic", "Two lines", "One line"],
            at,
        );
        if let Some(clicked) = clicked {
            general.index_rows = rows[clicked];
            changed = true;
        }
        y = next;

        let editors = [
            Editor::VsCode,
            Editor::PhpStorm,
            Editor::Zed,
            Editor::Custom,
        ];
        let at = editors
            .iter()
            .position(|editor| *editor == general.editor)
            .unwrap_or(0);
        let (next, clicked) = self.choice(
            ui,
            y,
            "Open files with",
            "",
            &["VS Code", "PhpStorm", "Zed", "Custom"],
            at,
        );
        if let Some(clicked) = clicked {
            general.editor = editors[clicked];
            changed = true;
        }
        // The command: fixed for an editor Mortise knows, written by the user for Custom.
        let field = Rect::from_min_size(
            pos2(self.left, next - 6.0),
            vec2(self.width.min(560.0), size::FIELD_H),
        );
        if general.editor == Editor::Custom {
            if !state.editor_command_set {
                state.editor_command = general.editor_command.clone();
                state.editor_command_set = true;
            }
            let response = draw::text_field(
                ui,
                self.pal,
                field,
                "Editor command",
                &mut state.editor_command,
                true,
                self.keyboard,
            );
            if response.lost_focus() && state.editor_command.trim() != general.editor_command {
                general.editor_command = state.editor_command.trim().to_owned();
                changed = true;
            }
        } else {
            draw::outlined(ui, field, radius::CONTROL, self.pal.well, self.pal.line);
            let command = general.editor_command();
            draw::label(
                ui,
                pos2(field.left() + 9.0, field.center().y),
                Align2::LEFT_CENTER,
                command,
                ty::CODE,
                self.pal.ink_2,
            );
        }
        y = field.bottom() + 12.0;

        // Only where Mortise draws the window controls.
        if custom_frame {
            let hint = "where the close, minimise and maximise buttons sit";
            let at = usize::from(general.window_controls_left);
            let (next, clicked) = self.choice(
                ui,
                y,
                "Window controls on Linux",
                hint,
                &["Right", "Left"],
                at,
            );
            if let Some(clicked) = clicked {
                general.window_controls_left = clicked == 1;
                changed = true;
            }
            y = next;

            // Not in the drawing: the fallback of section 8, offered as a setting.
            let hint = "The platform's own bar keeps what its window manager offers";
            let at = usize::from(general.system_title_bar);
            let (next, clicked) =
                self.choice(ui, y, "Title bar", hint, &["Mortise", "Platform"], at);
            if let Some(clicked) = clicked {
                general.system_title_bar = clicked == 1;
                changed = true;
            }
            y = next;
        }

        if changed {
            *action = Some(SettingsAction::Changed { reread: None });
        }
        y
    }

    // --- Apps (S8.1) ---

    fn apps(
        &self,
        ui: &mut Ui,
        mut y: f32,
        state: &mut SettingsState,
        settings: &mut Settings,
        versions: &[String],
        action: &mut Option<SettingsAction>,
    ) -> f32 {
        let pal = self.pal;
        if settings.apps.is_empty() {
            return draw::paragraph(
                ui,
                pos2(self.left, y),
                strings::NO_APPS_BODY,
                ty::BODY,
                pal.ink_2,
                self.width.min(560.0),
            )
            .bottom();
        }
        state.app = state.app.min(settings.apps.len() - 1);

        // The apps: a table whose selected row has its form below.
        let table = Rect::from_min_size(
            pos2(self.left, y),
            vec2(
                self.width,
                size::TABLE_HEAD_H + settings.apps.len() as f32 * size::TABLE_ROW_H + 1.0,
            ),
        );
        draw::outlined(ui, table, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
        let name_x = table.left() + 14.0;
        let folder_x = (table.left() + table.width() * 0.30).round();
        let versions_x = (table.left() + table.width() * 0.66).round();
        let head_mid = table.top() + size::TABLE_HEAD_H / 2.0;
        for (x, label) in [
            (name_x, "App"),
            (folder_x, "Folder"),
            (versions_x, "Versions"),
        ] {
            draw::label(
                ui,
                pos2(x, head_mid),
                Align2::LEFT_CENTER,
                label,
                ty::LABEL,
                pal.ink_2,
            );
        }
        draw::hline(
            ui,
            table.left() + 1.0..table.right() - 1.0,
            table.top() + size::TABLE_HEAD_H - 1.0,
            pal.line_2,
        );
        for (i, app) in settings.apps.iter().enumerate() {
            let top = table.top() + size::TABLE_HEAD_H + i as f32 * size::TABLE_ROW_H;
            let row = Rect::from_min_max(
                pos2(table.left() + 1.0, top),
                pos2(table.right() - 1.0, top + size::TABLE_ROW_H),
            );
            let selected = i == state.app;
            let response =
                ui.interact(row, ui.scope_id().with(("settings-app", i)), Sense::click());
            response.widget_info(|| WidgetInfo::selected(Role::Row, true, selected, &app.name));
            if response.clicked() {
                state.app = i;
            }
            if selected {
                ui.painter().rect_filled(row, 0, pal.well);
                draw::hline(ui, row.left()..row.right(), row.top() - 1.0, pal.line_2);
                draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line_2);
                draw::peg(ui, pal, row);
            } else {
                if response.hovered() {
                    ui.painter().rect_filled(row, 0, pal.well);
                }
                if i + 1 < settings.apps.len() {
                    draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line);
                }
            }
            let mid = row.center().y;
            let tile = Rect::from_min_size(pos2(name_x, mid - 10.0), vec2(26.0, 20.0));
            draw::outlined(ui, tile, radius::STAMP, Color32::TRANSPARENT, pal.line_2);
            draw::paint_text(
                ui,
                tile.center(),
                Align2::CENTER_CENTER,
                &app.monogram,
                Type {
                    size: 10.5,
                    ..ty::MONOGRAM
                },
                pal.ink_2,
            );
            let name = draw::galley_elided(
                ui,
                &app.name,
                ty::BODY.with_face(Face::Sans600),
                pal.ink,
                folder_x - 12.0 - (tile.right() + 8.0),
            );
            draw::paint_galley(ui, pos2(tile.right() + 8.0, mid), Align2::LEFT_CENTER, name);
            let folder = strings::tilde(&app.root);
            let folder = draw::galley_elided(
                ui,
                &folder,
                Type {
                    size: 11.5,
                    ..ty::CODE
                },
                pal.ink_2,
                versions_x - 12.0 - folder_x,
            );
            draw::paint_galley(ui, pos2(folder_x, mid), Align2::LEFT_CENTER, folder);
            let version_text = versions.get(i).map_or("", String::as_str);
            let version = draw::galley_elided(
                ui,
                version_text,
                ty::BODY,
                pal.ink_2,
                table.right() - 12.0 - versions_x,
            );
            let version_rect =
                draw::paint_galley(ui, pos2(versions_x, mid), Align2::LEFT_CENTER, version);
            if !version_text.is_empty() {
                draw::announce(ui, version_rect, Role::Label, version_text);
            }
        }
        y = table.bottom() + 16.0;

        // The form of the selected app. Its fields follow the settings until one is edited.
        let index = state.app;
        if state
            .fields_of
            .as_ref()
            .is_none_or(|(of, entry)| *of != index || entry != &settings.apps[index])
        {
            let entry = &settings.apps[index];
            state.fields = Fields {
                name: entry.name.clone(),
                monogram: entry.monogram.clone(),
                report: entry.coverage_report.clone().unwrap_or_default(),
                limit: format!("{}%", entry.coverage_limit),
            };
            state.fields_of = Some((index, entry.clone()));
        }
        let heading = settings.apps[index].name.clone();
        let heading_rect = draw::paint_text(
            ui,
            pos2(self.left, y),
            Align2::LEFT_TOP,
            &heading,
            ty::BODY.with_face(Face::Sans700),
            pal.ink,
        );
        draw::announce(ui, heading_rect, Role::Heading, &heading);
        y = heading_rect.bottom() + 6.0;
        draw::hline(
            ui,
            self.left..self.left + self.width.min(560.0),
            y,
            pal.line,
        );
        y += 9.0;

        let form_w = self.width.min(560.0);
        let mut changed: Option<bool> = None; // Some(true) when the app must be read again
        let taken: Vec<String> = settings
            .apps
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, app)| app.monogram.clone())
            .collect();
        let entry = &mut settings.apps[index];

        // Name and monogram, side by side.
        let after = self.label(ui, y, strings::NAME, "");
        draw::label(
            ui,
            pos2(self.left + form_w - 96.0, y),
            Align2::LEFT_TOP,
            strings::MONOGRAM,
            ty::LABEL,
            pal.ink,
        );
        let name = Rect::from_min_size(
            pos2(self.left, after),
            vec2(form_w - 96.0 - 12.0, size::FIELD_H),
        );
        if draw::text_field(
            ui,
            pal,
            name,
            strings::NAME,
            &mut state.fields.name,
            false,
            self.keyboard,
        )
        .lost_focus()
        {
            let value = state.fields.name.trim();
            if !value.is_empty() && value != entry.name {
                entry.name = value.to_owned();
                changed = Some(false);
            }
        }
        let monogram = Rect::from_min_size(
            pos2(self.left + form_w - 96.0, after),
            vec2(96.0, size::FIELD_H),
        );
        if draw::text_field(
            ui,
            pal,
            monogram,
            strings::MONOGRAM,
            &mut state.fields.monogram,
            false,
            self.keyboard,
        )
        .lost_focus()
        {
            // One or two letters or digits, not those of another app.
            let value: String = state
                .fields
                .monogram
                .trim()
                .to_uppercase()
                .chars()
                .take(2)
                .collect();
            let valid = !value.is_empty()
                && value.chars().all(char::is_alphanumeric)
                && !taken.contains(&value);
            if valid && value != entry.monogram {
                entry.monogram = value;
                changed = Some(false);
            }
        }
        y = name.bottom() + 12.0;

        y = self.label(ui, y, "Folder", "");
        let folder = Rect::from_min_size(pos2(self.left, y), vec2(form_w, size::FIELD_H));
        draw::outlined(ui, folder, radius::CONTROL, pal.well, pal.line);
        draw::label(
            ui,
            pos2(folder.left() + 9.0, folder.center().y),
            Align2::LEFT_CENTER,
            &strings::tilde(&entry.root),
            ty::CODE,
            pal.ink_2,
        );
        y = folder.bottom() + 12.0;

        let after = self.label(ui, y, strings::COVERAGE_REPORT, "clover.xml");
        draw::label(
            ui,
            pos2(self.left + form_w - 96.0, y),
            Align2::LEFT_TOP,
            "Coverage limit",
            ty::LABEL,
            pal.ink,
        );
        let report = Rect::from_min_size(
            pos2(self.left, after),
            vec2(form_w - 96.0 - 12.0, size::FIELD_H),
        );
        if draw::text_field(
            ui,
            pal,
            report,
            strings::COVERAGE_REPORT,
            &mut state.fields.report,
            true,
            self.keyboard,
        )
        .lost_focus()
        {
            let value = state.fields.report.trim();
            let value = (!value.is_empty()).then(|| value.to_owned());
            if value != entry.coverage_report {
                entry.coverage_report = value;
                changed = Some(true);
            }
        }
        if state.fields.report.is_empty() {
            // What Mortise does when the field is empty.
            let hint = "found in phpunit.xml or the usual places";
            draw::paint_text(
                ui,
                pos2(report.left() + 9.0, report.center().y),
                Align2::LEFT_CENTER,
                hint,
                ty::BODY,
                pal.ink_3,
            );
        }
        let limit = Rect::from_min_size(
            pos2(self.left + form_w - 96.0, after),
            vec2(96.0, size::FIELD_H),
        );
        if draw::text_field(
            ui,
            pal,
            limit,
            "Coverage limit",
            &mut state.fields.limit,
            false,
            self.keyboard,
        )
        .lost_focus()
        {
            let value = state
                .fields
                .limit
                .trim()
                .trim_end_matches('%')
                .trim()
                .parse::<u8>()
                .ok()
                .filter(|limit| *limit <= 100);
            if let Some(value) = value.filter(|value| *value != entry.coverage_limit) {
                entry.coverage_limit = value;
                changed = Some(true);
            }
        }
        y = report.bottom() + 16.0;

        // Removing an app: the folder and its files stay as they are.
        let label = format!("Remove {} from Mortise", entry.name);
        let remove = Button {
            label: &label,
            ..Default::default()
        };
        if remove
            .show(ui, pal, pos2(self.left, y), self.keyboard)
            .clicked()
        {
            *action = Some(SettingsAction::RemoveApp(index));
        }
        if let Some(reread) = changed {
            *action = Some(SettingsAction::Changed {
                reread: reread.then_some(index),
            });
        }
        y + size::CONTROL_H
    }

    // --- Keyboard (S8.4) ---

    fn keyboard(&self, ui: &mut Ui, mut y: f32, state: &mut SettingsState, mac: bool) -> f32 {
        let pal = self.pal;
        let field = Rect::from_min_size(
            pos2(self.left, y),
            vec2(240.0_f32.min(self.width * 0.4), size::CONTROL_H),
        );
        draw::text_field(
            ui,
            pal,
            field,
            "Find a shortcut",
            &mut state.shortcut_filter,
            false,
            self.keyboard,
        );
        if state.shortcut_filter.is_empty() {
            draw::paint_text(
                ui,
                pos2(field.left() + 9.0, field.center().y),
                Align2::LEFT_CENTER,
                "Find a shortcut",
                ty::BODY,
                pal.ink_3,
            );
        }
        let note = "On macOS Ctrl is Cmd. Shortcuts are fixed in this version.";
        draw::label(
            ui,
            pos2(field.right() + 12.0, field.center().y),
            Align2::LEFT_CENTER,
            note,
            ty::SECONDARY,
            pal.ink_2,
        );
        y = field.bottom() + 12.0;

        // On macOS the keys are written as they are pressed there.
        let keys_shown = |keys: &str| {
            if mac {
                keys.replace("Ctrl Alt", "Cmd Option")
                    .replace("Ctrl", "Cmd")
            } else {
                keys.to_owned()
            }
        };
        let wanted = state.shortcut_filter.trim().to_lowercase();
        let shown: Vec<_> = SHORTCUTS
            .iter()
            .filter(|(does, keys, place)| {
                wanted.is_empty()
                    || [does, keys, place]
                        .iter()
                        .any(|text| text.to_lowercase().contains(&wanted))
            })
            .collect();

        let table = Rect::from_min_size(
            pos2(self.left, y),
            vec2(
                self.width,
                size::TABLE_HEAD_H + shown.len() as f32 * size::TABLE_ROW_H + 1.0,
            ),
        );
        draw::outlined(ui, table, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
        let does_x = table.left() + 14.0;
        let keys_x = (table.left() + table.width() * 0.50).round();
        let place_x = (table.left() + table.width() * 0.78).round();
        let head_mid = table.top() + size::TABLE_HEAD_H / 2.0;
        for (x, label) in [(does_x, "Does"), (keys_x, "Keys"), (place_x, "Where")] {
            draw::label(
                ui,
                pos2(x, head_mid),
                Align2::LEFT_CENTER,
                label,
                ty::LABEL,
                pal.ink_2,
            );
        }
        draw::hline(
            ui,
            table.left() + 1.0..table.right() - 1.0,
            table.top() + size::TABLE_HEAD_H - 1.0,
            pal.line_2,
        );
        for (i, (does, keys, place)) in shown.iter().enumerate() {
            let top = table.top() + size::TABLE_HEAD_H + i as f32 * size::TABLE_ROW_H;
            let mid = top + size::TABLE_ROW_H / 2.0;
            draw::label(
                ui,
                pos2(does_x, mid),
                Align2::LEFT_CENTER,
                does,
                ty::BODY,
                pal.ink,
            );
            // A chord shares a cap; keys of a sequence get one cap each.
            let mut x = keys_x;
            for part in keys_shown(keys).split("  ") {
                // "or" stands between two ways to do the same thing.
                let part = match part.strip_prefix("or ") {
                    Some(rest) => {
                        x = draw::paint_text(
                            ui,
                            pos2(x, mid),
                            Align2::LEFT_CENTER,
                            "or",
                            ty::SECONDARY,
                            pal.ink_2,
                        )
                        .right()
                            + 6.0;
                        rest
                    }
                    None => part,
                };
                let cap = draw::keycap(ui, pal, pos2(x + draw::keycap_width(ui, part), mid), part);
                draw::announce(ui, cap, Role::Label, part);
                x = cap.right() + 6.0;
            }
            draw::label(
                ui,
                pos2(place_x, mid),
                Align2::LEFT_CENTER,
                place,
                ty::BODY,
                pal.ink_2,
            );
            if i + 1 < shown.len() {
                draw::hline(
                    ui,
                    table.left() + 1.0..table.right() - 1.0,
                    top + size::TABLE_ROW_H - 1.0,
                    pal.line,
                );
            }
        }
        let count = format!("{} of {} shown.", shown.len(), SHORTCUTS.len());
        draw::label(
            ui,
            pos2(self.left, table.bottom() + 10.0),
            Align2::LEFT_TOP,
            &count,
            ty::SECONDARY,
            pal.ink_2,
        )
        .bottom()
    }
}
