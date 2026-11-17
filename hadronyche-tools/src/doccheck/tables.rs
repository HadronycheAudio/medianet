//! Markdown tables as typed rows with header lookup.
//!
//! A table is a run of consecutive lines outside code that start with `|`
//! (after leading whitespace). When its second line is a separator row
//! (`| --- | :-: |`), the first line is the header and the data rows follow
//! the separator; otherwise the run has no header and every line is a row.
//! Cells are split on `|` not preceded by `\`, trimmed, `\|` unescaped.

use super::markdown::norm_title;
use super::policy::{HeaderlessRuns, RowSplit, SeparatorRule, TableRules};
use super::pycompat::{IntError, PyStr, parse_int};
use super::text::{full, rx};

/// One data row of a table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// 0-based line index in the document.
    pub line: usize,
    pub cells: Vec<String>,
}

impl Row {
    /// The cell at `k`, or `""` when the row is shorter.
    pub fn cell(&self, k: usize) -> &str {
        self.cells.get(k).map_or("", String::as_str)
    }

    /// The cells joined with `" | "` (how the scripts search a whole row).
    pub fn joined(&self) -> String {
        self.cells.join(" | ")
    }

    /// The cells after the first, joined with `" | "` and trimmed.
    pub fn rest(&self) -> String {
        self.cells
            .get(1..)
            .map(|c| c.join(" | "))
            .unwrap_or_default()
            .py_strip()
            .to_string()
    }
}

/// One table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// 0-based index of the table's first line.
    pub line: usize,
    /// The header cells (as written), when the table has a separator row.
    pub header: Option<Vec<String>>,
    /// The data rows (all lines when there is no header).
    pub rows: Vec<Row>,
}

impl Table {
    /// The header cells, or an empty slice.
    pub fn head(&self) -> &[String] {
        self.header.as_deref().unwrap_or(&[])
    }

    /// The header normalised with [`norm_title`].
    pub fn head_norm(&self) -> Vec<String> {
        self.head().iter().map(|c| norm_title(c)).collect()
    }

    /// The first column whose header matches `pattern` (a regex).
    pub fn col(&self, pattern: &str) -> Option<usize> {
        let re = rx(pattern);
        self.head().iter().position(|h| re.is_match(h))
    }

    /// The column whose normalised header equals `name`.
    pub fn col_named(&self, name: &str) -> Option<usize> {
        self.head_norm().iter().position(|h| h == name)
    }
}

/// Splits a table line into trimmed cells under `split`.
pub fn split_row_with(line: &str, split: RowSplit) -> Vec<String> {
    match split {
        RowSplit::Escaped => split_row(line),
        RowSplit::CharWalk => split_row_char_walk(line).unwrap_or_default(),
        RowSplit::Plain => {
            let s = line.py_strip();
            let s = s.strip_prefix('|').unwrap_or(s);
            let s = s.strip_suffix('|').unwrap_or(s);
            s.split('|').map(|c| c.py_strip().to_string()).collect()
        }
    }
}

/// Splits a table line into trimmed cells ([`RowSplit::Escaped`]).
pub fn split_row(line: &str) -> Vec<String> {
    let s = line.py_strip();
    let s = s.strip_prefix('|').unwrap_or(s);
    let s = s.strip_suffix('|').unwrap_or(s);
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut prev = '\0';
    for c in s.chars() {
        if c == '|' && prev != '\\' {
            cells.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
        prev = c;
    }
    cells.push(cur);
    cells
        .into_iter()
        .map(|c| c.py_strip().replace("\\|", "|"))
        .collect()
}

/// 006-01/006-02's `split_row` ([`RowSplit::CharWalk`]): `None` when the
/// stripped line does not start with `|`.
pub fn split_row_char_walk(line: &str) -> Option<Vec<String>> {
    let s = line.py_strip();
    let body = s.strip_prefix('|')?;
    let s = if s.ends_with('|') && !s.ends_with("\\|") && !body.is_empty() {
        &body[..body.len() - 1]
    } else {
        body
    };
    let chars: Vec<char> = s.chars().collect();
    let (mut cells, mut cur, mut i) = (Vec::new(), String::new(), 0);
    while i < chars.len() {
        match chars[i] {
            '\\' if chars.get(i + 1) == Some(&'|') => {
                cur.push('|');
                i += 1;
            }
            '|' => cells.push(std::mem::take(&mut cur).py_strip().to_string()),
            c => cur.push(c),
        }
        i += 1;
    }
    cells.push(cur.py_strip().to_string());
    Some(cells)
}

/// 006-02's separator cells ([`SeparatorRule::DashCells`]): at least one
/// cell, each `:?-{2,}:?` once spaces are removed.
pub fn is_dash_cells(cells: &[String]) -> bool {
    !cells.is_empty() && cells.iter().all(|c| full(":?-{2,}:?", &c.replace(' ', "")))
}

/// 004-14/004-15's `caption_above(ls, fm, i)`: up to two lines within the
/// five above line `i` that are not blank, not a table line and not in
/// code (`code` is the fence mask), each stripped of leading `#`, `*` and
/// spaces, nearest last, joined with a space.
pub fn caption_above(lines: &[&str], code: &[bool], i: usize) -> String {
    let mut got: Vec<String> = Vec::new();
    for k in (i.saturating_sub(5)..i).rev() {
        let s = lines[k].py_strip();
        if !s.is_empty() && !lines[k].py_lstrip().starts_with('|') && !code[k] {
            got.push(s.trim_start_matches(['#', '*', ' ']).py_strip().to_string());
            if got.len() == 2 {
                break;
            }
        }
    }
    got.reverse();
    got.join(" ")
}

/// 004-14's `label_of(cell)`: `*`, `_` and `` ` `` removed, stripped, lower
/// case.
pub fn cell_label(cell: &str) -> String {
    rx(r"[*_`]").replace_all(cell, "").py_strip().to_lowercase()
}

/// 004-14's `NUM_RE`: a number with digit groups (space, no-break space or
/// narrow no-break space) or plain digits.
pub const NUM_004_14: &str = r"\d{1,3}(?:[ \x{a0}\x{202f}]\d{3})+|\d+";

fn int_004_14(m: &str) -> Result<i128, IntError> {
    parse_int(&rx(r"[ \x{a0}\x{202f},]").replace_all(m, ""))
}

/// 004-14's `first_int(text)`: the first [`NUM_004_14`] match as an `int`
/// (separators and commas removed); `Ok(None)` when there is none. Python's
/// `int` is unbounded: past `i128` the result is `IntError::Overflow`.
pub fn first_int_004_14(text: &str) -> Result<Option<i128>, IntError> {
    rx(NUM_004_14)
        .find(text)
        .map(|m| int_004_14(m.as_str()))
        .transpose()
}

/// 004-14's dash cells (`DASHES`): a cell that reads 0.
pub const DASHES_004_14: &[&str] = &[
    "", "\u{2014}", "\u{2013}", "-", "none", "omitted", "absent", "n/a", "not sent",
];

/// 004-14's `parse_num(cell)`: a dash cell is 0; else the text after the
/// last `=`, without `*` and parenthesised notes, its first number, negated
/// when a minus sign, a `-` that no word character precedes, or `deficit`
/// is there; `Ok(None)` when there is no number.
pub fn cell_number_004_14(cell: &str) -> Result<Option<i128>, IntError> {
    let raw = cell.py_strip();
    if DASHES_004_14.contains(&cell_label(raw).as_str()) {
        return Ok(Some(0));
    }
    let s = raw.replace('*', "");
    let s = s.rsplit('=').next().unwrap_or_default();
    let s = rx(r"\([^)]*\)").replace_all(s, "");
    let Some(m) = rx(NUM_004_14).find(&s) else {
        return Ok(None);
    };
    let value = int_004_14(m.as_str())?;
    let negative =
        crate::doccheck::text::rx(r"(?i)[\u{2212}]\s*\d|(?<![\w])-\s*\d|deficit").is_match(&s);
    Ok(Some(if negative { -value } else { value }))
}

/// True when every non-empty cell is `:?-+:?` and at least one is not empty.
pub fn is_separator(cells: &[String]) -> bool {
    cells.iter().any(|c| !c.is_empty())
        && cells
            .iter()
            .filter(|c| !c.is_empty())
            .all(|c| full(":?-+:?", c))
}

/// The 004 family's separator line: `^\s*\|?[\s:|-]+\|?\s*$` holding a `-`.
pub fn is_separator_line(line: &str) -> bool {
    rx(r"^\s*\|?[\s:|-]+\|?\s*$").is_match(line) && line.contains('-')
}

/// The tables of `lines[lo..hi]`, given the code mask of the document.
pub fn parse_tables(
    lines: &[String],
    in_code: &[bool],
    lo: usize,
    hi: usize,
    rules: TableRules,
) -> Vec<Table> {
    let is_row = |i: usize| !in_code[i] && lines[i].py_lstrip().starts_with('|');
    let mut out = Vec::new();
    let mut i = lo;
    while i < hi {
        if !is_row(i) {
            i += 1;
            continue;
        }
        let start = i;
        while i < hi && is_row(i) {
            i += 1;
        }
        if let Some(t) = build(lines, start, i, rules) {
            out.push(t);
        }
    }
    out
}

fn build(lines: &[String], start: usize, end: usize, rules: TableRules) -> Option<Table> {
    let split = rules.split;
    let row = |k: usize| Row {
        line: k,
        cells: split_row_with(&lines[k], split),
    };
    let headed = end - start > 1
        && match rules.separator {
            SeparatorRule::CellWise => is_separator(&split_row_with(&lines[start + 1], split)),
            SeparatorRule::LineRegex => is_separator_line(&lines[start + 1]),
            SeparatorRule::DashCells => {
                end - start > 2 && is_dash_cells(&split_row_with(&lines[start + 1], split))
            }
        };
    if headed {
        return Some(Table {
            line: start,
            header: Some(split_row_with(&lines[start], split)),
            rows: (start + 2..end).map(row).collect(),
        });
    }
    (rules.headerless == HeaderlessRuns::Keep).then(|| Table {
        line: start,
        header: None,
        rows: (start..end).map(row).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::markdown::Markdown;
    use super::*;

    #[test]
    fn split_row_honours_escaped_pipes() {
        assert_eq!(split_row("| a | b \\| c |"), vec!["a", "b | c"]);
        assert_eq!(split_row("a|b"), vec!["a", "b"]);
        assert_eq!(split_row("| a | |"), vec!["a", ""]);
        assert_eq!(
            split_row_with("| a | b \\| c |", RowSplit::Plain),
            vec!["a", "b \\", "c"]
        );
    }

    #[test]
    fn separator_rows() {
        assert!(is_separator(&split_row("|---|:--:|")));
        assert!(!is_separator(&split_row("| a | --- |")));
        assert!(!is_separator(&split_row("| | |")));
    }

    #[test]
    fn tables_with_and_without_header_skip_code() {
        let md = Markdown::parse(
            "| Field | Width |\n|---|---|\n| `a` | 8 |\n| `b` | 16 |\n\n| x | y |\n\n```\n| in | code |\n```\n",
        );
        let t = md.tables(md.all());
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].head(), ["Field", "Width"]);
        assert_eq!(t[0].rows.len(), 2);
        assert_eq!(t[0].rows[1].line, 3);
        assert_eq!(t[0].col("(?i)width"), Some(1));
        assert_eq!(t[0].col_named("field"), Some(0));
        assert!(t[1].header.is_none());
        assert_eq!(t[1].rows[0].cells, vec!["x", "y"]);
        assert_eq!(t[0].rows[0].rest(), "8");
        assert_eq!(t[0].rows[0].cell(9), "");
    }
}
