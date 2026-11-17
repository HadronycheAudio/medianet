//! AC5: at least five invalid forms of the new types, with outcomes naming
//! control reasons.

use super::merged::Merged;
use crate::doccheck::Report;
use crate::doccheck::family004::tables;
use crate::doccheck::rejection_members::reasons::{CAMEL_DIGITS, NON_REJECTION_004_25};
use crate::doccheck::text::{has, has_i, head, rx};

/// The forms that may be silent.
pub(crate) const NEUTRAL: &str = r"repeat|unknown (section )?(type|code)|older|skipped";

/// The reason tokens of 004-24 to 004-29 except 004-27.
pub(crate) fn tokens(s: &str) -> Vec<String> {
    rx(CAMEL_DIGITS).find_iter(s).map(|m| m.as_str().to_string()).collect()
}

/// (form, outcome) of every row of the `Invalid form | Required outcome`
/// tables of `stamp` except the merged one (the table with a "too short"
/// row).
pub(crate) fn new_rows(stamp: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for tb in tables(stamp) {
        let h = tb.head();
        let valid = h.first().is_some_and(|x| x.to_lowercase().starts_with("invalid form")) && h.len() > 1 && h[1].to_lowercase().contains("required outcome");
        if !valid {
            continue;
        }
        if tb.rows.iter().any(|r| r.cell(0).to_lowercase().contains("too short")) {
            continue;
        }
        out.extend(tb.rows.iter().filter(|r| r.cells.len() >= 2).map(|r| (r.cell(0).to_string(), r.cell(1).to_string())));
    }
    out
}

/// One row's reason rules; `ac` is the criterion they report under.
pub(crate) fn row_rules(form: &str, outcome: &str, merged: &Merged, ac: &str, report: &mut Report) {
    let names: Vec<String> = tokens(outcome).into_iter().filter(|t| merged.is_member(t)).collect();
    let bad: Vec<&String> = names.iter().filter(|n| !merged.is_control(n)).collect();
    if !bad.is_empty() {
        report.fail(ac, format!("an invalid form names a reason outside the control layer {bad:?}: {}", head(form, 50)));
        return;
    }
    if names.is_empty() && !(NON_REJECTION_004_25.is_match(outcome) && has_i(NEUTRAL, form)) {
        report.fail(ac, format!("a malformed form names no Reject reason (only repeats, unknown codes and older versions may be silent): {}", head(form, 60)));
        return;
    }
    let fl = form.to_lowercase();
    let named = |n: &str| names.iter().any(|x| x == n);
    if has("reserved|non-?zero", &fl) && !named("ReservedNonZero") {
        report.fail(ac, format!("a reserved-bit form must name ReservedNonZero: {}", head(form, 60)));
    }
    if fl.contains("length") && !named("BadLength") {
        report.fail(ac, format!("a length form must name BadLength (Work 5): {}", head(form, 60)));
    }
    if fl.contains("count") && has("above|exceed|more than|maximum", &fl) && !named("BadLength") {
        report.fail(ac, format!("a count-above-maximum form must name BadLength: {}", head(form, 60)));
    }
}

/// AC5.
pub(crate) fn ac5(stamp: &str, merged: &Merged, report: &mut Report) {
    let rows = new_rows(stamp);
    report.check("AC5", rows.len() >= 5, format!("at least 5 invalid forms of the section types have a required outcome (found {})", rows.len()));
    for (form, outcome) in &rows {
        row_rules(form, outcome, merged, "AC5", report);
    }
    let cats = [
        ("unknown enumeration value", "unknown|not one of|outside|not assigned|undefined"),
        ("count above its maximum", "count|above|exceed|more than|maximum"),
        ("length inconsistent", "length"),
        ("reserved bits non-zero", "reserved|non-?zero"),
    ];
    for (cat, pat) in cats {
        report.check("AC5", rows.iter().any(|(f, _)| has_i(pat, f)), format!("an invalid form covers: {cat} (Work 5)"));
    }
}
