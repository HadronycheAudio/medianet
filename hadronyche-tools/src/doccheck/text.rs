//! Text helpers with the semantics of the Python the checks were written in.
//!
//! The scripts being ported lean on a handful of Python string behaviours
//! (`str.splitlines`, `str.split()`, `re.I`, digit groups written with a
//! space). Each has one implementation here so every port reads text the same
//! way.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use regex::Regex;

static CACHE: OnceLock<Mutex<HashMap<String, &'static Regex>>> = OnceLock::new();

/// A compiled regular expression, cached by pattern text for the process.
///
/// Patterns are written as in the scripts; prefix `(?i)` for `re.I`. A bad
/// pattern is a programming error in a check and panics with the pattern.
/// The compiled regex is shared, not cloned: a clone of a [`Regex`] starts
/// with an empty match cache, which made every call rebuild its automaton.
/// The cache holds one leaked regex per distinct pattern text, a set bounded
/// by the patterns written in the checks, and is shared by test threads.
pub fn rx(pattern: &str) -> &'static Regex {
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(found) = cache.lock().expect("regex cache").get(pattern) {
        return found;
    }
    let compiled: &'static Regex = Box::leak(Box::new(
        Regex::new(pattern).unwrap_or_else(|e| panic!("bad pattern {pattern:?}: {e}")),
    ));
    cache
        .lock()
        .expect("regex cache")
        .entry(pattern.to_string())
        .or_insert(compiled)
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

/// Python's `str.splitlines()`: splits on every line boundary Python knows
/// and drops the terminators; a trailing terminator yields no empty line.
pub fn splitlines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let is_break = matches!(
            c,
            '\n' | '\r'
                | '\u{0b}'
                | '\u{0c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if !is_break {
            continue;
        }
        out.push(&text[start..i]);
        let mut next = i + c.len_utf8();
        if c == '\r' && matches!(chars.peek(), Some((_, '\n'))) {
            chars.next();
            next += 1;
        }
        start = next;
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

/// Python's `str.split("\n")`: keeps a trailing empty line.
pub fn split_nl(text: &str) -> Vec<&str> {
    text.split('\n').collect()
}

/// Python's `str.split()` word count.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Collapses whitespace runs to one space and trims (Python
/// `re.sub(r"\s+", " ", t).strip()`).
pub fn squash_ws(text: &str) -> String {
    rx(r"\s+").replace_all(text, " ").trim().to_string()
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
        if separator && i > 0 && chars[i - 1].is_numeric() && three_digits_follow(&chars, i + 1) {
            continue;
        }
        out.push(c);
    }
    out
}

fn three_digits_follow(chars: &[char], at: usize) -> bool {
    let digit = |k: usize| chars.get(k).is_some_and(|c| c.is_numeric());
    digit(at) && digit(at + 1) && digit(at + 2) && !digit(at + 3)
}

/// The first integer in `text` (`re.search(r"\d+", s)`).
pub fn first_int(text: &str) -> Option<i64> {
    rx(r"\d+").find(text).and_then(|m| m.as_str().parse().ok())
}

/// All integers in `text`, in order.
pub fn ints(text: &str) -> Vec<i64> {
    rx(r"\d+")
        .find_iter(text)
        .filter_map(|m| m.as_str().parse().ok())
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
    }

    #[test]
    fn regex_helpers_follow_python_anchoring() {
        assert!(has_i("Owner", "the owner is"));
        assert!(!has("Owner", "the owner is"));
        assert!(starts(r"\d+", "12 ab"));
        assert!(!starts(r"\d+", "ab 12"));
        assert!(full(r"\d+", "12"));
        assert!(!full(r"\d+", "12a"));
        assert_eq!(first_int("C-17 at most 1252"), Some(17));
        assert_eq!(ints("32 to 35"), vec![32, 35]);
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
