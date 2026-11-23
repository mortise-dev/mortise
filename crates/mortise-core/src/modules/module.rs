//! The modules of an app: found on disk the way the package finds them.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

use ignore::WalkBuilder;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::modules::clover::Coverage;
use crate::modules::git::Churn;
use crate::modules::kind::{Kinds, TESTS};
use crate::modules::layout::Layout;
use crate::php::{Analysis, analyze, count_tests};

/// Whether a module is turned on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleState {
    /// The statuses file has the JSON value `true` for the module.
    On,
    /// Anything else, including no entry: the package treats a missing entry as off.
    Off,
    /// The app does not use the file activator, so the state is not in a file.
    Unknown,
}

/// What reading a module's `module.json` gave.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Manifest {
    Valid,
    /// Not valid JSON. The message is the parser's, without the position.
    InvalidJson {
        message: String,
        line: usize,
    },
    /// Valid JSON without a `name` that is a string. The package needs one.
    NoName,
}

/// How many files of one kind a module has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindCount {
    /// `Models`, `Controllers`, `Tests`, or `Other`.
    pub kind: String,
    pub files: u32,
}

/// One module of an app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Module {
    /// The `name` of `module.json`. The folder name when the file gives none.
    pub name: String,
    /// The three letter code on the module's stamp.
    pub code: String,
    /// The module's folder. Absolute.
    pub dir: PathBuf,
    pub state: ModuleState,
    pub description: String,
    /// The load order the package uses, lowest first.
    pub priority: i64,
    pub manifest: Manifest,
    /// The namespace the module's classes share: `Modules\Billing`. From the
    /// module's `composer.json`, else from the config and the module's name.
    pub namespace: String,
    /// Files in the module's folder. Hidden files, `vendor`, `node_modules` and
    /// whatever `.gitignore` excludes are not counted.
    pub files: u64,
    /// Lines in the module's PHP files. Blade views are not PHP files.
    pub lines: u64,
    /// The files by kind, most files first. The kind comes from the folder a file is in.
    pub kinds: Vec<KindCount>,
    /// Tests in the module's test files.
    pub tests: u32,
    /// Statements covered by tests, from the coverage report. `None` when there
    /// is no report or it has none of the module's files.
    pub coverage: Option<Coverage>,
    /// Commits in git. `None` when the app is not in a repository.
    pub churn: Option<Churn>,
    /// Files of the module that git cannot restore: changed, staged, added,
    /// deleted or untracked. `None` when the app is not in a repository, git is
    /// missing, or git failed.
    pub uncommitted: Option<u32>,
}

/// A module as read from disk, before it has a code, coverage and git data.
pub(crate) struct FoundModule {
    pub(crate) name: String,
    pub(crate) dir: PathBuf,
    pub(crate) state: ModuleState,
    pub(crate) description: String,
    pub(crate) priority: i64,
    pub(crate) manifest: Manifest,
    /// The namespace the module's classes share: `Modules\Billing`.
    pub(crate) namespace: String,
    pub(crate) scan: Scan,
}

/// What walking a module's folder gave.
#[derive(Debug, Default, Clone)]
pub(crate) struct Scan {
    pub(crate) files: u64,
    pub(crate) lines: u64,
    pub(crate) kinds: Vec<KindCount>,
    pub(crate) tests: u32,
    /// The module's PHP files, by path.
    pub(crate) sources: Vec<Source>,
}

/// A PHP file of a module.
#[derive(Debug, Clone)]
pub(crate) struct Source {
    /// The path inside the module, with `/` between the parts.
    pub(crate) relative: String,
    /// True when the file is in a folder of tests.
    pub(crate) is_test: bool,
    /// What the file names and declares.
    pub(crate) analysis: Arc<Analysis>,
}

/// What was worked out from the content of a file, kept so that a file that did
/// not change is not read again.
#[derive(Debug, Clone)]
struct Facts {
    size: u64,
    modified: Option<SystemTime>,
    /// Whether the file counted as a test file when it was read. Its tests are
    /// counted only then.
    is_test: bool,
    lines: u64,
    tests: u32,
    analysis: Arc<Analysis>,
}

/// How far a read of an app is: how many of its PHP files are read.
#[derive(Debug, Default)]
pub struct Progress {
    read: AtomicUsize,
    total: AtomicUsize,
}

impl Progress {
    /// The files read so far, and the files there are. Both are 0 before a read
    /// has found the modules.
    pub fn get(&self) -> (usize, usize) {
        (
            self.read.load(Ordering::Relaxed),
            self.total.load(Ordering::Relaxed),
        )
    }
}

/// The facts of the files read so far, by path.
#[derive(Debug, Default)]
pub struct ReadCache {
    files: HashMap<PathBuf, Facts>,
    progress: Arc<Progress>,
}

impl ReadCache {
    /// Forgets everything, so that the next read reads every file.
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// How far the read that uses this cache is. Another thread can watch it.
    pub fn progress(&self) -> Arc<Progress> {
        Arc::clone(&self.progress)
    }
}

/// Finds the modules of an app: every folder directly under the modules folder
/// that holds a `module.json`. Sorted by name.
///
/// Also gives the names the statuses file lists, for the check on entries without a module.
pub(crate) fn find_modules(
    layout: &Layout,
    kinds: &Kinds,
    cache: &mut ReadCache,
) -> (Vec<FoundModule>, Vec<String>) {
    let statuses = read_statuses(&layout.statuses_file);
    let listed: Vec<String> = statuses.keys().cloned().collect();

    let Ok(entries) = std::fs::read_dir(&layout.modules_dir) else {
        return (Vec::new(), listed);
    };
    let dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|dir| dir.join("module.json").is_file())
        .collect();
    // How many PHP files there are, so that a watcher can say how far the read is.
    let total: usize = dirs
        .iter()
        .map(|dir| {
            files_of(dir)
                .iter()
                .filter(|(_, relative)| is_php(relative))
                .count()
        })
        .sum();
    cache.progress.read.store(0, Ordering::Relaxed);
    cache.progress.total.store(total, Ordering::Relaxed);
    let mut modules: Vec<FoundModule> = dirs
        .into_iter()
        .filter_map(|dir| {
            let folder = dir.file_name()?.to_str()?.to_owned();
            // The package's glob does not match folders that start with a dot.
            if folder.starts_with('.') {
                return None;
            }
            let (manifest, fields) = read_manifest(&dir.join("module.json"));
            let name = fields.name.unwrap_or(folder);
            let state = if !layout.uses_file_activator() {
                ModuleState::Unknown
            } else if statuses.get(&name) == Some(&serde_json::Value::Bool(true)) {
                ModuleState::On
            } else {
                ModuleState::Off
            };
            Some(FoundModule {
                scan: scan(&dir, kinds, cache),
                namespace: read_namespace(&dir, layout, &name),
                name,
                dir,
                state,
                description: fields.description,
                priority: fields.priority,
                manifest,
            })
        })
        .collect();

    modules.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.dir.cmp(&b.dir))
    });
    (modules, listed)
}

#[derive(Default)]
struct ManifestFields {
    name: Option<String>,
    description: String,
    priority: i64,
}

fn read_manifest(path: &Path) -> (Manifest, ManifestFields) {
    let json = std::fs::read(path).unwrap_or_default();
    let value: serde_json::Value = match serde_json::from_slice(&json) {
        Ok(value) => value,
        Err(error) => {
            // serde_json prints "<message> at line L column C". Keep the message.
            let text = error.to_string();
            let message = text.split(" at line ").next().unwrap_or(&text).to_owned();
            let invalid = Manifest::InvalidJson {
                message,
                line: error.line(),
            };
            return (invalid, ManifestFields::default());
        }
    };

    let fields = ManifestFields {
        name: value
            .get("name")
            .and_then(|name| name.as_str())
            .map(str::to_owned),
        description: value
            .get("description")
            .and_then(|description| description.as_str())
            .unwrap_or_default()
            .to_owned(),
        priority: value
            .get("priority")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0),
    };
    let manifest = if fields.name.is_some() {
        Manifest::Valid
    } else {
        Manifest::NoName
    };
    (manifest, fields)
}

/// The entries of the statuses file. Empty when the file is missing or is not a JSON object.
fn read_statuses(path: &Path) -> BTreeMap<String, serde_json::Value> {
    std::fs::read(path)
        .ok()
        .and_then(|json| serde_json::from_slice(&json).ok())
        .unwrap_or_default()
}

/// The namespace of a module: the shortest PSR-4 prefix of its `composer.json`,
/// which is the one that maps to its app folder. Without one, the namespace of
/// the config and the name of the module.
fn read_namespace(dir: &Path, layout: &Layout, name: &str) -> String {
    let from_composer = std::fs::read(dir.join("composer.json"))
        .ok()
        .and_then(|json| serde_json::from_slice::<serde_json::Value>(&json).ok())
        .and_then(|composer| {
            let prefixes = composer.get("autoload")?.get("psr-4")?.as_object()?;
            prefixes.keys().min_by_key(|prefix| prefix.len()).cloned()
        })
        .map(|prefix| prefix.trim_matches('\\').to_owned())
        .filter(|prefix| !prefix.is_empty());
    from_composer.unwrap_or_else(|| format!("{}\\{name}", layout.namespace.trim_matches('\\')))
}

/// A file of a module, before its content is read.
struct Entry {
    path: PathBuf,
    /// The path inside the module, with `/` between the parts.
    relative: String,
    is_test: bool,
    size: u64,
    modified: Option<SystemTime>,
}

/// The files of a module, each with its path inside the module, `/` between the
/// parts. Hidden files, ignored files and installed packages are left out.
pub(crate) fn files_of(dir: &Path) -> Vec<(PathBuf, String)> {
    let walk = WalkBuilder::new(dir)
        // Apply .gitignore files also when the app is not a git repository.
        .require_git(false)
        .filter_entry(|entry| {
            let is_dir = entry.file_type().is_some_and(|kind| kind.is_dir());
            !(is_dir && matches!(entry.file_name().to_str(), Some("vendor" | "node_modules")))
        })
        .build();
    walk.filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .map(|entry| {
            let path = entry.path();
            let relative = path
                .strip_prefix(dir)
                .unwrap_or(path)
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            (path.to_owned(), relative)
        })
        .collect()
}

/// True for a PHP file that is not a Blade view: those end in `.php` but are templates.
pub(crate) fn is_php(relative: &str) -> bool {
    relative.ends_with(".php") && !relative.ends_with(".blade.php")
}

/// Walks a module's folder: counts its files by kind, the lines of its PHP files
/// and the tests in its test files, and reads which classes each PHP file names.
fn scan(dir: &Path, kinds: &Kinds, cache: &mut ReadCache) -> Scan {
    let mut scan = Scan::default();
    let mut by_kind: BTreeMap<String, u32> = BTreeMap::new();
    let mut php: Vec<Entry> = Vec::new();

    for (path, relative) in files_of(dir) {
        let path = path.as_path();
        let kind = kinds.of(&relative);
        let is_test = kind == TESTS;
        scan.files += 1;
        *by_kind.entry(kind).or_default() += 1;

        if is_php(&relative) {
            let metadata = std::fs::metadata(path).ok();
            php.push(Entry {
                path: path.to_owned(),
                relative,
                is_test,
                size: metadata.as_ref().map_or(0, std::fs::Metadata::len),
                modified: metadata.and_then(|metadata| metadata.modified().ok()),
            });
        }
    }

    // Read the files that are new or changed, on all cores.
    let changed: Vec<&Entry> = php
        .iter()
        .filter(|entry| {
            !cache.files.get(&entry.path).is_some_and(|facts| {
                facts.size == entry.size
                    && facts.modified == entry.modified
                    && entry.modified.is_some()
                    && facts.is_test == entry.is_test
            })
        })
        .collect();
    let progress = Arc::clone(&cache.progress);
    progress
        .read
        .fetch_add(php.len() - changed.len(), Ordering::Relaxed);
    let read: Vec<(PathBuf, Facts)> = changed
        .par_iter()
        .map(|entry| {
            let source = std::fs::read(&entry.path).unwrap_or_default();
            progress.read.fetch_add(1, Ordering::Relaxed);
            let facts = Facts {
                size: entry.size,
                modified: entry.modified,
                is_test: entry.is_test,
                lines: count_lines(&source),
                tests: if entry.is_test {
                    count_tests(&source)
                } else {
                    0
                },
                analysis: Arc::new(analyze(&source)),
            };
            (entry.path.clone(), facts)
        })
        .collect();
    cache.files.extend(read);

    for entry in php {
        let Some(facts) = cache.files.get(&entry.path) else {
            continue;
        };
        scan.lines += facts.lines;
        scan.tests += facts.tests;
        scan.sources.push(Source {
            relative: entry.relative,
            is_test: entry.is_test,
            analysis: Arc::clone(&facts.analysis),
        });
    }
    scan.sources.sort_by(|a, b| a.relative.cmp(&b.relative));

    scan.kinds = by_kind
        .into_iter()
        .map(|(kind, files)| KindCount { kind, files })
        .collect();
    // Most files first; kinds with the same number of files by name.
    scan.kinds
        .sort_by(|a, b| b.files.cmp(&a.files).then_with(|| a.kind.cmp(&b.kind)));
    scan
}

/// Lines of a file: line ends, plus a last line that has no line end.
fn count_lines(source: &[u8]) -> u64 {
    let ends = source.iter().filter(|byte| **byte == b'\n').count() as u64;
    match source.last() {
        Some(b'\n') | None => ends,
        Some(_) => ends + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn scan_dir(dir: &Path) -> Scan {
        scan(dir, &Kinds::new(None, "app/"), &mut ReadCache::default())
    }

    #[test]
    fn counts_files_but_not_hidden_ignored_or_installed_ones() {
        let dir = tempfile::tempdir().unwrap();
        let module = dir.path();
        write(&module.join("module.json"), "{}");
        write(&module.join("app/Models/Post.php"), "<?php");
        write(&module.join("resources/views/index.blade.php"), "");
        write(&module.join("tests/Unit/.gitkeep"), "");
        write(&module.join("node_modules/left-pad/index.js"), "");
        write(&module.join("vendor/autoload.php"), "<?php");
        write(&module.join(".gitignore"), "/build\n");
        write(&module.join("build/app.js"), "");
        assert_eq!(scan_dir(module).files, 3);
    }

    #[test]
    fn counts_lines_of_php_kinds_and_tests() {
        let dir = tempfile::tempdir().unwrap();
        let module = dir.path();
        write(&module.join("module.json"), "{\n}\n");
        write(
            &module.join("app/Models/Post.php"),
            "<?php\n\nclass Post {}\n",
        );
        write(&module.join("app/Models/Tag.php"), "<?php\nclass Tag {}");
        write(
            &module.join("app/Http/Controllers/PostController.php"),
            "<?php\n",
        );
        write(
            &module.join("resources/views/index.blade.php"),
            "<p>\n</p>\n<?php echo 1; ?>\n",
        );
        write(
            &module.join("tests/Unit/PostTest.php"),
            "<?php\nclass PostTest {\n  function testTitle() {}\n  function testSlug() {}\n}\n",
        );
        // Not in a tests folder: its methods are not counted as tests.
        write(
            &module.join("app/Support/Check.php"),
            "<?php class Check { function testThis() {} }",
        );

        let scan = scan_dir(module);
        assert_eq!(scan.files, 7);
        // 3 + 2 + 1 + 5 + 1. The Blade view and module.json are not PHP.
        assert_eq!(scan.lines, 12);
        assert_eq!(scan.tests, 2);
        let kinds: Vec<_> = scan
            .kinds
            .iter()
            .map(|kind| (kind.kind.as_str(), kind.files))
            .collect();
        assert_eq!(
            kinds,
            [
                ("Models", 2),
                ("Controllers", 1),
                ("Other", 1),
                ("Support", 1),
                ("Tests", 1),
                ("Views", 1)
            ]
        );
    }

    #[test]
    fn a_file_that_did_not_change_is_not_read_again() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("app/Models/Post.php");
        write(&file, "<?php\nclass Post {}\n");
        let kinds = Kinds::new(None, "app/");
        let mut cache = ReadCache::default();
        assert_eq!(scan(dir.path(), &kinds, &mut cache).lines, 2);

        // Same size and time: the cached facts are used even if the cache is wrong.
        cache.files.get_mut(&file).unwrap().lines = 99;
        assert_eq!(scan(dir.path(), &kinds, &mut cache).lines, 99);

        // Another size: read again.
        write(&file, "<?php\nclass Post {}\n// more\n");
        assert_eq!(scan(dir.path(), &kinds, &mut cache).lines, 3);

        cache.clear();
        assert_eq!(scan(dir.path(), &kinds, &mut cache).lines, 3);

        // The config makes a folder hold tests: its files are read again.
        write(
            &dir.path().join("spec/Unit/PostTest.php"),
            "<?php class PostTest { function testTitle() {} }",
        );
        assert_eq!(scan(dir.path(), &kinds, &mut cache).tests, 0);
        let config = crate::php::read_config(
            br#"<?php return ['paths' => ['generator' => ['test-unit' => 'spec/Unit']]];"#,
        )
        .unwrap();
        let moved = Kinds::new(Some(&config), "app/");
        assert_eq!(scan(dir.path(), &moved, &mut cache).tests, 1);
    }

    #[test]
    fn counts_a_last_line_without_a_line_end() {
        assert_eq!(count_lines(b""), 0);
        assert_eq!(count_lines(b"a"), 1);
        assert_eq!(count_lines(b"a\n"), 1);
        assert_eq!(count_lines(b"a\nb"), 2);
        assert_eq!(count_lines(b"a\r\nb\r\n"), 2);
    }

    #[test]
    fn reads_the_fields_the_package_reads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("module.json");

        write(
            &path,
            r#"{"name": "Blog", "description": "Posts", "priority": 5}"#,
        );
        let (manifest, fields) = read_manifest(&path);
        assert_eq!(manifest, Manifest::Valid);
        assert_eq!(fields.name.as_deref(), Some("Blog"));
        assert_eq!((fields.description.as_str(), fields.priority), ("Posts", 5));

        write(&path, r#"{"alias": "blog"}"#);
        assert_eq!(read_manifest(&path).0, Manifest::NoName);

        write(&path, "{\n  \"name\": \"Blog\",\n");
        let (manifest, fields) = read_manifest(&path);
        assert_eq!(
            manifest,
            Manifest::InvalidJson {
                message: "EOF while parsing a value".into(),
                line: 3
            }
        );
        assert_eq!(fields.name, None);
    }

    #[test]
    fn a_statuses_file_that_is_missing_or_not_an_object_has_no_entries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("modules_statuses.json");
        assert!(read_statuses(&path).is_empty());
        for content in ["", "[]", "[true]", "not json", "null"] {
            write(&path, content);
            assert!(read_statuses(&path).is_empty(), "{content:?}");
        }
        write(&path, r#"{"Blog": true}"#);
        assert_eq!(read_statuses(&path).len(), 1);
    }
}
