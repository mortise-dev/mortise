# Deviations from the specification

The specification in `docs/spec/` was drawn in HTML and CSS. Its notes on egui were written from memory.
This file lists where the current egui and eframe (0.37.0, winit 0.30.13) force a difference, with the reason.

- **Raised** means the difference changes how a screen looks or works, and the owner was told.
- **Checked** says how the fact was established.

Checked on 22 November 2026 by reading the 0.37.0 source and compiling a probe crate on macOS.
Nothing here was run on Windows or Linux.

## Open

| # | Spec | What egui allows | Effect | Raised | Checked |
|---|---|---|---|---|---|
| D1 | Section 8, T2: Windows 11 snapping from the maximise button with a custom title bar | Not possible. winit never handles `WM_NCHITTEST`, so a drawn maximise button cannot report itself to Windows. Dragging to an edge and Win with the arrow keys still snap | The flyout on the maximise button is absent with the custom bar. The spec leaves the choice to the owner: custom bar without it, or the platform's bar (T4) | Yes | Source of winit, not run |
| D2 | Section 8: a frameless window on Windows | winit keeps a 1 px line at the top of an undecorated window for the system shadow | One extra pixel line above the title bar on Windows | Yes, to be looked at on Windows | Source, not run |
| D3 | Section 9: reduced motion follows the system setting | Neither egui, eframe nor winit reads that setting | Mortise reads it itself per platform. Until that is built, "System" behaves as "Full" | No, same result once built | Source |
| D4 | Section 8: a system menu bar on macOS | eframe offers only the default app menu | The menu is built with `muda`, in milestone 6 | No, same result | Source |
| D5 | Tokens 3.3: text size scales type and sizes, strokes stay 1 px | Text size is egui's zoom factor, which scales strokes too. Hairlines are drawn at 1 divided by the zoom, so they stay 1 px at 100% display scale. At 110% and 125% a line can fall between pixels | Hairlines may look slightly soft at 110% and 125% | To be looked at in milestone 6 | Source |
| D7 | Section 11: static font files, because variable fonts were not checked | Variable fonts work in 0.37 (`FontTweak::coords`, `RichText::variation`). Font weight is still not part of `FontId` | Static files are kept as the spec says; each weight is its own named family | No | Source, compiled |

## Notes that are not deviations

Facts the spec was unsure of, which turned out to hold.

- The macOS window buttons stay over custom content: `with_fullsize_content_view`, `with_titlebar_shown(false)`, `with_title_shown(false)`, and `Frame::set_traffic_lights_position` to centre them in a 34 px bar. Double click to zoom is done by the app.
- Screen readers: AccessKit is always part of egui 0.37 and is on by default in eframe for macOS, Windows and Linux. A painted control gives its role, name and state through `Response::widget_info`. A live region is set through `Context::accesskit_node_builder`.
- The floating scrollbar is egui's default style.
- The two shadows fit egui's shadow type (whole pixels, blur up to 255).
- `/` and the backtick are matched as logical keys, so on some layouts they need Shift or a dead key. The spec already gives both a second binding (Ctrl F, Ctrl J).
- egui's own Cmd or Ctrl with plus, minus and 0 zoom is turned off, because text size is a setting.
- Numbers are tabular without a font feature: in all four Asap files and all three Inconsolata files
  the ten digits have one width (535 and 500 units of 1000). Checked with fontTools on 22 November 2026.
  This was D6; it is closed.
- On macOS the window runs with the system's three buttons over Mortise's bar, centred in its 34 px.
  Looked at in the running app on 22 November 2026 (milestone 1).

## Where the specification leaves a choice, or its drawings and its text differ

Not forced by egui. Each is small; each is what milestone 1 built, so the owner can change it.

| # | Subject | Specification | Built | Raised |
|---|---|---|---|---|
| S1 | Module codes | C10 gives a rule with three examples (CHK, NTF, SHP) and leaves the exact rule to the builders. The drawings also show BIL and CAT, which that rule does not give | The first letter, then the consonants that follow, each letter once: Billing is BLN, Catalog is CTL. Unique, never ALL. Tests in `mortise-core/src/code.rs` | Yes, milestone 1 |
| S2 | App monogram | S2.1: the initials of the first two words, or the first two letters. The drawings show SH for shop-api | The written rule: shop-api is SA | Yes, milestone 1 |
| S3 | Brand in the title bar | Tokens 3.3: `row-name`, weight 600. The drawings use 700 | 600, the token | No |
| S4 | Workspace header height | Section 5.14: 76 px. The drawings measure 74 px with buttons in the header and 73 px without | 76 px | No |
| S5 | Sizes without a token | The app name in the index (14 px, 700), the monogram on a rail tile (11 px, 600) and a path beside a title (Inconsolata at 0.95 of the text size) are in the drawings but not in section 3.3 | Kept as named constants beside the tokens in `mortise-app/src/theme.rs` | No |
| S6 | Text of the Actions and Metrics tabs in a phase 1 build | S7.4 gives the text for Generate and says the other two use the same block | Written to match: "Actions are not in this version", "Metrics are not in this version", each with one paragraph. In `mortise-app/src/workspace.rs` | Yes, milestone 1 |
| S7 | Restore | Section 8 names "maximise or restore"; the drawings have no restore icon | Two offset squares | No |

## Moved between milestones

What the plan placed in one milestone and was built, or will be built, in another. Nothing is dropped.

| What | Planned | Now | Why |
|---|---|---|---|
| Progress bar (C17), "Reading 312 of 551 files" | 2 | 9 | Reading an app takes milliseconds until PHP files are parsed. The bar is built with the work that can take long |
| Reordering rail tiles by dragging | 2 | 6 | Settings also changes the order; both are built together |
| "Open the command palette" on the first run screen (S1.1) | 2 | 6 | The palette is built in milestone 6. A button that opens nothing is not drawn |
| Removing an app | 2 | 6 | The only place the spec draws it is Settings (S8.1). The core can already remove an app |
| Settings tile on the rail | 1 | 6 | Drawn when Settings exist |
| F6 between regions, arrow keys in the graph, J and K inside tables, Enter or Right from the index into the workspace | 6 | 9 | Tab and Shift Tab reach every control and the focus ring shows on all of them. The shorter paths of section 7 are built on top of that |
| macOS menu bar | 6 | 9 | Needs `muda` and a check on a real bundle |
| Dragging the index edge between 220 and 320 px | 6 | 9 | The widths of section 5.14 that depend on the window are built |
| Motion "System" | 6 | 9 | egui does not expose the setting of the operating system (D3). "Reduced" and "Full" work |
| A shortcut keycap inside a tooltip | 6 | 9 | Tooltips name the control |
