# Design

## Context

See `proposal.md` for why. Milestones 10 to 13 (`openspec/changes/archive/2026-11-23-phase-2-*`) built
how PHP runs, the runner and drawer (`AppRuns::start`, `line_now`, history, Run again with
`asks_again`), the fake artisan with the recordings of `fixtures/output/`, `catalog/actions.rs`,
`output.rs`, the Actions tab (`actions_view.rs`) and its dialogs (`Dialog` with `danger`, `left`,
`code`, `typed`, `start_on_cancel`). The Generate tab draws the placeholder "Generate is not in this
version" (`workspace.rs`). `AppSnapshot::package` holds the package version from `composer.lock`.

Package facts, read at the tag `v13.3.0` (source read, not run):

- `src/Commands/Make` holds 41 file generators (`module:make-*` and `module:route-provider`), their base
  `GeneratorCommand`, and `ModuleMakeCommand` (`module:make`): 42 generators. `module:make-health-indicator`
  is on `master` only.
- `GeneratorCommand` adds `--force` ("Overwrite the file if it already exists") to every generator that
  lacks it, with `-f` only when no other option uses it. It prints the task line
  `Generating file <path> ... DONE` or `FAIL`, and on an existing file the error
  `File : <path> already exists.`. The recordings print `<path>` relative to the app (`Modules/Blog/...`).
- `module:make-model` sets the seven flags for `--all`, makes the model, then calls migration, controller,
  factory, request, resource, seed and policy, in that order, without `--force`. It returns 0 even when a
  call failed. The names it passes are `<Model>Controller`, `<Model>` (factory), `<Model>Request`,
  `<Model>Resource`, `<Model>Seeder`, `<Model>Policy` and `create_<models>_table`.
- `module:make` checks each name: `Str::studly($name)` must match `^[A-Za-z_][A-Za-z0-9_]*$`, else it prints
  "The module name [<name>] is not valid. Use letters, digits, "-", "_" or spaces, starting with a letter."
  and goes on with the next name. At the end it runs `composer dump-autoload`; on failure it prints
  "composer dump-autoload failed." and exits 1.

P3 (`module:generators --json`, PR nWidart/laravel-modules#2295, not released) prints
`{"generators":[{"name","group","description","arguments":[{"name","required","array","default",
"description"}],"options":[{"name","shortcut","accepts_value","value_required","array","negatable",
"default","description"}]}]}`, sorted by name, with groups `module`, `data`, `http`, `events`, `views`,
`classes`, `console`, `tests`, `other`, and no application-wide option.

## Goals / Non-Goals

**Goals**: the catalog in P3's shape, the name checks, the output reading, S9.1, S9.2 without the file
list, S9.4, S9.5, S9.6, the ways in, Replace.

**Non-Goals**: reading P2, P3 or P5 from the app (milestone 15); S9.3; the file list of S9.2 and S9.4;
recipes ("Save as recipe" is out of the spec). Mortise does not compute where a generator writes.

## Decisions

### D1. The catalog (`catalog/generators.rs`, `catalog/v13_3.json`, task 14.1 and 14.2)

`v13_3.json` is the P3 document for 13.3.0, written by hand from the source at the tag: the same keys,
`module:make` included, sorted by name, the package's own descriptions, `--force` listed where
`GeneratorCommand` adds it. Groups use P3's keys (`module:route-provider` is `http`).

```rust
#[derive(Deserialize)] pub struct Catalog { pub generators: Vec<Generator> }
#[derive(Deserialize)] pub struct Generator {
    pub name: String, pub group: String, pub description: String,
    pub arguments: Vec<Argument>, pub options: Vec<GeneratorOption>,
}
#[derive(Deserialize)] pub struct Argument {
    pub name: String, pub required: bool, pub array: bool,
    pub default: serde_json::Value, pub description: String,
}
#[derive(Deserialize)] pub struct GeneratorOption {
    pub name: String, pub shortcut: Option<String>, pub accepts_value: bool,
    pub value_required: bool, pub array: bool, pub negatable: bool,
    pub default: serde_json::Value, pub description: String,
}
impl Catalog {
    pub fn from_json(text: &str) -> Result<Catalog, serde_json::Error>; // the P3 document, for milestone 15
    pub fn v13_3() -> &'static Catalog;            // include_str! once, in a OnceLock
    pub fn get(&self, name: &str) -> Option<&Generator>;
    pub fn grouped(&self) -> Vec<(Group, Vec<&Generator>)>;
}
pub const MODULE_MAKE: &str = "module:make";
/// True when the shipped catalog describes this package version: 13.3.x.
pub fn shipped_for(version: &str) -> bool;
```

`default` stays a JSON value: Symfony gives `null`, `false`, a string or an array, and Mortise only
reads a string default into a field. Alternative: an enum of our own. Rejected: it would differ from P3
on the first odd default.

The groups and their order are Mortise's (S9.1): `Group::{Module, Data, Http, Events, Views, Classes,
Console, Tests, Other}` and a table of artisan names in the order S9.1 draws them, with
`module:make-health-indicator` under Classes as S9.1 draws it. `grouped` follows that table; a generator
it does not name goes to Other, sorted by name, whatever its P3 group. An empty group is left out.
The labels ("Model", "Seeder", "Component class", "Stub replacement") are interface text in
`strings.rs`; a generator with no label shows its artisan name without `module:` (not in the spec).

### D2. The words of a generator and the name checks (`catalog/generators.rs`, task 14.2)

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filled {
    pub arguments: BTreeMap<String, String>,  // by argument name; the module argument is not here
    pub flags: BTreeSet<String>,              // options without a value that are on
    pub values: BTreeMap<String, String>,     // options with a value
}
impl Generator {
    pub fn words(&self, filled: &Filled, module: Option<&str>) -> Vec<String>;
    pub fn name_rule(&self, argument: &str) -> NameRule;
}
pub enum NameRule { Class, Module, Free }
pub fn class_name_ok(name: &str) -> bool;
pub fn module_name_ok(name: &str) -> bool;
```

`words` gives: the artisan name; the arguments in their order (the `module` argument from `module`; an
array argument split on white space into one word each; an empty optional argument left out); then
`--<name>` for each flag that is on, in the order of the options; then `--<name>=<value>` for each value,
trimmed, left out when empty, in the order of the options. Flags come before values because S9.2 draws
`--migration --policy --fillable=...`, while the package lists `fillable` between `controller` and
`factory`. Each value is one word, so nothing is split or read by a shell. Long names only, as S9.2
draws them. A flag or value the generator does not have is ignored.

For `module:make-model`, `--all` stands in for the seven flags when all seven are on or `all` is on
(S9.2 "All of these"). If any argument word starts with `-`, `--` and the argument words go last, as in
`catalog/actions.rs`; the checks below keep that from happening for a name typed in the forms.

Name rules, each tested once:

- `Class`: one or more segments split on `/` or `\`, each an ASCII letter then ASCII letters, digits or
  `_`. A migration name such as `create_posts_table` passes. The sentence is the spec's: "A model name
  starts with a letter and has no spaces. Write PaymentIntent, not "payment intent"." with the label of the
  generator in place of "model" (only the model sentence is in the spec).
- `Module`, for `module:make`: an ASCII letter, then ASCII letters, digits, `-` or `_`. It is narrower
  than the package (a leading `_` passes there) and never starts with `-`. The sentence is the
  package's, with the name.
- `Free`: no white space and no leading `-`, for an argument whose names are not class names. Task 14.2
  reads the 13.3.0 generators and gives `Free` only where a class name would refuse a name the package
  takes (for example a view path); the design does not guess the list.

The run button is off while a checked field is empty or not valid. The error shows under the field as
soon as a non-empty value fails (S9.2), and is gone as soon as it passes.

### D3. Reading the output (`output.rs`, task 14.3)

```rust
pub enum FileOutcome { Created, Replaced, Left, Failed }
pub struct GeneratedFile { pub path: String, pub outcome: FileOutcome }
/// The files of a generator run, in output order. `force`: the run carried `--force`.
pub fn generated_files(lines: &[String], force: bool) -> Vec<GeneratedFile>;
/// True when `module:make` says that `composer dump-autoload` failed.
pub fn autoload_failed(lines: &[String]) -> bool;
```

Each line goes through `ansi::plain`. A task line starts, after spaces, with `Generating file ` and ends
with `DONE` or `FAIL`; the path is the text between, without the dots and the time. `DONE` is `Created`,
or `Replaced` when `force`. `FAIL` is `Left` when an error line `File : <path> already exists.` names the
same path, else `Failed`. An `already exists` line with no task line before it adds a `Left` file. The
path is kept as printed: relative to the app or absolute. The recordings decide the exact shapes.

Without P5 the reading breaks when the package changes these words; the real-PHP job of milestone 16 is
the guard. The exit code does not decide the result: `module:make-model` exits 0 after a call failed.

### D4. Replace (`catalog/generators.rs`, task 14.2)

```rust
/// The words that write `left` again with `--force`, if Mortise knows the generator that wrote it.
pub fn replace_words(catalog: &Catalog, run: &[String], left: &str) -> Option<Vec<String>>;
```

- A run of one generator that writes one file (not `module:make-model`, `module:make-component`,
  `module:make`): the run's words and `--force`, when the generator has a `force` option.
- `module:make-model`: the file name without `.php` (the part after the last `/` or `\`) is compared with
  the class each call writes: `<Model>Controller`, `<Model>Request`, `<Model>Resource`, `<Model>Seeder`,
  `<Model>Policy` give `module:make-<kind> <name> <module> --force` with the name the model passes.
  The factory gives `module:make-factory <Model> <module> --force` when its file name is the one
  `FactoryMakeCommand` writes for `<Model>` (task 14.2 reads it). The model file (`<Model>`, and not
  the factory) gives the run's words without the seven flags and `--all`, and with `--force`.
- Anything else, and a migration: `None`. The result then offers "Open the existing file" only.

The model makes its name studly (`Str::studly`) before it adds a suffix, so the comparison uses the
studly name, and a call matches only when the run carried its flag or `--all`.
`module:make-migration` gives `None` too: its file name gets a new date on each run.

These are the names the package passes, not paths: Mortise never builds a file path. A wrong match
cannot write elsewhere: the generator writes where the package says, and the dialog shows the line.

### D5. The Generate tab (`generate_view.rs`, task 14.4)

State per open app, in memory: `GenerateState { view: Picker | Form | Result, filter, current row,
generator, filled, last_shown, result }`. The module is the index selection on every frame. Changing
module keeps the form and its values; the module word follows. With "All modules" selected, the picker
lists "New module" only, with the line "Select a module in the index to generate files in it." (not in
the spec); a file generator's form goes back to the picker.

- **S9.1 picker**: the filter field (C3) "Find a generator", the count line "41 generators for Billing,
  and New module. Enter opens the form." from the catalog, then the groups of D1 in columns. "In the app"
  holds New module. Marked "for nwidart/laravel-modules 13.3" when `shipped_for(app.package)` is false.
  Typing filters by label and artisan name; Up, Down, Left, Right move; Enter opens the form; Esc clears
  the filter. 3 opens the tab; G opens it with the filter focused.
- **S9.2 form**: "Generator" with the label, the artisan name and "Change" (back to the picker, "42
  available" from the catalog). Fields from the catalog: each argument but `module` is a text field ("<Label>
  name" for the first, else the argument's description); each flag a checkbox with the package's
  description and the flag in dim mono; each value option a text field labelled the same. `--force` is
  not offered: Replace is the way (S9.6). The model form follows S9.2: "Model name", "Fillable attributes"
  with "--fillable, comma separated", "Also create" with the seven short labels and short flags, "All of
  these -a", "Use Eloquent attributes such as #[Fillable] --attributes".
  Under "Command", the line `line_now(runner, words)` (deviation S37), then the sentence "The files are
  listed after the run." (not in the spec, plan question 2), then the primary "Create files" (not in the
  spec: the count needs P2) with its keycap, and "Copy command" ("Copied the command.", Ctrl Shift C).
- **S9.4 new module form**: "Module names, one or more, separated by spaces"; Kind Web, API, Plain
  (segmented, Web adds no flag); "Inertia controller and pages, no Blade views --inertia"; "Turn the
  module on after creation" (checked; unchecked adds `--disabled`); "Replace the module if it exists
  --force"; Author Name, Email, Vendor. Then the line, the sentence of the P2 fallback, "Then the command
  runs composer dump-autoload in the app.", "Create module" and "Copy command". With Replace checked
  the primary button is `ButtonKind::Danger`.
- Without a runner, or while a command runs, the run button is off with the tooltips of the Actions tab.
  Ctrl Enter (Cmd Enter) runs only the words shown in the last frame, with their whole line inside the
  visible part of the tab, as on the Actions tab.

### D6. Runs and results (`app.rs`, task 14.4)

A run starts through the same path as an action (`start_run`), and the app keeps a `GenerateRun {
generator, module, args, force, modules_before }` beside it. At its end:

- **S9.5**: `generated_files` with no `Left` or `Failed`: the result replaces the sentence of the
  preview: "Created in Billing" and "3 files", one row per file with the path relative to the module
  folder (the app folder and the module's folder from the snapshot taken off the front when they match;
  else the path as printed) and "Open"; "Open PaymentIntent.php" (Enter) and "Generate another" (back to
  the form, the name cleared, the rest kept). Done toast "Created 3 files in Billing.". New rows carry
  the well fill for 600 ms, then fade, unless motion is reduced.
- **S9.6**: with a `Left` file: "Created 2 of 3 files" and "1 failed"; the left row reads "exists, left
  as it is"; the alert "Created 2 of 3 files in Billing. PaymentIntentPolicy.php already exists and was
  left as it is." with "Replace PaymentIntentPolicy.php" (when `replace_words` gives words) and "Open the
  existing file". It names the first left file; more left files get the sentence with the count (not in
  the spec). No toast while the result and drawer are on screen; else the failed toast of milestone 12.
- A failure with no file read: no result box; the drawer says it.
- **Replace**: the dialog "Replace PaymentIntentPolicy.php?", "The file exists. The generator writes a
  new one over it, and Mortise cannot bring the old one back.", confirm "Replace file", `danger`, focus on
  Cancel, and the line in `code` (the exact line on screen before it runs). Confirm runs the words; the
  result of that run lists the file as replaced.
- **module:make**: Ctrl Enter with Replace checked, or with `--force` in the runner's line, opens "Replace
  the module Loyalty?" with "Modules/Loyalty exists with 23 files. module:make --force deletes the folder,
  then generates the module again. Files you wrote in it are lost." when the snapshot has a module of that
  name (ASCII case ignored; its folder relative to the app; its file count), else "If the module exists,
  module:make --force deletes its folder, then generates the module again. Files you wrote in it are lost."
  (not in the spec); confirm "Replace module", the line in `code` (deviation S50: the package deletes the
  folder first). When the run created or replaced files, whatever its exit code, the app reads the app
  again, waits for a snapshot and selects the first module that was not in `modules_before`, else the
  module whose name equals a typed name (ASCII case ignored, `-` and `_` left out), and opens its
  Summary. At code 0 the toast reads "Created the module Loyalty." (not in the spec). When
  `autoload_failed`, nothing is selected: the form keeps the alert "The module was created, but composer
  dump-autoload failed. Fix the module (for example its composer.json), then run composer dump-autoload."
  on screen, under "All modules".
- Run again of a generator line from the drawer repeats the words; a line with `--force` asks again
  (`asks_again`, milestone 12).

### D7. Ways in (task 14.4)

- "New module" in the index footer (S3.1), a quiet button with an accessible name; Ctrl N (Cmd N)
  anywhere an app is open; the picker's row; the palette. Each selects "All modules", opens Generate on
  the new module form and focuses the name field. A menu item with the same key when `menu.rs` lists
  the other shortcuts.
- The palette gets the group "Generators" after "Actions for <module>", before findings (S12.1). The
  module is chosen as for the actions group. A row reads "Generate a model in **Checkout**" with
  `module:make-model` in dim mono and the hint "Form"; "New module" with `module:make`. At most four rows,
  matched against the label and the artisan name. A row opens the form; nothing runs. New
  `Command::Generate { module: Option<String>, generator: String }`.

### D8. Recordings and tests

New recordings (task 14.3), from a scratch app with 13.3.0, a module Billing, no agent variables, `args`
unique across the folder: `module:make-model PaymentIntent Billing --migration --policy
--fillable=order_id,amount,currency,status` (all new); `module:make-model Invoice Billing --migration
--policy` with `InvoicePolicy.php` there before; `module:make-policy InvoicePolicy Billing --force`;
`module:make Loyalty --api`. More only when a rule of D3 needs one.

Core: the catalog parses, has 42 generators, and every name of the order table but the health indicator
is in it; `grouped` puts an unknown generator in Other; `words` (order, array split, trimmed and empty
values, `--all`, a leading `-`); each name rule once; `replace_words` (one generator, a model's policy,
a migration); `generated_files` and `autoload_failed` on the recordings.

Interface, `tests/generate.rs` against the fake artisan: the picker's count line and filter; the model
form's line changes with a checkbox and the name error turns the button off; Create files runs and shows
the result and toast; the existing policy gives the left row, Replace opens its dialog with the line and
runs `--force`; Ctrl N opens the new module form on "All modules"; Replace module asks. The palette row
once in `tests/palette.rs`. Images S9.1, S9.2, S9.4, S9.5, S9.6, light and dark.

### Changed while building 14.4

- D5: the picker's filter shows G as its key hint, not "/": "/" stays the filter of the index. The
  filter matches a part of the label or of the artisan name, not letters in order, so that "polic" lists
  Policy only. The new module form also needs its scope: selecting a module goes back to the picker, as a
  file generator's form does under "All modules". A click on a row of the picker opens its form. The
  title of the picker under "All modules" is "Generate in shop-api" (deviation S45).
- D6: a run that left files, out of view, gets a failed toast with the sentence of the alert, not
  "stopped with exit code": the model exits 0. A run that wrote files again reads "Replaced in Billing"
  and the toast "Replaced 1 file in Billing." (deviation S47). Any change of the form drops the result
  and brings the line back. Run again from the drawer runs the words, with its own question for
  `--force`, and does not change the result on the tab. Mortise reads the app again after `module:make`.
  A result shows only while the form the run started from (its generator and module) is still on
  screen; otherwise the toasts of a run out of view apply. Whether a run carries `--force` is read from
  the whole line, the runner's words too. The run button, like Ctrl Enter, runs only a line that was
  whole on screen in the last frame; otherwise it is off with "Scroll up to see the whole command
  first." (not in the spec). The Actions tab follows the same rule.
- D7: `menu.rs` lists no shortcut, so no menu item was added. In the palette, a word of the query that
  matches a module names the module; the other words match the generators. A query that only names a
  module lists no generator, so "check" keeps the rows of S12.1 it had.
- D8: one more image, the Replace dialog of S9.6. The image of S7.4 shows Metrics now, as Generate is
  built; every image with the index changes for "New module" in its footer.

## Risks / Trade-offs

- [The package changes the words of its task lines] → `output.rs` reads them without P5; the real-PHP job
  of milestone 16 catches it.
- [The forms use the package's descriptions as labels] → They are English and sometimes terse ("Flag to
  create associated controllers"). The spec says the forms come from the package; the two drawn forms
  get the spec's labels.
- [`module:make-model` exits 0 with a file left] → The drawer reads "Done" while the result says a file
  was left. The result reads the lines, not the code. Recorded as a deviation if the recording shows it.
- [Replace guesses the generator of a model's file by name] → Only exact names the package passes count;
  otherwise there is no Replace button. The dialog shows the line before it runs.
- [Selecting the new module waits for the watcher] → Until the snapshot has it, "All modules" stays
  selected; nothing breaks.

## Migration Plan

No change to `settings.json` or to the history. Rollback is a revert.

Order: 14.1 (catalog JSON), 14.2 (types, words, rules, Replace) and 14.3 (recordings, output reading)
in parallel; 14.2 reads 14.1's file in its tests once both are on the branch. Then 14.4.
