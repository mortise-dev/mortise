//! Runtime: what the running app reports through the actuator.
//!
//! The runtime box of the Summary in each of its states (S6.5), and the tables of
//! routes and scheduled tasks. Runtime is never mixed with Findings: its marks are
//! strokes, and its words are Up, Down, Out of service and Unknown.

use eframe::egui::{Align2, Color32, Pos2, Rect, Role, Ui, pos2, vec2};
use mortise_core::{
    Answer, AppSnapshot, Component, Environment, Route, Runtime, RuntimeState, Status, Task,
    by_module, module_of_task,
};

use crate::runtime::strings;
use crate::ui::draw::{self, Button, ButtonKind, RuntimeMark, StampState};
use crate::ui::theme::{Face, Palette, Type, radius, size, ty};

/// What the interface knows about the runtime of the app on screen.
pub struct RuntimeView<'a> {
    /// The app has `nwidart/laravel-actuator` in its lock file.
    pub installed: bool,
    pub environments: &'a [Environment],
    /// The environment the runtime box shows.
    pub chosen: Option<&'a Environment>,
    pub state: RuntimeState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeAction {
    /// Show another environment in the runtime box.
    Choose(String),
    /// Open Settings on a new environment for the app.
    AddEnvironment,
    /// Open Settings on the environments of the app.
    OpenSettings,
    /// Ask the environment again, now.
    TryAgain,
    /// Copy a command or a line of configuration, and say so.
    Copy {
        text: &'static str,
        said: &'static str,
    },
}

/// The colour of a runtime state. Colour is only state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Good,
    Bad,
    Warn,
    Off,
}

impl Tone {
    pub fn color(self, pal: &Palette) -> Color32 {
        match self {
            Tone::Good => pal.good,
            Tone::Bad => pal.bad,
            Tone::Warn => pal.warn,
            Tone::Off => pal.off,
        }
    }
}

/// A runtime state in short: its mark, its word and its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Brief {
    pub mark: RuntimeMark,
    pub word: &'static str,
    pub tone: Tone,
}

impl Brief {
    pub fn of_status(status: Status) -> Self {
        let (mark, word, tone) = match status {
            Status::Up => (RuntimeMark::Up, strings::UP, Tone::Good),
            Status::Down => (RuntimeMark::Down, strings::DOWN, Tone::Bad),
            Status::OutOfService => (RuntimeMark::Bars, strings::OUT_OF_SERVICE, Tone::Warn),
            Status::Unknown => (RuntimeMark::Dashes, strings::RUNTIME_UNKNOWN, Tone::Off),
        };
        Self { mark, word, tone }
    }

    /// For the index header, which has one word for an app that gives nothing:
    /// "No answer", also where the app refused the credentials.
    pub fn for_header(self) -> Self {
        if self.word == strings::NOT_AUTHORIZED {
            Self {
                word: strings::NO_ANSWER,
                ..self
            }
        } else {
            self
        }
    }

    /// "No answer" and "Not authorized" are Mortise's words, not statuses of the app.
    pub fn of_answer(answer: &Answer) -> Self {
        let off = |word| Self {
            mark: RuntimeMark::Dashes,
            word,
            tone: Tone::Off,
        };
        match answer {
            Answer::Answered(runtime) => Self::of_status(runtime.status),
            Answer::NotAuthorized { .. } => off(strings::NOT_AUTHORIZED),
            Answer::NoAnswer { .. } | Answer::Unexpected { .. } => off(strings::NO_ANSWER),
        }
    }
}

impl RuntimeView<'_> {
    /// The name of the chosen environment.
    pub fn environment(&self) -> &str {
        self.chosen
            .map_or("", |environment| environment.name.as_str())
    }

    /// What the app answered, when it did.
    pub fn runtime(&self) -> Option<&Runtime> {
        match self.state.answer.as_deref() {
            Some(Answer::Answered(runtime)) => Some(runtime),
            _ => None,
        }
    }

    /// The state in short. `None` without an environment or before the first answer.
    pub fn brief(&self) -> Option<Brief> {
        self.chosen?;
        self.state.answer.as_deref().map(Brief::of_answer)
    }

    /// How many seconds ago the last answer came.
    pub fn checked(&self) -> Option<u64> {
        self.state
            .checked
            .map(|checked| checked.elapsed().as_secs())
    }
}

// ---------------------------------------------------------------------------
// The runtime box

/// One part of the value of a line, laid out from the right edge.
enum Part {
    Text(String, Type, Color32),
    State(Brief),
}

/// What a box holds under its heading, from top to bottom.
enum Item {
    /// A name at the left, a value at the right, 27 px high.
    Line {
        name: String,
        mono: bool,
        parts: Vec<Part>,
    },
    /// A sentence. Dim for what is not an error.
    Text(String, bool),
    /// Text to copy, or text of the app, quoted.
    Code(String),
    Buttons(Vec<(&'static str, ButtonKind, RuntimeAction)>),
}

const PAD: f32 = 13.0;

fn item_height(ui: &Ui, item: &Item, width: f32) -> f32 {
    match item {
        Item::Line { .. } => size::LIST_LINE_H,
        Item::Text(text, _) => {
            let text = draw::galley_wrapped(ui, text, ty::BODY, Color32::PLACEHOLDER, width);
            6.0 + text.size().y + 4.0
        }
        Item::Code(code) => 2.0 + draw::code_block_height(ui, code, width),
        Item::Buttons(_) => 8.0 + size::CONTROL_SM_H + 4.0,
    }
}

/// The lines of the components. When all are up each has its line. When some are
/// not, those come first and in full, and the rest share one line.
fn component_items(ui: &Ui, pal: &Palette, components: &[Component], width: f32) -> Vec<Item> {
    let dim = |text: String| Part::Text(text, ty::BODY, pal.ink_2);
    let line = |component: &Component, note: Option<String>| {
        let mut parts = Vec::new();
        if let Some(note) = note.filter(|note| !note.is_empty()) {
            parts.push(dim(note));
        }
        parts.push(Part::State(Brief::of_status(component.status)));
        Item::Line {
            name: component.name.clone(),
            mono: true,
            parts,
        }
    };
    if components
        .iter()
        .all(|component| component.status == Status::Up)
    {
        return components
            .iter()
            .map(|component| line(component, Some(detail(component))))
            .collect();
    }

    let mut items = Vec::new();
    let (up, not_up): (Vec<&Component>, Vec<&Component>) = components
        .iter()
        .partition(|component| component.status == Status::Up);
    for component in &not_up {
        items.push(line(component, None));
        let error = component.details.iter().find(|(key, _)| key == "error");
        match (error, component.status) {
            // The detail is the app's own text, quoted.
            (Some((_, error)), _) => items.push(Item::Code(error.clone())),
            (None, Status::OutOfService) => {
                items.push(Item::Text(strings::out_of_service(&component.name), true));
            }
            (None, _) => {
                let detail = detail(component);
                if !detail.is_empty() {
                    items.push(Item::Code(detail));
                }
            }
        }
    }
    if !up.is_empty() {
        // The names when they fit beside the count, else how many there are.
        let names = up
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let names_w = draw::galley(ui, &names, ty::CODE, Color32::PLACEHOLDER)
            .size()
            .x;
        let fits = names_w <= width - 70.0;
        // The count is said with the mark: "4 up".
        let mark = Brief {
            mark: RuntimeMark::Up,
            word: "",
            tone: Tone::Good,
        };
        items.push(Item::Line {
            name: if fits {
                names
            } else {
                strings::other_components(up.len())
            },
            mono: fits,
            parts: vec![
                Part::State(mark),
                Part::Text(
                    strings::n_up(up.len()),
                    Type {
                        size: 11.5,
                        ..ty::LABEL
                    },
                    pal.good,
                ),
            ],
        });
    }
    items
}

/// What a component says beside its status, in the app's own words.
fn detail(component: &Component) -> String {
    let get = |key: &str| {
        component
            .details
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    };
    let number = |key: &str| get(key).and_then(|value| value.parse::<f64>().ok());
    match component.name.as_str() {
        "diskSpace" => match (number("free"), number("total")) {
            (Some(free), Some(total)) => strings::disk(free, total),
            _ => String::new(),
        },
        "db" => get("connection").unwrap_or_default().to_owned(),
        "cache" => get("store").unwrap_or_default().to_owned(),
        // What the `modules` component keeps: the error of each name that is no module
        // of the app, one per line.
        "modules" => component
            .details
            .iter()
            .map(|(module, error)| strings::module_error(module, error))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => component
            .details
            .iter()
            .map(|(key, value)| format!("{key} {value}"))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// The status line at the top of a box: its state and a dim note at the right.
fn status_line(pal: &Palette, name: String, brief: Option<Brief>, note: Option<String>) -> Item {
    let mut parts: Vec<Part> = brief.into_iter().map(Part::State).collect();
    if let Some(note) = note {
        parts.push(Part::Text(note, ty::BODY, pal.ink_2));
    }
    Item::Line {
        name,
        mono: false,
        parts,
    }
}

fn items(
    ui: &Ui,
    pal: &Palette,
    name: &str,
    app: &AppSnapshot,
    view: &RuntimeView<'_>,
    width: f32,
) -> Vec<Item> {
    let default = ButtonKind::Default;
    if !view.installed {
        // a. The package is not installed.
        return vec![
            Item::Text(crate::strings::actuator_not_installed(name), false),
            Item::Code(strings::REQUIRE_ACTUATOR.to_owned()),
            Item::Buttons(vec![(
                crate::strings::COPY_COMMAND,
                default,
                RuntimeAction::Copy {
                    text: strings::REQUIRE_ACTUATOR,
                    said: crate::strings::COPIED_COMMAND,
                },
            )]),
        ];
    }
    let Some(environment) = view.chosen else {
        // b. Installed, no environment yet.
        return vec![
            Item::Text(crate::strings::no_environment(name), false),
            Item::Buttons(vec![(
                crate::strings::ADD_ENVIRONMENT,
                ButtonKind::Primary,
                RuntimeAction::AddEnvironment,
            )]),
        ];
    };
    let name = environment.name.as_str();
    let status = |brief, note| status_line(pal, strings::STATUS.to_owned(), brief, note);
    let Some(answer) = view.state.answer.as_deref() else {
        return vec![status(None, Some(strings::ASKING.to_owned()))];
    };
    let brief = Brief::of_answer(answer);
    let runtime = match answer {
        Answer::NoAnswer { url, message } => {
            // c. The app cannot be reached.
            return vec![
                status(Some(brief), None),
                Item::Text(strings::cannot_reach(name, url, message), false),
                Item::Buttons(vec![
                    (
                        strings::TRY_AGAIN,
                        ButtonKind::Primary,
                        RuntimeAction::TryAgain,
                    ),
                    (
                        crate::strings::OPEN_SETTINGS,
                        default,
                        RuntimeAction::OpenSettings,
                    ),
                ]),
            ];
        }
        Answer::NotAuthorized { url, status: code } => {
            // e. Not authorized.
            return vec![
                status(Some(brief), None),
                Item::Text(strings::refused_credentials(name, url, *code), false),
                Item::Buttons(vec![(
                    crate::strings::OPEN_SETTINGS,
                    ButtonKind::Primary,
                    RuntimeAction::OpenSettings,
                )]),
            ];
        }
        Answer::Unexpected {
            url,
            status: code,
            location,
        } => {
            // i. The actuator answers 500, or there is none at this address.
            let mut items = vec![
                status(Some(brief), None),
                Item::Text(
                    strings::answered_unexpected(name, url, *code, location.as_deref()),
                    false,
                ),
            ];
            if !(500..600).contains(code) {
                items.push(Item::Buttons(vec![(
                    crate::strings::OPEN_SETTINGS,
                    default,
                    RuntimeAction::OpenSettings,
                )]));
            }
            return items;
        }
        Answer::Answered(runtime) => runtime,
    };

    let mut items = vec![status(
        Some(brief),
        view.checked().map(crate::strings::checked_ago),
    )];
    match &runtime.components {
        Some(components) => {
            // The app's own components. Those of a module are in its own Summary.
            let split = by_module(app, components);
            items.extend(component_items(ui, pal, &split.app, width));
            // Not in the spec: each module that is not up, so that a Down app always
            // names what is down.
            for module in split.modules.iter().filter(|m| m.status != Status::Up) {
                items.push(Item::Line {
                    name: module.module.clone(),
                    mono: false,
                    parts: vec![Part::State(Brief::of_status(module.status))],
                });
            }
        }
        // f. The app hides the details.
        None => items.push(Item::Text(strings::COMPONENTS_HIDDEN.to_owned(), true)),
    }
    if let Some(info) = &runtime.info {
        let plain = |text: String| Part::Text(text, ty::BODY, pal.ink);
        let mono = |text: &str| {
            Part::Text(
                text.to_owned(),
                Type {
                    size: 11.5,
                    ..ty::CODE
                },
                pal.ink,
            )
        };
        let line = |name: &str, parts: Vec<Part>| Item::Line {
            name: name.to_owned(),
            mono: false,
            parts,
        };
        let environment: Vec<String> = info
            .environment
            .iter()
            .cloned()
            .chain(
                info.laravel
                    .as_ref()
                    .map(|laravel| format!("Laravel {laravel}")),
            )
            .collect();
        if !environment.is_empty() {
            items.push(line(
                strings::ENVIRONMENT,
                vec![plain(environment.join(", "))],
            ));
        }
        match (&info.git_branch, &info.git_commit) {
            (Some(branch), Some(commit)) => items.push(line(
                crate::strings::GIT,
                vec![mono(branch), plain(" at ".to_owned()), mono(commit)],
            )),
            (Some(one), None) | (None, Some(one)) => {
                items.push(line(crate::strings::GIT, vec![mono(one)]));
            }
            (None, None) => {}
        }
        let php: Vec<String> = info
            .php_version
            .iter()
            .chain(info.php_sapi.iter())
            .cloned()
            .collect();
        if !php.is_empty() {
            items.push(line(crate::strings::PHP, vec![plain(php.join(", "))]));
        }
        let memory = strings::memory(info.memory_usage, info.memory_peak, info.memory_limit);
        if !memory.is_empty() {
            items.push(line(strings::MEMORY, vec![plain(memory)]));
        }
    }
    // d. Endpoints that are not exposed.
    let missing = runtime.not_exposed();
    if !missing.is_empty() {
        items.push(Item::Text(strings::not_exposed(&missing, name), false));
        items.push(Item::Code(strings::EXPOSURE_LINE.to_owned()));
        items.push(Item::Buttons(vec![
            (
                strings::COPY_LINE,
                default,
                RuntimeAction::Copy {
                    text: strings::EXPOSURE_LINE,
                    said: strings::COPIED_LINE,
                },
            ),
            (strings::TRY_AGAIN, default, RuntimeAction::TryAgain),
        ]));
    }
    items
}

/// Paints the heading of a box whose right end holds a control or a note. Returns
/// the y where the content starts.
fn head(ui: &Ui, pal: &Palette, rect: Rect, title: &str, tall: bool) -> (f32, f32) {
    let extra = if tall { size::CONTROL_H - 18.0 } else { 0.0 };
    let mid = rect.top() + 1.0 + 8.0 + 9.0 + extra / 2.0;
    let title_rect = draw::paint_text(
        ui,
        pos2(rect.left() + PAD, mid),
        Align2::LEFT_CENTER,
        title,
        ty::BODY.with_face(Face::Sans700),
        pal.ink,
    );
    draw::announce(ui, title_rect, Role::Heading, title);
    let line = rect.top() + 1.0 + 8.0 + 18.0 + extra + 7.0;
    draw::hline(ui, rect.left() + PAD..rect.right() - PAD, line, pal.line);
    (mid, line + 1.0 + 6.0)
}

fn head_height(tall: bool) -> f32 {
    41.0 + if tall { size::CONTROL_H - 18.0 } else { 0.0 }
}

/// The runtime box of the Summary, in the state the connection is in (S6.5).
/// Returns its bottom edge and what was asked for. With `paint` false nothing is
/// drawn and nothing can be clicked.
#[expect(clippy::too_many_arguments)]
pub fn runtime_box(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    name: &str,
    app: &AppSnapshot,
    view: &RuntimeView<'_>,
    keyboard: bool,
    paint: bool,
) -> (f32, Option<RuntimeAction>) {
    let items = items(ui, pal, name, app, view, width - 2.0 * PAD);
    paint_box(ui, pal, min, width, view, &items, keyboard, paint)
}

/// The runtime box of a module's Summary (S4.3): the status of the module, the error
/// the app gives for it, then its own indicators by key. `None`, and nothing drawn,
/// when the module has no status in the last answer of the chosen environment.
/// Otherwise as [`runtime_box`].
#[expect(clippy::too_many_arguments)]
pub fn module_runtime_box(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    app: &AppSnapshot,
    module: &str,
    view: &RuntimeView<'_>,
    keyboard: bool,
    paint: bool,
) -> Option<(f32, Option<RuntimeAction>)> {
    let components = view.runtime()?.components.as_deref()?;
    let split = by_module(app, components);
    let runtime = split.of(module)?;
    let inner = width - 2.0 * PAD;
    let mut items = vec![status_line(
        pal,
        strings::status_of(module),
        Some(Brief::of_status(runtime.status)),
        view.checked().map(crate::strings::checked_ago),
    )];
    if let Some(error) = &runtime.error {
        // The app's own sentence, quoted.
        items.push(Item::Code(error.clone()));
    }
    items.extend(component_items(ui, pal, &runtime.indicators, inner));
    Some(paint_box(
        ui, pal, min, width, view, &items, keyboard, paint,
    ))
}

/// Paints a runtime box: the heading with the environment, then `items`.
#[expect(clippy::too_many_arguments)]
fn paint_box(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    view: &RuntimeView<'_>,
    items: &[Item],
    keyboard: bool,
    paint: bool,
) -> (f32, Option<RuntimeAction>) {
    let inner = width - 2.0 * PAD;
    // The switch between environments, when there is more than one.
    let switch = view.installed && view.environments.len() > 1;
    let height = head_height(switch)
        + items
            .iter()
            .map(|item| item_height(ui, item, inner))
            .sum::<f32>()
        + 8.0;
    let rect = Rect::from_min_size(min, vec2(width, height));
    if !paint {
        return (rect.bottom(), None);
    }
    let mut action = None;
    draw::outlined(ui, rect, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
    let title = match view.chosen.filter(|_| view.installed) {
        Some(environment) => strings::runtime_on(&environment.name),
        None => crate::strings::RUNTIME.to_owned(),
    };
    let (mid, mut y) = head(ui, pal, rect, &title, switch);
    if switch {
        let names: Vec<&str> = view.environments.iter().map(|e| e.name.as_str()).collect();
        let selected = names
            .iter()
            .position(|name| *name == view.environment())
            .unwrap_or(0);
        // Laid out once to learn its width, then painted at the right edge.
        let label_type = Type {
            size: 12.0,
            line: 12.0,
            ..ty::CONTROL
        };
        let track_w = 6.0
            + names
                .iter()
                .map(|name| {
                    (draw::galley(ui, name, label_type, Color32::PLACEHOLDER)
                        .size()
                        .x
                        + 20.0)
                        .ceil()
                })
                .sum::<f32>()
            + 2.0 * (names.len() as f32 - 1.0);
        let at = pos2(rect.right() - PAD - track_w, mid - size::CONTROL_H / 2.0);
        let (_, clicked) = draw::segmented(
            ui,
            pal,
            at,
            strings::ENVIRONMENT,
            &names,
            selected,
            keyboard,
        );
        if let Some(clicked) = clicked.filter(|clicked| *clicked != selected) {
            action = Some(RuntimeAction::Choose(names[clicked].to_owned()));
        }
    } else if view.installed && view.chosen.is_some() {
        draw::label(
            ui,
            pos2(rect.right() - PAD, mid),
            Align2::RIGHT_CENTER,
            view.environment(),
            ty::SECONDARY,
            pal.ink_2,
        );
    }

    let left = rect.left() + PAD;
    let right = rect.right() - PAD;
    for (i, item) in items.iter().enumerate() {
        let item_h = item_height(ui, item, inner);
        match item {
            Item::Line { name, mono, parts } => {
                let line_mid = y + size::LIST_LINE_H / 2.0;
                let name_type = if *mono {
                    Type {
                        size: 11.5,
                        ..ty::CODE
                    }
                } else {
                    ty::BODY
                };
                // The parts of the value, from the right edge. Text stops short of the
                // name and ends with an ellipsis when it is longer, as a detail an app
                // writes can be any length.
                let name_end = left + draw::galley(ui, name, name_type, pal.ink).size().x + 16.0;
                let mut x = right;
                for part in parts.iter().rev() {
                    let part_rect = match part {
                        Part::Text(text, ty, color) => {
                            if text.is_empty() {
                                continue;
                            }
                            let room = (x - name_end).max(24.0);
                            let galley = draw::galley_elided(ui, text, *ty, *color, room);
                            let rect = draw::paint_galley(
                                ui,
                                pos2(x, line_mid),
                                Align2::RIGHT_CENTER,
                                galley,
                            );
                            draw::announce(ui, rect, Role::Label, text);
                            rect
                        }
                        Part::State(brief) if brief.word.is_empty() => {
                            let at = Rect::from_min_size(
                                pos2(x - size::STATE_MARK, line_mid - size::STATE_MARK / 2.0),
                                vec2(size::STATE_MARK, size::STATE_MARK),
                            );
                            draw::runtime_mark(ui, at, brief.mark, brief.tone.color(pal));
                            at
                        }
                        Part::State(brief) => draw::runtime_state(
                            ui,
                            pos2(x, line_mid),
                            brief.mark,
                            brief.word,
                            brief.tone.color(pal),
                        ),
                    };
                    let gap = match part {
                        // Runs of one sentence touch; a state stands apart.
                        Part::Text(text, ..) if text.starts_with(' ') || text.ends_with(' ') => 0.0,
                        Part::Text(_, ty, _) if ty.face == Face::Mono400 => 0.0,
                        Part::Text(_, ty, _) if ty.face == Face::Sans600 => 5.0,
                        Part::State(brief) if brief.word.is_empty() => 5.0,
                        _ => 8.0,
                    };
                    x = part_rect.left() - gap;
                }
                let name_galley = draw::galley_elided(ui, name, name_type, pal.ink, x - 8.0 - left);
                let name_rect =
                    draw::paint_galley(ui, pos2(left, line_mid), Align2::LEFT_CENTER, name_galley);
                draw::announce(ui, name_rect, Role::Label, name);
                if i + 1 < items.len() && matches!(items[i + 1], Item::Line { .. }) {
                    draw::hline(ui, left..right, y + size::LIST_LINE_H - 1.0, pal.line);
                }
            }
            Item::Text(text, dim) => {
                let color = if *dim { pal.ink_2 } else { pal.ink };
                draw::paragraph(ui, pos2(left, y + 6.0), text, ty::BODY, color, inner);
            }
            Item::Code(code) => {
                draw::code_block(ui, pal, pos2(left, y + 2.0), inner, code);
            }
            Item::Buttons(buttons) => {
                let mut x = left;
                for (label, kind, what) in buttons {
                    let button = Button {
                        label,
                        kind: *kind,
                        small: true,
                        ..Default::default()
                    };
                    let response = button.show(ui, pal, pos2(x, y + 8.0), keyboard);
                    if response.clicked() {
                        action = Some(what.clone());
                    }
                    x = response.rect.right() + 8.0;
                }
            }
        }
        y += item_h;
    }
    (rect.bottom(), action)
}

/// A box with a heading and one sentence: what stands in the place of routes or
/// scheduled tasks when their endpoint is not exposed. Returns its bottom edge.
pub fn message_box(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    title: &str,
    text: &str,
    paint: bool,
) -> f32 {
    let inner = width - 2.0 * PAD;
    let galley = draw::galley_wrapped(ui, text, ty::BODY, pal.ink_2, inner);
    let rect = Rect::from_min_size(min, vec2(width, 41.0 + 2.0 + galley.size().y + 4.0 + 8.0));
    if paint {
        let y = draw::box_head(ui, pal, rect, title, "");
        let text_rect = draw::paint_galley(
            ui,
            pos2(rect.left() + PAD, y + 2.0),
            Align2::LEFT_TOP,
            galley,
        );
        draw::announce(ui, text_rect, Role::Label, text);
    }
    rect.bottom()
}

// ---------------------------------------------------------------------------
// Tables of routes and scheduled tasks

/// One cell of a table.
enum Cell {
    Text(String, Type, Color32),
    /// The code stamp of a module.
    Stamp(String),
}

/// How wide a column is: a fixed width, or a share of what the fixed ones leave.
#[derive(Clone, Copy)]
enum Width {
    Fixed(f32),
    Share(f32),
}

/// A box that is a table (C8): a heading, a header row, then 28 px rows with
/// hairlines between them. Returns its bottom edge.
#[expect(clippy::too_many_arguments)]
fn table_box(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    title: &str,
    note: &str,
    columns: &[(&str, Width)],
    rows: &[Vec<Cell>],
    paint: bool,
) -> f32 {
    let height = 41.0 - 6.0
        + draw::box_head_extra(ui, width, title, note)
        + size::TABLE_HEAD_H
        + rows.len() as f32 * size::TABLE_ROW_H
        + 1.0;
    let rect = Rect::from_min_size(min, vec2(width, height));
    if !paint {
        return rect.bottom();
    }
    let top = draw::box_head(ui, pal, rect, title, note) - 6.0;
    let left = rect.left() + 14.0;
    let right = rect.right() - 12.0;
    let gap = 12.0;
    let fixed: f32 = columns
        .iter()
        .map(|(_, width)| match width {
            Width::Fixed(width) => *width,
            Width::Share(_) => 0.0,
        })
        .sum();
    let shares: f32 = columns
        .iter()
        .map(|(_, width)| match width {
            Width::Share(share) => *share,
            Width::Fixed(_) => 0.0,
        })
        .sum();
    let free = (right - left - fixed - gap * (columns.len() as f32 - 1.0)).max(0.0);
    let mut edges = Vec::new();
    let mut x = left;
    for (_, width) in columns {
        let w = match width {
            Width::Fixed(width) => *width,
            Width::Share(share) => (free * share / shares).floor(),
        };
        edges.push((x, x + w));
        x += w + gap;
    }

    let head_bottom = top + size::TABLE_HEAD_H;
    draw::hline(
        ui,
        rect.left() + 1.0..rect.right() - 1.0,
        head_bottom - 1.0,
        pal.line_2,
    );
    for ((label, _), (from, _)) in columns.iter().zip(&edges) {
        let cell = draw::paint_text(
            ui,
            pos2(*from, top + size::TABLE_HEAD_H / 2.0),
            Align2::LEFT_CENTER,
            label,
            ty::LABEL,
            pal.ink_2,
        );
        draw::announce(ui, cell, Role::ColumnHeader, label);
    }
    for (i, row) in rows.iter().enumerate() {
        let row_top = head_bottom + i as f32 * size::TABLE_ROW_H;
        let mid = row_top + size::TABLE_ROW_H / 2.0;
        if i + 1 < rows.len() {
            draw::hline(
                ui,
                rect.left() + 1.0..rect.right() - 1.0,
                row_top + size::TABLE_ROW_H - 1.0,
                pal.line,
            );
        }
        for (cell, (from, to)) in row.iter().zip(&edges) {
            match cell {
                Cell::Text(text, ty, color) => {
                    if text.is_empty() {
                        continue;
                    }
                    let galley = draw::galley_elided(ui, text, *ty, *color, to - from);
                    let cell_rect =
                        draw::paint_galley(ui, pos2(*from, mid), Align2::LEFT_CENTER, galley);
                    draw::announce(ui, cell_rect, Role::Cell, text);
                }
                Cell::Stamp(code) => {
                    let at = pos2(*from, (mid - size::STAMP_H / 2.0).round());
                    let stamp = draw::stamp(ui, pal, at, code, StampState::default());
                    draw::announce(ui, stamp, Role::Cell, code);
                }
            }
        }
    }
    rect.bottom()
}

const CELL_CODE: Type = Type {
    size: 11.5,
    ..ty::CODE
};

/// The scheduled tasks. With `modules`, a column says which module a task belongs
/// to: the module of its class or of its artisan command. Returns its bottom edge.
#[expect(clippy::too_many_arguments)]
pub fn tasks_box(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    note: &str,
    tasks: &[&Task],
    modules: Option<&AppSnapshot>,
    now: i64,
    paint: bool,
) -> f32 {
    let mut columns = vec![
        (strings::COL_TASK, Width::Share(1.0)),
        (strings::COL_SCHEDULE, Width::Fixed(110.0)),
        (strings::COL_NEXT_RUN, Width::Fixed(150.0)),
    ];
    if modules.is_some() {
        columns.push((crate::strings::COL_MODULE, Width::Fixed(60.0)));
    }
    let rows: Vec<Vec<Cell>> = tasks
        .iter()
        .map(|task| {
            let schedule = match (&task.expression, task.interval_ms) {
                (Some(expression), _) => Cell::Text(expression.clone(), CELL_CODE, pal.ink),
                (None, Some(interval)) => {
                    Cell::Text(crate::strings::every(interval), ty::BODY, pal.ink)
                }
                (None, None) => Cell::Text(String::new(), ty::BODY, pal.ink),
            };
            let next = strings::next_run(task.next.as_deref(), task.timezone.as_deref(), now);
            let next_color = if task.next.is_some() {
                pal.ink
            } else {
                pal.ink_2
            };
            let mut row = vec![
                Cell::Text(task.target.clone(), CELL_CODE, pal.ink),
                schedule,
                Cell::Text(next, ty::BODY, next_color),
            ];
            if let Some(app) = modules {
                row.push(match module_of_task(app, task) {
                    Some(module) => Cell::Stamp(module.code.clone()),
                    None => Cell::Text(crate::strings::NONE.to_owned(), ty::BODY, pal.ink_3),
                });
            }
            row
        })
        .collect();
    table_box(
        ui,
        pal,
        min,
        width,
        crate::strings::SCHEDULED_TASKS,
        note,
        &columns,
        &rows,
        paint,
    )
}

/// The routes of one module. In a narrow workspace the name and then the
/// middleware give way. Returns its bottom edge.
pub fn routes_box(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    note: &str,
    routes: &[&Route],
    paint: bool,
) -> f32 {
    let name = width >= 760.0;
    let middleware = width >= 560.0;
    let mut columns = vec![
        (strings::COL_METHOD, Width::Fixed(60.0)),
        (strings::COL_URI, Width::Share(1.35)),
    ];
    if name {
        columns.push((strings::COL_NAME, Width::Share(1.0)));
    }
    columns.push((strings::COL_ACTION, Width::Share(1.0)));
    if middleware {
        columns.push((strings::COL_MIDDLEWARE, Width::Fixed(140.0)));
    }
    let rows: Vec<Vec<Cell>> = routes
        .iter()
        .map(|route| {
            // HEAD comes with every GET in Laravel and says nothing.
            let methods: Vec<&str> = route
                .methods
                .iter()
                .map(String::as_str)
                .filter(|method| *method != "HEAD" || route.methods.len() == 1)
                .collect();
            let mut row = vec![
                Cell::Text(methods.join(", "), CELL_CODE, pal.ink_2),
                Cell::Text(route.uri.clone(), CELL_CODE, pal.ink),
            ];
            if name {
                row.push(Cell::Text(
                    route.name.clone().unwrap_or_default(),
                    CELL_CODE,
                    pal.ink_2,
                ));
            }
            row.push(Cell::Text(
                short_action(&route.action).to_owned(),
                CELL_CODE,
                pal.ink,
            ));
            if middleware {
                row.push(Cell::Text(route.middleware.join(", "), ty::BODY, pal.ink));
            }
            row
        })
        .collect();
    table_box(
        ui,
        pal,
        min,
        width,
        crate::strings::ROUTES,
        note,
        &columns,
        &rows,
        paint,
    )
}

/// An action without its namespace: `CartController@show`.
fn short_action(action: &str) -> &str {
    action.rsplit('\\').next().unwrap_or(action)
}
