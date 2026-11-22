//! The Slate tokens of the specification, section 3: colour, type and size.
//!
//! The values are copied from the JSON block in `docs/spec/tokens.html`.
//! A test compares them with that block, so they cannot drift.

use eframe::egui::{Color32, FontFamily, FontId, TextFormat, Theme};

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// Every colour of one theme. Light and dark are both complete; neither is derived.
#[derive(Debug, PartialEq)]
pub struct Palette {
    /// Title bar and app rail.
    pub frame: Color32,
    /// Module index, workspace header strip.
    pub panel: Color32,
    /// Workspace, selected row, inputs, overlays.
    pub surface: Color32,
    /// Bar tracks, keycaps, hover fill.
    pub well: Color32,
    /// Primary text, bars, peg, focus ring.
    pub ink: Color32,
    /// Secondary text, code stamps, outlines on frame.
    pub ink_2: Color32,
    /// Placeholders, key hints, disabled labels. Only on surface and panel.
    pub ink_3: Color32,
    /// Hairlines inside a panel, pressed fill.
    pub line: Color32,
    /// Panel joints, control outlines on panel, surface and well. Never an outline on frame.
    pub line_2: Color32,
    /// Primary button fill, tooltip fill.
    pub key: Color32,
    pub key_fg: Color32,
    pub good: Color32,
    /// Never as text on frame.
    pub warn: Color32,
    pub bad: Color32,
    pub bad_fg: Color32,
    /// Module turned off, runtime Unknown. Same limits as `ink_3`.
    pub off: Color32,
    pub bad_soft: Color32,
    pub code_bg: Color32,
    pub code_fg: Color32,
    pub code_dim: Color32,
    pub code_ok: Color32,
    pub code_err: Color32,
    pub key_hover: Color32,
    pub key_press: Color32,
    /// Layer under palette and dialog. Has alpha.
    pub scrim: Color32,
    /// Overlay shadow colour. Has alpha.
    pub shadow: Color32,
}

pub const LIGHT: Palette = Palette {
    frame: hex(0xD1D8E2),
    panel: hex(0xE5EAF0),
    surface: hex(0xF9FAFC),
    well: hex(0xDBE1E9),
    ink: hex(0x1A1E25),
    ink_2: hex(0x48515C),
    ink_3: hex(0x5B6570),
    line: hex(0xBCC6D3),
    line_2: hex(0x747F8C),
    key: hex(0x1A1E25),
    key_fg: hex(0xF9FAFC),
    good: hex(0x2C6A37),
    warn: hex(0x855600),
    bad: hex(0xB02A20),
    bad_fg: hex(0xF9FAFC),
    off: hex(0x5B6570),
    bad_soft: hex(0xF8E2DD),
    code_bg: hex(0x1A1E25),
    code_fg: hex(0xE5EBF4),
    code_dim: hex(0x93A0B0),
    code_ok: hex(0x8FD49B),
    code_err: hex(0xF59A8E),
    key_hover: hex(0x3E4147),
    key_press: hex(0x5D6066),
    // rgba(26,30,37,.42) and rgba(10,13,18,.30), premultiplied.
    scrim: Color32::from_rgba_premultiplied(11, 13, 16, 107),
    shadow: Color32::from_rgba_premultiplied(3, 4, 5, 77),
};

pub const DARK: Palette = Palette {
    frame: hex(0x0F1216),
    panel: hex(0x15191E),
    surface: hex(0x1D2229),
    well: hex(0x11151B),
    ink: hex(0xE5EBF4),
    ink_2: hex(0xA1AEBE),
    ink_3: hex(0x8895A5),
    line: hex(0x2E353E),
    line_2: hex(0x657180),
    key: hex(0xE5EBF4),
    key_fg: hex(0x15191E),
    good: hex(0x84CC90),
    warn: hex(0xE5B152),
    bad: hex(0xF28B7F),
    bad_fg: hex(0x15191E),
    off: hex(0x8895A5),
    bad_soft: hex(0x3A201B),
    code_bg: hex(0x0B0E12),
    code_fg: hex(0xE5EBF4),
    code_dim: hex(0x93A0B0),
    code_ok: hex(0x8FD49B),
    code_err: hex(0xF59A8E),
    key_hover: hex(0xC5CBD4),
    key_press: hex(0xA9AFB7),
    // rgba(6,6,8,.6) and rgba(0,0,0,.50), premultiplied.
    scrim: Color32::from_rgba_premultiplied(4, 4, 5, 153),
    shadow: Color32::from_rgba_premultiplied(0, 0, 0, 128),
};

pub fn palette(theme: Theme) -> &'static Palette {
    match theme {
        Theme::Light => &LIGHT,
        Theme::Dark => &DARK,
    }
}

/// A bundled font file. Each weight is its own family, because egui has no font weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Sans400,
    Sans500,
    Sans600,
    Sans700,
    Mono400,
    Mono500,
    Mono600,
}

impl Face {
    pub const ALL: [Face; 7] = [
        Face::Sans400,
        Face::Sans500,
        Face::Sans600,
        Face::Sans700,
        Face::Mono400,
        Face::Mono500,
        Face::Mono600,
    ];

    /// The name of the egui font family that holds this face.
    pub fn family_name(self) -> &'static str {
        match self {
            Face::Sans400 => "asap-400",
            Face::Sans500 => "asap-500",
            Face::Sans600 => "asap-600",
            Face::Sans700 => "asap-700",
            Face::Mono400 => "inconsolata-400",
            Face::Mono500 => "inconsolata-500",
            Face::Mono600 => "inconsolata-600",
        }
    }

    pub fn family(self) -> FontFamily {
        FontFamily::Name(self.family_name().into())
    }
}

/// A type token: size and line height in logical pixels at 100% text size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Type {
    pub size: f32,
    pub line: f32,
    pub face: Face,
    /// Letter spacing in em.
    pub spacing: f32,
}

impl Type {
    const fn new(size: f32, line: f32, face: Face) -> Self {
        Self {
            size,
            line,
            face,
            spacing: 0.0,
        }
    }

    /// The same token in another weight.
    pub const fn with_face(self, face: Face) -> Self {
        Self { face, ..self }
    }

    pub fn format(self, color: Color32) -> TextFormat {
        TextFormat {
            font_id: FontId::new(self.size, self.face.family()),
            line_height: Some(self.line),
            extra_letter_spacing: self.spacing * self.size,
            color,
            ..Default::default()
        }
    }
}

/// The type tokens of section 3.3.
pub mod ty {
    use super::{Face, Type};

    /// Workspace title.
    pub const TITLE: Type = Type::new(18.0, 22.0, Face::Sans700);
    /// Stat figure.
    pub const FIGURE: Type = Type::new(19.0, 26.0, Face::Sans600);
    /// Command palette query.
    pub const PALETTE_INPUT: Type = Type::new(15.0, 20.0, Face::Sans400);
    /// Dialog title.
    pub const OVERLAY_TITLE: Type = Type::new(15.0, 20.0, Face::Sans700);
    /// Module name in the index, brand in the title bar.
    pub const ROW_NAME: Type = Type::new(13.0, 17.0, Face::Sans600);
    /// Body text, table cells, list lines.
    pub const BODY: Type = Type::new(12.5, 18.0, Face::Sans400);
    /// Tab label. 600 when selected.
    pub const TAB: Type = Type::new(12.5, 18.0, Face::Sans500);
    /// Button label.
    pub const CONTROL: Type = Type::new(12.0, 12.0, Face::Sans600);
    /// Field label, state word, table header.
    pub const LABEL: Type = Type::new(11.5, 15.0, Face::Sans600);
    /// Secondary text, hints.
    pub const SECONDARY: Type = Type::new(11.5, 15.0, Face::Sans400);
    /// Command preview, file tree.
    pub const CODE: Type = Type::new(12.0, 19.0, Face::Mono400);
    /// Runner output.
    pub const RUNNER: Type = Type::new(11.5, 18.0, Face::Mono400);
    /// Module code stamp.
    pub const STAMP: Type = Type {
        spacing: 0.03,
        ..Type::new(10.5, 16.0, Face::Mono600)
    };
    /// Key hint.
    pub const KEYCAP: Type = Type::new(10.5, 16.0, Face::Mono500);

    // Sizes the drawings use that section 3.3 does not name.

    /// App name at the top of the index: `.app-name` in the drawings.
    pub const APP_NAME: Type = Type::new(14.0, 20.0, Face::Sans700);
    /// Monogram on a rail tile: `.tile` in the drawings.
    pub const MONOGRAM: Type = Type {
        spacing: 0.02,
        ..Type::new(11.0, 11.0, Face::Sans600)
    };
    /// A path beside a title, in Inconsolata at 0.95 of the secondary size: `.mono` in the drawings.
    pub const PATH: Type = Type::new(11.0, 15.0, Face::Mono400);
}

/// The size tokens of section 3.4, in logical pixels.
pub mod size {
    pub const TITLEBAR_H: f32 = 34.0;
    pub const RAIL_W: f32 = 52.0;
    pub const RAIL_TILE: f32 = 34.0;
    pub const INDEX_W: f32 = 264.0;
    pub const INDEX_W_MIN: f32 = 220.0;
    pub const INDEX_W_MAX: f32 = 320.0;
    pub const INDEX_STRIP_W: f32 = 76.0;
    pub const INDEX_ROW_H: f32 = 40.0;
    pub const INDEX_ROW_COMPACT_H: f32 = 26.0;
    pub const CONTROL_H: f32 = 26.0;
    pub const CONTROL_SM_H: f32 = 22.0;
    pub const FIELD_H: f32 = 28.0;
    pub const TAB_H: f32 = 31.0;
    pub const TABLE_HEAD_H: f32 = 26.0;
    pub const TABLE_ROW_H: f32 = 28.0;
    pub const LIST_LINE_H: f32 = 27.0;
    pub const PALETTE_W: f32 = 620.0;
    pub const PALETTE_INPUT_H: f32 = 44.0;
    pub const PALETTE_ROW_H: f32 = 32.0;
    pub const DIALOG_W: f32 = 440.0;
    pub const TOAST_W: f32 = 320.0;
    pub const TOOLTIP_MAX_W: f32 = 240.0;
    pub const DRAWER_H_DEFAULT: f32 = 200.0;
    pub const DRAWER_H_MIN: f32 = 120.0;
    pub const DRAWER_COLLAPSED_H: f32 = 28.0;
    pub const SCROLLBAR_W: f32 = 10.0;
    pub const SCROLLBAR_THUMB_W: f32 = 6.0;
    pub const SCROLLBAR_THUMB_MIN: f32 = 24.0;
    pub const STATE_MARK: f32 = 12.0;
    pub const STAMP_W: f32 = 34.0;
    pub const STAMP_H: f32 = 18.0;
    pub const CHECKBOX: f32 = 14.0;
    pub const ICON: f32 = 16.0;
    pub const ICON_IN_CONTROL: f32 = 13.0;
    pub const PEG: f32 = 5.0;
    pub const BAR_H: f32 = 6.0;
    pub const WINDOW_MIN_W: f32 = 960.0;
    pub const WINDOW_MIN_H: f32 = 600.0;
    pub const HIT_MIN: f32 = 24.0;

    // Section 5.14: the workspace header is 76 px high.
    pub const WORKSPACE_HEAD_H: f32 = 76.0;
    // Section 8: the space left for the three window buttons of macOS.
    pub const MAC_BUTTONS_W: f32 = 78.0;
    // Section 8: a window control drawn by Mortise.
    pub const WINDOW_CONTROL_W: f32 = 44.0;
}

/// The radius tokens of section 3.5.
pub mod radius {
    pub const STAMP: u8 = 2;
    pub const CONTROL: u8 = 3;
    pub const OVERLAY: u8 = 4;
    pub const WINDOW: u8 = 8;
}

/// The stroke tokens of section 3.5.
pub mod stroke {
    pub const HAIRLINE: f32 = 1.0;
    pub const ICON: f32 = 1.5;
    pub const CHECKBOX: f32 = 1.5;
    pub const FOCUS_RING: f32 = 2.0;
    pub const FOCUS_OFFSET: f32 = 1.0;
    pub const PROBLEM_LINK: f32 = 2.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The JSON block of `docs/spec/tokens.html`.
    fn spec_tokens() -> serde_json::Value {
        let page = include_str!("../../../docs/spec/tokens.html");
        let open = "<pre class=\"json\" id=\"tokens-json\">";
        let start = page.find(open).expect("the JSON block") + open.len();
        let end = start
            + page[start..]
                .find("</pre>")
                .expect("the end of the JSON block");
        let json = page[start..end]
            .replace("&quot;", "\"")
            .replace("&amp;", "&");
        serde_json::from_str(&json).expect("valid JSON in the specification")
    }

    fn rgb(color: Color32) -> String {
        format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
    }

    #[test]
    fn colours_match_the_specification() {
        let spec = spec_tokens();
        for (theme, palette) in [("light", &LIGHT), ("dark", &DARK)] {
            let ours = [
                ("frame", palette.frame),
                ("panel", palette.panel),
                ("surface", palette.surface),
                ("well", palette.well),
                ("ink", palette.ink),
                ("ink-2", palette.ink_2),
                ("ink-3", palette.ink_3),
                ("line", palette.line),
                ("line-2", palette.line_2),
                ("key", palette.key),
                ("key-fg", palette.key_fg),
                ("good", palette.good),
                ("warn", palette.warn),
                ("bad", palette.bad),
                ("bad-fg", palette.bad_fg),
                ("off", palette.off),
                ("bad-soft", palette.bad_soft),
                ("code-bg", palette.code_bg),
                ("code-fg", palette.code_fg),
                ("code-dim", palette.code_dim),
                ("code-ok", palette.code_ok),
                ("code-err", palette.code_err),
                ("key-hover", palette.key_hover),
                ("key-press", palette.key_press),
            ];
            let colors = spec["color"].as_object().unwrap();
            assert_eq!(colors.len(), ours.len(), "a colour token is missing");
            for (name, color) in ours {
                assert_eq!(colors[name][theme], rgb(color), "{name}, {theme}");
            }
        }
    }

    #[test]
    fn colours_with_alpha_match_the_specification() {
        let spec = spec_tokens();
        for (theme, palette) in [("light", &LIGHT), ("dark", &DARK)] {
            for (name, color) in [("scrim", palette.scrim), ("shadow", palette.shadow)] {
                let css = spec["colorAlpha"][name][theme].as_str().unwrap();
                let parts: Vec<f32> = css
                    .trim_start_matches("rgba(")
                    .trim_end_matches(')')
                    .split(',')
                    .map(|part| part.trim().parse().unwrap())
                    .collect();
                let alpha = parts[3];
                let expected = [
                    (parts[0] * alpha).round() as u8,
                    (parts[1] * alpha).round() as u8,
                    (parts[2] * alpha).round() as u8,
                    (alpha * 255.0).round() as u8,
                ];
                assert_eq!(color.to_array(), expected, "{name}, {theme}");
            }
        }
    }

    #[test]
    fn type_matches_the_specification() {
        let spec = spec_tokens();
        let ours = [
            ("title", ty::TITLE),
            ("figure", ty::FIGURE),
            ("palette-input", ty::PALETTE_INPUT),
            ("overlay-title", ty::OVERLAY_TITLE),
            ("row-name", ty::ROW_NAME),
            ("body", ty::BODY),
            ("tab", ty::TAB),
            ("control", ty::CONTROL),
            ("label", ty::LABEL),
            ("secondary", ty::SECONDARY),
            ("code", ty::CODE),
            ("runner", ty::RUNNER),
            ("stamp", ty::STAMP),
            ("keycap", ty::KEYCAP),
        ];
        let types = spec["type"].as_object().unwrap();
        assert_eq!(types.len(), ours.len(), "a type token is missing");
        for (name, token) in ours {
            let face = match (
                types[name]["family"].as_str().unwrap(),
                &types[name]["weight"],
            ) {
                ("sans", weight) if weight == 400 => Face::Sans400,
                ("sans", weight) if weight == 500 => Face::Sans500,
                ("sans", weight) if weight == 600 => Face::Sans600,
                ("sans", weight) if weight == 700 => Face::Sans700,
                ("mono", weight) if weight == 400 => Face::Mono400,
                ("mono", weight) if weight == 500 => Face::Mono500,
                ("mono", weight) if weight == 600 => Face::Mono600,
                other => panic!("{name}: no bundled face for {other:?}"),
            };
            assert_eq!(types[name]["size"], token.size, "{name} size");
            assert_eq!(types[name]["line"], token.line, "{name} line");
            assert_eq!(face, token.face, "{name} face");
        }
    }

    #[test]
    fn sizes_radii_and_strokes_match_the_specification() {
        let spec = spec_tokens();
        let sizes = [
            ("titlebar-h", size::TITLEBAR_H),
            ("rail-w", size::RAIL_W),
            ("rail-tile", size::RAIL_TILE),
            ("index-w", size::INDEX_W),
            ("index-w-min", size::INDEX_W_MIN),
            ("index-w-max", size::INDEX_W_MAX),
            ("index-strip-w", size::INDEX_STRIP_W),
            ("index-row-h", size::INDEX_ROW_H),
            ("index-row-compact-h", size::INDEX_ROW_COMPACT_H),
            ("control-h", size::CONTROL_H),
            ("control-sm-h", size::CONTROL_SM_H),
            ("field-h", size::FIELD_H),
            ("tab-h", size::TAB_H),
            ("table-head-h", size::TABLE_HEAD_H),
            ("table-row-h", size::TABLE_ROW_H),
            ("list-line-h", size::LIST_LINE_H),
            ("palette-w", size::PALETTE_W),
            ("palette-input-h", size::PALETTE_INPUT_H),
            ("palette-row-h", size::PALETTE_ROW_H),
            ("dialog-w", size::DIALOG_W),
            ("toast-w", size::TOAST_W),
            ("tooltip-max-w", size::TOOLTIP_MAX_W),
            ("drawer-h-default", size::DRAWER_H_DEFAULT),
            ("drawer-h-min", size::DRAWER_H_MIN),
            ("drawer-collapsed-h", size::DRAWER_COLLAPSED_H),
            ("scrollbar-w", size::SCROLLBAR_W),
            ("scrollbar-thumb-w", size::SCROLLBAR_THUMB_W),
            ("scrollbar-thumb-min", size::SCROLLBAR_THUMB_MIN),
            ("state-mark", size::STATE_MARK),
            ("stamp-w", size::STAMP_W),
            ("stamp-h", size::STAMP_H),
            ("checkbox", size::CHECKBOX),
            ("icon", size::ICON),
            ("icon-in-control", size::ICON_IN_CONTROL),
            ("peg", size::PEG),
            ("bar-h", size::BAR_H),
            ("window-min-w", size::WINDOW_MIN_W),
            ("window-min-h", size::WINDOW_MIN_H),
            ("hit-min", size::HIT_MIN),
        ];
        let spec_sizes = spec["size"].as_object().unwrap();
        assert_eq!(spec_sizes.len(), sizes.len(), "a size token is missing");
        for (name, value) in sizes {
            assert_eq!(spec_sizes[name], value, "{name}");
        }

        assert_eq!(spec["radius"]["stamp"], radius::STAMP);
        assert_eq!(spec["radius"]["control"], radius::CONTROL);
        assert_eq!(spec["radius"]["overlay"], radius::OVERLAY);
        assert_eq!(spec["radius"]["window"], radius::WINDOW);

        assert_eq!(spec["stroke"]["hairline"], stroke::HAIRLINE);
        assert_eq!(spec["stroke"]["icon"], stroke::ICON);
        assert_eq!(spec["stroke"]["checkbox"], stroke::CHECKBOX);
        assert_eq!(spec["stroke"]["focus-ring"], stroke::FOCUS_RING);
        assert_eq!(spec["stroke"]["focus-offset"], stroke::FOCUS_OFFSET);
        assert_eq!(spec["stroke"]["problem-link"], stroke::PROBLEM_LINK);
    }

    /// The contrast of two colours, as WCAG 2 works it out.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let channel = |c: u8| {
            let c = f32::from(c) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let luminance = |c: Color32| {
            0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
        };
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn text_is_4_5_to_1_and_marks_and_outlines_are_3_to_1() {
        let mut low = Vec::new();
        for (theme, pal) in [("light", &LIGHT), ("dark", &DARK)] {
            // Text, on each fill it is drawn on. Disabled text and placeholders are
            // held to 4.5 to 1 as well.
            let fills = [
                ("surface", pal.surface),
                ("panel", pal.panel),
                ("well", pal.well),
            ];
            let texts = [
                ("ink", pal.ink),
                ("ink-2", pal.ink_2),
                ("ink-3", pal.ink_3),
                ("good", pal.good),
                ("bad", pal.bad),
                ("warn", pal.warn),
                ("off", pal.off),
            ];
            let mut pairs: Vec<(String, Color32, Color32, f32)> = Vec::new();
            for (fill_name, fill) in fills {
                for (text_name, text) in texts {
                    pairs.push((format!("{text_name} on {fill_name}"), text, fill, 4.5));
                }
                // An outline, and a mark, on the same fills.
                pairs.push((format!("line-2 on {fill_name}"), pal.line_2, fill, 3.0));
            }
            pairs.push(("ink on frame".into(), pal.ink, pal.frame, 4.5));
            pairs.push(("ink-2 on frame".into(), pal.ink_2, pal.frame, 4.5));
            pairs.push(("key-fg on key".into(), pal.key_fg, pal.key, 4.5));
            pairs.push(("code-fg on code-bg".into(), pal.code_fg, pal.code_bg, 4.5));
            for (name, a, b, least) in pairs {
                let ratio = contrast(a, b);
                if ratio < least {
                    low.push(format!("{theme}: {name} is {ratio:.2} to 1, under {least}"));
                }
            }
        }
        assert!(low.is_empty(), "\n{}", low.join("\n"));
    }
}
