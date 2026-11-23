//! The generators read from an app (P3), kept in Mortise's data folder.
//!
//! The file is `<data_dir>/catalogs/<key>.json`, with the key the history of runs
//! gives the app folder. It holds the package version it was read with, and is
//! used only while the app has that version (design D4).

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::generate::catalog::{Catalog, Generator};
use crate::modules::composer::Version;
use crate::run::history::app_key;

#[derive(Serialize)]
struct Stored<'a> {
    package: &'a str,
    read_ms: i64,
    generators: &'a [Generator],
}

#[derive(Deserialize)]
struct Loaded {
    package: String,
    read_ms: i64,
    generators: Vec<Generator>,
}

/// The catalog read from the app in `app` and when, in ms since 1970, while the
/// app's package is still `version`. A missing, damaged or stale file gives `None`.
pub fn load(data_dir: &Path, app: &Path, version: &Version) -> Option<(Catalog, i64)> {
    let json = std::fs::read(file(data_dir, app)).ok()?;
    let loaded: Loaded = serde_json::from_slice(&json).ok()?;
    (loaded.package == version.raw).then(|| {
        let catalog = Catalog {
            generators: loaded.generators,
        };
        (catalog, loaded.read_ms)
    })
}

/// Keeps the catalog read from the app in `app` with the package `version`.
/// It is written through a temporary file beside it, then renamed.
pub fn store(
    data_dir: &Path,
    app: &Path,
    version: &Version,
    catalog: &Catalog,
    read_ms: i64,
) -> std::io::Result<()> {
    let path = file(data_dir, app);
    let dir = path.parent().unwrap_or(data_dir);
    std::fs::create_dir_all(dir)?;
    let stored = Stored {
        package: &version.raw,
        read_ms,
        generators: &catalog.generators,
    };
    let json = serde_json::to_vec(&stored).expect("a catalog is plain data");
    // A temporary file in the same folder, so the rename stays on one volume.
    let mut partial = tempfile::NamedTempFile::new_in(dir)?;
    partial.write_all(&json)?;
    partial.persist(&path)?;
    Ok(())
}

fn file(data_dir: &Path, app: &Path) -> PathBuf {
    data_dir
        .join("catalogs")
        .join(format!("{}.json", app_key(app)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(raw: &str) -> Version {
        Version {
            raw: raw.to_string(),
            alias: None,
        }
    }

    #[test]
    fn a_stored_catalog_loads_for_its_version_only() {
        let dir = tempfile::tempdir().unwrap();
        let app = Path::new("/code/shop-api");
        let catalog = Catalog::v13_4();
        assert_eq!(load(dir.path(), app, &version("v13.4.0")), None);

        store(
            dir.path(),
            app,
            &version("v13.4.0"),
            catalog,
            1_700_000_000_000,
        )
        .unwrap();
        assert_eq!(
            load(dir.path(), app, &version("v13.4.0")),
            Some((catalog.clone(), 1_700_000_000_000))
        );
        // A new package version makes it stale.
        assert_eq!(load(dir.path(), app, &version("v13.4.1")), None);
        // A damaged file is no cache.
        std::fs::write(file(dir.path(), app), "{ not json").unwrap();
        assert_eq!(load(dir.path(), app, &version("v13.4.0")), None);
    }
}
