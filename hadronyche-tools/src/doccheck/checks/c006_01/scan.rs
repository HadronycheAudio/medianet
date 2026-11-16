//! Fences, headings, tables and the constants of 02-constants.md, as
//! `checks/006-01.sh` reads them.

use super::text::{norm_text, split_row};
use crate::doccheck::text::{has, has_i, rx};

/// A heading: (1-based line number, level, text).
pub(crate) type Head = (usize, usize, String);

/// What the line scan finds.
pub(crate) struct Scan {
    pub headings: Vec<Head>,
    pub errors: Vec<String>,
    pub visible: Vec<bool>,
}

/// `scan_fences_and_headings`: an opener is three or more backticks or
/// tildes; a closer is the same character, at least as long, with nothing
/// after it ([`FencePolicy::Matching`] in the library differs: it lets an
/// info string close a fence).
///
/// [`FencePolicy::Matching`]: crate::doccheck::policy::FencePolicy::Matching
pub(crate) fn scan(lines: &[&str]) -> Scan {
    let (mut headings, mut errors) = (Vec::new(), Vec::new());
    let mut visible = vec![true; lines.len()];
    let mut open: Option<(char, usize, usize)> = None;
    let fence = rx(r"^[ \t]*(`{3,}|~{3,})(.*)$");
    let heading = rx(r"^(#{1,6})[ \t]+(.*?)[ \t]*#*[ \t]*$");
    for (k, line) in lines.iter().enumerate() {
        let i = k + 1;
        if let Some(m) = fence.captures(line) {
            let (marker, tail) = (&m[1], m[2].trim());
            let c = marker.chars().next().unwrap_or('`');
            match open {
                None => open = Some((c, marker.len(), i)),
                Some((oc, on, _)) if c == oc && marker.len() >= on && tail.is_empty() => open = None,
                Some(_) => {}
            }
            visible[k] = false;
            continue;
        }
        if open.is_some() {
            visible[k] = false;
            continue;
        }
        if has(r"^[ \t]+#{1,6}[ \t]+\S", line) {
            errors.push(format!("line {i}: heading is indented and does not start the line with '#': {}", line.trim()));
        }
        if let Some(h) = heading.captures(line) {
            headings.push((i, h[1].len(), h[2].trim().to_string()));
        }
    }
    if let Some((c, n, at)) = open {
        errors.push(format!("line {at}: unclosed code fence opened with {}", c.to_string().repeat(n)));
    }
    Scan { headings, errors, visible }
}

/// A table: its (1-based line, cells) lines, header first.
pub(crate) type Table = Vec<(usize, Vec<String>)>;

fn is_separator(line: &str) -> bool {
    let s = line.trim();
    s.contains('-') && rx(r"^(?:\|?[\s:|-]+\|?)$").is_match(s)
}

/// `parse_tables`: a run of visible lines starting with `|`, at least two
/// lines, the second a separator.
pub(crate) fn tables(lines: &[&str], visible: &[bool]) -> Vec<Table> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if visible[i] && lines[i].trim_start().starts_with('|') {
            let mut j = i;
            let mut block = Vec::new();
            while j < lines.len() && visible[j] && lines[j].trim_start().starts_with('|') {
                block.push((j + 1, split_row(lines[j]).unwrap_or_default()));
                j += 1;
            }
            if block.len() >= 2 && is_separator(lines[i + 1]) {
                out.push(block);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// One constant of 02-constants.md.
#[derive(Debug, Clone)]
pub(crate) struct Source {
    pub value: String,
    pub status: String,
}

/// `parse_constants_source`: id -> value and status; a row of three cells
/// has the status `decided` unless its value says pending simulation.
pub(crate) fn constants_source(text: &str) -> std::collections::BTreeMap<String, Source> {
    let mut out = std::collections::BTreeMap::new();
    for line in crate::doccheck::text::splitlines(text) {
        let Some(cells) = split_row(line) else { continue };
        let cid = cells[0].trim().to_uppercase();
        if !rx(r"^C-\d{2,3}$").is_match(&cid) {
            continue;
        }
        if cells.len() >= 4 {
            out.insert(cid, Source { value: cells[2].clone(), status: cells[3].clone() });
        } else if cells.len() == 3 {
            let pending = has_i(r"pending\s*[- ]?\s*simulation", &cells[2]);
            out.insert(cid, Source {
                value: cells[2].clone(),
                status: if pending { "pending simulation" } else { "decided" }.to_string(),
            });
        }
    }
    out
}

/// `norm_value`.
pub(crate) fn norm_value(s: &str) -> String {
    norm_text(s).replace(['`'], "").replace("**", "").replace("__", "")
}

/// `norm_status`.
pub(crate) fn norm_status(s: &str) -> String {
    let t = norm_text(s).to_lowercase().replace(['_', '-'], " ");
    rx(r"\s+").replace_all(&t, " ").trim().to_string()
}

/// `section_body`: the lines after the heading up to the line before the
/// next heading of the same or a higher level.
pub(crate) fn section_body(lines: &[&str], headings: &[Head], h: &Head) -> String {
    let end = headings
        .iter()
        .find(|o| o.0 > h.0 && o.1 <= h.1)
        .map_or(lines.len(), |o| o.0 - 1);
    lines[h.0.min(end)..end].join("\n")
}
