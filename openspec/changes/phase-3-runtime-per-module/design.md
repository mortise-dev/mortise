# Design

## Context

See `proposal.md` for the reasons. The actuator client (`runtime/actuator.rs`) reads `health` into a
`Runtime` whose `components` are `Component { name, status, details }`, details as `(key, text)` pairs
in the app's order. A nested JSON value, such as the `modules` detail of the `modules` component, is
kept as its JSON text. `runtime/live.rs` asks `health` every 15 s on a worker thread. The index
(`index/view.rs`) gets the brief status of the environment through `IndexRuntime`. The app runtime box
(`runtime/view.rs`, `runtime_box`) lists `runtime.components`.

What the package does was read in the source of `nwidart/laravel-modules` v13.4.0
(`src/Actuator/ModulesHealthIndicator.php` and its test) and seen in `fixtures/actuator/health.json`:

- Module indicators are named `module:{getLowerName()}:{key}`. `getLowerName()` is PHP `strtolower`,
  which is ASCII only since PHP 8.2.
- The `modules` component has one detail, `modules`, an object keyed by module name in its own case:
  `{"Billing":{"enabled":true},"Legacy":{"enabled":false}}`. A module that is enabled and missing on
  disk has `{"enabled":true,"error":"Enabled in the status store, not found on disk."}`, and the
  component is Down. Open PR #2290 may add other errors in the same place.
- A turned-off module registers no indicator.

## Goals / Non-Goals

**Goals:**

- One core function decides which component belongs to which module, and the status of each module.
  It is tested once, on the recorded `health`.
- The index row, the strip and the module Summary show the status. The app box stops repeating it.

**Non-Goals:**

- No change to what the client asks for or how often. `actuator.rs` and `live.rs` are not edited, so
  milestone 20 can change them without a conflict.
- No reading of the `modules` endpoint (decision 1).
- Scheduled tasks of a module (milestone 22).

## Decisions

### Core: `runtime/modules.rs`

Behind the feature `actuator`, re-exported from `lib.rs`:

```rust
pub struct ModuleRuntime {
    pub module: String,          // the module's name, as the app's files give it
    pub status: Status,
    pub error: Option<String>,   // from the modules component, as the app wrote it
    pub indicators: Vec<Component>, // each named by its key: "storage"
}

pub struct RuntimeByModule {
    pub app: Vec<Component>,         // the app's own components, in the app's order
    pub modules: Vec<ModuleRuntime>, // modules with a status, in the order of app.modules
}

pub fn by_module(app: &AppSnapshot, components: &[Component]) -> RuntimeByModule;
impl RuntimeByModule { pub fn of(&self, module: &str) -> Option<&ModuleRuntime>; }
```

- A name matches by `to_ascii_lowercase()`, as PHP `strtolower` does.
- The status is the worst of the indicators by the rank Down, Out of service, Up, Unknown (Spring's
  default order, decision 4). An error makes it Down. No indicator and no error: no entry.
- The `modules` detail is parsed from its JSON text with `serde_json`, which the core already uses.
  `Component` does not change, so `actuator.rs` does not either.
- The `modules` component leaves `app` when its one detail is `modules` with the list of
  laravel-modules: an object whose every entry is `{"enabled": bool}` with an optional `"error"`
  string, or `[]` for an app with no module. Any other shape stays in `app`: a check that failed (Down with `error`, no list)
  or a component of the app's own under that name. An error the list gives for a name that is no
  module of the app goes to `RuntimeByModule::elsewhere`, apart from the details, so a module named
  `error` cannot pass for the exception of the check. An indicator `module:x:y` whose `x` is no
  module stays in `app` under its full name.
- Alternative: split the components in `actuator.rs` when `health` is read. Rejected: the client does
  not know the app's modules, and it is the file milestone 20 edits.

The function is cheap (a few components, at most a few hundred modules). The app calls it once a frame
where it needs it; nothing is cached.

### The marks on a row

The spec draws the runtime stroke mark to the left of the finding mark, 8 px apart (`.row .marks`,
`gap:8px`), on a two-line row. A one-line row has the same width, so it takes the same layout: the
stroke mark 8 px left of the finding mark (or of its count), and the name is elided 20 px sooner.
Row heights do not change.

The strip is 76 px wide and has no room right of its finding mark (deviation S29). Three layouts were
weighed:

- Stack the two marks in one column. A one-line row is 26 px and two 12 px marks leave 2 px. Rejected.
- Put the stroke mark left of the stamp. The peg of the selected row sits there. Rejected.
- Widen the strip by one mark and the 8 px gap, to 96 px, while a row it shows has a runtime mark.
  Chosen: both marks keep the order and spacing of the row, and the strip keeps one look.

The width changes only when module statuses appear or go, such as when an environment connects or
stops answering. `strip_width` takes whether any row has a mark. The track of the scrollbar is added
on top as it is today. Recorded as a deviation row.

Each stroke mark is announced with its row: the row's accessible name stays the module name, and the
mark gets its own label "Billing: Down on production" with the tooltip of the header mark ("Down on
production, checked 8 s ago"). The text goes in `index/strings.rs`.

### The module box

The box sits in the module Summary above the scheduled tasks box, full width. The spec draws it beside
the tasks box in a two-column row; milestone 22 changes the tasks of a module, so this change does not
touch that layout. Recorded as a deviation row.

Its items reuse `runtime_box`'s parts in `runtime/view.rs`: a heading "Runtime on {environment}" with
the environment switch, a line "Status of {module}" with the mark, the word and "checked 8 s ago", the
error of the `modules` component as quoted code when there is one, then the indicators through
`component_items`, named by key. Each indicator has its own line, also when some are not up, and its
details read as the app wrote them: a key such as `db` or `cache` does not take the wording of the
app's component of that name. The box is drawn only when `RuntimeByModule::of` finds the module.

### The app box

`items` lists `by_module(..).app` instead of `runtime.components`. After the components, each module
that is not Up has one line: its name (not mono) and its status mark and word. A Down app then always
names what is down. Each name of `elsewhere` has a line with Down, then its sentence quoted. Not in
the spec; recorded with the other rows.

### The stub

`StubActuator::healthy()` adds to its health answer the components `module:billing:gateway` and
`modules` as recorded in `fixtures/actuator/health.json`. `shop-api` has a module Billing, so screen
tests see one Up mark. `module:blog:storage` is left out of `healthy()`: `shop-api` has no Blog, and
the core test covers that case. The core test serves the recorded file itself through `set`.

## Risks / Trade-offs

- [The S4.2 and S3.1 images change for every test that uses `healthy()`] → Billing gains a mark. The
  images are made again and looked at.
- [The strip width changes when an environment connects] → It happens on a change of connection, not
  on each answer. The width depends on statuses, which a 15 s answer rarely adds or removes.
- [A non-ASCII module name] → Matched as PHP 8.2 does, ASCII lower case only. Not tested with a real
  app.
- [Milestones 20 and 22 edit nearby files] → No edit in `actuator.rs` or `live.rs`. In `runtime/view.rs`
  only `items` and a new module box function; in `summary/view.rs` one call above the tasks box.
