//! Fences, headings, tables and the constants of 02-constants.md, as
//! `checks/006-01.sh` reads them.

use crate::doccheck::markdown::{Markdown, code_mask};
use crate::doccheck::policy::{FencePolicy, TableRules};
use crate::doccheck::pycompat::{PyStr, nfkc, splitlines};
use crate::doccheck::tables::{Table, split_row_char_walk};
use crate::doccheck::text::{full, has, has_i, rx};

/// A heading: (1-based line number, level, text).
pub(crate) type Head = (usize, usize, String);

/// What the line scan finds.
pub(crate) struct Scan {
    pub headings: Vec<Head>,
    pub errors: Vec<String>,
}

/// `norm_text`: NFKC, the figure and thin spaces as spaces, zero-width
/// characters dropped, typographic dashes and quotes plain, blanks squashed.
pub(crate) fn norm_text(s: &str) -> String {
    let mut t = nfkc(s);
    for ch in ['\u{a0}', '\u{2007}', '\u{202f}', '\u{2009}', '\u{200a}'] {
        t = t.replace(ch, " ");
    }
    t = t.replace(['\u{200b}', '\u{feff}'], "");
    let t = rx(r"[‐-―−]").replace_all(&t, "-");
    let t = t
        .replace(['\u{2018}', '\u{2019}'], "'")
        .replace(['\u{201c}', '\u{201d}'], "\"");
    rx(r"\s+").replace_all(&t, " ").py_strip().to_string()
}

/// True when the fence mask ends inside an open fence (the script's final
/// `in_fence`). Two probes are appended to the lines: a backtick marker and
/// a tilde marker, both longer than any fence, then a plain line. The plain
/// line is outside code exactly when the open fence (if any) was closed by
/// the marker of its own character; when no fence was open the marker opens
/// one, which keeps the plain line inside code.
fn ends_open(lines: &[&str]) -> bool {
    let probe = |marker: &str| {
        let mut v: Vec<&str> = lines.to_vec();
        let owned = marker.repeat(1000);
        v.push(&owned);
        v.push("x");
        !code_mask(&v, FencePolicy::Matching)[v.len() - 1]
    };
    probe("`") || probe("~")
}

/// `scan_fences_and_headings`.
pub(crate) fn scan(lines: &[&str]) -> Scan {
    let mask = code_mask(lines, FencePolicy::Matching);
    let (mut headings, mut errors) = (Vec::new(), Vec::new());
    let heading = rx(r"^(#{1,6})[ \t]+(.*?)[ \t]*#*[ \t]*$");
    for (k, line) in lines.iter().enumerate() {
        if mask[k] {
            continue;
        }
        let i = k + 1;
        if has(r"^[ \t]+#{1,6}[ \t]+\S", line) {
            errors.push(format!(
                "line {i}: heading is indented and does not start the line with '#': {}",
                line.py_strip()
            ));
        }
        if let Some(h) = heading.captures(line) {
            headings.push((i, h[1].len(), h[2].py_strip().to_string()));
        }
    }
    if ends_open(lines) {
        let at = (0..lines.len())
            .rev()
            .find(|&k| mask[k] && (k == 0 || !mask[k - 1]))
            .unwrap_or_default();
        errors.push(format!(
            "line {}: unclosed code fence opened with {}",
            at + 1,
            lines[at].trim_start_matches([' ', '\t'])
        ));
    }
    Scan { headings, errors }
}

/// A table as the script keeps it: its (1-based line, cells) lines, header
/// and separator first.
pub(crate) type Block = Vec<(usize, Vec<String>)>;

/// The tables of the document (006-01's `parse_tables`): the library's
/// `T006_01` rules over the same fence mask, each with its header and
/// separator lines kept.
pub(crate) fn tables(lines: &[&str]) -> Vec<Block> {
    let md = Markdown::parse_with(&lines.join("\n"), FencePolicy::Matching);
    md.tables_with(md.all(), TableRules::T006_01)
        .into_iter()
        .filter_map(|t: Table| {
            let header = t.header.clone()?;
            let sep = split_row_char_walk(lines[t.line + 1]).unwrap_or_default();
            let mut block = vec![(t.line + 1, header), (t.line + 2, sep)];
            block.extend(t.rows.iter().map(|r| (r.line + 1, r.cells.clone())));
            Some(block)
        })
        .collect()
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
    for line in splitlines(text) {
        let Some(cells) = split_row_char_walk(line) else {
            continue;
        };
        let cid = cells[0].py_strip().to_uppercase();
        if !full(r"C-\d{2,3}", &cid) {
            continue;
        }
        if cells.len() >= 4 {
            out.insert(
                cid,
                Source {
                    value: cells[2].clone(),
                    status: cells[3].clone(),
                },
            );
        } else if cells.len() == 3 {
            let pending = has_i(r"pending\s*[- ]?\s*simulation", &cells[2]);
            out.insert(
                cid,
                Source {
                    value: cells[2].clone(),
                    status: if pending {
                        "pending simulation"
                    } else {
                        "decided"
                    }
                    .to_string(),
                },
            );
        }
    }
    out
}

/// `norm_value`.
pub(crate) fn norm_value(s: &str) -> String {
    norm_text(s)
        .replace('`', "")
        .replace("**", "")
        .replace("__", "")
}

/// `norm_status`.
pub(crate) fn norm_status(s: &str) -> String {
    let t = norm_text(s).to_lowercase().replace(['_', '-'], " ");
    rx(r"\s+").replace_all(&t, " ").py_strip().to_string()
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
