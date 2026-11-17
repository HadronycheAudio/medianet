//! AC4 of 004-08: at least 5 invalid forms with member reasons, including
//! the ticket's named forms.

use super::Ctx;
use crate::doccheck::Report;
use super::wp5::reasons::{non_members, reasons_in};
use super::wp5::md::{is_field_table, parse_tables};
use super::wp5::py::{head, py_strip, si};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::rx;

/// (label, pattern, the reason it must carry).
const FORMS: [(&str, &str, Option<&str>); 6] = [
    ("packet index >= packet count", r"packet\s+index[^.\n]{0,80}(?:≥|>=|outside|beyond|not\s+less\s+than|at\s+least|exceed)", Some("BadPacketIndex")),
    ("packet count = 0", r"packet\s+count[^.\n]{0,40}(?:=\s*0\b|\bis\s+0\b|of\s+0\b|zero\b)", None),
    ("frames = 0 or above maximum", r"frames?[^.\n]{0,80}(?:\b0\b|zero|above|maximum|over\s+the|exceed)", None),
    ("frames inconsistent with payload length", r"frames?[^.\n]{0,120}(?:payload|length|inconsist)", Some("BadLength")),
    ("packet count inconsistent across one slice", r"(?:packet\s+count|count)[^.\n]{0,100}(?:inconsist|disagree|differ|across\s+the\s+packets)", None),
    ("position not a multiple, when a rule applies", r"position[^.\n]{0,80}(?:multiple|align)", None),
];

/// The validity units: rows of the section's tables (those with a
/// field-table header left out when a field table was found), then the
/// bullet and numbered lines of the prose.
fn units(ctx: &Ctx) -> Vec<String> {
    let mut out = Vec::new();
    for (_, header, rows) in parse_tables(&ctx.section) {
        if ctx.field.is_some() && is_field_table(&header) {
            continue;
        }
        out.extend(rows.iter().map(|r| r.join(" | ")));
    }
    for line in splitlines(&ctx.prose) {
        let st = py_strip(line);
        if rx(r"^[-*]\s+").is_match(st) || rx(r"^\d+[.)]\s+").is_match(st) {
            out.push(st.to_string());
        }
    }
    out
}

/// AC4.
pub(crate) fn ac4(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    let units = units(ctx);
    let mut with_reason = 0;
    for (k, (label, pattern, need)) in FORMS.iter().enumerate() {
        let optional = k == 5;
        let Some(hit) = units.iter().find(|u| si(pattern, u)) else {
            if !optional {
                report.fail("AC4", format!("no invalid form listed for: {label}"));
            }
            continue;
        };
        let reasons = reasons_in(&ctx.members, hit);
        if let Some(n) = need
            && !reasons.iter().any(|r| r == n)
        {
            report.fail("AC4", format!("the invalid form {label:?} must be rejected under {n}: {}", head(hit, 110)));
            continue;
        }
        if reasons.is_empty() {
            report.fail(
                "AC4",
                format!("the invalid form {label:?} carries no member of the closed enumeration (expected one of {}): {}", ctx.members.join(", "), head(hit, 110)),
            );
            continue;
        }
        with_reason += 1;
    }
    let outside = non_members(&ctx.members, &units);
    if !outside.is_empty() {
        report.fail("AC4", format!("the validity rules name reason(s) outside the closed enumeration: {}", outside.into_iter().collect::<Vec<_>>().join(", ")));
    }
    if with_reason < 5 {
        report.fail("AC4", format!("only {with_reason} invalid form(s) carry a member rejection reason; at least 5 are required"));
    }
}
