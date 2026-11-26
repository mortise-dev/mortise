//! Settings (S8.1 to S8.4): they take the place of the index and the workspace.
//!
//! There is no Save button. A choice applies at once, and a field applies when it
//! loses focus.

use eframe::egui::{
    Align2, Color32, IdSalt, Rect, Role, ScrollArea, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2,
    vec2,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

use mortise_core::{
    AppEntry, Credentials, Editor, General, IndexRows, Kept, Motion, PhpSettings, RunnerKind,
    Secrets, Settings, TestOutcome, ThemeChoice,
};

use crate::runner::php_settings::{PhpEvent, PhpForm, PhpOptions, PhpTest, php_field, propose};
use crate::runtime::environment_form::{EnvironmentForm, FormEvent, FormOptions, environment_form};
use crate::runtime::view::Brief;
use crate::settings::strings;
use crate::ui::draw::{self, Button, Icon};
use crate::ui::theme::{Face, Palette, Type, radius, size, ty};

/// The version Settings shows. The tests show a fixed one, so a release does not
/// change the images of Settings.
#[cfg(not(feature = "testing"))]
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(feature = "testing")]
const VERSION: &str = "1.0.1";

/// The form of a problem on GitHub. It opens with the version and the platform filled in,
/// through the query of the address, and nothing else: no path, no module name.
pub fn report_problem_url() -> String {
    let platform = format!("{} {}", platform_name(), std::env::consts::ARCH);
    format!(
        "https://github.com/mortise-dev/mortise/issues/new?template=problem.yml&version={}&platform={}",
        query_encoded(VERSION),
        query_encoded(&platform)
    )
}

fn platform_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macOS",
        "windows" => "Windows",
        "linux" => "Linux",
        other => other,
    }
}

/// Percent-encodes text for a query value.
fn query_encoded(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// The sections of Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    #[default]
    Apps,
    Php,
    General,
    Keyboard,
}

impl Section {
    pub const ALL: [Section; 4] = [
        Section::Apps,
        Section::Php,
        Section::General,
        Section::Keyboard,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Section::Apps => strings::SECTION_APPS,
            Section::Php => crate::strings::PHP,
            Section::General => strings::SECTION_GENERAL,
            Section::Keyboard => strings::SECTION_KEYBOARD,
        }
    }

    fn icon(self) -> Icon {
        match self {
            Section::Apps => Icon::App,
            Section::Php => Icon::Run,
            Section::General => Icon::Settings,
            Section::Keyboard => Icon::List,
        }
    }
}

/// What Settings keep while they are open.
#[derive(Default)]
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
    /// The environment of that app whose form is shown.
    pub environment: usize,
    /// A new environment is added to that app in the next frame.
    pub add_environment: bool,
    /// The form of that environment, and the app and place it was filled from.
    environment_form: Option<(usize, usize, EnvironmentForm)>,
    /// The PHP fields of each app shown in the PHP section, by the folder of the app.
    /// They stay while Settings are open. The test of each app lives with the app.
    php: BTreeMap<PathBuf, PhpFields>,
    /// The app whose PHP fields were drawn last, to take in what they hold when
    /// another app is selected.
    php_shown: Option<PathBuf>,
}

/// The fields of the PHP section for one app.
struct PhpFields {
    form: PhpForm,
    /// What detection found, shown while the app has no runner chosen.
    hint: &'static str,
    folder: String,
    minutes: String,
}

impl PhpFields {
    /// Takes in what each field holds, as when it loses focus. True when `php` changed.
    fn commit(&mut self, php: &mut PhpSettings) -> bool {
        let mut changed = false;
        if self.form.commit() {
            php.runner = Some(self.form.runner.clone());
            changed = true;
        }
        changed |= self.commit_folder(php);
        changed | self.commit_minutes(php)
    }

    /// Takes in the working folder. Empty is the app folder.
    fn commit_folder(&mut self, php: &mut PhpSettings) -> bool {
        let value = self.folder.trim();
        let value = (!value.is_empty()).then(|| PathBuf::from(value));
        let changed = value != php.working_folder;
        php.working_folder = value;
        self.folder = folder_text(php);
        changed
    }

    /// Takes in the time limit: a number of minutes from 1 to 120. Other text gives
    /// the limit back as it was.
    fn commit_minutes(&mut self, php: &mut PhpSettings) -> bool {
        let value = self
            .minutes
            .split_whitespace()
            .next()
            .and_then(|number| number.parse::<u16>().ok())
            .filter(|minutes| (1..=120).contains(minutes));
        let changed = value.is_some_and(|value| value != php.time_limit_minutes);
        if let Some(value) = value {
            php.time_limit_minutes = value;
        }
        self.minutes = crate::strings::minutes(php.time_limit_minutes);
        changed
    }
}

/// What Settings show about an app besides what they keep.
pub struct AppInfo<'a> {
    /// `Laravel 13.2, package 13.3.0`
    pub versions: String,
    /// The app has `nwidart/laravel-actuator`.
    pub actuator: bool,
    /// `APP_URL` of the app, to propose the base URL of a new environment.
    pub app_url: Option<String>,
    /// The last answer of each environment in this run, by its name.
    pub last: &'a mut BTreeMap<String, Brief>,
    /// The test of how PHP runs that runs, and the last one, in this run.
    pub php_test: &'a mut PhpTest,
    /// `module:migrate --all is running`, while the app runs a command.
    pub running: Option<String>,
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
    /// Open the form of a problem in the browser.
    ReportProblem,
}

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

    let version = strings::version(VERSION);
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
        strings::TITLE,
        ty::APP_NAME,
        pal.ink,
    );
    draw::announce(ui, title, Role::Heading, strings::TITLE);
    draw::label(
        ui,
        pos2(left, rect.top() + 33.0),
        Align2::LEFT_TOP,
        strings::SUBTITLE,
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
    let back = strings::back_to(back_to);
    let foot = Rect::from_min_max(
        pos2(rect.left(), foot_top + 1.0),
        pos2(rect.right() - 1.0, rect.bottom()),
    );
    let response = ui.interact(foot, ui.scope_id().with("settings-back"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, &back));
    if response.hovered() {
        ui.painter().rect_filled(foot, 0, pal.well);
    }
    let cap = draw::keycap(ui, pal, pos2(right, foot.center().y), strings::ESC_KEY);
    let text = draw::galley_elided(ui, &back, ty::SECONDARY, pal.ink_2, cap.left() - 8.0 - left);
    draw::paint_galley(ui, pos2(left, foot.center().y), Align2::LEFT_CENTER, text);
    response.clicked().then_some(SettingsAction::Close)
}

/// Paints the open section into the workspace. Returns what it asks for, in the
/// order to do it: a change comes before the action of the button that was clicked.
#[expect(clippy::too_many_arguments)]
pub fn section(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    state: &mut SettingsState,
    settings: &mut Settings,
    infos: &mut [AppInfo<'_>],
    secrets: &Secrets,
    custom_frame: bool,
    keyboard: bool,
    mac: bool,
) -> Vec<SettingsAction> {
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

    // A test of PHP that ended is the last test of its app, in whichever section is open.
    for info in infos.iter_mut() {
        info.php_test.poll();
    }

    let body = Rect::from_min_max(pos2(rect.left(), head.bottom()), rect.max);
    let mut actions = Vec::new();
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
                        &mut actions,
                    ),
                    Section::Apps => {
                        form.apps(ui, top, state, settings, infos, secrets, &mut actions)
                    }
                    Section::Php => form.php(ui, top, state, settings, infos, &mut actions),
                    Section::Keyboard => form.keyboard(ui, top, state, mac),
                };
                ui.allocate_rect(
                    Rect::from_min_max(origin, pos2(origin.x + width, bottom + 16.0)),
                    Sense::hover(),
                );
            });
    });
    actions
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
        actions: &mut Vec<SettingsAction>,
    ) -> f32 {
        let mut changed = false;

        let themes = [ThemeChoice::System, ThemeChoice::Light, ThemeChoice::Dark];
        let at = themes
            .iter()
            .position(|theme| *theme == general.theme)
            .unwrap_or(0);
        let hint = strings::THEME_HINT;
        let (next, clicked) = self.choice(ui, y, strings::THEME, hint, strings::THEME_CHOICES, at);
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
        let hint = strings::TEXT_SIZE_HINT;
        let (next, clicked) = self.choice(
            ui,
            y,
            strings::TEXT_SIZE,
            hint,
            strings::TEXT_SIZE_CHOICES,
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
        let hint = strings::MOTION_HINT;
        let (next, clicked) =
            self.choice(ui, y, strings::MOTION, hint, strings::MOTION_CHOICES, at);
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
        let hint = strings::ROWS_HINT;
        let (next, clicked) = self.choice(ui, y, strings::ROWS, hint, strings::ROWS_CHOICES, at);
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
            strings::OPEN_FILES_WITH,
            "",
            strings::EDITOR_CHOICES,
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
                strings::EDITOR_COMMAND,
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
            let hint = strings::CONTROLS_HINT;
            let at = usize::from(general.window_controls_left);
            let (next, clicked) = self.choice(
                ui,
                y,
                strings::CONTROLS,
                hint,
                strings::CONTROLS_CHOICES,
                at,
            );
            if let Some(clicked) = clicked {
                general.window_controls_left = clicked == 1;
                changed = true;
            }
            y = next;

            // Not in the drawing: the fallback of section 8, offered as a setting.
            let hint = strings::TITLE_BAR_HINT;
            let at = usize::from(general.system_title_bar);
            let (next, clicked) = self.choice(
                ui,
                y,
                strings::TITLE_BAR,
                hint,
                strings::TITLE_BAR_CHOICES,
                at,
            );
            if let Some(clicked) = clicked {
                general.system_title_bar = clicked == 1;
                changed = true;
            }
            y = next;
        }

        // Not in the drawing: a way to report a problem. It opens the form in the browser.
        // Mortise sends nothing by itself, and the address holds the version and the
        // platform only.
        let y = self.label(ui, y, strings::FEEDBACK, strings::FEEDBACK_HINT);
        let report = Button {
            label: strings::REPORT_A_PROBLEM,
            ..Default::default()
        };
        let response = report.show(ui, self.pal, pos2(self.left, y), self.keyboard);
        if response.clicked() {
            actions.push(SettingsAction::ReportProblem);
        }
        let y = y + size::CONTROL_H + 12.0;

        if changed {
            actions.push(SettingsAction::Changed { reread: None });
        }
        y
    }

    // --- Apps (S8.1) ---

    #[expect(clippy::too_many_arguments)]
    fn apps(
        &self,
        ui: &mut Ui,
        mut y: f32,
        state: &mut SettingsState,
        settings: &mut Settings,
        infos: &mut [AppInfo<'_>],
        secrets: &Secrets,
        actions: &mut Vec<SettingsAction>,
    ) -> f32 {
        let pal = self.pal;
        if settings.apps.is_empty() {
            return draw::paragraph(
                ui,
                pos2(self.left, y),
                crate::strings::NO_APPS_BODY,
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
        let versions_x = (table.left() + table.width() * 0.60).round();
        let environments_x = (table.left() + table.width() * 0.86).round();
        let head_mid = table.top() + size::TABLE_HEAD_H / 2.0;
        for (x, label) in [
            (name_x, strings::COL_APP),
            (folder_x, strings::COL_FOLDER),
            (versions_x, strings::COL_VERSIONS),
            (environments_x, strings::COL_ENVIRONMENTS),
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
        // The table is one stop: J and K move through the apps.
        let apps_nav =
            draw::ListNav::over(ui, "settings-apps", settings.apps.len(), &mut state.app);
        for (i, app) in settings.apps.iter().enumerate() {
            let top = table.top() + size::TABLE_HEAD_H + i as f32 * size::TABLE_ROW_H;
            let row = Rect::from_min_max(
                pos2(table.left() + 1.0, top),
                pos2(table.right() - 1.0, top + size::TABLE_ROW_H),
            );
            let selected = i == state.app;
            let response = ui.interact(row, apps_nav.id(i), apps_nav.sense(i));
            apps_nav.after(ui, i, &response);
            response.widget_info(|| {
                WidgetInfo::selected(Role::Row, true, selected, app.display_name())
            });
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
                app.display_name(),
                ty::BODY.with_face(Face::Sans600),
                pal.ink,
                folder_x - 12.0 - (tile.right() + 8.0),
            );
            draw::paint_galley(ui, pos2(tile.right() + 8.0, mid), Align2::LEFT_CENTER, name);
            let folder = crate::strings::tilde(&app.root);
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
            let version_text = infos.get(i).map_or("", |info| info.versions.as_str());
            let version = draw::galley_elided(
                ui,
                version_text,
                ty::BODY,
                pal.ink_2,
                environments_x - 12.0 - versions_x,
            );
            let version_rect =
                draw::paint_galley(ui, pos2(versions_x, mid), Align2::LEFT_CENTER, version);
            if !version_text.is_empty() {
                draw::announce(ui, version_rect, Role::Label, version_text);
            }
            // How many environments the app has.
            let count = match app.environments.len() {
                0 => crate::strings::NONE.to_owned(),
                n => n.to_string(),
            };
            let color = if app.environments.is_empty() {
                pal.ink_3
            } else {
                pal.ink
            };
            draw::paint_text(
                ui,
                pos2(environments_x, mid),
                Align2::LEFT_CENTER,
                &count,
                ty::BODY,
                color,
            );
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
                limit: strings::percent(entry.coverage_limit),
            };
            state.fields_of = Some((index, entry.clone()));
        }
        let heading = settings.apps[index].display_name().to_owned();
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
        let after = self.label(ui, y, crate::strings::NAME, "");
        draw::label(
            ui,
            pos2(self.left + form_w - 96.0, y),
            Align2::LEFT_TOP,
            crate::strings::MONOGRAM,
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
            crate::strings::NAME,
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
            // A value that is not taken goes back to the saved one.
            state.fields.name = entry.name.clone();
        }
        let monogram = Rect::from_min_size(
            pos2(self.left + form_w - 96.0, after),
            vec2(96.0, size::FIELD_H),
        );
        if draw::text_field(
            ui,
            pal,
            monogram,
            crate::strings::MONOGRAM,
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
            state.fields.monogram = entry.monogram.clone();
        }
        y = name.bottom() + 12.0;

        y = self.label(ui, y, strings::COL_FOLDER, "");
        let folder = Rect::from_min_size(pos2(self.left, y), vec2(form_w, size::FIELD_H));
        draw::outlined(ui, folder, radius::CONTROL, pal.well, pal.line);
        draw::label(
            ui,
            pos2(folder.left() + 9.0, folder.center().y),
            Align2::LEFT_CENTER,
            &crate::strings::tilde(&entry.root),
            ty::CODE,
            pal.ink_2,
        );
        y = folder.bottom() + 12.0;

        let after = self.label(ui, y, crate::strings::COVERAGE_REPORT, "clover.xml");
        draw::label(
            ui,
            pos2(self.left + form_w - 96.0, y),
            Align2::LEFT_TOP,
            strings::COVERAGE_LIMIT,
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
            crate::strings::COVERAGE_REPORT,
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
            state.fields.report = entry.coverage_report.clone().unwrap_or_default();
        }
        if state.fields.report.is_empty() {
            // What Mortise does when the field is empty.
            let hint = strings::COVERAGE_HINT;
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
            strings::COVERAGE_LIMIT,
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
            state.fields.limit = strings::percent(entry.coverage_limit);
        }
        y = report.bottom() + 20.0;

        if let Some(info) = infos.get_mut(index) {
            let (bottom, kept) = self.environments(ui, y, state, index, entry, info, secrets);
            y = bottom + 20.0;
            if kept {
                changed = changed.or(Some(false));
            }
        }

        // Removing an app: the folder and its files stay as they are. Not while it
        // runs a command, which would lose its stop and its history.
        let label = strings::remove_app(entry.display_name());
        let running = infos.get(index).and_then(|info| info.running.as_deref());
        let remove = Button {
            label: &label,
            disabled: running.is_some(),
            ..Default::default()
        };
        let response = remove.show(ui, pal, pos2(self.left, y), self.keyboard);
        let remove_clicked = response.clicked();
        if let Some(running) = running {
            draw::tooltip(response, pal, running);
        }
        // The click that asks to remove may also leave an edited field: that edit first.
        if let Some(reread) = changed {
            actions.push(SettingsAction::Changed {
                reread: reread.then_some(index),
            });
        }
        if remove_clicked {
            actions.push(SettingsAction::RemoveApp(index));
        }
        y + size::CONTROL_H
    }

    // --- The environments of an app (S8.1, S8.5) ---

    /// Paints the environments of the app at `index`: a table, and under it the form
    /// of the selected one. Returns the bottom edge, and true when the settings changed.
    #[expect(clippy::too_many_arguments)]
    fn environments(
        &self,
        ui: &mut Ui,
        mut y: f32,
        state: &mut SettingsState,
        index: usize,
        entry: &mut AppEntry,
        info: &mut AppInfo<'_>,
        secrets: &Secrets,
    ) -> (f32, bool) {
        let pal = self.pal;
        let mut changed = false;
        let heading = strings::environments_of(entry.display_name());
        let heading_rect = draw::paint_text(
            ui,
            pos2(self.left, y + 5.0),
            Align2::LEFT_TOP,
            &heading,
            ty::BODY.with_face(Face::Sans700),
            pal.ink,
        );
        draw::announce(ui, heading_rect, Role::Heading, &heading);
        if !info.actuator {
            // Without the package there is nothing to connect to.
            let line = heading_rect.bottom() + 6.0;
            draw::hline(ui, self.left..self.left + self.width, line, pal.line);
            let text = draw::paragraph(
                ui,
                pos2(self.left, line + 9.0),
                &crate::strings::actuator_not_installed(entry.display_name()),
                ty::BODY,
                pal.ink_2,
                self.width.min(560.0),
            );
            return (text.bottom(), false);
        }
        let add = Button {
            label: crate::strings::ADD_ENVIRONMENT,
            icon: Some(Icon::Plus),
            ..Default::default()
        };
        let add_at = pos2(self.left + self.width - add.width(ui), y);
        if add.show(ui, pal, add_at, self.keyboard).clicked() || state.add_environment {
            // A name no other environment of the app has.
            let taken = |name: &str| entry.environments.iter().any(|e| e.name == name);
            let name = ["local", "staging", "production"]
                .into_iter()
                .map(str::to_owned)
                .chain((2..).map(|n| format!("environment {n}")))
                .find(|name| !taken(name))
                .expect("a free name");
            let base_url = info
                .app_url
                .clone()
                .filter(|_| entry.environments.is_empty());
            let new = EnvironmentForm::new(&name, base_url.as_deref().unwrap_or_default());
            entry.environments.push(new.environment());
            if entry.environments.len() == 1 {
                entry.environment = Some(name);
            }
            state.environment = entry.environments.len() - 1;
            state.environment_form = None;
            changed = true;
        }
        state.add_environment = false;
        y += size::CONTROL_H + 6.0;
        draw::hline(ui, self.left..self.left + self.width, y, pal.line);
        y += 9.0;
        if entry.environments.is_empty() {
            let text = draw::paragraph(
                ui,
                pos2(self.left, y),
                &crate::strings::no_environment(entry.display_name()),
                ty::BODY,
                pal.ink_2,
                self.width.min(560.0),
            );
            return (text.bottom(), changed);
        }
        state.environment = state.environment.min(entry.environments.len() - 1);

        // The table: one row per environment, the selected one has its form below.
        let table = Rect::from_min_size(
            pos2(self.left, y),
            vec2(
                self.width,
                size::TABLE_HEAD_H + entry.environments.len() as f32 * size::TABLE_ROW_H + 1.0,
            ),
        );
        draw::outlined(ui, table, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
        let w = table.width();
        let wide = w >= 620.0;
        let name_x = table.left() + 14.0;
        let url_x = (table.left() + w * 0.16).round().max(name_x + 90.0);
        let path_x = (table.left() + w * 0.50).round();
        let credentials_x = (table.left() + w * 0.60).round();
        let answer_x = (table.left() + w * 0.84).round();
        let url_end = if wide { path_x } else { answer_x };
        let head_mid = table.top() + size::TABLE_HEAD_H / 2.0;
        let mut heads = vec![
            (name_x, strings::COL_ENVIRONMENT),
            (url_x, crate::strings::BASE_URL),
            (answer_x, strings::COL_LAST_ANSWER),
        ];
        if wide {
            heads.push((path_x, strings::COL_PATH));
            heads.push((credentials_x, crate::strings::CREDENTIALS));
        }
        for (x, label) in heads {
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
        let cell_code = Type {
            size: 11.5,
            ..ty::CODE
        };
        let count = entry.environments.len();
        let nav = draw::ListNav::over(
            ui,
            ("settings-environments", index),
            count,
            &mut state.environment,
        );
        for (i, environment) in entry.environments.iter().enumerate() {
            let top = table.top() + size::TABLE_HEAD_H + i as f32 * size::TABLE_ROW_H;
            let row = Rect::from_min_max(
                pos2(table.left() + 1.0, top),
                pos2(table.right() - 1.0, top + size::TABLE_ROW_H),
            );
            let selected = i == state.environment;
            let response = ui.interact(row, nav.id(i), nav.sense(i));
            nav.after(ui, i, &response);
            let said = format!("{} {}", strings::COL_ENVIRONMENT, environment.name);
            response.widget_info(|| WidgetInfo::selected(Role::Row, true, selected, &said));
            if response.clicked() {
                state.environment = i;
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
                if i + 1 < entry.environments.len() {
                    draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line);
                }
            }
            let mid = row.center().y;
            let cell = |ui: &Ui, x: f32, end: f32, text: &str, ty: Type, color: Color32| {
                let galley = draw::galley_elided(ui, text, ty, color, end - 10.0 - x);
                draw::paint_galley(ui, pos2(x, mid), Align2::LEFT_CENTER, galley)
            };
            let name_type = ty::BODY.with_face(Face::Sans600);
            cell(ui, name_x, url_x, &environment.name, name_type, pal.ink);
            cell(
                ui,
                url_x,
                url_end,
                &environment.base_url,
                cell_code,
                pal.ink,
            );
            if wide {
                cell(
                    ui,
                    path_x,
                    credentials_x,
                    &environment.path,
                    cell_code,
                    pal.ink,
                );
                let kind = match environment.credentials {
                    Credentials::None => 0,
                    Credentials::Bearer => 1,
                    Credentials::Basic { .. } => 2,
                    Credentials::Header { .. } => 3,
                };
                let saved = kind != 0 && secrets.has(&entry.root, &environment.name);
                let mut x = credentials_x;
                if saved {
                    let lock = Rect::from_min_size(pos2(x, mid - 6.0), Vec2::splat(12.0));
                    draw::icon(ui, lock, Icon::Lock, pal.ink_2);
                    x = lock.right() + 5.0;
                }
                let text =
                    strings::credentials_saved(crate::strings::CREDENTIAL_KINDS[kind], saved);
                let text_rect = cell(ui, x, answer_x, &text, ty::BODY, pal.ink);
                draw::announce(ui, text_rect, Role::Cell, &text);
            }
            // The last answer of this run: a runtime mark and its word.
            match info.last.get(&environment.name) {
                Some(brief) => {
                    let color = brief.tone.color(pal);
                    let mark = Rect::from_min_size(
                        pos2(answer_x, mid - 6.0),
                        Vec2::splat(size::STATE_MARK),
                    );
                    draw::runtime_mark(ui, mark, brief.mark, color);
                    let word_type = Type {
                        size: 11.5,
                        ..ty::LABEL
                    };
                    let at = pos2(mark.right() + 5.0, mid);
                    let word =
                        draw::paint_text(ui, at, Align2::LEFT_CENTER, brief.word, word_type, color);
                    let said = format!("{} {}", strings::COL_LAST_ANSWER, brief.word);
                    draw::announce(ui, word, Role::Cell, &said);
                }
                None => {
                    cell(
                        ui,
                        answer_x,
                        table.right(),
                        strings::NOT_ASKED,
                        ty::BODY,
                        pal.ink_3,
                    );
                }
            }
        }
        y = table.bottom() + 12.0;

        // The form of the selected environment. It follows the settings until it is edited.
        let place = state.environment;
        if state
            .environment_form
            .as_ref()
            .is_none_or(|(app, at, _)| *app != index || *at != place)
        {
            let environment = &entry.environments[place];
            let saved = environment.credentials.needs_secret()
                && secrets.has(&entry.root, &environment.name);
            let form = EnvironmentForm::of(environment, saved);
            state.environment_form = Some((index, place, form));
        }
        let Some((_, _, form)) = &mut state.environment_form else {
            return (y, changed);
        };
        let options = FormOptions {
            keyboard: self.keyboard,
            from_env: false,
            keep_typed: false,
            salt: "",
        };
        let form_w = self.width.min(720.0);
        let (bottom, events) = environment_form(ui, pal, pos2(self.left, y), form_w, form, options);
        y = bottom + 12.0;
        let before = entry.environments[place].clone();
        for event in events {
            match event {
                FormEvent::Changed => {
                    let mut now = form.environment();
                    // A name is what the environment is known by: not empty, and not
                    // that of another one.
                    let taken = entry
                        .environments
                        .iter()
                        .enumerate()
                        .any(|(i, other)| i != place && other.name == now.name);
                    if now.name.is_empty() || taken {
                        now.name = before.name.clone();
                        form.name = before.name.clone();
                    }
                    if now != before {
                        if now.name != before.name {
                            // The secret is kept under the name: it moves with it.
                            if let Some(secret) = secrets.get(&entry.root, &before.name) {
                                secrets.set(&entry.root, &now.name, &secret);
                                secrets.remove(&entry.root, &before.name);
                            }
                            if entry.environment.as_ref() == Some(&before.name) {
                                entry.environment = Some(now.name.clone());
                            }
                            if let Some(last) = info.last.remove(&before.name) {
                                info.last.insert(now.name.clone(), last);
                            }
                        }
                        entry.environments[place] = now;
                        changed = true;
                    }
                }
                FormEvent::Secret(secret) => {
                    let name = entry.environments[place].name.clone();
                    form.until_close = secrets.set(&entry.root, &name, &secret) == Kept::UntilClose;
                    changed = true;
                }
                FormEvent::RemoveSecret => {
                    secrets.remove(&entry.root, &entry.environments[place].name);
                    changed = true;
                }
                FormEvent::Test => {
                    let secret = secrets.get(&entry.root, &entry.environments[place].name);
                    form.start_test(secret, ui.ctx());
                }
            }
        }
        // The answer of a test is the last answer of the environment.
        if let Some(answer) = form.tested() {
            let name = entry.environments[place].name.clone();
            info.last.insert(name, Brief::of_answer(answer));
        }

        let remove = Button {
            label: strings::REMOVE_ENVIRONMENT,
            small: true,
            ..Default::default()
        };
        if remove
            .show(ui, pal, pos2(self.left, y), self.keyboard)
            .clicked()
        {
            let removed = entry.environments.remove(place);
            secrets.remove(&entry.root, &removed.name);
            info.last.remove(&removed.name);
            if entry.environment.as_ref() == Some(&removed.name) {
                entry.environment = entry.environments.first().map(|e| e.name.clone());
            }
            state.environment = 0;
            state.environment_form = None;
            changed = true;
        }
        (y + size::CONTROL_SM_H, changed)
    }

    // --- PHP (S8.2) ---

    fn php(
        &self,
        ui: &mut Ui,
        mut y: f32,
        state: &mut SettingsState,
        settings: &mut Settings,
        infos: &mut [AppInfo<'_>],
        actions: &mut Vec<SettingsAction>,
    ) -> f32 {
        let pal = self.pal;
        if settings.apps.is_empty() {
            return draw::paragraph(
                ui,
                pos2(self.left, y),
                crate::strings::NO_APPS_BODY,
                ty::BODY,
                pal.ink_2,
                self.width.min(560.0),
            )
            .bottom();
        }
        state.app = state.app.min(settings.apps.len() - 1);

        // The apps: how each runs PHP and what its last test said.
        let table = Rect::from_min_size(
            pos2(self.left, y),
            vec2(
                self.width,
                size::TABLE_HEAD_H + settings.apps.len() as f32 * size::TABLE_ROW_H + 1.0,
            ),
        );
        draw::outlined(ui, table, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
        let w = table.width();
        let last_w = (w * 0.3).min(270.0).round();
        let runs_w = 100.0;
        let rest = w - runs_w - last_w;
        let name_x = table.left() + 14.0;
        let runs_x = (table.left() + rest / 2.4).round();
        let command_x = runs_x + runs_w;
        let last_x = table.right() - last_w;
        let head_mid = table.top() + size::TABLE_HEAD_H / 2.0;
        for (x, label) in [
            (name_x, strings::COL_APP),
            (runs_x, strings::COL_RUNS_WITH),
            (command_x, strings::COL_COMMAND),
            (last_x, strings::COL_LAST_TEST),
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
        let cell_code = Type {
            size: 11.5,
            ..ty::CODE
        };
        let nav = draw::ListNav::over(ui, "settings-php-apps", settings.apps.len(), &mut state.app);
        for (i, app) in settings.apps.iter().enumerate() {
            let top = table.top() + size::TABLE_HEAD_H + i as f32 * size::TABLE_ROW_H;
            let row = Rect::from_min_max(
                pos2(table.left() + 1.0, top),
                pos2(table.right() - 1.0, top + size::TABLE_ROW_H),
            );
            let selected = i == state.app;
            let response = ui.interact(row, nav.id(i), nav.sense(i));
            nav.after(ui, i, &response);
            response.widget_info(|| {
                WidgetInfo::selected(Role::Row, true, selected, app.display_name())
            });
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
            let cell = |ui: &Ui, x: f32, end: f32, text: &str, ty: Type, color: Color32| {
                let galley = draw::galley_elided(ui, text, ty, color, end - 10.0 - x);
                draw::paint_galley(ui, pos2(x, mid), Align2::LEFT_CENTER, galley)
            };
            let name_type = ty::BODY.with_face(Face::Sans600);
            cell(ui, name_x, runs_x, app.display_name(), name_type, pal.ink);
            match &app.php.runner {
                Some(runner) => {
                    let kind = crate::strings::RUNNERS[crate::strings::runner_place(runner)];
                    cell(ui, runs_x, command_x, kind, ty::BODY, pal.ink);
                    let line = runner.line();
                    let text = cell(ui, command_x, last_x, &line, cell_code, pal.ink);
                    draw::announce(ui, text, Role::Cell, &line);
                }
                None => {
                    cell(
                        ui,
                        runs_x,
                        command_x,
                        strings::NOT_CHOSEN,
                        ty::BODY,
                        pal.ink_3,
                    );
                }
            }
            // The last test of this run: a state mark and what it said.
            let tested = infos.get(i).and_then(|info| info.php_test.last());
            let mark = Rect::from_min_size(
                pos2(last_x, mid - size::STATE_MARK / 2.0),
                Vec2::splat(size::STATE_MARK),
            );
            let (text, color) = match tested.map(|tested| &tested.outcome) {
                Some(TestOutcome::Answered(answer)) => {
                    draw::mark_good(ui, mark, pal.good);
                    (answer.clone(), pal.ink)
                }
                Some(TestOutcome::Failed(failure)) => {
                    let behind = if selected || response.hovered() {
                        pal.well
                    } else {
                        pal.surface
                    };
                    draw::mark_bad(ui, mark, pal.bad, behind);
                    (strings::no_answer(failure), pal.ink)
                }
                None => {
                    let text = cell(
                        ui,
                        last_x,
                        table.right(),
                        strings::NOT_TESTED,
                        ty::BODY,
                        pal.ink_3,
                    );
                    draw::announce(ui, text, Role::Cell, strings::NOT_TESTED);
                    continue;
                }
            };
            let text_rect = cell(
                ui,
                mark.right() + 6.0,
                table.right(),
                &text,
                ty::BODY,
                color,
            );
            draw::announce(ui, text_rect, Role::Cell, &text);
        }
        y = table.bottom() + 16.0;

        // The selected app: its field, where commands start, and how long they may run.
        let index = state.app;
        let heading = settings.apps[index].display_name().to_owned();
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
        // The box of what each choice runs stands at the right where there is room.
        let side = self.width >= 820.0;
        let form_w = if side {
            ((self.width - 24.0) * 0.58).min(560.0).round()
        } else {
            self.width.min(560.0)
        };
        draw::hline(ui, self.left..self.left + form_w, y, pal.line);
        y += 9.0;
        let detail_top = y;

        // Another app is selected: what the fields of the one before hold is taken
        // in, as when they lose focus. Its fields are not drawn again to lose it.
        let mut changed = false;
        let root = settings.apps[index].root.clone();
        if let Some(before) = state.php_shown.replace(root.clone())
            && before != root
            && let Some(fields) = state.php.get_mut(&before)
            && let Some(app) = settings.apps.iter_mut().find(|app| app.root == before)
        {
            changed |= fields.commit(&mut app.php);
        }

        let entry = &mut settings.apps[index];
        // An app with no runner shows what detection proposes, and saves it only
        // when the user changes it or tests it.
        let fields = state.php.entry(root).or_insert_with(|| {
            let (runner, hint) = match &entry.php.runner {
                Some(runner) => (runner.clone(), ""),
                None => {
                    let proposal = propose(&entry.root);
                    (proposal.runner, proposal.hint)
                }
            };
            PhpFields {
                form: PhpForm::new(runner),
                hint,
                folder: folder_text(&entry.php),
                minutes: crate::strings::minutes(entry.php.time_limit_minutes),
            }
        });
        let mut spare = PhpTest::default();
        let test = match infos.get_mut(index) {
            Some(info) => &mut *info.php_test,
            None => &mut spare,
        };
        // The widgets of each app have their own ids, so focus does not pass from
        // the field of one app to the same field of another.
        let salt = IdSalt::new(("settings-php", &entry.root));
        let folder_now = entry.php.folder(&entry.root);
        let options = PhpOptions {
            keyboard: self.keyboard,
            hint: if entry.php.runner.is_none() {
                fields.hint
            } else {
                ""
            },
            folder: &folder_now,
            salt,
        };
        let (bottom, events) = php_field(
            ui,
            pal,
            pos2(self.left, y),
            form_w,
            &mut fields.form,
            test,
            options,
        );
        // A change or a test saves the runner, also the one that detection proposed.
        let runner = Some(fields.form.runner.clone());
        if !events.is_empty() && entry.php.runner != runner {
            entry.php.runner = runner;
            changed = true;
        }
        if events.contains(&PhpEvent::Test) {
            // The working folder may still have focus: what it holds counts.
            changed |= fields.commit_folder(&mut entry.php);
            test.start(&fields.form.runner, entry.php.folder(&entry.root), ui.ctx());
        }
        y = bottom + 12.0;

        // Where the command starts. Empty is the app folder.
        y = self.label(ui, y, strings::WORKING_FOLDER, strings::WORKING_FOLDER_HINT);
        let folder = Rect::from_min_size(pos2(self.left, y), vec2(form_w, size::FIELD_H));
        let left_folder = ui
            .push_id(salt, |ui| {
                draw::text_field(
                    ui,
                    pal,
                    folder,
                    strings::WORKING_FOLDER,
                    &mut fields.folder,
                    true,
                    self.keyboard,
                )
            })
            .inner
            .lost_focus();
        if left_folder {
            changed |= fields.commit_folder(&mut entry.php);
        }
        if fields.folder.is_empty() {
            draw::paint_text(
                ui,
                pos2(folder.left() + 9.0, folder.center().y),
                Align2::LEFT_CENTER,
                strings::THE_APP_FOLDER,
                ty::BODY,
                pal.ink_3,
            );
        }
        y = folder.bottom() + 12.0;

        // How long a command may run, in minutes: a number from 1 to 120.
        y = self.label(ui, y, strings::STOP_AFTER, strings::STOP_AFTER_HINT);
        let limit = Rect::from_min_size(pos2(self.left, y), vec2(160.0, size::FIELD_H));
        let left_limit = ui
            .push_id(salt, |ui| {
                draw::text_field(
                    ui,
                    pal,
                    limit,
                    strings::STOP_AFTER,
                    &mut fields.minutes,
                    false,
                    self.keyboard,
                )
            })
            .inner
            .lost_focus();
        if left_limit {
            changed |= fields.commit_minutes(&mut entry.php);
        }
        y = limit.bottom();

        // What each choice runs: the words before the artisan arguments.
        let lines = [
            (crate::strings::RUNNERS[0], RunnerKind::Herd.line()),
            (crate::strings::RUNNERS[1], RunnerKind::Sail.line()),
            (
                crate::strings::RUNNERS[2],
                RunnerKind::Docker {
                    service: "<service>".to_owned(),
                }
                .line(),
            ),
        ];
        let box_min = if side {
            pos2(self.left + form_w + 24.0, detail_top)
        } else {
            pos2(self.left, y + 20.0)
        };
        let box_w = if side {
            (self.width - form_w - 24.0).min(440.0)
        } else {
            form_w
        };
        let head = draw::box_head_extra(
            ui,
            box_w,
            strings::WHAT_EACH_RUNS,
            strings::WHAT_EACH_RUNS_NOTE,
        );
        let choices = Rect::from_min_size(
            box_min,
            vec2(box_w, 41.0 + head + 4.0 * size::LIST_LINE_H + 8.0),
        );
        let mut line_y = draw::box_head(
            ui,
            pal,
            choices,
            strings::WHAT_EACH_RUNS,
            strings::WHAT_EACH_RUNS_NOTE,
        );
        for (name, words) in &lines {
            let value = [(words.as_str(), cell_code, pal.ink)];
            line_y = draw::box_line(ui, pal, choices, line_y, (name, ty::BODY), &value, false);
        }
        let custom = [(strings::CUSTOM_RUNS, cell_code, pal.ink_3)];
        draw::box_line(
            ui,
            pal,
            choices,
            line_y,
            (crate::strings::RUNNERS[3], ty::BODY),
            &custom,
            true,
        );

        if changed {
            actions.push(SettingsAction::Changed { reread: None });
        }
        y.max(choices.bottom())
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
            strings::FIND_SHORTCUT,
            &mut state.shortcut_filter,
            false,
            self.keyboard,
        );
        if state.shortcut_filter.is_empty() {
            draw::paint_text(
                ui,
                pos2(field.left() + 9.0, field.center().y),
                Align2::LEFT_CENTER,
                strings::FIND_SHORTCUT,
                ty::BODY,
                pal.ink_3,
            );
        }
        let note = strings::SHORTCUTS_NOTE;
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
                // The drawer key stays Ctrl `, and its second key is Cmd J.
                keys.replace("Ctrl Alt", "Cmd Option")
                    .replace("Ctrl `", "\u{0}")
                    .replace("Ctrl", "Cmd")
                    .replace('\u{0}', "Ctrl `")
            } else {
                keys.to_owned()
            }
        };
        let wanted = state.shortcut_filter.trim().to_lowercase();
        let shown: Vec<_> = strings::SHORTCUTS
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
        for (x, label) in [
            (does_x, strings::COL_DOES),
            (keys_x, strings::COL_KEYS),
            (place_x, strings::COL_WHERE),
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
                            strings::KEYS_OR,
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
        let count = strings::shown_of(shown.len(), strings::SHORTCUTS.len());
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

/// The text of the working folder field: empty for the app folder.
fn folder_text(php: &PhpSettings) -> String {
    php.working_folder
        .as_ref()
        .map(|folder| folder.display().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_address_holds_the_version_and_the_platform_only() {
        let url = report_problem_url();
        let (base, query) = url.split_once('?').unwrap();
        assert_eq!(base, "https://github.com/mortise-dev/mortise/issues/new");
        let keys: Vec<_> = query
            .split('&')
            .map(|pair| pair.split_once('=').unwrap().0)
            .collect();
        assert_eq!(keys, ["template", "version", "platform"]);
        assert!(query.contains(&format!("platform={}%20", platform_name())));
    }
}
