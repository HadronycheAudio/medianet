//! Python semantics for the WP7 ports (004-16 to 004-20), private for now.
//!
//! The scripts are Python 3 over `str`: `re` patterns are Unicode-aware and
//! differ from Rust's `regex` in four places this module closes, and a few
//! `str` methods differ from Rust's:
//!
//! * `\w` (and so `\b`, `\W`): Python's word characters are the letters and
//!   numbers (`str.isalnum`, general categories L and N) and `_`; Rust's are
//!   Alphabetic, the marks, Nd, Pc and the joiners. `²` is a word character
//!   for Python only, a combining accent for Rust only.
//! * `\s`: Python's whitespace (`str.isspace`) also has U+001C to U+001F.
//! * `re.I`: Python matches `i` with `ı` (U+0131) and `İ` (U+0130); Rust's
//!   case folding does not.
//! * look-behind: Rust has none; the ports that need one emulate it at the
//!   call site.
//!
//! The first two are closed by matching on a *shadow* of the text: every
//! character whose class differs is replaced by one of the same UTF-8 length
//! whose Rust class is the Python one, so byte offsets agree and matches are
//! sliced from the original text. The third is closed by rewriting a
//! case-insensitive pattern's `i`. Everything here should move to the
//! library's `text` module (see the WP7 report).

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::doccheck::rejection_members::digit_value;
use crate::doccheck::text::rx;

/// Python's `open()` text mode: `\r\n` and a lone `\r` read as `\n`.
pub(crate) fn universal(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// `str.splitlines()`.
pub(crate) fn splitlines(text: &str) -> Vec<&str> {
    crate::doccheck::text::splitlines(text)
}

/// `str.isspace()` for one character.
pub(crate) fn is_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// `str.strip()`.
pub(crate) fn strip(s: &str) -> &str {
    s.trim_matches(is_space)
}

/// `str.lstrip()`.
pub(crate) fn lstrip(s: &str) -> &str {
    s.trim_start_matches(is_space)
}

/// `int(s)` for a string of decimal digits (any Unicode Nd) with optional
/// surrounding whitespace and sign. Python's integers are unbounded; this
/// saturates at [`HUGE`] beyond 36 digits (no document number comes near).
pub(crate) fn int(s: &str) -> Option<i128> {
    let t = strip(s);
    let (neg, digits) = match t.strip_prefix('-') {
        Some(d) => (true, d),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    if digits.is_empty() {
        return None;
    }
    let mut v: i128 = 0;
    for c in digits.chars() {
        let d = digit_value(c)?;
        v = v.saturating_mul(10).saturating_add(i128::from(d)).min(HUGE);
    }
    Some(if neg { -v } else { v })
}

/// The saturation value of [`int`]: far above any number a document states,
/// low enough that sums and products of a few never overflow.
pub(crate) const HUGE: i128 = 10_i128.pow(36);

/// Python `\w` for one character.
pub(crate) fn py_word(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric() || (!c.is_ascii() && rx(r"^[\p{L}\p{N}]$").is_match(c.encode_utf8(&mut [0; 4])))
}

fn rust_word(c: char) -> bool {
    rx(r"^\w$").is_match(c.encode_utf8(&mut [0; 4]))
}

thread_local! {
    static SUBST: RefCell<HashMap<char, Option<char>>> = RefCell::new(HashMap::new());
}

/// The shadow character of `c`, if its Rust class differs from its Python
/// class.
fn subst(c: char) -> Option<char> {
    if c.is_ascii() {
        return ('\u{1c}'..='\u{1f}').contains(&c).then_some('\u{0b}');
    }
    SUBST.with(|m| {
        *m.borrow_mut().entry(c).or_insert_with(|| {
            let (py, ru) = (py_word(c), rust_word(c));
            let len = c.len_utf8();
            match (py, ru) {
                // Word for Python only: a letter of the same length.
                (true, false) => Some(['\u{1c2}', '\u{a000}', '\u{20000}'][len - 2]),
                // Word for Rust only: a symbol of the same length.
                (false, true) => Some(['\u{a6}', '\u{2016}', '\u{1d100}'][len - 2]),
                _ => None,
            }
        })
    })
}

/// The text on which a Python pattern is matched by Rust's engine.
pub(crate) fn shadow(text: &str) -> Cow<'_, str> {
    if !text.chars().any(|c| subst(c).is_some()) {
        return Cow::Borrowed(text);
    }
    Cow::Owned(text.chars().map(|c| subst(c).unwrap_or(c)).collect())
}

/// A case-insensitive pattern with `i` widened to Python's `[iIıİ]`.
fn fold_i(pattern: &str) -> String {
    let mut out = String::from("(?i)");
    let mut chars = pattern.chars().peekable();
    let mut class: Option<String> = None;
    while let Some(c) = chars.next() {
        if c == '\\' {
            let n = chars.next().unwrap_or('\\');
            let piece = format!("\\{n}");
            match class.as_mut() {
                Some(k) => k.push_str(&piece),
                None => out.push_str(&piece),
            }
            if matches!(n, 'p' | 'P') && chars.peek() == Some(&'{') {
                for d in chars.by_ref() {
                    match class.as_mut() {
                        Some(k) => k.push(d),
                        None => out.push(d),
                    }
                    if d == '}' {
                        break;
                    }
                }
            }
            continue;
        }
        match (&mut class, c) {
            (None, '[') => {
                let mut k = String::from("[");
                if chars.peek() == Some(&'^') {
                    k.push(chars.next().unwrap_or('^'));
                }
                if chars.peek() == Some(&']') {
                    k.push(chars.next().unwrap_or(']'));
                }
                class = Some(k);
            }
            (Some(k), ']') => {
                let body = k.to_lowercase();
                let spans_i = body.contains('i') || rx(r"[a-h]-[j-z]").is_match(&body);
                if spans_i {
                    k.push_str("\u{131}\u{130}");
                }
                k.push(']');
                out.push_str(k);
                class = None;
            }
            (Some(k), _) => k.push(c),
            (None, 'i' | 'I') => out.push_str("[iI\u{131}\u{130}]"),
            (None, _) => out.push(c),
        }
    }
    out
}

fn compile(pattern: &str, icase: bool) -> &'static regex::Regex {
    if icase { rx(&fold_i(pattern)) } else { rx(pattern) }
}

/// One match: byte span in the original text and its groups.
#[derive(Debug, Clone)]
pub(crate) struct Hit<'t> {
    pub start: usize,
    pub end: usize,
    pub groups: Vec<Option<&'t str>>,
    text: &'t str,
}

impl<'t> Hit<'t> {
    /// `m.group(0)`.
    pub fn all(&self) -> &'t str {
        &self.text[self.start..self.end]
    }

    /// `m.group(k)` (k >= 1), `""` when the group did not take part.
    pub fn g(&self, k: usize) -> &'t str {
        self.groups.get(k).copied().flatten().unwrap_or("")
    }

    /// `m.group(k)` as an option.
    pub fn opt(&self, k: usize) -> Option<&'t str> {
        self.groups.get(k).copied().flatten()
    }
}

/// Every match of a Python pattern in `text` (`re.finditer`); `icase` is
/// `re.I`.
pub(crate) fn finditer<'t>(pattern: &str, text: &'t str, icase: bool) -> Vec<Hit<'t>> {
    let re = compile(pattern, icase);
    let sh = shadow(text);
    re.captures_iter(&sh)
        .map(|c| {
            let m = c.get(0).expect("group 0");
            Hit {
                start: m.start(),
                end: m.end(),
                groups: (0..c.len()).map(|k| c.get(k).map(|g| &text[g.start()..g.end()])).collect(),
                text,
            }
        })
        .collect()
}

/// `re.search(pattern, text[, re.I])`.
pub(crate) fn search<'t>(pattern: &str, text: &'t str, icase: bool) -> Option<Hit<'t>> {
    let re = compile(pattern, icase);
    let sh = shadow(text);
    re.captures(&sh).map(|c| {
        let m = c.get(0).expect("group 0");
        Hit {
            start: m.start(),
            end: m.end(),
            groups: (0..c.len()).map(|k| c.get(k).map(|g| &text[g.start()..g.end()])).collect(),
            text,
        }
    })
}

/// `bool(re.search(pattern, text))`, case-sensitive.
pub(crate) fn has(pattern: &str, text: &str) -> bool {
    compile(pattern, false).is_match(&shadow(text))
}

/// `bool(re.search(pattern, text, re.I))`.
pub(crate) fn has_i(pattern: &str, text: &str) -> bool {
    compile(pattern, true).is_match(&shadow(text))
}

/// `re.match(pattern, text[, re.I])`: anchored at the start.
pub(crate) fn matches<'t>(pattern: &str, text: &'t str, icase: bool) -> Option<Hit<'t>> {
    search(&format!("^(?:{pattern})"), text, icase)
}

/// `re.fullmatch(pattern, text[, re.I])`.
pub(crate) fn fullmatch<'t>(pattern: &str, text: &'t str, icase: bool) -> Option<Hit<'t>> {
    search(&format!("^(?:{pattern})$"), text, icase)
}

/// `re.sub(pattern, repl, text[, re.I])` with a literal replacement.
pub(crate) fn sub(pattern: &str, repl: &str, text: &str, icase: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for h in finditer(pattern, text, icase) {
        out.push_str(&text[last..h.start]);
        out.push_str(repl);
        last = h.end;
    }
    out.push_str(&text[last..]);
    out
}

/// `len(s)`: characters, not bytes.
pub(crate) fn len(s: &str) -> usize {
    s.chars().count()
}

/// `s[a:b]` by character index (clamped, as Python slices are).
pub(crate) fn slice(s: &str, a: usize, b: usize) -> &str {
    let at = |k: usize| s.char_indices().nth(k).map_or(s.len(), |(i, _)| i);
    let (a, b) = (at(a), at(b));
    if a >= b { "" } else { &s[a..b] }
}

/// The character index of byte offset `byte` in `s`.
pub(crate) fn char_index(s: &str, byte: usize) -> usize {
    s[..byte].chars().count()
}

/// `s[:n]`.
pub(crate) fn head(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_boundaries_follow_python() {
        // `²` is a word character for Python: no boundary after "Magic".
        assert!(!has(r"\bBadMagic\b", "BadMagic\u{b2}"));
        // A combining accent is not: a boundary after "Magic".
        assert!(has(r"\bBadMagic\b", "BadMagic\u{301}"));
        assert!(has(r"\w", "\u{b2}"));
        assert!(!has(r"\w", "\u{301}"));
        assert!(has(r"a\sb", "a\u{1f}b"));
        let h = search(r"(\w+)", "x \u{b2}\u{b3}y", false).expect("hit");
        assert_eq!(h.g(1), "x");
        let all: Vec<&str> = finditer(r"\w+", "\u{e9}t\u{e9} \u{b2}", false).iter().map(Hit::all).collect();
        assert_eq!(all, ["\u{e9}t\u{e9}", "\u{b2}"]);
    }

    #[test]
    fn case_folding_follows_python() {
        assert!(has_i("fits", "f\u{131}ts"));
        assert!(has_i("FITS", "F\u{130}TS"));
        assert!(has_i(r"[a-z]+s", "\u{131}s"));
        assert!(!has_i(r"[^a-z]", "\u{131}"));
        assert!(!has("fits", "f\u{131}ts"));
        assert!(has_i(r"\p{Lu}", "A"));
    }

    #[test]
    fn ints_and_slices() {
        assert_eq!(int(" 12 "), Some(12));
        assert_eq!(int("\u{663}\u{664}"), Some(34));
        assert_eq!(int("1x"), None);
        assert_eq!(int("123456789012345678901234567890"), Some(123_456_789_012_345_678_901_234_567_890));
        assert_eq!(int(&"9".repeat(60)), Some(HUGE));
        assert_eq!(slice("h\u{e9}llo", 1, 3), "\u{e9}l");
        assert_eq!(slice("abc", 2, 10), "c");
        assert_eq!(strip("\u{1f} a \u{a0}"), "a");
        assert_eq!(universal("a\r\nb\rc"), "a\nb\nc");
        assert_eq!(sub(r"`[^`]*`", "", "a `b` c", false), "a  c");
    }
}
