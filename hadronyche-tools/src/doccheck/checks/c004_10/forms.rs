//! AC3 (at least 3 invalid forms with member outcomes, no reason outside
//! the enumeration in the units or the prose) and AC4 (no field identifies
//! a receiver or destination, rule 8) of 004-10.

use super::Ctx;
use crate::doccheck::Report;
use super::wp5::reasons::{non_members, reasons_in};
use super::wp5::md::{is_field_table, parse_tables};
use super::wp5::py::{head, py_strip, si};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::rx;

/// (label, pattern, the extra pattern, the reasons allowed).
const FORMS: [(&str, &str, Option<&str>, &[&str]); 3] = [
    (
        "flag set but the extension's bytes missing",
        r"flag|presence|present|bit\s*4",
        Some(r"missing|short|fewer|truncat|absent|runs?\s+past|not\s+fill|do(?:es)?\s+not\s+fit"),
        &["BadLength", "TooShort"],
    ),
    ("hop count beyond the maximum", r"\bhop\b", Some(r"beyond|above|exceed|greater|more\s+than|over\s+the|larger"), &["BadLength"]),
    ("reserved bits set", "reserved", None, &["ReservedNonZero"]),
];

/// The rows of the non-field tables, then the bullet and numbered lines.
fn vunits(ctx: &Ctx) -> Vec<String> {
    let mut out = Vec::new();
    for (_, header, rows) in parse_tables(&ctx.section) {
        if !is_field_table(&header) {
            out.extend(rows.iter().map(|r| r.join(" | ")));
        }
    }
    for line in splitlines(&ctx.prose) {
        let st = py_strip(line);
        if rx(r"^[-*]\s+").is_match(st) || rx(r"^\d+[.)]\s+").is_match(st) {
            out.push(st.to_string());
        }
    }
    out
}

/// AC3.
pub(crate) fn ac3(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    let units = vunits(ctx);
    let mut with_reason = 0;
    for (label, pattern, extra, allowed) in FORMS {
        let Some(hit) = units.iter().find(|u| si(pattern, u) && extra.is_none_or(|x| si(x, u))) else {
            report.fail("AC3", format!("no invalid form listed for: {label}"));
            continue;
        };
        let reasons = reasons_in(&ctx.members, hit);
        if reasons.is_empty() {
            report.fail("AC3", format!("the invalid form {label:?} carries no member of the closed enumeration: {}", head(hit, 110)));
            continue;
        }
        if !reasons.iter().any(|r| allowed.contains(&r.as_str())) {
            report.fail("AC3", format!("the invalid form {label:?} must be rejected under {}: {}", allowed.join(" or "), head(hit, 110)));
            continue;
        }
        with_reason += 1;
    }
    let scanned: Vec<String> = units.iter().chain(ctx.prose_blks.iter()).cloned().collect();
    let outside = non_members(&ctx.members, &scanned);
    if !outside.is_empty() {
        report.fail("AC3", format!("the section names rejection reason(s) outside the closed enumeration: {}", outside.into_iter().collect::<Vec<_>>().join(", ")));
    }
    if with_reason < 3 {
        report.fail("AC3", format!("only {with_reason} invalid form(s) carry a member outcome; at least 3 are required"));
    }
}

/// AC4: a prose block, with the next one, says no field identifies a
/// receiver or destination (rule 8).
pub(crate) fn ac4(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    let b = &ctx.prose_blks;
    let ok = (0..b.len()).any(|k| {
        let window = format!("{}\n{}", b[k], b.get(k + 1).map_or("", String::as_str));
        si(r"no\s+field|none\s+of\s+the\s+fields|no\s+field\s+of", &window) && si("receiver|destination", &window) && si(r"rule\s+8", &window)
    });
    if !ok {
        report.fail("AC4", "no statement that no field identifies a receiver or destination (rule 8)");
    }
}
