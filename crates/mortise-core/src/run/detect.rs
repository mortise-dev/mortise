//! Herd, Sail and Docker, found from the folder of an app. Reads files only.
//!
//! Nothing here starts a program. A file that is missing or not valid means "not found".

use std::fs;
use std::path::{Path, PathBuf};

/// The compose files that Docker Compose reads, in its own order.
const COMPOSE_FILES: [&str; 4] = [
    "compose.yaml",
    "compose.yml",
    "docker-compose.yaml",
    "docker-compose.yml",
];

/// A runner that the folder of an app points to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detected {
    /// The folder is a Herd site, parked or linked.
    Herd,
    /// The folder holds `vendor/bin/sail` and a compose file. Never on Windows.
    Sail,
    /// The folder holds a compose file and Sail is not found. The service is always `app`.
    Docker,
}

/// What the folder points to, the best first. Reads files only.
///
/// `herd_config` is the valet folder of Herd, as [`herd_config_dir`] gives it.
pub fn detect(root: &Path, herd_config: Option<&Path>) -> Vec<Detected> {
    let mut found = Vec::new();
    if herd_config.is_some_and(|valet| is_herd_site(root, valet)) {
        found.push(Detected::Herd);
    }
    let compose = COMPOSE_FILES.iter().any(|name| root.join(name).is_file());
    // Sail is a bash script. On Windows it runs in WSL only, so Mortise does not offer it.
    let sail = !cfg!(windows) && compose && root.join("vendor/bin/sail").is_file();
    if sail {
        found.push(Detected::Sail);
    } else if compose {
        found.push(Detected::Docker);
    }
    found
}

/// The valet folder of Herd on this platform. `None` on Linux.
///
/// These folders are assumed from the documentation of Herd and Valet.
/// They were not checked on a machine with Herd.
pub fn herd_config_dir() -> Option<PathBuf> {
    let home = directories::BaseDirs::new()?.home_dir().to_path_buf();
    if cfg!(target_os = "macos") {
        Some(home.join("Library/Application Support/Herd/config/valet"))
    } else if cfg!(windows) {
        Some(home.join(r".config\herd\config\valet"))
    } else {
        None
    }
}

/// Whether `root` is parked (its parent is in `paths` of `config.json`) or linked
/// (an entry in `Sites/` points to it). Both sides are compared after `canonicalize`.
fn is_herd_site(root: &Path, valet: &Path) -> bool {
    let Ok(root) = fs::canonicalize(root) else {
        return false;
    };
    parked(&root, valet) || linked(&root, valet)
}

fn parked(root: &Path, valet: &Path) -> bool {
    let Some(parent) = root.parent() else {
        return false;
    };
    let Ok(text) = fs::read_to_string(valet.join("config.json")) else {
        return false;
    };
    let Ok(config) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    let Some(paths) = config.get("paths").and_then(|paths| paths.as_array()) else {
        return false;
    };
    paths
        .iter()
        .filter_map(|path| path.as_str())
        .filter_map(|path| fs::canonicalize(path).ok())
        .any(|path| path == parent)
}

/// `canonicalize` follows the link, so a link of any kind (a junction on Windows too) is compared
/// by its target.
fn linked(root: &Path, valet: &Path) -> bool {
    let Ok(entries) = fs::read_dir(valet.join("Sites")) else {
        return false;
    };
    entries
        .flatten()
        .filter_map(|entry| fs::canonicalize(entry.path()).ok())
        .any(|target| target == root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }

    /// A folder `parked/shop` and an empty valet folder.
    fn herd() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("parked/shop");
        fs::create_dir_all(&root).unwrap();
        let valet = temp.path().join("valet");
        fs::create_dir_all(&valet).unwrap();
        (temp, root, valet)
    }

    /// Parks the folder `parked` in the valet folder.
    fn park(temp: &tempfile::TempDir, valet: &Path) {
        let config = serde_json::json!({ "paths": [temp.path().join("parked")] });
        fs::write(valet.join("config.json"), config.to_string()).unwrap();
    }

    #[test]
    fn a_folder_in_a_parked_path_is_herd() {
        let (temp, root, valet) = herd();
        park(&temp, &valet);
        assert_eq!(detect(&root, Some(&valet)), [Detected::Herd]);
        assert_eq!(detect(&root, None), []);
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_linked_from_sites_is_herd() {
        let (temp, _, valet) = herd();
        let root = temp.path().join("elsewhere/shop");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(valet.join("Sites")).unwrap();
        std::os::unix::fs::symlink(&root, valet.join("Sites/shop")).unwrap();
        assert_eq!(detect(&root, Some(&valet)), [Detected::Herd]);
    }

    #[test]
    fn a_broken_config_is_not_herd() {
        let (_temp, root, valet) = herd();
        fs::write(valet.join("config.json"), "{ \"paths\": [").unwrap();
        assert_eq!(detect(&root, Some(&valet)), []);
    }

    #[test]
    fn sail_needs_a_compose_file_and_not_windows() {
        let temp = tempfile::tempdir().unwrap();
        touch(&temp.path().join("vendor/bin/sail"));
        assert_eq!(detect(temp.path(), None), []);
        touch(&temp.path().join("compose.yaml"));
        let sail = if cfg!(windows) {
            Detected::Docker
        } else {
            Detected::Sail
        };
        assert_eq!(detect(temp.path(), None), [sail]);
    }

    #[test]
    fn each_compose_file_is_docker() {
        for name in [
            "compose.yaml",
            "compose.yml",
            "docker-compose.yaml",
            "docker-compose.yml",
        ] {
            let temp = tempfile::tempdir().unwrap();
            touch(&temp.path().join(name));
            assert_eq!(detect(temp.path(), None), [Detected::Docker], "{name}");
        }
    }

    #[test]
    fn herd_comes_before_sail() {
        let (temp, root, valet) = herd();
        park(&temp, &valet);
        touch(&root.join("vendor/bin/sail"));
        touch(&root.join("docker-compose.yml"));
        let sail = if cfg!(windows) {
            Detected::Docker
        } else {
            Detected::Sail
        };
        assert_eq!(detect(&root, Some(&valet)), [Detected::Herd, sail]);
    }
}
