//! Evidence tags and the house "untagged claim" rule.
//!
//! The design set has six evidence tags: `[code]`, `[measured]`,
//! `[kernel-doc]`, `[memory]` and `[decision]` (defined in
//! `doc/design/00-context.md`) and `[standard]` (constant C-132 in
//! `doc/design/02-constants.md`).
//!
//! The rule "every paragraph of 5 or more words carries a tag" is written
//! differently by the scripts (inventory
//! `swarm/notes/checks-inventory-2026-11-16.md`, sections 4a and 4b): they
//! do not agree on the tag set, its case, what a paragraph is, how words are
//! counted, or whether a table inherits the tag of the block before it. Each
//! variant is a named [`TagPolicy`]; a port names the policy of the script it
//! replaces and never gets another one silently. Unifying the variants is a
//! later, reviewed step.
//!
//! | policy | scripts | set, case | paragraph | words | tables |
//! |---|---|---|---|---|---|
//! | [`TagPolicy::UNITS_004_22`] | 004-22 | six, exact | [`units`]: blank, table row, heading or list marker ends it | `\S+` after links reduced to text, >= 5 | exempt |
//! | [`TagPolicy::PROSE_004_27`] | 004-27 | six, any | [`prose_blocks`] `p` | `split()` after `` `*_\|># `` blanked, >= 5 | exempt |
//! | [`TagPolicy::PROSE_004_29`] | 004-25, 004-28, 004-29 | six, any | as 004-27 | as 004-27, `-` also blanked | exempt |
//! | [`TagPolicy::PROSE_004_26`] | 004-26 | six, any | as 004-27 | `\S+`, >= 5 | exempt |
//! | [`TagPolicy::ALPHA_004_16`] | 004-16, 004-17 (004-18 to 004-21 differ on tables) | six, any | as 004-27 | `[A-Za-z]{2,}` outside code spans, >= 5 | exempt |
//! | [`TagPolicy::ALNUM_004_14`] | 004-14 (004-15 on its own units) | six, any | units, `+` also a list marker | `[A-Za-z0-9]+`, >= 5 | exempt |
//! | [`TagPolicy::EVERY_BLOCK_004_09`] | 004-03, 05, 06, 08, 09, 10, 11 | six, any | every blank-separated block | none | inherit from the block before |
//! | [`TagPolicy::EVERY_BLOCK_004_13`] | 004-13 | six, any | every blank-separated block | none | need their own |
//!
//! Not implemented yet (each port adds its own constant, never edits one):
//! the five-tag case-sensitive set of epic 002 with its pointer-line and
//! figure-paragraph windows, 003-08's three-tag set, 004-04's list
//! inheritance, 004-07's "three tags per section and a tag within 500
//! characters", 004-12's word pattern with its `[code]` pointer rule, the
//! single-tag rules of 003-02, 003-05, 004-01, 006-01 and 006-02. 004-27 also
//! requires a tag in every `###` subsection: [`subsections_untagged`].

use super::text::{rx, strip_links};
use super::units::{BlockKind, UnitKind, prose_blocks, units};

/// A set of evidence tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagSet {
    /// `[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`
    /// (epic 002, before C-132).
    Five,
    /// The five and `[standard]` (C-132).
    Six,
    /// `[code]`, `[measured]`, `[decision]` (003-08).
    CodeMeasuredDecision,
}

impl TagSet {
    /// The alternation of tag names.
    pub fn names(self) -> &'static str {
        match self {
            TagSet::Five => "code|measured|kernel-doc|memory|decision",
            TagSet::Six => "code|measured|kernel-doc|standard|memory|decision",
            TagSet::CodeMeasuredDecision => "code|measured|decision",
        }
    }

    /// True when `text` carries a tag of this set (`exact_case`: lower case
    /// only).
    pub fn found_in(self, text: &str, exact_case: bool) -> bool {
        let flag = if exact_case { "" } else { "(?i)" };
        rx(&format!(r"{flag}\[(?:{})\]", self.names())).is_match(text)
    }
}

/// The six evidence tags.
pub const TAGS: [&str; 6] = [
    "code",
    "measured",
    "kernel-doc",
    "standard",
    "memory",
    "decision",
];

/// True when `text` carries an evidence tag (`[code]` ...), any case.
pub fn has_tag(text: &str) -> bool {
    rx(r"(?i)\[(?:code|measured|kernel-doc|standard|memory|decision)\]").is_match(text)
}

/// True when `text` carries an evidence tag written in lower case exactly.
pub fn has_tag_exact(text: &str) -> bool {
    rx(r"\[(?:decision|code|measured|kernel-doc|standard|memory)\]").is_match(text)
}

/// The tags `text` carries, in order of appearance, lower-cased.
pub fn tags_in(text: &str) -> Vec<String> {
    rx(r"(?i)\[(code|measured|kernel-doc|standard|memory|decision)\]")
        .captures_iter(text)
        .map(|c| c[1].to_lowercase())
        .collect()
}

/// How a paragraph is cut out of a section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segmentation {
    /// [`units`] paragraphs and list items (table rows and headings exempt).
    Units,
    /// [`units`] with `+` also a list marker (004-14).
    UnitsPlus,
    /// [`prose_blocks`] of kind prose.
    ProseBlocks,
    /// Every blank-separated block; with `inherit`, a table or code block
    /// right after a tagged block inherits its tag.
    EveryBlock { inherit: bool },
}

/// How words are counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordCount {
    /// `\S+` after Markdown links are reduced to their text.
    NonSpaceNoLinks,
    /// `\S+`.
    NonSpace,
    /// `str.split()` after the given characters are replaced by spaces.
    SplitBlanking(&'static str),
    /// `[A-Za-z]{2,}` after code spans are removed.
    AlphaNoCode,
    /// `[A-Za-z0-9]+`.
    Alnum,
}

/// One variant of the untagged-claim rule (see the module table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagPolicy {
    pub name: &'static str,
    pub segmentation: Segmentation,
    pub words: WordCount,
    pub min_words: usize,
    /// True: the tag must be written in lower case (`[decision]`).
    pub exact_case: bool,
    /// The tags that count.
    pub set: TagSet,
}

impl TagPolicy {
    pub const UNITS_004_22: TagPolicy = TagPolicy {
        name: "units-004-22",
        segmentation: Segmentation::Units,
        words: WordCount::NonSpaceNoLinks,
        min_words: 5,
        exact_case: true,
        set: TagSet::Six,
    };
    pub const PROSE_004_27: TagPolicy = TagPolicy {
        name: "prose-004-27",
        segmentation: Segmentation::ProseBlocks,
        words: WordCount::SplitBlanking("`*_|>#"),
        min_words: 5,
        exact_case: false,
        set: TagSet::Six,
    };
    pub const PROSE_004_29: TagPolicy = TagPolicy {
        name: "prose-004-29",
        segmentation: Segmentation::ProseBlocks,
        words: WordCount::SplitBlanking("`*_|>#-"),
        min_words: 5,
        exact_case: false,
        set: TagSet::Six,
    };
    pub const PROSE_004_26: TagPolicy = TagPolicy {
        name: "prose-004-26",
        segmentation: Segmentation::ProseBlocks,
        words: WordCount::NonSpace,
        min_words: 5,
        exact_case: false,
        set: TagSet::Six,
    };
    pub const ALPHA_004_16: TagPolicy = TagPolicy {
        name: "alpha-004-16",
        segmentation: Segmentation::ProseBlocks,
        words: WordCount::AlphaNoCode,
        min_words: 5,
        exact_case: false,
        set: TagSet::Six,
    };
    pub const ALNUM_004_14: TagPolicy = TagPolicy {
        name: "alnum-004-14",
        segmentation: Segmentation::UnitsPlus,
        words: WordCount::Alnum,
        min_words: 5,
        exact_case: false,
        set: TagSet::Six,
    };
    pub const EVERY_BLOCK_004_09: TagPolicy = TagPolicy {
        name: "every-block-004-09",
        segmentation: Segmentation::EveryBlock { inherit: true },
        words: WordCount::NonSpace,
        min_words: 0,
        exact_case: false,
        set: TagSet::Six,
    };
    pub const EVERY_BLOCK_004_13: TagPolicy = TagPolicy {
        name: "every-block-004-13",
        segmentation: Segmentation::EveryBlock { inherit: false },
        words: WordCount::NonSpace,
        min_words: 0,
        exact_case: false,
        set: TagSet::Six,
    };
}

/// An untagged paragraph: its first line (0-based, relative to the text
/// given) and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Untagged {
    pub line: usize,
    pub text: String,
}

/// The number of words of `text` under `count`.
pub fn count_words(text: &str, count: WordCount) -> usize {
    match count {
        WordCount::NonSpaceNoLinks => rx(r"\S+").find_iter(&strip_links(text)).count(),
        WordCount::NonSpace => rx(r"\S+").find_iter(text).count(),
        WordCount::SplitBlanking(chars) => super::text::blank_chars(text, chars)
            .split_whitespace()
            .count(),
        WordCount::AlphaNoCode => {
            let no_code = rx(r"`[^`]*`").replace_all(text, "");
            rx(r"[A-Za-z]{2,}").find_iter(&no_code).count()
        }
        WordCount::Alnum => rx(r"[A-Za-z0-9]+").find_iter(text).count(),
    }
}

fn tagged(text: &str, policy: &TagPolicy) -> bool {
    policy.set.found_in(text, policy.exact_case)
}

/// The paragraphs of `text` that break the rule under `policy`.
pub fn untagged(text: &str, policy: &TagPolicy) -> Vec<Untagged> {
    let candidates: Vec<(usize, String)> = match policy.segmentation {
        Segmentation::Units | Segmentation::UnitsPlus => units(text)
            .into_iter()
            .filter(|u| matches!(u.kind, UnitKind::Paragraph | UnitKind::ListItem))
            .map(|u| (u.lo, u.text))
            .collect(),
        Segmentation::ProseBlocks => prose_blocks(text)
            .into_iter()
            .filter(|b| b.kind == BlockKind::Prose)
            .map(|b| (b.line, b.text))
            .collect(),
        Segmentation::EveryBlock { inherit } => {
            return every_block_untagged(text, policy, inherit);
        }
    };
    candidates
        .into_iter()
        .filter(|(_, t)| count_words(t, policy.words) >= policy.min_words && !tagged(t, policy))
        .map(|(line, text)| Untagged { line, text })
        .collect()
}

/// 004-09's rule: every blank-separated block (heading lines removed)
/// carries a tag, except a table or code block right after a tagged block.
fn every_block_untagged(text: &str, policy: &TagPolicy, inherit: bool) -> Vec<Untagged> {
    let mut out = Vec::new();
    let mut prev_tagged = true;
    let mut line = 0;
    for block in text.split("\n\n") {
        let start = line;
        line += block.matches('\n').count() + 2;
        let content: Vec<&str> = block
            .trim()
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect();
        if block.trim().is_empty() {
            continue;
        }
        if content.is_empty() {
            prev_tagged = true;
            continue;
        }
        let body = content.join("\n");
        let body = body.trim();
        let is_table = content
            .iter()
            .filter(|l| !l.trim().is_empty())
            .all(|l| l.trim().starts_with('|'));
        let is_code = content[0].trim_start().starts_with("```");
        if tagged(body, policy) || (inherit && (is_table || is_code) && prev_tagged) {
            prev_tagged = true;
            continue;
        }
        out.push(Untagged {
            line: start,
            text: body.to_string(),
        });
        prev_tagged = false;
    }
    out
}

/// The titles of the `###` subsections of `text` that carry no tag at all
/// (004-27's "every subsection carries an evidence tag").
pub fn subsections_untagged(subsections: &[(String, String)]) -> Vec<String> {
    subsections
        .iter()
        .filter(|(_, body)| !has_tag(body))
        .map(|(title, _)| title.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_tags_any_case_or_exact() {
        for t in TAGS {
            assert!(has_tag(&format!("x [{t}] y")));
            assert!(has_tag_exact(&format!("x [{t}] y")));
        }
        assert!(has_tag("[Decision]"));
        assert!(!has_tag_exact("[Decision]"));
        assert!(!has_tag("[decided]"));
        assert_eq!(tags_in("a [code] b [Measured]"), vec!["code", "measured"]);
    }

    #[test]
    fn units_policy_splits_list_items_and_counts_link_text() {
        let text = "Five words in this paragraph `[decision]`.\n- one two three four five\n- [a b](x.md) e\n";
        let bad = untagged(text, &TagPolicy::UNITS_004_22);
        assert_eq!(bad.len(), 1, "{bad:?}");
        assert_eq!(bad[0].line, 1);
        let short = untagged("[a](b) c d\n", &TagPolicy::UNITS_004_22);
        assert!(short.is_empty(), "three words are below the minimum");
        assert_eq!(
            count_words("[a b](t.md#x) c", WordCount::NonSpaceNoLinks),
            3
        );
    }

    #[test]
    fn prose_policies_differ_on_dashes() {
        let text = "a - b - c - d\n";
        assert_eq!(untagged(text, &TagPolicy::PROSE_004_27).len(), 1);
        assert!(untagged(text, &TagPolicy::PROSE_004_29).is_empty());
        let list = "- one two\n- three four five six\n";
        assert_eq!(untagged(list, &TagPolicy::PROSE_004_27).len(), 1);
        assert_eq!(untagged(list, &TagPolicy::UNITS_004_22).len(), 1);
        assert_eq!(untagged(list, &TagPolicy::UNITS_004_22)[0].line, 1);
    }

    #[test]
    fn case_matters_only_for_the_exact_policy() {
        let text = "one two three four five [Decision]\n";
        assert_eq!(untagged(text, &TagPolicy::UNITS_004_22).len(), 1);
        assert!(untagged(text, &TagPolicy::PROSE_004_27).is_empty());
    }

    #[test]
    fn tag_sets() {
        assert!(TagSet::Six.found_in("[standard]", true));
        assert!(!TagSet::Five.found_in("[standard]", true));
        assert!(!TagSet::CodeMeasuredDecision.found_in("[memory]", false));
        assert!(TagSet::Five.found_in("[MEMORY]", false));
        assert!(!TagSet::Five.found_in("[MEMORY]", true));
        let t = "Intro [decision]\n\n| a |\n|---|\n";
        assert!(untagged(t, &TagPolicy::EVERY_BLOCK_004_09).is_empty());
        assert_eq!(untagged(t, &TagPolicy::EVERY_BLOCK_004_13).len(), 1);
    }

    #[test]
    fn word_counters() {
        assert_eq!(count_words("`snake_case` is it", WordCount::AlphaNoCode), 2);
        assert_eq!(count_words("a1 b2-c3", WordCount::Alnum), 3);
        assert_eq!(count_words("**a** `b`", WordCount::SplitBlanking("`*")), 2);
    }

    #[test]
    fn every_block_policy_lets_tables_inherit() {
        let ok = "Intro [decision]\n\n| a |\n|---|\n\n```\ncode\n```\n";
        assert!(untagged(ok, &TagPolicy::EVERY_BLOCK_004_09).is_empty());
        let bad = "Intro\n\n| a |\n|---|\n";
        assert_eq!(untagged(bad, &TagPolicy::EVERY_BLOCK_004_09).len(), 2);
        let subs = vec![
            ("A".to_string(), "x [code]".to_string()),
            ("B".to_string(), "y".to_string()),
        ];
        assert_eq!(subsections_untagged(&subs), vec!["B"]);
    }
}
