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
use crate::doccheck::Report;
use crate::doccheck::checks::c004_24::Stop;
use crate::doccheck::checks::c004_24::py::{has_number, tables};
use crate::doccheck::checks::c004_27::Ctx;
use crate::doccheck::evidence::has_tag;
use crate::doccheck::text::{full, has, has_i, join_digit_groups, rx};
use crate::doccheck::units::{BlockKind, prose_blocks};

/// `re.split(r"(?<=[.;])\s+|\n\s*\n|\n(?=[|*#-])", text)`.
fn split_pieces(text: &str) -> Vec<&str> {
    split_pieces_with(text, true)
}

/// The same split, with or without the third alternative `\n(?=[|*#-])`
/// (004-29's `sentences_of` has only the first two).
pub(crate) fn split_pieces_with(text: &str, pipe_lines: bool) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut start, mut i) = (0, 0);
    let bytes = text.as_bytes();
    while i < text.len() {
        if !text.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let ws_run = |at: usize| text[at..].char_indices().find(|(_, c)| !c.is_whitespace()).map_or(text.len(), |(k, _)| at + k);
        let prev = text[..i].chars().next_back();
        let here = text[i..].chars().next().expect("char");
        let end = if prev.is_some_and(|p| p == '.' || p == ';') && here.is_whitespace() {
            Some(ws_run(i))
        } else if here == '\n' {
            let run = ws_run(i + 1);
            let last_nl = text[i + 1..run].rfind('\n').map(|k| i + 1 + k + 1);
            match last_nl {
                Some(e) => Some(e),
                None => bytes.get(i + 1).filter(|b| pipe_lines && b"|*#-".contains(b)).map(|_| i + 1),
            }
        } else {
            None
        };
        match end {
            Some(e) => {
                out.push(&text[start..i]);
                start = e;
                i = e;
            }
            None => i += here.len_utf8(),
        }
    }
    out.push(&text[start..]);
    out
}

/// 004-25's and 004-28's `sentences`: the pieces with text, a piece that
/// starts with `[` or a backtick joined to the one before.
pub(crate) fn sentences(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for s in split_pieces(text).into_iter().filter(|s| !s.trim().is_empty()) {
        match out.last_mut() {
            Some(last) if rx(r"^\s*[\[`]").is_match(s) => {
                last.push(' ');
                last.push_str(s);
            }
            _ => out.push(s.to_string()),
        }
    }
    out
}

const RULE7: &str = "a stamp that needs more cannot be sent";

fn c1_report(st: &Stamp, h: &SType, s: i64, section_max: Option<i64>, report: &mut Report) {
    let f_h = h.list_counts()[0];
    let s_other = s - h.nt.unwrap_or(0);
    let t = (section_max.filter(|&m| m != 0).unwrap_or(64) - s_other) * f_h;
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
}

/// C1.
pub(crate) fn c1(st: &Stamp, types: &[SType], s: i64, section_max: Option<i64>, consts: &Consts, report: &mut Report) {
    report.check("DoD", st.text.contains(RULE7), format!("review C1: the merged sender rule 7 is quoted by the report: \"{RULE7}\""));
    if s <= section_max.filter(|&m| m != 0).unwrap_or(64) {
        return;
    }
    let Some(h) = types.iter().find(|t| t.key == "health").filter(|h| h.root.as_ref().is_some_and(|r| r.rows.iter().any(|x| x.child.is_some()))) else { return };
    let product = match (consts.c09_max, consts.peers) {
        (Some(a), Some(b)) if a != 0 && b != 0 => Some(2 * a * b),
        _ => None,
    };
    let m_h = h.m_stated.first().copied();
    report.check("DoD", product.is_some() && m_h == product, format!("review C1: the health maximum equals the constants' product: {:?} streams (C-09) × {:?} peers (C-217) × 2 directions = {product:?} (stated {m_h:?})", consts.c09_max, consts.peers));
    let states = product.is_some_and(|p| has_number(&join_digit_groups(&h.text), p)) && h.text.contains("C-09") && h.text.contains("C-217");
    report.check("DoD", states, "review C1: the health subsection states the product with its constants C-09 and C-217");
    c1_report(st, h, s, section_max, report);
}

fn capnorm(x: &str) -> String {
    rx(r"[_\-]+").replace_all(&x.to_lowercase(), " ").into_owned()
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
pub(crate) fn c2(st: &Stamp, types: &[SType], report: &mut Report) {
    let Some(cap) = types.iter().find(|t| t.key == "capability") else { return };
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
        let n: i64 = if w.chars().all(|x| x.is_ascii_digit()) {
            w.parse().unwrap_or(-1)
        } else {
            words.iter().position(|x| *x == w.to_lowercase()).map_or(-1, |p| p as i64)
        };
        report.check("DoD", n == id_rows as i64, format!("review R2-1: the stated count of {n} capabilities equals the numbered rows of the capability id table ({id_rows} numbered rows)"));
    }
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
pub(crate) fn c4(types: &[SType], report: &mut Report) -> Result<(), Stop> {
    let Some(h) = types.iter().find(|t| t.key == "health") else { return Ok(()) };
    if h.root.is_none() {
        return Err(Stop("KeyError: the health type has no entries".into()));
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
    fn the_scripts_sentence_split() {
        assert_eq!(split_pieces("A. B; c\n\nd\n| e\nf"), vec!["A.", "B;", "c", "d", "| e\nf"]);
        assert_eq!(sentences("A. `[decision]` b."), vec!["A. `[decision]` b."]);
        assert_eq!(split_pieces("a\n \n\nb"), vec!["a", "b"]);
    }
}
