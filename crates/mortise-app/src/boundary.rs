//! The Boundary tab (S5.1 to S5.4): which module uses which, the files that make a
//! link, the links that break a rule, and the rules.

use eframe::egui::{
    Align2, Color32, Pos2, Rect, Role, ScrollArea, Sense, Shape, Stroke, Ui, UiBuilder, Vec2,
    WidgetInfo, pos2, vec2,
};
use mortise_core::{
    AppSnapshot, Link, Module, ModuleState, Reference, Rules, RulesSource, columns,
};

use crate::draw::{self, Button, ButtonKind, StampState};
use crate::strings;
use crate::theme::{Face, Palette, Type, radius, size, stroke, ty};

/// Above this many modules the graph of the whole app gives way to a table of links.
pub const GRAPH_LIMIT: usize = 12;
/// At most this many neighbours are drawn on one side of a module.
const NEIGHBOUR_LIMIT: usize = 8;

/// What the Boundary tab keeps for one app.
#[derive(Debug, Default)]
pub struct BoundaryState {
    /// The neighbour whose link is shown under the graph of one module.
    pub other: Option<String>,
    /// The row of the file list that is selected.
    pub file: usize,
    /// "Problems only" in the view of the whole app.
    pub problems_only: bool,
    /// The text of the links filter.
    pub filter: String,
    /// The text of the folders field of the second rule, and the folders it was made from.
    folders_text: String,
    folders_from: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundaryAction {
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
}

/// Paints the Boundary tab into `body`, which scrolls.
pub fn boundary(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    app: &AppSnapshot,
    module: Option<&Module>,
    state: &mut BoundaryState,
    keyboard: bool,
) -> Option<BoundaryAction> {
    let mut action = None;
    let salt = module.map_or("all", |module| module.name.as_str());
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        ScrollArea::vertical()
            .id_salt(("boundary", salt))
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let origin = ui.cursor().min;
                let width = ui.available_width();
                let view = View {
                    pal,
                    app,
                    keyboard,
                    left: origin.x + 16.0,
                    width: width - 32.0,
                };
                let mut y = origin.y + 14.0;
                if let RulesSource::Invalid(message) = &app.rules_source {
                    y = view.note(ui, y, &strings::rules_invalid(message)) + 12.0;
                }
                let bottom = match module {
                    Some(module) => view.one_module(ui, y, module, state, &mut action),
                    None if app.modules.len() > GRAPH_LIMIT => {
                        view.links_table(ui, y, state, &mut action)
                    }
                    None => view.all_modules(ui, y, state, &mut action),
                };
                // Tell the scroll area how high the content is.
                ui.allocate_rect(
                    Rect::from_min_max(origin, pos2(origin.x + width, bottom + 16.0)),
                    Sense::hover(),
                );
            });
    });
    action
}

struct View<'a> {
    pal: &'a Palette,
    app: &'a AppSnapshot,
    keyboard: bool,
    /// The left edge of the content and its width, inside the 16 px padding.
    left: f32,
    width: f32,
}

/// A node of a graph: where it is and which module it stands for.
struct Node<'a> {
    rect: Rect,
    module: &'a Module,
}

impl View<'_> {
    fn code(&self, name: &str) -> &str {
        self.app
            .module(name)
            .map_or("", |module| module.code.as_str())
    }

    fn is_off(&self, name: &str) -> bool {
        self.app
            .module(name)
            .is_some_and(|module| module.state == ModuleState::Off)
    }

    fn paragraph(&self, ui: &Ui, y: f32, text: &str) -> f32 {
        draw::paragraph(
            ui,
            pos2(self.left, y),
            text,
            ty::BODY,
            self.pal.ink_2,
            self.width.min(560.0),
        )
        .bottom()
    }

    /// A note (C19): panel fill, line-2 outline, no mark. Returns its bottom edge.
    fn note(&self, ui: &Ui, y: f32, text: &str) -> f32 {
        let galley = draw::galley_wrapped(ui, text, ty::BODY, self.pal.ink, self.width - 24.0);
        let rect =
            Rect::from_min_size(pos2(self.left, y), vec2(self.width, galley.size().y + 18.0));
        draw::outlined(ui, rect, radius::CONTROL, self.pal.panel, self.pal.line_2);
        let text_rect =
            draw::paint_galley(ui, rect.min + vec2(12.0, 9.0), Align2::LEFT_TOP, galley);
        draw::announce(ui, text_rect, Role::Label, text);
        rect.bottom()
    }

    // --- One module (S5.1, S5.2) ---

    fn one_module(
        &self,
        ui: &mut Ui,
        y: f32,
        module: &Module,
        state: &mut BoundaryState,
        action: &mut Option<BoundaryAction>,
    ) -> f32 {
        let pal = self.pal;
        let rules = &self.app.rules;
        let by_weight = |links: &mut Vec<&Link>, name: fn(&Link) -> &str| {
            links.sort_by(|a, b| {
                b.refs
                    .len()
                    .cmp(&a.refs.len())
                    .then_with(|| name(a).cmp(name(b)))
            });
        };
        let mut uses: Vec<&Link> = self.app.uses(&module.name).collect();
        by_weight(&mut uses, |link| &link.to);
        let mut used_by: Vec<&Link> = self.app.used_by(&module.name).collect();
        by_weight(&mut used_by, |link| &link.from);
        if uses.is_empty() && used_by.is_empty() {
            return self.paragraph(ui, y, &strings::no_links_of(&module.name));
        }

        // The neighbour whose link is shown: the one chosen, else the first that breaks a rule.
        let neighbours: Vec<&str> = uses
            .iter()
            .map(|link| link.to.as_str())
            .chain(used_by.iter().map(|link| link.from.as_str()))
            .collect();
        let chosen = state
            .other
            .as_deref()
            .filter(|other| neighbours.contains(other));
        let other = chosen
            .or_else(|| {
                uses.iter()
                    .find(|link| link.is_problem(rules))
                    .map(|link| link.to.as_str())
            })
            .or_else(|| {
                used_by
                    .iter()
                    .find(|link| link.is_problem(rules))
                    .map(|link| link.from.as_str())
            })
            .unwrap_or(neighbours[0])
            .to_owned();

        // The graph: what uses the module on the left, what it uses on the right.
        let rows = uses.len().max(used_by.len()).clamp(1, NEIGHBOUR_LIMIT) as f32;
        let (node_h, gap) = (40.0, 18.0);
        let inner_h = rows * node_h + (rows - 1.0) * gap;
        let graph = Rect::from_min_size(
            pos2(self.left, y),
            vec2(self.width, 16.0 + 20.0 + 14.0 + inner_h + 26.0),
        );
        draw::outlined(ui, graph, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);

        let side_w = ((self.width - 64.0 - 180.0 - 160.0) / 2.0).clamp(130.0, 200.0);
        let left_x = graph.left() + 32.0;
        let right_x = graph.right() - 32.0 - side_w;
        let centre = Rect::from_center_size(
            pos2(
                ((left_x + side_w + right_x) / 2.0).round(),
                (graph.top() + 50.0 + inner_h / 2.0).round(),
            ),
            vec2(180.0, node_h),
        );
        let caption = |ui: &Ui, x: f32, text: &str| {
            draw::label(
                ui,
                pos2(x, graph.top() + 16.0),
                Align2::LEFT_TOP,
                text,
                ty::LABEL,
                pal.ink_2,
            );
        };
        caption(ui, left_x, &strings::used_by_count(used_by.len()));
        caption(ui, centre.left(), strings::SELECTED);
        caption(ui, right_x, &strings::uses_count(uses.len()));

        let column = |count: usize, x: f32| -> Vec<Rect> {
            let shown = count.min(NEIGHBOUR_LIMIT) as f32;
            let height = shown * node_h + (shown - 1.0).max(0.0) * gap;
            let top = graph.top() + 50.0 + ((inner_h - height) / 2.0).round();
            (0..count.min(NEIGHBOUR_LIMIT))
                .map(|i| {
                    Rect::from_min_size(
                        pos2(x, top + i as f32 * (node_h + gap)),
                        vec2(side_w, node_h),
                    )
                })
                .collect()
        };
        let left_slots = column(used_by.len(), left_x);
        let right_slots = column(uses.len(), right_x);

        // Connectors first, so that the nodes are painted over their ends.
        let port = |i: usize, count: usize| {
            centre.top() + (i as f32 + 1.0) * node_h / (count as f32 + 1.0)
        };
        for (i, slot) in left_slots.iter().enumerate() {
            let problem = left_slots.len() == used_by.len() || i + 1 < left_slots.len();
            let problem = problem && used_by[i].is_problem(rules);
            let to = pos2(centre.left(), port(i, left_slots.len()).round());
            connector(ui, pal, pos2(slot.right(), slot.center().y), to, problem);
        }
        for (i, slot) in right_slots.iter().enumerate() {
            let problem = right_slots.len() == uses.len() || i + 1 < right_slots.len();
            let problem = problem && uses[i].is_problem(rules);
            let from = pos2(centre.right(), port(i, right_slots.len()).round());
            connector(ui, pal, from, pos2(slot.left(), slot.center().y), problem);
        }

        // The module itself, in the middle.
        ui.painter().rect(
            centre,
            0,
            pal.panel,
            Stroke::new(1.5, pal.ink),
            eframe::egui::StrokeKind::Inside,
        );
        self.node_text(
            ui,
            centre,
            module,
            &strings::files(module.files),
            pal.ink_2,
            false,
        );

        let mut neighbour = |ui: &Ui, slots: &[Rect], links: &[&Link], name: fn(&Link) -> &str| {
            for (i, slot) in slots.iter().enumerate() {
                // When a side has more than fits, the last node stands for the rest.
                if links.len() > slots.len() && i + 1 == slots.len() {
                    draw::outlined(ui, *slot, 0, pal.surface, pal.line_2);
                    let more = strings::more(links.len() - (slots.len() - 1));
                    draw::label(
                        ui,
                        slot.center(),
                        Align2::CENTER_CENTER,
                        &more,
                        ty::BODY,
                        pal.ink_2,
                    );
                    continue;
                }
                let link = links[i];
                let Some(neighbour) = self.app.module(name(link)) else {
                    continue;
                };
                let selected = neighbour.name == other;
                let response = {
                    // A module can be on both sides, so the side is part of the id.
                    let side = slot.left() < centre.left();
                    ui.interact(
                        *slot,
                        ui.scope_id().with(("neighbour", &neighbour.name, side)),
                        Sense::click(),
                    )
                };
                response.widget_info(|| {
                    WidgetInfo::selected(Role::ListBoxOption, true, selected, &neighbour.name)
                });
                if response.clicked() {
                    state.other = Some(neighbour.name.clone());
                    state.file = 0;
                }
                if selected {
                    ui.painter().rect(
                        *slot,
                        0,
                        pal.well,
                        Stroke::new(1.5, pal.ink),
                        eframe::egui::StrokeKind::Inside,
                    );
                    draw::peg(ui, pal, *slot);
                } else {
                    let fill = if response.hovered() {
                        pal.well
                    } else {
                        pal.surface
                    };
                    draw::outlined(ui, *slot, 0, fill, pal.line_2);
                }
                let problem = link.is_problem(rules);
                let color = if problem { pal.bad } else { pal.ink_2 };
                self.node_text(
                    ui,
                    *slot,
                    neighbour,
                    &strings::refs_state(link, rules),
                    color,
                    selected,
                );
            }
        };
        neighbour(ui, &left_slots, &used_by, |link| &link.from);
        neighbour(ui, &right_slots, &uses, |link| &link.to);

        // Under the graph: the files that make the link with the chosen neighbour.
        let out = self.app.link(&module.name, &other);
        let back = self.app.link(&other, &module.name);
        self.link_files(
            ui,
            graph.bottom() + 12.0,
            &module.name,
            &other,
            out,
            back,
            state,
            action,
        )
    }

    /// The stamp, the name and a second line of a node.
    fn node_text(
        &self,
        ui: &Ui,
        rect: Rect,
        module: &Module,
        second: &str,
        color: Color32,
        selected: bool,
    ) {
        let pal = self.pal;
        let mid = rect.center().y;
        let off = module.state == ModuleState::Off;
        let x = rect.left() + if selected { 14.0 } else { 8.0 };
        let stamp = draw::stamp(
            ui,
            pal,
            pos2(x, (mid - size::STAMP_H / 2.0).round()),
            &module.code,
            StampState { selected, off },
        );
        let text_x = stamp.right() + 8.0;
        let text_w = (rect.right() - 8.0 - text_x).max(0.0);
        let name_type = Type {
            size: 12.0,
            line: 16.0,
            ..ty::BODY.with_face(Face::Sans600)
        };
        let name = draw::galley_elided(ui, &module.name, name_type, pal.ink, text_w);
        if second.is_empty() {
            draw::paint_galley(ui, pos2(text_x, mid), Align2::LEFT_CENTER, name);
        } else {
            draw::paint_galley(ui, pos2(text_x, mid - 15.0), Align2::LEFT_TOP, name);
            let second_type = Type {
                size: 11.0,
                line: 14.0,
                ..ty::CODE
            };
            let second = draw::galley_elided(ui, second, second_type, color, text_w);
            draw::paint_galley(ui, pos2(text_x, mid + 1.0), Align2::LEFT_TOP, second);
        }
    }

    /// The files of a link, or of both directions when two modules use each other.
    #[expect(clippy::too_many_arguments)]
    fn link_files(
        &self,
        ui: &mut Ui,
        y: f32,
        module: &str,
        other: &str,
        out: Option<&Link>,
        back: Option<&Link>,
        state: &mut BoundaryState,
        action: &mut Option<BoundaryAction>,
    ) -> f32 {
        let pal = self.pal;
        let rules = &self.app.rules;
        let circle = out.zip(back).filter(|(out, _)| out.circular);

        // One section for a plain link, two for a circle.
        let sections: Vec<(&Link, bool)> = match (circle, out, back) {
            (Some((out, back)), ..) => vec![(out, true), (back, true)],
            (None, Some(out), _) => vec![(out, false)],
            (None, None, Some(back)) => vec![(back, false)],
            (None, None, None) => return y,
        };
        let rows: usize = sections.iter().map(|(link, _)| link.refs.len()).sum();
        let titled = sections.len() > 1;
        let section_head = if titled { 30.0 } else { 0.0 };
        let footer_text = match circle {
            Some(_) => {
                let carrier = self
                    .app
                    .cycles
                    .iter()
                    .find(|cycle| cycle.modules.iter().any(|member| member == module))
                    .map_or(module, |cycle| cycle.carrier.as_str());
                strings::reported_on(carrier)
            }
            None => strings::rule_of_link(&sections[0].0.to, rules),
        };
        let copy = Button {
            label: strings::COPY_PATH,
            ..Default::default()
        };
        let open = Button {
            label: "Open in editor",
            kind: ButtonKind::Primary,
            ..Default::default()
        };
        let footer_w = self.width - 26.0 - copy.width(ui) - 8.0 - open.width(ui) - 16.0;
        let footer = draw::galley_wrapped(ui, &footer_text, ty::SECONDARY, pal.ink_2, footer_w);
        let footer_h = (footer.size().y + 20.0).max(46.0);

        let height = 35.0
            + sections.len() as f32 * (section_head + size::TABLE_HEAD_H)
            + rows as f32 * size::TABLE_ROW_H
            + footer_h;
        let panel = Rect::from_min_size(pos2(self.left, y), vec2(self.width, height));

        // Head: what the link is, and how much of it breaks a rule.
        let (title, note, bad) = match circle {
            Some((out, back)) => {
                // The same sentence from either side: the module that carries the finding first.
                let carrier_first = self.app.cycles.iter().any(|cycle| {
                    cycle.carrier == other && cycle.modules.iter().any(|member| member == module)
                });
                let (first, second) = if carrier_first {
                    (other, module)
                } else {
                    (module, other)
                };
                (
                    strings::use_each_other(first, second),
                    strings::circular_files(out.files() + back.files()),
                    rules.no_circular_use,
                )
            }
            None => {
                let link = sections[0].0;
                let internal = link.internal_files();
                if rules.public_surface_only && internal > 0 {
                    (
                        strings::a_uses_b(&link.from, &link.to),
                        strings::files_read_internals(internal, link.files()),
                        true,
                    )
                } else {
                    (
                        strings::a_uses_b(&link.from, &link.to),
                        strings::files(link.files() as u64),
                        false,
                    )
                }
            }
        };
        let mut row_y = draw::box_head(ui, pal, panel, &title, if bad { "" } else { &note }) - 6.0;
        if bad {
            let mid = panel.top() + 18.0;
            let text = draw::label(
                ui,
                pos2(panel.right() - 13.0, mid),
                Align2::RIGHT_CENTER,
                &note,
                ty::LABEL,
                pal.bad,
            );
            let mark = Rect::from_min_size(
                pos2(text.left() - 5.0 - 12.0, mid - 6.0),
                Vec2::splat(size::STATE_MARK),
            );
            draw::mark_bad(ui, mark, pal.bad, pal.surface);
        }

        // The table: a mark, the file, the line, the class it reads.
        let file_x = panel.left() + 42.0;
        let reads_x = (panel.left() + panel.width() * 0.63).round();
        let line_right = reads_x - 12.0;
        let mut index = 0;
        let mut selected_path = None;
        for (link, _) in &sections {
            if titled {
                let mid = row_y + section_head / 2.0;
                ui.painter().rect_filled(
                    Rect::from_min_max(
                        pos2(panel.left() + 1.0, row_y),
                        pos2(panel.right() - 1.0, row_y + section_head),
                    ),
                    0,
                    pal.panel,
                );
                draw::label(
                    ui,
                    pos2(panel.left() + 13.0, mid),
                    Align2::LEFT_CENTER,
                    &strings::a_uses_b(&link.from, &link.to),
                    ty::BODY.with_face(Face::Sans600),
                    pal.ink,
                );
                draw::label(
                    ui,
                    pos2(panel.right() - 13.0, mid),
                    Align2::RIGHT_CENTER,
                    &strings::files(link.files() as u64),
                    ty::SECONDARY,
                    pal.ink_2,
                );
                row_y += section_head;
            }
            let head_mid = row_y + size::TABLE_HEAD_H / 2.0;
            draw::paint_text(
                ui,
                pos2(file_x, head_mid),
                Align2::LEFT_CENTER,
                strings::COL_FILE,
                ty::LABEL,
                pal.ink_2,
            );
            draw::paint_text(
                ui,
                pos2(line_right, head_mid),
                Align2::RIGHT_CENTER,
                strings::COL_LINE,
                ty::LABEL,
                pal.ink_2,
            );
            draw::paint_text(
                ui,
                pos2(reads_x, head_mid),
                Align2::LEFT_CENTER,
                strings::COL_READS,
                ty::LABEL,
                pal.ink_2,
            );
            row_y += size::TABLE_HEAD_H;
            draw::hline(
                ui,
                panel.left() + 1.0..panel.right() - 1.0,
                row_y - 1.0,
                pal.line_2,
            );

            for reference in &link.refs {
                let row = Rect::from_min_max(
                    pos2(panel.left() + 1.0, row_y),
                    pos2(panel.right() - 1.0, row_y + size::TABLE_ROW_H),
                );
                let selected = index == state.file.min(rows - 1);
                if self.file_row(
                    ui,
                    row,
                    reference,
                    link,
                    selected,
                    (file_x, line_right, reads_x),
                ) {
                    state.file = index;
                }
                if selected {
                    selected_path = Some((reference.file.clone(), reference.line));
                }
                index += 1;
                row_y += size::TABLE_ROW_H;
            }
        }

        // Footer: the rule, and what can be done with the selected file.
        let foot = Rect::from_min_max(
            pos2(panel.left() + 1.0, row_y),
            pos2(panel.right() - 1.0, panel.bottom() - 1.0),
        );
        ui.painter().rect_filled(foot, 0, pal.panel);
        draw::hline(ui, foot.left()..foot.right(), foot.top(), pal.line);
        let text = draw::paint_galley(
            ui,
            pos2(foot.left() + 12.0, foot.center().y),
            Align2::LEFT_CENTER,
            footer,
        );
        draw::announce(ui, text, Role::Label, &footer_text);
        let copy_min = pos2(
            foot.right() - 12.0 - copy.width(ui),
            foot.center().y - copy.height() / 2.0,
        );
        let open_min = pos2(copy_min.x - 8.0 - open.width(ui), copy_min.y);
        if open.show(ui, pal, open_min, self.keyboard).clicked()
            && let Some((file, line)) = &selected_path
        {
            // The file at the line that names the class.
            *action = Some(BoundaryAction::OpenInEditor {
                path: self.app.root.join(file),
                line: *line,
            });
        }
        if copy.show(ui, pal, copy_min, self.keyboard).clicked()
            && let Some((path, _)) = selected_path
        {
            *action = Some(BoundaryAction::CopyPath(path));
        }
        panel.bottom()
    }

    /// One file of a link. Returns true when the row was clicked.
    fn file_row(
        &self,
        ui: &Ui,
        row: Rect,
        reference: &Reference,
        link: &Link,
        selected: bool,
        (file_x, line_right, reads_x): (f32, f32, f32),
    ) -> bool {
        let pal = self.pal;
        let id = ui
            .scope_id()
            .with(("link-file", &link.from, &reference.file, &reference.class));
        let response = ui.interact(row, id, Sense::click());
        response.widget_info(|| WidgetInfo::selected(Role::Row, true, selected, &reference.file));
        if selected {
            // A selected list row: well fill, a line above and below, and the peg.
            ui.painter().rect_filled(row, 0, pal.well);
            draw::hline(ui, row.left()..row.right(), row.top() - 1.0, pal.line_2);
            draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line_2);
            draw::peg(ui, pal, row);
        } else {
            if response.hovered() {
                ui.painter().rect_filled(row, 0, pal.well);
            }
            draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line);
        }

        let mid = row.center().y;
        let behind = if selected || response.hovered() {
            pal.well
        } else {
            pal.surface
        };
        let mark = Rect::from_min_size(
            pos2(row.left() + 18.0, mid - 6.0),
            Vec2::splat(size::STATE_MARK),
        );
        // Every file in a circle is marked: removing either direction ends it.
        if reference.internal || (link.circular && self.app.rules.no_circular_use) {
            draw::mark_bad(ui, mark, pal.bad, behind);
        } else {
            draw::mark_good(ui, mark, pal.good);
        }
        let mono = Type {
            size: 11.5,
            ..ty::CODE
        };
        let file = draw::galley_elided(
            ui,
            &reference.file,
            mono,
            pal.ink,
            line_right - 44.0 - file_x,
        );
        draw::paint_galley(ui, pos2(file_x, mid), Align2::LEFT_CENTER, file);
        draw::paint_text(
            ui,
            pos2(line_right, mid),
            Align2::RIGHT_CENTER,
            &reference.line.to_string(),
            mono,
            pal.ink,
        );
        let reads = draw::galley_elided(
            ui,
            &reference.class,
            mono,
            pal.ink,
            row.right() - 12.0 - reads_x,
        );
        draw::paint_galley(ui, pos2(reads_x, mid), Align2::LEFT_CENTER, reads);
        response.clicked()
    }

    // --- All modules (S5.3) ---

    fn all_modules(
        &self,
        ui: &mut Ui,
        y: f32,
        state: &mut BoundaryState,
        action: &mut Option<BoundaryAction>,
    ) -> f32 {
        if self.app.modules.is_empty() {
            return self.paragraph(ui, y, &strings::no_modules(self.app));
        }
        let two = self.width >= 720.0;
        let graph_w = if two {
            ((self.width - 14.0) * 0.6).floor()
        } else {
            self.width
        };
        let graph_bottom = self.app_graph(ui, pos2(self.left, y), graph_w, state, action);

        let side = if two {
            pos2(self.left + graph_w + 14.0, y)
        } else {
            pos2(self.left, graph_bottom + 12.0)
        };
        let side_w = if two {
            self.width - graph_w - 14.0
        } else {
            self.width
        };
        let broken_bottom = self.broken_links(ui, side, side_w, action);
        let rules_bottom = self.rules_box(
            ui,
            pos2(side.x, broken_bottom + 12.0),
            side_w,
            state,
            action,
        );
        graph_bottom.max(rules_bottom)
    }

    /// The graph of the whole app: modules in columns, a module left of what it uses.
    fn app_graph(
        &self,
        ui: &Ui,
        min: Pos2,
        width: f32,
        state: &mut BoundaryState,
        action: &mut Option<BoundaryAction>,
    ) -> f32 {
        let pal = self.pal;
        let rules = &self.app.rules;
        let names: Vec<&str> = self
            .app
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect();
        let layout = columns(&names, &self.app.links, &self.app.cycles);
        let tallest = layout.iter().map(Vec::len).max().unwrap_or(1) as f32;
        let (node_w, node_h, row_gap) = (
            132.0_f32.min((width - 44.0) / layout.len() as f32 - 24.0),
            30.0,
            46.0,
        );
        let area_h = tallest * node_h + (tallest - 1.0) * row_gap;
        let rect = Rect::from_min_size(min, vec2(width, 52.0 + area_h + 20.0 + 44.0));
        draw::outlined(ui, rect, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);

        // Head: how many links, and whether all of them are drawn.
        let title = strings::links_between(self.app.links.len(), self.app.modules.len());
        let title_type = ty::BODY.with_face(Face::Sans700);
        let title_rect = draw::paint_text(
            ui,
            pos2(rect.left() + 13.0, rect.top() + 23.0),
            Align2::LEFT_CENTER,
            &title,
            title_type,
            pal.ink,
        );
        draw::announce(ui, title_rect, Role::Heading, &title);
        let labels = [strings::ALL_LINKS, strings::PROBLEMS_ONLY];
        let selected = usize::from(state.problems_only);
        let control_w = 3.0
            + 3.0
            + 2.0
            + labels
                .iter()
                .map(|label| {
                    (draw::galley(
                        ui,
                        label,
                        Type {
                            size: 12.0,
                            line: 12.0,
                            ..ty::CONTROL
                        },
                        Color32::PLACEHOLDER,
                    )
                    .size()
                    .x + 20.0)
                        .ceil()
                })
                .sum::<f32>();
        let control = pos2(rect.right() - 13.0 - control_w, rect.top() + 10.0);
        if let (_, Some(clicked)) =
            draw::segmented(ui, pal, control, "links", &labels, selected, self.keyboard)
        {
            state.problems_only = clicked == 1;
        }

        // Place the nodes.
        let area_top = rect.top() + 52.0;
        let step = (width - 44.0 - node_w) / (layout.len() as f32 - 1.0).max(1.0);
        let mut nodes: Vec<Node<'_>> = Vec::new();
        for (c, column) in layout.iter().enumerate() {
            let height = column.len() as f32 * node_h + (column.len() as f32 - 1.0) * row_gap;
            let top = area_top + ((area_h - height) / 2.0).round();
            for (r, name) in column.iter().enumerate() {
                let Some(module) = self.app.module(name) else {
                    continue;
                };
                let node_min = pos2(
                    (rect.left() + 22.0 + c as f32 * step).round(),
                    top + r as f32 * (node_h + row_gap),
                );
                nodes.push(Node {
                    rect: Rect::from_min_size(node_min, vec2(node_w, node_h)),
                    module,
                });
            }
        }
        let place = |name: &str| {
            nodes
                .iter()
                .find(|node| node.module.name == name)
                .map(|node| node.rect)
        };

        // Links: lines of straight pieces, with an arrow at the module that is used.
        for link in &self.app.links {
            let problem = link.is_problem(rules);
            if state.problems_only && !problem {
                continue;
            }
            let (Some(from), Some(to)) = (place(&link.from), place(&link.to)) else {
                continue;
            };
            if (from.left() - to.left()).abs() < 1.0 {
                // Two modules of a circle, in one column: one line down, one line up.
                let down = from.top() < to.top();
                let x = from.center().x + if down { 22.0 } else { -22.0 };
                let (start, end) = if down {
                    (from.bottom(), to.top())
                } else {
                    (from.top(), to.bottom())
                };
                straight(ui, pal, pos2(x, start), pos2(x, end), problem);
            } else {
                // The line runs level from the module that uses, and turns in the gap
                // before the column of the module that is used. Lines to one module
                // from different rows turn at slightly different places.
                let lane = ((from.center().y - area_top) / (node_h + row_gap)).round() * 6.0;
                let bend = to.left() - 16.0 - lane.clamp(0.0, 30.0);
                connector_at(
                    ui,
                    pal,
                    pos2(from.right(), from.center().y),
                    pos2(to.left(), to.center().y),
                    bend,
                    problem,
                );
            }
        }

        for node in &nodes {
            let response = ui.interact(
                node.rect,
                ui.scope_id().with(("graph-node", &node.module.name)),
                Sense::click(),
            );
            response.widget_info(|| WidgetInfo::labeled(Role::Button, true, &node.module.name));
            let fill = if response.hovered() {
                pal.well
            } else {
                pal.surface
            };
            draw::outlined(ui, node.rect, 0, fill, pal.line_2);
            self.node_text(ui, node.rect, node.module, "", pal.ink_2, false);
            if response.clicked() {
                *action = Some(BoundaryAction::Select(node.module.name.clone()));
            }
        }

        let caption_type = Type {
            size: 11.0,
            line: 14.0,
            ..ty::CODE
        };
        let caption = draw::galley_wrapped(
            ui,
            strings::GRAPH_CAPTION,
            caption_type,
            pal.ink_2,
            width - 44.0,
        );
        let caption_rect = draw::paint_galley(
            ui,
            pos2(rect.left() + 22.0, rect.bottom() - 12.0),
            Align2::LEFT_BOTTOM,
            caption,
        );
        draw::announce(ui, caption_rect, Role::Label, strings::GRAPH_CAPTION);
        rect.bottom()
    }

    /// The links that break a rule: a circle once, and each link that reads internals.
    fn broken(&self) -> Vec<(String, usize, String, String)> {
        let rules = &self.app.rules;
        let mut rows = Vec::new();
        if rules.no_circular_use {
            for cycle in &self.app.cycles {
                let files = cycle.files.iter().map(|(_, files)| files).sum();
                let other = cycle
                    .modules
                    .iter()
                    .find(|member| **member != cycle.carrier)
                    .cloned()
                    .unwrap_or_default();
                let text = match cycle.modules.as_slice() {
                    [a, b] => strings::use_each_other(a, b),
                    modules => strings::use_each_other_in_a_circle(modules),
                };
                rows.push((text, files, cycle.carrier.clone(), other));
            }
        }
        if rules.public_surface_only {
            for link in self
                .app
                .links
                .iter()
                .filter(|link| link.internal_files() > 0)
            {
                let text = strings::reads_internals(&link.from, &link.to);
                rows.push((
                    text,
                    link.internal_files(),
                    link.from.clone(),
                    link.to.clone(),
                ));
            }
        }
        rows
    }

    fn broken_links(
        &self,
        ui: &Ui,
        min: Pos2,
        width: f32,
        action: &mut Option<BoundaryAction>,
    ) -> f32 {
        let pal = self.pal;
        let rows = self.broken();
        const ROW: f32 = 30.0;
        let rect = Rect::from_min_size(min, vec2(width, 35.0 + rows.len().max(1) as f32 * ROW));
        let mut y = draw::box_head(ui, pal, rect, strings::LINKS_THAT_BREAK, "") - 6.0;
        let mid = rect.top() + 18.0;
        if rows.is_empty() {
            draw::label(
                ui,
                pos2(rect.right() - 13.0, mid),
                Align2::RIGHT_CENTER,
                strings::GOOD,
                ty::LABEL,
                pal.good,
            );
            // Not in the spec: an app whose links break no rule.
            draw::label(
                ui,
                pos2(rect.left() + 14.0, y + ROW / 2.0),
                Align2::LEFT_CENTER,
                "No link breaks a rule.",
                ty::BODY,
                pal.ink_2,
            );
            return rect.bottom();
        }
        let count = strings::problems(rows.len());
        let text = draw::label(
            ui,
            pos2(rect.right() - 13.0, mid),
            Align2::RIGHT_CENTER,
            &count,
            ty::LABEL,
            pal.bad,
        );
        let mark = Rect::from_min_size(
            pos2(text.left() - 5.0 - 12.0, mid - 6.0),
            Vec2::splat(size::STATE_MARK),
        );
        draw::mark_bad(ui, mark, pal.bad, pal.surface);

        for (i, (text, files, module, other)) in rows.iter().enumerate() {
            let row = Rect::from_min_max(
                pos2(rect.left() + 1.0, y),
                pos2(rect.right() - 1.0, y + ROW),
            );
            let response = ui.interact(row, ui.scope_id().with(("broken-link", i)), Sense::click());
            response.widget_info(|| WidgetInfo::labeled(Role::Row, true, text));
            if response.hovered() {
                ui.painter()
                    .rect_filled(row.shrink2(vec2(0.0, 0.5)), 0, pal.well);
            }
            if response.clicked() {
                *action = Some(BoundaryAction::ShowLink {
                    module: module.clone(),
                    other: other.clone(),
                });
            }
            let files_text = strings::files(*files as u64);
            let files_type = Type {
                size: 11.5,
                ..ty::CODE
            };
            let files_rect = draw::paint_text(
                ui,
                pos2(row.right() - 10.0, row.center().y),
                Align2::RIGHT_CENTER,
                &files_text,
                files_type,
                pal.ink_2,
            );
            let text = draw::galley_elided(
                ui,
                text,
                ty::BODY,
                pal.ink,
                files_rect.left() - 12.0 - (row.left() + 13.0),
            );
            draw::paint_galley(
                ui,
                pos2(row.left() + 13.0, row.center().y),
                Align2::LEFT_CENTER,
                text,
            );
            if i + 1 < rows.len() {
                draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line);
            }
            y += ROW;
        }
        rect.bottom()
    }

    /// The two rules of the app, and the one that waits for the command line tool.
    fn rules_box(
        &self,
        ui: &mut Ui,
        min: Pos2,
        width: f32,
        state: &mut BoundaryState,
        action: &mut Option<BoundaryAction>,
    ) -> f32 {
        let pal = self.pal;
        let rules = &self.app.rules;
        // The folders field shows the folders of the rules until it is edited.
        if state.folders_from.as_ref() != Some(&rules.public_folders) {
            state.folders_text = rules.public_folders.join(", ");
            state.folders_from = Some(rules.public_folders.clone());
        }

        let indent = size::CHECKBOX + 8.0;
        let hint_w = width - 26.0 - indent;
        let accepted = strings::accepted_for(0, self.app.modules.len());
        let hint = draw::galley_wrapped(ui, &accepted, ty::SECONDARY, pal.ink_2, hint_w);
        let ci_hint = draw::galley_wrapped(
            ui,
            strings::RULE_FAIL_CI_HINT,
            ty::SECONDARY,
            pal.ink_2,
            hint_w,
        );
        let height = 41.0
            + 24.0
            + 4.0
            + 24.0
            + hint.size().y
            + 6.0
            + size::FIELD_H
            + 8.0
            + 24.0
            + ci_hint.size().y
            + 12.0;
        let rect = Rect::from_min_size(min, vec2(width, height));
        let mut y = draw::box_head(
            ui,
            pal,
            rect,
            &strings::rules_for(&self.app.name),
            strings::RULES_KEPT,
        );
        let x = rect.left() + 13.0;

        if draw::checkbox(
            ui,
            pal,
            pos2(x, y),
            strings::RULE_NO_CIRCULAR,
            rules.no_circular_use,
            false,
            self.keyboard,
        )
        .clicked()
        {
            *action = Some(BoundaryAction::SetRules(Rules {
                no_circular_use: !rules.no_circular_use,
                ..rules.clone()
            }));
        }
        y += 24.0 + 4.0;
        if draw::checkbox(
            ui,
            pal,
            pos2(x, y),
            strings::RULE_PUBLIC_SURFACE,
            rules.public_surface_only,
            false,
            self.keyboard,
        )
        .clicked()
        {
            *action = Some(BoundaryAction::SetRules(Rules {
                public_surface_only: !rules.public_surface_only,
                ..rules.clone()
            }));
        }
        y += 24.0;
        let hint_rect = draw::paint_galley(ui, pos2(x + indent, y), Align2::LEFT_TOP, hint);
        draw::announce(ui, hint_rect, Role::Label, &accepted);
        y = hint_rect.bottom() + 6.0;

        let field = Rect::from_min_size(pos2(x + indent, y), vec2(hint_w, size::FIELD_H));
        let response = draw::text_field(
            ui,
            pal,
            field,
            strings::PUBLIC_FOLDERS,
            &mut state.folders_text,
            true,
            self.keyboard,
        );
        if response.lost_focus() {
            let folders: Vec<String> = state
                .folders_text
                .split(',')
                .map(str::trim)
                .filter(|folder| !folder.is_empty())
                .map(str::to_owned)
                .collect();
            if folders != rules.public_folders {
                *action = Some(BoundaryAction::SetRules(Rules {
                    public_folders: folders,
                    ..rules.clone()
                }));
            }
        }
        y = field.bottom() + 8.0;

        draw::checkbox(
            ui,
            pal,
            pos2(x, y),
            strings::RULE_FAIL_CI,
            false,
            true,
            self.keyboard,
        );
        y += 24.0;
        let ci_rect = draw::paint_galley(ui, pos2(x + indent, y), Align2::LEFT_TOP, ci_hint);
        draw::announce(ui, ci_rect, Role::Label, strings::RULE_FAIL_CI_HINT);
        rect.bottom()
    }

    // --- An app with many modules (S5.4) ---

    fn links_table(
        &self,
        ui: &mut Ui,
        y: f32,
        state: &mut BoundaryState,
        action: &mut Option<BoundaryAction>,
    ) -> f32 {
        let pal = self.pal;
        let rules = &self.app.rules;

        // Problems first, then the links with the most references.
        let wanted = state.filter.trim().to_lowercase();
        let mut links: Vec<&Link> = self
            .app
            .links
            .iter()
            .filter(|link| !state.problems_only || link.is_problem(rules))
            .filter(|link| {
                wanted.is_empty()
                    || [&link.from, &link.to].iter().any(|name| {
                        name.to_lowercase().contains(&wanted)
                            || self.code(name).to_lowercase().contains(&wanted)
                    })
            })
            .collect();
        links.sort_by(|a, b| {
            b.is_problem(rules)
                .cmp(&a.is_problem(rules))
                .then_with(|| b.refs.len().cmp(&a.refs.len()))
                .then_with(|| (&a.from, &a.to).cmp(&(&b.from, &b.to)))
        });

        // Above the table: the filter, the choice of links, and how many are shown.
        let field = Rect::from_min_size(
            pos2(self.left, y),
            vec2(240.0_f32.min(self.width * 0.4), size::CONTROL_H),
        );
        draw::text_field(
            ui,
            pal,
            field,
            strings::FILTER_LINKS,
            &mut state.filter,
            false,
            self.keyboard,
        );
        if state.filter.is_empty() {
            draw::paint_text(
                ui,
                pos2(field.left() + 9.0, field.center().y),
                Align2::LEFT_CENTER,
                strings::FILTER_LINKS,
                ty::BODY,
                pal.ink_3,
            );
        }
        let labels = [strings::ALL_LINKS, strings::PROBLEMS_ONLY];
        let (control, clicked) = draw::segmented(
            ui,
            pal,
            pos2(field.right() + 8.0, y),
            "links",
            &labels,
            usize::from(state.problems_only),
            self.keyboard,
        );
        if let Some(clicked) = clicked {
            state.problems_only = clicked == 1;
        }
        let problems = self.app.problem_links().count();
        let shown = strings::links_shown(self.app.links.len(), problems, links.len());
        draw::label(
            ui,
            pos2(control.right() + 12.0, control.center().y),
            Align2::LEFT_CENTER,
            &shown,
            ty::SECONDARY,
            pal.ink_2,
        );
        let top = y + size::CONTROL_H + 12.0;

        let table = Rect::from_min_size(
            pos2(self.left, top),
            vec2(
                self.width,
                size::TABLE_HEAD_H + links.len() as f32 * size::TABLE_ROW_H + 1.0,
            ),
        );
        draw::outlined(ui, table, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
        let from_x = table.left() + 14.0;
        let uses_x = (table.left() + table.width() * 0.30).round();
        let state_x = (table.left() + table.width() * 0.76).round();
        let files_right = state_x - 24.0;
        let refs_right = files_right - 64.0;
        let head_mid = table.top() + size::TABLE_HEAD_H / 2.0;
        for (x, align, label) in [
            (from_x, Align2::LEFT_CENTER, strings::COL_FROM),
            (uses_x, Align2::LEFT_CENTER, strings::COL_USES),
            (refs_right, Align2::RIGHT_CENTER, strings::COL_REFS),
            (files_right, Align2::RIGHT_CENTER, strings::COL_FILES),
            (state_x, Align2::LEFT_CENTER, strings::COL_STATE),
        ] {
            draw::label(ui, pos2(x, head_mid), align, label, ty::LABEL, pal.ink_2);
        }
        draw::hline(
            ui,
            table.left() + 1.0..table.right() - 1.0,
            table.top() + size::TABLE_HEAD_H - 1.0,
            pal.line_2,
        );

        for (i, link) in links.iter().enumerate() {
            let row_top = table.top() + size::TABLE_HEAD_H + i as f32 * size::TABLE_ROW_H;
            let row = Rect::from_min_max(
                pos2(table.left() + 1.0, row_top),
                pos2(table.right() - 1.0, row_top + size::TABLE_ROW_H),
            );
            let name = strings::a_uses_b(&link.from, &link.to);
            let response = ui.interact(
                row,
                ui.scope_id().with(("link-row", &link.from, &link.to)),
                Sense::click(),
            );
            response.widget_info(|| WidgetInfo::labeled(Role::Row, true, &name));
            if response.hovered() {
                ui.painter().rect_filled(row, 0, pal.well);
            }
            if response.clicked() {
                *action = Some(BoundaryAction::ShowLink {
                    module: link.from.clone(),
                    other: link.to.clone(),
                });
            }
            if i + 1 < links.len() {
                draw::hline(ui, row.left()..row.right(), row.bottom() - 1.0, pal.line);
            }

            let mid = row.center().y;
            for (x, module) in [(from_x, &link.from), (uses_x, &link.to)] {
                let stamp_min = pos2(x, (mid - size::STAMP_H / 2.0).round());
                let state = StampState {
                    selected: false,
                    off: self.is_off(module),
                };
                let stamp = draw::stamp(ui, pal, stamp_min, self.code(module), state);
                draw::paint_text(
                    ui,
                    pos2(stamp.right() + 8.0, mid),
                    Align2::LEFT_CENTER,
                    module,
                    ty::BODY.with_face(Face::Sans600),
                    pal.ink,
                );
            }
            draw::paint_text(
                ui,
                pos2(refs_right, mid),
                Align2::RIGHT_CENTER,
                &link.refs.len().to_string(),
                ty::BODY,
                pal.ink,
            );
            draw::paint_text(
                ui,
                pos2(files_right, mid),
                Align2::RIGHT_CENTER,
                &link.files().to_string(),
                ty::BODY,
                pal.ink,
            );

            let internal = link.internal_files();
            let (word, color, bad) = if rules.no_circular_use && link.circular {
                (strings::CIRCULAR.to_owned(), pal.bad, true)
            } else if rules.public_surface_only && internal > 0 {
                (strings::n_files_read_internals(internal), pal.bad, true)
            } else {
                (strings::GOOD.to_owned(), pal.good, false)
            };
            let mark = Rect::from_min_size(pos2(state_x, mid - 6.0), Vec2::splat(size::STATE_MARK));
            if bad {
                draw::mark_bad(
                    ui,
                    mark,
                    color,
                    if response.hovered() {
                        pal.well
                    } else {
                        pal.surface
                    },
                );
            } else {
                draw::mark_good(ui, mark, color);
            }
            draw::paint_text(
                ui,
                pos2(mark.right() + 5.0, mid),
                Align2::LEFT_CENTER,
                &word,
                ty::LABEL,
                color,
            );
        }

        let note = strings::many_modules(&self.app.name, self.app.modules.len());
        let note_bottom = self.note(ui, table.bottom() + 12.0, &note);
        // The rules, under the table.
        self.rules_box(
            ui,
            pos2(self.left, note_bottom + 12.0),
            self.width.min(560.0),
            state,
            action,
        )
    }
}

/// A connector of straight pieces from `from` to `to`: out, across at the middle,
/// in, with an arrow at `to`. A link that breaks a rule is 2 px in the problem
/// colour and carries the problem mark.
fn connector(ui: &Ui, pal: &Palette, from: Pos2, to: Pos2, problem: bool) {
    connector_at(ui, pal, from, to, ((from.x + to.x) / 2.0).round(), problem);
}

/// A connector that turns at the x of `bend`.
fn connector_at(ui: &Ui, pal: &Palette, from: Pos2, to: Pos2, bend: f32, problem: bool) {
    let (color, width) = line_of(pal, problem);
    let points = vec![
        from,
        pos2(bend, from.y),
        pos2(bend, to.y),
        pos2(to.x - 6.0, to.y),
    ];
    ui.painter()
        .add(Shape::line(points, Stroke::new(width, color)));
    arrow(ui, to, vec2(1.0, 0.0), color);
    if problem {
        let at = if (from.y - to.y).abs() > 14.0 {
            pos2(bend, (from.y + to.y) / 2.0)
        } else {
            pos2(bend, to.y)
        };
        problem_mark(ui, pal, at);
    }
}

/// A straight vertical connector, for two modules in one column.
fn straight(ui: &Ui, pal: &Palette, from: Pos2, to: Pos2, problem: bool) {
    let (color, width) = line_of(pal, problem);
    let direction = if to.y > from.y { 1.0 } else { -1.0 };
    ui.painter().line_segment(
        [from, pos2(to.x, to.y - 6.0 * direction)],
        Stroke::new(width, color),
    );
    arrow(ui, to, vec2(0.0, direction), color);
    if problem {
        problem_mark(ui, pal, pos2(from.x, (from.y + to.y) / 2.0));
    }
}

fn line_of(pal: &Palette, problem: bool) -> (Color32, f32) {
    if problem {
        (pal.bad, stroke::PROBLEM_LINK)
    } else {
        (pal.ink_2, stroke::HAIRLINE)
    }
}

/// A filled arrow head whose tip is at `tip`, pointing along `direction`.
fn arrow(ui: &Ui, tip: Pos2, direction: Vec2, color: Color32) {
    let back = tip - direction * 7.0;
    let side = vec2(-direction.y, direction.x) * 3.5;
    ui.painter().add(Shape::convex_polygon(
        vec![tip, back + side, back - side],
        color,
        Stroke::NONE,
    ));
}

/// The problem mark on a line, with the surface colour around it so that the line stops at it.
fn problem_mark(ui: &Ui, pal: &Palette, center: Pos2) {
    let center = pos2(center.x.round(), center.y.round());
    ui.painter().rect_filled(
        Rect::from_center_size(center, Vec2::splat(14.0)),
        0,
        pal.surface,
    );
    draw::mark_bad(
        ui,
        Rect::from_center_size(center, Vec2::splat(size::STATE_MARK)),
        pal.bad,
        pal.surface,
    );
}
