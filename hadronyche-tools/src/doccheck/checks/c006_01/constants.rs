//! AC 1: the constants table of timeline.md against 02-constants.md.

use std::collections::{BTreeMap, BTreeSet};

use super::scan::{Block as Table, Head, Scan, Source, norm_status, norm_value};
use crate::doccheck::Report;
use crate::doccheck::family006::norm_text;
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::text::{full, has, rx};

/// The ids of the ticket.
pub(crate) const IDS: [&str; 47] = [
    "C-01", "C-02", "C-10", "C-18", "C-19", "C-30", "C-33", "C-37", "C-38", "C-43", "C-44", "C-47",
    "C-61", "C-64", "C-65", "C-67", "C-78", "C-80", "C-102", "C-105", "C-111", "C-113", "C-120",
    "C-121", "C-122", "C-124", "C-134", "C-135", "C-136", "C-137", "C-159", "C-141", "C-162",
    "C-169", "C-181", "C-184", "C-187", "C-188", "C-189", "C-195", "C-197", "C-200", "C-202",
    "C-205", "C-207", "C-209", "C-212",
];

/// `header_index`: a column whose normalised header is one of `names` or
/// starts with one followed by a space or `(`.
fn header_index(header: &[String], names: &[&str]) -> Option<usize> {
    header.iter().position(|cell| {
        let c = norm_text(cell).to_lowercase();
        let c = c.trim_end_matches(['.', ':']);
        names
            .iter()
            .any(|n| c == *n || c.starts_with(&format!("{n} ")) || c.starts_with(&format!("{n}(")))
    })
}

fn is_cid(s: &str) -> bool {
    full(r"C-\d{2,3}", s)
}

fn constants_table(tables: &[Table]) -> Option<&Table> {
    tables.iter().find(|t| {
        let header = &t[0].1;
        let (Some(id), Some(_)) = (
            header_index(header, &["id", "constant id"]),
            header_index(header, &["value"]),
        ) else {
            return false;
        };
        t.iter().skip(2).any(|(_, cells)| {
            cells
                .get(id)
                .is_some_and(|c| is_cid(&c.py_strip().to_uppercase()))
        })
    })
}

/// `one_sentence_wins`.
fn one_sentence_wins(text: &str) -> bool {
    let text = norm_text(text);
    let mut start = 0;
    let mut sentences = Vec::new();
    for m in rx(r"[.!?]\s+").find_iter(&text) {
        sentences.push(&text[start..m.start() + 1]);
        start = m.end();
    }
    sentences.push(&text[start..]);
    sentences.iter().any(|s| {
        let low = s.to_lowercase();
        has(r"\bwins?\b|\bprevails?\b|\bprecedence\b", &low)
            && has(r"\bdiffer|\bconflict|\bdisagree|\bdiscrepan", &low)
    })
}

type Rows<'a> = BTreeMap<String, (usize, &'a Vec<String>)>;

fn rows_of<'a>(t: &'a Table, id_col: usize, r: &mut Report) -> Rows<'a> {
    let (mut rows, mut malformed) = (BTreeMap::new(), Vec::new());
    for (line, cells) in t.iter().skip(2) {
        if cells.iter().all(|c| c.py_strip().is_empty()) {
            continue;
        }
        let Some(cell) = cells.get(id_col) else {
            malformed.push(format!("line {line}: row has no id cell"));
            continue;
        };
        let cid = cell.py_strip().to_uppercase();
        if !is_cid(&cid) {
            malformed.push(format!("line {line}: unexpected row id {cid:?} (the table must carry only the ticket's C- ids)"));
        } else if let std::collections::btree_map::Entry::Vacant(slot) = rows.entry(cid.clone()) {
            slot.insert((*line, cells));
        } else {
            malformed.push(format!("line {line}: duplicate row {cid}"));
        }
    }
    if !malformed.is_empty() {
        r.fail(
            "AC1",
            malformed
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join("; "),
        );
    }
    rows
}

fn compare(
    rows: &Rows,
    cols: (usize, Option<usize>),
    source: &BTreeMap<String, Source>,
    r: &mut Report,
) {
    let (value_col, status_col) = cols;
    let (mut values, mut statuses) = (Vec::new(), Vec::new());
    for cid in IDS {
        let Some((line, cells)) = rows.get(cid) else {
            continue;
        };
        match (cells.get(value_col), source.get(cid)) {
            (None, _) => values.push(format!("line {line}: {cid} has no value cell")),
            (Some(v), Some(s)) if norm_value(v) != norm_value(&s.value) => {
                values.push(format!(
                    "{cid} at line {line}: document has {:?}; 02-constants.md has {:?}",
                    v.py_strip(),
                    s.value.py_strip()
                ));
            }
            _ => {}
        }
        if let (Some(sc), Some(s)) = (status_col.and_then(|c| cells.get(c)), source.get(cid))
            && norm_status(sc) != norm_status(&s.status)
        {
            statuses.push(format!(
                "{cid} at line {line}: document has {:?}; 02-constants.md has {:?}",
                sc.py_strip(),
                s.status.py_strip()
            ));
        }
    }
    for (what, list) in [("value", values), ("status", statuses)] {
        if !list.is_empty() {
            let more = if list.len() <= 3 {
                String::new()
            } else {
                format!(" (+{} more)", list.len() - 3)
            };
            r.fail(
                "AC1",
                format!(
                    "{what} mismatch against 02-constants.md: {}{more}",
                    list.iter().take(3).cloned().collect::<Vec<_>>().join(" | ")
                ),
            );
        }
    }
    for cid in ["C-61", "C-195"] {
        let Some((_, cells)) = rows.get(cid) else {
            continue;
        };
        let status = status_col
            .and_then(|c| cells.get(c))
            .map_or(String::new(), |s| norm_status(s));
        if !status.contains("pending") || !status.contains("simulation") {
            let shown = if status.is_empty() {
                "no status".to_string()
            } else {
                status
            };
            r.fail(
                "AC1",
                format!("{cid} must be marked pending simulation (found {shown:?})"),
            );
        }
    }
}

fn introduced(const_heading: Option<&Head>, headings: &[Head], r: &mut Report) {
    let Some(h) = const_heading.filter(|h| h.1 >= 2) else {
        r.fail("AC1", "the constants table is not an introduced section; it must be the first section after the introduction, under its own heading");
        return;
    };
    let h2s: Vec<&Head> = headings.iter().filter(|h| h.1 == 2).collect();
    let Some(ci) = h2s.iter().position(|x| x.0 == h.0) else {
        r.fail("AC1", "constants table is not under a level-2 heading");
        return;
    };
    let intro =
        r"(?i)\b(introduction|overview|scope|product frame|conventions?|evidence tags?|purpose)\b";
    if h2s[..ci].iter().any(|x| !has(intro, &x.2)) {
        let names: Vec<String> = h2s[..ci].iter().map(|x| format!("{:?}", x.2)).collect();
        r.fail("AC1", format!("the constants table is not the first section after the introduction; headings before it: {}", names.join(", ")));
    }
}

/// AC 1.
pub(crate) fn ac1(
    lines: &[&str],
    scan: &Scan,
    tables: &[Table],
    source: Option<&BTreeMap<String, Source>>,
    r: &mut Report,
) {
    let empty = BTreeMap::new();
    let src = source.unwrap_or(&empty);
    match source {
        None => r.fail(
            "AC1",
            "doc/design/02-constants.md does not exist; cannot verify values",
        ),
        Some(s) => {
            let missing: Vec<&str> = sorted_ids()
                .into_iter()
                .filter(|i| !s.contains_key(*i))
                .collect();
            if !missing.is_empty() {
                r.fail(
                    "AC1",
                    format!(
                        "02-constants.md lacks ids required by the ticket: {}",
                        missing.join(", ")
                    ),
                );
            }
        }
    }
    let Some(t) = constants_table(tables) else {
        r.fail("AC1", "no constants table found (expected a Markdown table with Id and Value columns and C-01 to C-212 rows)");
        return;
    };
    let header = &t[0].1;
    let (Some(id_col), Some(value_col)) = (
        header_index(header, &["id", "constant id"]),
        header_index(header, &["value"]),
    ) else {
        return;
    };
    let status_col = header_index(header, &["status", "state"]);
    if status_col.is_none() {
        r.fail("AC1", "constants table has no status column");
    }
    let rows = rows_of(t, id_col, r);
    let want: BTreeSet<&str> = IDS.iter().copied().collect();
    let have: BTreeSet<&str> = rows.keys().map(String::as_str).collect();
    let missing: Vec<&str> = want.difference(&have).copied().collect();
    let extra: Vec<&str> = have.difference(&want).copied().collect();
    if !missing.is_empty() {
        r.fail(
            "AC1",
            format!("constants table is missing ids: {}", missing.join(", ")),
        );
    }
    if !extra.is_empty() {
        r.fail(
            "AC1",
            format!(
                "constants table has ids not in the ticket's 47 rows: {}",
                extra.join(", ")
            ),
        );
    }
    if missing.is_empty() && extra.is_empty() && rows.len() != 47 {
        r.fail(
            "AC1",
            format!(
                "constants table has {} rows; expected exactly 47",
                rows.len()
            ),
        );
    }
    compare(&rows, (value_col, status_col), src, r);
    let table_start = t[0].0;
    let ch = scan.headings.iter().rfind(|h| h.0 < table_start);
    introduced(ch, &scan.headings, r);
    let table_end = t[t.len() - 1].0;
    let end = match ch {
        Some(h) => scan
            .headings
            .iter()
            .find(|o| o.0 > h.0 && o.1 <= h.1)
            .map_or(lines.len(), |o| o.0 - 1),
        None => table_end,
    };
    if !one_sentence_wins(&lines[..end.min(lines.len())].join("\n")) {
        r.fail("AC1", "no sentence states that the constants table wins on any difference (searched the introduction and constants section)");
    }
}

fn sorted_ids() -> Vec<&'static str> {
    let mut v: Vec<&str> = IDS.to_vec();
    v.sort_unstable();
    v
}
