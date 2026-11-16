//! Units: the paragraphs, list items, table rows and headings of a section,
//! each with its line numbers; and the coarser prose/table blocks some
//! checks use.
//!
//! Two segmentations exist because the scripts use two:
//!
//! * [`units`] — a blank line, a table row or a heading ends a paragraph, and
//!   a list item marker (`-`, `*`, `1.`) starts a new unit even without a
//!   blank line. Table rows and headings are units of their own. This is the
//!   segmentation of `checks/004-22.sh` (its DoD tag rule).
//! * [`prose_blocks`] — a blank line or a heading ends a block, and a block
//!   is either prose (`p`) or table (`t`): a change of kind ends it. A list
//!   is one prose block. This is `prose_blocks` of `checks/004-27.sh` and
//!   `checks/004-29.sh`.
//!
//! Fenced code is excluded from both.

use super::text::{rx, split_nl, splitlines};

/// What a unit is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    Paragraph,
    ListItem,
    TableRow,
    Heading,
}

/// One unit of a section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    pub kind: UnitKind,
    /// 0-based index (relative to the text given) of the first line.
    pub lo: usize,
    /// 0-based index of the last line (inclusive).
    pub hi: usize,
    /// The lines, each trimmed, joined with one space.
    pub text: String,
}

fn is_list_marker(line: &str) -> bool {
    rx(r"^\s*([-*]|\d+\.)\s+").is_match(line)
}

struct Acc {
    out: Vec<Unit>,
    cur: Vec<(usize, String)>,
}

impl Acc {
    fn flush(&mut self) {
        if self.cur.is_empty() {
            return;
        }
        let cur = std::mem::take(&mut self.cur);
        let kind = if is_list_marker(&cur[0].1) {
            UnitKind::ListItem
        } else {
            UnitKind::Paragraph
        };
        let text = cur
            .iter()
            .map(|(_, l)| l.trim())
            .collect::<Vec<_>>()
            .join(" ");
        self.out.push(Unit {
            kind,
            lo: cur[0].0,
            hi: cur[cur.len() - 1].0,
            text,
        });
    }

    fn single(&mut self, kind: UnitKind, i: usize, line: &str) {
        self.out.push(Unit {
            kind,
            lo: i,
            hi: i,
            text: line.trim().to_string(),
        });
    }
}

/// The units of `text` (see the module documentation). A fence line toggles
/// code and, as in the script, does not end the paragraph before it.
pub fn units(text: &str) -> Vec<Unit> {
    let mut acc = Acc {
        out: Vec::new(),
        cur: Vec::new(),
    };
    let mut in_code = false;
    for (i, line) in split_nl(text).into_iter().enumerate() {
        let s = line.trim();
        if s.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if s.is_empty() {
            acc.flush();
        } else if s.starts_with('|') || s.starts_with('#') {
            acc.flush();
            let kind = if s.starts_with('|') {
                UnitKind::TableRow
            } else {
                UnitKind::Heading
            };
            acc.single(kind, i, line);
        } else {
            if is_list_marker(line) {
                acc.flush();
            }
            acc.cur.push((i, line.to_string()));
        }
    }
    acc.flush();
    acc.out
}

/// The kind of a [`Block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Prose,
    Table,
}

/// A prose or table block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    /// 0-based index of the first line, relative to the text given.
    pub line: usize,
    /// The lines as written, joined with `\n`.
    pub text: String,
}

/// `text` with fenced code (fence lines included) blanked, line for line.
pub fn strip_fences(text: &str) -> String {
    let mut in_fence = false;
    let mut out = Vec::new();
    for line in splitlines(text) {
        if rx(r"^\s*```").is_match(line) {
            in_fence = !in_fence;
            out.push("");
            continue;
        }
        out.push(if in_fence { "" } else { line });
    }
    out.join("\n")
}

/// The prose and table blocks of `text` (see the module documentation).
pub fn prose_blocks(text: &str) -> Vec<Block> {
    let stripped = strip_fences(text);
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    let mut start = 0;
    let mut kind: Option<BlockKind> = None;
    let push = |out: &mut Vec<Block>, kind: Option<BlockKind>, start, cur: &mut Vec<&str>| {
        if let (Some(kind), false) = (kind, cur.is_empty()) {
            out.push(Block {
                kind,
                line: start,
                text: cur.join("\n"),
            });
        }
        cur.clear();
    };
    for (i, line) in splitlines(&stripped).into_iter().enumerate() {
        if rx(r"^#{1,6}\s").is_match(line) || line.trim().is_empty() {
            push(&mut out, kind, start, &mut cur);
            kind = None;
            continue;
        }
        let k = if line.trim_start().starts_with('|') {
            BlockKind::Table
        } else {
            BlockKind::Prose
        };
        if kind.is_some() && kind != Some(k) {
            push(&mut out, kind, start, &mut cur);
        }
        if cur.is_empty() {
            start = i;
        }
        kind = Some(k);
        cur.push(line);
    }
    push(&mut out, kind, start, &mut cur);
    out
}

/// A unit's text with leading markup (`*`, `_`, `` ` ``, `>`, `-`, spaces)
/// removed, as the scripts' `clean_start`.
pub fn clean_start(text: &str) -> String {
    rx(r"^[\s*_`>-]+").replace(text, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "Intro line one\nline two\n- item one\n  more\n- item two\n\n| a | b |\n|---|---|\n### Sub\n```\ncode here\n```\nafter\n";

    #[test]
    fn units_split_paragraphs_items_rows_and_headings() {
        let u = units(TEXT);
        let kinds: Vec<_> = u.iter().map(|u| u.kind).collect();
        use UnitKind::*;
        assert_eq!(
            kinds,
            vec![
                Paragraph, ListItem, ListItem, TableRow, TableRow, Heading, Paragraph
            ]
        );
        assert_eq!(u[0].text, "Intro line one line two");
        assert_eq!((u[0].lo, u[0].hi), (0, 1));
        assert_eq!(u[1].text, "- item one more");
        assert_eq!(u[6].text, "after");
        assert_eq!(u[6].lo, 12);
    }

    #[test]
    fn prose_blocks_keep_lists_together_and_split_on_kind() {
        let b = prose_blocks(TEXT);
        assert_eq!(b.len(), 3);
        assert_eq!(b[0].kind, BlockKind::Prose);
        assert!(b[0].text.contains("item two"));
        assert_eq!(b[1].kind, BlockKind::Table);
        assert_eq!(b[1].line, 6);
        assert_eq!(b[2].text, "after");
        let mixed = prose_blocks("text\n| a |\nmore");
        assert_eq!(mixed.len(), 3);
    }

    #[test]
    fn fences_are_blanked_line_for_line() {
        assert_eq!(strip_fences("a\n```\nb\n```\nc"), "a\n\n\n\nc");
        assert_eq!(clean_start("**Open item 3.** x"), "Open item 3.** x");
    }
}
