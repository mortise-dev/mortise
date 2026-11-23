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
| D3 | Section 9: reduced motion follows the system setting | Neither egui, eframe nor winit reads that setting | macOS is asked through AppKit, in safe Rust. On Windows and Linux reading it needs unsafe code, which this workspace forbids: there "System" moves as "Full". "Reduced" and "Full" work everywhere | Yes, milestone 9 | macOS: built, the call returns. Windows, Linux: not built |
| D4 | Section 8: a system menu bar on macOS | eframe offers only the default app menu | Built with `muda`: Mortise, File, View, Window, Help. The items of Mortise carry no key in the menu: the window handles the keys, and a key the menu handled too would act twice. The first menu reads "mortise" until the app is a bundle with a name | Yes, milestone 9 | Run on macOS: the menus are there, and choosing an item acts |
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
- "health every 15 s while the window is visible": egui 0.37 does not run `App::ui` while a window is
  minimized or occluded. The worker that asks the app stops when no frame was drawn for 3 s and
  starts again with the next frame. No platform code is needed. Tested with a stub, not in a real
  hidden window.
- The public surface scan reads 8 files in each frame, in the interface's own thread, so that it
  can show how far it is and be stopped without a second thread. A module of 15 files is read in two
  frames; the scanning view is then on screen for a moment only.
- HTTPS uses the certificates the computer trusts (`rustls-platform-verifier`), so a local site with
  its own certificate authority, such as a Herd `.test` site, works as it does in the browser.

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
| S8 | Routes of a module | S4.3: "18 routes, 8 shown". No control shows the other ten | Every route of the module is listed; the Summary scrolls | Yes, milestone 7 |
| S9 | A way to remove one environment | S8.1 draws "Add environment" and no way to remove one | A small "Remove environment" button under the form of the selected environment | Yes, milestone 7 |
| S10 | "Last answer" in the table of environments | S8.1 draws a status for each environment. Only the chosen environment is asked regularly | The last answer of this run: from the regular check, or from "Test". An environment that was not asked reads "not asked yet" | Yes, milestone 7 |
| S11 | The index header where the app refuses the credentials | S6.5 text: the header reads "No answer on local" in c and e. The runtime box and the Runtime stat read "Not authorized" in e | As written: the header reads "No answer", the box and the stat read "Not authorized" | No |
| S12 | The runtime box before the first answer | No drawing | The status line reads "asking" until the first answer is there | No |
| S13 | An address with no actuator (404) | S6.5 i gives the sentence for 500 only | "local answered 404 for /actuator. Correct the base URL or the actuator path in Settings.", with "Open settings" | No |
| S14 | Layout of the app form in Settings | S8.1 draws name, monogram, folder, coverage report and limit in one row, and the credentials beside the secret | Stacked, at most 560 px wide; the environment form at most 720 px. The same form is in the add panel, as S1.3 draws it | No |
| S15 | `APP_ENV` | S1.3: Mortise reads `APP_URL` and `APP_ENV` from `.env` | Only `APP_URL` is read. `APP_ENV` is for asking before a command in production, which is phase 2 | No |
| S16 | Environment names | No rule | Unique in an app and not empty; a rename that breaks this is not taken. The first three are proposed as local, staging, production | No |
| S17 | The palette and the scan | S5.5: the palette starts the scan with "Scan the public surface of Billing". F6: Enter on that row opens the view and does not start the scan | The row has the words of S5.5 and does what F6 says: it opens the view. A scan never starts by itself | Yes, milestone 8 |
| S18 | Copying the proposal | S5.7: until the package can store a surface "the proposal can be copied as text". No control is drawn | A "Copy as text" button left of "Scan again". It copies the entries that are checked, by kind | Yes, milestone 8 |
| S19 | Who runs a command | S5.7 draws "the scheduler, daily at 06:00" | "the scheduler, 0 6 * * *": the schedule as the app wrote it. Mortise does not put a cron expression into words | Yes, milestone 8 |
| S20 | The lines of the scanning view | S5.6 draws "reading from local, 1.2 s" for routes and "waiting" for commands | Routes are those the chosen environment already gave, so their line is complete at once. Interfaces, events and commands are counted as the files are read, in one pass | No |
| S21 | An entry added by hand | S5.7: "checked against the module's files as it is typed" | A line under the field says what the name is in the module. Enter adds it only when the module has it: a class it declares, a command, or a pattern that some of its routes are under | No |
| S22 | Route patterns | S5.7 draws `api/billing/*` and `billing/*` and gives no rule | A route is proposed under its first part, or its first two parts when the first is `api`. A route whose first part is a parameter stands for itself | No |
| S23 | A scan that finds nothing | No drawing | The table holds one sentence that says so | No |
| S24 | The edge that sets the index width | Section 9: every control is a target of at least 24 by 24 px | The edge is 6 px wide, as such an edge is on every platform. A wider one would take clicks from the rows beside it. Left and Right move it while it has focus, 8 px a step | No |
| S25 | The order of the rail | S2.1: "changed by dragging or in Settings". Settings draws no control for it | Dragging a tile, or Alt with Up or Down while a tile has focus | No |
| S26 | The first control of a tab | K1: Enter or Right moves focus to the first control of the tab | A tab without a control keeps focus in the index | No |
| S27 | Two "Open in editor" buttons on one screen | S5.1 draws both with the same label | The labels stay. A screen reader hears "Open Checkout in the editor" and "Open the selected file in the editor" | No |
| S28 | The one file Mortise writes | S5.3, and `AGENTS.md`: `mortise.json` is the only file Mortise writes in an app | Saving the rules writes a short-lived temporary file with a unique name, `.mortise.json.` and random letters, next to `mortise.json`, and then puts it in place of `mortise.json` in one step. So the file is never half written, and no other file of the user is touched. Only a crash during the save can leave the temporary file | No |
| S29 | The strip when its rows do not fit | S13.1 draws the 76 px strip with 7 modules. C16 floats the scrollbar over the right padding of the content, and the strip has no padding right of its marks | When the rows do not fit, the strip is 86 px: the 10 px track of the scrollbar (`scrollbar-w`) is added at its right, and the workspace gives way. The focus ring of a row in the strip runs beside the marks, inside the strip. Rendered with 20 modules at 960 by 600 | No |
| S30 | The order of the neighbours in the graph of one module | S5.1: the 7 with the most references are drawn, and an eighth node reads "12 more" | Links that break a rule first, then the most references, then by name. A link that breaks a rule with few references would otherwise be in the "more" node, which cannot be clicked yet. Tested with 9 modules on one side in `tests/boundary.rs` | No |
| S31 | A node of the graph of the app whose name does not fit | S5.3 draws each node with its code stamp and its full name, for 7 modules. S5.4 allows the graph up to 12 modules and draws no node that is too narrow for its name | A node draws its code stamp, and the name only when all of it fits. A cut name could read as another module. The full name is the tooltip and the accessible name of the node. Rendered with 12 modules at 1100 by 700: Addresses and Notifications show their stamps only. Tested in `tests/boundary.rs` | No |

## Moved between milestones

What the plan placed in one milestone and was built in another. Nothing is dropped.

| What | Planned | Now | Why |
|---|---|---|---|
| Progress bar (C17), "Reading 312 of 551 files" | 2 | 9, built | Reading an app takes milliseconds until PHP files are parsed. The bar is built with the work that can take long |
| Reordering rail tiles by dragging | 2 | 9, built | Built with its key path: Alt with Up or Down. See S25 |
| "Open the command palette" on the first run screen (S1.1) | 2 | 6 | The palette is built in milestone 6. A button that opens nothing is not drawn |
| Removing an app | 2 | 6 | The only place the spec draws it is Settings (S8.1). The core can already remove an app |
| Settings tile on the rail | 1 | 6 | Drawn when Settings exist |
| F6 between regions, arrow keys in the graph, J and K inside tables, Enter or Right from the index into the workspace | 6 | 9, built | Tab and Shift Tab reach every control and the focus ring shows on all of them. The shorter paths of section 7 are built on top of that |
| macOS menu bar | 6 | 9, built | Needs `muda` and a check on a real bundle |
| Dragging the index edge between 220 and 320 px | 6 | 9, built | The widths of section 5.14 that depend on the window are built |
| Motion "System" | 6 | 9, built for macOS | See D3 |
| A shortcut keycap inside a tooltip | 6 | 9, built | Tooltips name the control |
| J and K through the entries of a proposal | 8 | 9, built | With the other table keys. Tab and Shift Tab reach every entry, Space takes it in or leaves it out, and focus is on the first entry when a scan ends |
