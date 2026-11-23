//! Apps: open, add, select, move and remove an app, and the add panel.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui;
use mortise_core::{
    LiveState, ReadCache, ReadOptions, RunnerKind, propose_monogram, read_app_cached,
};

use super::{MortiseApp, Reading, open_app};
use crate::actions::view::ActionsState;
use crate::apps::view::{AddAction, AddPanel, Checked};
use crate::index::view::Selection;
use crate::strings;
use crate::ui::toast::Toast;

impl MortiseApp {
    // --- Apps ---

    /// Shows an app. Only the app on screen is watched for changes; it is read again
    /// when it comes on screen.
    pub(super) fn select_app(&mut self, index: usize) {
        if index >= self.apps.len() {
            return;
        }
        if let Some(previous) = self.current.filter(|previous| *previous != index) {
            self.apps[previous].live.watch(false);
            self.apps[index].live.refresh();
            // Another app: its Actions tab starts again on Migrate (design D4).
            self.apps[index].actions = ActionsState::default();
        }
        self.apps[index].live.watch(true);
        self.current = Some(index);
        self.adding = None;
        if self.settings.current != Some(index) {
            self.settings.current = Some(index);
            self.save();
        }
    }

    /// Reads a folder for the add panel, on another thread. A folder that is read
    /// in a moment gives its result at once; a larger one gives `None`, and the
    /// result arrives in a later frame.
    fn check(&mut self, ctx: &egui::Context, folder: &Path) -> Option<Checked> {
        self.reading = None;
        if let Some(index) = self.settings.position(folder) {
            return Some(Checked::Known {
                index,
                name: self.settings.apps[index].display_name().to_owned(),
            });
        }
        let mut cache = ReadCache::default();
        let progress = cache.progress();
        let (sender, result) = std::sync::mpsc::channel();
        let (root, ctx) = (folder.to_owned(), ctx.clone());
        std::thread::Builder::new()
            .name("mortise-add-reader".to_owned())
            .spawn(move || {
                let read =
                    read_app_cached(&root, &BTreeMap::new(), &ReadOptions::default(), &mut cache);
                let _ = sender.send(match read {
                    Ok(app) => Checked::Found(Box::new(app)),
                    Err(error) => Checked::Failed(error),
                });
                ctx.request_repaint();
            })
            .expect("a thread can be started");
        match result.recv_timeout(self.quick_read) {
            Ok(checked) => Some(checked),
            Err(_) => {
                self.reading = Some(Reading {
                    result,
                    progress,
                    since: std::time::Instant::now(),
                });
                None
            }
        }
    }

    /// Opens the add panel on a folder that was chosen or dropped.
    pub(super) fn open_folder(&mut self, ctx: &egui::Context, folder: &Path) {
        let path = strings::tilde(folder);
        self.adding = Some(match self.check(ctx, folder) {
            Some(checked) => AddPanel::checked(path, checked),
            None => AddPanel::reading(path),
        });
    }

    /// Takes in the folder of the add panel once it is read.
    pub(super) fn take_in_folder(&mut self, ctx: &egui::Context) {
        let Some(reading) = &self.reading else {
            return;
        };
        match reading.result.try_recv() {
            Ok(checked) => {
                self.reading = None;
                if let Some(panel) = self.adding.as_mut().filter(|panel| panel.reading) {
                    panel.set_checked(checked);
                }
            }
            // The bar moves while the files are read.
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.reading = None,
        }
        if self.adding.is_none() {
            self.reading = None;
        }
    }

    /// Adds the app the panel found, shows it and says so.
    fn add_found(&mut self, ctx: &egui::Context) {
        let Some(AddPanel {
            checked: Some(Checked::Found(app)),
            runtime,
            php,
            php_test,
            ..
        }) = self.adding.take()
        else {
            return;
        };
        let index = self.settings.add(&app);
        // Add saves the runner the field shows (design D8), read from its text in
        // case the field still has focus.
        if let Some(form) = &php {
            self.settings.apps[index].php.runner = Some(RunnerKind::from_line(&form.line));
        }
        // Runtime is optional: without a base URL the app is added without it.
        if let Some(form) = runtime.filter(|form| !form.base_url.trim().is_empty()) {
            let environment = form.environment();
            if environment.credentials.needs_secret() && !form.secret.is_empty() {
                self.secrets.set(&app.root, &environment.name, &form.secret);
            }
            let entry = &mut self.settings.apps[index];
            entry.environment = Some(environment.name.clone());
            entry.environments.push(environment);
        }
        let mut open = open_app(ctx, &self.settings.apps[index], self.data_dir.as_deref());
        // The test is kept in memory, as Settings shows it. One that runs still lands.
        open.php_test = php_test;
        self.apps.push(open);
        self.settings_open = None;
        self.select_app(index);
        self.save();
        self.toast = Some(Toast::done(strings::added(
            self.settings.apps[index].display_name(),
            app.modules.len(),
        )));
        // Focus moves to the "All modules" row of the new app.
        self.keyboard = true;
    }

    /// The monogram the app in the add panel would get.
    pub(super) fn monogram_preview(&self) -> String {
        let taken: BTreeSet<String> = self
            .settings
            .apps
            .iter()
            .map(|app| app.monogram.clone())
            .collect();
        match &self.adding {
            Some(AddPanel {
                checked: Some(Checked::Found(app)),
                ..
            }) => propose_monogram(&app.name, &taken),
            _ => String::new(),
        }
    }

    /// Takes in a newer snapshot of the current app: keeps the codes it gave, and
    /// drops a selection whose module is gone.
    pub(super) fn take_in(&mut self) {
        let Some(current) = self.current else {
            return;
        };
        let open = &mut self.apps[current];
        let LiveState::Ready(app) = open.live.state() else {
            return;
        };
        if open
            .seen
            .as_ref()
            .is_some_and(|seen| Arc::ptr_eq(seen, &app))
        {
            return;
        }
        if let Selection::Module(name) = &open.selection
            && !app.modules.iter().any(|module| &module.name == name)
        {
            open.selection = Selection::All;
        }
        open.seen = Some(Arc::clone(&app));

        let entry = &mut self.settings.apps[current];
        let before = entry.codes.len();
        entry.codes.extend(app.codes());
        if entry.codes.len() != before {
            self.save();
        }
    }

    /// Switches the index of the current app between by name and problems first.
    pub(super) fn toggle_order(&mut self) {
        if let Some(current) = self.current {
            let entry = &mut self.settings.apps[current];
            entry.problems_first = !entry.problems_first;
            self.save();
        }
    }

    /// Moves an app to another place on the rail. The app on screen stays on screen.
    pub(super) fn move_app(&mut self, from: usize, to: usize) {
        if from >= self.apps.len() || to >= self.apps.len() || from == to {
            return;
        }
        let entry = self.settings.apps.remove(from);
        self.settings.apps.insert(to, entry);
        let open = self.apps.remove(from);
        self.apps.insert(to, open);
        // Where a place that was held before the move is now.
        let moved = |place: usize| {
            if place == from {
                to
            } else if from < place && place <= to {
                place - 1
            } else if to <= place && place < from {
                place + 1
            } else {
                place
            }
        };
        self.current = self.current.map(moved);
        self.palette_scope = moved(self.palette_scope);
        self.settings.current = self.current;
        if let Some(state) = &mut self.settings_open {
            state.app = moved(state.app);
        }
        self.save();
    }

    /// Forgets an app. Its folder and its files stay as they are.
    pub(super) fn remove_app(&mut self, index: usize) {
        if index >= self.apps.len() {
            return;
        }
        let was_current = self.current == Some(index);
        // Mortise forgets the app, its environments and its saved credentials.
        let removed = self.settings.remove(index);
        for environment in &removed.environments {
            self.secrets.remove(&removed.root, &environment.name);
        }
        self.apps.remove(index);
        self.current = match self.current {
            Some(current) if current > index => Some(current - 1),
            Some(current) if current == index => None,
            other => other,
        };
        if was_current && !self.apps.is_empty() {
            self.select_app(0);
        }
        self.settings.current = self.current;
        self.save();
        if let Some(state) = &mut self.settings_open {
            state.app = 0;
        }
    }

    /// Opens the folder dialog and, if a folder is chosen, the add panel on it.
    pub(super) fn pick(&mut self, ctx: &egui::Context) {
        if let Some(folder) = (self.pick_folder)() {
            self.open_folder(ctx, &folder);
            ctx.request_repaint();
        }
    }

    pub(super) fn add_action(&mut self, ctx: &egui::Context, action: AddAction) {
        match action {
            AddAction::Back => self.adding = None,
            AddAction::Pick => self.pick(ctx),
            AddAction::Check => {
                let Some(path) = self.adding.as_ref().map(|panel| expand(panel.path.trim())) else {
                    return;
                };
                if let Some(panel) = &mut self.adding {
                    panel.start_reading();
                }
                let checked = self.check(ctx, &path);
                if let (Some(panel), Some(checked)) = (&mut self.adding, checked) {
                    panel.set_checked(checked);
                }
            }
            // What was found is used only while it belongs to the path in the field.
            AddAction::Add | AddAction::Open(_) | AddAction::Test | AddAction::TestPhp
                if !self.adding.as_ref().is_some_and(AddPanel::is_current) => {}
            AddAction::Add => self.add_found(ctx),
            AddAction::Open(index) => {
                self.settings_open = None;
                self.select_app(index);
            }
            AddAction::CopyCommand => {
                ctx.copy_text(strings::REQUIRE_PACKAGE.to_owned());
                self.toast = Some(Toast::done(strings::COPIED_COMMAND));
            }
            AddAction::Test => {
                if let Some(form) = self
                    .adding
                    .as_mut()
                    .and_then(|panel| panel.runtime.as_mut())
                {
                    let secret = (!form.secret.is_empty()).then(|| form.secret.clone());
                    form.start_test(secret, ctx);
                }
            }
            // The add panel has no working folder: the test starts in the app folder.
            AddAction::TestPhp => {
                if let Some(AddPanel {
                    checked: Some(Checked::Found(app)),
                    php: Some(form),
                    php_test,
                    ..
                }) = &mut self.adding
                {
                    php_test.start(&form.runner, app.root.clone(), ctx);
                }
            }
        }
    }
}

/// A path as typed: `~` at its start is the home folder.
fn expand(path: &str) -> PathBuf {
    match (path.strip_prefix('~'), std::env::home_dir()) {
        (Some(rest), Some(home)) if rest.is_empty() || rest.starts_with(['/', '\\']) => {
            home.join(rest.trim_start_matches(['/', '\\']))
        }
        _ => PathBuf::from(path),
    }
}
