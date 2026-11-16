//! Markdown tables as typed rows with header lookup.
//!
//! A table is a run of consecutive lines outside code that start with `|`
//! (after leading whitespace). When its second line is a separator row
//! (`| --- | :-: |`), the first line is the header and the data rows follow
//! the separator; otherwise the run has no header and every line is a row.
//! Cells are split on `|` not preceded by `\`, trimmed, `\|` unescaped.

use super::markdown::norm_title;
use super::policy::RowSplit;
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
            .trim()
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
        RowSplit::Plain => {
            let s = line.trim();
            let s = s.strip_prefix('|').unwrap_or(s);
            let s = s.strip_suffix('|').unwrap_or(s);
            s.split('|').map(|c| c.trim().to_string()).collect()
        }
    }
}

/// Splits a table line into trimmed cells ([`RowSplit::Escaped`]).
pub fn split_row(line: &str) -> Vec<String> {
    let s = line.trim();
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
        .map(|c| c.trim().replace("\\|", "|"))
        .collect()
}

/// True when every non-empty cell is `:?-+:?` and at least one is not empty.
pub fn is_separator(cells: &[String]) -> bool {
    cells.iter().any(|c| !c.is_empty())
        && cells
            .iter()
            .filter(|c| !c.is_empty())
            .all(|c| full(":?-+:?", c))
}

/// The tables of `lines[lo..hi]`, given the code mask of the document.
pub fn parse_tables(
    lines: &[String],
    in_code: &[bool],
    lo: usize,
    hi: usize,
    split: RowSplit,
) -> Vec<Table> {
    let is_row = |i: usize| !in_code[i] && lines[i].trim_start().starts_with('|');
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
        out.push(build(lines, start, i, split));
    }
    out
}

fn build(lines: &[String], start: usize, end: usize, split: RowSplit) -> Table {
    let row = |k: usize| Row {
        line: k,
        cells: split_row_with(&lines[k], split),
    };
    let headed = end - start > 1 && is_separator(&split_row_with(&lines[start + 1], split));
    if headed {
        Table {
            line: start,
            header: Some(split_row_with(&lines[start], split)),
            rows: (start + 2..end).map(row).collect(),
        }
    } else {
        Table {
            line: start,
            header: None,
            rows: (start..end).map(row).collect(),
        }
    }
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
