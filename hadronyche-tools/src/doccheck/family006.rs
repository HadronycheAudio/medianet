//! The 006 family's shared text helpers (006-01 and 006-02 carry identical
//! copies of `norm_text`; one implementation here).
//!
//! Proven on `data/markdown_corpus.txt` (`tools/gen_markdown_corpus.py`,
//! kinds `norm_006_01` and `norm_006_02`): each script's own `norm_text`,
//! run by Python 3.10 and 3.13 over edge strings and every line of the
//! design documents, equals [`norm_text`].

use super::pycompat::{PyStr, nfkc, splitlines};
use super::text::rx;

/// The scripts' `norm_text(s)`: NFKC; the no-break and thin spaces
/// (U+00A0, U+2007, U+202F, U+2009, U+200A) become spaces; U+200B and
/// U+FEFF are dropped; the dashes U+2010–U+2015 and U+2212 become `-`;
/// typographic quotes become `'` and `"`; runs of Python whitespace become
/// one space; the result is stripped.
pub fn norm_text(s: &str) -> String {
    let s: String = nfkc(s)
        .chars()
        .filter(|c| !matches!(c, '\u{200b}' | '\u{feff}'))
        .map(|c| match c {
            '\u{a0}' | '\u{2007}' | '\u{202f}' | '\u{2009}' | '\u{200a}' => ' ',
            '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
            '\u{2018}' | '\u{2019}' => '\'',
            '\u{201c}' | '\u{201d}' => '"',
            c => c,
        })
        .collect();
    rx(r"\s+")
        .replace_all_literal(&s, " ")
        .py_strip()
        .to_string()
}

/// 006-02's `norm_lines(text)`: [`norm_text`] of each line of
/// `text.splitlines()`.
pub fn norm_lines(text: &str) -> Vec<String> {
    splitlines(text).iter().map(|l| norm_text(l)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_text_folds_spaces_dashes_and_quotes() {
        assert_eq!(
            norm_text("  a\u{a0}\u{2009}b\t\u{2013} c\u{200b} "),
            "a b - c"
        );
        assert_eq!(
            norm_text("\u{201c}x\u{2019}\u{201d} 10\u{207b}\u{b3} \u{b5}s"),
            "\"x'\" 10-3 \u{3bc}s"
        );
        assert_eq!(norm_lines("a  b\n c \r\nd"), vec!["a b", "c", "d"]);
    }
}
