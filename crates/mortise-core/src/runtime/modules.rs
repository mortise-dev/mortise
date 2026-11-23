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
    /// The errors the `modules` component gives for names that are no module of the
    /// app: each name, in the app's case, with its sentence.
    pub elsewhere: Vec<(String, String)>,
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
    // The package does not refuse an empty key: `module:billing:` is Billing's.
    (!module.is_empty()).then_some((module, key))
}

/// The errors of the `modules` component: the name of each module it reports, in its
/// own case, with the sentence. The detail is the JSON text of an object, or `[]` when
/// the app has no module: PHP writes an empty array so.
///
/// `None` when the component is not the list of laravel-modules: its one detail is
/// not `modules`, or an entry is not `{"enabled": bool}` with an optional `"error"`
/// sentence. That is a check that failed, with the exception as its `error`, or a
/// component of the app's own under that name.
fn errors(component: &Component) -> Option<Vec<(String, String)>> {
    let [(key, text)] = component.details.as_slice() else {
        return None;
    };
    if key != MODULES {
        return None;
    }
    let modules = match serde_json::from_str(text).ok()? {
        Value::Object(modules) => modules,
        Value::Array(empty) if empty.is_empty() => return Some(Vec::new()),
        _ => return None,
    };
    let mut errors = Vec::new();
    for (name, module) in modules {
        let Value::Object(fields) = module else {
            return None;
        };
        let shape = fields.iter().all(|(field, value)| match field.as_str() {
            "enabled" => value.is_boolean(),
            "error" => value.is_string(),
            _ => false,
        });
        if !shape || !fields.contains_key("enabled") {
            return None;
        }
        if let Some(Value::String(error)) = fields.get("error") {
            errors.push((name, error.clone()));
        }
    }
    Some(errors)
}

/// Splits the components of an answer between the app and its modules.
///
/// An indicator `module:{name}:{key}` belongs to the module whose name, in ASCII lower
/// case, is `{name}`: PHP's `strtolower` changes ASCII only. The `modules` component
/// of laravel-modules leaves the app's list; an error it gives for a name that is no
/// module of the app goes to `elsewhere`. A component named `modules` of another shape
/// stays the app's.
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
    let mut elsewhere = Vec::new();
    for component in components {
        if let Some(errors) = (component.name == MODULES)
            .then(|| errors(component))
            .flatten()
        {
            for (name, error) in errors {
                match position(&name.to_ascii_lowercase()) {
                    Some(at) => errors_of[at] = Some(error),
                    None => elsewhere.push((name, error)),
                }
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
    RuntimeByModule {
        app: own,
        modules,
        elsewhere,
    }
}
