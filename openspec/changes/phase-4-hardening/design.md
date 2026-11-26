# Design

## Context

- The core writes every path inside the app relative to the app folder, with its parts joined by `/`:
  `AppSnapshot::path_shown` and a copy of the same function in `findings.rs`. The references of a link
  are joined the same way from the folder of the module and the relative path of the file.
- When `strip_prefix` fails, the function joined every component of the whole path with `/`. The root
  component is itself `/` on Unix and `\` on Windows, after a prefix such as `C:`. So a modules folder
  at `/shared/Modules` came out as `//shared/Modules`, and `C:\shared\Modules` as `C:/\/shared/Modules`.
  A modules folder outside the app is possible: `config/modules.php` can set an absolute path.
- The push document has its own `relative` in `push/mod.rs`. It climbs out of the app with `..` and keeps
  only the name on another drive, so no absolute path leaves CI. It is right and stays.
- `crates/mortise-cli/tests/check.rs` compares the exact text and github output of shop-api, and
  `cargo test --workspace` runs it on Windows in CI. So the paths inside the app were proven on Windows
  already; the path outside the app was not.
- The job "Command line tool" built the tool on the three platforms but checked only the exit code of
  one text run and that a dry run exits 0.

## Goals / Non-Goals

**Goals:**

- One function that shows a path, right on every platform, tested once in the core.
- The built tool, not only its tests, run on the three platforms with its key output checked.
- The controls of milestones 24 to 27 in the accessibility audit.

**Non-Goals:**

- Paths relative to the root of the repository in the `github` format. GitHub marks a line only when
  the app folder is the root of the repository; the how-to says so. A prefix would need the top of the
  git checkout, which `check` does not read. Left for a request.
- A change to the push document or its schema.

## Decisions

### D1. A path outside the app is printed whole, with `/`

`shown(root, path)` strips the app folder when it can. It then walks the components: a Windows prefix
is kept with `\` turned to `/`, the root becomes one `/`, and every other part is joined with `/`.
`findings.rs` calls the same function. The check prints the whole path because the user reads it on
their own machine; the push document keeps its own rule (Context).

Alternative: print `..` paths in `check` too. Rejected: a path such as `../../../shared/Modules` is
harder to read in a terminal than the folder itself, and a path on another drive has no `..` form.

### D2. The CI job checks output with grep and jq

`grep` and `jq` are on the Linux, macOS and Windows runners, and the steps run in bash on all three.
The github format goes to a file, so the fixture's problems do not mark the pull request. The job
checks: exit 1 in each format, the circle line, three reads of `Invoice`, the summary, one `::error
file=` line, the order of the checks in json, exit 3 for a folder that is not an app, and in the dry
run `app_path`, the surface of Users and that no path names a drive, a root or a backslash.

### D3. The Accept dialog is audited in its own test

The workbench test reads the app with no way to run PHP, so Accept is off there. The new test sets
Herd, so that nothing runs, and a lock with laravel-modules 13.5.0, the first that stores a surface.

## Risks / Trade-offs

- The `jq` of the Windows runner could print `\r\n`. The job uses `jq -e` for every check, so it
  compares no printed text.
- Windows and Linux were not run on the Mac. CI runs them.
