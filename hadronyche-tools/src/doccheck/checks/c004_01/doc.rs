//! The wire.md reading of `checks/004-01.sh`: headings outside fences, the
//! section of a heading, and the script's `parse_tables`.

use crate::doccheck::tables::split_row;
use crate::doccheck::text::{has, rx, splitlines};

/// A heading: (line index, level, title).
pub(crate) type Head = (usize, usize, String);

/// The document.
pub(crate) struct Doc<'a> {
    pub lines: Vec<&'a str>,
    pub heads: Vec<Head>,
}

/// A table: (start line, header cells, rows).
pub(crate) type Tab = (usize, Vec<String>, Vec<Vec<String>>);

impl<'a> Doc<'a> {
    /// Headings outside fences (`^\s*```` toggles a fence).
    pub fn new(text: &'a str) -> Doc<'a> {
        let lines = splitlines(text);
        let re = rx(r"^(#{1,6})\s+(.*?)\s*#*\s*$");
        let (mut fence, mut heads) = (false, Vec::new());
        for (i, l) in lines.iter().enumerate() {
            if has(r"^\s*```", l) {
                fence = !fence;
                continue;
            }
            if fence {
                continue;
            }
            if let Some(m) = re.captures(l) {
                heads.push((i, m[1].len(), m[2].trim().to_string()));
            }
        }
        Doc { lines, heads }
    }

    /// `section_of(i)`: the lines after the heading to the next heading of
    /// the same or a higher level.
    pub fn section(&self, i: usize) -> String {
        let (line, level, _) = &self.heads[i];
        let end = self.heads[i + 1..].iter().find(|h| h.1 <= *level).map_or(self.lines.len(), |h| h.0);
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

/// `parse_tables(text)`: a run of lines starting `|` (outside fences) whose
/// second line holds a dash and only `\s : | -`.
pub(crate) fn parse_tables(text: &str) -> Vec<Tab> {
    let lines = splitlines(text);
    let (mut out, mut fence, mut i) = (Vec::new(), false, 0);
    while i < lines.len() {
        let stripped = lines[i].trim_start();
        if stripped.starts_with("```") {
            fence = !fence;
            i += 1;
            continue;
        }
        if !fence && stripped.starts_with('|') {
            let mut j = i;
            while j < lines.len() && lines[j].trim_start().starts_with('|') {
                j += 1;
            }
            let block = &lines[i..j];
            if block.len() >= 2 && block[1].contains('-') && has(r"^\s*\|?[\s:|-]+\|?\s*$", block[1]) {
                out.push((i, split_row(block[0]), block[2..].iter().map(|r| split_row(r)).collect()));
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// `norm_cell`: parentheses removed, lower case, only `a-z0-9`.
pub(crate) fn norm_cell(s: &str) -> String {
    let t = rx(r"\([^)]*\)").replace_all(s, "").to_lowercase();
    t.chars().filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit()).collect()
}

/// `nearby(text, a, b, window)`: a match of `a` with a match of `b` within
/// `window` characters of it (both ignoring case).
pub(crate) fn nearby(text: &str, a: &str, b: &str, width: usize) -> bool {
    use crate::doccheck::checks::c004_15::num::window;
    let rb = rx(&format!("(?i){b}"));
    rx(&format!("(?i){a}")).find_iter(text).any(|m| rb.is_match(&window(text, m.start(), m.end(), width, width)))
}
