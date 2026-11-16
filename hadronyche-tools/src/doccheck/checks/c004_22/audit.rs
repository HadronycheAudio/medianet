//! AC3: the per-receiver field audit.

use crate::doccheck::pycompat::PyStr;
use std::collections::BTreeSet;

use super::{Inputs, WIRE_MD};
use crate::doccheck::Report;
use crate::doccheck::policy::TableRules;
use crate::doccheck::text::rx;

/// What AC3 hands to the record and DoD checks.
pub struct Audit {
    /// True when a heading matches "audit".
    pub found: bool,
    /// The text of every audit section, joined.
    pub text: String,
    /// The media fields given a destination-specific or per-receiver
    /// verdict.
    pub bad_fields: Vec<String>,
}

fn cell_plain(c: &str) -> String {
    c.chars()
        .filter(|ch| *ch != '`' && *ch != '*')
        .collect::<String>()
        .py_strip()
        .to_lowercase()
}

fn names_in_field_cell(cell: &str) -> Vec<String> {
    let toks: Vec<String> = rx(r"`([^`]+)`").captures_iter(cell).map(|c| c[1].to_string()).collect();
    if !toks.is_empty() {
        return toks;
    }
    rx(r",|\band\b")
        .split(&cell_plain(cell))
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(String::from)
        .collect()
}

/// One audit row: the field names of its FIELD cell, its cells, its line.
type AuditRow = (Vec<String>, Vec<String>, usize);

/// Whether an audit section exists, the audit text, and every audit row.
fn audit_rows(inp: &Inputs) -> (bool, String, Vec<AuditRow>) {
    let heads = inp.wire.find("(?i)audit");
    let spans: Vec<_> = heads.iter().map(|h| inp.wire.section(h)).collect();
    let text = spans.iter().map(|s| inp.wire.text(*s)).collect::<Vec<_>>().join("\n");
    let mut rows = Vec::new();
    for span in &spans {
        for t in inp.wire.tables_with(*span, TableRules::T004_22) {
            let k = t.head_norm().iter().position(|h| rx(r"\bfield\b").is_match(h)).unwrap_or(0);
            for r in &t.rows {
                if r.cells.len() > k {
                    rows.push((names_in_field_cell(&r.cells[k]), r.cells.clone(), r.line + 1));
                }
            }
        }
    }
    (!heads.is_empty(), text, rows)
}

/// AC3: every media field named in the FIELD column of an audit row with a
/// verdict; a destination-specific or per-receiver verdict stops the
/// ticket; the conclusion says no such field exists.
pub fn ac3(inp: &Inputs, report: &mut Report) -> Audit {
    let expected: Vec<String> = {
        let mut out: Vec<String> = Vec::new();
        for (_, names) in &inp.media {
            for n in names {
                if !out.contains(n) {
                    out.push(n.clone());
                }
            }
        }
        out
    };
    let (found, text, rows) = audit_rows(inp);
    if !found {
        report.fail("AC3", format!("{WIRE_MD} has no section headed 'audit' (the per-receiver field audit of Work 2)"));
    }
    let covered: BTreeSet<&String> = rows.iter().flat_map(|(n, _, _)| n).collect();
    let missing: Vec<&str> = expected.iter().filter(|n| !covered.contains(n)).map(String::as_str).collect();
    if found && !missing.is_empty() {
        report.fail("AC3", format!("the audit has no row whose FIELD cell names media field(s): {}", missing.join(", ")));
    }
    let ok = rx(r"(?i)^(none|no|clear|pass|absent|ok|not (destination|per)|none found)\b");
    let bad = rx(r"(?i)^(yes,?\s*)?(destination[- ]specific|per[- ]receiver)( field)?\.?$");
    let mut bad_fields = Vec::new();
    for (names, cells, line) in &rows {
        let own: Vec<&String> = names.iter().filter(|n| expected.contains(n)).collect();
        if own.is_empty() {
            continue;
        }
        let plain: Vec<String> = cells.iter().map(|c| cell_plain(c)).collect();
        if plain.iter().any(|c| bad.is_match(c)) {
            bad_fields.extend(own.into_iter().cloned());
            continue;
        }
        if !plain.iter().any(|c| ok.is_match(c)) {
            let list: Vec<&str> = own.iter().map(|s| s.as_str()).collect();
            report.fail_at("AC3", WIRE_MD, *line, format!("the audit row for {} states no verdict (none / no destination-specific or per-receiver content)", list.join(", ")));
        }
    }
    if !bad_fields.is_empty() {
        let set: BTreeSet<&String> = bad_fields.iter().collect();
        let list: Vec<&str> = set.into_iter().map(String::as_str).collect();
        report.fail("AC3", format!("STOP: the audit finds a destination-specific or per-receiver media field ({}); epic AC-5 is violated: the ticket stops here, the field is a discrepancy to record and escalate, and no audit may pass with it", list.join(", ")));
    }
    let conclusion = rx(concat!(
        r"(?i)no\s+destination[- ]specific\s+(?:or|and|nor)\s+(?:no\s+)?(?:a\s+|any\s+)?per[- ]receiver\s+field",
        r"|no\s+per[- ]receiver\s+(?:or|and|nor)\s+(?:no\s+)?(?:a\s+|any\s+)?destination[- ]specific\s+field"
    ));
    if found && !conclusion.is_match(&text) {
        report.fail("AC3", "the audit's conclusion does not state that no destination-specific or per-receiver field exists");
    }
    Audit {
        found,
        text,
        bad_fields,
    }
}
