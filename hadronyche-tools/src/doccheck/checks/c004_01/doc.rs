//! The wire.md reading of `checks/004-01.sh`: headings outside fences, the
//! section of a heading, and the script's `parse_tables`.

use crate::doccheck::family004::heading_index;
use crate::doccheck::markdown::Markdown;
use crate::doccheck::numwin::window;
use crate::doccheck::policy::TableRules;
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::rx;

pub(crate) use crate::doccheck::family004::Head;

/// The document.
pub(crate) struct Doc<'a> {
    pub lines: Vec<&'a str>,
    pub heads: Vec<Head>,
}

/// A table: (start line, header cells, rows).
pub(crate) type Tab = (usize, Vec<String>, Vec<Vec<String>>);

impl<'a> Doc<'a> {
    /// The lines of `text` (`splitlines`) and `heading_index` of the 004
    /// family (headings outside `^\s*```` fences).
    pub fn new(text: &'a str) -> Doc<'a> {
        let lines = splitlines(text);
        let heads = heading_index(&lines);
        Doc { lines, heads }
    }

    /// `section_of(i)`: the lines after the heading to the next heading of
    /// the same or a higher level.
    pub fn section(&self, i: usize) -> String {
        let (line, level, _) = &self.heads[i];
        let end = self.heads[i + 1..]
            .iter()
            .find(|h| h.1 <= *level)
            .map_or(self.lines.len(), |h| h.0);
        self.lines[(line + 1).min(end)..end].join("\n")
    }

    /// `full_section(i)`: the title, a newline and the section.
    pub fn full(&self, i: usize) -> String {
        format!("{}\n{}", self.heads[i].2, self.section(i))
    }

    /// `section_for_title`: the first heading whose title satisfies `pred`.
    pub fn find(&self, pred: impl Fn(&str) -> bool) -> Option<usize> {
        self.heads.iter().position(|h| pred(&h.2))
    }
}

/// `parse_tables(text)`: the 004 family's rules ([`TableRules::FAMILY_004`]
/// over the fence mask `^\s*```` toggles).
pub(crate) fn parse_tables(text: &str) -> Vec<Tab> {
    let md = Markdown::parse(text);
    md.tables_with(md.all(), TableRules::FAMILY_004)
        .into_iter()
        .filter_map(|t| {
            Some((
                t.line,
                t.header?,
                t.rows.into_iter().map(|r| r.cells).collect(),
            ))
        })
        .collect()
}

/// `norm_cell`: parentheses removed, lower case, only `a-z0-9`.
pub(crate) fn norm_cell(s: &str) -> String {
    let t = rx(r"\([^)]*\)").replace_all(s, "").to_lowercase();
    t.chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .collect()
}

/// `nearby(text, a, b, window)`: a match of `a` with a match of `b` within
/// `window` characters of it (both ignoring case).
pub(crate) fn nearby(text: &str, a: &str, b: &str, width: usize) -> bool {
    let rb = rx(&format!("(?i){b}"));
    rx(&format!("(?i){a}"))
        .find_iter(text)
        .any(|m| rb.is_match(window(text, m.start(), m.end(), width, width)))
}
