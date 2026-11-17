//! Python 3.10 semantics the WP5 scripts (004-06, 004-08, 004-09, 004-10)
//! rely on, reproduced for their ports. Candidates for the library.
//!
//! * **Reading**: `open(path, encoding="utf-8").read()` translates `\r\n`
//!   and a bare `\r` to `\n` (universal newlines): [`read_universal`].
//! * **Lines**: `str.splitlines()` (the library's
//!   [`splitlines`](crate::doccheck::text::splitlines)).
//! * **Character classes**: Python `\w` is letters, numbers (`Nd`, `Nl`,
//!   `No`: `²` is a word character) and `_`, at Unicode 13; Rust's `\w` adds
//!   marks and every connector punctuation and drops `No`. Python `\s` and
//!   `str.strip()` include `\x1c`–`\x1f`; Rust's do not. The library's `rx`
//!   now translates `\w` and `\s`, but `\b` keeps Rust's word definition
//!   (`pycompat` "known limit"), and `trim` keeps Rust's spaces. [`shadow`]
//!   maps each character on which Python and Rust disagree to a stand-in
//!   Rust classes the way Python classed the original, so `\b`, `\w`, `\s`
//!   and `trim` all behave as in the script, translated or not. The
//!   documents are shadowed once, when read. (The library's `pycompat::is_word`
//!   covers the current Unicode; the table here is Python 3.10's Unicode 13.)
//! * **`re.I`**: Python folds `İ` (U+0130) and `ı` (U+0131) onto `i`; Rust's
//!   simple case folding does not. [`fold_i`] maps them, one character for
//!   one, before a case-insensitive search ([`si`], [`find_i`]).
//! * **`int()`**: unbounded ([`Big`]) and reading any Unicode decimal digit
//!   (`int("١٢")` is 12): [`py_int`], [`py_int_base`]. **`float()`**:
//!   [`py_float`]; `repr(float)`: [`float_repr`].

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use super::big::Big;
use super::pyuni::{PY_DIGIT, PY_WORD};
use crate::doccheck::Docs;
use crate::doccheck::text::rx;

/// Stands for a character Python counts as a word character and Rust does
/// not (`²`): a caseless letter (VAI SYLLABLE NDOLE FA).
const WORD_STANDIN: char = '\u{A562}';
/// Stands for a character Rust counts as a word character and Python does
/// not (a combining mark, `‿`): a currency sign.
const NONWORD_STANDIN: char = '\u{A4}';
/// Stands for `\x1f`, a Python space (and not a line break): EN QUAD.
const SPACE_STANDIN: char = '\u{2000}';

fn in_ranges(c: char, ranges: &[(u32, u32)]) -> bool {
    let c = c as u32;
    ranges
        .binary_search_by(|&(lo, hi)| {
            if hi < c {
                std::cmp::Ordering::Less
            } else if lo > c {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// Python `\w` (`str.isalnum() or c == "_"`).
pub(crate) fn py_word(c: char) -> bool {
    in_ranges(c, PY_WORD)
}

/// Python `\d`.
pub(crate) fn py_digit(c: char) -> bool {
    in_ranges(c, PY_DIGIT)
}

/// The value of a Python decimal digit (`int()` reads every `Nd` digit).
pub(crate) fn digit_value(c: char) -> Option<u32> {
    let cu = c as u32;
    PY_DIGIT
        .iter()
        .find(|&&(lo, hi)| lo <= cu && cu <= hi)
        .map(|&(lo, _)| (cu - lo) % 10)
}

/// Python `str.isspace()` (and `\s`).
pub(crate) fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python `str.strip()`.
pub(crate) fn py_strip(s: &str) -> &str {
    s.trim_matches(py_space)
}

fn shadow_char(c: char) -> char {
    if c.is_ascii() {
        return if c == '\u{1f}' { SPACE_STANDIN } else { c };
    }
    static CACHE: OnceLock<Mutex<HashMap<char, char>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(&m) = cache.lock().expect("shadow cache").get(&c) {
        return m;
    }
    let mut buf = [0u8; 4];
    let rust_word = crate::doccheck::text::rx_raw(r"\A\w\z").is_match(c.encode_utf8(&mut buf));
    let m = match (py_word(c), rust_word) {
        (true, false) => WORD_STANDIN,
        (false, true) => NONWORD_STANDIN,
        _ => c,
    };
    cache.lock().expect("shadow cache").insert(c, m);
    m
}

/// The text with every character Python and Rust class differently replaced
/// by a stand-in Rust classes as Python classed the original (one character
/// for one: character offsets are kept).
pub(crate) fn shadow(text: &str) -> String {
    text.chars().map(shadow_char).collect()
}

/// `open(...).read()` with universal newlines.
pub(crate) fn universal(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// The file as the script read it (universal newlines), or `None`.
pub(crate) fn read_universal(docs: &Docs, path: &str) -> Option<String> {
    docs.read(path).map(|t| universal(&t))
}

/// `İ` and `ı` folded onto `i`, as Python's `re.I` matches them.
pub(crate) fn fold_i(text: &str) -> Cow<'_, str> {
    if text.contains(['\u{130}', '\u{131}']) {
        Cow::Owned(text.chars().map(|c| if c == '\u{130}' || c == '\u{131}' { 'i' } else { c }).collect())
    } else {
        Cow::Borrowed(text)
    }
}

/// `re.search(pattern, text)` (case-sensitive) on shadowed text.
pub(crate) fn s(pattern: &str, text: &str) -> bool {
    rx(pattern).is_match(text)
}

/// `re.search(pattern, text, re.I)` on shadowed text.
pub(crate) fn si(pattern: &str, text: &str) -> bool {
    rx(&format!("(?i){pattern}")).is_match(&fold_i(text))
}

/// The first `re.I` match of `pattern` in `text` as character offsets
/// (start, end), with its groups.
pub(crate) fn find_i(pattern: &str, text: &str) -> Option<(usize, usize, Vec<Option<String>>)> {
    let folded = fold_i(text);
    let c = rx(&format!("(?i){pattern}")).captures(&folded)?;
    let m = c.get(0)?;
    let start = folded[..m.start()].chars().count();
    let end = start + m.as_str().chars().count();
    let groups = c.iter().skip(1).map(|g| g.map(|g| g.as_str().to_string())).collect();
    Some((start, end, groups))
}

/// `text[a:b]` in characters, clamped as Python slices are.
pub(crate) fn char_slice(text: &str, a: usize, b: usize) -> String {
    text.chars().skip(a).take(b.saturating_sub(a)).collect()
}

/// `text[:n]` in characters.
pub(crate) fn head(text: &str, n: usize) -> String {
    text.chars().take(n).collect()
}

/// `float(n)` of a Python int: `None` where Python raised OverflowError
/// ("int too large to convert to float").
pub(crate) fn big_to_f64(n: &Big) -> Option<f64> {
    let v: f64 = n.to_string().parse().ok()?;
    v.is_finite().then_some(v)
}

/// `re.search("(?<![...])" + pattern, text, re.I)`: a match of `pattern`
/// whose preceding character (in the searched string) is not one `bad_prev`
/// accepts. Rust's regex has no look-behind; every match start is tried in
/// turn, as Python's search does.
pub(crate) fn search_i_not_after(pattern: &str, text: &str, bad_prev: fn(char) -> bool) -> bool {
    let folded = fold_i(text);
    let re = rx(&format!("(?i){pattern}"));
    let mut pos = 0;
    while pos <= folded.len() {
        let Some(m) = re.find_at(&folded, pos) else { return false };
        if !text_prev(&folded, m.start()).is_some_and(bad_prev) {
            return true;
        }
        pos = m.start() + folded[m.start()..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

fn text_prev(text: &str, at: usize) -> Option<char> {
    text[..at].chars().next_back()
}

/// The class `[a-zA-Zµ]` under `re.I`: ASCII letters, `µ`, `μ`, `Μ`, and
/// the characters Python folds onto ASCII letters (`İ`, `ı`, `ſ`, `K`).
pub(crate) fn letter_or_micro_i(c: char) -> bool {
    c.is_ascii_alphabetic() || matches!(c, '\u{b5}' | '\u{3bc}' | '\u{39c}' | '\u{130}' | '\u{131}' | '\u{17f}' | '\u{212a}')
}

/// Python `int(s)` (base 10): surrounding spaces, a sign, Unicode decimal
/// digits, single underscores between digits.
pub(crate) fn py_int(text: &str) -> Option<Big> {
    py_int_base(text, 10)
}

/// Python `int(s, base)` for base 2, 8, 10 or 16 (prefix allowed when it
/// names the base).
pub(crate) fn py_int_base(text: &str, base: u32) -> Option<Big> {
    let t = py_strip(text);
    let (neg, t) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let prefix = match base {
        16 => Some("0x"),
        8 => Some("0o"),
        2 => Some("0b"),
        _ => None,
    };
    let mut body = t;
    let mut after_prefix = false;
    if let Some(p) = prefix
        && body.len() >= 2
        && body.is_char_boundary(2)
        && body[..2].eq_ignore_ascii_case(p)
    {
        body = &body[2..];
        after_prefix = true;
    }
    let mut values = Vec::new();
    let chars: Vec<char> = body.chars().collect();
    for (k, &c) in chars.iter().enumerate() {
        if c == '_' {
            let ok_before = k > 0 && chars[k - 1] != '_' || k == 0 && after_prefix;
            if !ok_before || k + 1 >= chars.len() || chars[k + 1] == '_' {
                return None;
            }
            continue;
        }
        let v = digit_value(c).or_else(|| c.to_digit(16).filter(|_| c.is_ascii_alphabetic()))?;
        if v >= base {
            return None;
        }
        values.push(v);
    }
    if values.is_empty() {
        return None;
    }
    let v = Big::from_radix(&values, base);
    Some(if neg { v.neg() } else { v })
}

/// Python `float(s)`.
pub(crate) fn py_float(text: &str) -> Option<f64> {
    let t: String = py_strip(text)
        .chars()
        .map(|c| match digit_value(c) {
            Some(v) if !c.is_ascii_digit() => char::from_digit(v, 10).unwrap_or(c),
            _ => c,
        })
        .collect();
    let lower = t.to_ascii_lowercase();
    let (neg, body) = match lower.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, lower.strip_prefix('+').unwrap_or(&lower)),
    };
    let v = match body {
        "inf" | "infinity" => f64::INFINITY,
        "nan" => f64::NAN,
        _ => {
            let grammar = r"\A(?:[0-9](?:_?[0-9])*(?:\.(?:[0-9](?:_?[0-9])*)?)?|\.[0-9](?:_?[0-9])*)(?:e[+-]?[0-9](?:_?[0-9])*)?\z";
            if !rx(grammar).is_match(body) {
                return None;
            }
            body.replace('_', "").parse::<f64>().ok()?
        }
    };
    Some(if neg { -v } else { v })
}

/// Python `repr(float)`.
pub(crate) fn float_repr(v: f64) -> String {
    if v.is_nan() {
        return "nan".into();
    }
    if v.is_infinite() {
        return if v > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let sci = format!("{v:e}");
    let (mant, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let (sign, mant) = mant.strip_prefix('-').map_or(("", mant), |m| ("-", m));
    let digits: String = mant.chars().filter(char::is_ascii_digit).collect();
    if (-4..16).contains(&exp) {
        let n = digits.len() as i32;
        let s = if exp >= n - 1 {
            format!("{digits}{}.0", "0".repeat((exp - n + 1) as usize))
        } else if exp >= 0 {
            format!("{}.{}", &digits[..(exp + 1) as usize], &digits[(exp + 1) as usize..])
        } else {
            format!("0.{}{digits}", "0".repeat((-exp - 1) as usize))
        };
        return format!("{sign}{s}");
    }
    let m = if digits.len() > 1 { format!("{}.{}", &digits[..1], &digits[1..]) } else { digits };
    let es = if exp < 0 { "-" } else { "+" };
    format!("{sign}{m}e{es}{:02}", exp.abs())
}

/// Python `repr(str)`.
pub(crate) fn str_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if !printable(c) => {
                let u = c as u32;
                if u < 0x100 {
                    out.push_str(&format!("\\x{u:02x}"));
                } else if u < 0x10000 {
                    out.push_str(&format!("\\u{u:04x}"));
                } else {
                    out.push_str(&format!("\\U{u:08x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// Python `str.isprintable()` for one character (not `C*`, not `Z*` except
/// the space).
fn printable(c: char) -> bool {
    if c == ' ' {
        return true;
    }
    let mut buf = [0u8; 4];
    !rx(r"\A[\p{C}\p{Z}]\z").is_match(c.encode_utf8(&mut buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_and_numbers_as_python() {
        assert!(py_word('²') && !py_word('\u{301}') && py_word('_') && !py_word('‿'));
        assert_eq!(shadow("a²\u{301}\u{1f}b").chars().count(), 5);
        assert!(s(r"\bTooShort\b", &shadow("TooShort\u{301}")));
        assert!(!s(r"\bTooShort\b", &shadow("TooShort²")));
        assert!(s(r"\s", &shadow("\u{1f}")));
        assert!(si(r"\bmagic\b", "MAGıC"));
        assert_eq!(py_int("١٢").unwrap(), Big::from(12));
        assert_eq!(py_int(" -1_000 ").unwrap(), Big::from(-1000));
        assert!(py_int("1__0").is_none() && py_int("_1").is_none() && py_int("²").is_none());
        assert_eq!(py_int_base("0x1F", 16).unwrap(), Big::from(31));
        assert_eq!(py_int_base("0x_1F", 16).unwrap(), Big::from(31));
        assert!(py_int_base("0x", 16).is_none());
        assert_eq!(py_float("1e3"), Some(1000.0));
        assert_eq!(py_float(" -Infinity "), Some(f64::NEG_INFINITY));
        assert!(py_float("1e").is_none() && py_float(".").is_none() && py_float("0x10").is_none());
        assert_eq!(float_repr(46.0), "46.0");
        assert_eq!(float_repr(1e16), "1e+16");
        assert_eq!(float_repr(0.0001), "0.0001");
        assert_eq!(float_repr(0.00001), "1e-05");
        assert_eq!(float_repr(123.456), "123.456");
        assert_eq!(str_repr("it's\n"), "\"it's\\n\"");
        assert_eq!(str_repr("a\u{1f}"), "'a\\x1f'");
    }
}
