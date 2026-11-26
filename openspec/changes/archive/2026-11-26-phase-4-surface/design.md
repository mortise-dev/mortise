# Design

## Context

The scan of milestone 8 proposes a surface (`mortise-core/src/surface.rs`). The rule of the links
(`boundary/links.rs`) marked a reference as internal when the class was outside the folder list. The
runner of phase 2 starts artisan with Mortise's standard input closed. laravel-modules 13.5.0 stores the
surface as plain, sorted JSON, so a tool can read it without PHP.

## Goals / Non-Goals

**Goals:**

- One rule in the core for the window and `mortise check`.
- Mortise writes no `module.json`. The package command writes it, through the PHP the user chose, with
  the exact line on screen first.
- The surface that is stored is the one on screen: no second form of it on the command line.

**Non-Goals:**

- A date of acceptance or a history of surfaces (plan section 1).
- Checking routes or commands in a rule. The second rule is about classes.

## Decisions

### The surface lives beside the scan

`AcceptedSurface` is in `surface.rs`, beside the scan that proposes one. `modules/module.rs` reads it into
`Module::surface` with the manifest. `serde` reads the key with unknown fields denied, as the package
does. A key it cannot read counts as no surface, so the folder list applies: an invalid file never hides
a finding.

### The rule asks the surface first

A reference is internal when the rule is on, both modules are on, and the class is not public. Public is
`surface.has_class(class)` when the owner has a surface, else `rules.is_public(inside)`. Class names
compare without case, as PHP names do; a leading backslash is ignored.

### Standard input, not repeated options

`module:surface` takes `--set` with repeated `--class`, `--route` and `--command`, or `--set --stdin`
with one JSON object. Mortise uses standard input: a long surface does not make a long command line, a
name never needs quoting for a shell, and the line on screen stays short and exact.
`Runs::start_with_input` pipes standard input only when input is given, writes it from a thread of its
own and closes it. Every other run keeps a closed standard input.

### Accept is a read, not a run of the drawer

The command runs as a read of package data (`ReadFor::Surface`): its JSON answer decides the toast, it
shares the lock of one command per app, and it is not kept in the history. A run again from the history
would have no standard input.

### Which package can store a surface

`stores_surface` is true for a released 13.5.0 or later, read from `composer.lock` as `gives_data` reads
13.4. Without it, Accept is off with the reason. A dev build gets false. When artisan still answers that
it has no `module:surface`, the toast says to update the package.

### Change and compare start a scan

The entries of S5.7 come from the files of the module, so Change and Scan again scan first. A change
checks what the surface lists. A comparison keeps only the differences: what the scan proposes that the
surface lacks, and what the surface lists that the module no longer has. A route is gone only when the
routes were read. Accepting the differences stores the surface with the checked ones applied.

### The push document

`Module` gains `surface`, so the push document gains a member. The schema of the cloud allows members it
does not name, and the contract raises the version only for a member removed, renamed or retyped. The
committed document of shop-api is written again; `SCHEMA_VERSION` stays 1.

## Risks / Trade-offs

- A team edits `surface` by hand into a form the package refuses: Mortise uses the folder list for that
  module, and `module:surface` shows the error. The deviation S71 says so.
- The fixture app pins 13.3.0, so the window tests of Accept change the lock to 13.5.0 and run the
  recorded answer through the fake artisan. A real run of Mortise against 13.5.0 is not automated.
