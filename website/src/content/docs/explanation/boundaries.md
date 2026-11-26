---
title: Module boundaries
description: What a boundary between modules is, how Mortise finds the links that cross it, and why the rules live in mortise.json.
sidebar:
  order: 3
---

`nwidart/laravel-modules` splits a Laravel app into modules, each in its own folder with its own
namespace. The split is only as good as the lines between the modules. This page explains how Mortise
sees those lines, what its three rules check, and why they are kept where they are.

## What a boundary is

A module is meant to be a unit you can reason about on its own. Other modules depend on it, but they
should depend on a small, stable part of it: an interface, an event, a command. That part is its
public surface. The rest are its internals: models, services, controllers, the code that can change
without notice.

The boundary is the line between the two. When another module reaches past it and uses an internal
class, the two modules are tied together. A change inside one can break the other. Over time the
modules stop being separate at all.

PHP does not enforce any of this. Any class can name any other class. So the boundary exists only as
an agreement, and Mortise makes that agreement visible.

## Links between modules

Mortise finds links in the source, without running PHP. For each PHP file of a module, it lists every
class, interface, trait and enum the file names, resolved the way PHP resolves them. When a name
belongs to another module, that file makes a link from its module to the other one.

A link therefore has a direction: Checkout uses Billing. It is made of references: each file, the
first line that names the class, and the class itself. The Boundary tab draws the links as a graph,
with the arrow at the module that is used, and lists the references of a link in a table.

Test files make no links. A test of one module may well set up another, and that is not a dependency
of the code.

Because the links come from names in the source, a class that is built from a string or resolved from
the container at run time makes no link. [Reading an app without PHP](/mortise/explanation/reading-without-php/)
explains that limit.

## Rule 1: no circular use

Two modules that use each other, directly or through others, form a circle. Neither can be understood,
tested or removed without the other. A circle is the strongest tie two modules can have.

Mortise finds every circle among the modules that are on. It reports each circle once, as one finding,
and counts it on the module of the circle with the fewest files to change. That is the cheaper side to
cut. Every file in the circle is marked, because removing any one direction ends it.

A module that is off is not loaded by the app, so a circle through it breaks no rule. The graph still
draws it, so that you see it before you turn the module back on.

## Rule 2: other modules use only a module's public surface

This rule checks that a module reads only the public part of another module. A reference to any other
class is a reference to internals.

What counts as public? The full answer is a list of the classes, routes and commands that a module
offers to others: its accepted public surface. The Boundary tab can scan a module and propose such a list.
When you accept it, the package command `module:surface` stores it under the key `surface` of the
module's `module.json`. The list then travels with the module, and every copy of the app reads it, with no
PHP. For a module with a surface, only the classes it lists are public.

A module without a surface gets a simpler rule: a class in one of a few folders counts as public.
By default those folders are `Contracts` and `Events`, because interfaces and events are the usual way
for modules to talk without knowing each other's insides. A class directly in the namespace of the
module, in no folder, is not public. A `surface` key that the package would refuse, such as one whose
lists are not lists of strings, counts as no surface.

You can change the list of folders. A shorter list is stricter. The rule applies only when both modules
are on.

## Rule 3: allowed use

The first two rules hold for every module. Some apps also have layers: a module low in the app, such as
`Audit`, must use no other module, and a module above it may use only the ones below. The rule
**Allowed use** writes those layers down as a list for each module, in the key `rules.allowed-use` of
`mortise.json`.

A module with a list may use the modules of its list only. An empty list means it may use no module.
A module without a list is not checked. So an app that already has links across its layers can adopt
the rule one module at a time, with no baseline. A module that is off breaks no rule, and no link to it
does. Without the key, the rule does nothing.

The rule says which module. The second rule still says which classes of it. Mortise checks the imports
it reads without PHP. [nwidart/larch](https://github.com/nWidart/larch) reads the same list in Pest, and
can also check the links that only the booted app shows.

## Why the rules live in mortise.json

The rules are a decision about the app, not about one person's copy of Mortise. So they are kept in
`mortise.json` in the app folder, next to `composer.json`. A team commits the file and shares one set of
rules, and a change to the rules shows up in a pull request like any other.

That is also why `mortise.json` is the only file Mortise writes into an app, and why it asks before it
creates it. An app without the file follows the default rules: the first two rules on, with `Contracts` and
`Events` as public folders. If the file cannot be read as rules, Mortise says so and uses the defaults
until it is corrected. A save keeps the other keys of the file, and Mortise never replaces a
file that is not a JSON object.

The file holds only what the team shares: the rules, whether a broken rule fails CI, and the
coverage limit and report when the team sets them. Settings that belong to you, such as how PHP runs or
the URL of an environment, live in Mortise's own settings outside the app. [mortise.json](/mortise/reference/mortise-json/)
describes its format.

The rules box also has **Fail CI when a rule breaks**. The command line tool `mortise check` reads the
same `mortise.json` through the same code as the window, so CI and the window cannot disagree on a rule.
It judges one commit: it fails on every problem in it, not only on new ones.

## Findings, not errors

A broken rule is a finding. Mortise does not stop you, block a command or change your code. It shows
where the line is crossed, which file does it and on which line, and leaves the decision to you. Some
links are deliberate. The point is to see them.
