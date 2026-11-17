//! The Size budget section of wire.md: its lines, its tables with their
//! captions, and the lines outside it.

use crate::doccheck::family004::heading_index;
use crate::doccheck::markdown::code_mask;
use crate::doccheck::policy::{FencePolicy, TableRules};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::tables::{caption_above, cell_label, cell_number_004_14, parse_tables};
use crate::doccheck::text::{full, has, rx};

/// One Markdown table of the section, with the caption lines above it.
#[derive(Debug, Clone)]
pub(crate) struct STable {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub caption: String,
}

/// The document split the way the script splits it.
#[derive(Debug)]
pub(crate) struct Sec {
    pub found: bool,
    pub sec_lines: Vec<String>,
    pub sec_fence: Vec<bool>,
    pub glob_lines: Vec<String>,
    pub sec_text: String,
    pub tables: Vec<STable>,
}

/// `norm`: whitespace runs squashed to one space (not trimmed).
pub(crate) fn norm(s: &str) -> String {
    rx(r"\s+").replace_all(s, " ")
}

impl Sec {
    /// Reads the section from the text of wire.md.
    pub fn read(wire: &str) -> Sec {
        let lines: Vec<&str> = splitlines(wire);
        let heads = heading_index(&lines);
        let Some(k) = heads
            .iter()
            .position(|h| h.1 == 2 && full(r"(?i)size budget", &h.2))
        else {
            return Sec {
                found: false,
                sec_lines: Vec::new(),
                sec_fence: Vec::new(),
                glob_lines: lines.iter().map(|l| l.to_string()).collect(),
                sec_text: String::new(),
                tables: Vec::new(),
            };
        };
        let start = heads[k].0;
        let end = heads[k + 1..]
            .iter()
            .find(|h| h.1 <= 2)
            .map_or(lines.len(), |h| h.0);
        let sec: Vec<String> = lines[start..end].iter().map(|l| l.to_string()).collect();
        let fence = code_mask(&sec, FencePolicy::Backticks);
        let text = norm(
            &sec.iter()
                .zip(&fence)
                .filter(|(_, f)| !**f)
                .map(|(l, _)| l.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        );
        let refs: Vec<&str> = sec.iter().map(String::as_str).collect();
        let tables = parse_tables(&sec, &fence, 0, sec.len(), TableRules::FAMILY_004)
            .into_iter()
            .filter_map(|t| {
                Some(STable {
                    caption: caption_above(&refs, &fence, t.line),
                    header: t.header?,
                    rows: t.rows.into_iter().map(|r| r.cells).collect(),
                })
            })
            .collect();
        let glob = lines[..start]
            .iter()
            .chain(&lines[end..])
            .map(|l| l.to_string())
            .collect();
        Sec {
            found: true,
            sec_lines: sec,
            sec_fence: fence,
            glob_lines: glob,
            sec_text: text,
            tables,
        }
    }
}

/// `row_kind`: what a budget row's label names.
pub(crate) fn row_kind(lab: &str) -> &'static str {
    let has_ = |p: &str| has(p, lab);
    if lab.starts_with("total") {
        "total"
    } else if has_(r"\b(spare|margin|headroom|slack|deficit)\b") {
        "spare"
    } else if lab.contains("packet header") {
        "pkt"
    } else if lab.contains("slice header") {
        "slice"
    } else if lab.contains("announce") {
        "ann"
    } else if has_(r"content[- ]message|content header") {
        "content"
    } else if lab.contains("audio") {
        "audio"
    } else if lab.contains("checksum") || has_(r"\bcrc\b") {
        "cksum"
    } else if lab.contains("measurement") || lab.contains("extension") {
        "ext"
    } else if has_(r"controller|dense|sparse|transport") {
        "ctrl"
    } else {
        "other"
    }
}

/// `classify_case`.
pub(crate) fn classify_case(text: &str) -> Option<&'static str> {
    let has_ = |p: &str| has(p, text);
    if has_(r"(?i)fallback|\b1\s?280\b|\b1\s?252\b") {
        Some("fb")
    } else if has_(r"(?i)worst|maximum|maximal|all optional") {
        Some("worst")
    } else if has_(r"(?i)default|mandatory|\b1\s?472\b") {
        Some("default")
    } else {
        None
    }
}

/// The label of a row's first cell.
pub(crate) fn row_label(row: &[String]) -> String {
    cell_label(row.first().map_or("", String::as_str))
}

/// The number in column `c` of a row (a short row reads 0, `parse_num(None)`);
/// `None` when the cell holds no number.
pub(crate) fn cell_num(row: &[String], c: usize) -> Option<i128> {
    cell_number_004_14(row.get(c).map_or("", String::as_str))
        .ok()
        .flatten()
}

/// `re.split(r"(?<=[.;])\s+", text)`.
pub(crate) fn sentences(text: &str) -> Vec<String> {
    rx(r"(?<=[.;])\s+").split(text).map(String::from).collect()
}
