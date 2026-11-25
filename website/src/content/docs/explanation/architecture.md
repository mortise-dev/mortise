---
title: How Mortise is built
description: The two crates of Mortise, the choice of egui and eframe, the ban on unsafe code, the optional features and what differs per platform.
sidebar:
  order: 5
---

Mortise is a native desktop app written in Rust. This page explains how its code is split, why it
draws its own interface, and what those choices cost on each platform. It is for readers who want to
know why Mortise behaves as it does, or who want to work on it.

## Two crates: the core and the app

The workspace has two crates.

- **`mortise-core`** does everything that is not drawing. It reads an app folder into a snapshot,
  finds the links between modules, checks the rules, runs commands, talks to the actuator and keeps
  the settings. It has no interface dependency at all.
- **`mortise-app`** is the window. It draws the screens and wires each control to the core. Its binary
  is `mortise`.

The line between them is strict, and it pays in three ways. The core can be tested without a screen:
reading an app, the file watcher, the actuator client and the scan all have tests that run in plain
Rust. The core could serve another front end, such as the command line tool that **Fail CI when a rule
breaks** waits for. And the interface stays thin: it asks for the current snapshot and draws it.

A snapshot is never changed. A read on a worker thread produces a new one and tells the window that it
is there. So the window never shows an app that is half read, and a slow read never freezes it.

All use of the PHP parser stays in one file of the core. The parser's interface changes between minor
versions, so nothing else names one of its types, and its version is pinned. Moving past that version
needs a newer Rust, and the two move together.

## egui and eframe

The interface uses egui, drawn by eframe. egui is an immediate mode library: each frame, the code
describes the whole screen from the current state. There is no tree of widgets to keep in sync with the
data, which suits an app whose data is a snapshot that is replaced as a whole.

eframe runs the same code on macOS, Windows and Linux, and draws with the GPU through wgpu. The app is
one binary with its fonts inside. Asap and Inconsolata are bundled, so text looks the same everywhere.

The cost is that egui is not the platform's own toolkit. Mortise draws its own title bar, buttons and
lists, and some platform features need work or are out of reach:

- **Accessibility.** egui reports each control to the platform's screen reader through AccessKit.
  Mortise gives every painted control a role and a name, and a test checks that every control has a
  name and a target of at least 24 by 24 pixels.
- **The macOS menu bar.** eframe offers only a default menu. Mortise builds the menus Mortise, File,
  View, Window and Help with a separate crate. The window handles the keys, not the menu.
- **Snapping on Windows 11.** With a drawn title bar, the snap layouts that appear over the maximise
  button are absent. Dragging to an edge and the Windows key with the arrows still snap.
- **Title bars on Linux.** Mortise asks for a window without decorations. Some compositors add their own
  bar anyway, such as Hyprland with the hyprbars plugin.

## No unsafe code

The workspace forbids `unsafe` code. Rust's guarantees then hold for every line Mortise owns. That
matters for a tool that runs commands, holds credentials and parses files it did not write.

The rule has a price. When the only route to a platform feature is an `unsafe` call, Mortise goes
without it and records the difference:

- **Reduced motion.** On macOS Mortise asks the system through a safe binding. On Windows and Linux,
  reading the setting needs `unsafe` code, so there the choice "System" moves as "Full". "Reduced" and
  "Full" work everywhere.
- **Stopping a process tree on Windows.** A job object would hold every process a run starts, but each
  call to it is `unsafe`. Without it, a PHP process started by a launcher that already exited can go on
  after Stop. [What Mortise changes, and what it never does](/mortise/explanation/safety/) covers this.

Dependencies may use `unsafe` inside them. The ban is on Mortise's own code.

## Features of the core

Parts of the core sit behind Cargo features, so a tool that needs less can build less:

| Feature | Gives | On by default |
|---|---|---|
| `watch` | Watching the app folder and reading it again on change | Yes |
| `actuator` | The HTTP client for a running app | Yes |
| `os-secrets` | Keeping secrets in the credential store of the operating system | Yes |
| `run` | Running artisan through the PHP chosen for an app | Yes |
| `testing` | A small stub actuator for tests | No |

The app uses the defaults. A tool that only reads an app once, such as a check in CI, could leave out
`watch`, `actuator` and `run`.

## Platforms

Mortise runs on macOS, Windows and Linux from one code base. Continuous integration builds and tests
every change on all three.

| Platform | Release file | Signed |
|---|---|---|
| macOS, Apple silicon | `.dmg` | Yes, with a Developer ID, and notarized by Apple |
| Windows, x64 | Setup `.exe` | No: SmartScreen warns once |
| Linux, x86_64 | `.AppImage`, or `.deb` for Debian and Ubuntu | No |

Some behaviour differs by platform on purpose:

- The command key is <kbd>Cmd</kbd> on macOS and <kbd>Ctrl</kbd> on Windows and Linux.
- Herd runs as `herd.bat` on Windows. Sail is off on Windows, because it runs in WSL only.
- The credential store is Keychain on macOS, Credential Manager on Windows and the Secret Service on
  Linux.
- HTTPS trusts the certificates of the computer, so a local certificate authority works as it does in
  the browser.

The screenshots on this site come from the image tests, which run on macOS only. Some screens were also
looked at on Linux. Windows is checked by CI, not by eye.

[How to install Mortise](/mortise/start-here/installation/) and [How to build Mortise from source](/mortise/how-to/build-from-source/)
have the steps.
