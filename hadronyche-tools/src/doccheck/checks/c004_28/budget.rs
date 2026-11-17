//! AC1: the merged 004-25 types (the sections they need), the two new
//! types (sizes, codes, statements, arithmetic), and the budget: a fit
//! with its arithmetic, or the complete overflow report with the cap
//! condition of review C2.

use std::collections::BTreeMap;

use super::analyse::{Analysis, analyse, exact_product_results, has_num2};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_25::Stamp;
use crate::doccheck::checks::c004_25::review::sentences;
use crate::doccheck::checks::c004_25::tree::{bad_arithmetic_lenient, cdiv};
use crate::doccheck::checks::c004_25::types::{TYPE_SPECS, stated_sizes};
use crate::doccheck::checks::c004_27::find_sub;
use crate::doccheck::evidence::has_tag;
use crate::doccheck::text::{has, has_i, rx};
use crate::doccheck::units::BlockKind;

/// The merged types and the new ones.
pub(crate) struct Types {
    pub merged: BTreeMap<&'static str, Analysis>,
    pub s_other: i64,
    /// `F_H`: the health type's entries per section.
    pub f_h: Option<i64>,
    pub new: BTreeMap<&'static str, Analysis>,
}

impl Types {
    /// `HEALTH`.
    pub fn health(&self) -> Option<&Analysis> {
        self.merged.get("health")
    }
}

/// The merged types, `S_OTHER` and `F_H`.
type MergedTypes = (BTreeMap<&'static str, Analysis>, i64, Option<i64>);

/// Reads the merged types; `Err` where the script crashes (`F_H` of a
/// health table with no list row).
pub(crate) fn merged(st: &Stamp, report: &mut Report) -> Result<MergedTypes, String> {
    let mut m = BTreeMap::new();
    for (key, pat) in TYPE_SPECS {
        let hits = find_sub(&st.subs, pat);
        if hits.len() == 1 {
            m.insert(key, analyse(&hits[0].0, &hits[0].1, "AC1", report));
        }
    }
    report.check("DoD", m.len() == 7, format!("the seven merged section types of 004-25 are found once each (found {})", m.len()));
    let s_other: i64 = m.iter().filter(|(k, _)| **k != "health").filter_map(|(_, v)| v.nt).filter(|&n| n != 0).sum();
    report.check("AC1", s_other == 7, format!("the six other merged section types need 7 sections at their maxima (computed {s_other})"));
    let health = m.get("health");
    let f_h = match health.filter(|h| h.root.is_some()) {
        Some(h) => Some(*h.lists().first().ok_or("IndexError: the health table has no list row")?),
        None => None,
    };
    let ok = health.is_some_and(|h| h.entries().first().is_some_and(|e| cdiv(e.size, 8) == 65)) && f_h == Some(18);
    report.check("DoD", ok, format!("the merged health entry is 65 bytes and 18 entries fit a section (read {f_h:?})"));
    Ok((m, s_other, f_h))
}

/// The two new types (Work 1 and Work 2).
pub(crate) fn new_types(st: &Stamp, report: &mut Report) -> BTreeMap<&'static str, Analysis> {
    let mut out = BTreeMap::new();
    for (key, pat, name) in [("dscp", "dscp", "observed DSCP of peers"), ("alignment", "alignment", "alignment group")] {
        let hits = find_sub(&st.subs, pat);
        report.check("AC1", hits.len() == 1, format!("exactly one subsection of the regime stamp section names the {name} section type (found {})", hits.len()));
        if hits.len() == 1 {
            out.insert(key, analyse(&hits[0].0, &hits[0].1, "AC1", report));
        }
    }
    out
}

/// The per-type checks of AC1; `Err` where the script crashes.
pub(crate) fn per_type(t: &Types, ctrl_bound: Option<i64>, block: Option<(i64, i64)>, report: &mut Report) -> Result<(), String> {
    let merged_codes: Vec<i64> = t.merged.values().filter_map(|v| v.code).collect();
    for (key, a) in &t.new {
        if !a.ok {
            report.fail("AC1", format!("the {key} section type has no usable field table"));
            continue;
        }
        let sec_max = a.sec_max.unwrap_or(0);
        let b = ctrl_bound.ok_or("TypeError: int <= None")?;
        report.check("AC1", sec_max <= b, format!("the {key} section at its maxima is {sec_max} bytes, no more than the control bound (C-17) of {b} bytes"));
        let stated = stated_sizes(&a.text);
        report.check("AC1", stated.contains(&sec_max), format!("a sentence of the {key} subsection states its maximum section size as {sec_max} bytes (the table's sum); stated {stated:?}"));
        let inside = matches!((a.code, block), (Some(c), Some((lo, hi))) if lo <= c && c <= hi && !merged_codes.contains(&c));
        report.check("AC1", inside, format!("the {key} section type has a type code inside the block the merged table gives 004-28 ({block:?}) and not a code of 004-25 (found {:?})", a.code));
        for f in a.lists() {
            let Some(&(m, ff, n)) = a.stmts.iter().find(|s| s.1 == f) else {
                report.fail("AC1", format!("the {key} list row (at most {f} entries per section) has no statement of the form ⌈M / {f}⌉ = N"));
                continue;
            };
            report.check("AC1", n == cdiv(m, ff), format!("the {key} section count is ⌈{m} / {ff}⌉ = {}, the text states {n}", cdiv(m, ff)));
        }
        let bad = bad_arithmetic_lenient(&a.text);
        report.check("AC1", bad.is_empty(), format!("every product and sum stated in the {key} subsection is exact: {bad:?}"));
    }
    Ok(())
}

const RULE7: &str = "a stamp that needs more cannot be sent";

/// `limit(Fa)`: the largest entry count whose health and alignment
/// sections fit `budget`; `Err` where Python divides by zero.
fn limit(f_h: i64, fa: i64, budget: i64) -> Result<i64, String> {
    if f_h <= 0 || fa <= 0 {
        return Err("ZeroDivisionError".into());
    }
    let mut e = 0;
    while cdiv(e + 1, f_h) + cdiv(e + 1, fa) <= budget {
        e += 1;
    }
    Ok(e)
}

fn cap_condition(t: &Types, paras: &[&str], section_max: i64, report: &mut Report) -> Result<(), String> {
    let caps: Vec<&str> = paras.iter().copied().filter(|x| x.contains("18⌉") && has(r"\bcap\b", x)).collect();
    let union = caps.join("\n");
    report.check("AC1", !caps.is_empty(), "the cap condition is stated in closed form over ⌈ ⌉ of 18 member entries per section");
    let width = sentences(&union).iter().any(|x| has_i("assum", x) && has_i("bit|bytes", x) && has_tag(x));
    report.check("AC1", width, "the cap condition states, in a tagged sentence, the figure width it assumes (review C2 (a))");
    report.check("AC1", has_i("per[- ]input", &union) && union.contains("Open item 17"), "the cap condition assumes per-input alignment entries and names Open item 17 (review C2 (b))");
    for (tok, what) in [("path[ -]MTU", "the path MTU"), ("013-08", "013-08"), ("006-19", "006-19")] {
        report.check("AC1", has_i(tok, &union), format!("the cap condition names {what}, a section type not yet laid out that lowers the bound (review C2 (c))"));
    }
    let (Some(al), Some(ds)) = (t.new.get("alignment").filter(|a| a.ok), t.new.get("dscp").filter(|a| a.ok)) else { return Ok(()) };
    let own_f = *al.lists().first().ok_or("IndexError: the alignment table has no list row")?;
    let own_size = al.entries().first().ok_or("IndexError: the alignment table has no entry table")?.size.div_euclid(8);
    let f_h = t.f_h.ok_or("TypeError: F_H is None")?;
    let budget = section_max - t.s_other - ds.nt.unwrap_or(0);
    let l_own = limit(f_h, own_f, budget)?;
    let alt: Vec<(i64, i64)> = rx(r"⌊\s*1193\s*/\s*(\d+)\s*⌋\s*=\s*(\d+)")
        .captures_iter(&union)
        .filter_map(|c| Some((c[1].parse().ok()?, c[2].parse().ok()?)))
        .filter(|(x, y)| *x != own_size && *y != own_f)
        .collect();
    let prods = exact_product_results(&union);
    report.check("AC1", prods.contains(&i128::from(l_own)), format!("the cap condition gives the limit for its own entry size as exact arithmetic k × n = {l_own} (review C2 (a))"));
    let alt_l = match alt.first() {
        Some(&(_, y)) => Some(limit(f_h, y, budget)?),
        None => None,
    };
    report.check("AC1", alt_l.is_some_and(|l| prods.contains(&i128::from(l))), format!("the cap condition gives the limit for one alternative entry size as exact arithmetic k × n = {alt_l:?}, with ⌊1 193 / X⌋ = Y stating the alternative (review C2 (a))"));
    Ok(())
}

fn overflow(t: &Types, paras: &[&str], totals: (i64, i64, i64), section_max: i64, report: &mut Report) -> Result<(), String> {
    let (s_total, thresh, s_new) = totals;
    let score = |p: &str| -> usize {
        let l = p.to_lowercase();
        [
            has_num2(p, s_total),
            has_i("exceed|falsif", p),
            l.contains("design owner"),
            l.contains(RULE7) && l.contains("cannot be sent"),
            has_i("no stamp", p),
            has_num2(p, thresh) && has_i("threshold|fit", p),
            has_tag(p),
            has_i(r"\bcap", p),
            l.contains("open item"),
        ]
        .into_iter()
        .filter(|x| *x)
        .count()
    };
    let cands: Vec<&str> = paras.iter().copied().filter(|x| x.to_lowercase().contains("open item") || has_num2(x, s_total)).collect();
    let best = cands.iter().fold(None::<&str>, |b, p| match b {
        Some(x) if score(x) >= score(p) => Some(x),
        _ => Some(p),
    });
    let best = best.unwrap_or("");
    let l = best.to_lowercase();
    report.check("AC1", has_num2(best, s_total) && l.contains("open item"), format!("the overflow report names an Open item and states the total {s_total} sections against the bound {section_max}"));
    report.check("AC1", has_i("exceed|falsif", best) && l.contains("design owner"), "the overflow report says the bound is exceeded or falsified, and that it goes to the design owner");
    report.check("AC1", l.contains(RULE7) && l.contains("cannot be sent"), "the overflow report cites merged sender rule 7 (a stamp that needs more cannot be sent)");
    report.check("AC1", has_i("no stamp", best), "the overflow report says that above the bound a module sends no stamp at all");
    report.check("AC1", has_num2(best, thresh) && has_i("threshold|fit", best), format!("the overflow report gives the threshold in entries, (64 − {} − {s_new}) × {:?} = {thresh}", t.s_other, t.f_h));
    report.check("AC1", has_i(r"\bcap", best), "the overflow report names the connection cap that the design owner sets");
    report.check("DoD", has_tag(best), "the overflow report paragraph carries an evidence tag");
    cap_condition(t, paras, section_max, report)
}

/// The budget (AC1); `Err` where the script crashes.
pub(crate) fn budget(st: &Stamp, t: &Types, section_max: Option<i64>, report: &mut Report) -> Result<(), String> {
    let s_new: i64 = t.new.values().filter(|a| a.ok).filter_map(|a| a.nt).filter(|&n| n != 0).sum();
    let s_total = t.s_other + s_new;
    let sm = section_max.filter(|&m| m != 0).unwrap_or(64);
    let thresh = (sm - t.s_other - s_new) * t.f_h.filter(|&f| f != 0).unwrap_or(0);
    let paras: Vec<&str> = st.prose.iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text.as_str()).collect();
    if t.new.len() != 2 {
        let keys: Vec<&&str> = t.new.keys().collect();
        report.fail("AC1", format!("the budget cannot be computed: both new section types must exist (found {keys:?})"));
        return Ok(());
    }
    if s_total <= sm {
        let fit = paras.iter().any(|x| has_num2(x, s_total) && has_num2(x, thresh) && has_i(r"\bfit", x) && has_tag(x));
        report.check("AC1", fit, format!("the budget fits with its arithmetic stated: the six other types and the two new types need {s_total} sections of 64, and the threshold (64 − {} − {s_new}) × {:?} = {thresh} entries is stated in one tagged paragraph", t.s_other, t.f_h));
        return Ok(());
    }
    overflow(t, &paras, (s_total, thresh, s_new), sm, report)
}
