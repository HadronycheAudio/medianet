//! The repertoire guard (lead ruling R13).
//!
//! Matching Python's regex and Unicode behaviour character by character has
//! no end, so the ports close the tail by construction: every check first
//! fails with its own criterion `REPERTOIRE` (file:line:column and the code
//! point) when an input document holds a character outside the repertoire,
//! and runs nothing else. Inside the repertoire the port must equal its
//! script; outside it the port is red by rule, and the harness classifies
//! such a row "port stricter (outside repertoire)".
//!
//! The repertoire is `hadronyche-tools/data/repertoire.tsv`: ASCII
//! printable, `\t`, `\n`, `\r`, and the 44 other characters that occur in
//! `doc/` at c930037, each with its Python facts (`\w`, `\d`, `\s`,
//! `splitlines` boundary, `re.I` partners in the repertoire, `lower`,
//! `upper`, `casefold`, NFKC), generated once by
//! `hadronyche-tools/tools/gen_repertoire.py` with the two interpreters the
//! scripts ran on (Python 3.10.19 and 3.13.5, which agree on every row). The
//! tests below prove the library's classes, boundaries and case mapping
//! agree with that table for every character, without running Python.
//!
//! `\r` is in the base set: every script read with `open()` in text mode, so
//! `\r` and `\r\n` reached its code as `\n`, which
//! [`super::pycompat::universal_newlines`] reproduces.
//!
//! A new character enters the repertoire only by adding its row (rerun the
//! generator); the tests then force the proof for it. NFKC is recorded but
//! not implemented by the library: a port that needs it (006-01) adds it with
//! its own proof.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use super::Docs;
use super::report::Report;

/// The committed table.
pub const TABLE: &str = include_str!("../../data/repertoire.tsv");

/// The criterion a check reports for a character outside the repertoire.
pub const CRITERION: &str = "REPERTOIRE";

/// One row of the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub name: String,
    pub word: bool,
    pub digit: bool,
    pub space: bool,
    pub line_boundary: bool,
    /// The other repertoire characters that `re.I` matches.
    pub fold: Vec<char>,
    pub lower: String,
    pub upper: String,
    pub casefold: String,
    pub nfkc: String,
}

fn chars_of(field: &str) -> String {
    if field == "-" {
        return String::new();
    }
    field
        .split(' ')
        .filter_map(|h| u32::from_str_radix(h, 16).ok().and_then(char::from_u32))
        .collect()
}

fn parse_row(line: &str) -> Option<(char, Facts)> {
    let f: Vec<&str> = line.split('\t').collect();
    if f.len() != 11 {
        return None;
    }
    let c = char::from_u32(u32::from_str_radix(f[0], 16).ok()?)?;
    let bit = |s: &str| s == "1";
    Some((
        c,
        Facts {
            name: f[1].to_string(),
            word: bit(f[2]),
            digit: bit(f[3]),
            space: bit(f[4]),
            line_boundary: bit(f[5]),
            fold: chars_of(f[6]).chars().collect(),
            lower: chars_of(f[7]),
            upper: chars_of(f[8]),
            casefold: chars_of(f[9]),
            nfkc: chars_of(f[10]),
        },
    ))
}

/// The repertoire with its facts.
pub fn table() -> &'static BTreeMap<char, Facts> {
    static T: OnceLock<BTreeMap<char, Facts>> = OnceLock::new();
    T.get_or_init(|| {
        TABLE
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(parse_row)
            .collect()
    })
}

/// True when `c` is in the repertoire.
pub fn allowed(c: char) -> bool {
    (' '..='~').contains(&c) || matches!(c, '\t' | '\n' | '\r') || table().contains_key(&c)
}

/// The first character of `text` outside the repertoire: (1-based line,
/// 1-based column in characters, the character).
pub fn first_outside(text: &str) -> Option<(usize, usize, char)> {
    let (mut line, mut col) = (1, 0);
    for c in text.chars() {
        col += 1;
        if !allowed(c) {
            return Some((line, col, c));
        }
        if c == '\n' {
            line += 1;
            col = 0;
        }
    }
    None
}

/// Checks one text; reports and returns false when it holds a character
/// outside the repertoire.
pub fn guard_text(path: &str, text: &str, report: &mut Report) -> bool {
    match first_outside(text) {
        None => true,
        Some((line, col, c)) => {
            report.fail_at(
                CRITERION,
                path,
                line,
                format!(
                    "{path}:{line}:{col}: U+{:04X} is outside the repertoire of the ported checks \
                     (hadronyche-tools/data/repertoire.tsv): the check is red by rule",
                    u32::from(c)
                ),
            );
            false
        }
    }
}

/// Checks every listed document that exists (a missing one is the check's
/// own business); true when all are inside the repertoire.
pub fn guard(docs: &Docs, paths: &[&str], report: &mut Report) -> bool {
    let mut ok = true;
    for p in paths {
        if let Some(text) = docs.read(p) {
            ok &= guard_text(p, &text, report);
        }
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::super::pycompat::{PyStr, is_nd, is_space, is_word, splitlines};
    use super::super::text::rx;
    use super::*;

    fn all() -> Vec<char> {
        (' '..='~').chain(['\t', '\n', '\r']).collect()
    }

    #[test]
    fn the_table_covers_the_base_set_and_the_documents() {
        assert_eq!(
            table().len(),
            142,
            "ASCII printable, \\t, \\n, \\r and the 44 characters of doc/"
        );
        for c in all() {
            assert!(
                table().contains_key(&c),
                "U+{:04X} of the base set has no row",
                u32::from(c)
            );
        }
        assert!(table().keys().all(|c| allowed(*c)));
    }

    #[test]
    fn classes_agree_with_python_for_every_character() {
        for (c, f) in table() {
            let s = c.to_string();
            let tag = format!("U+{:04X} {}", u32::from(*c), f.name);
            assert_eq!(is_word(*c), f.word, "is_word {tag}");
            assert_eq!(rx(r"^\w$").is_match(&s), f.word, r"\w {tag}");
            assert_eq!(is_nd(*c), f.digit, "is_nd {tag}");
            assert_eq!(rx(r"^\d$").is_match(&s), f.digit, r"\d {tag}");
            assert_eq!(is_space(*c), f.space, "is_space {tag}");
            assert_eq!(rx(r"^\s$").is_match(&s), f.space, r"\s {tag}");
            assert_eq!(s.py_strip().is_empty(), f.space, "strip {tag}");
            assert_eq!(
                splitlines(&format!("a{c}b")).len() > 1,
                f.line_boundary,
                "splitlines {tag}"
            );
        }
    }

    #[test]
    fn word_boundaries_agree_with_python_for_every_character() {
        // Python: a\b matches "a" + c exactly when c is not a word character
        for (c, f) in table() {
            let tag = format!("U+{:04X} {}", u32::from(*c), f.name);
            assert_eq!(
                rx(r"^a\b").is_match(&format!("a{c}")),
                !f.word,
                r"^a\b {tag}"
            );
            assert_eq!(
                rx(r"\ba$").is_match(&format!("{c}a")),
                !f.word,
                r"\ba$ {tag}"
            );
            assert_eq!(
                rx(r"^a\B").is_match(&format!("a{c}")),
                f.word,
                r"^a\B {tag}"
            );
        }
    }

    #[test]
    fn case_folding_agrees_with_python_for_every_pair() {
        for (a, f) in table() {
            let pat = rx(&format!("(?i)^{}$", regex::escape(&a.to_string())));
            for b in table().keys().filter(|b| *b != a) {
                assert_eq!(
                    pat.is_match(&b.to_string()),
                    f.fold.contains(b),
                    "re.I U+{:04X} against U+{:04X}",
                    u32::from(*a),
                    u32::from(*b)
                );
            }
        }
    }

    #[test]
    fn lower_and_upper_agree_with_python_for_every_character() {
        for (c, f) in table() {
            assert_eq!(
                c.to_lowercase().collect::<String>(),
                f.lower,
                "lower U+{:04X}",
                u32::from(*c)
            );
            assert_eq!(
                c.to_uppercase().collect::<String>(),
                f.upper,
                "upper U+{:04X}",
                u32::from(*c)
            );
        }
    }

    #[test]
    fn the_guard_reports_line_column_and_code_point() {
        assert_eq!(first_outside("ok § µ ²\nfine"), None);
        assert_eq!(first_outside("a\nbc\u{301}"), Some((2, 3, '\u{301}')));
        let mut r = Report::default();
        assert!(!guard_text("doc/x.md", "x\ny İ", &mut r));
        assert!(
            r.render()
                .contains("doc/x.md:2:3: U+0130 is outside the repertoire")
        );
        assert!(guard_text("doc/y.md", "plain", &mut r));
        for bad in [
            '\u{2028}', '\u{0c}', '\u{85}', '\u{1d}', '\u{1f}', '\u{a0}', '\u{667}', '\u{130}',
            '\u{301}',
        ] {
            assert!(!allowed(bad), "U+{:04X}", u32::from(bad));
        }
    }
}
