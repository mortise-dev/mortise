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
| D6 | Tokens 3.3: numbers are tabular everywhere | Not known whether 0.37 can switch on a font's tabular figures | To be decided in milestone 1 | Yes, in the plan | Not checked |
| D7 | Section 11: static font files, because variable fonts were not checked | Variable fonts work in 0.37 (`FontTweak::coords`, `RichText::variation`). Font weight is still not part of `FontId` | Static files are kept as the spec says; each weight is its own named family | No | Source, compiled |

## Notes that are not deviations

Facts the spec was unsure of, which turned out to hold.

- The macOS window buttons stay over custom content: `with_fullsize_content_view`, `with_titlebar_shown(false)`, `with_title_shown(false)`, and `Frame::set_traffic_lights_position` to centre them in a 34 px bar. Double click to zoom is done by the app.
- Screen readers: AccessKit is always part of egui 0.37 and is on by default in eframe for macOS, Windows and Linux. A painted control gives its role, name and state through `Response::widget_info`. A live region is set through `Context::accesskit_node_builder`.
- The floating scrollbar is egui's default style.
- The two shadows fit egui's shadow type (whole pixels, blur up to 255).
- `/` and the backtick are matched as logical keys, so on some layouts they need Shift or a dead key. The spec already gives both a second binding (Ctrl F, Ctrl J).
- egui's own Cmd or Ctrl with plus, minus and 0 zoom is turned off, because text size is a setting.
