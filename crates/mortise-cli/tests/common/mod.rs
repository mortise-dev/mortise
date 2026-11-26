//! What the tests of the command line tool share.

use std::path::{Path, PathBuf};

pub fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shop-api")
}

/// A copy of shop-api that a test may change. It is in no git repository.
pub fn copy_of_fixture() -> tempfile::TempDir {
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
    let app = tempfile::tempdir().unwrap();
    copy_dir(&fixture(), app.path());
    app
}
