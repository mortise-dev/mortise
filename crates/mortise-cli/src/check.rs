//! `mortise check`: reads one app at one commit and fails when a finding breaks it.
//!
//! The findings come from `mortise-core` as the window shows them. This file only
//! places each one on a file and a line, and prints it.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use mortise_core::{
    AppSnapshot, Check, CoverageSettings, Detail, Finding, RULES_FILE, ReadCache, ReadError,
    ReadOptions, RulesSource, Severity, read_app_cached,
};

use crate::{printable, say, strings};

pub(crate) struct Options {
    /// The app folder.
    pub(crate) path: PathBuf,
    /// The clover report, relative to the current folder. Wins over `mortise.json`.
    pub(crate) coverage: Option<PathBuf>,
    /// Wins over `mortise.json`.
    pub(crate) coverage_limit: Option<u8>,
    pub(crate) format: Format,
    /// Warnings fail the check too.
    pub(crate) fail_on_warnings: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    /// One line per problem: `file:line: problem: sentence`.
    Text,
    /// `Findings` as serde writes it.
    Json,
    /// GitHub Actions workflow commands, which mark each line in the pull request.
    Github,
}

/// Runs the check and gives the exit code.
pub(crate) fn run(options: &Options) -> u8 {
    // A path in a flag is relative to the current folder, as the shell completes it.
    let report = options
        .coverage
        .as_ref()
        .map(|path| std::path::absolute(path).unwrap_or_else(|_| path.clone()));
    let read_options = ReadOptions {
        coverage_given: CoverageSettings {
            limit: options.coverage_limit,
            report,
        },
        ..ReadOptions::default()
    };
    let app = match read_app_cached(
        &options.path,
        &BTreeMap::new(),
        &read_options,
        &mut ReadCache::default(),
    ) {
        Ok(app) => app,
        // A report given with --coverage that cannot be read is a wrong value of the flag.
        Err(error @ ReadError::CoverageReport { .. }) => {
            say(&strings::unreadable_report(&error.to_string()));
            return crate::USAGE;
        }
        Err(error) => {
            say(&strings::not_an_app("check", &error.to_string()));
            return crate::NOT_AN_APP;
        }
    };
    if let RulesSource::Invalid(message) = &app.rules_source {
        say(&strings::invalid_rules_file(message));
    }
    if let Some(message) = &app.coverage_ignored {
        say(&strings::coverage_ignored(message));
    }

    let findings = &app.findings;
    let fails = findings.problems() > 0 || (options.fail_on_warnings && findings.warnings() > 0);
    let report = match options.format {
        Format::Json => {
            let mut json = serde_json::to_string_pretty(findings).expect("findings are plain data");
            json.push('\n');
            json
        }
        Format::Text | Format::Github => {
            let mut text = String::new();
            for line in findings
                .list
                .iter()
                .flat_map(|finding| lines(finding, &app))
            {
                text.push_str(&printable(&match options.format {
                    Format::Github => line.github(),
                    _ => line.text(),
                }));
                text.push('\n');
            }
            let (rules, checks) = findings.rules_and_checks();
            text.push_str(&strings::summary(
                findings.problems(),
                findings.warnings(),
                app.modules.len(),
                rules,
                checks,
            ));
            text.push('\n');
            text
        }
    };
    let mut out = std::io::stdout().lock();
    match out.write_all(report.as_bytes()).and_then(|()| out.flush()) {
        // A closed pipe, as with `| head`, is not an error of the check.
        Err(error) if error.kind() != std::io::ErrorKind::BrokenPipe => {
            say(&strings::not_written(&error.to_string()));
            return crate::NOT_WRITTEN;
        }
        _ => {}
    }

    match (fails, app.rules.fail_ci) {
        (false, _) => crate::PASSED,
        (true, true) => crate::FAILED,
        (true, false) => {
            say(strings::CI_FAIL_OFF);
            crate::PASSED
        }
    }
}

/// One line of the report: a finding, or a file of one, placed where it is.
struct Line {
    severity: Severity,
    check: Check,
    /// Relative to the app folder, with `/` between the parts.
    path: String,
    /// The line in `path`, when the finding has one.
    line: Option<u32>,
    /// True when `path` is a file. A module folder is not.
    file: bool,
    sentence: String,
}

impl Line {
    /// `Modules/Checkout/app/Services/InvoiceSync.php:7: problem: Checkout reads ...`
    fn text(&self) -> String {
        let place = match self.line {
            Some(line) => format!("{}:{line}", self.path),
            None => self.path.clone(),
        };
        format!(
            "{place}: {}: {}",
            strings::severity(self.severity),
            self.sentence
        )
    }

    /// `::error file=...,line=7,title=Public surface only::Checkout reads ...`
    fn github(&self) -> String {
        let command = match self.severity {
            Severity::Problem => "error",
            Severity::Warning => "warning",
        };
        let mut properties = Vec::new();
        if self.file {
            properties.push(format!("file={}", property(&self.path)));
            if let Some(line) = self.line {
                properties.push(format!("line={line}"));
            }
        }
        properties.push(format!("title={}", property(strings::check(self.check))));
        format!(
            "::{command} {}::{}",
            properties.join(","),
            data(&self.sentence)
        )
    }
}

/// The lines of one finding. A finding of files that read internals, or that use a module
/// outside their list, gives one line per file, so that each file is marked.
fn lines(finding: &Finding, app: &AppSnapshot) -> Vec<Line> {
    let module = finding.module.as_deref().unwrap_or_default();
    let line = |path: &str, line: Option<u32>, file: bool, sentence: String| Line {
        severity: finding.severity,
        check: finding.check,
        path: path.to_owned(),
        line,
        file,
        sentence,
    };
    let folder = app
        .module(module)
        .map(|found| app.path_shown(&found.dir))
        .unwrap_or_else(|| module.to_owned());
    let in_folder = |sentence: String| line(&folder, None, false, sentence);

    match &finding.detail {
        // On the first file of the link the window opens the circle on.
        Detail::CircularUse { files } => {
            let sentence = strings::circle(files);
            let first = app
                .circle_neighbour(module)
                .and_then(|to| app.link(module, to))
                // The first reference, as the Boundary panel lists them.
                .and_then(|link| link.refs.first());
            vec![match first {
                Some(first) => line(&first.file, Some(first.line), true, sentence),
                None => in_folder(sentence),
            }]
        }
        Detail::ReadsInternals { of, .. } => {
            // Each file, at the first line that names an internal class.
            let mut files: BTreeMap<&str, (u32, Vec<&str>)> = BTreeMap::new();
            let refs = app.link(module, of).map(|link| link.refs.as_slice());
            for reference in refs.unwrap_or_default().iter().filter(|r| r.internal) {
                let (first, classes) = files
                    .entry(&reference.file)
                    .or_insert((reference.line, Vec::new()));
                *first = (*first).min(reference.line);
                if !classes.contains(&reference.class.as_str()) {
                    classes.push(&reference.class);
                }
            }
            let folders = &app.rules.public_folders;
            let accepted = app.module(of).is_some_and(|of| of.surface.is_some());
            files
                .into_iter()
                .map(|(file, (first, mut classes))| {
                    classes.sort_unstable();
                    let sentence =
                        strings::reads_internals(module, of, &classes, folders, accepted);
                    line(file, Some(first), true, sentence)
                })
                .collect()
        }
        Detail::OutsideAllowedUse { of, .. } => {
            // Each file, at the first line that names a class of `of`.
            let mut files: BTreeMap<&str, u32> = BTreeMap::new();
            let refs = app.link(module, of).map(|link| link.refs.as_slice());
            for reference in refs.unwrap_or_default() {
                let first = files.entry(&reference.file).or_insert(reference.line);
                *first = (*first).min(reference.line);
            }
            let list = app
                .rules
                .allowed_use
                .as_ref()
                .and_then(|allowed| allowed.modules.get(module))
                .map(Vec::as_slice)
                .unwrap_or_default();
            let sentence = strings::outside_allowed_use(module, of, list);
            files
                .into_iter()
                .map(|(file, first)| line(file, Some(first), true, sentence.clone()))
                .collect()
        }
        Detail::UnknownAllowedModule { entry } => vec![line(
            RULES_FILE,
            None,
            true,
            strings::unknown_allowed_module(entry),
        )],
        Detail::InvalidManifest {
            file,
            message,
            line: at,
        } => {
            let at = u32::try_from(*at).ok();
            vec![line(
                file,
                at,
                true,
                strings::invalid_manifest(module, message),
            )]
        }
        Detail::NoManifestName { file } => {
            vec![line(file, None, true, strings::no_manifest_name(module))]
        }
        Detail::NamespaceMismatch { namespace, folder } => vec![line(
            folder,
            None,
            false,
            strings::namespace_mismatch(module, namespace, folder),
        )],
        Detail::StaleStatus { entry, file } => {
            vec![line(file, None, true, strings::stale_status(entry, file))]
        }
        Detail::LowCoverage { percent, limit } => {
            vec![in_folder(strings::low_coverage(module, *percent, *limit))]
        }
        Detail::MissingTranslations {
            locale,
            keys,
            files,
            ..
        } => vec![in_folder(strings::missing_translations(
            module, locale, *keys, *files,
        ))],
    }
}

/// The message of a workflow command, escaped as the GitHub runner reads it. Other
/// control characters are escaped for every format, by `printable`.
fn data(text: &str) -> String {
    text.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

/// A property of a workflow command, escaped as the GitHub runner reads it.
fn property(text: &str) -> String {
    data(text).replace(':', "%3A").replace(',', "%2C")
}
