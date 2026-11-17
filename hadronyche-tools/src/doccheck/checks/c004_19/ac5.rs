//! AC5: at least 6 invalid forms, exact reasons by the form cell, the size
//! rows by their outcome, every named reason a member (script lines
//! 1016-1113).

use std::collections::BTreeMap;

use super::Ctx;
use crate::doccheck::Report;
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::rejection_members::reasons::{CAMEL_LETTERS, LOOKBACK_004_19, negated_at};
use crate::doccheck::text::{has, has_i, head, rx};

const SHORT_FORM: &str = r"\btoo\s+short\b|\bshorter\s+than\b|\bfewer\s+than\b|\bless\s+than\b|\bunder\s+\d+\s+(?:bytes?|octets?)\b|\bsmaller\s+than\b";
const FIXED_WORD: &str = r"fixed|not\s+equal|equal\s+to|other\s+than";
const SIZE_FORM: &str = r"\bsize\b|\bbytes?\b|\boctets?\b|\blength\b";
const CATS: [(&str, &str, bool); 6] = [
    ("ReservedNonZero", r"\bpad|reserved", true),
    ("BadMagic", "magic", true),
    ("VersionMismatch", "version", true),
    ("ForeignDomain", "foreign", true),
    ("WrongEpoch", "epoch", true),
    ("holder", "holder", false),
];

/// The rows of the first table whose header has a form/invalid cell and an
/// outcome cell (letters only, lower case).
fn form_table(ctx: &Ctx) -> Option<Vec<Vec<String>>> {
    ctx.tables
        .iter()
        .find(|t| {
            let heads: Vec<String> = t.head().iter().map(|h| rx("[^a-z]").replace_all_literal(&h.to_lowercase(), "")).collect();
            heads.iter().any(|h| h.contains("form") || h.contains("invalid")) && heads.iter().any(|h| h.contains("outcome"))
        })
        .map(|t| t.rows.iter().map(|r| r.cells.clone()).collect())
}

fn reason_tokens(rows: &[Vec<String>], ctx: &Ctx, report: &mut Report) {
    for r in rows {
        let outcome = r[1..].join(" | ");
        for h in rx(CAMEL_LETTERS).find_iter(&outcome) {
            let at = outcome[..h.start()].chars().count();
            if !ctx.members.contains(h.as_str()) && !negated_at(&outcome, at, LOOKBACK_004_19) {
                report.fail("AC5", format!("the outcome names '{}', which is not in the closed enumeration", h.as_str()));
            }
        }
    }
}

fn categories(rows: &[Vec<String>], ctx: &Ctx, report: &mut Report) {
    let mut seen: BTreeMap<&str, String> = BTreeMap::new();
    let (mut short, mut fixed) = (Vec::new(), Vec::new());
    for r in rows {
        let (form, outcome) = (&r[0], r[1..].join(" | "));
        let Some(cat) = CATS.iter().find(|c| has_i(c.1, form)) else {
            if has_i(SHORT_FORM, form) && !has_i(FIXED_WORD, form) {
                short.push((form, outcome));
            } else if has_i(SIZE_FORM, form) {
                fixed.push((form, outcome));
            }
            continue;
        };
        if seen.contains_key(cat.0) {
            continue;
        }
        seen.insert(cat.0, outcome.clone());
        if cat.2 {
            if !has(&format!(r"\b{}\b", cat.0), &outcome) {
                report.fail("AC5", format!("the invalid form '{}' must be counted under {} (the ticket names it): {}", head(form, 60), cat.0, head(&outcome, 90)));
            }
        } else if !ctx.members.names().any(|n| has(&format!(r"\b{n}\b"), &outcome)) {
            report.fail("AC5", format!("the invalid form '{}' has no member reason in its outcome", head(form, 60)));
        }
    }
    for cat in CATS {
        if cat.0 != "holder" && !seen.contains_key(cat.0) {
            report.fail("AC5", format!("no invalid form for {} (ticket list: size, bad magic, bad version, foreign domain, request padding non-zero, reply from a non-holder, wrong epoch)", cat.0));
        }
        if cat.0 == "holder" && !seen.contains_key("holder") {
            report.fail("AC5", "no invalid form for a reply from a non-holder");
        }
    }
    for (form, outcome) in &short {
        if !has(r"\bTooShort\b", outcome) || has(r"\bBadLength\b", outcome) {
            report.fail("AC5", format!("the invalid form '{}' is shorter than the minimum that carries the magic and the version: it must be counted under TooShort: {}", head(form, 60), head(outcome, 90)));
        }
    }
    for (form, outcome) in &fixed {
        if has(r"\bTooShort\b", outcome) {
            report.fail("AC5", format!("the invalid form '{}' is a size that is not the fixed size, but it is mapped to TooShort; it must be counted under BadLength (the ticket names it)", head(form, 60)));
        }
    }
    if !fixed.iter().any(|(_, o)| has(r"\bBadLength\b", o) && !has(r"\bTooShort\b", o)) {
        report.fail("AC5", "no invalid form for a size that is not the fixed size counted under BadLength (the ticket names it)");
    }
}

/// AC5; returns the form table's rows (for AC6's exact reasons).
pub(crate) fn forms(ctx: &Ctx, report: &mut Report) -> Option<Vec<Vec<String>>> {
    let table = form_table(ctx);
    if ctx.placeholder {
        return table;
    }
    match &table {
        None => report.fail("AC5", "no table of invalid forms with an outcome column"),
        Some(all) => {
            let rows: Vec<Vec<String>> = all.iter().filter(|r| r.len() >= 2 && !r[0].py_strip().is_empty() && !r[1..].join(" ").py_strip().is_empty()).cloned().collect();
            if rows.len() < 6 {
                report.fail("AC5", format!("only {} invalid form(s) with outcomes; at least 6 are required", rows.len()));
            }
            categories(&rows, ctx, report);
            reason_tokens(&rows, ctx, report);
        }
    }
    for h in rx(r"\bunder\s+`?([A-Z][A-Za-z]+)`?").captures_iter(&ctx.section) {
        if !ctx.members.contains(h.get(1).map_or("", |m| m.as_str())) {
            report.fail("AC5", format!("the section counts a rejection under '{}', which is not a member of the closed enumeration", h.get(1).map_or("", |m| m.as_str())));
        }
    }
    table
}
