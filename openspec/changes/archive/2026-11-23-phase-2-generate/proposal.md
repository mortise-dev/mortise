# Proposal

## Why

The Generate tab still shows "Generate is not in this version". Milestone 14 of `docs/plan/phase-2.md`
adds it: Mortise creates modules and files through the package's own generators, with the exact
command on screen first. The package does not describe its generators yet (P3 is open, not
released), so Mortise ships a description of the 42 generators of 13.3.0 in the shape P3 gives.

## What Changes

- A catalog of the 42 generators of `nwidart/laravel-modules` 13.3.0 (`module:make` and 41 that write
  a file in a module), in `catalog/v13_3.json`, read into types that match the JSON of
  `module:generators --json` (P3, PR nWidart/laravel-modules#2295). Milestone 15 fills the same
  types from P3.
- Name checks before a run: a class name starts with a letter and has no spaces; a module name follows
  the package's rule and its sentence. The run button stays off while a name is not valid.
- Reading a generator's output: the files it created, replaced, or left because they exist, from the
  lines "Generating file ... DONE" and "File : ... already exists." (without P5).
- S9.1, the generator picker: "41 generators for Billing, and New module", seven groups of Mortise's own
  and "Other", a filter, the keys of S9.1. Marked "for nwidart/laravel-modules 13.3" when the app has
  another version.
- S9.2, the form of a generator, built from the catalog, with the command line on every keystroke and
  "Copy command". Without P2 there is no file list: the form says "The files are listed after the run"
  (not in the spec, plan question 2).
- S9.4, the new module form for `module:make`, from "New module" in the index footer, the picker,
  Ctrl N (Cmd N) and the palette. "Replace the module if it exists" asks first.
- S9.5 and S9.6, the result after a run: the files created, with "Open"; the file left because it
  exists, with "Replace <file>", which runs the generator with `--force` after its confirmation, and
  "Open the existing file". A done toast; no toast for a failure on screen.
- After `module:make`, the new module is selected and its Summary opens.
- The palette gets a "Generators" group after the actions. A row opens the form; it does not run.
- S9.3 (the preview when a file exists) waits for P2 in milestone 15.

## Capabilities

### New Capabilities

- `module-generators`: the generator catalog and its versions, the name checks, the picker, the forms,
  the new module form, the result of a run read from its output, Replace and its confirmation, and the
  ways in (index footer, Ctrl N, palette, keys).

### Modified Capabilities

None. The runner, the drawer and Run again already cover a generator's run: a line with `--force`
asks again (`runner-drawer`).

## Impact

- `mortise-core`: `catalog/generators.rs` and `catalog/v13_3.json` (new), `catalog/mod.rs`,
  `output.rs` (generator lines). No new dependency: `serde` and `serde_json` are already used.
- `mortise-app`: `generate_view.rs` (new), `workspace.rs`, `index.rs` (footer), `palette.rs`,
  `app.rs` (keys, runs, toasts, selection after `module:make`), `dialog.rs` if needed, `strings.rs`,
  tests and images.
- `fixtures/output/`: more recordings of 13.3.0 generators.
- Docs: `docs/deviations.md`.
- Not run here: Windows and Linux. CI runs the build and tests there.
