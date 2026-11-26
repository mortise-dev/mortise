//! Accepting a public surface (S5.7 to S5.9): the package command `module:surface`,
//! shown first, then read for its answer. Mortise writes no `module.json` itself.

use mortise_core::{AcceptedSurface, DataError, ReadEnd, RunEnd, StartError, SurfaceAnswer};

use super::{MortiseApp, Pending, ReadFor};
use crate::boundary::strings;
use crate::runner::view::line_now;
use crate::ui::dialog::Dialog;
use crate::ui::toast::Toast;

impl MortiseApp {
    /// Shows the command that stores `surface` as the accepted surface of `module`,
    /// with its exact line, before it runs.
    pub(super) fn ask_accept_surface(
        &mut self,
        index: usize,
        module: String,
        surface: AcceptedSurface,
    ) {
        let (Some(entry), Some(app)) = (self.settings.apps.get(index), self.snapshot(index)) else {
            return;
        };
        let Some(found) = app.module(&module) else {
            return;
        };
        let args = mortise_core::surface_args(&module);
        // The view offers Accept only with a runner, so the line is there.
        let Some(line) = line_now(entry.php.runner.as_ref(), &args) else {
            return;
        };
        let manifest = app.path_shown(&found.dir.join("module.json"));
        let dialog = Dialog {
            code: Some(line),
            lines: vec![strings::accept_input(surface.entries())],
            ..Dialog::new(
                strings::accept_title(&module),
                strings::accept_body(&manifest),
                strings::accept_entries(surface.entries()),
            )
        };
        self.dialog = Some((
            dialog,
            Pending::AcceptSurface {
                index,
                module,
                surface,
            },
        ));
        self.dialog_opened = true;
    }

    /// Starts `module:surface <module> --set --stdin --json` with the surface on its
    /// standard input.
    pub(super) fn accept_surface(
        &mut self,
        index: usize,
        module: String,
        surface: AcceptedSurface,
    ) {
        let args = mortise_core::surface_args(&module);
        let input = surface.to_stdin().into_bytes();
        self.start_read(
            index,
            args,
            Some(input),
            ReadFor::Surface { module, surface },
        );
    }

    /// What `module:surface` answered. Stored: a toast, and the app is read again, so
    /// that the rule uses the new surface at once. Otherwise a toast says why.
    ///
    /// The package writes `module.json` before it prints its answer. An answer cut at
    /// the line limit of a command that ended with 0 counts as stored when
    /// `module.json` holds the surface that was sent.
    pub(super) fn surface_read(
        &mut self,
        index: usize,
        module: &str,
        surface: &AcceptedSurface,
        end: Result<ReadEnd, StartError>,
    ) {
        let reason = match end {
            Err(error) => crate::strings::start_reason(&error),
            Ok(end) => match end.document::<SurfaceAnswer>() {
                Ok(answer) if answer.errors.is_empty() && answer.surface.is_some() => {
                    self.surface_stored(index, module, surface.entries());
                    return;
                }
                Err(DataError::Cut)
                    if matches!(end.end, RunEnd::Exited { code: Some(0), .. })
                        && self.holds(index, module, surface) =>
                {
                    self.surface_stored(index, module, surface.entries());
                    return;
                }
                Ok(answer) => {
                    let messages: Vec<&str> = answer
                        .errors
                        .iter()
                        .map(|problem| problem.message.as_str())
                        .collect();
                    messages.join(" ")
                }
                Err(_) if lacks_the_command(&end) => strings::NO_SURFACE_COMMAND.to_owned(),
                Err(error) => crate::strings::no_document(&end, &error),
            },
        };
        self.toast = Some(Toast::failed(strings::surface_not_stored(module, &reason)));
    }

    /// The surface of `module` is stored: the view starts again from it.
    fn surface_stored(&mut self, index: usize, module: &str, entries: usize) {
        if let Some(open) = self.apps.get_mut(index) {
            open.boundary.surfaces.remove(module);
        }
        self.set_package(index, |_| {});
        self.toast = Some(Toast::done(strings::accepted_surface(module, entries)));
    }

    /// True when the `module.json` of `module` holds `surface` now.
    fn holds(&self, index: usize, module: &str, surface: &AcceptedSurface) -> bool {
        let Some(app) = self.snapshot(index) else {
            return false;
        };
        app.module(module)
            .and_then(|found| AcceptedSurface::of_file(&found.dir.join("module.json")))
            .is_some_and(|stored| stored == surface.clone().normalized())
    }
}

/// True when artisan says that it has no `module:surface`: a package before 13.5.
fn lacks_the_command(end: &ReadEnd) -> bool {
    end.stdout
        .iter()
        .chain(&end.stderr)
        .any(|line| line.contains("\"module:surface\" is not defined"))
}
