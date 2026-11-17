//! Open items, status sentences and labelled paragraphs ("**Reported
//! contradiction: ...**", "**Hand-on: ...**").
//!
//! Like the tag rule, the open-item rule is written many ways by the
//! scripts; each is an explicit [`OpenItemPolicy`] (inventory section 4c
//! lists about fifteen). A policy fixes what a candidate block is, what makes
//! it an open item, and which words it must hold (and whether they are
//! whole words):
//!
//! | Policy | Scripts | Blocks | An open item when | Must hold |
//! |---|---|---|---|---|
//! | [`OpenItemPolicy::Mention`] | 004-22 | [`units`] paragraphs and list items | "open item" anywhere | owner, falsifier |
//! | [`OpenItemPolicy::LeadingWithTicket`] | 004-27 | [`prose_blocks`] prose | `^\W*open\s+item` after `clean_start` | owner, falsifier, `\d{3}-\d{2}` or `\bepic\b` |
//! | [`OpenItemPolicy::Leading`] | 004-29 | [`prose_blocks`] prose | `^\s*\**\s*Open item` (case-sensitive) | owner, falsifier |
//! | [`OpenItemPolicy::MentionBlocks`] | 004-24, 004-25, 004-28 | [`prose_blocks`] of BOTH kinds | "open item" anywhere | owner, falsifier |
//! | [`OpenItemPolicy::LeadingClean`] | 004-16, 004-17, 004-18 | [`prose_blocks`] prose | `^\s*\*{0,2}\s*open\s+items?\b` after `clean_start` | owner, falsifier |
//! | [`OpenItemPolicy::Head40`] | 004-26 | [`prose_blocks`] prose | "open item" in the first 40 characters after `*`, `_`, backticks are removed and the text stripped | `\bowner\b`, `\bfalsifier\b` |
//! | [`OpenItemPolicy::Window3`] | 004-19 | every `section_blocks` block (headings too) | `open\s+item` anywhere | owner, falsifier in the block and the two after it |
//! | [`OpenItemPolicy::WordLeading`] | 004-20 | `section_blocks` of role prose | `^\W*open\s+items?\b` after strip | `\bowner\b`, falsifier |
//!
//! "Any case" throughout, except where noted. The 004 policies are proven
//! against the scripts' inline code on `data/family004_corpus.txt`
//! (`doccheck::proofs004`).

use super::pycompat::PyStr;
use super::text::{has_i, rx};
use super::units::{BlockKind, UnitKind, clean_start, prose_blocks, units};

/// How a check finds open items and what it requires of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenItemPolicy {
    /// 004-22: any paragraph or list item ([`units`]) that mentions "open
    /// item" anywhere must name an owner and a falsifier.
    Mention,
    /// 004-27: a prose block ([`prose_blocks`]) that starts with "Open item"
    /// (after leading markup) must name an owner, a ticket (`NNN-NN`) or an
    /// epic, and a falsifier.
    LeadingWithTicket,
    /// 004-29: a prose block that starts with optional `**` and "Open item"
    /// must name an owner and a falsifier.
    Leading,
    /// 004-24, 004-25, 004-28: any prose OR table block that mentions
    /// "open item".
    MentionBlocks,
    /// 004-16 to 004-18: a prose block starting (after `clean_start`) with
    /// optional `**` and "open item(s)" as a word.
    LeadingClean,
    /// 004-26: a prose block with "open item" in its first 40 characters
    /// once `*`, `_` and backticks are removed; owner and falsifier as whole
    /// words.
    Head40,
    /// 004-19: any `section_blocks` block mentioning `open\s+item`; owner and
    /// falsifier may be in it or in the two blocks after it.
    Window3,
    /// 004-20: a `section_blocks` prose block starting (after strip and
    /// non-word characters) with "open item(s)" as a word; owner as a whole
    /// word.
    WordLeading,
}

/// An open item: its first line (0-based, relative to the text given), its
/// text, and the text searched for its owner and falsifier (the item itself,
/// except under [`OpenItemPolicy::Window3`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenItem {
    pub line: usize,
    pub text: String,
    pub scope: String,
}

impl OpenItem {
    fn new(line: usize, text: String) -> OpenItem {
        OpenItem {
            line,
            scope: text.clone(),
            text,
        }
    }
}

fn section_block_items(text: &str) -> Vec<(usize, Vec<String>)> {
    super::blocks::section_blocks_at(&super::pycompat::splitlines(text))
}

/// The open items of `text` under `rule`.
pub fn open_items(text: &str, rule: OpenItemPolicy) -> Vec<OpenItem> {
    let prose = || {
        prose_blocks(text)
            .into_iter()
            .filter(|b| b.kind == BlockKind::Prose)
    };
    match rule {
        OpenItemPolicy::Mention => units(text)
            .into_iter()
            .filter(|u| matches!(u.kind, UnitKind::Paragraph | UnitKind::ListItem))
            .filter(|u| has_i("open item", &u.text))
            .map(|u| OpenItem::new(u.lo, u.text))
            .collect(),
        OpenItemPolicy::LeadingWithTicket => prose()
            .map(|b| OpenItem::new(b.line, clean_start(&b.text)))
            .filter(|b| rx(r"(?i)^\W*open\s+item").is_match(&b.text))
            .collect(),
        OpenItemPolicy::Leading => prose()
            .filter(|b| rx(r"^\s*\**\s*Open item").is_match(&b.text))
            .map(|b| OpenItem::new(b.line, b.text))
            .collect(),
        OpenItemPolicy::MentionBlocks => prose_blocks(text)
            .into_iter()
            .filter(|b| has_i("open item", &b.text))
            .map(|b| OpenItem::new(b.line, b.text))
            .collect(),
        OpenItemPolicy::LeadingClean => prose()
            .filter(|b| rx(r"(?i)^\s*\*{0,2}\s*open\s+items?\b").is_match(&clean_start(&b.text)))
            .map(|b| OpenItem::new(b.line, b.text))
            .collect(),
        OpenItemPolicy::Head40 => prose()
            .filter(|b| {
                let bare = rx("[*_`]").replace_all(&b.text, "");
                let head: String = bare.py_strip().chars().take(40).collect();
                has_i("open item", &head)
            })
            .map(|b| OpenItem::new(b.line, b.text))
            .collect(),
        OpenItemPolicy::Window3 => {
            let blocks: Vec<(usize, String)> = section_block_items(text)
                .into_iter()
                .map(|(l, b)| (l, b.join("\n")))
                .collect();
            let flat: Vec<&str> = blocks.iter().map(|(_, b)| b.as_str()).collect();
            blocks
                .iter()
                .enumerate()
                .filter(|(_, (_, b))| has_i(r"open\s+item", b))
                .map(|(i, (l, b))| OpenItem {
                    line: *l,
                    text: b.clone(),
                    scope: flat[i..(i + 3).min(flat.len())].join("\n"),
                })
                .collect()
        }
        OpenItemPolicy::WordLeading => section_block_items(text)
            .into_iter()
            .filter(|(_, b)| super::blocks::block_role(b) == super::blocks::BlockRole::Prose)
            .map(|(l, b)| OpenItem::new(l, b.join("\n")))
            .filter(|i| rx(r"(?i)^\W*open\s+items?\b").is_match(i.text.py_strip()))
            .collect(),
    }
}

/// True when the item satisfies `rule` (owner, falsifier, and for
/// [`OpenItemPolicy::LeadingWithTicket`] a ticket or an epic).
pub fn item_is_owned(item: &OpenItem, rule: OpenItemPolicy) -> bool {
    let t = &item.scope;
    match rule {
        OpenItemPolicy::LeadingWithTicket => {
            has_i("owner", t) && has_i("falsifier", t) && has_i(r"\d{3}-\d{2}|\bepic\b", t)
        }
        OpenItemPolicy::Head40 => has_i(r"\bowner\b", t) && has_i(r"\bfalsifier\b", t),
        OpenItemPolicy::WordLeading => has_i(r"\bowner\b", t) && has_i("falsifier", t),
        _ => has_i("owner", t) && has_i("falsifier", t),
    }
}

/// The open items of `text` that break `rule`.
pub fn unowned_items(text: &str, rule: OpenItemPolicy) -> Vec<OpenItem> {
    open_items(text, rule)
        .into_iter()
        .filter(|i| !item_is_owned(i, rule))
        .collect()
}

/// A labelled paragraph: `**Label: title.** body` or `**Label N.** body`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labelled {
    pub line: usize,
    /// The bold lead without the label (`the media packet's version range`).
    pub title: String,
    /// The whole paragraph.
    pub text: String,
}

/// The paragraphs of `text` that open with a bold `label` (a regex, matched
/// case-insensitively), e.g. `Reported contradiction`, `Hand-on`,
/// `Open item \d+`.
pub fn labelled(text: &str, label: &str) -> Vec<Labelled> {
    let lead = rx(&format!(
        r"(?i)^\s*\*\*\s*(?:{label})\s*[:.]?\s*([^*]*)\*\*"
    ));
    prose_blocks(text)
        .into_iter()
        .filter(|b| b.kind == BlockKind::Prose)
        .filter_map(|b| {
            let title = lead.captures(&b.text)?[1]
                .py_strip()
                .trim_end_matches('.')
                .to_string();
            Some(Labelled {
                line: b.line,
                title,
                text: b.text,
            })
        })
        .collect()
}

/// The sentences of `text` that state a status (`Status: ...`, "its status
/// is ..."), split at `.` or `;` followed by white space.
pub fn status_sentences(text: &str) -> Vec<String> {
    rx(r"[.;]\s+|\n\s*\n")
        .split(text)
        .filter(|s| has_i(r"\bstatus\b", s))
        .map(|s| s.py_strip().to_string())
        .collect()
}

/// True when a status sentence of `text` names something matching `what`.
pub fn status_names(text: &str, what: &str) -> bool {
    status_sentences(text).iter().any(|s| has_i(what, s))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "Para.\n\n**Open item 3.** Owner: 004-23. Falsifier: a run.\n\n**Open item 4.** Owner: design owner.\n\nThis mentions an open item without owner.\n\n**Reported contradiction: the version range.** Text. [decision]\n\n**Hand-on: the rules.** Handed on.\n";

    #[test]
    fn open_item_rules_differ() {
        assert_eq!(open_items(TEXT, OpenItemPolicy::Mention).len(), 3);
        assert_eq!(open_items(TEXT, OpenItemPolicy::LeadingWithTicket).len(), 2);
        assert_eq!(open_items(TEXT, OpenItemPolicy::Leading).len(), 2);
        let bad = unowned_items(TEXT, OpenItemPolicy::LeadingWithTicket);
        assert_eq!(bad.len(), 1);
        assert_eq!(bad[0].line, 4);
        assert_eq!(unowned_items(TEXT, OpenItemPolicy::Mention).len(), 2);
    }

    #[test]
    fn owned_needs_ticket_only_for_that_rule() {
        let item = OpenItem::new(0, "Open item. Owner: the design owner; falsifier: x".into());
        assert!(item_is_owned(&item, OpenItemPolicy::Leading));
        assert!(!item_is_owned(&item, OpenItemPolicy::LeadingWithTicket));
    }

    #[test]
    fn the_004_policies() {
        let t = "**Open item 1:** owners and falsifiers.\n\n| open item | owner falsifier |\n\n> Open item 2 no owner.\n\nText.\n\nOwner: x, falsifier: y.";
        assert_eq!(open_items(t, OpenItemPolicy::MentionBlocks).len(), 3);
        assert_eq!(open_items(t, OpenItemPolicy::LeadingClean).len(), 2);
        assert_eq!(open_items(t, OpenItemPolicy::Head40).len(), 2);
        assert_eq!(
            unowned_items(t, OpenItemPolicy::Head40).len(),
            2,
            "owners is not the word owner"
        );
        assert_eq!(unowned_items(t, OpenItemPolicy::LeadingClean).len(), 1);
        let w = open_items(t, OpenItemPolicy::Window3);
        assert_eq!(w.len(), 3);
        assert!(
            item_is_owned(&w[2], OpenItemPolicy::Window3),
            "owner two blocks later"
        );
        assert_eq!(open_items(t, OpenItemPolicy::WordLeading).len(), 2);
    }

    #[test]
    fn labelled_paragraphs() {
        let rc = labelled(TEXT, "Reported contradiction");
        assert_eq!(rc.len(), 1);
        assert_eq!(rc[0].title, "the version range");
        assert_eq!(labelled(TEXT, "Hand-on")[0].title, "the rules");
        assert_eq!(labelled(TEXT, r"Open item \d+").len(), 2);
    }

    #[test]
    fn status_sentences_are_found() {
        let t = "Open item 11. Status: decided by Commissioning window messages (004-29). Other.";
        assert_eq!(status_sentences(t).len(), 1);
        assert!(status_names(t, "commissioning window messages"));
        assert!(!status_names("No such sentence.", "x"));
    }
}
