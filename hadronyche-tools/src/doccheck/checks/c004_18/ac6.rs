//! AC6: invalid forms with outcomes over the two closed enumerations, and
//! the open item (script lines 1074-1175).

use std::collections::BTreeSet;

use crate::doccheck::wp7::common::is_open_item;
use crate::doccheck::wp7::py;
use super::{CAMEL, Ctx, DISCOVERY};
use crate::doccheck::Report;

/// 004-18's `NON_REJECTION` (with `never a miss` and `not a refusal`).
pub(crate) const NON_REJECTION: &str = r"not\s+a\s+(?:parse\s+)?rejection|never\s+a\s+rejection|is\s+not\s+counted|cannot\s+occur|ignored|never\s+a\s+miss|not\s+a\s+refusal";

/// `classify(form)` (004-18's categories).
pub(crate) fn classify(form: &str) -> Option<&'static str> {
    let f = form.to_lowercase();
    let has = |p: &str| py::has(p, &f);
    let refus = has("refus");
    Some(if refus && py::has_i(r"VersionMismatch|NameTaken|ModuleCap|version\s+mismatch|name\s+taken|module\s+cap", form) {
        "discovery"
    } else if refus && has(r"unknown|outside|not\s+in|not\s+a\s+(known|member|valid)") {
        "unknown_reason"
    } else if f.contains("downstream") {
        "downstream"
    } else if f.contains("address") {
        "address"
    } else if f.contains("member") || f.contains("duplicate") {
        "member"
    } else if has(r"\brate") {
        "rate"
    } else if has(r"too\s+short|shorter|fewer\s+than") {
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

/// `named_in(outcome)`: names of either enumeration not negated in the 24
/// characters before them.
pub(crate) fn named_in(outcome: &str, ctx: &Ctx) -> Vec<String> {
    let mut found = Vec::new();
    for m in py::finditer(r"\b[A-Za-z]+\b", outcome, false) {
        let w = m.all();
        if !ctx.in_union(w) {
            continue;
        }
        let at = py::char_index(outcome, m.start);
        let before = py::slice(outcome, at.saturating_sub(24), at);
        if py::has_i(r"\b(not|never|nor|no|neither)\b[^|;.]*$", before) {
            continue;
        }
        found.push(w.to_string());
    }
    found
}

fn check_form(form: &str, outcome: &str, place: &str, ctx: &Ctx, seen: &mut BTreeSet<&'static str>, report: &mut Report) {
    for h in py::finditer(r"count\s+under\s+`?([A-Za-z]+)`?", outcome, false) {
        if !ctx.in_union(h.g(1)) {
            report.fail("AC6", format!("{place}: the form '{}' counts under '{}', which is in neither closed enumeration", py::head(form, 50), h.g(1)));
        }
    }
    for h in py::finditer(CAMEL, outcome, false) {
        let name = h.all();
        if !ctx.in_union(name) && !py::has(&format!(r"\b(not|never|nor|no)\b[^|;.]*{}", regex::escape(name)), outcome) {
            report.fail("AC6", format!("{place}: the outcome names '{name}', which is in neither enumeration: no invented reason"));
        }
    }
    let cat = classify(form);
    if let Some(c) = cat {
        seen.insert(c);
    }
    let named = named_in(outcome, ctx);
    if let Some(reason) = cat.and_then(exact)
        && !py::has(&format!(r"\b{reason}\b"), outcome)
    {
        report.fail("AC6", format!("{place}: the form '{}' must be counted under {reason}: {}", py::head(form, 50), py::head(outcome, 70)));
    }
    if cat == Some("member") && py::has_i("32|unknown|duplicate", form) && !outcome.contains("BadMemberList") {
        report.fail("AC6", format!("{place}: the member form '{}' must be refused with BadMemberList (C-173, C-185)", py::head(form, 50)));
    }
    if cat == Some("discovery") && named.iter().any(|n| DISCOVERY.contains(&n.as_str())) {
        report.fail("AC6", format!("{place}: the form '{}' is refused with a discovery-raised reason: VersionMismatch, NameTaken and ModuleCap are never an admission refusal", py::head(form, 50)));
    }
    if named.is_empty() && !py::has_i(NON_REJECTION, outcome) {
        report.fail("AC6", format!("{place}: the form '{}' names no member of the closed enumerations and argues no non-rejection", py::head(form, 50)));
    }
}

/// AC6 and the open item.
pub(crate) fn forms(ctx: &Ctx, report: &mut Report) {
    let mut seen = BTreeSet::new();
    for m in &ctx.msgs {
        let total = ctx.shared.len() + m.forms.len();
        if total < 6 {
            report.fail("AC6", format!("'{}' has {total} invalid forms with outcomes (the shared table and its own); at least 6", m.title));
        }
        for (f, o) in &m.forms {
            check_form(f, o, &format!("'{}'", m.title), ctx, &mut seen, report);
        }
    }
    for (f, o) in &ctx.shared {
        check_form(f, o, "the shared Validity rules", ctx, &mut seen, report);
    }
    let all = ctx.shared.len() + ctx.msgs.iter().map(|m| m.forms.len()).sum::<usize>();
    if all < 8 {
        report.fail("AC6", format!("the admission set has {all} invalid forms with outcomes; at least 8 are required (AC 6)"));
    }
    for c in ["short", "magic", "version", "foreign", "length", "reserved", "oversize", "domain0", "rate", "member", "address", "unknown_reason", "discovery", "downstream"] {
        if !seen.contains(c) {
            report.fail("AC6", format!("no invalid form covers the '{c}' case of the ticket's list"));
        }
    }
    let owned = ctx.ad_paras.iter().filter(|t| is_open_item(t)).any(|t| {
        py::has_i("owner", t) && py::has(r"\b004\b", t) && py::has(r"\b005\b", t) && py::has_i("falsifier", t) && py::has_i(r"needs\s+a\s+message\s+or\s+field", t)
    });
    if !owned {
        report.fail("DoD", "no paragraph starting 'Open item' in the Admission messages section names owner 004, epic 005 as the confirming owner, and the falsifier 'epic 005's admission state machine needs a message or field the set lacks'");
    }
}
