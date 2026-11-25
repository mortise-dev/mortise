//! Interface text of the Boundary tab (section 10 of the specification).

pub const SELECTED: &str = "Selected";
pub const ALL_LINKS: &str = "All links";
pub const PROBLEMS_ONLY: &str = "Problems only";
pub const LINKS_THAT_BREAK: &str = "Links that break a rule";
pub const RULE_FAIL_CI: &str = "Fail CI when a rule breaks";
pub const RULES_KEPT: &str = "kept in mortise.json in the app folder";
pub const GRAPH_CAPTION: &str = "Arrow points at the module that is used. A square with a cross marks a link that breaks a rule.";
pub const COPY_PATH: &str = "Copy path";
/// What a screen reader says for "Open in editor" under the files of a link.
pub const OPEN_FILE_IN_EDITOR: &str = "Open the selected file in the editor";

pub const COL_FILE: &str = "File";
pub const COL_LINE: &str = "Line";
pub const COL_READS: &str = "Reads";
pub const COL_FROM: &str = "From";
pub const COL_REFS: &str = "Refs";
pub const COL_STATE: &str = "State";
pub const CIRCULAR: &str = "Circular";
pub const FILTER_LINKS: &str = "Filter links";
/// The name of the folders field of the second rule, for a screen reader. Not in the spec.
pub const PUBLIC_FOLDERS: &str = "Folders that count as public";

/// `Used by, 2`
pub fn used_by_count(n: usize) -> String {
    format!("Used by, {n}")
}

/// `Uses, 4`
pub fn uses_count(n: usize) -> String {
    format!("Uses, {n}")
}

/// `Checkout reads Billing internals`
pub fn reads_internals(a: &str, b: &str) -> String {
    format!("{a} reads {b} internals")
}

/// `3 of 6 files read internals`
pub fn files_read_internals(internal: usize, files_count: usize) -> String {
    format!(
        "{internal} of {} read internals",
        crate::strings::files(files_count as u64)
    )
}

/// `3 files read internals`
pub fn n_files_read_internals(internal: usize) -> String {
    if internal == 1 {
        "1 file reads internals".to_owned()
    } else {
        format!("{internal} files read internals")
    }
}

/// `Circular, 6 files`
pub fn circular_files(files_count: usize) -> String {
    format!("Circular, {}", crate::strings::files(files_count as u64))
}

/// `9 links between 7 modules`
pub fn links_between(links: usize, modules_count: usize) -> String {
    let links = if links == 1 {
        "1 link".to_owned()
    } else {
        format!("{links} links")
    };
    format!("{links} between {}", crate::strings::modules(modules_count))
}

/// `214 links, 5 problems, 14 shown`
pub fn links_shown(links: usize, problem_count: usize, shown: usize) -> String {
    let links = if links == 1 {
        "1 link".to_owned()
    } else {
        format!("{links} links")
    };
    format!(
        "{links}, {}, {shown} shown",
        crate::strings::problems(problem_count)
    )
}

/// `12 more`
pub fn more(n: usize) -> String {
    format!("{n} more")
}

/// `Rules for shop-api`
pub fn rules_for(app: &str) -> String {
    format!("Rules for {app}")
}

/// `Accepted for 0 of 7 modules. For a module without one, these folders count:`
pub fn accepted_for(accepted: usize, total: usize) -> String {
    format!(
        "Accepted for {accepted} of {}. For a module without one, these folders count:",
        crate::strings::modules(total)
    )
}

/// `Rule: other modules use only a module's public surface. Billing has none yet, so the
/// folders Contracts and Events count.`
pub fn rule_of_link(module: &str, rules: &mortise_core::Rules) -> String {
    public_surface_rule(module, "has", rules)
}

/// Not in the spec: the rule under the files of two modules that use each other while
/// the rule "No circular use" is off.
/// `Rule: other modules use only a module's public surface. Checkout and Shipping have
/// none yet, so the folders Contracts and Events count.`
pub fn rule_of_links(a: &str, b: &str, rules: &mortise_core::Rules) -> String {
    public_surface_rule(&format!("{a} and {b}"), "have", rules)
}

fn public_surface_rule(modules: &str, has: &str, rules: &mortise_core::Rules) -> String {
    if !rules.public_surface_only {
        // Not in the spec: the rule is turned off.
        return "The rule about a module's public surface is turned off for this app.".to_owned();
    }
    let folders = match rules.public_folders.as_slice() {
        [] => "no folders count".to_owned(),
        [one] => format!("the folder {one} counts"),
        [rest @ .., last] => format!("the folders {} and {last} count", rest.join(", ")),
    };
    format!(
        "Rule: other modules use only a module's public surface. {modules} {has} none yet, so {folders}."
    )
}

/// `Reported on Checkout, the side with fewer files to change.`
pub fn reported_on(module: &str) -> String {
    format!(
        "Reported on {module}, the side with fewer files to change. Every file in a circle is \
         marked: removing either direction ends it."
    )
}

/// Not in the spec: the footer of a link in a circle of three or more modules,
/// where removing one direction need not end the circle.
pub fn reported_on_in_circle(module: &str) -> String {
    format!(
        "Reported on {module}, the module of the circle with the fewest files to change. Every \
         file of a link in a circle is marked."
    )
}

/// `platform has 60 modules. Above 12 modules the graph of the whole app gives way to this table.`
pub fn many_modules(app: &str, modules_count: usize) -> String {
    format!(
        "{app} has {modules_count} modules. Above 12 modules the graph of the whole app gives way \
         to this table. Select one module for its own graph."
    )
}

/// Not in the spec: an app whose modules do not use each other.
pub fn no_links(app: &str) -> String {
    format!("No module of {app} uses another.")
}

/// Not in the spec: a module that uses no other module and is used by none.
pub fn no_links_of(module: &str) -> String {
    format!("{module} uses no other module, and no other module uses it.")
}

/// Not in the spec: mortise.json is there but cannot be read as rules.
pub fn rules_invalid(message: &str) -> String {
    format!("mortise.json is not valid: {message}. The default rules apply until it is corrected.")
}

pub const LINKS: &str = "Links";
pub const SWITCH_WITH: &str = "Switch with";
pub const SCAN_AGAIN: &str = "Scan again";
pub const STOP_THE_SCAN: &str = "Stop the scan";
pub const NOTHING_IS_WRITTEN: &str = "nothing is written";
pub const CLASSES_OTHERS_USE: &str = "Classes that other modules use";
pub const CONTRACTS: &str = "Contracts";
pub const EVENTS: &str = "Events";
pub const OTHER_CLASSES: &str = "Other classes that other modules use";
pub const CONSOLE_COMMANDS: &str = "Console commands";
pub const COL_ENTRY: &str = "Entry";
pub const COL_WHY: &str = "Why it is here";
pub const COL_USED_TODAY: &str = "Used today by";
pub const AN_EVENT: &str = "an event";
pub const ROUTES_WITH_PREFIX: &str = "routes with this prefix";
pub const A_CONSOLE_COMMAND: &str = "a console command";
pub const NOT_USED_OUTSIDE: &str = "not used outside yet";
pub const ADD_BY_HAND: &str = "Add a class, a route or a command";
pub const CANNOT_STORE_SURFACE: &str = "The package cannot store a public surface yet";
pub const ROUTES_NOT_READ: &str =
    "Routes were not read. Connect an environment or set how PHP runs, then scan again.";
/// Not in the spec as a control. The spec says that the proposal can be copied as text.
pub const COPY_AS_TEXT: &str = "Copy as text";
/// Not in the spec: what the toast says after "Copy as text".
pub const COPIED_PROPOSAL: &str = "Copied the proposal.";
/// Not in the spec: an entry the developer added.
pub const ADDED_BY_HAND: &str = "added by hand";
/// Not in the spec: the routes line of the scanning view when no environment is connected.
pub const NOT_READ: &str = "not read";

/// `Billing has no public surface yet`
pub fn no_surface_yet(module: &str) -> String {
    format!("{module} has no public surface yet")
}

pub fn scan_explained(module: &str, folders: &[String]) -> String {
    let names: Vec<&str> = folders.iter().map(String::as_str).collect();
    let rule = match names.len() {
        0 => "Until then the rule has no folders that count as public.".to_owned(),
        1 => format!("Until then the rule uses the folder {}.", names[0]),
        _ => format!(
            "Until then the rule uses the folders {}.",
            crate::strings::and_list(&names)
        ),
    };
    format!(
        "A scan reads the files of {module} and proposes what other modules, HTTP clients and \
         the command line may use. Nothing is written until you accept the proposal. {rule}"
    )
}

/// `Scan Billing`
pub fn scan_module(module: &str) -> String {
    format!("Scan {module}")
}

/// `Scanning Billing`
pub fn scanning(module: &str) -> String {
    format!("Scanning {module}")
}

/// `3 classes in 6 files`
pub fn classes_in_files(classes: usize, in_files: usize) -> String {
    let classes = match classes {
        1 => "1 class".to_owned(),
        n => format!("{n} classes"),
    };
    format!("{classes} in {}", crate::strings::files(in_files as u64))
}

/// `2 interfaces`
pub fn interfaces(n: usize) -> String {
    match n {
        1 => "1 interface".to_owned(),
        n => format!("{n} interfaces"),
    }
}

/// `3 events`
pub fn events(n: usize) -> String {
    match n {
        1 => "1 event".to_owned(),
        n => format!("{n} events"),
    }
}

/// `2 commands`
pub fn commands(n: usize) -> String {
    match n {
        1 => "1 command".to_owned(),
        n => format!("{n} commands"),
    }
}

/// `Proposed public surface of Billing`
pub fn proposed_surface_of(module: &str) -> String {
    format!("Proposed public surface of {module}")
}

/// `22 found, 21 proposed`
pub fn found_proposed(found: usize, proposed: usize) -> String {
    format!("{found} found, {proposed} proposed")
}

/// `scanned just now`
pub fn scanned_ago(seconds: u64) -> String {
    match seconds {
        0..60 => "scanned just now".to_owned(),
        _ => format!("scanned {}", crate::strings::ago(seconds as i64, 0)),
    }
}

/// `an interface in Contracts`
pub fn an_interface_in(folder: &str) -> String {
    format!("an interface in {folder}")
}

/// `Checkout, 3 files`
pub fn used_by_modules(used_by: &[(String, usize)]) -> String {
    let names: Vec<&str> = used_by.iter().map(|(module, _)| module.as_str()).collect();
    let total: usize = used_by.iter().map(|(_, n)| n).sum();
    format!(
        "{}, {}",
        names.join(", "),
        crate::strings::files(total as u64)
    )
}

/// `in Models, used by Checkout, 3 files`
pub fn in_folder_used_by(folder: &str, used_by: &[(String, usize)]) -> String {
    match (folder.is_empty(), used_by.is_empty()) {
        (false, false) => format!("in {folder}, used by {}", used_by_modules(used_by)),
        (true, false) => format!("used by {}", used_by_modules(used_by)),
        (false, true) => format!("in {folder}, {ADDED_BY_HAND}"),
        (true, true) => ADDED_BY_HAND.to_owned(),
    }
}

pub fn not_proposed(modules: &[(String, usize)]) -> String {
    let names: Vec<&str> = modules.iter().map(|(module, _)| module.as_str()).collect();
    format!(
        "Used from outside, not proposed. Include it to allow these reads, or leave it out and \
         keep the finding on {}.",
        crate::strings::and_list(&names)
    )
}

/// Not in the spec: a scan that found nothing.
pub fn nothing_to_propose(module: &str) -> String {
    format!(
        "The scan found no interface, no event and no console command in {module}, and no \
         class that another module uses."
    )
}

/// `Routes, read from local`
pub fn routes_group(environment: &str) -> String {
    format!("Routes, read from {environment}")
}

/// `9 routes, HTTP clients`
pub fn routes_for_clients(n: usize) -> String {
    match n {
        1 => "1 route, HTTP clients".to_owned(),
        n => format!("{n} routes, HTTP clients"),
    }
}

/// `the scheduler, 0 6 * * *`. The schedule is as the app wrote it.
pub fn the_scheduler(schedule: &str) -> String {
    format!("the scheduler, {schedule}")
}

/// `Accept 21 entries`
pub fn accept_entries(n: usize) -> String {
    match n {
        1 => "Accept 1 entry".to_owned(),
        n => format!("Accept {n} entries"),
    }
}

/// Not in the spec: what a name typed by hand turned out to be.
pub fn typed_verdict(typed: &mortise_core::Typed, module: &str) -> String {
    use mortise_core::Typed;
    match typed {
        Typed::Class { folder } if folder.is_empty() => "A class. Enter adds it.".to_owned(),
        Typed::Class { folder } => format!("A class in {folder}. Enter adds it."),
        Typed::Command => "A console command. Enter adds it.".to_owned(),
        Typed::Route { routes: 1 } => "1 route. Enter adds it.".to_owned(),
        Typed::Route { routes } => format!("{routes} routes. Enter adds it."),
        Typed::Listed => "It is in the list.".to_owned(),
        Typed::Unknown => format!("Nothing in {module} has this name."),
    }
}

/// The accessible name of the control that switches between the links and the public surface.
pub const VIEW: &str = "View";
/// Not in the spec: an app whose links break no rule.
pub const NO_LINK_BREAKS_A_RULE: &str = "No link breaks a rule.";
