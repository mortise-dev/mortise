//! Adding an app (S1.2 to S1.4): choose the folder, see what was found, add it.
//!
//! The panel takes the place of the index and the workspace until the app is
//! added or Esc is pressed.

use eframe::egui::{
    self, Align2, Color32, FontId, Key, KeyboardShortcut, Modifiers, Rect, RichText, Role,
    ScrollArea, Ui, UiBuilder, Vec2, pos2, vec2,
};
use mortise_core::{AppSnapshot, ReadError, RunnerKind, TestOutcome};

use crate::environment_form::{EnvironmentForm, FormEvent, FormOptions, environment_form};
use crate::php_settings::{PhpEvent, PhpForm, PhpOptions, PhpTest, php_field, propose};
use crate::strings;
use crate::ui::draw::{self, Button, ButtonKind, Icon};
use crate::ui::theme::{Face, Palette, Type, radius, size, ty};

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
    /// The path that `checked` belongs to, or that is being read.
    checked_path: String,
    /// Where focus goes in the next frame.
    focus: Option<Focus>,
    /// The runtime connection of the app that was found, when it has the actuator.
    pub runtime: Option<EnvironmentForm>,
    /// How PHP runs for the app that was found (S1.3). Add saves the runner it shows.
    pub php: Option<PhpForm>,
    /// The test of [`AddPanel::php`]. Add gives it to the app, as Settings shows it.
    pub php_test: PhpTest,
    /// What detection found in the folder, after the label of [`AddPanel::php`].
    php_hint: &'static str,
    /// The folder is being read on another thread.
    pub reading: bool,
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
    /// Run `artisan --version` through the runner of "Run PHP with", in the app folder.
    TestPhp,
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
            php: None,
            php_test: PhpTest::default(),
            php_hint: "",
            reading: false,
        }
    }

    /// The panel while a folder is read.
    pub fn reading(path: String) -> Self {
        Self {
            checked_path: path.clone(),
            path,
            reading: true,
            focus: None,
            ..Self::empty()
        }
    }

    /// The panel after a folder was chosen. Focus is on the button that adds or opens.
    pub fn checked(path: String, checked: Checked) -> Self {
        let mut panel = Self {
            checked_path: path.clone(),
            path,
            ..Self::empty()
        };
        panel.set_checked(checked);
        panel.focus = Some(Focus::Primary);
        panel
    }

    /// Starts reading the folder in the path field.
    pub fn start_reading(&mut self) {
        self.checked_path = self.path.clone();
        self.checked = None;
        self.runtime = None;
        self.php = None;
        self.php_test = PhpTest::default();
        self.reading = true;
    }

    /// True when what was found belongs to the path in the field.
    pub fn is_current(&self) -> bool {
        self.checked.is_some() && self.path.trim() == self.checked_path.trim()
    }

    /// Takes in what reading the folder gave. The path that was read is kept, so a
    /// field changed during the read does not take a result that is not its own.
    pub fn set_checked(&mut self, checked: Checked) {
        self.focus = match checked {
            Checked::Failed(_) => None,
            _ => Some(Focus::Primary),
        };
        self.runtime = runtime_form(&checked);
        (self.php, self.php_hint) = match &checked {
            // The proposal of detection is selected first (S1.3).
            Checked::Found(app) => {
                let proposal = propose(&app.root);
                (Some(PhpForm::new(proposal.runner)), proposal.hint)
            }
            _ => (None, ""),
        };
        self.php_test = PhpTest::default();
        self.checked = Some(checked);
        self.reading = false;
    }

    /// True when the last test of "Run PHP with" failed for the runner the field shows,
    /// in the app folder: the app is then added read only (S1.4 c).
    fn php_failed(&self) -> bool {
        let (Some(Checked::Found(app)), Some(form)) = (&self.checked, &self.php) else {
            return false;
        };
        self.php_test
            .last_of(&RunnerKind::from_line(&form.line), &app.root)
            .is_some_and(|tested| matches!(tested.outcome, TestOutcome::Failed(_)))
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
/// `progress` is how far the read of the folder is, once it took longer than a moment
/// and the files are counted.
#[expect(clippy::too_many_arguments)]
pub fn add_panel(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    panel: &mut AddPanel,
    monogram: &str,
    progress: Option<(usize, usize)>,
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

    // The body scrolls when it does not fit, at 150% text size for one.
    let body = Rect::from_min_max(pos2(rect.left(), head.bottom()), rect.max);
    let back_id = ui.scope_id().with(("button", strings::BACK));
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        ScrollArea::vertical()
            .id_salt("add-panel")
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                reveal_focus(ui, body, back_id);
                let origin = ui.cursor().min;
                let area = Rect::from_min_size(origin, vec2(ui.available_width(), body.height()));
                let (body_action, bottom) =
                    add_body(ui, pal, area, panel, monogram, progress, keyboard, mac);
                if body_action.is_some() {
                    action = body_action;
                }
                // The controls are placed by hand: tell the scroll area how far they reach.
                ui.allocate_space(vec2(area.width(), bottom + BODY_PAD - origin.y));
            });
    });
    action
}

/// Scrolls to the control that just got focus, so that Tab never leaves it out of
/// sight. The way back, in the header, does not scroll with the body, and neither
/// does a control beside the panel, such as a rail tile.
fn reveal_focus(ui: &mut Ui, body: Rect, header_id: egui::Id) {
    let store = ui.scope_id().with("focused");
    let focused = ui.memory(|memory| memory.focused());
    let before = ui.data_mut(|data| data.get_temp::<Option<egui::Id>>(store).flatten());
    ui.data_mut(|data| data.insert_temp(store, focused));
    if focused == before || focused == Some(header_id) {
        return;
    }
    if let Some(response) = focused.and_then(|id| ui.ctx().read_response(id))
        && body.x_range().contains(response.rect.center().x)
    {
        ui.scroll_to_rect(response.rect.expand(8.0), None);
    }
}

/// Room under the last control of the panel.
const BODY_PAD: f32 = 16.0;

/// The field, what was found and the buttons, placed from the top left of `rect`.
/// Returns the action, and the bottom edge of what was painted.
#[expect(clippy::too_many_arguments)]
fn add_body(
    ui: &mut Ui,
    pal: &Palette,
    rect: Rect,
    panel: &mut AddPanel,
    monogram: &str,
    progress: Option<(usize, usize)>,
    keyboard: bool,
    mac: bool,
) -> (Option<AddAction>, f32) {
    let mut action = None;
    let left = rect.left() + 16.0;
    let column_w = 560.0_f32.min(rect.width() - 32.0);
    let top = rect.top() + 14.0;
    let mut y = top;
    // The bottom edge of the right column, when it is drawn.
    let mut right_bottom = 0.0_f32;

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
    // A changed path drops what was found for the folder before: Add, Open and Test
    // act only on the folder the field shows.
    if response.changed() && panel.path.trim() != panel.checked_path.trim() {
        panel.checked = None;
        panel.runtime = None;
        panel.php = None;
        panel.reading = false;
        panel.checked_path.clear();
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
        pos2((left + column_w + 45.0).min(rect.right() - 16.0), top),
        pos2(rect.right() - 16.0, rect.bottom()),
    );
    let command_enter = format!("{} Enter", strings::command_key(mac));

    // What is under the field depends on what the folder gave.
    let (mut primary, others): (Button<'_>, &[&str]);
    let primary_label;
    // The app name in the label of the primary button, and how the label is made.
    let mut named: Option<(&str, Label)> = None;
    let mut primary_action = AddAction::Add;
    match &panel.checked {
        None if panel.reading => {
            // Reading the folder. The bar is drawn only when it takes longer than a
            // moment, so that quick work does not flash.
            if let Some((read, total)) = progress {
                let share = read as f32 / total as f32;
                let label = strings::reading_files(read, total);
                let bar = draw::progress(ui, pal, pos2(left, y + 12.0), column_w, &label, share);
                return (action, bar);
            }
            return (action, y + 12.0);
        }
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
            let mut bottom = y + choose.height();
            if right_column.width() > 200.0 {
                let reach = note(
                    ui,
                    pal,
                    right_column.min,
                    right_column.width(),
                    strings::READS_NOTE,
                );
                bottom = bottom.max(reach);
            }
            return (action, bottom);
        }
        Some(Checked::Found(app)) => {
            others = &[strings::CANCEL];
            y += 16.0;
            if right_column.width() > 200.0 {
                right_bottom = found(ui, pal, right_column, app, monogram);
            }
            // Run PHP with: what detection proposes, and its Test (S1.3).
            if let Some(form) = &mut panel.php {
                let options = PhpOptions {
                    keyboard,
                    hint: panel.php_hint,
                    folder: &app.root,
                    salt: egui::IdSalt::new("add-php"),
                };
                let test = &mut panel.php_test;
                let (bottom, events) =
                    php_field(ui, pal, pos2(left, y), column_w, form, test, options);
                if events.contains(&PhpEvent::Test) {
                    action = Some(AddAction::TestPhp);
                }
                y = bottom + 16.0;
            }
            // Read after the field, so that a test that ended in this frame shows.
            let make: Label = if panel.php_failed() {
                strings::add_read_only
            } else {
                strings::add_named
            };
            primary_label = make(&app.name);
            named = Some((&app.name, make));
            primary = Button {
                label: &primary_label,
                kind: ButtonKind::Primary,
                keys: Some(&command_enter),
                ..Default::default()
            };
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
            named = Some((name, strings::open_named));
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
                return (action, y + again.height());
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

    // The buttons: the one that adds or opens, then the quiet ones. A long app name is
    // cut in the label so that every button stays in the panel; the full label stays
    // the accessible name.
    let others_w: f32 = others
        .iter()
        .map(|label| {
            let button = Button {
                label,
                ..Default::default()
            };
            button.width(ui) + 8.0
        })
        .sum();
    let room = column_w - others_w;
    let short_label;
    if let Some((name, make)) = named
        && primary.width(ui) > room
    {
        short_label = shortened(name, make, |label| {
            Button { label, ..primary }.width(ui) <= room
        });
        primary.said = Some(primary.label);
        primary.label = &short_label;
    }
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
    (action, (y + primary.height()).max(right_bottom))
}

/// Makes the label of a button from an app name.
type Label = fn(&str) -> String;

/// The label `make` gives for the longest start of `name`, with "…" after it, that `fits`.
fn shortened(name: &str, make: Label, fits: impl Fn(&str) -> bool) -> String {
    let cut = |end: usize| make(&format!("{}…", &name[..end]));
    let ends: Vec<usize> = name.char_indices().map(|(at, _)| at).skip(1).collect();
    let fitting = ends.partition_point(|&end| fits(&cut(end)));
    cut(fitting.checked_sub(1).map_or(0, |last| ends[last]))
}

/// The two boxes at the right once a folder is read: what was found, and how the
/// app will be shown on the rail.
fn found(ui: &Ui, pal: &Palette, column: Rect, app: &AppSnapshot, monogram: &str) -> f32 {
    let package = Type {
        size: 11.5,
        ..ty::CODE
    };
    let body = (ty::BODY, pal.ink);
    let dim = pal.ink_2;

    let found = Rect::from_min_size(
        column.min,
        vec2(
            column.width(),
            41.0 + draw::box_head_extra(
                ui,
                column.width(),
                strings::FOUND_TITLE,
                strings::FOUND_NOTE,
            ) + 6.0 * size::LIST_LINE_H
                + 8.0,
        ),
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
        vec2(
            column.width(),
            41.0 + draw::box_head_extra(
                ui,
                column.width(),
                strings::SHOWN_TITLE,
                strings::SHOWN_NOTE,
            ) + size::LIST_LINE_H
                + 34.0
                + 8.0,
        ),
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
    shown.bottom()
}

/// A note (C19): panel fill, line-2 outline, no mark.
fn note(ui: &Ui, pal: &Palette, min: egui::Pos2, width: f32, text: &str) -> f32 {
    let galley = draw::galley_wrapped(ui, text, ty::BODY, pal.ink, width - 26.0);
    let rect = Rect::from_min_size(min, vec2(width, galley.size().y + 20.0));
    draw::outlined(ui, rect, radius::CONTROL, pal.panel, pal.line_2);
    let text_rect = draw::paint_galley(ui, rect.min + vec2(13.0, 10.0), Align2::LEFT_TOP, galley);
    draw::announce(ui, text_rect, Role::Label, text);
    rect.bottom()
}
