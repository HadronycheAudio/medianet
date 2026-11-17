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
//! `fancy-regex` reports an error instead of a match when its backtracking
//! limit is exceeded; [`Rx`] then counts "no match" and records the event in
//! [`Rx::limit_hits`] (a check that hits it is suspect, and the counter is
//! asserted to stay zero by the library tests on the real documents).

use std::ops::Index;
use std::sync::atomic::{AtomicUsize, Ordering};

static LIMIT_HITS: AtomicUsize = AtomicUsize::new(0);

enum Engine {
    Plain(regex::Regex),
    /// The look-around pattern, and a plain prefilter that matches wherever
    /// it does (the same pattern without its boundary assertions).
    Fancy(fancy_regex::Regex, regex::Regex),
}

/// A compiled pattern.
pub struct Rx {
    engine: Engine,
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

fn limit_hit() {
    LIMIT_HITS.fetch_add(1, Ordering::Relaxed);
}

impl Rx {
    /// Compiles `source` (already translated); a `prefilter` (the pattern
    /// without boundaries) selects the look-around engine.
    pub fn new(source: &str, prefilter: Option<&str>) -> Result<Rx, String> {
        let engine = if let Some(pre) = prefilter {
            Engine::Fancy(
                fancy_regex::RegexBuilder::new(source)
                    .backtrack_limit(10_000_000)
                    .build()
                    .map_err(|e| e.to_string())?,
                regex::Regex::new(pre).map_err(|e| e.to_string())?,
            )
        } else {
            Engine::Plain(regex::Regex::new(source).map_err(|e| e.to_string())?)
        };
        Ok(Rx { engine })
    }

    /// How many times a fancy match gave up at its backtracking limit.
    pub fn limit_hits() -> usize {
        LIMIT_HITS.load(Ordering::Relaxed)
    }

    /// `re.search` as a boolean.
    pub fn is_match(&self, hay: &str) -> bool {
        match &self.engine {
            Engine::Plain(r) => r.is_match(hay),
            Engine::Fancy(r, pre) => {
                pre.is_match(hay)
                    && r.is_match(hay).unwrap_or_else(|_| {
                        limit_hit();
                        false
                    })
            }
        }
    }

    /// The leftmost match.
    pub fn find<'h>(&self, hay: &'h str) -> Option<M<'h>> {
        self.captures(hay).and_then(|c| c.get(0))
    }

    /// Every non-overlapping match, left to right.
    pub fn find_iter<'h>(&self, hay: &'h str) -> std::vec::IntoIter<M<'h>> {
        self.captures_iter(hay)
            .filter_map(|c| c.get(0))
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// The groups of the leftmost match.
    pub fn captures<'h>(&self, hay: &'h str) -> Option<Caps<'h>> {
        match &self.engine {
            Engine::Plain(r) => r.captures(hay).map(|c| plain_caps(hay, &c)),
            Engine::Fancy(_, pre) if !pre.is_match(hay) => None,
            Engine::Fancy(r, _) => match r.captures(hay) {
                Ok(c) => c.map(|c| fancy_caps(hay, &c)),
                Err(_) => {
                    limit_hit();
                    None
                }
            },
        }
    }

    /// The groups of every non-overlapping match, left to right.
    pub fn captures_iter<'h>(&self, hay: &'h str) -> std::vec::IntoIter<Caps<'h>> {
        let all: Vec<Caps<'h>> = match &self.engine {
            Engine::Plain(r) => r.captures_iter(hay).map(|c| plain_caps(hay, &c)).collect(),
            Engine::Fancy(_, pre) if !pre.is_match(hay) => Vec::new(),
            Engine::Fancy(r, _) => r
                .captures_iter(hay)
                .map_while(|c| match c {
                    Ok(c) => Some(fancy_caps(hay, &c)),
                    Err(_) => {
                        limit_hit();
                        None
                    }
                })
                .collect(),
        };
        all.into_iter()
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

    #[test]
    fn both_engines_share_one_api() {
        for pre in [None, Some(r"(\d+)-(\d+)")] {
            let r = Rx::new(r"(\d+)-(\d+)", pre).unwrap();
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
        let lit = Rx::new("a", None).unwrap();
        assert_eq!(lit.replace_all_literal("aXa", "$1"), "$1X$1");
    }
}
