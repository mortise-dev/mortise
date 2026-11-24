//! Command output with its escape sequences read into four styles.
//!
//! Colour codes map to [`Style`]. Every other escape sequence is removed.

/// How a piece of an output line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Text,
    Dim,
    Success,
    Error,
}

/// A run of text in one style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub style: Style,
    pub text: String,
}

/// The spans of one line. The line starts in `Text`. Neighbouring spans of one
/// style are merged and empty spans are dropped.
pub fn spans(line: &str) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::new();
    let mut style = Style::Text;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            // Keep only the text after the last carriage return.
            '\r' => out.clear(),
            '\u{1b}' => match chars.next() {
                Some('[') => {
                    let mut params = String::new();
                    // Parameter and intermediate bytes run up to a final byte in `@` to `~`.
                    for c in chars.by_ref() {
                        if ('@'..='~').contains(&c) {
                            if c == 'm' {
                                style = sgr(&params, style);
                            }
                            break;
                        }
                        params.push(c);
                    }
                }
                Some(']') => {
                    // An OSC string ends at BEL or at `ESC \`.
                    while let Some(c) = chars.next() {
                        if c == '\u{7}' {
                            break;
                        }
                        if c == '\u{1b}' {
                            chars.next();
                            break;
                        }
                    }
                }
                // A lone ESC and the next character.
                _ => {}
            },
            c => match out.last_mut() {
                Some(last) if last.style == style => last.text.push(c),
                _ => out.push(Span {
                    style,
                    text: c.to_string(),
                }),
            },
        }
    }
    out
}

/// The line without any escape sequence.
pub fn plain(line: &str) -> String {
    spans(line).into_iter().map(|span| span.text).collect()
}

/// The deepest JSON that [`json_lines`] indents. Each level adds two spaces to every
/// line inside it, so the depth bounds the indented form at a few times the line.
const JSON_DEPTH: usize = 32;

/// The lines of a line that holds one JSON object or array, indented by two spaces
/// per level, with the keys in the order the command wrote them. `None` when the
/// line holds something else, an empty object or array, or JSON nested deeper than
/// `JSON_DEPTH` levels.
pub fn json_lines(line: &str) -> Option<Vec<String>> {
    let text = line.trim();
    if !(text.starts_with('{') || text.starts_with('['))
        || serde_json::from_str::<serde::de::IgnoredAny>(text).is_err()
    {
        return None;
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = text.chars().peekable();
    let break_line = |lines: &mut Vec<String>, current: &mut String, depth: usize| {
        lines.push(std::mem::take(current));
        current.push_str(&"  ".repeat(depth));
    };
    while let Some(c) = chars.next() {
        if in_string {
            current.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                current.push(c);
            }
            '{' | '[' => {
                while chars.next_if(|next| next.is_whitespace()).is_some() {}
                current.push(c);
                if chars.next_if(|next| matches!(next, '}' | ']')).is_some() {
                    current.push(if c == '{' { '}' } else { ']' });
                } else {
                    depth += 1;
                    // Deep nesting would grow the indentation past any use.
                    if depth > JSON_DEPTH {
                        return None;
                    }
                    break_line(&mut lines, &mut current, depth);
                }
            }
            '}' | ']' => {
                depth -= 1;
                break_line(&mut lines, &mut current, depth);
                current.push(c);
            }
            ',' => {
                current.push(c);
                break_line(&mut lines, &mut current, depth);
            }
            ':' => current.push_str(": "),
            c if c.is_whitespace() => {}
            c => current.push(c),
        }
    }
    lines.push(current);
    (lines.len() > 1).then_some(lines)
}

/// The style after the codes of one SGR sequence, read left to right.
fn sgr(params: &str, mut style: Style) -> Style {
    let mut codes = params.split(';');
    while let Some(code) = codes.next() {
        match code {
            "" | "0" | "22" | "39" | "49" => style = Style::Text,
            "31" | "91" | "41" | "101" => style = Style::Error,
            "32" | "92" | "42" | "102" => style = Style::Success,
            "2" | "33" | "93" | "43" | "103" | "90" => style = Style::Dim,
            // Extended colours keep the style. Their arguments are not codes.
            "38" | "48" => {
                let skip = match codes.next() {
                    Some("5") => 1,
                    Some("2") => 3,
                    _ => 0,
                };
                codes.by_ref().take(skip).for_each(drop);
            }
            _ => {}
        }
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_follow_the_table() {
        use Style::*;
        // A line, its spans and its plain text.
        type Row<'a> = (&'a str, &'a [(Style, &'a str)], &'a str);
        let table: &[Row] = &[
            ("plain", &[(Text, "plain")], "plain"),
            (
                "\x1b[31mbad\x1b[0m ok",
                &[(Error, "bad"), (Text, " ok")],
                "bad ok",
            ),
            (
                "\x1b[102mdone\x1b[39m!",
                &[(Success, "done"), (Text, "!")],
                "done!",
            ),
            ("\x1b[33mRUNNING\x1b[m", &[(Dim, "RUNNING")], "RUNNING"),
            ("\x1b[90ma\x1b[22mb", &[(Dim, "a"), (Text, "b")], "ab"),
            // The last code that sets a style wins, and an unknown code changes nothing.
            ("\x1b[31;1;32mx", &[(Success, "x")], "x"),
            // Extended colour arguments are not read as codes.
            ("\x1b[31m\x1b[38;5;31ma", &[(Error, "a")], "a"),
            ("\x1b[2m\x1b[48;2;32;31;2mb", &[(Dim, "b")], "b"),
            // Neighbouring spans of one style merge and empty spans go.
            ("\x1b[31ma\x1b[41mb\x1b[0m\x1b[0m", &[(Error, "ab")], "ab"),
            // Other escape sequences are removed.
            ("a\x1b[2Kb\x1b[1;5Hc", &[(Text, "abc")], "abc"),
            (
                "\x1b]8;;http://x\x07link\x1b]8;;\x1b\\ end",
                &[(Text, "link end")],
                "link end",
            ),
            ("a\x1bMb", &[(Text, "ab")], "ab"),
            // Only the text after the last carriage return stays.
            ("10%\r50%\r\x1b[32m100%", &[(Success, "100%")], "100%"),
            ("", &[], ""),
        ];
        for (line, want, text) in table {
            let want: Vec<Span> = want
                .iter()
                .map(|(style, text)| Span {
                    style: *style,
                    text: text.to_string(),
                })
                .collect();
            assert_eq!(spans(line), want, "{line:?}");
            assert_eq!(plain(line), *text, "{line:?}");
        }
    }

    #[test]
    fn json_lines_indent_a_json_line_in_its_order() {
        let line = r#"  {"b":[1, {"x":"a,b:{\"}"}],"a":{},"c":[ ]}"#;
        assert_eq!(
            json_lines(line).unwrap(),
            [
                "{",
                r#"  "b": ["#,
                "    1,",
                "    {",
                r#"      "x": "a,b:{\"}""#,
                "    }",
                "  ],",
                r#"  "a": {},"#,
                r#"  "c": []"#,
                "}",
            ]
        );
        for other in [
            "INFO Checking",
            "{not json",
            "[]",
            "\"text\"",
            "{\"a\":1} tail",
            &format!("{}1{}", "[".repeat(33), "]".repeat(33)),
        ] {
            assert_eq!(json_lines(other), None, "{other:?}");
        }
    }
}
