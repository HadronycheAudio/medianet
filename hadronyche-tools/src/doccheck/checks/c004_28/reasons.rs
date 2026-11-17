//! The reason rules of an invalid form (004-28's copy of the rules it
//! shares with 004-25; no port imports another).

use super::merged::Merged;
use crate::doccheck::Report;
use crate::doccheck::rejection_members::reasons::{CAMEL_DIGITS, NON_REJECTION_004_25};
use crate::doccheck::text::{has, has_i, head, rx};

/// The forms that may be silent.
pub(crate) const NEUTRAL: &str = r"repeat|unknown (section )?(type|code)|older|skipped";

/// The reason tokens of 004-24 to 004-29 except 004-27.
pub(crate) fn tokens(s: &str) -> Vec<String> {
    rx(CAMEL_DIGITS).find_iter(s).map(|m| m.as_str().to_string()).collect()
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

