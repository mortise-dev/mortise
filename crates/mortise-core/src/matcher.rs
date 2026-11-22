//! The matcher of the command palette: letters in order, not necessarily adjacent.

/// How well a query matches a text. Lower is better. `None` when it does not match.
///
/// The query is split at spaces; every part must be found in the text, in order,
/// as letters in order: `mm bil` finds `module:make-model Billing`. Case does not
/// matter. Matches at the start of a word and without gaps come first.
pub fn fuzzy(query: &str, text: &str) -> Option<u32> {
    let text: Vec<char> = text.chars().flat_map(char::to_lowercase).collect();
    let mut from = 0;
    let mut score = 0;
    for part in query.split_whitespace() {
        let part: Vec<char> = part.chars().flat_map(char::to_lowercase).collect();
        let (cost, end) = best(&part, &text, from)?;
        score += cost;
        from = end;
    }
    Some(score)
}

/// Finds `part` in `text` from `from` on, trying every place it can start, and
/// gives the lowest cost with the place after its last letter.
fn best(part: &[char], text: &[char], from: usize) -> Option<(u32, usize)> {
    let first = *part.first()?;
    let mut found: Option<(u32, usize)> = None;
    for start in (from..text.len()).filter(|at| text[*at] == first) {
        let mut at = start;
        let mut gaps = 0;
        let mut matched = true;
        for letter in &part[1..] {
            match text[at + 1..].iter().position(|c| c == letter) {
                Some(skipped) => {
                    gaps += skipped as u32;
                    at += 1 + skipped;
                }
                None => {
                    matched = false;
                    break;
                }
            }
        }
        if !matched {
            // A later start has even less text to match in.
            break;
        }
        // A match at the start of a word costs nothing for where it starts.
        let word_start = start == 0 || !text[start - 1].is_alphanumeric();
        let cost = gaps * 2
            + if word_start {
                0
            } else {
                3 + (start - from).min(20) as u32
            };
        if found.is_none_or(|(lowest, _)| cost < lowest) {
            found = Some((cost, at + 1));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_in_order_match_also_with_gaps() {
        assert!(fuzzy("chk", "Checkout").is_some());
        assert!(fuzzy("check", "Checkout CHK").is_some());
        assert!(fuzzy("COUT", "checkout").is_some());
        assert!(fuzzy("mm bil", "module:make-model Billing").is_some());
        assert!(fuzzy("", "anything").is_some());
        assert!(fuzzy("  ", "anything").is_some());
    }

    #[test]
    fn letters_out_of_order_or_missing_do_not_match() {
        assert_eq!(fuzzy("zzq", "Checkout"), None);
        assert_eq!(fuzzy("tuo", "Checkout"), None);
        assert_eq!(fuzzy("bil mm", "module:make-model Billing"), None);
        assert_eq!(fuzzy("a", ""), None);
    }

    #[test]
    fn a_closer_match_scores_lower() {
        let score = |text: &str| fuzzy("ship", text).unwrap();
        // The start of the name, without gaps, beats letters spread over it.
        assert!(score("Shipping") < score("Scholarship"));
        // The start of a word beats the middle of one.
        assert!(score("Order shipping") < score("Fellowship"));
        assert_eq!(fuzzy("check", "Checkout"), Some(0));
    }
}
