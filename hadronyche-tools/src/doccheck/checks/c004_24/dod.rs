//! DoD (document part): evidence tags on every prose line and every
//! subsection, owned open items.
//!
//! 004-24's variants of the two disputed rules are not in the library
//! (inventory sections 4b and 4c), so they are named here, private to this
//! port until proposed:
//!
//! * [`LINES_004_24`] — the untagged-claim rule applied per LINE, not per
//!   paragraph: every line of the section outside fences that is not blank,
//!   not a heading (`#`) and not a table row (`|`) and has 5 or more words
//!   after `` `*_|>#- `` are blanked carries a tag (six tags, any case);
//! * [`MENTION_BLOCKS_004_24`] — an open item is any [`prose_blocks`] block,
//!   prose OR table, that contains "open item" (any case); it must contain
//!   "owner" and "falsifier" (any case). The library's
//!   `OpenItemPolicy::Mention` reads [`crate::doccheck::units::units`]
//!   (paragraphs and list items, table rows excluded), which is a different
//!   segmentation.

use super::stamp::Stamp;
use crate::doccheck::Report;
use crate::doccheck::evidence::{TagSet, WordCount, count_words, subsections_untagged};
use crate::doccheck::text::{has_i, head, splitlines};
use crate::doccheck::units::{prose_blocks, strip_fences};

/// A per-line untagged-claim rule.
pub(crate) struct LinePolicy {
    pub name: &'static str,
    /// Lines starting (after trimming) with one of these are skipped.
    pub skip_prefixes: &'static [char],
    pub words: WordCount,
    pub min_words: usize,
    pub set: TagSet,
    pub exact_case: bool,
}

/// 004-24:1103 (see the module documentation).
pub(crate) const LINES_004_24: LinePolicy = LinePolicy {
    name: "lines-004-24",
    skip_prefixes: &['#', '|'],
    words: WordCount::SplitBlanking("`*_|>#-"),
    min_words: 5,
    set: TagSet::Six,
    exact_case: false,
};

/// The lines of `text` that break `policy`, trimmed.
pub(crate) fn untagged_lines(text: &str, policy: &LinePolicy) -> Vec<String> {
    splitlines(&strip_fences(text))
        .into_iter()
        .map(str::trim)
        .filter(|t| !t.is_empty() && !t.starts_with(policy.skip_prefixes))
        .filter(|t| count_words(t, policy.words) >= policy.min_words && !policy.set.found_in(t, policy.exact_case))
        .map(str::to_string)
        .collect()
}

/// An open-item rule over [`prose_blocks`] of both kinds.
pub(crate) struct BlockItemPolicy {
    pub name: &'static str,
    /// The mention that makes a block an open item (regex, any case).
    pub mention: &'static str,
}

/// 004-24:1117 (see the module documentation).
pub(crate) const MENTION_BLOCKS_004_24: BlockItemPolicy = BlockItemPolicy {
    name: "mention-blocks-004-24",
    mention: "open item",
};

/// (open items, those without an owner and a falsifier).
pub(crate) fn block_items(text: &str, policy: &BlockItemPolicy) -> (usize, Vec<String>) {
    let items: Vec<String> = prose_blocks(text).into_iter().filter(|b| has_i(policy.mention, &b.text)).map(|b| b.text).collect();
    let bad = items.iter().filter(|t| !(has_i("owner", t) && has_i("falsifier", t))).map(|t| head(t, 50)).collect();
    (items.len(), bad)
}

/// The DoD of the regime stamp section.
pub(crate) fn dod(st: &Stamp, report: &mut Report) {
    let bad: Vec<String> = untagged_lines(&st.text, &LINES_004_24).iter().map(|t| head(t, 60)).collect();
    let first: Vec<&String> = bad.iter().take(3).collect();
    report.check("DoD", bad.is_empty(), format!("every prose paragraph of 5 or more words in the regime stamp section carries an evidence tag ({}); untagged: {first:?}", LINES_004_24.name));
    let subs = subsections_untagged(&st.subs);
    report.check("DoD", subs.is_empty(), format!("every subsection of the regime stamp section carries an evidence tag; untagged: {subs:?}"));
    let (n, unowned) = block_items(&st.text, &MENTION_BLOCKS_004_24);
    report.check("DoD", unowned.is_empty(), format!("every Open item paragraph of the regime stamp section names an owner and a falsifier ({n} found; {}); without: {unowned:?}", MENTION_BLOCKS_004_24.name));
}
