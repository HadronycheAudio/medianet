//! Text helpers with the semantics of the Python the checks were written in.
//!
//! The scripts being ported lean on a handful of Python string behaviours
//! (`str.splitlines`, `str.split()`, `re.I`, digit groups written with a
//! space). Each has one implementation here so every port reads text the same
//! way; the Python semantics themselves are in [`super::pycompat`].

use super::pycompat::PyStr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use regex::Regex;

use super::pycompat::{IntError, is_nd, parse_int, translate};

/// The most distinct patterns the cache keeps. Patterns are keyed by their
/// text; nearly all are literals written in the checks (a few hundred), a
/// few embed a number read from a document (`\b1280\b`). Past the bound a
/// pattern is compiled and returned without being cached, so a long harness
/// run over thousands of mutated numbers cannot grow the cache without
/// limit. Nothing is leaked: entries are `Arc`s.
pub const RX_CACHE_LIMIT: usize = 4096;

type Cache = Mutex<HashMap<String, Arc<Regex>>>;
static PY_CACHE: OnceLock<Cache> = OnceLock::new();
static RAW_CACHE: OnceLock<Cache> = OnceLock::new();

fn cached(pattern: &str, python: bool) -> Arc<Regex> {
    let cache =
        if python { &PY_CACHE } else { &RAW_CACHE }.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(found) = cache.lock().expect("regex cache").get(pattern) {
        return found.clone();
    }
    let source = if python {
        translate(pattern)
    } else {
        pattern.to_string()
    };
    let compiled =
        Arc::new(Regex::new(&source).unwrap_or_else(|e| panic!("bad pattern {pattern:?}: {e}")));
    let mut cache = cache.lock().expect("regex cache");
    if cache.len() < RX_CACHE_LIMIT {
        cache.insert(pattern.to_string(), compiled.clone());
    }
    compiled
}

/// A compiled regular expression with Python's semantics, cached by pattern
/// text (see [`RX_CACHE_LIMIT`]).
///
/// Patterns are written as in the scripts; prefix `(?i)` for `re.I`. The
/// classes `\w`, `\W`, `\s`, `\S` are Python's
/// ([`super::pycompat::translate`]). The compiled regex is shared, not
/// cloned: a clone of a [`Regex`] starts with an empty match cache. A bad
/// pattern is a programming error in a check and panics with the pattern; a
/// pattern that embeds document text must escape it ([`regex::escape`]).
pub fn rx(pattern: &str) -> Arc<Regex> {
    cached(pattern, true)
}

/// As [`rx`], with the `regex` crate's own semantics (Unicode properties
/// written by the library itself, never a script's pattern).
pub fn rx_raw(pattern: &str) -> Arc<Regex> {
    cached(pattern, false)
}

/// `re.search(pattern, text)` as a boolean.
pub fn has(pattern: &str, text: &str) -> bool {
    rx(pattern).is_match(text)
}

/// `re.search(pattern, text, re.I)` as a boolean.
pub fn has_i(pattern: &str, text: &str) -> bool {
    rx(&format!("(?i){pattern}")).is_match(text)
}

/// `re.match(pattern, text)`: a match anchored at the start of `text`.
pub fn starts(pattern: &str, text: &str) -> bool {
    rx(&format!("^(?:{pattern})")).is_match(text)
}

/// `re.fullmatch(pattern, text)`.
pub fn full(pattern: &str, text: &str) -> bool {
    rx(&format!("^(?:{pattern})$")).is_match(text)
}

pub use super::pycompat::splitlines;

/// Python's `str.split("\n")`: keeps a trailing empty line.
pub fn split_nl(text: &str) -> Vec<&str> {
    text.split('\n').collect()
}

/// Python's `str.split()` word count.
pub fn word_count(text: &str) -> usize {
    text.py_split().count()
}

/// Collapses whitespace runs to one space and trims (Python
/// `re.sub(r"\s+", " ", t).strip()`).
pub fn squash_ws(text: &str) -> String {
    rx(r"\s+").replace_all(text, " ").py_strip().to_string()
}

/// Removes every character of `chars` from `text`.
pub fn drop_chars(text: &str, chars: &str) -> String {
    text.chars().filter(|c| !chars.contains(*c)).collect()
}

/// Replaces every character of `chars` with a space.
pub fn blank_chars(text: &str, chars: &str) -> String {
    text.chars()
        .map(|c| if chars.contains(c) { ' ' } else { c })
        .collect()
}

/// Removes a digit-group separator (space, no-break space or narrow no-break
/// space) between a digit and exactly three digits: "1 252" reads "1252".
///
/// The Python is `re.sub(r"(?<=\d)[   ](?=\d{3}(?!\d))", "", t)`,
/// applied in one left-to-right pass; this reproduces it without lookaround.
pub fn join_digit_groups(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        let separator = matches!(c, ' ' | '\u{a0}' | '\u{202f}');
        if separator && i > 0 && is_nd(chars[i - 1]) && three_digits_follow(&chars, i + 1) {
            continue;
        }
        out.push(c);
    }
    out
}

fn three_digits_follow(chars: &[char], at: usize) -> bool {
    let digit = |k: usize| chars.get(k).is_some_and(|c| is_nd(*c));
    digit(at) && digit(at + 1) && digit(at + 2) && !digit(at + 3)
}

/// The first integer in `text` (`int(re.search(r"\d+", s).group(0))`),
/// checked: `Err` when it is beyond `i128` (Python would have kept it).
pub fn first_int(text: &str) -> Result<Option<i128>, IntError> {
    rx(r"\d+")
        .find(text)
        .map(|m| parse_int(m.as_str()))
        .transpose()
}

/// All integers in `text`, in order, checked as [`first_int`].
pub fn ints(text: &str) -> Result<Vec<i128>, IntError> {
    rx(r"\d+")
        .find_iter(text)
        .map(|m| parse_int(m.as_str()))
        .collect()
}

/// A prefix of at most `n` characters (Python `s[:n]`).
pub fn head(text: &str, n: usize) -> String {
    text.chars().take(n).collect()
}

/// Strips Markdown links to their text: `[text](target)` becomes `text`.
pub fn strip_links(text: &str) -> String {
    rx(r"\[([^\]]*)\]\([^)]*\)")
        .replace_all(text, "$1")
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitlines_matches_python() {
        assert_eq!(splitlines("a\nb\r\nc\rd\n"), vec!["a", "b", "c", "d"]);
        assert_eq!(splitlines("a\n\nb"), vec!["a", "", "b"]);
        assert_eq!(splitlines("a\u{2028}b"), vec!["a", "b"]);
        assert!(splitlines("").is_empty());
        assert_eq!(split_nl("a\n"), vec!["a", ""]);
    }

    #[test]
    fn digit_groups_join_only_before_three_digits() {
        assert_eq!(join_digit_groups("1 252 bytes"), "1252 bytes");
        assert_eq!(join_digit_groups("65 507"), "65507");
        assert_eq!(join_digit_groups("1\u{a0}280"), "1280");
        assert_eq!(join_digit_groups("rule 1 2345"), "rule 1 2345");
        assert_eq!(join_digit_groups("1 25"), "1 25");
        assert_eq!(join_digit_groups("a 123"), "a 123");
        assert_eq!(join_digit_groups("² 123"), "² 123", "² is not a Python \\d");
    }

    #[test]
    fn regex_helpers_follow_python_anchoring() {
        assert!(has_i("Owner", "the owner is"));
        assert!(!has("Owner", "the owner is"));
        assert!(starts(r"\d+", "12 ab"));
        assert!(!starts(r"\d+", "ab 12"));
        assert!(full(r"\d+", "12"));
        assert!(!full(r"\d+", "12a"));
        assert_eq!(first_int("C-17 at most 1252"), Ok(Some(17)));
        assert_eq!(first_int("none"), Ok(None));
        assert!(first_int(&"9".repeat(50)).is_err());
        assert_eq!(ints("32 to 35"), Ok(vec![32, 35]));
    }

    #[test]
    fn small_helpers() {
        assert_eq!(squash_ws("  a \n b  "), "a b");
        assert_eq!(drop_chars("`a`*b", "`*"), "ab");
        assert_eq!(blank_chars("a|b", "|"), "a b");
        assert_eq!(word_count(" one two\tthree "), 3);
        assert_eq!(head("abcdef", 3), "abc");
        assert_eq!(strip_links("see [Hello](#hello) now"), "see Hello now");
    }
}
