//! The content of the types: the Context values (AC1), the capability and
//! timestamp enumerations (AC2), the health counters (AC3), control
//! isolation and the panic hook (AC4).

use std::collections::{BTreeMap, BTreeSet};

use super::Consts;
use super::types::SType;
use crate::doccheck::Report;
use crate::doccheck::family004::tables;
use crate::doccheck::family004::Field;
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::tables::Row;
use crate::doccheck::text::{has, has_i};

fn rows_text(fl: &[Field]) -> Vec<String> {
    fl.iter().map(|f| format!("{} {}", f.meaning, f.valid)).collect()
}

fn get<'a>(types: &'a [SType], key: &str) -> Option<&'a SType> {
    types.iter().find(|t| t.key == key)
}

/// AC1 (Context) and AC4 per type.
pub(crate) fn context(types: &[SType], report: &mut Report) {
    if let Some(t) = get(types, "multi") {
        let mt = rows_text(&t.fl_all);
        for (label, pat) in [("the stream number", "stream"), ("the packets per slice", "packet"), ("the frames per slice or its equivalent", "frame|slice length|slice")] {
            report.check("AC1", mt.iter().any(|x| has_i(pat, x)), format!("the multi-packet section carries {label} (Context, the values 004-08 hands over)"));
        }
    }
    if let Some(t) = get(types, "commissioning") {
        report.check("AC1", t.fl_all.iter().any(|f| has_i("expir", &f.meaning)), "the commissioning window section carries the expiry time of the open window (Context, C-153)");
    }
    if let Some(t) = get(types, "calibration") {
        let ok = has_i("calibration factor", &t.text) && t.fl_all.iter().any(|f| has_i("operation", &f.meaning));
        report.check("AC1", ok, "the calibration section carries the calibration factor, time per counted operation (Context, C-106)");
    }
    if let Some(t) = get(types, "injection") {
        let items = [
            ("a forced module id or nonce", "module id|nonce", "AC1"),
            ("a stream forced onto a timeline epoch", "epoch", "AC1"),
            ("raw packets on a media or control socket", r"\braw\b", "AC1"),
            ("a panic injected into a thread (AC 4: the panic hook)", "panic", "AC4"),
        ];
        for (label, pat, ac) in items {
            report.check(ac, has_i(pat, &t.text), format!("the injection section names {label} (Context, C-120, C-145)"));
        }
    }
    if let Some(t) = get(types, "capability") {
        report.check("AC4", has_i("isolat", &t.text), "the capability section includes the control-isolation capability (AC 4, C-161)");
    }
    if let Some(t) = get(types, "timestamp") {
        report.check("AC1", has_i(r"capabilit\w*\s+state|C-129", &t.text), "the receive-timestamp section reports its source with a C-129 capability state (Context, C-131)");
    }
}

/// `c.strip().strip("`*").strip().lower()`.
fn cell_norm(c: &str) -> String {
    c.trim().trim_matches(|x| x == '`' || x == '*').trim().to_lowercase()
}

/// `closed_enum`: (matched rows, all rows) of the table with the most rows
/// naming a value of `names` (the first on a tie).
fn closed_enum(text: &str, names: &BTreeSet<String>) -> Option<(Vec<Row>, usize)> {
    let mut best: Option<(Vec<Row>, usize)> = None;
    for tb in tables(text) {
        let matched: Vec<Row> = tb.rows.iter().filter(|r| r.cells.iter().any(|c| names.contains(&cell_norm(c)))).cloned().collect();
        if best.as_ref().is_none_or(|b| matched.len() > b.0.len()) {
            best = Some((matched, tb.rows.len()));
        }
    }
    best
}

fn enumeration(t: &SType, names: &[String], what: &str, report: &mut Report) {
    let set: BTreeSet<String> = names.iter().cloned().collect();
    let best = closed_enum(&t.text, &set);
    let ok = best.as_ref().is_some_and(|(m, n)| {
        let cells: BTreeSet<String> = m.iter().flat_map(|r| r.cells.iter().map(|c| cell_norm(c))).collect();
        *n == 4 && m.len() == 4 && set.is_subset(&cells)
    });
    let (rows, matched) = best.as_ref().map_or((0, 0), |(m, n)| (*n, m.len()));
    report.check("AC2", ok, format!("the {what} enumeration is a table of exactly the four {} (table rows {rows}, matched {matched})", if what.contains("capability") { "C-129 states" } else { "C-131 sources" }));
}

/// AC2.
pub(crate) fn ac2(types: &[SType], consts: &Consts, report: &mut Report) {
    if let Some(t) = get(types, "capability").filter(|_| !consts.cap_states.is_empty()) {
        enumeration(t, &consts.cap_states, "capability-state", report);
    }
    if let Some(t) = get(types, "timestamp").filter(|_| !consts.ts_sources.is_empty()) {
        enumeration(t, &consts.ts_sources, "receive-timestamp", report);
    }
}

/// `counter_re`.
pub(super) fn counter_re(n: &str) -> String {
    let special: BTreeMap<&str, &str> = [
        ("marked gap", r"marked[\s-]+gap"),
        ("other-epoch", r"other[\s-]+epoch"),
        ("too-early", r"too[\s-]+early"),
        ("incomplete reassemblies", r"incomplete[\s-]+reassembl"),
        ("demoted rounds", "demoted"),
        ("interrupted events", "interrupted"),
        ("duplicates", "duplicat"),
        ("dead", r"\bdead\b"),
        ("received", r"\breceiv"),
        ("late", r"\blate\b"),
        ("missing", r"\bmissing\b"),
    ]
    .into_iter()
    .collect();
    special.get(n).map_or_else(|| regex::escape(n), |p| (*p).to_string())
}

/// `max_match`: a maximum bipartite matching by augmenting paths, in the
/// script's order; (matched, counter index → text index).
pub(super) fn max_match(keys: &[String], texts: &[&str]) -> (usize, BTreeMap<usize, usize>) {
    let cand: Vec<Vec<usize>> = keys.iter().map(|k| texts.iter().enumerate().filter(|(_, t)| has_i(k, t)).map(|(j, _)| j).collect()).collect();
    let mut owner: BTreeMap<usize, usize> = BTreeMap::new();
    fn aug(i: usize, cand: &[Vec<usize>], seen: &mut BTreeSet<usize>, owner: &mut BTreeMap<usize, usize>) -> bool {
        for &j in &cand[i] {
            if !seen.insert(j) {
                continue;
            }
            let free = match owner.get(&j) {
                None => true,
                Some(&o) => aug(o, cand, seen, owner),
            };
            if free {
                owner.insert(j, i);
                return true;
            }
        }
        false
    }
    let matched = (0..keys.len()).filter(|&i| aug(i, &cand, &mut BTreeSet::new(), &mut owner)).count();
    (matched, owner.into_iter().map(|(j, i)| (i, j)).collect())
}

/// AC3.
pub(crate) fn ac3(types: &[SType], consts: &Consts, report: &mut Report) -> Flow<()> {
    let Some(h) = get(types, "health").filter(|_| consts.c159.len() == 11) else { return Ok(()) };
    if h.root.is_none() {
        return crash("KeyError: 'entries'");
    }
    let hf = h.entry_fields();
    let ht: Vec<&str> = hf.iter().map(|f| f.meaning.as_str()).collect();
    let keys: Vec<String> = consts.c159.iter().map(|c| counter_re(c)).collect();
    let (n, assign) = max_match(&keys, &ht);
    report.check("AC3", n == 11, format!("each of the eleven C-159 counters has its own row of the health section (matched {n})"));
    for (i, c) in consts.c159.iter().enumerate() {
        let Some(&j) = assign.get(&i) else { continue };
        let f = hf[j];
        let both = format!("{} {}", f.meaning, f.valid);
        report.check("AC3", has_i("wrap|saturat", &both), format!("the counter {c} states its wrap or saturation rule (Work 1)"));
        report.check("AC3", has_i("packet|slice|event|round|report|connection|stream", &both), format!("the counter {c} states its unit (Work 1)"));
    }
    report.check("AC3", hf.iter().any(|f| has("DSCP", &f.meaning) && has_i("observ", &f.meaning)), "the health section carries the observed DSCP of the connection (C-47, Context)");
    Ok(())
}
