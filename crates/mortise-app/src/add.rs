//! Adding an app (S1.2 to S1.4): choose the folder, see what was found, add it.
//!
//! The panel takes the place of the index and the workspace until the app is
//! added or Esc is pressed.

use eframe::egui::{
    self, Align2, Color32, FontId, Key, KeyboardShortcut, Modifiers, Rect, RichText, Role, Ui,
    Vec2, pos2, vec2,
};
use mortise_core::{AppSnapshot, ReadError};

use crate::draw::{self, Button, ButtonKind, Icon};
use crate::environment_form::{EnvironmentForm, FormEvent, FormOptions, environment_form};
use crate::strings;
use crate::theme::{Face, Palette, Type, radius, size, ty};

/// What checking a folder gave.
pub enum Checked {
    Found(Box<AppSnapshot>),
    Failed(ReadError),
    /// The folder is an app Mortise has already: its place on the rail and its name.
    Known {
        index: usize,
        name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Field,
    Primary,
}

pub struct AddPanel {
    /// The text of the path field.
    pub path: String,
    pub checked: Option<Checked>,
    /// The path that `checked` belongs to.
    checked_path: String,
    /// Where focus goes in the next frame.
    focus: Option<Focus>,
    /// The runtime connection of the app that was found, when it has the actuator.
    pub runtime: Option<EnvironmentForm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddAction {
    /// Close the panel.
    Back,
    /// Open the folder dialog of the operating system.
    Pick,
    /// Read the folder in the path field.
    Check,
    /// Add the app that was found.
    Add,
    /// Open the app that is already in Mortise.
    Open(usize),
    /// Copy the command that installs the package.
    CopyCommand,
    /// Ask the environment of the runtime section now.
    Test,
}

impl AddPanel {
    /// The panel before a folder is chosen. Focus is in the path field.
    pub fn empty() -> Self {
        Self {
            path: String::new(),
            checked: None,
            checked_path: String::new(),
            focus: Some(Focus::Field),
            runtime: None,
        }
    }

    /// The panel after a folder was chosen. Focus is on the button that adds or opens.
    pub fn checked(path: String, checked: Checked) -> Self {
        Self {
            checked_path: path.clone(),
            path,
            runtime: runtime_form(&checked),
            checked: Some(checked),
            focus: Some(Focus::Primary),
        }
    }

    pub fn set_checked(&mut self, checked: Checked) {
        self.checked_path = self.path.clone();
        self.focus = match checked {
            Checked::Failed(_) => None,
            _ => Some(Focus::Primary),
        };
        self.runtime = runtime_form(&checked);
        self.checked = Some(checked);
    }
}

/// The runtime section for a folder that was read: only an app with
/// `nwidart/laravel-actuator` has one. The base URL is proposed from `APP_URL`.
fn runtime_form(checked: &Checked) -> Option<EnvironmentForm> {
    match checked {
        Checked::Found(app) if app.actuator.is_some() => Some(EnvironmentForm::new(
            "local",
            app.app_url.as_deref().unwrap_or_default(),
        )),
        _ => None,
    }
}

/// Paints the panel into `rect`. `monogram` is what the app would get on the rail.
pub fn add_panel(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    panel: &mut AddPanel,
    monogram: &str,
    keyboard: bool,
    mac: bool,
) -> Option<AddAction> {
    let mut action = None;
    ui.painter().rect_filled(rect, 0, pal.surface);

    // Header strip: the title, and the way back.
    let head = Rect::from_min_size(rect.min, vec2(rect.width(), 46.0));
    ui.painter().rect_filled(head, 0, pal.panel);
    draw::hline(
        ui,
        head.left()..head.right(),
        head.bottom() - 1.0,
        pal.line_2,
    );
    let title = draw::paint_text(
        ui,
        pos2(head.left() + 16.0, head.top() + 12.0),
        Align2::LEFT_TOP,
        strings::ADD_APP,
        ty::TITLE,
        pal.ink,
    );
    draw::announce(ui, title, Role::Heading, strings::ADD_APP);
    let back = Button {
        label: strings::BACK,
        kind: ButtonKind::Ghost,
        small: true,
        keys: Some("Esc"),
        ..Default::default()
    };
    let back_min = pos2(
        head.right() - 16.0 - back.width(ui),
        head.center().y - back.height() / 2.0,
    );
    if back.show(ui, pal, back_min, keyboard).clicked() {
        action = Some(AddAction::Back);
    }

    let left = rect.left() + 16.0;
    let column_w = 560.0_f32.min(rect.width() - 32.0);
    let mut y = head.bottom() + 14.0;

    // The label of the field, with a hint until a folder is chosen.
    let label = draw::label(
        ui,
        pos2(left, y),
        Align2::LEFT_TOP,
        strings::APP_FOLDER,
        ty::LABEL,
        pal.ink,
    );
    if panel.checked.is_none() {
        draw::label(
            ui,
            pos2(label.right() + 4.0, y),
            Align2::LEFT_TOP,
            strings::APP_FOLDER_HINT,
            ty::SECONDARY,
            pal.ink_2,
        );
    }
    y += ty::LABEL.line + 4.0;

    // The path field. Paths are set in Inconsolata.
    let field = Rect::from_min_size(pos2(left, y), vec2(column_w, size::FIELD_H));
    let field_id = ui.scope_id().with("add-path");
    let focused = ui.memory(|memory| memory.has_focus(field_id));
    let failed = matches!(panel.checked, Some(Checked::Failed(_)));
    let hovered = ui.rect_contains_pointer(field);
    let outline = if failed {
        pal.bad
    } else if focused {
        pal.ink
    } else if hovered {
        pal.ink_2
    } else {
        pal.line_2
    };
    draw::outlined(ui, field, radius::CONTROL, pal.surface, outline);

    // Once a folder is chosen, the button that chooses another sits inside the field.
    let inside = Button {
        label: strings::CHOOSE_FOLDER,
        small: true,
        ..Default::default()
    };
    let inside_w = if panel.checked.is_some() {
        inside.width(ui) + 3.0
    } else {
        0.0
    };
    let text_rect = Rect::from_min_max(
        pos2(field.left() + 9.0, field.top() + 1.0),
        pos2(field.right() - 9.0 - inside_w, field.bottom() - 1.0),
    );
    let hint = RichText::new(strings::PATH_PLACEHOLDER)
        .color(pal.ink_3)
        .font(FontId::new(ty::BODY.size, Face::Sans400.family()));
    let edit = egui::TextEdit::singleline(&mut panel.path)
        .id(field_id)
        .frame(egui::Frame::NONE)
        .margin(egui::Margin::ZERO)
        .font(FontId::new(ty::CODE.size, ty::CODE.face.family()))
        .text_color(pal.ink)
        .desired_width(text_rect.width())
        .align(Align2::LEFT_CENTER)
        .hint_text(hint);
    let response = ui.put(text_rect, edit);
    response.widget_info(|| egui::WidgetInfo::labeled(Role::TextInput, true, strings::APP_FOLDER));
    if panel.focus == Some(Focus::Field) {
        response.request_focus();
        panel.focus = None;
    }
    if keyboard && focused {
        draw::focus_ring(ui, pal, field, radius::CONTROL);
    }
    // Enter in the field, or leaving it after a change, reads the folder.
    let changed = panel.path.trim() != panel.checked_path.trim() && !panel.path.trim().is_empty();
    if response.lost_focus() && (changed || ui.input(|input| input.key_pressed(Key::Enter))) {
        action = Some(AddAction::Check);
    }
    if panel.checked.is_some() {
        let min = pos2(
            field.right() - 3.0 - inside.width(ui),
            field.center().y - inside.height() / 2.0,
        );
        if inside.show(ui, pal, min, keyboard).clicked() {
            action = Some(AddAction::Pick);
        }
    }
    y = field.bottom();

    let right_column = Rect::from_min_max(
        pos2(
            (left + column_w + 45.0).min(rect.right() - 16.0),
            head.bottom() + 14.0,
        ),
        pos2(rect.right() - 16.0, rect.bottom()),
    );
    let command_enter = format!("{} Enter", strings::command_key(mac));

    // What is under the field depends on what the folder gave.
    let (primary, others): (Button<'_>, &[&str]);
    let primary_label;
    let mut primary_action = AddAction::Add;
    match &panel.checked {
        None => {
            y += 12.0;
            let choose = Button {
                label: strings::CHOOSE_FOLDER,
                kind: ButtonKind::Primary,
                icon: Some(Icon::Folder),
                ..Default::default()
            };
            if choose.show(ui, pal, pos2(left, y), keyboard).clicked() {
                action = Some(AddAction::Pick);
            }
            draw::label(
                ui,
                pos2(left + choose.width(ui) + 8.0, y + choose.height() / 2.0),
                Align2::LEFT_CENTER,
                strings::CHOOSE_FOLDER_HINT,
                ty::SECONDARY,
                pal.ink_2,
            );
            if right_column.width() > 200.0 {
                note(
                    ui,
                    pal,
                    right_column.min,
                    right_column.width(),
                    strings::READS_NOTE,
                );
            }
            return action;
        }
        Some(Checked::Found(app)) => {
            primary_label = strings::add_named(&app.name);
            primary = Button {
                label: &primary_label,
                kind: ButtonKind::Primary,
                keys: Some(&command_enter),
                ..Default::default()
            };
            others = &[strings::CANCEL];
            y += 16.0;
            if right_column.width() > 200.0 {
                found(ui, pal, right_column, app, monogram);
            }
            // Runtime, optional: the app is added without it when the URL is empty.
            if let Some(form) = &mut panel.runtime {
                let heading = draw::paint_text(
                    ui,
                    pos2(left, y),
                    Align2::LEFT_TOP,
                    strings::RUNTIME_OPTIONAL,
                    ty::BODY.with_face(Face::Sans700),
                    pal.ink,
                );
                draw::announce(ui, heading, Role::Heading, strings::RUNTIME_OPTIONAL);
                y = heading.bottom() + 6.0;
                draw::hline(ui, left..left + column_w, y, pal.line);
                y += 9.0;
                let options = FormOptions {
                    keyboard,
                    from_env: app.app_url.is_some(),
                    keep_typed: true,
                    salt: "",
                };
                let (bottom, events) =
                    environment_form(ui, pal, pos2(left, y), column_w, form, options);
                if events.contains(&FormEvent::Test) {
                    action = Some(AddAction::Test);
                }
                y = bottom + 16.0;
            }
        }
        Some(Checked::Known { index, name }) => {
            y += 4.0;
            let sentence = strings::already_added(name);
            let text = draw::paragraph(
                ui,
                pos2(left, y),
                &sentence,
                ty::SECONDARY,
                pal.ink_2,
                column_w,
            );
            y = text.bottom() + 12.0;
            primary_label = strings::open_named(name);
            primary = Button {
                label: &primary_label,
                kind: ButtonKind::Primary,
                ..Default::default()
            };
            primary_action = AddAction::Open(*index);
            others = &[strings::CANCEL];
        }
        Some(Checked::Failed(error)) => {
            // The sentence at the field: what is wrong, then what to do.
            y += 4.0;
            let mark = Rect::from_min_size(pos2(left, y + 2.0), Vec2::splat(size::STATE_MARK));
            draw::mark_bad(ui, mark, pal.bad, pal.surface);
            let sentence = strings::read_error(error);
            let sentence_type = Type {
                line: 16.0,
                ..ty::SECONDARY
            };
            let text = draw::paragraph(
                ui,
                pos2(mark.right() + 6.0, y),
                &sentence,
                sentence_type,
                pal.bad,
                column_w - size::STATE_MARK - 6.0,
            );
            y = text.bottom() + 12.0;

            if matches!(error, ReadError::NoPackage { .. }) {
                // The command that installs the package, to copy.
                let block = Rect::from_min_size(pos2(left, y), vec2(column_w, 39.0));
                ui.painter()
                    .rect_filled(block, radius::CONTROL, pal.code_bg);
                draw::label(
                    ui,
                    pos2(block.left() + 12.0, block.center().y),
                    Align2::LEFT_CENTER,
                    strings::REQUIRE_PACKAGE,
                    ty::CODE,
                    pal.code_fg,
                );
                y = block.bottom() + 10.0;
                let copy = Button {
                    label: strings::COPY_COMMAND,
                    ..Default::default()
                };
                if copy.show(ui, pal, pos2(left, y), keyboard).clicked() {
                    action = Some(AddAction::CopyCommand);
                }
                let again = Button {
                    label: strings::CHECK_AGAIN,
                    kind: ButtonKind::Primary,
                    ..Default::default()
                };
                let response = again.show(ui, pal, pos2(left + copy.width(ui) + 8.0, y), keyboard);
                if response.clicked() {
                    action = Some(AddAction::Check);
                }
                return action;
            }
            // The add button is disabled until the folder changes.
            primary = Button {
                label: strings::ADD_APP,
                kind: ButtonKind::Primary,
                disabled: true,
                ..Default::default()
            };
            others = match error {
                ReadError::NotLaravel { .. } => &[strings::CANCEL],
                _ => &[strings::CANCEL, strings::CHECK_AGAIN],
            };
        }
    }

    // The buttons: the one that adds or opens, then the quiet ones.
    let response = primary.show(ui, pal, pos2(left, y), keyboard);
    if panel.focus == Some(Focus::Primary) {
        response.request_focus();
        panel.focus = None;
    }
    let shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Enter);
    let by_key = !primary.disabled
        && matches!(panel.checked, Some(Checked::Found(_)))
        && ui.input_mut(|input| input.consume_shortcut(&shortcut));
    if response.clicked() || by_key {
        action = Some(primary_action);
    }
    let mut x = left + primary.width(ui) + 8.0;
    for label in others {
        let button = Button {
            label,
            ..Default::default()
        };
        if button.show(ui, pal, pos2(x, y), keyboard).clicked() {
            action = Some(if *label == strings::CANCEL {
                AddAction::Back
            } else {
                AddAction::Check
            });
        }
        x += button.width(ui) + 8.0;
    }
    action
}

/// The two boxes at the right once a folder is read: what was found, and how the
/// app will be shown on the rail.
fn found(ui: &Ui, pal: &Palette, column: Rect, app: &AppSnapshot, monogram: &str) {
    let package = Type {
        size: 11.5,
        ..ty::CODE
    };
    let body = (ty::BODY, pal.ink);
    let dim = pal.ink_2;

    let found = Rect::from_min_size(
        column.min,
        vec2(column.width(), 41.0 + 6.0 * size::LIST_LINE_H + 8.0),
    );
    let mut y = draw::box_head(ui, pal, found, strings::FOUND_TITLE, strings::FOUND_NOTE);

    let laravel = app
        .laravel
        .as_ref()
        .map(|version| version.short().to_owned());
    let laravel: Vec<(&str, Type, Color32)> = match &laravel {
        Some(version) => vec![(version, body.0, body.1), (" composer.lock", ty::BODY, dim)],
        None => vec![(strings::NOT_INSTALLED, ty::BODY, dim)],
    };
    y = draw::box_line(
        ui,
        pal,
        found,
        y,
        (strings::LARAVEL, ty::BODY),
        &laravel,
        false,
    );
    let version = [(app.package.full(), body.0, body.1)];
    y = draw::box_line(
        ui,
        pal,
        found,
        y,
        ("nwidart/laravel-modules", package),
        &version,
        false,
    );
    let (count, folder, on) = strings::modules_found(app);
    let modules = [
        (count.as_str(), body.0, body.1),
        (folder.as_str(), package, body.1),
        (on.as_str(), body.0, body.1),
    ];
    let modules: Vec<_> = modules
        .into_iter()
        .filter(|(text, ..)| !text.is_empty())
        .collect();
    y = draw::box_line(
        ui,
        pal,
        found,
        y,
        (strings::MODULES, ty::BODY),
        &modules,
        false,
    );
    let actuator = match &app.actuator {
        Some(version) => [(version.full(), body.0, body.1)],
        None => [(strings::NOT_INSTALLED, ty::BODY, dim)],
    };
    y = draw::box_line(
        ui,
        pal,
        found,
        y,
        ("nwidart/laravel-actuator", package),
        &actuator,
        false,
    );
    let report = app
        .coverage_report
        .as_ref()
        .map(|report| app.path_shown(&report.path));
    let report = match &report {
        Some(path) => [(path.as_str(), package, body.1)],
        None => [(strings::NO_REPORT_FOUND, ty::BODY, dim)],
    };
    y = draw::box_line(
        ui,
        pal,
        found,
        y,
        (strings::COVERAGE_REPORT, ty::BODY),
        &report,
        false,
    );
    let (words, branch) = strings::git_found(&app.git);
    let in_repository = matches!(app.git, mortise_core::Git::Repository { .. });
    let git = [
        (
            words.as_str(),
            ty::BODY,
            if in_repository { body.1 } else { dim },
        ),
        (branch.as_str(), package, body.1),
    ];
    let git: Vec<_> = git
        .into_iter()
        .filter(|(text, ..)| !text.is_empty())
        .collect();
    draw::box_line(ui, pal, found, y, (strings::GIT, ty::BODY), &git, true);

    let shown = Rect::from_min_size(
        pos2(column.left(), found.bottom() + 12.0),
        vec2(column.width(), 41.0 + size::LIST_LINE_H + 34.0 + 8.0),
    );
    let y = draw::box_head(ui, pal, shown, strings::SHOWN_TITLE, strings::SHOWN_NOTE);
    let name = [(app.name.as_str(), body.0, body.1)];
    let y = draw::box_line(ui, pal, shown, y, (strings::NAME, ty::BODY), &name, false);
    let mid = y + 17.0;
    draw::label(
        ui,
        pos2(shown.left() + 13.0, mid),
        Align2::LEFT_CENTER,
        strings::MONOGRAM,
        ty::BODY,
        pal.ink,
    );
    let tile = Rect::from_min_size(
        pos2(shown.right() - 13.0 - 28.0, mid - 14.0),
        Vec2::splat(28.0),
    );
    draw::outlined(ui, tile, radius::CONTROL, pal.panel, pal.ink_2);
    let letters = draw::paint_text(
        ui,
        tile.center(),
        Align2::CENTER_CENTER,
        monogram,
        ty::MONOGRAM,
        pal.ink_2,
    );
    draw::announce(ui, letters, Role::Label, monogram);
}

/// A note (C19): panel fill, line-2 outline, no mark.
fn note(ui: &Ui, pal: &Palette, min: egui::Pos2, width: f32, text: &str) {
    let galley = draw::galley_wrapped(ui, text, ty::BODY, pal.ink, width - 26.0);
    let rect = Rect::from_min_size(min, vec2(width, galley.size().y + 20.0));
    draw::outlined(ui, rect, radius::CONTROL, pal.panel, pal.line_2);
    let text_rect = draw::paint_galley(ui, rect.min + vec2(13.0, 10.0), Align2::LEFT_TOP, galley);
    draw::announce(ui, text_rect, Role::Label, text);
}
