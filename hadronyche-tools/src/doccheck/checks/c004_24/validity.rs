//! AC4: the invalid forms and their outcomes. A row whose outcome is not a
//! rejection is allowed and is not counted.

use std::collections::{BTreeMap, BTreeSet};

use super::hello::Landed;
use super::stamp::Stamp;
use crate::doccheck::Report;
use crate::doccheck::rejection_members::reasons::{CAMEL_DIGITS, NON_REJECTION_004_24};
use crate::doccheck::text::{has, has_i, head, rx};

/// One validity row: (invalid form, outcome).
pub(crate) type Form = (String, String);

/// 004-24's reason tokens (`outcome_tokens`): `[A-Z][a-z]+` then one or more
/// `[A-Z][a-z0-9]+`.
pub(crate) fn tokens(cell: &str) -> Vec<String> {
    rx(CAMEL_DIGITS).find_iter(cell).map(|m| m.as_str().to_string()).collect()
}


fn members_of(o: &str, landed: &Landed) -> Vec<String> {
    tokens(o).into_iter().filter(|t| landed.is_member(t)).collect()
}

fn category(form: &str) -> Option<&'static str> {
    let f = form.to_lowercase();
    let h = |p: &str| has(p, &f);
    if h("oversize|over-size|1252|control bound") {
        Some("oversize")
    } else if h("index") && h("count") {
        Some("index_vs_count")
    } else if h("count") && h(r"above|exceed|greater|more than|beyond|maximum|\bmax\b") {
        Some("count_above_max")
    } else if h("count") && h("differ|disagree|mismatch|inconsisten|between|not the same") {
        Some("count_differs")
    } else if h("duplicate|twice|repeat|same section") {
        Some("duplicate")
    } else if h("length|overrun|overflow|body") {
        Some("length_overrun")
    } else if h("magic|version|domain|authentication|scheme|tag|datagram|too short|control header|counter|generic") {
        Some("generic")
    } else {
        None
    }
}

/// The rows of the first `Invalid form | Required outcome` table.
fn rows(st: &Stamp, report: &mut Report) -> Vec<Form> {
    let t = st.tables.iter().find(|t| {
        let h = t.head();
        h.len() >= 2 && has_i("invalid", &h[0]) && has_i("outcome", &h[1])
    });
    report.check("AC4", t.is_some(), "the stamp section has a Validity rules table of 'Invalid form | Required outcome'");
    t.map(|t| t.rows.iter().filter(|r| r.cells.len() >= 2).map(|r| (r.cell(0).to_string(), r.cell(1).to_string())).collect())
        .unwrap_or_default()
}

fn categories(invalid: &[&Form], landed: &Landed, report: &mut Report) {
    let mut cats: BTreeMap<&str, &Form> = BTreeMap::new();
    for row in invalid {
        if let Some(c) = category(&row.0) {
            cats.entry(c).or_insert(row);
        }
    }
    let labels = [
        ("oversize", "an oversize section"),
        ("index_vs_count", "a section index not below the section count"),
        ("count_above_max", "a section count above its maximum"),
        ("count_differs", "a section count differing between sections of one version"),
        ("duplicate", "a duplicate section"),
        ("length_overrun", "a section length overrun"),
        ("generic", "a generic control violation"),
    ];
    for (c, label) in labels {
        report.check("AC4", cats.contains_key(c), format!("the invalid forms include {label}"));
    }
    for row in invalid {
        let mem = members_of(&row.1, landed);
        let unknown: Vec<String> = tokens(&row.1).into_iter().filter(|t| !landed.is_member(t)).collect();
        let ok = !mem.is_empty() && unknown.is_empty() && mem.iter().all(|t| landed.layer(t).contains("control"));
        report.check("AC4", ok, format!("the outcome of '{}' names only members of the Rejection enumeration whose Layers include control; names {mem:?} and {unknown:?}", head(&row.0, 48)));
    }
    let needs = [
        ("oversize", "OversizeControl", "the oversize section (C-17)"),
        ("count_above_max", "BadLength", "a count above its fixed maximum (Rejection reasons)"),
        ("length_overrun", "BadLength", "a section length overrun (the ticket's outcome)"),
    ];
    for (c, need, why) in needs {
        if let Some(row) = cats.get(c) {
            report.check("AC4", tokens(&row.1).iter().any(|t| t == need), format!("{why} is outcome {need}"));
        }
    }
}

fn repeats(st: &Stamp, all: &[Form], landed: &Landed, report: &mut Report) {
    let disagree: Vec<&Form> = all.iter().filter(|r| has_i("disagree|differ", &r.0)).collect();
    let one = |r: &Form| members_of(&r.1, landed).into_iter().collect::<BTreeSet<_>>().len() == 1;
    report.check("AC4", !disagree.is_empty() && disagree.iter().all(|r| one(r)), "every row whose sections disagree names exactly one Reject reason");
    let ident: Vec<&Form> = all.iter().filter(|r| has_i("identical", &r.0)).collect();
    let row_ok = ident.iter().any(|r| members_of(&r.1, landed).is_empty() && NON_REJECTION_004_24.is_match(&r.1) && has_i("kept|held", &r.1));
    let sent_ok = st.paras().any(|t| {
        has_i("identical", t)
            && has_i("repeat", t)
            && has_i("not a rejection|never a rejection", t)
            && has_i("kept|held", t)
            && members_of(t, landed).is_empty()
    });
    let ok = ident.iter().all(|r| members_of(&r.1, landed).is_empty()) && (row_ok || sent_ok);
    report.check("AC4", ok, "one stated outcome for an identical repeat of a held section of one version: not a rejection, the held section kept (a table row or a tagged sentence), and no row for it names a Reject reason");
}

/// AC4; returns the reasons the invalid forms name (`_reasons_md`).
pub(crate) fn ac4(st: &Stamp, landed: &Landed, report: &mut Report) -> BTreeSet<String> {
    let all = rows(st, report);
    let invalid: Vec<&Form> = all.iter().filter(|r| !(NON_REJECTION_004_24.is_match(&r.1) && members_of(&r.1, landed).is_empty())).collect();
    report.check("AC4", invalid.len() >= 7, format!("the stamp section lists at least 7 invalid forms with outcomes, not counting rows whose outcome is not a rejection (found {})", invalid.len()));
    categories(&invalid, landed, report);
    repeats(st, &all, landed, report);
    invalid.iter().flat_map(|r| members_of(&r.1, landed)).collect()
}
