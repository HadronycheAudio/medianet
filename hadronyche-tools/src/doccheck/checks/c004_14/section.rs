//! The Size budget section of wire.md: its lines, its tables with their
//! captions, and the lines outside it.

use super::util::{label_of, norm, parse_num};
use crate::doccheck::markdown::Markdown;
use crate::doccheck::tables::split_row;
use crate::doccheck::text::{rx, splitlines};

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

fn is_table_line(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

/// `caption_above`: up to two non-table lines within five lines above.
fn caption_above(ls: &[&str], fm: &[bool], i: usize) -> String {
    let mut got: Vec<String> = Vec::new();
    let mut k = i as isize - 1;
    while k >= 0 && i as isize - k <= 5 {
        let line = ls[k as usize];
        let s = line.trim();
        if !s.is_empty() && !is_table_line(line) && !fm[k as usize] {
            got.push(s.trim_start_matches(['#', '*', ' ']).trim().to_string());
            if got.len() == 2 {
                break;
            }
        }
        k -= 1;
    }
    got.reverse();
    got.join(" ")
}

/// `section_tables`.
fn section_tables(ls: &[&str], fm: &[bool]) -> Vec<STable> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < ls.len() {
        if fm[i] || !is_table_line(ls[i]) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < ls.len() && !fm[j] && is_table_line(ls[j]) {
            j += 1;
        }
        let block = &ls[i..j];
        if block.len() >= 2
            && block[1].contains('-')
            && rx(r"^\s*\|?[\s:|-]+\|?\s*$").is_match(block[1])
        {
            out.push(STable {
                header: split_row(block[0]),
                rows: block[2..].iter().map(|r| split_row(r)).collect(),
                caption: caption_above(ls, fm, i),
            });
        }
        i = j;
    }
    out
}

impl Sec {
    /// Reads the section from the text of wire.md.
    pub fn read(wire: &str) -> Sec {
        let lines: Vec<&str> = splitlines(wire);
        let md = Markdown::parse(&lines.join("\n"));
        let at = md.heads.iter().position(|h| {
            h.level == 2 && rx(r"(?i)^(?:size budget)$").is_match(&h.text)
        });
        let Some(k) = at else {
            return Sec {
                found: false,
                sec_lines: Vec::new(),
                sec_fence: Vec::new(),
                glob_lines: lines.iter().map(|l| l.to_string()).collect(),
                sec_text: String::new(),
                tables: Vec::new(),
            };
        };
        let start = md.heads[k].line;
        let end = md.heads[k + 1..]
            .iter()
            .find(|h| h.level <= 2)
            .map_or(lines.len(), |h| h.line);
        let sec: Vec<&str> = lines[start..end].to_vec();
        let fence = fence_mask(&sec);
        let text = norm(
            &sec.iter()
                .zip(&fence)
                .filter(|(_, f)| !**f)
                .map(|(l, _)| *l)
                .collect::<Vec<_>>()
                .join(" "),
        );
        let tables = section_tables(&sec, &fence);
        let glob: Vec<String> = lines[..start]
            .iter()
            .chain(&lines[end..])
            .map(|l| l.to_string())
            .collect();
        Sec {
            found: true,
            sec_lines: sec.iter().map(|l| l.to_string()).collect(),
            sec_fence: fence,
            glob_lines: glob,
            sec_text: text,
            tables,
        }
    }
}

/// `fence_mask`: true on a fence line and inside a fence.
pub(crate) fn fence_mask(ls: &[&str]) -> Vec<bool> {
    let mut inside = false;
    ls.iter()
        .map(|line| {
            if line.trim_start().starts_with("```") {
                inside = !inside;
                return true;
            }
            inside
        })
        .collect()
}

/// `row_kind`: what a budget row's label names.
pub(crate) fn row_kind(lab: &str) -> &'static str {
    let has = |p: &str| rx(p).is_match(lab);
    if lab.starts_with("total") {
        "total"
    } else if has(r"\b(spare|margin|headroom|slack|deficit)\b") {
        "spare"
    } else if lab.contains("packet header") {
        "pkt"
    } else if lab.contains("slice header") {
        "slice"
    } else if lab.contains("announce") {
        "ann"
    } else if has(r"content[- ]message|content header") {
        "content"
    } else if lab.contains("audio") {
        "audio"
    } else if lab.contains("checksum") || has(r"\bcrc\b") {
        "cksum"
    } else if lab.contains("measurement") || lab.contains("extension") {
        "ext"
    } else if has(r"controller|dense|sparse|transport") {
        "ctrl"
    } else {
        "other"
    }
}

/// `classify_case`.
pub(crate) fn classify_case(text: &str) -> Option<&'static str> {
    let has = |p: &str| rx(p).is_match(text);
    if has(r"(?i)fallback|\b1\s?280\b|\b1\s?252\b") {
        Some("fb")
    } else if has(r"(?i)worst|maximum|maximal|all optional") {
        Some("worst")
    } else if has(r"(?i)default|mandatory|\b1\s?472\b") {
        Some("default")
    } else {
        None
    }
}

/// The number in column `c` of a row (a short row reads 0).
pub(crate) fn cell_num(row: &[String], c: usize) -> Option<i64> {
    parse_num(row.get(c).map(String::as_str))
}

/// The label of a row's first cell.
pub(crate) fn row_label(row: &[String]) -> String {
    label_of(row.first().map_or("", String::as_str))
}
