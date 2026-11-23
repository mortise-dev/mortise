//! The last runs of each app, kept on disk.
//!
//! The layout is `<data_dir>/runs/<key>/runs.json` (the entries, the latest first)
//! and `<key>/<id>.log` (the output lines of one run). `<key>` names the app folder.

use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// How many runs of one app are kept.
pub const HISTORY_LIMIT: usize = 50;

/// The most bytes of output one run keeps. A line counts as its length plus one for its newline.
pub const OUTPUT_LIMIT: usize = 1_000_000;

/// How a run ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Ended {
    Done {
        ms: u64,
    },
    Failed {
        code: Option<i32>,
        ms: u64,
    },
    /// Stopped by the user, or at the time limit when `limit` is true.
    Stopped {
        ms: u64,
        limit: bool,
    },
}

/// One run in the history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The start time in ms since 1970. It is one more when that id is taken.
    pub id: u64,
    pub args: Vec<String>,
    pub label: String,
    pub started_ms: i64,
    pub ended: Ended,
    /// True when the log lost lines to the output limit.
    pub cut: bool,
}

#[derive(Serialize, Deserialize)]
struct File {
    app: String,
    runs: Vec<Entry>,
}

/// The history of one app.
#[derive(Debug, Clone)]
pub struct History {
    dir: PathBuf,
    app: PathBuf,
}

impl History {
    /// The history of the app in `app`. Nothing is created until `record`.
    pub fn open(data_dir: &Path, app: &Path) -> Self {
        let key = format!("{:016x}", fnv1a(app.to_string_lossy().as_bytes()));
        Self {
            dir: data_dir.join("runs").join(key),
            app: app.to_owned(),
        }
    }

    /// The runs, the latest first. A missing or damaged file gives none.
    pub fn entries(&self) -> Vec<Entry> {
        std::fs::read(self.dir.join("runs.json"))
            .ok()
            .and_then(|json| serde_json::from_slice::<File>(&json).ok())
            .map(|file| file.runs)
            .unwrap_or_default()
    }

    /// The lines of one run. A missing log gives none.
    pub fn output(&self, id: u64) -> Vec<String> {
        std::fs::read_to_string(self.log(id))
            .map(|text| text.lines().map(str::to_owned).collect())
            .unwrap_or_default()
    }

    /// Keeps a run and its output. The oldest runs past [`HISTORY_LIMIT`] go, with their logs.
    /// A run with the id of a kept run replaces it.
    pub fn record(&self, mut entry: Entry, lines: &[String]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;

        let mut kept = String::new();
        for line in lines {
            if kept.len() + line.len() + 1 > OUTPUT_LIMIT {
                entry.cut = true;
                break;
            }
            kept.push_str(line);
            kept.push('\n');
        }
        std::fs::write(self.log(entry.id), kept)?;

        let mut runs = self.entries();
        runs.retain(|run| run.id != entry.id);
        runs.insert(0, entry);
        runs.truncate(HISTORY_LIMIT);

        let file = File {
            app: self.app.to_string_lossy().into_owned(),
            runs,
        };
        let json = serde_json::to_vec_pretty(&file).expect("history is plain data");
        // A temporary file in the same folder, so the rename stays on one volume.
        let mut partial = tempfile::NamedTempFile::new_in(&self.dir)?;
        partial.write_all(&json)?;
        partial.persist(self.dir.join("runs.json"))?;

        self.remove_unnamed_logs(&file.runs);
        Ok(())
    }

    fn log(&self, id: u64) -> PathBuf {
        self.dir.join(format!("{id}.log"))
    }

    /// Deletes each `.log` that no entry names. A failure leaves the file for the next record.
    fn remove_unnamed_logs(&self, runs: &[Entry]) {
        let named: HashSet<String> = runs.iter().map(|run| format!("{}.log", run.id)).collect();
        let Ok(files) = std::fs::read_dir(&self.dir) else {
            return;
        };
        for file in files.flatten() {
            let name = file.file_name().to_string_lossy().into_owned();
            if name.ends_with(".log") && !named.contains(&name) {
                let _ = std::fs::remove_file(file.path());
            }
        }
    }
}

/// The folder for data Mortise keeps: `MORTISE_DATA_DIR`, or the data folder of the platform.
pub fn data_dir() -> Option<PathBuf> {
    match std::env::var_os("MORTISE_DATA_DIR") {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => Some(
            directories::ProjectDirs::from("", "", "Mortise")?
                .data_dir()
                .to_owned(),
        ),
    }
}

/// FNV-1a, 64 bits. It is stable across releases and needs no crate.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: u64) -> Entry {
        Entry {
            id,
            args: vec!["about".into()],
            label: "about".into(),
            started_ms: id as i64,
            ended: Ended::Done { ms: 5 },
            cut: false,
        }
    }

    fn history(dir: &Path) -> History {
        History::open(dir, Path::new("/code/shop-api"))
    }

    #[test]
    fn the_51st_run_drops_the_oldest_and_its_log() {
        let dir = tempfile::tempdir().unwrap();
        let history = history(dir.path());
        assert!(history.entries().is_empty());
        for id in 1..=51 {
            history.record(entry(id), &[format!("run {id}")]).unwrap();
        }
        let entries = history.entries();
        assert_eq!(entries.len(), HISTORY_LIMIT);
        assert_eq!(entries[0].id, 51);
        assert_eq!(entries[49].id, 2);
        assert!(history.output(1).is_empty());
        assert_eq!(history.output(2), vec!["run 2"]);
    }

    #[test]
    fn a_damaged_file_reads_empty_and_the_next_record_works() {
        let dir = tempfile::tempdir().unwrap();
        let history = history(dir.path());
        history.record(entry(1), &[]).unwrap();
        std::fs::write(history.dir.join("runs.json"), "{ not json").unwrap();
        assert!(history.entries().is_empty());
        history.record(entry(2), &["ok".into()]).unwrap();
        assert_eq!(history.entries(), vec![entry(2)]);
        // The log of the lost run has no entry, so it is gone.
        assert!(!history.log(1).exists());
    }

    #[test]
    fn output_past_the_limit_is_cut() {
        let dir = tempfile::tempdir().unwrap();
        let history = history(dir.path());
        let lines = vec!["x".repeat(999); 1_100];
        history.record(entry(1), &lines).unwrap();
        assert!(history.entries()[0].cut);
        assert!(std::fs::metadata(history.log(1)).unwrap().len() <= OUTPUT_LIMIT as u64);
        assert_eq!(history.output(1).len(), 1_000);
    }
}
