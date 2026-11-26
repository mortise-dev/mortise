# Design

## Context

`Rules` holds two rules and `fail_ci`. `boundary/rules.rs` reads `mortise.json` once, gives the defaults
for a file whose `rules` or `ci` cannot be read, and saves through a temporary file beside it, keeping
other keys. `findings::find` runs each rule on the links that `boundary/links.rs` found. The push
document serializes `Rules` and `Findings` as they are, and its committed copy for `shop-api` must not
change.

## Goals / Non-Goals

**Goals:** one key that Mortise and Larch read the same way; the check on the links Mortise already
reads; every format of `mortise check`; the cloud gets the findings with no change on its side.

**Non-Goals:** the window (milestone 30); runtime links (Larch does that in Pest); a baseline.

## Decisions

- **`Rules::allowed_use: Option<AllowedUse>`.** `None` means the file has no key. A save writes the
  key only when it is `Some`, so a save in an app without the key adds none. The field is left out of
  the push document when it is `None`, so the document of `shop-api` does not change. `AllowedUse` has
  `enabled` (`true` when absent inside the key) and `modules`, a map from a module name to its list.
  The map is a `BTreeMap`: a save writes the modules by name. Order has no meaning for the rule.
- **The check lives in `findings::find`, not in the links.** A `Link` gains no member. The check is a
  lookup of `from` and `to` in the lists. `find` already knows which modules are off. A new member of
  `Link` would change every link of the push document. Milestone 30 asks the same lookup when it marks
  a link on the graph. `Link::is_problem` stays as it is until then, so this change draws nothing new.
- **One finding per link, counted on the module that uses.** The detail is `OutsideAllowedUse { of,
  files }`, as `ReadsInternals` has `of` and `files`. The files and lines are those of the link, which
  `mortise check` and the cloud already have.
- **A name that is no module is a warning of the same check.** It is a `Finding` with no module and the
  detail `UnknownAllowedModule { entry }`, once per name, for a module with a list and for a name in a
  list. A finding reaches the window, every format and the cloud with no new path. It does not fail CI
  unless `--fail-on warnings` is given. A module that is off is still a module: its name is no warning.
- **The rule runs only when `enabled` is true.** Off, it gives no result and no finding, as the other
  rules. `checked` counts the modules of the app that have a list.
- **A wrong value gives the defaults for the whole file**, with the parser's message, as for the other
  keys: `allowed-use` and `modules` must be objects, each list a list of strings. Larch stops on such a
  file instead. The shared fixtures hold valid files only.

## Risks / Trade-offs

- [A link that breaks the rule is not marked on the graph until milestone 30] The findings list,
  the summary and `mortise check` show it now.
- [Mortise sees imports only] A use through the container or a facade is not checked. Larch with
  `via(Edge::all())` checks it in Pest. The docs say so.

## Platforms

The core and the tool are plain Rust with no platform code. CI builds and tests them on Linux, macOS and
Windows. Only macOS was run by hand for this change.
