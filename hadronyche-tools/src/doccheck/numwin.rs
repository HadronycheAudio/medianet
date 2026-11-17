//! Python look-arounds and character windows: the number guards of the
//! scripts (`(?<![\w.\-])(\d+)\.(\d+)(?!\w)`, `(?<=[.;])\s+`, …).
//!
//! [`super::text::rx`] refuses a look-behind (`pycompat::unsupported`):
//! the `regex` crate has none, and a hand-written scan that drops a match
//! whose previous character is wrong, then retries, is not Python's
//! `finditer` once the look-ahead forces the engine to backtrack into a
//! shorter match (`(\d{1,3}(?: \d{3})+|\d+)(?![\w]|\.\d)`). [`lookaround`]
//! instead compiles the script's pattern as written, Python's classes
//! translated ([`super::pycompat::translate`]), on the backtracking engine
//! (`fancy-regex`), which tries every start position and backtracks exactly
//! as `re` does. Every look-behind of the scripts has a fixed width (one
//! class, or a word such as `not `), which is all that engine (and Python's
//! `re`) accepts. A plain-regex prefilter (the pattern without its
//! look-arounds, `\b` and `$`) skips texts that cannot match.
//!
//! The proof is `data/lookaround_corpus.txt` (`tools/gen_markdown_corpus.py`):
//! every complete look-around pattern of `checks/*.sh` at b7c71c5, run by
//! Python 3.10 and 3.13 (which agree) over the lines of the design documents
//! and over edge strings; [`lookaround`] gives the same matches, spans and
//! groups on every one.
//!
//! The other refused constructs (`\Z`, `{,n}`, back-references) stay
//! refused here too. Like any [`Rx`], an engine failure turns the check red
//! (`REGEX_LIMIT`, R19), never "no match".

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use super::pycompat::{relaxed, translate, unsupported};
use super::rx::Rx;
use super::text::RX_CACHE_LIMIT;

type Cache = Mutex<HashMap<String, Arc<Rx>>>;
static CACHE: OnceLock<Cache> = OnceLock::new();

/// A Python pattern that may hold fixed-width look-behinds and
/// look-aheads, with Python's semantics (classes, `\b`, `$`, `finditer`,
/// `split`, `sub`), cached like [`super::text::rx`].
pub fn lookaround(pattern: &str) -> Arc<Rx> {
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(found) = cache.lock().expect("lookaround cache").get(pattern) {
        return found.clone();
    }
    let probe = pattern.replace("(?<=", "(?:").replace("(?<!", "(?:");
    let compiled = Arc::new(match unsupported(&probe) {
        Some(why) => Rx::refused(pattern, &why),
        None => {
            let (source, _) = translate(pattern);
            let pre = relaxed(&strip_lookarounds(pattern));
            Rx::new(pattern, &source, Some(&pre))
        }
    });
    let mut cache = cache.lock().expect("lookaround cache");
    if cache.len() < RX_CACHE_LIMIT {
        cache.insert(pattern.to_string(), compiled.clone());
    }
    compiled
}

/// `pattern` with every look-around group removed (an assertion removed
/// only widens what matches, so the result is a sound prefilter).
fn strip_lookarounds(pattern: &str) -> String {
    let c: Vec<char> = pattern.chars().collect();
    let mut out = String::with_capacity(pattern.len());
    let mut i = 0;
    let mut in_class = false;
    while i < c.len() {
        let rest: String = c[i..c.len().min(i + 4)].iter().collect();
        if c[i] == '\\' {
            out.extend(&c[i..c.len().min(i + 2)]);
            i += 2;
            continue;
        }
        if in_class {
            in_class = c[i] != ']';
            out.push(c[i]);
            i += 1;
            continue;
        }
        if c[i] == '[' {
            in_class = true;
            out.push('[');
            i += 1;
            // a `]` right after `[` or `[^` is a literal
            for lit in ['^', ']'] {
                if c.get(i) == Some(&lit) {
                    out.push(lit);
                    i += 1;
                }
            }
            continue;
        }
        let look = ["(?<=", "(?<!"].iter().any(|p| rest.starts_with(p))
            || ["(?=", "(?!"].iter().any(|p| rest.starts_with(p));
        if look {
            i = group_end(&c, i);
            continue;
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// The index after the group that opens at `c[open]`.
fn group_end(c: &[char], open: usize) -> usize {
    let (mut depth, mut i, mut in_class) = (0usize, open, false);
    while i < c.len() {
        match c[i] {
            '\\' => i += 1,
            '[' if !in_class => in_class = true,
            ']' if in_class => in_class = false,
            '(' if !in_class => depth += 1,
            ')' if !in_class => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    c.len()
}

/// `text[max(0, start - before) : end + after]` in characters, `start` and
/// `end` byte offsets of a match (the scripts' context windows:
/// `SUB_JOINED[max(0, m.start() - 80):m.end() + 80]`). Python clamps both
/// ends; so does this.
pub fn window(text: &str, start: usize, end: usize, before: usize, after: usize) -> &str {
    let lo_char = text[..start].chars().count().saturating_sub(before);
    let lo = text
        .char_indices()
        .nth(lo_char)
        .map_or(text.len(), |(b, _)| b);
    let hi = text[end..]
        .char_indices()
        .nth(after)
        .map_or(text.len(), |(b, _)| end + b);
    &text[lo..hi]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_behind_and_ahead_follow_python() {
        let int = lookaround(r"(?<![\w.\-])(\d{1,3}(?: \d{3})+|\d+)(?![\w]|\.\d)");
        let got: Vec<&str> = int
            .find_iter("1 280 a1 2.5 -7 65 507x 12 345")
            .map(|m| m.as_str())
            .collect();
        // Python 3.10/3.13: ["1 280", "65", "12 345"] ("65 507x" backtracks to "65")
        assert_eq!(got, vec!["1 280", "65", "12 345"]);
        let split: Vec<&str> = lookaround(r"(?<=[.;])\s+").split("a. b; c d.").collect();
        assert_eq!(split, vec!["a.", "b;", "c d."]);
        assert!(lookaround(r"(?<!not )\bfits\b").is_match("it fits"));
        assert!(!lookaround(r"(?<!not )\bfits\b").is_match("it does not fits"));
        assert!(
            lookaround(r"a\Z").compile_error().is_some(),
            r"\Z stays refused"
        );
        assert!(
            lookaround(r"(?<=\d+)x").compile_error().is_some(),
            "variable width"
        );
    }

    #[test]
    fn prefilter_drops_every_assertion() {
        assert_eq!(
            strip_lookarounds(r"(?<![\w.(])(\d+)(?=[)\]])x(?!(a|b))"),
            r"(\d+)x"
        );
        assert_eq!(strip_lookarounds(r"[(?<=]a\(?<=b"), r"[(?<=]a\(?<=b");
    }

    #[test]
    fn windows_clamp_in_characters() {
        let t = "µµµ 1280 µµ";
        let s = t.find("1280").expect("in the text");
        assert_eq!(window(t, s, s + 4, 2, 2), "µ 1280 µ");
        assert_eq!(window(t, s, s + 4, 80, 80), t);
        assert_eq!(window(t, 0, 0, 3, 0), "");
    }
}
