//! What the output of a run says about generated files.
//!
//! A generator prints `Generating file <path> ... <time> DONE` or `FAIL` for each
//! file. A `FAIL` is followed by an error line `File : <path> already exists.`
//! when the file is there. `module:make` prints `composer dump-autoload failed.`
//! when the autoload step fails.
//!
//! From 13.4.0 on a generator run with `--json` gives its files as a document
//! ([`GeneratorReport`]); [`reported_files`] reads it into the same rows.

use std::path::Path;

use crate::package::{FileResult, GeneratorReport};
use crate::run::ansi::plain;

/// What a generator run did with one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOutcome {
    /// The file is new.
    Created,
    /// The file existed and the run, which carried `--force`, wrote it again.
    Replaced,
    /// The file exists and the run did not write it.
    Left,
    /// The file was not written for another reason.
    Failed,
}

/// One file of a generator run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFile {
    /// The path as the output prints it.
    pub path: String,
    pub outcome: FileOutcome,
}

/// The files of a generator run, in output order. `force` is true when the run
/// carried `--force`.
///
/// The exit code does not say what happened to each file: `module:make-model`
/// exits 0 when it leaves its policy.
pub fn generated_files(lines: &[String], force: bool) -> Vec<GeneratedFile> {
    let mut files: Vec<GeneratedFile> = Vec::new();
    for line in lines {
        let line = plain(line);
        if let Some((path, done)) = generating_line(&line) {
            let outcome = match (done, force) {
                (true, true) => FileOutcome::Replaced,
                (true, false) => FileOutcome::Created,
                (false, _) => FileOutcome::Failed,
            };
            files.push(GeneratedFile {
                path: path.to_string(),
                outcome,
            });
        } else if let Some(path) = exists_path(&line) {
            let failed = files
                .iter_mut()
                .rev()
                .find(|file| file.path == path && file.outcome == FileOutcome::Failed);
            match failed {
                Some(file) => file.outcome = FileOutcome::Left,
                None => files.push(GeneratedFile {
                    path: path.to_string(),
                    outcome: FileOutcome::Left,
                }),
            }
        }
    }
    files
}

/// The files of a generator run from its document (P5), in write order. The folder
/// that `module:make --force` deletes is not a file. A path is made relative to the
/// app folder `root`, as the output lines print it, through [`app_relative`] with the
/// modules folder `modules`; a path it cannot place stays as printed.
pub fn reported_files(
    report: &GeneratorReport,
    root: &Path,
    modules: &str,
    names: &[&str],
    folders: &[&str],
) -> Vec<GeneratedFile> {
    report
        .files
        .iter()
        .filter_map(|file| {
            let outcome = match file.result? {
                FileResult::Created => FileOutcome::Created,
                FileResult::Replaced => FileOutcome::Replaced,
                FileResult::Exists => FileOutcome::Left,
                FileResult::Failed => FileOutcome::Failed,
                FileResult::Deleted => return None,
            };
            let path = app_relative(root, modules, names, folders, &file.path)
                .unwrap_or_else(|| file.path.clone());
            Some(GeneratedFile { path, outcome })
        })
        .collect()
}

/// A path the package printed, relative to the app folder `root` with `/` between its
/// parts, or `None` when Mortise cannot place it. The package prints absolute paths
/// with `/` on every platform, from the real folder of the app, so `root` is compared
/// as given and with its links resolved.
///
/// PHP in a container (Sail, Docker) prints the folder of the container, such as
/// `/var/www/html/Modules/Billing/app/Models/Invoice.php`. A path outside `root` is
/// then read from a part `/<modules>/<Module>/` on, where `modules` is the modules
/// folder relative to the app (`Modules`): `Modules/Billing/app/Models/Invoice.php`.
/// The container's folder may hold that name too (`/srv/Modules/shop-api/Modules/...`),
/// and so may the class name (`.../Models/Admin/Modules/Billing/Invoice.php`). The
/// first part counts whose `<Module>` is one of `names`, the modules the run writes
/// in: its own module, or those `module:make` asks for (the package makes them
/// studly: case, `-` and `_` do not count). Else the first part whose `<Module>` is
/// one of `folders`, the folder names of the app's modules, counts, and with none of
/// them, the last part.
pub fn app_relative(
    root: &Path,
    modules: &str,
    names: &[&str],
    folders: &[&str],
    printed: &str,
) -> Option<String> {
    let printed = printed.replace('\\', "/");
    [Some(root.to_path_buf()), root.canonicalize().ok()]
        .into_iter()
        .flatten()
        .find_map(|root| {
            let root = root.to_string_lossy().replace('\\', "/");
            // A resolved path on Windows starts with `\\?\`.
            let root = root.strip_prefix("//?/").unwrap_or(&root);
            let rest = printed.strip_prefix(root.trim_end_matches('/'))?;
            let rest = rest.strip_prefix('/')?;
            (!rest.is_empty()).then(|| rest.to_owned())
        })
        .or_else(|| from_modules(modules, names, folders, &printed))
}

/// The part of `printed` from a `/<modules>/<Module>/` on, when a file follows: the
/// first whose `<Module>` is one of `names`, else the first whose `<Module>` is one of
/// `folders`, else the last.
fn from_modules(modules: &str, names: &[&str], folders: &[&str], printed: &str) -> Option<String> {
    let modules = modules.trim_matches('/');
    if modules.is_empty() {
        return None;
    }
    let part = format!("/{modules}/");
    let found: Vec<(&str, &str)> = printed
        .char_indices()
        .filter(|(at, _)| printed[*at..].starts_with(&part))
        .filter_map(|(at, _)| {
            let rest = &printed[at + 1..];
            let (module, file) = rest[modules.len() + 1..].split_once('/')?;
            (!module.is_empty() && !file.is_empty()).then_some((module, rest))
        })
        .collect();
    let studly = |name: &str| {
        name.chars()
            .filter(|c| *c != '-' && *c != '_')
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>()
    };
    let asked: Vec<String> = names.iter().map(|name| studly(name)).collect();
    let first = |known: &dyn Fn(&str) -> bool| found.iter().find(|(module, _)| known(module));
    first(&|module| asked.contains(&studly(module)))
        .or_else(|| first(&|module| folders.contains(&module)))
        .or(found.last())
        .map(|(_, rest)| (*rest).to_owned())
}

/// True when the output says that `composer dump-autoload` failed.
pub fn autoload_failed(lines: &[String]) -> bool {
    lines.iter().any(|line| {
        plain(line)
            .trim()
            .ends_with("composer dump-autoload failed.")
    })
}

/// The path and the result (`true` for `DONE`) of a `Generating file` line. The
/// path ends before the dots and the time.
fn generating_line(line: &str) -> Option<(&str, bool)> {
    let rest = line.trim().strip_prefix("Generating file ")?;
    let (rest, end) = rest.trim_end().rsplit_once(' ')?;
    let done = match end {
        "DONE" => true,
        "FAIL" => false,
        _ => return None,
    };
    let rest = rest.trim_end();
    let rest = match rest.rsplit_once(' ') {
        Some((before, time)) if is_time(time) => before,
        _ => rest,
    };
    let path = rest.trim_end_matches(['.', ' ']);
    (!path.is_empty()).then_some((path, done))
}

/// A time such as `0.24ms` or `1s`.
fn is_time(word: &str) -> bool {
    word.starts_with(|c: char| c.is_ascii_digit()) && word.ends_with('s')
}

/// The path of a `File : <path> already exists.` line.
fn exists_path(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once("File : ")?;
    let path = rest.trim_end().strip_suffix(" already exists.")?;
    (!path.is_empty()).then_some(path)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::actions::output::tests::recording;

    fn files(name: &str, force: bool) -> Vec<(String, FileOutcome)> {
        generated_files(&recording(name), force)
            .into_iter()
            .map(|file| (file.path, file.outcome))
            .collect()
    }

    #[test]
    fn a_generator_run_creates_its_files() {
        let created = |path: &str| (path.to_string(), FileOutcome::Created);
        assert_eq!(
            files("module-make-model-migration-policy", false),
            vec![
                created("Modules/Billing/app/Models/PaymentIntent.php"),
                created(
                    "Modules/Billing/database/migrations/2026_11_23_071900_create_payment_intents_table.php"
                ),
                created("Modules/Billing/app/Policies/PaymentIntentPolicy.php"),
            ]
        );
        let module = files("module-make", false);
        assert_eq!(module[0], created("Modules/Blog/module.json"));
        assert!(
            module
                .iter()
                .all(|(_, outcome)| *outcome == FileOutcome::Created)
        );
        assert_eq!(files("module-make-api", false).len(), 16);
    }

    #[test]
    fn a_file_that_exists_is_left() {
        // The model run exits 0 and leaves only its policy.
        let outcomes: Vec<_> = files("module-make-model-policy-exists", false)
            .into_iter()
            .map(|(_, outcome)| outcome)
            .collect();
        assert_eq!(
            outcomes,
            [
                FileOutcome::Created,
                FileOutcome::Created,
                FileOutcome::Left
            ]
        );
        assert_eq!(
            files("module-make-controller-exists", false),
            vec![(
                "Modules/Blog/app/Http/Controllers/PostController.php".to_string(),
                FileOutcome::Left
            )]
        );
    }

    #[test]
    fn a_forced_run_replaces_its_file() {
        assert_eq!(
            files("module-make-policy-force", true),
            vec![(
                "Modules/Billing/app/Policies/InvoicePolicy.php".to_string(),
                FileOutcome::Replaced
            )]
        );
    }

    #[test]
    fn a_report_gives_the_files_relative_to_the_app() {
        let root = Path::new("/tmp/scratch-app");
        let read = |name: &str| {
            let report: GeneratorReport = crate::package::document(&recording(name)).unwrap();
            reported_files(&report, root, "Modules", &[], &[])
                .into_iter()
                .map(|file| (file.path, file.outcome))
                .collect::<Vec<_>>()
        };
        let created = read("module-make-model-json");
        assert_eq!(
            created[0],
            (
                "Modules/Billing/app/Models/PaymentIntent.php".to_string(),
                FileOutcome::Created
            )
        );
        assert!(
            created
                .iter()
                .all(|(_, outcome)| *outcome == FileOutcome::Created)
        );
        // The answer says the policy exists, as the lines do.
        let left = read("module-make-model-policy-exists-json");
        assert_eq!(
            left[2],
            (
                "Modules/Billing/app/Policies/InvoicePolicy.php".to_string(),
                FileOutcome::Left
            )
        );
        assert_eq!(
            app_relative(
                Path::new("/tmp/other"),
                "Modules",
                &[],
                &[],
                "/tmp/scratch-app/x.php"
            ),
            None
        );
        // PHP in a container prints the container's folder: the path is read from
        // the modules folder of the app's layout on.
        let container = |modules: &str, printed: &str| {
            app_relative(
                Path::new("/Users/me/shop"),
                modules,
                &[],
                &["Billing"],
                printed,
            )
        };
        assert_eq!(
            container(
                "Modules",
                "/var/www/html/Modules/Billing/app/Models/Invoice.php"
            )
            .as_deref(),
            Some("Modules/Billing/app/Models/Invoice.php")
        );
        assert_eq!(
            container("src/Domains", "/app/src/Domains/Billing/Invoice.php").as_deref(),
            Some("src/Domains/Billing/Invoice.php")
        );
        assert_eq!(container("Modules", "/var/www/html/Modules/x.php"), None);
        // A container folder that holds the name of the modules folder: the part of a
        // module of the app counts.
        assert_eq!(
            container(
                "Modules",
                "/srv/Modules/shop-api/Modules/Billing/app/Models/Invoice.php"
            )
            .as_deref(),
            Some("Modules/Billing/app/Models/Invoice.php")
        );
        // A new module comes before a known one: `module:make loyalty` placed in a
        // container folder named after Billing.
        assert_eq!(
            app_relative(
                Path::new("/Users/me/shop"),
                "Modules",
                &["loyalty"],
                &["Billing"],
                "/srv/Modules/Billing/Modules/Loyalty/module.json"
            )
            .as_deref(),
            Some("Modules/Loyalty/module.json")
        );
        // A class name that repeats the module's part: the first part of the run's
        // own module counts.
        assert_eq!(
            app_relative(
                Path::new("/Users/me/shop"),
                "Modules",
                &["Billing"],
                &["Billing"],
                "/var/www/html/Modules/Billing/app/Models/Admin/Modules/Billing/Invoice.php"
            )
            .as_deref(),
            Some("Modules/Billing/app/Models/Admin/Modules/Billing/Invoice.php")
        );
    }

    #[test]
    fn a_failed_autoload_is_read_from_its_error_line() {
        assert!(!autoload_failed(&recording("module-make-api")));
        assert!(autoload_failed(&[
            "  \x1b[37;41m ERROR \x1b[39;49m composer dump-autoload failed.  ".to_string()
        ]));
    }
}
