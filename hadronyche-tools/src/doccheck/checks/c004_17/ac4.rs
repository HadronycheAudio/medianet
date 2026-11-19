//! AC4: invalid forms with outcomes, per message (shared table plus its
//! own, at least 6), the ticket's list covered, exact reasons, member or
//! non-rejection outcomes, no invented reason (script lines 623-711).

use std::collections::BTreeSet;

use super::Ctx;
use crate::doccheck::Report;
use crate::doccheck::rejection_members::Enumeration;
use crate::doccheck::rejection_members::reasons::{CAMEL_LETTERS, LOOKBACK_004_16, NON_REJECTION_004_16, named_reasons};
use crate::doccheck::text::{has, has_i, head, rx};

const ELECTION_CATS: [&str; 3] = ["epoch", "eligibility", "timestate"];

/// `classify(form)` (004-17's categories).
pub(crate) fn classify(form: &str) -> Option<&'static str> {
    let f = form.to_lowercase();
    let has = |p: &str| has(p, &f);
    Some(if has(r"too\s+short|shorter|fewer\s+than") {
        "short"
    } else if f.contains("magic") {
        "magic"
    } else if f.contains("version") {
        "version"
    } else if has(r"domain\s*(id\s*)?0\b|unconfigured") {
        "domain0"
    } else if has(r"foreign|different\s+non-?zero") {
        "foreign"
    } else if has(r"oversiz|larger\s+than") {
        "oversize"
    } else if f.contains("epoch") {
        "epoch"
    } else if f.contains("eligib") {
        "eligibility"
    } else if has(r"time[\s-]*state") {
        "timestate"
    } else if has(r"\bsize\b|length") {
        "length"
    } else if has(r"reserved|non-?zero|scheme|\btag\b") {
        "reserved"
    } else {
        return None;
    })
}

fn exact(cat: &str) -> Option<&'static str> {
    Some(match cat {
        "short" => "TooShort",
        "magic" => "BadMagic",
        "version" => "VersionMismatch",
        "domain0" => "UnconfiguredDomain",
        "foreign" => "ForeignDomain",
        "oversize" => "OversizeControl",
        "length" => "BadLength",
        "reserved" => "ReservedNonZero",
        _ => return None,
    })
}

/// `check_form(form, outcome, where)`.
fn check_form(form: &str, outcome: &str, place: &str, members: &Enumeration, seen: &mut BTreeSet<&'static str>, report: &mut Report) {
    for h in rx(r"count\s+under\s+`?([A-Za-z]+)`?").captures_iter(outcome) {
        if !members.contains(h.get(1).map_or("", |m| m.as_str())) {
            report.fail("AC4", format!("{place}: the form '{}' counts under '{}', which is not in the closed enumeration", head(form, 50), h.get(1).map_or("", |m| m.as_str())));
        }
    }
    for h in rx(CAMEL_LETTERS).find_iter(outcome) {
        let name = h.as_str();
        if !members.contains(name) && !has(&format!(r"\b(not|never|nor|no)\b[^|;.]*{}", regex::escape(name)), outcome) {
            report.fail("AC4", format!("{place}: the outcome names '{name}', which is not in the closed enumeration: no invented reason"));
        }
    }
    let cat = classify(form);
    if let Some(c) = cat {
        seen.insert(c);
    }
    let named = named_reasons(outcome, |w| members.contains(w), LOOKBACK_004_16);
    let argued = !named.is_empty() || NON_REJECTION_004_16.is_match(outcome);
    match cat.and_then(exact) {
        Some(reason) => {
            if !has(&format!(r"\b{reason}\b"), outcome) {
                report.fail("AC4", format!("{place}: the form '{}' must be counted under {reason}: {}", head(form, 50), head(outcome, 70)));
            }
            if cat == Some("domain0") && named.iter().any(|n| n == "ForeignDomain") {
                report.fail("AC4", format!("{place}: domain 0 is mapped to ForeignDomain; it is ignored and counted as UnconfiguredDomain"));
            }
            if cat == Some("version") && !has_i(r"\bevent\b", outcome) {
                report.fail("AC4", format!("{place}: the version mismatch row does not say that it raises a named event"));
            }
        }
        None if cat.is_some_and(|c| ELECTION_CATS.contains(&c)) => {
            if !argued {
                report.fail("AC4", format!("{place}: the election form '{}' needs a member reason or an argued non-rejection", head(form, 50)));
            }
        }
        None => {
            if !argued {
                report.fail("AC4", format!("{place}: the form '{}' names no member of the closed enumeration and argues no non-rejection", head(form, 50)));
            }
        }
    }
}

/// AC4.
pub(crate) fn forms(ctx: &Ctx, report: &mut Report) {
    let mut seen = BTreeSet::new();
    for m in &ctx.msgs {
        let total = ctx.shared.len() + m.forms.len();
        if total < 6 {
            report.fail("AC4", format!("'{}' has {total} invalid forms with outcomes (the shared table and its own); at least 6 are required", m.title));
        }
        for (f, o) in &m.forms {
            check_form(f, o, &format!("'{}'", m.title), &ctx.members, &mut seen, report);
        }
    }
    for (f, o) in &ctx.shared {
        check_form(f, o, "the shared Validity rules", &ctx.members, &mut seen, report);
    }
    if ctx.shared.is_empty() && !ctx.msgs.iter().any(|m| !m.forms.is_empty()) {
        report.fail("AC4", "no invalid-form table with 'Form' and 'Outcome' columns in the Election messages section");
    }
    for c in ["short", "magic", "version", "domain0", "foreign", "oversize", "length", "reserved", "epoch", "eligibility", "timestate"] {
        if !seen.contains(c) {
            report.fail("AC4", format!("no invalid form covers the '{c}' case of the ticket's list"));
        }
    }
}
