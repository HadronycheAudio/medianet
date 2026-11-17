//! The two guards over the design set: no stale statement of 37 or 77 as
//! the achieved sync rate, and no sentence saying task 003 fixes the loss
//! accounting.

use super::{DESIGN_DIR, EVIDENCE, Rep, bad, cells, cut, is_table_row, read_lines};
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::text::{has_i, rx};
use crate::doccheck::{Docs, Report};

/// `CLAUSE_SPLIT` of the script.
const CLAUSE_SPLIT: &str = r"(?<=\.)\s+|;\s+|:\s+";

/// `re.split(CLAUSE_SPLIT, s)`; with `semi_colon` false,
/// `re.split(r'(?<=\.)\s+', s)` (the loss guard's prose split).
pub(super) fn split_clauses(s: &str, semi_colon: bool) -> Vec<&str> {
    let pattern = if semi_colon { CLAUSE_SPLIT } else { r"(?<=\.)\s+" };
    rx(pattern).split(s).collect()
}

/// `STALE_RX.search(cl)`.
fn stale_rate(cl: &str) -> bool {
    has_i(
        r"(?<![\w.,/-])(?:37|77)(?:\s*/\s*s\b|/s\b|\s*/\s*(?:37|77)\b|\s+and\s+(?:37|77)\b|\s+(?:exchanges?|completed|per second)\b)|\b(?:achieved|reached|delivered|gives?|gave)\s+(?:37|77)\b",
        cl,
    )
}

/// `CTX_RX` and `MARK` of the script.
const CTX: &str = r"sync|exchange|configured|--sync-hz|\bhz\b|\brate\b|\b50\b|\b1000\b";
const MARK: &str = r"corrupt|superseded|earlier|originally|corrected|36\.1|71\.4";

/// The `.md` files the script's `os.walk(DESIGN_DIR)` visits: no directory
/// named `tickets` on the way, not named `REPORT-0b.md`.
fn design_files(docs: &Docs) -> Vec<String> {
    docs.files_under(DESIGN_DIR)
        .into_iter()
        .filter(|p| {
            let rel = &p[DESIGN_DIR.len() + 1..];
            let parts: Vec<&str> = rel.split('/').collect();
            let (name, dirs) = parts.split_last().unwrap_or((&"", &[]));
            name.ends_with(".md") && *name != "REPORT-0b.md" && !dirs.contains(&"tickets")
        })
        .collect()
}

/// The stale sync-rate guard. False when the script would crash (a file
/// that is not UTF-8).
pub(super) fn stale(docs: &Docs, report: &mut Report) -> bool {
    let mut hits = 0;
    for path in design_files(docs) {
        let Some(lines) = read_lines(docs, &path) else {
            report.crash(format!("UnicodeDecodeError: {path} is not UTF-8"));
            return false;
        };
        let rel = &path[DESIGN_DIR.len() + 1..];
        for (n, l) in lines.iter().enumerate() {
            // a table row: the marker must sit in the same cell as the
            // rate; prose: the same clause
            let table = is_table_row(l);
            let parts: Vec<String> = if table { cells(l) } else { vec![l.clone()] };
            let hit = parts
                .iter()
                .flat_map(|c| {
                    split_clauses(c, true)
                        .into_iter()
                        .map(move |cl| (cl, if table { c.as_str() } else { cl }))
                })
                .find(|(cl, scope)| stale_rate(cl) && has_i(CTX, cl) && !has_i(MARK, scope));
            if let Some((cl, _)) = hit {
                hits += 1;
                bad(report, format!("stale sync-rate statement (37 or 77 stated as the achieved rate, not marked historical or corrected) {rel}:{}: {}", n + 1, cut(cl.py_strip(), 220)));
            }
        }
    }
    if hits == 0 {
        report.passes.push(("stale".into(), "no stale statement".into()));
    }
    true
}

const LOSS: &str = r"loss counter|loss accounting|\brx_lost\b|\blost_fwd\b|loss display";
const FIX003: &str = r"\b(?:fix(?:e[sd])?|resolv(?:e[sd]?)|repair(?:s|ed)?)\b[^.;|]{0,40}\b003\b|\b003\b[^.;|]{0,60}\bfix(?:es|ed)?\b|\bfixed by (?:task |ticket )?003\b";
const NEG: &str = r"\b(?:not|no|neither|nor|never|without|unfixed|cannot)\b|n't\b";

/// `loss_findings(line)`: the clauses saying task 003 fixes the loss
/// counter, with no negation in the clause.
fn loss_findings(line: &str) -> Vec<String> {
    let table = is_table_row(line);
    let cand: Vec<String> = if table {
        if !has_i(LOSS, line) {
            return Vec::new();
        }
        cells(line)
            .iter()
            .flat_map(|c| split_clauses(c, true).into_iter().map(str::to_string).collect::<Vec<_>>())
            .collect()
    } else {
        split_clauses(line, false)
            .into_iter()
            .filter(|u| has_i(LOSS, u))
            .map(str::to_string)
            .collect()
    };
    cand.into_iter()
        .filter(|cl| has_i(FIX003, cl) && !has_i(NEG, cl) && (table || has_i(LOSS, cl)))
        .collect()
}

/// The loss-accounting guard over the report, the appendix and the design
/// set (each file once). False when the script would crash.
pub(super) fn loss(docs: &Docs, rep: &Rep, ev: &[String], report: &mut Report) -> bool {
    let mut sources: Vec<(String, Vec<String>)> = vec![
        (super::REPORT.to_string(), rep.lines.clone()),
        (EVIDENCE.to_string(), ev.to_vec()),
    ];
    for path in design_files(docs) {
        if path == EVIDENCE {
            continue; // `seen`: the same path string
        }
        let Some(lines) = read_lines(docs, &path) else {
            report.crash(format!("UnicodeDecodeError: {path} is not UTF-8"));
            return false;
        };
        sources.push((path, lines));
    }
    let mut hits = 0;
    for (path, lines) in &sources {
        let base = path.rsplit('/').next().unwrap_or(path);
        for (n, l) in lines.iter().enumerate() {
            for u in loss_findings(l) {
                hits += 1;
                bad(report, format!("loss accounting stated as fixed by task 003 (no negation) {base}:{}: {}", n + 1, cut(u.py_strip(), 220)));
            }
        }
    }
    if hits == 0 {
        report.passes.push(("loss".into(), "no 003-fixes statement".into()));
    }
    true
}
