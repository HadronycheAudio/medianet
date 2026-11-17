//! AC3 (Work 2): the alignment group section; and review C1, the
//! no-sample value of the maximum since live.

use super::Consts;
use super::analyse::{Analysis, statements};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_25::review::sentences;
use crate::doccheck::checks::c004_27::Field;
use crate::doccheck::text::{has, has_i, rx};

const UNIT: &str = r"(?i)nanosecond|microsecond|millisecond|\bns\b|µs|\bus\b|\bms\b|slice|sample|packet|second";

fn lat_cat(m: &str) -> &'static str {
    if has(r"99\.9th", m) {
        "99.9th"
    } else if has(r"\b99th\b", m) {
        "99th"
    } else if has(r"\b50th\b", m) {
        "50th"
    } else if has_i("maximum", m) {
        "maximum"
    } else {
        "other"
    }
}

fn keys(ent: &[&Field], health: &[&Field], report: &mut Report) {
    for (pat, label) in [("module id", "peer module id"), ("stream number", "stream number"), ("request id", "admission request id"), (r"\brole\b", "role")] {
        let hr = health.iter().find(|f| has_i(pat, &f.meaning));
        let gr = ent.iter().find(|f| has_i(pat, &f.meaning));
        let ok = matches!((hr, gr), (Some(h), Some(g)) if g.bits == h.bits);
        report.check("AC3", ok, format!("the alignment input identifier carries the health key's {label} at the health width (004-25; hand-off 004-28)"));
    }
}

fn figures(g: &Analysis, ent: &[&Field], consts: &Consts, report: &mut Report) {
    let lat: Vec<&&Field> = ent.iter().filter(|f| !has_i("since|reclaimable", &f.meaning) && has_i(r"50th|99th|99\.9th|maximum|percentile|lateness", &f.meaning)).collect();
    let mut cats: Vec<&str> = lat.iter().map(|f| lat_cat(&f.meaning)).collect();
    cats.sort_unstable();
    report.check("AC3", cats == ["50th", "99.9th", "99th", "maximum"], format!("the alignment group carries exactly the four lateness figures of C-160: 50th, 99th, 99.9th and the maximum (found {cats:?})"));
    report.check("AC3", !lat.is_empty() && lat.iter().all(|f| f.bits != 0 && has(UNIT, &f.meaning)), "each lateness figure has a width and a unit (Work 2)");
    let w = consts.window_s.map_or("None".to_string(), |v| v.to_string());
    report.check("AC3", has(&format!(r"rolling\s+{w}\s*s\b"), &g.text), format!("the alignment group states its window as a rolling {w} s window in wire.md (C-182)"));
    let since: Vec<&&Field> = ent.iter().filter(|f| has_i("since", &f.meaning)).collect();
    report.check("AC3", since.len() == 1 && since[0].bits != 0 && has(UNIT, &since[0].meaning), "the maximum since the connection went live is one figure with a width and a unit (C-182)");
    let recl: Vec<&&Field> = ent.iter().filter(|f| has_i("reclaimable", &f.meaning)).collect();
    report.check("AC3", recl.len() == 1 && recl[0].bits != 0 && has(UNIT, &recl[0].meaning), "the reclaimable latency is one figure with a width and a unit (C-157)");
    let a = recl.len() == 1 && has(r"\bA\b|audition", &format!("{} {}", recl[0].valid, recl[0].meaning));
    report.check("AC3", a, "the reclaimable latency's valid range is bounded by A (C-53, C-96)");
    if let Some(&f) = g.lists().first()
        && g.stmts.iter().any(|s| s.1 == f) {
            let with_c = sentences(&g.text).iter().filter(|s| !statements(s).is_empty()).any(|s| has(r"C-\d+", s));
            report.check("AC3", with_c, "the maximum alignment entry count is stated with the constant it derives from (C-nn in the same sentence)");
        }
    report.check("AC3", has_i("split|several sections|each section", &g.text), "the split of the alignment type over several sections is stated (Work 2)");
    report.check("AC3", has_i("order", &g.text), "the entry order of the alignment group is stated");
}

/// AC3.
pub(crate) fn ac3(g: Option<&Analysis>, health: Option<&Analysis>, consts: &Consts, report: &mut Report) {
    let (Some(g), Some(h)) = (g.filter(|g| g.ok), health.filter(|h| !h.entries().is_empty())) else {
        report.fail("AC3", "the alignment group section is missing: the input identifier, the four lateness figures, the since-live maximum and the reclaimable latency cannot be checked");
        return;
    };
    let ent = g.entry_fields();
    keys(&ent, &h.entry_fields(), report);
    figures(g, &ent, consts, report);
}

/// The values of the no-sample clauses of the since-live rows.
fn no_sample_values(nos: &[&Field]) -> Vec<String> {
    let mut vals: Vec<String> = Vec::new();
    for f in nos {
        for cl in f.valid.split(';').filter(|cl| has_i("sample", cl)) {
            vals.extend(rx(r"-2\^63|-?\d+").find_iter(cl).map(|m| m.as_str().to_string()));
        }
    }
    vals.sort();
    vals.dedup();
    vals
}

/// True when `v` occurs in `text` not after a word character, `^` or `-`,
/// and not before a word character or `^` (`(?<![\w^-])v(?![\w^])`); with
/// `after_other_than`, only right after "other than" and optional spaces.
fn occurs(text: &str, v: &str, after_other_than: bool) -> bool {
    let w = |c: char| rx(r"^\w$").is_match(&c.to_string());
    text.match_indices(v).any(|(at, _)| {
        let next_ok = !text[at + v.len()..].chars().next().is_some_and(|c| w(c) || c == '^');
        if after_other_than {
            return next_ok && rx(r"other than\s*$").is_match(&text[..at]);
        }
        let prev_ok = !text[..at].chars().next_back().is_some_and(|c| w(c) || c == '^' || c == '-');
        prev_ok && next_ok
    })
}

/// Review C1; `rows` are the invalid forms of the alignment section.
pub(crate) fn c1(g: Option<&Analysis>, rows: &[(String, String)], report: &mut Report) {
    let Some(g) = g.filter(|g| g.ok) else { return };
    let ent = g.entry_fields();
    let nos: Vec<&Field> = ent
        .iter()
        .copied()
        .filter(|f| has_i("since", &f.meaning) && has_i(r"no (live |lateness )?sample|first live sample", &format!("{} {}", f.meaning, f.valid)))
        .collect();
    report.check("AC3", !nos.is_empty(), "the maximum since live states the value it carries when it has no sample (C-182; 009-13: it starts when finish_aligning switches to Live)");
    let vals = no_sample_values(&nos);
    let excluded = nos.iter().any(|f| vals.iter().any(|v| occurs(&f.valid, v, true)));
    let listed_first = has_i("first live sample", &g.text);
    report.check("AC3", !nos.is_empty() && ((!vals.is_empty() && !excluded) || listed_first), "the no-sample value of the maximum since live is admitted by its own valid range, or entries are listed only from the first live sample (C1)");
    let pct: Vec<String> = sentences(&g.text).into_iter().filter(|x| has_i("non-decreasing", x)).collect();
    let exempt = pct.iter().any(|x| has_i(r"exempt|holds? for|no (live |lateness )?sample|no-sample|first live sample|-2\^63", x));
    report.check("AC3", exempt, "the percentile order rule is stated to hold or be exempt for the no-sample encoding (C1)");
    for (form, _) in rows.iter().filter(|(f, _)| has_i("since", f)) {
        for v in &vals {
            if occurs(form, v, false) {
                report.fail("AC4", format!("an invalid form rejects the since-live maximum that holds its no-sample value {v}"));
            }
        }
    }
}
