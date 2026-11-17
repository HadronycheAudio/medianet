//! AC 3 (the Open sub-questions table) and AC 4 (the evidence-appendix
//! verdict entries, the resolved M-3 / M-4 rows).

use std::collections::BTreeMap;

use super::{Rep, TAG, bad, cells, cut, is_sep, is_table_row};
use crate::doccheck::Report;
use crate::doccheck::text::{has, has_i};

/// The four investigation questions: (name, pattern on the first cell,
/// `re.I`), in the script's dict order.
const KEYS: &[(&str, &str)] = &[
    ("voice comparison", "voice comparison|bit-level"),
    ("I/O-pressure finding", "I/O[- ]pressure"),
    ("the 21.7 M value", r"21\.7"),
    ("the 5 ms timeout", "5 ms"),
];

/// AC 3.
pub(super) fn ac3(rep: &Rep, report: &mut Report) {
    let Some(oi) = rep.heading_index("Open sub-questions") else {
        bad(report, "AC 3: \"Open sub-questions\" section not found".into());
        return;
    };
    let oj = rep.section_end(oi, 2);
    let rows: Vec<&String> = rep.lines[oi + 1..oj]
        .iter()
        .filter(|l| is_table_row(l) && !is_sep(l))
        .skip(1) // header row
        .collect();
    let mut found: BTreeMap<&str, &String> = BTreeMap::new();
    for row in rows {
        let c = cells(row);
        let answered = has_i(r"\banswered\b", row);
        for (name, pat) in KEYS {
            if has_i(pat, &c[0]) {
                found.insert(name, row);
            }
        }
        if !answered {
            let owner = c.get(1).map_or("", String::as_str);
            let falsifier = c.get(2).map_or("", String::as_str);
            if has_i("this epic|002-", owner) {
                bad(report, format!("AC 3: open sub-question not answered and owned inside this epic: {}", cut(&c[0], 80)));
            } else if owner.is_empty() || falsifier.is_empty() {
                bad(report, format!("AC 3: open sub-question not answered and names no owner or no falsifier: {}", cut(&c[0], 80)));
            }
        }
    }
    for (name, _) in KEYS {
        let Some(row) = found.get(name) else {
            bad(report, format!("AC 3: investigation question \"{name}\" has no row in the Open sub-questions table"));
            continue;
        };
        if !has_i(r"\banswered\b", row) {
            bad(report, format!("AC 3: \"{name}\" is not marked answered"));
        } else if !has_i(r"\([a-e]\)|section\s*\(?[a-e]\)?\b", row) {
            bad(
                report,
                format!("AC 3: \"{name}\" is answered but names no section (x) of the report"),
            );
        } else {
            report.passes.push(("AC3".into(), format!("\"{name}\" answered")));
        }
    }
}

/// AC 4.
pub(super) fn ac4(ev_lines: &[String], report: &mut Report) {
    let ev_rows: Vec<&String> = ev_lines
        .iter()
        .filter(|l| is_table_row(l) && !is_sep(l))
        .collect();
    let mut entries: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    let letter = crate::doccheck::text::rx(r"\(([a-e])\)");
    for r in &ev_rows {
        let c = cells(r);
        if !has_i("REPORT-0b|archaeology", r) {
            continue;
        }
        if let Some(m) = letter.captures(&c[0]) {
            entries.entry(m[1].to_string()).or_default().push(r);
        }
    }
    for l in ["a", "b", "c", "d", "e"] {
        let rs = entries.get(l).map_or(&[][..], Vec::as_slice);
        if rs.is_empty() {
            bad(report, format!("AC 4: the evidence appendix has no verdict entry for ({l}) (a row whose first cell carries \"({l})\" and that mentions REPORT-0b)"));
        } else if !rs.iter().all(|r| has(TAG, r)) {
            bad(
                report,
                format!("AC 4: the verdict entry for ({l}) carries no evidence tag"),
            );
        } else {
            report.passes.push(("AC4".into(), format!("entry ({l})")));
        }
    }
    for r in entries.get("e").map_or(&[][..], Vec::as_slice) {
        if has_i("not a bug", r) {
            bad(report, "AC 4: the appendix entry for verdict (e) says \"not a bug\"; the sync pacing is an instrument bug".into());
        }
    }
    // M-3 / M-4: no appendix row still reads Unresolved with Task 002 as its resolver
    for r in &ev_rows {
        if has(r"\bUnresolved\b", r) && r.contains("Task 002") {
            bad(report, format!("AC 4: an appendix row still says \"Unresolved\" with \"Task 002\" as its resolver: {}", cut(r, 90)));
        }
    }
}
