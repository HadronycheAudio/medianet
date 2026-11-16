//! Open items, status sentences and labelled paragraphs ("**Reported
//! contradiction: ...**", "**Hand-on: ...**").
//!
//! Like the tag rule, the open-item rule is written two ways by the scripts;
//! each is an explicit [`OpenItemPolicy`]
//! (inventory section 4c lists about fifteen; these three are implemented,
//! a port adds its own variant).

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
}

/// An open item: its first line (0-based, relative to the text given) and
/// its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenItem {
    pub line: usize,
    pub text: String,
}

/// The open items of `text` under `rule`.
pub fn open_items(text: &str, rule: OpenItemPolicy) -> Vec<OpenItem> {
    match rule {
        OpenItemPolicy::Mention => units(text)
            .into_iter()
            .filter(|u| matches!(u.kind, UnitKind::Paragraph | UnitKind::ListItem))
            .filter(|u| has_i("open item", &u.text))
            .map(|u| OpenItem {
                line: u.lo,
                text: u.text,
            })
            .collect(),
        OpenItemPolicy::LeadingWithTicket => prose_blocks(text)
            .into_iter()
            .filter(|b| b.kind == BlockKind::Prose)
            .map(|b| OpenItem {
                line: b.line,
                text: clean_start(&b.text),
            })
            .filter(|b| rx(r"(?i)^\W*open\s+item").is_match(&b.text))
            .collect(),
        OpenItemPolicy::Leading => prose_blocks(text)
            .into_iter()
            .filter(|b| b.kind == BlockKind::Prose)
            .filter(|b| rx(r"^\s*\**\s*Open item").is_match(&b.text))
            .map(|b| OpenItem {
                line: b.line,
                text: b.text,
            })
            .collect(),
    }
}

/// True when the item satisfies `rule` (owner, falsifier, and for
/// [`OpenItemPolicy::LeadingWithTicket`] a ticket or an epic).
pub fn item_is_owned(item: &OpenItem, rule: OpenItemPolicy) -> bool {
    let t = &item.text;
    let base = has_i("owner", t) && has_i("falsifier", t);
    match rule {
        OpenItemPolicy::LeadingWithTicket => base && has_i(r"\d{3}-\d{2}|\bepic\b", t),
        _ => base,
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
                .trim()
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
        .map(|s| s.trim().to_string())
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
        let item = OpenItem {
            line: 0,
            text: "Open item. Owner: the design owner; falsifier: x".into(),
        };
        assert!(item_is_owned(&item, OpenItemPolicy::Leading));
        assert!(!item_is_owned(&item, OpenItemPolicy::LeadingWithTicket));
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
