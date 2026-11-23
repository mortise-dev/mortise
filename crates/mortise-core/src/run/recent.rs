//! The actions and generators that ran last in an app, read from its history (S12.2 a).

use crate::run::history::Entry;
use crate::{Action, AppSnapshot, Catalog, ModuleState, Target};

/// What a run of the history was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ran {
    /// An action on a module, or on all modules with `None`.
    Action {
        action: Action,
        module: Option<String>,
    },
    /// A generator in a module, or `module:make` with `None`.
    Generator {
        name: String,
        module: Option<String>,
    },
}

/// A run that was an action or a generator, and when it started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentRun {
    pub ran: Ran,
    /// In ms since 1970.
    pub started_ms: i64,
}

/// The distinct actions and generators of `runs`, which hold the latest first: at most
/// `limit`, the latest first. A run is the action of [`Action::from_artisan`] or a generator of
/// `catalog`, by its first word, with the module of its words. A run that is neither,
/// whose module is not in `app`, or whose action the Actions tab does not list for the
/// module as it is now (Turn off of a module that is off), is left out. Of two runs of the same action or
/// generator on the same module, the latest stays.
pub fn recent_runs(
    runs: &[Entry],
    catalog: &Catalog,
    app: &AppSnapshot,
    limit: usize,
) -> Vec<RecentRun> {
    let mut recent: Vec<RecentRun> = Vec::new();
    for run in runs {
        if recent.len() == limit {
            break;
        }
        let Some(ran) = ran(&run.args, catalog, app) else {
            continue;
        };
        if recent.iter().all(|kept| kept.ran != ran) {
            recent.push(RecentRun {
                ran,
                started_ms: run.started_ms,
            });
        }
    }
    recent
}

/// What the words of a run were. The words that are not options are the arguments:
/// the words after `--` when there is one, as [`crate::plan`] and
/// [`crate::Generator::words`] write a name that starts with `-`.
fn ran(args: &[String], catalog: &Catalog, app: &AppSnapshot) -> Option<Ran> {
    let (first, rest) = args.split_first()?;
    let options: Vec<&str> = rest
        .iter()
        .map(String::as_str)
        .take_while(|word| *word != "--")
        .collect();
    let arguments: Vec<&str> = if options.len() < rest.len() {
        rest[options.len() + 1..]
            .iter()
            .map(String::as_str)
            .collect()
    } else {
        options
            .iter()
            .copied()
            .filter(|word| !word.starts_with('-'))
            .collect()
    };
    // The name the snapshot has for a module of the words.
    let module = |word: Option<&&str>| Some(app.module(word?)?.name.clone());

    if let Some(action) = Action::from_artisan(first) {
        let module = if options.contains(&"--all") {
            None
        } else {
            Some(module(arguments.first())?)
        };
        // The actions the Actions tab lists for the target: the everyday ones for the
        // state of the module, and the destructive ones of the target.
        let everyday = match &module {
            Some(name) => {
                let on = app.module(name)?.state != ModuleState::Off;
                Action::for_module(on).contains(&action)
            }
            None => Action::for_all().contains(&action),
        };
        let target = module.clone().map_or(Target::All, Target::Module);
        if !everyday && !Action::destructive(&target).contains(&action) {
            return None;
        }
        return Some(Ran::Action { action, module });
    }
    let generator = catalog.get(first)?;
    let module = match generator
        .arguments
        .iter()
        .position(|argument| argument.name == "module")
    {
        Some(at) => Some(module(arguments.get(at))?),
        None => None,
    };
    Some(Ran::Generator {
        name: generator.name.clone(),
        module,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::history::Ended;
    use std::path::Path;

    fn run(words: &str, started_ms: i64) -> Entry {
        let args: Vec<String> = words.split_whitespace().map(str::to_owned).collect();
        Entry {
            id: started_ms as u64,
            label: words.to_owned(),
            args,
            started_ms,
            ended: Ended::Done { ms: 5 },
            cut: false,
        }
    }

    #[test]
    fn the_latest_distinct_actions_and_generators_of_modules_in_the_app() {
        let root = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/shop-api"
        ));
        let app = crate::read_app(root).unwrap();
        let runs = [
            run("module:migrate Checkout --pretend", 9),
            // Not an action or a generator, and a module the app does not have.
            run("about", 8),
            run("module:migrate Nowhere", 7),
            // Checkout is on: the Actions tab offers Turn off, not Turn on.
            run("module:enable Checkout", 7),
            // A generator without its module runs in the module the package uses.
            run("module:make-model Invoice", 6),
            // The same action on the same module: the latest stays.
            run("module:migrate Checkout", 5),
            run("module:make-model --migration -- -Invoice Billing", 4),
            run("module:migrate --all", 3),
            // Past the limit.
            run("module:make Loyalty --api", 2),
        ];
        let recent = recent_runs(&runs, Catalog::v13_3(), &app, 3);
        assert_eq!(
            recent,
            [
                RecentRun {
                    ran: Ran::Action {
                        action: Action::Migrate,
                        module: Some("Checkout".into()),
                    },
                    started_ms: 9,
                },
                RecentRun {
                    ran: Ran::Generator {
                        name: "module:make-model".into(),
                        module: Some("Billing".into()),
                    },
                    started_ms: 4,
                },
                RecentRun {
                    ran: Ran::Action {
                        action: Action::Migrate,
                        module: None,
                    },
                    started_ms: 3,
                },
            ]
        );
        // `module:make` has no module.
        assert_eq!(
            recent_runs(&runs[runs.len() - 1..], Catalog::v13_3(), &app, 3)[0].ran,
            Ran::Generator {
                name: crate::MODULE_MAKE.into(),
                module: None,
            }
        );
    }
}
