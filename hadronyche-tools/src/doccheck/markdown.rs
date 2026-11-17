//! The document model for the project's Markdown.
//!
//! A [`Markdown`] holds the lines of a document, which of them are inside a
//! fenced code block, and its headings with their anchors as GitHub renders
//! them. Sections are addressed by heading (text pattern, level, or path of
//! headings); everything a check reads from a section — units, tables, links —
//! excludes fenced code.
//!
//! Line numbers in this module are 0-based indices into [`Markdown::lines`];
//! [`Markdown::line_no`] gives the 1-based number a report prints.

use super::policy::{FencePolicy, HeadingMatch, SectionEnd, TableRules};
use super::pycompat::PyStr;
use super::tables::{Table, parse_tables};
use super::text::{rx, split_nl};

/// One heading of a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// 0-based line index.
    pub line: usize,
    /// 1 to 6.
    pub level: usize,
    /// The heading text, trimmed, closing `#`s removed.
    pub text: String,
    /// The anchor GitHub gives it, made unique with `-1`, `-2`, ...
    pub slug: String,
}

/// A parsed Markdown document.
#[derive(Debug, Clone)]
pub struct Markdown {
    /// The lines (`text.split("\n")`).
    pub lines: Vec<String>,
    /// For each line: true when it is a fence line or inside a fence.
    pub in_code: Vec<bool>,
    /// The headings outside code, in order.
    pub heads: Vec<Heading>,
}

/// A line range `[lo, hi)` of a document, usually a heading's section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

/// GitHub's anchor for a heading text: links reduced to their text, `` ` ``
/// `*` `~` removed, lower case, every character that is not a word
/// character, `-` or a space removed, spaces turned into `-`.
pub fn slugify(text: &str) -> String {
    let t = rx(r"\[([^\]]*)\]\([^)]*\)").replace_all(text, "$1");
    let t: String = t.chars().filter(|c| !"`*~".contains(*c)).collect();
    let t = t.py_strip().to_lowercase();
    let t = rx(r"[^\w\- ]").replace_all(&t, "");
    t.replace(' ', "-")
}

/// A heading's text normalised for comparison: `` ` `` `*` `_` removed,
/// whitespace squashed, lower case, a trailing `.` dropped.
pub fn norm_title(text: &str) -> String {
    let t = rx(r"[`*_]").replace_all(text, "");
    let t = rx(r"\s+").replace_all(&t, " ").py_strip().to_lowercase();
    t.trim_end_matches('.').to_string()
}

/// A heading text without its section number (`3.7 Copies` -> `Copies`).
pub fn unnumbered(text: &str) -> String {
    rx(r"^\d+(\.\d+)*\.?\s+").replace(text, "")
}

/// The fence mask of `lines` under `policy`: true on a fence line and on
/// every line inside a fence (the scripts' `fence_mask`, `visible_lines`
/// negated, ...). Proven on `data/markdown_corpus.txt`.
pub fn code_mask<S: AsRef<str>>(lines: &[S], policy: FencePolicy) -> Vec<bool> {
    fence_scan(lines, policy).mask
}

/// A fence scan: the mask of [`code_mask`], and whether the text ends
/// inside an open fence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenceScan {
    pub mask: Vec<bool>,
    /// The fence still open at the end: (0-based line of its opener, the
    /// marker as written, e.g. "```"). 006-01 reports it as `line N:
    /// unclosed code fence opened with ```` (N 1-based); proven on the
    /// corpus (`fence_end_006_01`).
    pub open_at_end: Option<(usize, String)>,
}

/// [`code_mask`] with the fence left open at the end, if any.
pub fn fence_scan<S: AsRef<str>>(lines: &[S], policy: FencePolicy) -> FenceScan {
    let mut open: Option<(char, usize, usize)> = None;
    let mask = lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let line = line.as_ref();
            let lead = match policy {
                // `^[ \t]*`
                FencePolicy::Matching => line.trim_start_matches([' ', '\t']),
                _ => line.py_lstrip(),
            };
            let run = |c: char| lead.chars().take_while(|&x| x == c).count();
            let fence = match policy {
                FencePolicy::NoFences => None,
                FencePolicy::Backticks => (run('`') >= 3).then_some(('`', 3)),
                _ => {
                    let (b, w) = (run('`'), run('~'));
                    if b >= 3 {
                        Some(('`', b))
                    } else {
                        (w >= 3).then_some(('~', w))
                    }
                }
            };
            let Some((c, n)) = fence else {
                return open.is_some();
            };
            let tail_empty = lead[n..].py_strip().is_empty();
            open = match (open, policy) {
                (None, _) => Some((c, n, idx)),
                (Some((oc, on, at)), FencePolicy::Matching) if oc != c || n < on || !tail_empty => {
                    Some((oc, on, at))
                }
                (Some((oc, on, at)), FencePolicy::SameChar) if oc != c => Some((oc, on, at)),
                (Some(_), _) => None,
            };
            true
        })
        .collect();
    FenceScan {
        mask,
        open_at_end: open.map(|(c, n, at)| (at, c.to_string().repeat(n))),
    }
}

/// 006-01's `heading_tokens` over a title already normalised with the
/// script's `norm_text` (NFKC, typographic dashes and quotes): the
/// `[a-z0-9]+` words of its lower case, without `stopwords`, each stemmed
/// once (`-ies` → `-y` past 4 letters, `-ing` dropped past 5, `-ed` past 4,
/// a final `s` past 3 unless `ss`).
pub fn stem_tokens(normalised: &str, stopwords: &[&str]) -> std::collections::BTreeSet<String> {
    let lower = normalised.to_lowercase();
    let mut out = std::collections::BTreeSet::new();
    for m in rx("[a-z0-9]+").find_iter(&lower) {
        let w = m.as_str();
        if stopwords.contains(&w) {
            continue;
        }
        let n = w.chars().count();
        let stem = if n > 4 && w.ends_with("ies") {
            format!("{}y", &w[..w.len() - 3])
        } else if n > 5 && w.ends_with("ing") {
            w[..w.len() - 3].to_string()
        } else if n > 4 && w.ends_with("ed") {
            w[..w.len() - 2].to_string()
        } else if n > 3 && w.ends_with('s') && !w.ends_with("ss") {
            w[..w.len() - 1].to_string()
        } else {
            w.to_string()
        };
        out.insert(stem);
    }
    out
}

/// 006-01's `STOPWORDS`.
pub const STOPWORDS_006_01: &[&str] = &[
    "a", "an", "and", "the", "of", "in", "on", "or", "to", "for", "with", "as", "at", "by", "from",
    "is", "are", "be", "into", "than", "this", "that", "these", "those", "its", "it",
];

/// 006-01's `matching_heading(headings, groups)`: the first heading of
/// level 2 or deeper whose [`stem_tokens`] (after `norm`, the script's
/// `norm_text`) hold, for every group, one of its alternatives.
pub fn stem_match<'a>(
    heads: &'a [Heading],
    groups: &[&[&str]],
    norm: impl Fn(&str) -> String,
) -> Option<&'a Heading> {
    heads.iter().filter(|h| h.level >= 2).find(|h| {
        let toks = stem_tokens(&norm(&h.text), STOPWORDS_006_01);
        groups
            .iter()
            .all(|g| g.iter().any(|alt| toks.contains(*alt)))
    })
}

fn find_heads(lines: &[String], in_code: &[bool]) -> Vec<Heading> {
    let re = rx(r"^(#{1,6})\s+(.*?)\s*#*\s*$");
    let mut seen = std::collections::HashMap::<String, usize>::new();
    let mut heads = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if in_code[i] {
            continue;
        }
        let Some(m) = re.captures(line) else { continue };
        let text = m[2].py_strip().to_string();
        let mut slug = slugify(&text);
        match seen.get_mut(&slug) {
            Some(n) => {
                *n += 1;
                slug = format!("{slug}-{n}");
            }
            None => {
                seen.insert(slug.clone(), 0);
            }
        }
        heads.push(Heading {
            line: i,
            level: m[1].len(),
            text,
            slug,
        });
    }
    heads
}

impl Markdown {
    /// Parses `text` with the default [`FencePolicy::Backticks`].
    pub fn parse(text: &str) -> Markdown {
        Markdown::parse_with(text, FencePolicy::Backticks)
    }

    /// Parses `text`, recognising code fences by `fences`.
    pub fn parse_with(text: &str, fences: FencePolicy) -> Markdown {
        let lines: Vec<String> = split_nl(text).into_iter().map(String::from).collect();
        let in_code = code_mask(&lines, fences);
        let heads = find_heads(&lines, &in_code);
        Markdown {
            lines,
            in_code,
            heads,
        }
    }

    /// 1-based line number of a 0-based index.
    pub fn line_no(&self, idx: usize) -> usize {
        idx + 1
    }

    /// The whole document as a span.
    pub fn all(&self) -> Span {
        Span {
            lo: 0,
            hi: self.lines.len(),
        }
    }

    /// The section of `head`: from its line to the next heading of the same
    /// or a higher level.
    pub fn section(&self, head: &Heading) -> Span {
        self.section_with(head, SectionEnd::SameOrHigher)
    }

    /// The section of `head`, ending where `end` says.
    pub fn section_with(&self, head: &Heading, end: SectionEnd) -> Span {
        let stops = |h: &Heading| match end {
            SectionEnd::SameOrHigher => h.level <= head.level,
            SectionEnd::AnyHeading => true,
            SectionEnd::NextLevel2 => h.level <= 2,
        };
        let hi = self
            .heads
            .iter()
            .find(|h| h.line > head.line && stops(h))
            .map_or(self.lines.len(), |h| h.line);
        Span { lo: head.line, hi }
    }

    /// The headings that `pattern` finds under the matching rule `how`.
    ///
    /// Only the regex variants compile `pattern`; `Substring`, `Exact`,
    /// `NormEq` and `Slug` compare it literally, so a title such as
    /// `Foo (x` is safe to pass.
    pub fn find_by(&self, how: HeadingMatch, pattern: &str) -> Vec<&Heading> {
        let pred: Box<dyn Fn(&Heading) -> bool> = match how {
            HeadingMatch::Regex => {
                let re = rx(pattern);
                Box::new(move |h| re.is_match(&h.text))
            }
            HeadingMatch::RegexAtLevel(l) => {
                let re = rx(pattern);
                Box::new(move |h| h.level == l && re.is_match(&h.text))
            }
            HeadingMatch::FullMatch => {
                let re = rx(&format!("(?i)^(?:{pattern})$"));
                Box::new(move |h| re.is_match(&h.text))
            }
            HeadingMatch::Substring => {
                let lower = pattern.to_lowercase();
                Box::new(move |h| h.text.to_lowercase().contains(&lower))
            }
            HeadingMatch::Exact => Box::new(move |h| h.text == pattern),
            HeadingMatch::NormEq => {
                let norm = norm_title(pattern);
                Box::new(move |h| norm_title(&h.text) == norm)
            }
            HeadingMatch::Slug => Box::new(move |h| h.slug == pattern),
        };
        self.heads.iter().filter(|h| pred(h)).collect()
    }

    /// Headings whose text matches `pattern` (a regex; add `(?i)` for case).
    pub fn find(&self, pattern: &str) -> Vec<&Heading> {
        let re = rx(pattern);
        self.heads.iter().filter(|h| re.is_match(&h.text)).collect()
    }

    /// The first heading at `level` whose text matches `pattern`.
    pub fn find_at(&self, pattern: &str, level: usize) -> Option<&Heading> {
        let re = rx(pattern);
        self.heads
            .iter()
            .find(|h| h.level == level && re.is_match(&h.text))
    }

    /// The heading reached by a path of patterns, each a descendant of the
    /// previous (`["^Hello$", "common control header"]`).
    pub fn find_path(&self, path: &[&str]) -> Option<&Heading> {
        let mut scope = self.all();
        let mut found = None;
        for pattern in path {
            let re = rx(pattern);
            let h = self
                .heads
                .iter()
                .find(|h| h.line >= scope.lo && h.line < scope.hi && re.is_match(&h.text))?;
            scope = self.section(h);
            scope.lo += 1;
            found = Some(h);
        }
        found
    }

    /// The headings strictly inside `span`.
    pub fn heads_in(&self, span: Span) -> Vec<&Heading> {
        self.heads
            .iter()
            .filter(|h| h.line > span.lo && h.line < span.hi)
            .collect()
    }

    /// The text of `span`, lines joined with `\n`.
    pub fn text(&self, span: Span) -> String {
        self.lines[span.lo..span.hi].join("\n")
    }

    /// The tables of `span` outside code (see [`Table`]).
    pub fn tables(&self, span: Span) -> Vec<Table> {
        self.tables_with(span, TableRules::default())
    }

    /// The tables of `span` read under `rules` (row split, separator rule,
    /// headerless runs; [`TableRules::T004_22`], [`TableRules::FAMILY_004`]).
    pub fn tables_with(&self, span: Span, rules: TableRules) -> Vec<Table> {
        parse_tables(&self.lines, &self.in_code, span.lo, span.hi, rules)
    }

    /// Every anchor of the document.
    pub fn slugs(&self) -> std::collections::BTreeSet<String> {
        self.heads.iter().map(|h| h.slug.clone()).collect()
    }

    /// The normalised heading texts, with and without section numbers.
    pub fn head_norms(&self) -> std::collections::BTreeSet<String> {
        let mut out = std::collections::BTreeSet::new();
        for h in &self.heads {
            out.insert(norm_title(&h.text));
            out.insert(norm_title(&unnumbered(&h.text)));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# Title\n\n## Hello\n\ntext\n\n### Common control header\n\n```\n## not a heading\n```\n\n## Hello\n\n## 3.7 Copies, and `x`\n";

    #[test]
    fn headings_skip_code_and_get_unique_github_anchors() {
        let md = Markdown::parse(DOC);
        let slugs: Vec<_> = md.heads.iter().map(|h| h.slug.as_str()).collect();
        assert_eq!(
            slugs,
            vec![
                "title",
                "hello",
                "common-control-header",
                "hello-1",
                "37-copies-and-x"
            ]
        );
        assert!(md.in_code[8] && md.in_code[9] && md.in_code[10]);
    }

    #[test]
    fn slug_matches_github_for_section_signs_and_links() {
        assert_eq!(
            slugify("Discrepancies against § 3 Protocol and packetisation"),
            "discrepancies-against--3-protocol-and-packetisation"
        );
        assert_eq!(slugify("See [wire](wire.md) now"), "see-wire-now");
        assert_eq!(slugify("snake_case_name"), "snake_case_name");
    }

    #[test]
    fn sections_end_at_the_next_heading_of_same_or_higher_level() {
        let md = Markdown::parse(DOC);
        let hello = md.find("^Hello$")[0];
        let span = md.section(hello);
        assert_eq!((span.lo, span.hi), (2, 12));
        assert_eq!(md.heads_in(span).len(), 1);
        let sub = md.find_path(&["^Hello$", "(?i)common"]).unwrap();
        assert_eq!(sub.line, 6);
        assert!(md.find_path(&["^Title$", "^nothing$"]).is_none());
        assert_eq!(md.find_at("Hello", 3), None);
    }

    #[test]
    fn fence_policies() {
        let text = "~~~\n## a\n~~~\n````\n```\n## b\n````\n## c\n";
        let titles = |f| {
            Markdown::parse_with(text, f)
                .heads
                .iter()
                .map(|h| h.text.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(titles(FencePolicy::Backticks), vec!["a", "b"]);
        assert_eq!(titles(FencePolicy::BackticksOrTildes), vec!["b"]);
        assert_eq!(titles(FencePolicy::Matching), vec!["c"]);
    }

    #[test]
    fn heading_match_and_section_end_policies() {
        let md = Markdown::parse(DOC);
        assert_eq!(md.find_by(HeadingMatch::Regex, "ell").len(), 2);
        assert_eq!(
            md.find_by(HeadingMatch::RegexAtLevel(3), "(?i)common")
                .len(),
            1
        );
        assert_eq!(md.find_by(HeadingMatch::FullMatch, "hello").len(), 2);
        assert_eq!(md.find_by(HeadingMatch::FullMatch, "hell").len(), 0);
        assert_eq!(md.find_by(HeadingMatch::Substring, "COPIES").len(), 1);
        assert_eq!(md.find_by(HeadingMatch::Exact, "hello").len(), 0);
        assert_eq!(
            md.find_by(HeadingMatch::NormEq, "3.7 Copies, and x").len(),
            1
        );
        assert_eq!(md.find_by(HeadingMatch::Slug, "hello-1").len(), 1);
        assert!(
            md.find_by(HeadingMatch::Substring, "Foo (x").is_empty(),
            "no regex, no panic"
        );
        assert!(md.find_by(HeadingMatch::Exact, "a[").is_empty());
        let title = md.find("^Title$")[0];
        assert_eq!(md.section_with(title, SectionEnd::AnyHeading).hi, 2);
        assert_eq!(md.section_with(title, SectionEnd::NextLevel2).hi, 2);
        assert_eq!(md.section(title).hi, md.lines.len());
    }

    #[test]
    fn titles_normalise_with_and_without_numbers() {
        let md = Markdown::parse(DOC);
        let norms = md.head_norms();
        assert!(norms.contains("3.7 copies, and x"));
        assert!(norms.contains("copies, and x"));
        assert_eq!(norm_title("  Copy   *index*. "), "copy index");
    }
}
