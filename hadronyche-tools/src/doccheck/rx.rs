//! One regular-expression type over two engines.
//!
//! Patterns are compiled by the `regex` crate, except those that need
//! Python's `\b`/`\B`: Python's word boundary is between characters of
//! Python's word class (`[\p{L}\p{N}_]`: `²` is a word character, a combining
//! mark is not), which the `regex` crate cannot express (its `\b` has Rust's
//! class and there is no look-around). [`super::pycompat::translate`] rewrites
//! them as look-arounds over Python's class, and those patterns are compiled
//! by `fancy-regex`. Callers see one API ([`Rx`], [`M`], [`Caps`]) whichever
//! engine runs.
//!
//! Iteration (`find_iter`, `captures_iter`, `replace*`, `split`) follows
//! Python 3.7+: an empty match is allowed right after a non-empty one, and
//! after an empty match the next one must advance (`re.sub("x*", "-",
//! "abxd")` is `-a-b--d-`).
//!
//! A regex engine error is never "no match" (lead ruling R19). When
//! `fancy-regex` exceeds its backtracking limit, or a pattern did not
//! compile, the event is recorded in this thread's failure list
//! ([`take_failures`]) and the match is reported empty. A check never sees
//! that list: the only entry point to a check,
//! [`crate::doccheck::checks::CheckSpec::check`], clears it before the run
//! and turns the report red under criterion `REGEX_LIMIT` when it is not
//! empty afterwards, with the pattern and the place of the text it was
//! matching. A check cannot ignore or discard it, and a source test forbids
//! calling a check's `run` other than through `check`.

use std::cell::RefCell;
use std::ops::Index;

/// The backtracking steps `fancy-regex` may take for one match attempt.
/// Python has no limit (it is slow, then right); past this one the check is
/// red under `REGEX_LIMIT`, never silently "no match" (R19). The real
/// documents stay far below it (the pilots' tests assert no `REGEX_LIMIT`).
pub const BACKTRACK_LIMIT: usize = 1_000_000;

/// One regex engine failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RxFailure {
    /// The pattern as written in the check.
    pub pattern: String,
    /// What went wrong (backtracking limit, compile error).
    pub what: String,
    /// The start of the text being matched (to locate it in a document).
    pub hay: String,
}

thread_local! {
    static FAILURES: RefCell<Vec<RxFailure>> = const { RefCell::new(Vec::new()) };
}

/// The failures recorded on this thread since the last call, emptied.
pub fn take_failures() -> Vec<RxFailure> {
    FAILURES.with(|f| std::mem::take(&mut *f.borrow_mut()))
}

/// Puts back failures taken by an enclosing evaluation (a nested one leaves
/// its caller's record as it found it).
pub fn restore_failures(failures: Vec<RxFailure>) {
    FAILURES.with(|f| {
        let mut f = f.borrow_mut();
        let mine = std::mem::take(&mut *f);
        *f = failures;
        f.extend(mine);
    });
}

fn record(pattern: &str, what: &str, hay: &str) {
    FAILURES.with(|f| {
        f.borrow_mut().push(RxFailure {
            pattern: pattern.to_string(),
            what: what.to_string(),
            hay: hay.chars().take(200).collect(),
        });
    });
}

enum Engine {
    Plain(regex::Regex),
    /// The look-around pattern, and a plain prefilter that matches wherever
    /// it does (the same pattern without its boundary assertions).
    Fancy(fancy_regex::Regex, regex::Regex),
    /// A pattern that did not compile: every use records the failure.
    Broken(String),
}

/// A compiled pattern.
pub struct Rx {
    engine: Engine,
    pattern: String,
}

/// A match: byte offsets into the haystack, and the matched text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M<'h> {
    hay: &'h str,
    start: usize,
    end: usize,
}

impl<'h> M<'h> {
    /// The matched text.
    pub fn as_str(&self) -> &'h str {
        &self.hay[self.start..self.end]
    }
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
}

/// The groups of one match (`caps[0]` is the whole match).
#[derive(Debug, Clone)]
pub struct Caps<'h> {
    hay: &'h str,
    groups: Vec<Option<(usize, usize)>>,
}

impl<'h> Caps<'h> {
    /// Group `i`, if it took part in the match.
    pub fn get(&self, i: usize) -> Option<M<'h>> {
        self.groups.get(i).copied().flatten().map(|(start, end)| M {
            hay: self.hay,
            start,
            end,
        })
    }
    /// The number of groups, the whole match included.
    pub fn len(&self) -> usize {
        self.groups.len()
    }
    /// True when there is no group at all (never, in practice).
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}

impl Index<usize> for Caps<'_> {
    type Output = str;
    /// Group `i`'s text; panics when the group did not take part, as the
    /// `regex` crate does (use [`Caps::get`] for optional groups).
    fn index(&self, i: usize) -> &str {
        match self.get(i) {
            Some(m) => &self.hay[m.start..m.end],
            None => panic!("group {i} did not take part in the match"),
        }
    }
}

impl Rx {
    /// Compiles `source` (the translated form of `pattern`); a `prefilter`
    /// (the pattern without its look-arounds) selects the look-around
    /// engine. A compile error does not panic: it gives a pattern whose
    /// every use records the failure (R19).
    pub fn new(pattern: &str, source: &str, prefilter: Option<&str>) -> Rx {
        let engine = match prefilter {
            Some(pre) => {
                let fancy = fancy_regex::RegexBuilder::new(source)
                    .backtrack_limit(BACKTRACK_LIMIT)
                    .build();
                match (fancy, regex::Regex::new(pre)) {
                    (Ok(f), Ok(p)) => Engine::Fancy(f, p),
                    (Err(e), _) => Engine::Broken(e.to_string()),
                    (_, Err(e)) => Engine::Broken(e.to_string()),
                }
            }
            None => regex::Regex::new(source)
                .map_or_else(|e| Engine::Broken(e.to_string()), Engine::Plain),
        };
        Rx {
            engine,
            pattern: pattern.to_string(),
        }
    }

    /// A pattern refused before compiling (`why`): every use records it.
    pub fn refused(pattern: &str, why: &str) -> Rx {
        Rx {
            engine: Engine::Broken(why.to_string()),
            pattern: pattern.to_string(),
        }
    }

    /// The compile error, if the pattern did not compile.
    pub fn compile_error(&self) -> Option<&str> {
        match &self.engine {
            Engine::Broken(e) => Some(e),
            _ => None,
        }
    }

    fn broken(&self, hay: &str) {
        if let Engine::Broken(e) = &self.engine {
            record(
                &self.pattern,
                &format!("the pattern does not compile ({e})"),
                hay,
            );
        }
    }

    /// `re.search` as a boolean.
    pub fn is_match(&self, hay: &str) -> bool {
        match &self.engine {
            Engine::Plain(r) => r.is_match(hay),
            Engine::Fancy(r, pre) => {
                pre.is_match(hay)
                    && r.is_match(hay).unwrap_or_else(|e| {
                        record(&self.pattern, &e.to_string(), hay);
                        false
                    })
            }
            Engine::Broken(_) => {
                self.broken(hay);
                false
            }
        }
    }

    /// The leftmost match.
    pub fn find<'h>(&self, hay: &'h str) -> Option<M<'h>> {
        self.captures(hay).and_then(|c| c.get(0))
    }

    /// Every match, left to right, with Python's empty-match rule.
    pub fn find_iter<'h>(&self, hay: &'h str) -> std::vec::IntoIter<M<'h>> {
        self.captures_iter(hay)
            .filter_map(|c| c.get(0))
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// The groups of the leftmost match at or after `pos`; `Err` on an engine
    /// failure (recorded).
    fn captures_at<'h>(&self, hay: &'h str, pos: usize) -> Option<Caps<'h>> {
        match &self.engine {
            Engine::Plain(r) => r.captures_at(hay, pos).map(|c| plain_caps(hay, &c)),
            Engine::Fancy(r, _) => match r.captures_from_pos(hay, pos) {
                Ok(c) => c.map(|c| fancy_caps(hay, &c)),
                Err(e) => {
                    record(&self.pattern, &e.to_string(), hay);
                    None
                }
            },
            Engine::Broken(_) => {
                self.broken(hay);
                None
            }
        }
    }

    /// The groups of the leftmost match.
    pub fn captures<'h>(&self, hay: &'h str) -> Option<Caps<'h>> {
        if let Engine::Fancy(_, pre) = &self.engine
            && !pre.is_match(hay)
        {
            return None;
        }
        self.captures_at(hay, 0)
    }

    /// The groups of every match, left to right: after an empty match the
    /// next must advance; an empty match right after a non-empty one counts
    /// (Python 3.7+).
    pub fn captures_iter<'h>(&self, hay: &'h str) -> std::vec::IntoIter<Caps<'h>> {
        let mut out = Vec::new();
        if let Engine::Fancy(_, pre) = &self.engine
            && !pre.is_match(hay)
        {
            return out.into_iter();
        }
        let (mut pos, mut must_advance) = (0, false);
        while pos <= hay.len() {
            let Some(c) = self.captures_at(hay, pos) else {
                break;
            };
            let Some((start, end)) = c.groups.first().copied().flatten() else {
                break;
            };
            if must_advance && start == pos && end == pos {
                must_advance = false;
                match hay[pos..].chars().next() {
                    Some(ch) => pos += ch.len_utf8(),
                    None => break,
                }
                continue;
            }
            must_advance = start == end;
            pos = end;
            out.push(c);
        }
        out.into_iter()
    }

    /// `re.sub(pattern, rep, hay, count=1)`; `rep` may use `$1`, `${1}`, `$$`.
    pub fn replace(&self, hay: &str, rep: &str) -> String {
        self.replace_n(hay, Some(1), |c| expand(rep, c))
    }

    /// `re.sub(pattern, rep, hay)`; `rep` may use `$1`, `${1}`, `$$`.
    pub fn replace_all(&self, hay: &str, rep: &str) -> String {
        self.replace_n(hay, None, |c| expand(rep, c))
    }

    /// The first match replaced by `rep`, taken literally.
    pub fn replace_literal(&self, hay: &str, rep: &str) -> String {
        self.replace_n(hay, Some(1), |_| rep.to_string())
    }

    /// Every match replaced by `rep`, taken literally.
    pub fn replace_all_literal(&self, hay: &str, rep: &str) -> String {
        self.replace_n(hay, None, |_| rep.to_string())
    }

    fn replace_n(&self, hay: &str, n: Option<usize>, f: impl Fn(&Caps) -> String) -> String {
        let mut out = String::with_capacity(hay.len());
        let mut last = 0;
        for (k, c) in self.captures_iter(hay).enumerate() {
            if n.is_some_and(|n| k >= n) {
                break;
            }
            let Some(m) = c.get(0) else { continue };
            out.push_str(&hay[last..m.start]);
            out.push_str(&f(&c));
            last = m.end;
        }
        out.push_str(&hay[last..]);
        out
    }

    /// `re.split(pattern, hay)` (no groups kept).
    pub fn split<'h>(&self, hay: &'h str) -> std::vec::IntoIter<&'h str> {
        let mut out = Vec::new();
        let mut last = 0;
        for m in self.find_iter(hay) {
            out.push(&hay[last..m.start]);
            last = m.end;
        }
        out.push(&hay[last..]);
        out.into_iter()
    }
}

fn plain_caps<'h>(hay: &'h str, c: &regex::Captures<'h>) -> Caps<'h> {
    Caps {
        hay,
        groups: c.iter().map(|g| g.map(|m| (m.start(), m.end()))).collect(),
    }
}

fn fancy_caps<'h>(hay: &'h str, c: &fancy_regex::Captures<'h>) -> Caps<'h> {
    Caps {
        hay,
        groups: (0..c.len())
            .map(|i| c.get(i).map(|m| (m.start(), m.end())))
            .collect(),
    }
}

/// `$N`, `${N}` and `$$` in a replacement (no named groups: the checks use
/// numbered ones).
fn expand(rep: &str, c: &Caps) -> String {
    let mut out = String::new();
    let chars: Vec<char> = rep.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '$' || i + 1 == chars.len() {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        if chars[i + 1] == '$' {
            out.push('$');
            i += 2;
            continue;
        }
        let braced = chars[i + 1] == '{';
        let start = if braced { i + 2 } else { i + 1 };
        let mut j = start;
        while j < chars.len() && chars[j].is_ascii_digit() {
            j += 1;
        }
        if j == start || (braced && chars.get(j) != Some(&'}')) {
            out.push('$');
            i += 1;
            continue;
        }
        let n: usize = chars[start..j]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap_or(usize::MAX);
        if let Some(m) = c.get(n) {
            out.push_str(m.as_str());
        }
        i = if braced { j + 1 } else { j };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(p: &str) -> Rx {
        Rx::new(p, p, None)
    }

    #[test]
    fn both_engines_share_one_api() {
        for pre in [None, Some(r"(\d+)-(\d+)")] {
            let r = Rx::new("p", r"(\d+)-(\d+)", pre);
            let c = r.captures("a 1-22 b 3-4").unwrap();
            assert_eq!(&c[1], "1");
            assert_eq!(c.get(2).unwrap().as_str(), "22");
            assert_eq!(r.find_iter("1-2 3-4").count(), 2);
            assert_eq!(r.replace_all("1-2 3-4", "$2:${1}$$"), "2:1$ 4:3$");
            assert_eq!(r.replace("1-2 3-4", "x"), "x 3-4");
            assert_eq!(
                r.split("a1-2b3-4c").collect::<Vec<_>>(),
                vec!["a", "b", "c"]
            );
            assert_eq!(r.find("x 5-6").map(|m| (m.start(), m.end())), Some((2, 5)));
        }
        assert_eq!(plain("a").replace_all_literal("aXa", "$1"), "$1X$1");
    }

    #[test]
    fn empty_matches_follow_python() {
        assert_eq!(plain("x*").replace_all("abxd", "-"), "-a-b--d-");
        assert_eq!(plain(r"\s*").replace_all("a b", "-"), "-a--b-");
        assert_eq!(
            plain(",?").split("a,b").collect::<Vec<_>>(),
            vec!["", "a", "", "b", ""]
        );
    }

    #[test]
    fn engine_failures_are_recorded_never_silent() {
        take_failures();
        let bad = Rx::new("(", "(", None);
        assert!(bad.compile_error().is_some());
        assert!(!bad.is_match("x"));
        let f = take_failures();
        assert_eq!(f.len(), 1);
        assert!(f[0].what.contains("does not compile"));
        // exponential backtracking (review r3, finding 1): the limit is a
        // recorded failure, never "no match"
        let r = crate::doccheck::text::rx(r"\b[A-Z][a-z0-9]+(?:[A-Z][A-Za-z0-9]*)+\b");
        let hay = format!("Aa{}_ x BadMagik", "B".repeat(26));
        let _ = r.find_iter(&hay).count();
        assert!(
            !take_failures().is_empty(),
            "a backtracking limit is recorded"
        );
        assert!(take_failures().is_empty(), "taken once");
    }
}
