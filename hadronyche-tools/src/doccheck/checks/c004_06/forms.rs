//! AC4 of 004-06: at least 10 invalid forms, each with a member reason (or
//! the VersionMismatch named event), every form the ticket names, and no
//! reason outside the enumeration.


use super::Ctx;
use super::wp5::md::parse_tables;
use super::wp5::reasons::{non_members, reasons_in};
use super::wp5::py::{head, py_strip, si};
use crate::doccheck::Report;
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::rx;

/// The invalid forms the ticket names, with the script's patterns.
const FORMS: [(&str, &str); 11] = [
    ("shorter than the header (TooShort)", r"shorter\s+than\s+the\s+header|shorter\s+than|too\s+short|fewer\s+bytes\s+than|less\s+than\s+the\s+header|below\s+the\s+header\s+size"),
    ("bad magic (BadMagic)", r"bad\s+magic|wrong\s+magic|magic\s+mismatch|magic\s+(?:is\s+)?(?:not|wrong)|incorrect\s+magic|magic\s+does\s+not\s+match|magic\s+differs"),
    ("version mismatch (VersionMismatch)", r"version\s+mismatch|wrong\s+version|version\s+(?:is\s+)?(?:not|other\s+than|differs)|different\s+major|other\s+major|major\s+differs"),
    ("foreign domain (ForeignDomain)", r"foreign\s+domain|domain[^.\n]{0,30}(?:≠|!=|is\s+not|other\s+than|differs\s+from)\s*(?:the\s+)?local|different\s+(?:non-?zero\s+)?domain|another\s+domain|other\s+than\s+the\s+local\s+domain|non-?local\s+domain"),
    ("domain 0 (UnconfiguredDomain)", r"domain\s*(?:id\s*)?(?:is\s*)?(?:=|is)?\s*0\b|domain\s+zero|unconfigured\s+domain|domain\s*=\s*0\b"),
    ("unknown stream (UnknownStream)", r"unknown\s+stream|stream\s+(?:number\s+)?(?:is\s+)?unknown|no\s+(?:such\s+)?stream|stream\s+[^.\n]{0,30}unknown|not\s+a\s+known\s+stream"),
    ("unadmitted source (UnadmittedSource)", r"unadmitted\s+source|source\s+not\s+admitted|not\s+admitted|no\s+admitted\s+connection|source\s+is\s+not\s+admitted"),
    ("wrong epoch (WrongEpoch)", r"wrong\s+epoch|epoch\s+(?:is\s+)?not\s+current|other\s+epoch|epoch\s+mismatch|stale\s+epoch|epoch\s+differs|not\s+the\s+current\s+epoch"),
    ("reserved non-zero (ReservedNonZero)", r"reserved[^.\n]{0,40}non-?zero|non-?zero[^.\n]{0,20}reserved|reserved\s+bits?[^.\n]{0,25}(?:not\s+zero|set|nonzero)|reserved[^.\n]{0,20}\bset\b"),
    ("gap flag with non-empty payload", r"gap[^.\n]{0,60}payload|payload[^.\n]{0,60}gap"),
    ("invalid flags combination", r"flag[^.\n]{0,60}combin|combin\w+[^.\n]{0,60}flag|invalid\s+flags?|flags\s+combination|inconsistent\s+flags|illegal\s+flag|forbidden\s+flag"),
];

/// The validity units: rows of every table of the section but the field
/// table, then the bullet and numbered lines of the prose.
fn units(ctx: &Ctx) -> Vec<String> {
    let mut out = Vec::new();
    for (tstart, _, rows) in parse_tables(&ctx.section) {
        if ctx.field_table_found && Some(tstart) == ctx.field_table_start {
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
    if !ctx.open() {
        return;
    }
    let units = units(ctx);
    let mut with_reason = 0;
    for (label, pattern) in FORMS {
        let Some(hit) = units.iter().find(|u| si(pattern, u)) else {
            report.fail("AC4", format!("no invalid form listed for: {label}"));
            continue;
        };
        let reasons = reasons_in(&ctx.members, hit);
        let event = si(r"named\s+event|\bevent\b", hit);
        if reasons.is_empty() && !(event && label.starts_with("version")) {
            report.fail(
                "AC4",
                format!("the invalid form {label:?} carries no member of the closed enumeration (expected one of {}): {}", ctx.members.join(", "), head(hit, 110)),
            );
            continue;
        }
        if !reasons.is_empty() {
            with_reason += 1;
        }
        if label.starts_with("version mismatch") && !event {
            report.fail("AC4", "VersionMismatch is listed without its named event (004-03: a version mismatch is never a silent drop; it raises the VersionMismatch event in addition to being counted)");
        }
        if (label.starts_with("gap flag") || label.starts_with("invalid flags")) && reasons.is_empty() {
            report.fail("AC4", format!("the invalid form {label:?} is named by the ticket but carries neither a member reason nor a named event: {}", head(hit, 110)));
        }
    }
    let outside = non_members(&ctx.members, &units);
    if !outside.is_empty() {
        report.fail(
            "AC4",
            format!("the validity rules name reason(s) outside the closed enumeration: {}", outside.into_iter().collect::<Vec<_>>().join(", ")),
        );
    }
    if with_reason < 10 {
        report.fail("AC4", format!("only {with_reason} invalid form(s) carry a member rejection reason; at least 10 are required"));
    }
}
