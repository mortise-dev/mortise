//! The public surface of a module: a scan that proposes one.
//!
//! The surface is everything of a module that something outside it may use: classes
//! for other modules, routes for HTTP clients, console commands for the command line
//! and the scheduler. The scan reads the files of the module and proposes a list.
//! It writes nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::app::AppSnapshot;
use crate::kind::{Kinds, TESTS};
use crate::layout::Layout;
use crate::module::{files_of, is_php};
use crate::php::{DeclaredKind, analyze};

/// The folder kind of the package's command generator.
const COMMANDS: &str = "Commands";
/// The folder kind of the package's event generator.
const EVENTS: &str = "Events";

/// What an entry of a public surface is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SurfaceKind {
    /// An interface the module declares.
    Contract,
    /// A class in the module's events folder.
    Event,
    /// A class that another module uses and that is neither a contract nor an event.
    OtherClass,
    /// The routes of the module under one prefix.
    Route,
    /// A console command of the module.
    Command,
}

/// One thing a module offers to the outside.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceEntry {
    pub kind: SurfaceKind,
    /// The full class name, the route pattern or the command name.
    pub name: String,
    /// For a class: the kind of the folder its file is in, such as `Contracts` or `Models`.
    pub folder: String,
    /// For a class: the modules that use it today, each with how many of its files do.
    pub used_by: Vec<(String, usize)>,
    /// For a route pattern: how many routes it stands for.
    pub routes: usize,
    /// For a command: the schedule of a scheduled task that runs it, as the app wrote it.
    pub schedule: Option<String>,
    /// True when the entry is in the proposal. The developer can change it.
    pub proposed: bool,
    /// True for an entry the developer added by hand.
    pub by_hand: bool,
}

impl SurfaceEntry {
    fn new(kind: SurfaceKind, name: String) -> Self {
        Self {
            kind,
            name,
            folder: String::new(),
            used_by: Vec::new(),
            routes: 0,
            schedule: None,
            proposed: true,
            by_hand: false,
        }
    }

    /// How many files outside the module use the entry.
    pub fn used_files(&self) -> usize {
        self.used_by.iter().map(|(_, files)| files).sum()
    }
}

/// What a name typed by hand turned out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Typed {
    /// A class the module declares, in this folder.
    Class { folder: String },
    /// A console command of the module.
    Command,
    /// A route pattern that stands for this many routes of the module.
    Route { routes: usize },
    /// It is in the proposal already.
    Listed,
    /// Nothing in the module has this name.
    Unknown,
}

/// What a scan found, as a list the developer can change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub module: String,
    /// Contracts, events, other classes, routes, then commands. In each kind what is
    /// used from outside comes first.
    pub entries: Vec<SurfaceEntry>,
    /// False when no routes were given to the scan: no environment is connected.
    pub routes_read: bool,
    /// Every class the module declares outside its tests, with the kind of its folder.
    declared: BTreeMap<String, String>,
    /// The names of the module's console commands.
    commands: BTreeSet<String>,
    /// The URIs of the module's routes.
    uris: Vec<String>,
}

impl Proposal {
    /// How many entries the scan found or the developer added.
    pub fn found(&self) -> usize {
        self.entries.len()
    }

    /// How many entries are in the proposal.
    pub fn proposed(&self) -> usize {
        self.entries.iter().filter(|entry| entry.proposed).count()
    }

    /// The entries of one kind.
    pub fn of(&self, kind: SurfaceKind) -> impl Iterator<Item = (usize, &SurfaceEntry)> {
        self.entries
            .iter()
            .enumerate()
            .filter(move |(_, entry)| entry.kind == kind)
    }

    /// Says what a name typed by hand is in this module.
    pub fn check(&self, typed: &str) -> Typed {
        let typed = typed.trim().trim_start_matches('\\');
        if typed.is_empty() {
            return Typed::Unknown;
        }
        if self.entries.iter().any(|entry| entry.name == typed) {
            return Typed::Listed;
        }
        if let Some(folder) = self.declared.get(typed) {
            return Typed::Class {
                folder: folder.clone(),
            };
        }
        if self.commands.contains(typed) {
            return Typed::Command;
        }
        let routes = self
            .uris
            .iter()
            .filter(|uri| pattern_matches(typed, uri))
            .count();
        if routes > 0 {
            Typed::Route { routes }
        } else {
            Typed::Unknown
        }
    }

    /// Adds an entry by hand, when the name is something of the module. Returns
    /// false when it is not, or when it is listed already.
    pub fn add(&mut self, typed: &str) -> bool {
        let name = typed.trim().trim_start_matches('\\').to_owned();
        let mut entry = match self.check(&name) {
            Typed::Class { folder } => {
                let mut entry = SurfaceEntry::new(SurfaceKind::OtherClass, name);
                entry.folder = folder;
                entry
            }
            Typed::Command => SurfaceEntry::new(SurfaceKind::Command, name),
            Typed::Route { routes } => {
                let mut entry = SurfaceEntry::new(SurfaceKind::Route, name);
                entry.routes = routes;
                entry
            }
            Typed::Listed | Typed::Unknown => return false,
        };
        entry.by_hand = true;
        // After the last entry of its kind, so that the kinds stay together.
        let at = self
            .entries
            .iter()
            .rposition(|other| other.kind <= entry.kind)
            .map_or(0, |last| last + 1);
        self.entries.insert(at, entry);
        true
    }

    /// Takes over what the developer chose in an earlier proposal, where the entry
    /// still exists: what was left out stays out, what was added by hand stays in.
    pub fn keep_choices_of(&mut self, earlier: &Proposal) {
        for entry in &earlier.entries {
            let same = self
                .entries
                .iter_mut()
                .find(|other| other.kind == entry.kind && other.name == entry.name);
            match same {
                Some(same) => same.proposed = entry.proposed,
                None if entry.by_hand => {
                    if self.add(&entry.name)
                        && let Some(added) = self
                            .entries
                            .iter_mut()
                            .find(|other| other.name == entry.name)
                    {
                        added.proposed = entry.proposed;
                    }
                }
                None => {}
            }
        }
    }

    /// The proposal as text, for a place the package does not give yet: the entries
    /// that are in it, by kind.
    pub fn as_text(&self) -> String {
        let mut text = format!("Public surface of {}, proposed by Mortise\n", self.module);
        let groups = [
            (SurfaceKind::Contract, "Contracts"),
            (SurfaceKind::Event, "Events"),
            (SurfaceKind::OtherClass, "Other classes"),
            (SurfaceKind::Route, "Routes"),
            (SurfaceKind::Command, "Console commands"),
        ];
        for (kind, title) in groups {
            let names: Vec<&str> = self
                .of(kind)
                .filter(|(_, entry)| entry.proposed)
                .map(|(_, entry)| entry.name.as_str())
                .collect();
            if names.is_empty() {
                continue;
            }
            text.push_str(&format!("\n{title}\n"));
            for name in names {
                text.push_str(&format!("  {name}\n"));
            }
        }
        text
    }
}

/// How far a scan is, for the lines of the scanning view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScanCounts {
    /// Classes of the module that other modules use, and in how many files.
    pub used_classes: usize,
    pub used_files: usize,
    pub contracts: usize,
    pub events: usize,
    pub commands: usize,
}

/// A class the module declares, as the scan needs it.
struct Found {
    class: String,
    interface: bool,
    /// The kind of the folder its file is in.
    folder: String,
    command: Option<String>,
}

/// A scan in progress. It reads the files of one module, a few at each step, so
/// that the interface can show how far it is and stop it.
pub struct SurfaceScan {
    module: String,
    kinds: Kinds,
    /// The module's PHP files outside its tests: the path, and the path inside the module.
    files: Vec<(PathBuf, String)>,
    next: usize,
    found: Vec<Found>,
    /// Each class of the module that another module uses, with the files that use
    /// it by the module they are in.
    used: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
}

impl SurfaceScan {
    /// Starts a scan of one module of the app. `None` when the app has no such module.
    pub fn start(app: &AppSnapshot, module: &str) -> Option<Self> {
        let found = app.module(module)?;
        let (layout, config) = Layout::read(&app.root);
        let kinds = Kinds::new(config.as_ref(), &layout.app_folder);
        let files = files_of(&found.dir)
            .into_iter()
            .filter(|(_, relative)| is_php(relative) && kinds.of(relative) != TESTS)
            .collect();

        // Classes that other modules use: the links Mortise already has.
        let mut used: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
        for link in app.links.iter().filter(|link| link.to == module) {
            for reference in &link.refs {
                used.entry(reference.class.clone())
                    .or_default()
                    .entry(link.from.clone())
                    .or_default()
                    .insert(reference.file.clone());
            }
        }
        Some(Self {
            module: module.to_owned(),
            kinds,
            files,
            next: 0,
            found: Vec::new(),
            used,
        })
    }

    /// How many files the scan reads in all.
    pub fn total(&self) -> usize {
        self.files.len()
    }

    /// How many files are read.
    pub fn read(&self) -> usize {
        self.next
    }

    pub fn module(&self) -> &str {
        &self.module
    }

    /// Reads up to `files` more files. Returns true when every file is read.
    pub fn step(&mut self, files: usize) -> bool {
        let end = (self.next + files).min(self.files.len());
        for (path, relative) in &self.files[self.next..end] {
            let folder = self.kinds.of(relative);
            let analysis = analyze(&std::fs::read(path).unwrap_or_default());
            for declared in analysis.declares {
                // An older command has its name in `$name`.
                let named = declared.name_property.filter(|_| folder == COMMANDS);
                self.found.push(Found {
                    class: declared.class,
                    interface: declared.kind == DeclaredKind::Interface,
                    folder: folder.clone(),
                    command: declared.command.or(named),
                });
            }
        }
        self.next = end;
        self.next == self.files.len()
    }

    /// What was found so far.
    pub fn counts(&self) -> ScanCounts {
        let files: BTreeSet<&String> = self
            .used
            .values()
            .flat_map(|users| users.values().flatten())
            .collect();
        ScanCounts {
            used_classes: self.used.len(),
            used_files: files.len(),
            contracts: self.found.iter().filter(|found| found.interface).count(),
            events: self.found.iter().filter(|found| is_event(found)).count(),
            commands: self
                .found
                .iter()
                .filter(|found| found.command.is_some())
                .count(),
        }
    }

    /// Makes the proposal from what was read.
    ///
    /// `routes` are the URIs of the module's routes, when a running app gave them.
    /// `scheduled` are the scheduled tasks of the app: a target and its schedule.
    pub fn finish(self, routes: Option<&[String]>, scheduled: &[(String, String)]) -> Proposal {
        let users = |class: &str| -> Vec<(String, usize)> {
            let mut users: Vec<(String, usize)> = self
                .used
                .get(class)
                .map(|users| {
                    users
                        .iter()
                        .map(|(module, files)| (module.clone(), files.len()))
                        .collect()
                })
                .unwrap_or_default();
            users.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            users
        };
        let class_entry = |kind: SurfaceKind, found: &Found| {
            let mut entry = SurfaceEntry::new(kind, found.class.clone());
            entry.folder = found.folder.clone();
            entry.used_by = users(&found.class);
            entry
        };
        // What is used from outside first, then by name.
        let sorted = |mut entries: Vec<SurfaceEntry>| {
            entries.sort_by(|a, b| {
                b.used_files()
                    .cmp(&a.used_files())
                    .then_with(|| a.name.cmp(&b.name))
            });
            entries
        };

        let contracts: Vec<SurfaceEntry> = self
            .found
            .iter()
            .filter(|found| found.interface)
            .map(|found| class_entry(SurfaceKind::Contract, found))
            .collect();
        let events: Vec<SurfaceEntry> = self
            .found
            .iter()
            .filter(|found| is_event(found))
            .map(|found| class_entry(SurfaceKind::Event, found))
            .collect();
        // A class another module uses that is neither: shown, and not proposed.
        let listed: BTreeSet<&str> = contracts
            .iter()
            .chain(&events)
            .map(|entry| entry.name.as_str())
            .collect();
        let others: Vec<SurfaceEntry> = self
            .used
            .keys()
            .filter(|class| !listed.contains(class.as_str()))
            .map(|class| {
                let mut entry = SurfaceEntry::new(SurfaceKind::OtherClass, class.clone());
                entry.folder = self
                    .found
                    .iter()
                    .find(|found| &found.class == class)
                    .map(|found| found.folder.clone())
                    .unwrap_or_default();
                entry.used_by = users(class);
                entry.proposed = false;
                entry
            })
            .collect();

        // The routes of the module, by their prefix.
        let mut patterns: BTreeMap<String, usize> = BTreeMap::new();
        for uri in routes.unwrap_or_default() {
            *patterns.entry(pattern_of(uri)).or_default() += 1;
        }
        let mut route_entries: Vec<SurfaceEntry> = patterns
            .into_iter()
            .map(|(pattern, count)| {
                let mut entry = SurfaceEntry::new(SurfaceKind::Route, pattern);
                entry.routes = count;
                entry
            })
            .collect();
        route_entries.sort_by(|a, b| b.routes.cmp(&a.routes).then_with(|| a.name.cmp(&b.name)));

        let mut commands: Vec<SurfaceEntry> = self
            .found
            .iter()
            .filter_map(|found| found.command.as_ref())
            .map(|name| {
                let mut entry = SurfaceEntry::new(SurfaceKind::Command, name.clone());
                // A scheduled task runs the command when its target names it.
                entry.schedule = scheduled
                    .iter()
                    .find(|(target, _)| target.split_whitespace().any(|word| word == name))
                    .map(|(_, schedule)| schedule.clone());
                entry
            })
            .collect();
        commands.sort_by(|a, b| {
            b.schedule
                .is_some()
                .cmp(&a.schedule.is_some())
                .then_with(|| a.name.cmp(&b.name))
        });

        let mut entries = sorted(contracts);
        entries.extend(sorted(events));
        entries.extend(sorted(others));
        entries.extend(route_entries);
        entries.extend(commands);
        Proposal {
            module: self.module,
            entries,
            routes_read: routes.is_some(),
            declared: self
                .found
                .iter()
                .map(|found| (found.class.clone(), found.folder.clone()))
                .collect(),
            commands: self
                .found
                .iter()
                .filter_map(|found| found.command.clone())
                .collect(),
            uris: routes.unwrap_or_default().to_vec(),
        }
    }
}

fn is_event(found: &Found) -> bool {
    !found.interface && found.folder == EVENTS
}

/// The pattern a route is proposed under: its first part and everything below it,
/// or its first two parts for an API route. `api/billing/invoices/{invoice}` is
/// under `api/billing/*`.
fn pattern_of(uri: &str) -> String {
    let parts: Vec<&str> = uri.trim_matches('/').split('/').collect();
    let take = if parts.first() == Some(&"api") { 2 } else { 1 };
    let prefix: Vec<&str> = parts
        .iter()
        .copied()
        .take(take)
        .take_while(|part| !part.is_empty() && !part.starts_with('{'))
        .collect();
    if prefix.len() < take || prefix.is_empty() {
        // Nothing fixed to group by: the route stands for itself.
        return uri.trim_matches('/').to_owned();
    }
    format!("{}/*", prefix.join("/"))
}

/// True when a route is under a pattern: the pattern itself, or with `/*` at its
/// end, the prefix and everything below it.
fn pattern_matches(pattern: &str, uri: &str) -> bool {
    let (pattern, uri) = (pattern.trim_matches('/'), uri.trim_matches('/'));
    match pattern.strip_suffix("/*") {
        Some(prefix) => {
            uri == prefix
                || uri
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with('/'))
        }
        None => pattern == uri,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_route_is_proposed_under_its_prefix() {
        assert_eq!(
            pattern_of("api/billing/invoices/{invoice}"),
            "api/billing/*"
        );
        assert_eq!(pattern_of("billing/invoices"), "billing/*");
        assert_eq!(pattern_of("checkout"), "checkout/*");
        assert_eq!(pattern_of("/api/billing"), "api/billing/*");
        // Nothing fixed to group by.
        assert_eq!(pattern_of("{tenant}/invoices"), "{tenant}/invoices");
        assert_eq!(
            pattern_of("api/{version}/invoices"),
            "api/{version}/invoices"
        );
        assert_eq!(pattern_of("api"), "api");

        assert!(pattern_matches("billing/*", "billing"));
        assert!(pattern_matches("billing/*", "billing/invoices/{invoice}"));
        assert!(!pattern_matches("billing/*", "billings/list"));
        assert!(pattern_matches("checkout/confirm", "checkout/confirm"));
        assert!(!pattern_matches("checkout/confirm", "checkout/confirm/now"));
    }
}
