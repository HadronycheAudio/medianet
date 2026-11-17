//! AC3: the maximum manifest size, computed from the table no counted row
//! references and stated in bytes, within the landed envelope; and review
//! T1, a falsified criterion reported as falsified, never denied.

use std::collections::BTreeSet;

use super::fields::{Man, Model};
use super::landed::Landed;
use super::{NUM, to_int};
use crate::doccheck::Report;
use crate::doccheck::text::{has_i, rx};

/// The top-level table: its size in bytes and its index.
pub(crate) struct Top {
    pub bytes: i128,
    pub model: usize,
}

/// `TOP_CANDIDATES`: the tables whose size no counted row takes as its
/// element width, or every table when none is.
pub(crate) fn candidates(man: &Man) -> Vec<usize> {
    let referenced: BTreeSet<i64> = man.all_rows().filter(|r| r.maxc.is_some()).map(|r| r.bits).collect();
    let c: Vec<usize> = (0..man.models.len()).filter(|&k| !referenced.contains(&man.models[k].total)).collect();
    if c.is_empty() { (0..man.models.len()).collect() } else { c }
}

fn bytes(m: &Model) -> i128 {
    i128::from(m.total.div_euclid(8))
}

/// AC3; returns `TOP_BYTES` and the model it came from (`TOP_MODEL`).
pub(crate) fn ac3(man: &Man, landed: &Landed, report: &mut Report) -> Option<Top> {
    let cands = candidates(man);
    let mut claims = Vec::new();
    for s in man.flat_sentences() {
        if has_i("maximum", &s) && has_i("manifest", &s) {
            claims.extend(rx(&format!(r"({NUM})\s*bytes")).captures_iter(&s).filter_map(|c| to_int(&c[1])));
        }
    }
    let top = cands.iter().map(|&k| bytes(&man.models[k])).filter(|b| claims.contains(b)).max();
    let sizes: Vec<i128> = cands.iter().map(|&k| bytes(&man.models[k])).collect();
    report.check("AC3", top.is_some(), format!("the maximum manifest size is stated in bytes and equals the computed size of a manifest table that no counted row references (candidates {sizes:?} bytes; claimed {claims:?})"));
    let top = top.map(|b| Top { bytes: b, model: *cands.iter().find(|&&k| bytes(&man.models[k]) == b).expect("matched") })?;
    if let (Some(l), Some(s)) = (landed.length_max.filter(|&x| x != 0), landed.section_max.filter(|&x| x != 0)) {
        let sections = (top.bytes + l - 1).div_euclid(l);
        report.check("AC3", sections <= s, format!("the manifest needs {sections} section(s) of {l} bytes, within the landed count maximum {s}"));
    }
    Some(top)
}

/// `affirmed_for(text, idpat)`.
fn affirmed_for(text: &str, idpat: &str) -> bool {
    let a = rx(&format!(r"(?i){idpat}([^.;]{{0,400}}?)\b(?:is|are)\s+falsified\b"));
    let b = rx(&format!(r"(?i)\bfalsifies\b([^.;]{{0,400}}?){idpat}"));
    a.captures_iter(text).any(|c| !has_i(r"\bnot\b", &c[1])) || b.captures_iter(text).any(|c| !has_i(r"\bnot\b", &c[1]))
}

fn report_paragraph(r: &str, top: i128, landed: &Landed, report: &mut Report) {
    report.check("AC3", affirmed_for(r, r"AC[- ]3\b"), "the report affirmatively falsifies the epic's AC-3 (is/are falsified, or falsifies, with no negation between)");
    report.check("AC3", affirmed_for(r, "012-08"), "the report affirmatively falsifies 012-08 AC 2 (is/are falsified, or falsifies, with no negation between)");
    let nums: BTreeSet<i128> = rx(NUM).find_iter(r).filter_map(|m| to_int(m.as_str())).collect();
    report.check("AC3", nums.contains(&top) && nums.contains(&landed.bound) && has_i("exceed", r), format!("the same paragraph states the computed maximum, {top} bytes, and that it exceeds {} bytes", landed.bound));
    report.check("AC3", has_i("design owner", r), "the same paragraph says the report goes to the design owner");
    let cap = i128::from(landed.manifest_max.unwrap_or(0));
    let need = (cap != 0).then(|| (top + cap - 1).div_euclid(cap));
    let claimed: BTreeSet<i128> = rx(&format!(r"(?i)({NUM})\s+(?:manifest\s+)?sections?")).captures_iter(r).filter_map(|c| to_int(&c[1])).collect();
    let ok = need.is_some_and(|n| claimed.contains(&n) && n <= landed.section_max.unwrap_or(0)) && has_i("split", r);
    report.check("AC3", ok, format!("the same paragraph proposes the split: ceil({top} / {cap}) = {need:?} sections, within the landed count maximum {:?} (claimed section counts {claimed:?})", landed.section_max));
}

/// Review T1 (and the join-delay clause it carries).
pub(crate) fn t1(man: &Man, landed: &Landed, top: Option<&Top>, join_open: &[String], report: &mut Report) {
    let denial = rx(r"(?i)no\s+(constant|statement|criterion|design statement)[^.]*?falsif|not\s+falsif");
    let affirm = rx(r"(?i)\b(is|are)\s+falsified\b|\bfalsifies\b");
    let ac3 = rx(r"\bAC[- ]3\b");
    let hits: Vec<&String> = man.paras_flat.iter().filter(|p| denial.is_match(p) && !(affirm.is_match(p) && ac3.is_match(p))).collect();
    for p in &hits {
        let m = denial.find(p).map(|m| m.as_str()).unwrap_or_default();
        report.fail("AC3", format!("a paragraph denies a falsification without reporting one as falsified: '{m}'"));
    }
    report.check("AC3", hits.is_empty(), "no paragraph denies a falsification without the affirmative report of AC-3");
    let Some(top) = top.filter(|t| t.bytes > landed.bound) else { return };
    let rep: Vec<&String> = man.paras_flat.iter().filter(|p| affirm.is_match(p) && ac3.is_match(p) && p.contains("012-08")).collect();
    report.check("AC3", !rep.is_empty(), "a paragraph reports as falsified (is/are falsified) the epic's AC-3 and 012-08 AC 2, which assume a one-message manifest");
    if let Some(r) = rep.first() {
        report_paragraph(r, top.bytes, landed, report);
    }
    let clause = rx("(?i)(already|is met|are met)[^.]{0,80}1[ \u{a0}\u{202f}]?252|1[ \u{a0}\u{202f}]?252[^.]{0,80}(already|is met)");
    report.check("AC4", join_open.iter().any(|p| clause.is_match(p)), "the join-delay open item says the second clause of the ticket's falsifier is already met at 1 252 bytes");
}
