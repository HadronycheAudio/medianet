//! The review findings of PR 35 (labelled DoD with the finding named): C1
//! the overflow report, C2 capabilities named with a ticket id, C3 no
//! hand-back of a Manifest open item owned by 004-25, C4 a health entry
//! that tells one admission from the next.
//!
//! The header's "Declared limits" say "the capability vocabulary is the
//! list in review.py"; there is no review.py in the repository, and the
//! code holds the list inline (`CAP_VOCAB`). The port follows the code.

use std::collections::BTreeSet;

use super::types::SType;
use super::{Consts, Stamp};
use crate::doccheck::checked;
use crate::doccheck::Report;
use crate::doccheck::family004::{Ctx, tables};
use crate::doccheck::pycompat::{is_nd, parse_int};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::sentences::{SentenceSplit, split};
use crate::doccheck::evidence::has_tag;
use crate::doccheck::text::{full, has, has_i, join_digit_groups, rx};
use crate::doccheck::units::{BlockKind, prose_blocks};

/// 004-25's `sentences` ([`SentenceSplit::BLOCKS_004_25`]).
fn sentences(text: &str) -> Vec<String> {
    split(text, SentenceSplit::BLOCKS_004_25)
}

/// `re.search(rf"(?<!\d){n}(?!\d)", text)` (`has_num`).
pub(super) fn has_number(text: &str, n: i128) -> bool {
    has(&format!(r"(?<!\d){n}(?!\d)"), text)
}

/// `str.isdigit()` inside the repertoire: a decimal digit, or one of the
/// superscript and subscript digits (`int()` of which raises).
pub(super) fn py_isdigit(c: char) -> bool {
    is_nd(c) || "¹²³⁴⁵⁶⁹₁₂₃".contains(c)
}


const RULE7: &str = "a stamp that needs more cannot be sent";

fn c1_report(st: &Stamp, h: &SType, s: i128, section_max: Option<i128>, report: &mut Report) -> Flow<()> {
    let f_h = h.list_counts()[0];
    // `_S - HEALTH["Nt"]`: set whenever the health type has entries
    let Some(h_nt) = h.nt else { return crash("TypeError: unsupported operand type(s) for -: 'int' and 'NoneType'") };
    let s_other = checked::sub(s, h_nt);
    let sm = section_max.filter(|&m| m != 0).unwrap_or(64);
    let t = checked::mul(checked::sub(sm, s_other), f_h);
    let paras: Vec<&str> = st.prose.iter().filter(|b| b.kind == BlockKind::Prose && has(r"Open item 1(?:\D|$)", &b.text)).map(|b| b.text.as_str()).collect();
    let score = |p: &str| -> usize {
        let lower = p.to_lowercase();
        [
            has_number(p, s),
            has_i("exceed|falsif", p),
            lower.contains("design owner"),
            lower.contains(RULE7) && lower.contains("cannot be sent"),
            has_i("no stamp", p),
            has_number(p, t) && has_i("threshold|fit", p),
            has_tag(p),
        ]
        .into_iter()
        .filter(|x| *x)
        .count()
    };
    // `max(paras, key=score)`: the first of the highest.
    let best = paras.iter().fold(None::<&str>, |b, p| match b {
        Some(x) if score(x) >= score(p) => Some(x),
        _ => Some(p),
    });
    let best = best.unwrap_or("");
    let lower = best.to_lowercase();
    report.check("DoD", has_number(best, s) && !paras.is_empty(), format!("review C1: one tagged report paragraph names Open item 1 and states the total {s} sections against the bound 64"));
    report.check("DoD", has_i("exceed|falsif", best) && lower.contains("design owner"), "review C1: the report says the bound is exceeded or falsified, and that it goes to the design owner");
    report.check("DoD", lower.contains(RULE7) && lower.contains("cannot be sent"), "review C1: the report cites merged sender rule 7 (a stamp that needs more cannot be sent)");
    report.check("DoD", has_i("no stamp", best), "review C1: the report says that above the bound a module sends no stamp at all");
    report.check("DoD", has_number(best, t) && has_i("threshold|fit", best), format!("review C1: the report gives the threshold in entries, (64 − {s_other}) × {f_h} = {t}, below which a stamp still fits"));
    report.check("DoD", has_tag(best), "review C1: the report paragraph carries an evidence tag");
    Ok(())
}

/// C1.
pub(crate) fn c1(st: &Stamp, types: &[SType], s: i128, section_max: Option<i128>, consts: &Consts, report: &mut Report) -> Flow<()> {
    report.check("DoD", st.text.contains(RULE7), format!("review C1: the merged sender rule 7 is quoted by the report: \"{RULE7}\""));
    if s <= section_max.filter(|&m| m != 0).unwrap_or(64) {
        return Ok(());
    }
    let Some(h) = types.iter().find(|t| t.key == "health").filter(|h| h.root.as_ref().is_some_and(|r| r.rows.iter().any(|x| x.child.is_some()))) else { return Ok(()) };
    let product = match (consts.c09_max, consts.peers) {
        (Some(a), Some(b)) if a != 0 && b != 0 => Some(checked::product([2, a, b])),
        _ => None,
    };
    let m_h = h.m_stated.first().copied();
    report.check("DoD", product.is_some() && m_h == product, format!("review C1: the health maximum equals the constants' product: {:?} streams (C-09) × {:?} peers (C-217) × 2 directions = {product:?} (stated {m_h:?})", consts.c09_max, consts.peers));
    let states = product.is_some_and(|p| has_number(&join_digit_groups(&h.text), p)) && h.text.contains("C-09") && h.text.contains("C-217");
    report.check("DoD", states, "review C1: the health subsection states the product with its constants C-09 and C-217");
    c1_report(st, h, s, section_max, report)
}

pub(super) fn capnorm(x: &str) -> String {
    rx(r"[_\-]+").replace_all(&x.to_lowercase(), " ")
}

const CAP_VOCAB: [&str; 9] = [
    "drop accounting",
    "dscp marking",
    "fault injection",
    "holder eligible",
    "real time scheduling",
    "memory lock",
    "control isolation",
    "idle states",
    "bounded real time",
];

/// C2 and its round-2 count.
pub(crate) fn c2(st: &Stamp, types: &[SType], report: &mut Report) -> Flow<()> {
    let Some(cap) = types.iter().find(|t| t.key == "capability") else { return Ok(()) };
    let (mut cap_rows, mut id_rows) = (Vec::new(), 0usize);
    for tb in tables(&cap.text) {
        let is_id = tb.head().first().is_some_and(|h| h.trim().to_lowercase() == "id");
        for r in &tb.rows {
            if full(r"\d+", r.cell(0).trim()) {
                cap_rows.push(capnorm(&r.cells.join(" ")));
                if is_id {
                    id_rows += 1;
                }
            }
        }
    }
    let mut needs: BTreeSet<&str> = BTreeSet::new();
    for s in sentences(&st.text) {
        if has(r"\b\d{3}[a-z]?-\d{2}\b", &s) {
            let sl = capnorm(&s);
            needs.extend(CAP_VOCAB.iter().copied().filter(|term| sl.contains(*term)));
        }
    }
    for term in needs {
        report.check("DoD", cap_rows.iter().any(|row| row.contains(term)), format!("review C2: the capability \"{term}\" named with a ticket id has a numbered row in the capability table"));
    }
    let words = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];
    for c in rx(r"(?i)stamps? (\w+) capabilities").captures_iter(&st.text) {
        let w = &c[1];
        // `int(w) if w.isdigit() else WORDS.get(w.lower(), -1)`: `int("²")` raises
        let n: i128 = if w.chars().all(py_isdigit) {
            match parse_int(w) {
                Ok(n) => n,
                Err(_) => return crash(format!("ValueError: invalid literal for int() with base 10: {w:?}")),
            }
        } else {
            words.iter().position(|x| *x == w.to_lowercase()).map_or(-1, |p| i128::try_from(p).expect("an index fits i128"))
        };
        report.check("DoD", n == i128::try_from(id_rows).expect("a count fits i128"), format!("review R2-1: the stated count of {n} capabilities equals the numbered rows of the capability id table ({id_rows} numbered rows)"));
    }
    Ok(())
}

/// C3.
pub(crate) fn c3(ctx: &Ctx, st: &Stamp, report: &mut Report) {
    let man = ctx.body(ctx.span_of(r"^manifest$", 2));
    let mut owned = BTreeSet::new();
    for b in prose_blocks(&man).into_iter().filter(|b| b.kind == BlockKind::Prose) {
        if has(r"Owner:[^.]*\b004-25\b", &b.text)
            && let Some(c) = rx(r"Open item (\d+)").captures(&b.text) {
                owned.insert(c[1].to_string());
            }
    }
    let mut bad = Vec::new();
    for s in sentences(&st.text) {
        for c in rx(r"Manifest,? Open item (\d+)").captures_iter(&s) {
            if owned.contains(&c[1]) && has_i("owner|hands|left to|belongs to", &s) {
                bad.push(format!("Manifest, Open item {}", &c[1]));
            }
        }
    }
    report.check("DoD", bad.is_empty(), format!("review C3: the regime stamp section does not hand a Manifest open item owned by 004-25 back to it undecided: {bad:?} (owned by 004-25: {owned:?})"));
}

/// C4.
pub(crate) fn c4(types: &[SType], report: &mut Report) -> Flow<()> {
    let Some(h) = types.iter().find(|t| t.key == "health") else { return Ok(()) };
    if h.root.is_none() {
        return crash("KeyError: 'entries'");
    }
    let field = h.entry_fields().iter().any(|f| has_i("request id|admission|generation|epoch of the connection", &f.meaning));
    let sentence = sentences(&h.text).iter().any(|s| has_i("reset", s) && has_i("wrap", s) && has_tag(s));
    report.check("DoD", field || sentence, "review C4: a health entry has a field that tells one admission from the next, or a tagged sentence tells a counter reset on re-admission from a wrap");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scripts_sentences_and_digits() {
        assert_eq!(sentences("A. `[decision]` b."), vec!["A. `[decision]` b."]);
        assert!(has_number("count 64.", 64) && !has_number("640", 64));
        assert!(py_isdigit('²') && py_isdigit('7') && !py_isdigit('x'));
    }
}
