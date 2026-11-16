//! The text helpers of `checks/006-01.sh`, copied by `checks/006-02.sh`:
//! `norm_text` (NFKC and typographic normalisation), the character-walking
//! table row splitter, and the table separator test.
//!
//! Python's `unicodedata.normalize("NFKC", s)` has no counterpart in the
//! standard library and the crate has no normalisation dependency, so
//! [`nfkc`] maps the compatibility characters a design document can carry
//! (the superscripts and subscripts, vulgar fractions, ligatures, full-width
//! forms, the micro sign, the ellipsis, the wide spaces and a few symbols).
//! A character it does not know passes through unchanged, which can only make
//! a comparison stricter than the script's (never weaker): two strings the
//! script equates only through a mapping missing here stay different. A test
//! checks that every non-ASCII character of the design documents is covered.

use crate::doccheck::text::rx;

/// The compatibility decomposition of one character, `None` when NFKC
/// leaves it alone (or when it is not known here).
fn compat(c: char) -> Option<&'static str> {
    Some(match c {
        '\u{b5}' => "\u{3bc}",
        '\u{2026}' => "...",
        '\u{2024}' => ".",
        '\u{2025}' => "..",
        '\u{2153}' => "1\u{2044}3",
        '\u{2154}' => "2\u{2044}3",
        '\u{bc}' => "1\u{2044}4",
        '\u{bd}' => "1\u{2044}2",
        '\u{be}' => "3\u{2044}4",
        '\u{2155}' => "1\u{2044}5",
        '\u{2156}' => "2\u{2044}5",
        '\u{2157}' => "3\u{2044}5",
        '\u{2158}' => "4\u{2044}5",
        '\u{2159}' => "1\u{2044}6",
        '\u{215a}' => "5\u{2044}6",
        '\u{215b}' => "1\u{2044}8",
        '\u{215c}' => "3\u{2044}8",
        '\u{215d}' => "5\u{2044}8",
        '\u{215e}' => "7\u{2044}8",
        '\u{b9}' => "1",
        '\u{b2}' => "2",
        '\u{b3}' => "3",
        '\u{2070}' => "0",
        '\u{2074}'..='\u{2079}' => return None,
        '\u{fb00}' => "ff",
        '\u{fb01}' => "fi",
        '\u{fb02}' => "fl",
        '\u{fb03}' => "ffi",
        '\u{fb04}' => "ffl",
        '\u{2122}' => "TM",
        '\u{2116}' => "No",
        '\u{2103}' => "\u{b0}C",
        '\u{2109}' => "\u{b0}F",
        '\u{aa}' => "a",
        '\u{ba}' => "o",
        '\u{212b}' => "\u{c5}",
        '\u{2126}' => "\u{3a9}",
        '\u{212a}' => "K",
        '\u{a0}' | '\u{2002}'..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}' => " ",
        _ => return None,
    })
}

/// NFKC for the characters of [`compat`], the super- and subscript digits and
/// the full-width ASCII block.
pub(crate) fn nfkc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\u{2074}'..='\u{2079}' => out.push(char::from(b'0' + (c as u32 - 0x2070) as u8)),
            '\u{2080}'..='\u{2089}' => out.push(char::from(b'0' + (c as u32 - 0x2080) as u8)),
            '\u{ff01}'..='\u{ff5e}' => out.push(char::from_u32(c as u32 - 0xfee0).unwrap_or(c)),
            _ => match compat(c) {
                Some(m) => out.push_str(m),
                None => out.push(c),
            },
        }
    }
    out
}

/// `norm_text`.
pub(crate) fn norm_text(s: &str) -> String {
    let mut t = nfkc(s);
    for ch in ['\u{a0}', '\u{2007}', '\u{202f}', '\u{2009}', '\u{200a}'] {
        t = t.replace(ch, " ");
    }
    t = t.replace(['\u{200b}', '\u{feff}'], "");
    let t = rx(r"[\u{2010}-\u{2015}\u{2212}]").replace_all(&t, "-").into_owned();
    let t = t
        .replace(['\u{2018}', '\u{2019}'], "'")
        .replace(['\u{201c}', '\u{201d}'], "\"");
    rx(r"\s+").replace_all(&t, " ").trim().to_string()
}

/// `split_row`: a row split on unescaped pipes by walking its characters;
/// `\|` is a pipe inside the cell, a final `|` after a backslash is content.
/// `None` when the line is not a table row.
pub(crate) fn split_row(line: &str) -> Option<Vec<String>> {
    let s = line.trim();
    if !s.starts_with('|') {
        return None;
    }
    let s = if s.ends_with('|') && !s.ends_with("\\|") { &s[..s.len() - 1] } else { s };
    let chars: Vec<char> = s.chars().skip(1).collect();
    let (mut cells, mut cur, mut i) = (Vec::new(), String::new(), 0);
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && chars.get(i + 1) == Some(&'|') {
            cur.push('|');
            i += 2;
            continue;
        }
        if c == '|' {
            cells.push(std::mem::take(&mut cur).trim().to_string());
            i += 1;
            continue;
        }
        cur.push(c);
        i += 1;
    }
    cells.push(cur.trim().to_string());
    Some(cells)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doccheck::Docs;

    #[test]
    fn normalisation_matches_the_script() {
        assert_eq!(norm_text("a\u{a0}\u{2212}\u{2019}b\u{200b}  c"), "a -'b c");
        assert_eq!(norm_text("\u{b5}s \u{2153} \u{2026}"), "\u{3bc}s 1\u{2044}3 ...");
        assert_eq!(norm_text("\u{ff21}\u{ff11}"), "A1");
    }

    #[test]
    fn rows_split_on_unescaped_pipes() {
        assert_eq!(split_row("| a | b \\| c |"), Some(vec!["a".into(), "b | c".into()]));
        assert_eq!(split_row("| a \\|"), Some(vec!["a |".into()]));
        assert_eq!(split_row("|"), Some(vec![String::new()]));
        assert_eq!(split_row("no"), None);
    }

    /// Every non-ASCII character of the documents these checks read is one
    /// that NFKC leaves alone or that [`nfkc`] maps (the list comes from
    /// Python's `unicodedata`).
    #[test]
    fn the_design_documents_need_no_other_compatibility_mapping() {
        let invariant = "\u{2014}\u{2013}\u{2212}\u{2261}\u{b7}\u{d7}\u{b1}\u{2192}\u{2190}\u{2248}\u{2264}\u{2265}\u{230a}\u{230b}\u{2308}\u{2309}\u{b0}\u{a7}\u{2019}\u{2018}\u{201c}\u{201d}\u{3bc}\u{2260}\u{221e}\u{3c0}\u{394}\u{3c4}\u{3b1}\u{3b2}\u{2022}\u{2713}";
        let docs = Docs::repo();
        for path in ["doc/design/spec/timeline.md", "doc/design/02-constants.md"] {
            let text = docs.read(path).expect("document");
            for c in text.chars().filter(|c| !c.is_ascii()) {
                assert!(
                    invariant.contains(c) || compat(c).is_some() || ('\u{2074}'..='\u{2079}').contains(&c),
                    "{path}: U+{:04X} {c} is neither invariant under NFKC nor mapped by nfkc()",
                    c as u32
                );
            }
        }
    }
}
