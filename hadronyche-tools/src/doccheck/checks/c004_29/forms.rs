//! AC4: the invalid forms of the section — at least six; the control
//! violations name their merged reason (`GENERIC`) and are not ignored; the
//! three cases the ticket names; review F3, a late announcement after an
//! explicit close.

use super::{Merged29, Section};
use crate::doccheck::Report;
use crate::doccheck::family004::tables;
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::rejection_members::reasons::{CAMEL_DIGITS, NON_REJECTION_004_29};
use crate::doccheck::items::{OpenItemPolicy, open_items};
use crate::doccheck::text::{has, has_i, head, rx};
use crate::doccheck::units::{BlockKind, prose_blocks};

/// The six control violations and the merged reason each must name.
const GENERIC: [(&str, &str); 6] = [
    ("TooShort", "too short|under 4 bytes|under four bytes|fewer than 4 bytes"),
    ("BadMagic", "bad magic"),
    ("VersionMismatch", "version mismatch"),
    ("ForeignDomain", "foreign domain"),
    ("OversizeControl", "oversize"),
    ("ReservedNonZero", "reserved non-?zero|scheme non-?zero|tag non-?zero"),
];

/// The reason tokens of an outcome.
fn tokens(s: &str) -> Vec<String> {
    rx(CAMEL_DIGITS).find_iter(s).map(|m| m.as_str().to_string()).collect()
}

/// `ROWS`: the (form, outcome) rows of every `Invalid form | Required
/// outcome` table of the section with a form.
pub(crate) fn rows(s: &Section) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for t in tables(&s.text) {
        let h = t.head();
        if h.first().is_some_and(|x| x.to_lowercase().starts_with("invalid form")) && h.len() > 1 && h[1].to_lowercase().contains("required outcome") {
            out.extend(t.rows.iter().filter(|r| r.cells.len() >= 2 && !r.cell(0).py_strip().is_empty()).map(|r| (r.cell(0).to_string(), r.cell(1).to_string())));
        }
    }
    out
}

fn one_row(form: &str, outcome: &str, m: &Merged29, report: &mut Report) {
    let names: Vec<String> = tokens(outcome).into_iter().filter(|t| m.is_member(t)).collect();
    let outside: Vec<&String> = names.iter().filter(|n| !m.is_control(n)).collect();
    if !outside.is_empty() {
        report.fail("AC4", format!("an invalid form names a reason outside the control layer {outside:?}: {}", head(form, 60)));
        return;
    }
    let generic: Vec<&str> = GENERIC.iter().filter(|(_, p)| has_i(p, form)).map(|(n, _)| *n).collect();
    for name in &generic {
        if !names.iter().any(|n| n == name) {
            report.fail("AC4", format!("the control violation '{}' must name {name}, as the merged shared rows do", head(form, 60)));
        }
        if NON_REJECTION_004_29.is_match(outcome) {
            report.fail("AC4", format!("the control violation '{}' is marked ignored or not a rejection; it is a counted rejection", head(form, 60)));
        }
        if *name == "VersionMismatch" && !has_i("event", outcome) {
            report.fail("AC4", "the version mismatch form does not raise the named VersionMismatch event");
        }
    }
    if generic.is_empty() && names.is_empty() && !NON_REJECTION_004_29.is_match(outcome) {
        report.fail("AC4", format!("the form '{}' states no outcome: no Reject reason and no non-rejection", head(form, 60)));
    }
}

/// AC4.
pub(crate) fn ac4(rows: &[(String, String)], m: &Merged29, report: &mut Report) {
    report.check("AC4", rows.len() >= 6, format!("at least 6 invalid forms with outcomes across the two messages (found {})", rows.len()));
    for (f, o) in rows {
        one_row(f, o, m, report);
    }
    let present = [
        ("the expiry out of range", "expir.*range|range.*expir|expir.*too (large|high)"),
        ("an expiry already past", "expir.*(past|already|elapsed|before)|(past|already|elapsed).*expir"),
        ("a close for an unknown window", "clos.*(unknown|not held|does not hold|no open|names no|not open)|(unknown|not held|does not hold|no open|names no).*(window|clos)"),
    ];
    for (label, pat) in present {
        let ok = rows.iter().any(|(f, o)| has_i(pat, f) && has(r"\S", o));
        report.check("AC4", ok, format!("an invalid form covers {label}, with its outcome stated (ticket item 4)"));
    }
}

/// `ALL_PARAS`: the prose blocks of the section and every table row of it,
/// joined.
pub(crate) fn all_paras(s: &Section) -> Vec<String> {
    let prose = prose_blocks(&s.text).into_iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text);
    let rows = tables(&s.text).into_iter().flat_map(|t| t.rows.into_iter().map(|r| r.cells.join(" ")));
    prose.chain(rows).collect()
}

/// Review F3.
pub(crate) fn f3(s: &Section, report: &mut Report) {
    let after = r"(?i)after (?:the |an |its |a )?(?:window'?s? )?(?:explicit )?close\b|already closed|closed window|closed by|had closed|explicitly closed";
    let hits: Vec<String> = all_paras(s).into_iter().filter(|x| has(after, x) && has_i("announc|copy|repeat", x) && has_i("ignored|reopen|remember|refus", x)).collect();
    report.check("AC4", !hits.is_empty(), "a late or repeated announcement of a window the module closed by an explicit close has a stated outcome: ignored and remembered, or reopens (Work 3 stated outcomes; C-153 closes on explicit close) (review F3)");
    let reopens = hits.iter().any(|x| has_i(r"\breopen", x) && !has_i("not reopen|does not reopen|do not reopen|never reopen", x));
    if reopens {
        let open = open_items(&s.text, OpenItemPolicy::Leading);
        let ok = open.iter().any(|i| has_i("clos", &i.text) && has_i("reopen|late|after|repeat", &i.text) && has_i("owner", &i.text) && has_i("falsifier", &i.text));
        report.check("DoD", ok, "the reopening outcome after a close is an open item with an owner and a falsifier (review F3)");
    }
}
