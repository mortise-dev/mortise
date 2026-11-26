//! Mortise and Larch give the same result on the same app: the shared fixtures of
//! Larch, copied to `fixtures/larch`, each with the result Larch expects.

use std::path::Path;

use serde_json::{Value, json};

use mortise_core::{AppSnapshot, Check, Detail, ModuleState, read_app};

/// The class a file declares, by the layout of laravel-modules: `Modules/Billing/app/Models/Invoice.php`
/// declares `Modules\Billing\Models\Invoice`. A file outside `app`, such as a route
/// file, declares no class: Larch then names the file.
fn user(app: &AppSnapshot, module: &str, file: &str) -> String {
    let namespace = &app.module(module).unwrap().namespace;
    match file
        .strip_prefix(&format!("Modules/{module}/app/"))
        .and_then(|rest| rest.strip_suffix(".php"))
    {
        Some(rest) => format!("{namespace}\\{}", rest.replace('/', "\\")),
        None => file.to_owned(),
    }
}

/// Mortise's result in the shape of Larch's `expected.json`.
fn result(app: &AppSnapshot) -> Value {
    let rules = &app.rules;
    let circular = rules.no_circular_use.then(|| {
        app.cycles
            .iter()
            .map(|cycle| json!(cycle.modules))
            .collect::<Vec<_>>()
    });
    let internal = rules.public_surface_only.then(|| {
        let mut pairs: Vec<(String, String)> = app
            .links
            .iter()
            .flat_map(|link| {
                link.refs
                    .iter()
                    .filter(|reference| reference.internal)
                    .map(|reference| {
                        (
                            user(app, &link.from, &reference.file),
                            reference.class.clone(),
                        )
                    })
            })
            .collect();
        pairs.sort();
        pairs.dedup();
        pairs
            .into_iter()
            .map(|(from, to)| json!({"from": from, "to": to}))
            .collect::<Vec<_>>()
    });
    let allowed = rules
        .allowed_use
        .as_ref()
        .filter(|allowed| allowed.enabled)
        .map(|_| {
            app.findings
                .list
                .iter()
                .filter(|finding| finding.check == Check::AllowedUse)
                .filter_map(|finding| match &finding.detail {
                    Detail::OutsideAllowedUse { of, files } => {
                        Some(json!({"from": finding.module, "to": of, "files": files}))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        });
    json!({
        "no-circular-use": circular,
        "public-surface-only": internal,
        "allowed-use": allowed,
    })
}

/// The apps where Larch differs. They have no `modules_statuses.json`. Larch turns off
/// only a module whose entry is `false`, so it counts every module of them on.
/// laravel-modules turns off a module without an entry (`FileActivator::hasStatus`),
/// and so does Mortise: every module is off and nothing breaks a rule. With a statuses
/// file that turns every module on, Mortise gives what Larch expects.
const LARCH_COUNTS_UNLISTED_ON: [&str; 3] = ["all-off", "custom-folders", "defaults"];

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn mortise_gives_the_result_larch_expects() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/larch");
    let mut apps: Vec<_> = std::fs::read_dir(&fixtures)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    apps.sort();
    assert_eq!(apps.len(), 5);
    for root in apps {
        let name = root.file_name().unwrap().to_str().unwrap();
        let expected: Value =
            serde_json::from_slice(&std::fs::read(root.join("expected.json")).unwrap()).unwrap();
        let app = read_app(&root).unwrap();
        if !LARCH_COUNTS_UNLISTED_ON.contains(&name) {
            assert_eq!(result(&app), expected, "{name}");
            continue;
        }

        assert!(!root.join("modules_statuses.json").exists(), "{name}");
        assert!(
            app.modules
                .iter()
                .all(|module| module.state == ModuleState::Off),
            "{name}"
        );
        // Each rule that is on finds nothing.
        let nothing: Value = expected
            .as_object()
            .unwrap()
            .iter()
            .map(|(rule, found)| (rule.clone(), found.as_array().map(|_| json!([])).into()))
            .collect::<serde_json::Map<_, _>>()
            .into();
        assert_eq!(result(&app), nothing, "{name}");

        let copy = tempfile::tempdir().unwrap();
        copy_dir(&root, copy.path());
        let on: serde_json::Map<String, Value> = app
            .modules
            .iter()
            .map(|module| (module.name.clone(), json!(true)))
            .collect();
        std::fs::write(
            copy.path().join("modules_statuses.json"),
            serde_json::to_vec(&on).unwrap(),
        )
        .unwrap();
        assert_eq!(
            result(&read_app(copy.path()).unwrap()),
            expected,
            "{name} with every module on"
        );
    }
}
