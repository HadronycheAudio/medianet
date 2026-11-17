//! AC1 of 004-09: the container (presence flag, count, type and length
//! fields, skip by length), the named future position of 64 bits, and per
//! type a code, a field table and a byte size.

use std::collections::BTreeMap;

use super::{Ctx, small_ints};
use crate::doccheck::Report;
use super::wp5::big::Big;
use super::wp5::md::width_to_bits;
use super::wp5::py::{find_i, py_int, si};

/// The four types and their patterns.
pub(crate) const TYPES: [(&str, &str); 4] = [
    ("end-of-stream", r"end[\s_-]*of[\s_-]*stream"),
    ("re-alignment", r"re[\s_-]*alignment"),
    ("timeline epoch switch", r"epoch[\s_-]*switch"),
    ("transport reference change", r"transport[\s_-]*reference"),
];

fn first_group(g: Vec<Option<String>>) -> Big {
    py_int(&g.into_iter().flatten().find(|x| !x.is_empty()).expect("a group")).expect("digits")
}

/// The container rows, the skip rule, the position, the future.
fn container(ctx: &Ctx, report: &mut Report) -> Option<Vec<Vec<String>>> {
    let p = &ctx.prose;
    if !si(r"bit\s*5|announcement\s+present", p) || !si(r"004-06|flags?\s+byte", p) {
        report.fail("AC1", "the container does not tie the presence flag to the 004-06 flags byte (announcement present, bit 5)");
    }
    let rows = ctx
        .field_tables
        .iter()
        .find(|(text, _, intro)| {
            let j = format!("{text}\n{intro}");
            si(r"\bcount\b", &j) && si(r"\btype\b", &j) && si(r"\blength\b", &j)
        })
        .map(|t| t.1.clone())
        .filter(|r| !r.is_empty());
    if rows.is_none() {
        report.fail("AC1", "no container field table with count, per-announcement type and per-announcement length fields");
    }
    if !si("skip", p) || !si("length", p) {
        report.fail("AC1", "the section does not state the skip-by-length requirement (004-04's minor-change rule: a length prefix per announcement, so unknown types are skippable by a same-major receiver)");
    } else if !si(r"same\s+major|minor|004-04", p) {
        report.fail("AC1", "the skip-by-length rule is not tied to the version rule (same-major receiver, 004-04's minor change)");
    }
    let pos = ctx.field_tables.iter().flat_map(|t| t.1.iter()).any(|row| {
        si(r"\bposition\b", &row.join(" | ")) && row.get(1).and_then(|c| width_to_bits(c)) == Some(Big::from(64))
    });
    if !pos {
        report.fail("AC1", "no announcement field table carries the named future position as 64 bits (the slice header's position form, u64 / 8 bytes)");
    }
    if !si("future", p) {
        report.fail("AC1", "the section never says the named position is a FUTURE position (the ticket: each announcement names a future position, as in the slice header)");
    }
    rows
}

/// A type code stated in one of the units naming the type.
fn code_in_units(ctx: &Ctx, pattern: &str) -> bool {
    for u in ctx.units.iter().filter(|u| si(pattern, u)) {
        if let Some((_, _, g)) = find_i(r"type\s+code\s*(?:is|:|=)?\s*(\d{1,3})\b", u)
            && py_int(g[0].as_deref().unwrap_or("")).is_some_and(|v| v <= Big::from(255))
        {
            return true;
        }
        if small_ints(u).iter().any(|&v| v <= 255) && si(r"\bcode\b", u) {
            return true;
        }
    }
    false
}

/// AC1; returns the container rows.
pub(crate) fn ac1(ctx: &Ctx, report: &mut Report) -> Option<Vec<Vec<String>>> {
    if !ctx.open {
        return None;
    }
    let rows = container(ctx, report);
    let global = find_i(
        r"each\s+announcement[^.;\n]{0,80}?(\d+)\s*bytes?|announcement[^.;\n]{0,60}?\bis\s+(\d+)\s*bytes?|(\d+)\s*bytes?\s+per\s+announcement",
        &ctx.prose,
    )
    .map(|(_, _, g)| first_group(g));
    let mut code_hits: BTreeMap<&str, ()> = BTreeMap::new();
    for row in ctx.tables.iter().flat_map(|t| t.2.iter()) {
        let joined = row.join(" | ");
        if !small_ints(&joined).iter().any(|&v| v <= 255) {
            continue;
        }
        for (label, pattern) in TYPES {
            if si(pattern, &joined) {
                code_hits.insert(label, ());
            }
        }
    }
    for (label, pattern) in TYPES {
        if !ctx.units.iter().any(|u| si(pattern, u)) {
            report.fail("AC1", format!("the section never names the {label} announcement type"));
            continue;
        }
        if !code_hits.contains_key(label) && !code_in_units(ctx, pattern) {
            report.fail("AC1", format!("the {label} announcement has no stated TYPE CODE (an integer in a code table row, or 'type code N' beside the name)"));
        }
        let table = ctx.field_tables.iter().find(|(text, _, intro)| si(pattern, &format!("{text}\n{intro}")));
        if table.is_none() {
            report.fail("AC1", format!("the {label} announcement has no field table (Offset/Width/Type/Meaning/Valid range) of its own"));
        }
        let sized = global.is_some() || table.is_some_and(|(text, _, intro)| si(r"(\d+)\s*bytes?", &format!("{text}\n{intro}")));
        if !sized {
            report.fail("AC1", format!("the {label} announcement has no stated BYTE SIZE (state it in the type's table or as a per-announcement size covering all four types)"));
        }
    }
    rows
}
