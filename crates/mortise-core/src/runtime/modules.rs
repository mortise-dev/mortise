//! The runtime status of each module, from the health components of an environment.
//!
//! `nwidart/laravel-modules` 13.4 names the indicators of a module
//! `module:{lower case module name}:{key}`, and adds a component `modules` that lists each
//! module and gives an `error` for one that is enabled but missing on disk. Whether an
//! environment has them is read from what it answers, not from `composer.lock`.

use serde_json::Value;

use crate::app::AppSnapshot;
use crate::runtime::actuator::{Component, Status};

/// The name of the component that lists the modules.
const MODULES: &str = "modules";

/// What the running app reports about one module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleRuntime {
    /// The name of the module, as the app's files give it.
    pub module: String,
    /// The worst status of its indicators. Down when `error` is there.
    pub status: Status,
    /// What the `modules` component says is wrong with the module, as the app wrote it.
    pub error: Option<String>,
    /// The module's own indicators, each named by its key: `storage` for
    /// `module:blog:storage`. In the app's order.
    pub indicators: Vec<Component>,
}

/// The components of an answer, split between the app and its modules.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeByModule {
    /// The components of the app itself, in the app's order.
    pub app: Vec<Component>,
    /// The modules that have a status, in the order of the app's modules.
    pub modules: Vec<ModuleRuntime>,
}

impl RuntimeByModule {
    /// The runtime of a module, by its name. `None` when it has no status.
    pub fn of(&self, module: &str) -> Option<&ModuleRuntime> {
        self.modules.iter().find(|runtime| runtime.module == module)
    }
}

/// How bad a status is: Down, Out of service, Up, Unknown, as Spring orders them.
fn rank(status: Status) -> u8 {
    match status {
        Status::Down => 0,
        Status::OutOfService => 1,
        Status::Up => 2,
        Status::Unknown => 3,
    }
}

/// The module and the key of a module indicator: `("blog", "storage")` for
/// `module:blog:storage`.
fn indicator_name(name: &str) -> Option<(&str, &str)> {
    let (module, key) = name.strip_prefix("module:")?.split_once(':')?;
    (!module.is_empty() && !key.is_empty()).then_some((module, key))
}

/// The errors of the `modules` component: the name of each module it reports, in its
/// own case, with the sentence. The detail is the JSON text of an object.
fn errors(component: &Component) -> Vec<(String, String)> {
    let Some(Value::Object(modules)) = component
        .details
        .iter()
        .find(|(key, _)| key == MODULES)
        .and_then(|(_, text)| serde_json::from_str(text).ok())
    else {
        return Vec::new();
    };
    modules
        .into_iter()
        .filter_map(|(name, module)| {
            let error = module.get("error")?.as_str()?.to_owned();
            Some((name, error))
        })
        .collect()
}

/// Splits the components of an answer between the app and its modules.
///
/// An indicator `module:{name}:{key}` belongs to the module whose name, in ASCII lower
/// case, is `{name}`: PHP's `strtolower` changes ASCII only. The `modules` component
/// leaves the app's list. Only an error it gives for a name that is no module of the
/// app keeps it there, with those errors as its details.
pub fn by_module(app: &AppSnapshot, components: &[Component]) -> RuntimeByModule {
    let lower: Vec<String> = app
        .modules
        .iter()
        .map(|module| module.name.to_ascii_lowercase())
        .collect();
    let position = |name: &str| lower.iter().position(|lower| *lower == name);

    let mut indicators: Vec<Vec<Component>> = vec![Vec::new(); app.modules.len()];
    let mut errors_of: Vec<Option<String>> = vec![None; app.modules.len()];
    let mut own = Vec::new();
    for component in components {
        if component.name == MODULES {
            let mut elsewhere = Vec::new();
            for (name, error) in errors(component) {
                match position(&name.to_ascii_lowercase()) {
                    Some(at) => errors_of[at] = Some(error),
                    None => elsewhere.push((name, error)),
                }
            }
            if !elsewhere.is_empty() {
                own.push(Component {
                    details: elsewhere,
                    ..component.clone()
                });
            }
            continue;
        }
        match indicator_name(&component.name)
            .and_then(|(module, key)| Some((position(module)?, key)))
        {
            Some((at, key)) => indicators[at].push(Component {
                name: key.to_owned(),
                ..component.clone()
            }),
            None => own.push(component.clone()),
        }
    }

    let modules = app
        .modules
        .iter()
        .zip(indicators)
        .zip(errors_of)
        .filter_map(|((module, indicators), error)| {
            let worst = indicators
                .iter()
                .map(|indicator| indicator.status)
                .min_by_key(|status| rank(*status));
            let status = if error.is_some() {
                Status::Down
            } else {
                worst?
            };
            Some(ModuleRuntime {
                module: module.name.clone(),
                status,
                error,
                indicators,
            })
        })
        .collect();
    RuntimeByModule { app: own, modules }
}
