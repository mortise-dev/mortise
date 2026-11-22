//! The three letter code of a module, as shown on its stamp.

use std::collections::BTreeSet;

/// The code of the "All modules" row. No module can have it.
pub const ALL_CODE: &str = "ALL";

/// Proposes a code for every name, in the order given. Each code is three capital
/// letters or digits, differs from all the others and is never [`ALL_CODE`].
///
/// A code should be kept once given, so that it can be learned: pass the codes that
/// exist already in `taken`, and store the result.
pub fn propose_codes<'a>(
    names: impl IntoIterator<Item = &'a str>,
    taken: &BTreeSet<String>,
) -> Vec<String> {
    let mut taken = taken.clone();
    taken.insert(ALL_CODE.to_owned());
    names
        .into_iter()
        .map(|name| {
            let code = propose_code(name, &taken);
            taken.insert(code.clone());
            code
        })
        .collect()
}

fn propose_code(name: &str, taken: &BTreeSet<String>) -> String {
    let letters: Vec<char> = name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_uppercase())
        .collect();
    let letters = if letters.is_empty() {
        vec!['M']
    } else {
        letters
    };
    let first = letters[0];
    let free = |code: &String| !taken.contains(code);

    // The first letter and the consonants that follow: Checkout is CHK, Notifications is NTF.
    let consonants = consonant_code(&letters);
    if free(&consonants) {
        return consonants;
    }

    // The first three letters.
    if let [a, b, c, ..] = letters[..] {
        let code = String::from_iter([a, b, c]);
        if free(&code) {
            return code;
        }
    }

    // The first letter and any two later letters, in the order of the name.
    for (i, &second) in letters.iter().enumerate().skip(1) {
        for &third in &letters[i + 1..] {
            let code = String::from_iter([first, second, third]);
            if free(&code) {
                return code;
            }
        }
    }

    // Two letters and a digit, then one letter and a number.
    let second = consonants.chars().nth(1).unwrap_or('X');
    (2..=9)
        .map(|n| format!("{first}{second}{n}"))
        .chain((10..=99).map(|n| format!("{first}{n}")))
        .chain((100..=999).map(|n| n.to_string()))
        .find(free)
        .unwrap_or(consonants)
}

/// Proposes the two letter monogram of an app from the name of its folder: the
/// initials of its first two words, or its first two letters. If that is taken,
/// the first two letters; if that is taken too, the first letter and the next
/// free letter.
pub fn propose_monogram(name: &str, taken: &BTreeSet<String>) -> String {
    let words: Vec<&str> = name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    let letters: Vec<char> = words
        .iter()
        .flat_map(|word| word.chars())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    let Some(&first) = letters.first() else {
        return "??".to_owned();
    };

    let initials: String = match words[..] {
        [one, two, ..] => one.chars().take(1).chain(two.chars().take(1)).collect(),
        _ => String::new(),
    };
    let first_two: String = letters.iter().take(2).collect();
    let later = letters[1..]
        .iter()
        .copied()
        .chain('A'..='Z')
        .chain('0'..='9');

    [initials.to_ascii_uppercase(), first_two]
        .into_iter()
        .filter(|candidate| !candidate.is_empty())
        .chain(later.map(|second| String::from_iter([first, second])))
        .find(|candidate| !taken.contains(candidate))
        .unwrap_or_else(|| String::from_iter([first, first]))
}

fn consonant_code(letters: &[char]) -> String {
    let mut code = vec![letters[0]];
    let is_vowel = |c: char| matches!(c, 'A' | 'E' | 'I' | 'O' | 'U');

    // Consonants first, then whatever letters are left, each at most once.
    for pass in [true, false] {
        for &letter in &letters[1..] {
            if code.len() == 3 {
                break;
            }
            if (!pass || !is_vowel(letter)) && !code.contains(&letter) {
                code.push(letter);
            }
        }
    }
    // A consonant found late must not jump in front of an earlier vowel that was
    // only added to fill up: keep the order of the name.
    code[1..].sort_by_key(|c| letters[1..].iter().position(|l| l == c));
    while code.len() < 3 {
        code.push('X');
    }
    code.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(names: &[&str]) -> Vec<String> {
        propose_codes(names.iter().copied(), &BTreeSet::new())
    }

    #[test]
    fn the_examples_of_the_specification() {
        assert_eq!(
            codes(&["Checkout", "Notifications", "Shipping"]),
            ["CHK", "NTF", "SHP"]
        );
    }

    #[test]
    fn first_letter_then_consonants_then_the_rest() {
        assert_eq!(
            codes(&[
                "Users",
                "Inventory",
                "Search",
                "Cart",
                "Blog",
                "Api",
                "Io",
                "X"
            ]),
            ["USR", "INV", "SRC", "CRT", "BLG", "API", "IOX", "XXX"]
        );
    }

    #[test]
    fn a_code_is_unique_and_never_all() {
        let names = ["Inventory", "Invoices", "Invites", "Alloy", "Allow"];
        let codes = codes(&names);
        let unique: BTreeSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), names.len(), "{codes:?}");
        assert!(!codes.iter().any(|code| code == ALL_CODE), "{codes:?}");
        assert_eq!(codes[0], "INV");
        assert!(codes.iter().all(|code| code.len() == 3), "{codes:?}");
    }

    #[test]
    fn existing_codes_are_not_given_again() {
        let taken = BTreeSet::from(["CHK".to_owned()]);
        let codes = propose_codes(["Checkout"], &taken);
        assert_ne!(codes[0], "CHK");
        assert!(codes[0].starts_with('C'));
    }

    #[test]
    fn many_modules_with_the_same_letters_still_get_codes() {
        let names: Vec<String> = (0..200).map(|_| "Aa".to_owned()).collect();
        let codes = propose_codes(names.iter().map(String::as_str), &BTreeSet::new());
        let unique: BTreeSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), 200);
    }

    #[test]
    fn a_monogram_is_two_initials_or_the_first_two_letters() {
        let none = BTreeSet::new();
        assert_eq!(propose_monogram("booking-engine", &none), "BE");
        assert_eq!(propose_monogram("landing_pages.v2", &none), "LP");
        assert_eq!(propose_monogram("crm", &none), "CR");
        assert_eq!(propose_monogram("x", &none), "X");
        assert_eq!(propose_monogram("shop api final", &none), "SA");
        assert_eq!(propose_monogram("---", &none), "??");
    }

    #[test]
    fn a_taken_monogram_is_not_given_again() {
        let taken = |codes: &[&str]| codes.iter().map(|code| (*code).to_owned()).collect();
        // The initials are taken: the first two letters.
        assert_eq!(propose_monogram("booking-engine", &taken(&["BE"])), "BO");
        // Both are taken: the first letter and the next free letter.
        assert_eq!(
            propose_monogram("booking-engine", &taken(&["BE", "BO"])),
            "BK"
        );
        assert_eq!(propose_monogram("crm", &taken(&["CR"])), "CM");
        assert_eq!(propose_monogram("x", &taken(&["X"])), "XA");
    }

    #[test]
    fn names_without_ascii_letters_get_a_code() {
        assert_eq!(codes(&["Ünïcödé"]), ["NCD"]);
        assert_eq!(codes(&["日本"]), ["MXX"]);
        assert_eq!(codes(&[""]), ["MXX"]);
    }
}
