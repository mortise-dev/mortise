//! Runtime: the chosen environment of the app on screen, and what its box asks for.

use eframe::egui;
use std::sync::Arc;

use mortise_core::{LiveMetrics, LiveRuntime, LiveState};

use super::{Connected, MortiseApp};
use crate::runtime::view::{Brief, RuntimeAction};
use crate::settings::view::Section;
use crate::ui::toast::Toast;

impl MortiseApp {
    /// Keeps the runtime connection in step with the settings: the app on screen is
    /// asked through its chosen environment, and no other app is asked.
    pub(super) fn sync_runtime(&mut self, ctx: &egui::Context) {
        for (index, (open, entry)) in self.apps.iter_mut().zip(&self.settings.apps).enumerate() {
            let installed =
                matches!(open.live.state(), LiveState::Ready(app) if app.actuator.is_some());
            let wanted = entry
                .chosen_environment()
                .filter(|_| self.current == Some(index) && installed);
            let Some(environment) = wanted else {
                open.runtime = None;
                continue;
            };
            let secret = if environment.credentials.needs_secret() {
                self.secrets.get(&entry.root, &environment.name)
            } else {
                None
            };
            let same = open.runtime.as_ref().is_some_and(|connected| {
                &connected.environment == environment && connected.secret == secret
            });
            if !same {
                let ctx = ctx.clone();
                let metrics = match &self.metrics_clock {
                    Some(clock) => LiveMetrics::with_clock(Arc::clone(clock)),
                    None => LiveMetrics::new(),
                };
                let live = LiveRuntime::open_with_metrics(
                    environment.clone(),
                    secret.clone(),
                    self.pace,
                    move || ctx.request_repaint(),
                    metrics.sink(),
                );
                open.runtime = Some(Connected {
                    environment: environment.clone(),
                    secret,
                    live,
                    metrics,
                    figures: None,
                });
            }
            if let Some(connected) = &open.runtime {
                // The window is visible: this frame is drawn.
                connected.live.seen();
                if let Some(answer) = connected.live.state().answer {
                    open.last_answers
                        .insert(environment.name.clone(), Brief::of_answer(&answer));
                }
            }
        }
    }

    /// What the runtime box of the current app asked for.
    pub(super) fn runtime_action(&mut self, ctx: &egui::Context, action: RuntimeAction) {
        let Some(current) = self.current else {
            return;
        };
        match action {
            RuntimeAction::Choose(name) => {
                self.settings.apps[current].environment = Some(name);
                self.save();
            }
            RuntimeAction::TryAgain => {
                if let Some(connected) = &self.apps[current].runtime {
                    connected.live.refresh();
                }
            }
            RuntimeAction::AddEnvironment => {
                self.open_settings(Section::Apps);
                if let Some(state) = &mut self.settings_open {
                    state.add_environment = true;
                }
            }
            RuntimeAction::OpenSettings => {
                self.open_settings(Section::Apps);
                let entry = &self.settings.apps[current];
                let chosen = entry.chosen_environment().map(|chosen| &chosen.name);
                let place = entry
                    .environments
                    .iter()
                    .position(|e| Some(&e.name) == chosen);
                if let Some(state) = &mut self.settings_open {
                    state.environment = place.unwrap_or(0);
                }
            }
            RuntimeAction::Copy { text, said } => {
                ctx.copy_text(text.to_owned());
                self.toast = Some(Toast::done(said));
            }
        }
    }
}
