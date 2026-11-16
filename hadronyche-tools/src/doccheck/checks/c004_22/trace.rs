//! AC1 (rules 1 to 22) and AC2 (the constants of the ticket's list).

use std::collections::{BTreeMap, BTreeSet};

use super::refs::{base_of, dangling, py_list, reference_resolves, rule_number};
use super::{Inputs, Trace, WIRE_MD};
use crate::doccheck::policy::RowSplit;
use crate::doccheck::tables::Row;
use crate::doccheck::text::{full, head, rx};
use crate::doccheck::ticket::{cid, expand_constant_ids};
use crate::doccheck::{Docs, Report};

/// The ticket's "Constants traced:" sentence: (every id, the wire-visible
/// ids after "wire-visible rulings").
pub struct Lists {
    pub all: BTreeSet<u32>,
    pub visible: BTreeSet<u32>,
}

/// Reads the two lists from the ticket; `None` without the sentence.
pub fn constant_lists(ticket: &str) -> Option<Lists> {
    let at = ticket.find("Constants traced:")?;
    let sentence = ticket[at + "Constants traced:".len()..].split('\n').next().unwrap_or("");
    let visible = sentence.find("wire-visible rulings").map_or("", |i| &sentence[i..]);
    Some(Lists {
        all: expand_constant_ids(sentence),
        visible: expand_constant_ids(visible),
    })
}

fn rule_rows(trace: &Trace) -> BTreeMap<u32, Vec<&Row>> {
    let mut seen: BTreeMap<u32, Vec<&Row>> = BTreeMap::new();
    for row in &trace.rows {
        if let Some(n) = rule_number(row.cell(0)) {
            seen.entry(n).or_default().push(row);
        }
    }
    seen
}

/// AC1: one row per rule 1..22 whose mapping resolves; every link of the
/// section resolves; every backticked snake_case name is a wire.toml field.
pub fn ac1(inp: &Inputs, trace: Option<&Trace>, docs: &Docs, report: &mut Report) {
    let Some(trace) = trace else { return };
    let seen = rule_rows(trace);
    let missing: Vec<u32> = (1..=22).filter(|n| !seen.contains_key(n)).collect();
    let dup: Vec<u32> = (1..=22).filter(|n| seen.get(n).is_some_and(|r| r.len() > 1)).collect();
    if !missing.is_empty() {
        report.fail("AC1", format!("the Traceability table has no row for rule(s) {missing:?} of the ticket's 1..22 (rows are identified by a 'Rule N' or 'N' first cell)"));
    }
    if !dup.is_empty() {
        report.fail("AC1", format!("rule(s) {dup:?} have more than one row in the Traceability table"));
    }
    let norms = inp.wire.head_norms();
    for n in 1..=22 {
        let Some(row) = seen.get(&n).and_then(|r| r.first()) else { continue };
        let mapping = row.rest();
        let line = row.line + 1;
        if mapping.is_empty() || rx(r"[\s|\-]").replace_all(&mapping, "").is_empty() {
            report.fail_at("AC1", WIRE_MD, line, format!("rule {n}: the mapping cell is empty"));
        } else if !reference_resolves(&mapping, &norms) {
            report.fail_at("AC1", WIRE_MD, line, format!("rule {n}: no reference in its mapping cell resolves to a heading of wire.md ('{}')", head(&mapping, 90)));
        }
    }
    for row in &trace.rows {
        let text = row.joined();
        for d in dangling(docs, &text, base_of(WIRE_MD), &inp.wire, WIRE_MD, "Traceability row") {
            report.fail_at("AC1", WIRE_MD, row.line + 1, d);
        }
        for c in rx(r"`([^`]+)`").captures_iter(&text) {
            let tok = &c[1];
            if full(r"[a-z][a-z0-9]*(_[a-z0-9]+)+", tok) && !inp.widths.contains_key(tok) {
                report.fail_at("AC1", WIRE_MD, row.line + 1, format!("a Traceability row cites `{tok}`, which is not a field of wire.toml"));
            }
        }
    }
}

const NAMES: &str = r"((?:`[a-z][a-z0-9_]*`)(?:\s*(?:,|and|or)\s*`[a-z][a-z0-9_]*`)*)";
const UNIT_AFTER: &str = r"\b(bytes?|ns|fs|ps|µs|us|ms|samples?|frames?|packets?|slices?|Hz|counts?|codes?|ids?|identifiers?|flags?|none|epochs?|operations?)\b|\bper\s+\w+|bits?/s|unit:\s*none\s*\(";

/// `width_problems`: (fields stated with a width, problems) for one row:
/// "`name` (N bits, unit)" or "`a`, `b`: N bits each, unit", N equal to a
/// wire.toml width of the field (bytes converted), a unit after the width,
/// and no other wire.toml field of the row without a width.
pub fn width_problems(row: &Row, widths: &BTreeMap<String, Vec<Option<i64>>>) -> (usize, Vec<String>) {
    let text = row.joined();
    let res = [
        rx(&format!(r"{NAMES}\s*\(\s*(\d+)\s*(bits?|bytes?)\b([^)]*)\)")),
        rx(&format!(r"{NAMES}\s*:\s*(\d+)\s*(bits?|bytes?)\b([^;|]*)")),
    ];
    let (mut stated, mut problems, mut covered) = (0, Vec::new(), BTreeSet::new());
    for re in &res {
        for gm in re.captures_iter(&text) {
            let names: Vec<String> = rx(r"`([^`]+)`").captures_iter(&gm[1]).map(|c| c[1].to_string()).collect();
            let width: i64 = gm[2].parse().unwrap_or(0);
            let bits = if gm[3].starts_with("bit") { width } else { width * 8 };
            for n in &names {
                covered.insert(n.clone());
                stated += 1;
                match widths.get(n) {
                    None => problems.push(format!("`{n}` is not a wire.toml field")),
                    Some(w) if !w.contains(&Some(bits)) => {
                        problems.push(format!("`{n}` is stated at {bits} bits but wire.toml gives {}", py_list(w)));
                    }
                    Some(_) => {}
                }
            }
            if !rx(UNIT_AFTER).is_match(&gm[4]) {
                problems.push(format!("no unit in the row for {} (write e.g. (N bits, ns) or unit: none (reason))", names.join(", ")));
            }
        }
    }
    for c in rx(r"`([^`]+)`").captures_iter(&text) {
        if widths.contains_key(&c[1]) && !covered.contains(&c[1]) {
            problems.push(format!("`{}` is named in the row without its width", &c[1]));
        }
    }
    (stated, problems)
}

fn constant_rows(inp: &Inputs, trace: &Trace) -> Vec<Row> {
    inp.wire
        .tables_with(trace.span, RowSplit::Plain)
        .into_iter()
        .filter(|t| t.header.is_some() && t.head_norm()[0].contains("constant"))
        .flat_map(|t| t.rows)
        .collect()
}

/// AC2: every listed constant in the CONSTANT cell of a row whose mapping
/// resolves; wire-visible ones with a field, width and unit in the row;
/// "not wire-visible" with a reason of three words.
pub fn ac2(inp: &Inputs, trace: Option<&Trace>, lists: &Lists, report: &mut Report) {
    let table = trace.map(|t| constant_rows(inp, t)).unwrap_or_default();
    if trace.is_some() && table.is_empty() {
        report.fail("AC2", "the Traceability section has no constants table (a table whose first header cell is 'Constant')");
    }
    let mut by_id: BTreeMap<u32, Vec<&Row>> = BTreeMap::new();
    for row in &table {
        for i in expand_constant_ids(row.cell(0)) {
            by_id.entry(i).or_default().push(row);
        }
    }
    for row in &table {
        if !expand_constant_ids(row.cell(0)).is_disjoint(&lists.visible) {
            for p in width_problems(row, &inp.widths).1 {
                report.fail_at("AC2", WIRE_MD, row.line + 1, format!("wire-visible row '{}': {p}", head(row.cell(0), 40)));
            }
        }
    }
    let norms = inp.wire.head_norms();
    for &i in &lists.all {
        let Some(rows) = by_id.get(&i) else {
            report.fail("AC2", format!("constant {} of the ticket's list is not named in the CONSTANT cell of any row of the constants table (a mention elsewhere does not count)", cid(i)));
            continue;
        };
        let resolved: Vec<&&Row> = rows.iter().filter(|r| reference_resolves(&r.rest(), &norms)).collect();
        if resolved.is_empty() {
            report.fail("AC2", format!("constant {}: no row naming it in the constants table has a mapping cell that resolves to a heading of wire.md", cid(i)));
            continue;
        }
        let ok = resolved.iter().any(|r| {
            let (stated, probs) = width_problems(r, &inp.widths);
            stated > 0 && probs.is_empty()
        });
        if lists.visible.contains(&i) && !ok {
            report.fail("AC2", format!("wire-visible constant {} has no row stating its field with a width and a unit in the row (form: `name` (N bits, unit))", cid(i)));
        }
    }
    not_visible_reasons(trace, report);
}

fn not_visible_reasons(trace: Option<&Trace>, report: &mut Report) {
    let Some(trace) = trace else { return };
    for row in &trace.rows {
        let text = row.joined();
        let Some(m) = rx(r"(?i)not wire[- ]visible|not on the wire").find(&text) else { continue };
        let ids = expand_constant_ids(&text);
        let Some(first) = ids.first() else { continue };
        let rest = rx(r"^[\s:;,\-–—(]+").replace(&text[m.end()..], "").into_owned();
        if rest.split_whitespace().count() < 3 {
            report.fail_at("AC2", WIRE_MD, row.line + 1, format!("a row says {} is not wire-visible without a reason of at least three words", cid(*first)));
        }
    }
}
