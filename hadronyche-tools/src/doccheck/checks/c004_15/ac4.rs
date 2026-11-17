//! AC 4: the controller-stream wire cost of C-219, its formula and its
//! worked examples.

use super::num::Fails;
use super::setup::Calc;
use super::units::{Eq, Sub};
use crate::doccheck::pynum::PyNum;
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::{has, has_i};

/// `=\s*(?:H|c|B)\b[^=.;]*(?<![\d.])28 ?\+ ?38(?![\d])`, as written.
fn symbolic_formula(text: &str) -> bool {
    has(r"=\s*(?:H|c|B)\b[^=.;]*(?<![\d.])28 ?\+ ?38(?![\d])", text)
}

fn formula(btxt: &str, k: &Calc, f: &mut Fails) {
    if !has(r"(?<![\d.])28 ?\+ ?38(?![\d])", btxt) || !has(r"(?<![\d.])66(?![\d.])", btxt) {
        f.fail(
            "AC4",
            "the 66 bytes of Ethernet, IP and UDP overhead are not shown as 28 + 38 with 66",
        );
    }
    if !symbolic_formula(btxt) {
        f.fail("AC4", "no symbolic formula (H, c or B on the left, 28 + 38 on the right of an '=') gives the bytes");
    }
    if !has(r"2 ?000", btxt) || !has(r"packets?/s|packets? per second|per second", btxt) {
        f.fail(
            "AC4",
            "the controller stream is not stated at 2 000 packets/s",
        );
    }
    if !has(r"8 ?[\u{d7}x*] ?2 ?000", btxt) {
        f.fail(
            "AC4",
            "no bit-rate formula of the form bytes x 8 x 2 000 for the controller stream",
        );
    }
    if !(has(&format!(r"(?<![\d.]){}(?![\d.])", k.cap), btxt) && has_i("cap|budget", btxt)) {
        f.fail("AC4", "the header term of the formula is not named as the 100-byte cap or a Size budget figure");
    }
}

/// The worked examples `H + c + 28 + 38 = N`: (N, c).
fn examples(eqs: &[Eq], k: &Calc, f: &mut Fails) -> Vec<(PyNum, i128)> {
    let mut out = Vec::new();
    for e in eqs {
        if e.ops.is_empty()
            || !e.ops.iter().all(|o| o == "+")
            || !e.terms.contains(&28)
            || !e.terms.contains(&38)
            || !e.ok
        {
            continue;
        }
        let mut rest = e.terms.clone();
        for v in [28, 38] {
            if let Some(i) = rest.iter().position(|&t| t == v) {
                rest.remove(i);
            }
        }
        match rest.iter().position(|r| k.hdr_allowed.contains(r)) {
            Some(h) if rest.len() == 2 => out.push((e.rhs, rest[1 - h])),
            _ => f.fail("AC4", format!("worked example {:?}: the header term is not a figure of the budget (the cap 100 or a 004-14 table figure) and two terms remain: {rest:?}", e.raw)),
        }
    }
    out
}

/// AC 4.
pub(crate) fn controller_cost(sub: &Sub, eqs: &[Eq], k: &Calc, f: &mut Fails) -> Flow<()> {
    let block: Vec<&str> = sub
        .units
        .iter()
        .filter(|u| {
            has("C-219", &u.text)
                || (has_i("controller", &u.text) && has_i(r"per slice|controller bytes", &u.text))
        })
        .map(|u| u.text.as_str())
        .collect();
    if block.is_empty() {
        f.fail(
            "AC4",
            "no paragraph states the C-219 wire cost of a controller stream",
        );
        return Ok(());
    }
    formula(&block.join(" "), k, f);
    let ex = examples(eqs, k, f);
    if ex.is_empty() {
        f.fail(
            "AC4",
            "no worked example 'H + c + 28 + 38 = N' of the C-219 formula (verified by evaluation)",
        );
    }
    let totals: Vec<PyNum> = ex
        .iter()
        .filter(|(_, c)| *c != k.audio)
        .map(|(t, _)| *t)
        .collect();
    if totals.is_empty() {
        f.fail("AC4", format!("the C-219 section has no controller example (every worked example carries the audio {} bytes as c); a transport state or sparse message example is required", k.audio));
    }
    let bit_ex: Vec<&Eq> = eqs
        .iter()
        .filter(|e| e.ok && e.terms.len() == 3 && e.terms.contains(&8) && e.terms.contains(&2000))
        .collect();
    if bit_ex.is_empty() {
        f.fail(
            "AC4",
            "no worked bit-rate example 'N x 8 x 2 000 = R' of the controller stream",
        );
        return Ok(());
    }
    for e in &bit_ex {
        let Some(t) = e.terms.iter().find(|t| **t != 8 && **t != 2000) else {
            return crash("StopIteration in a generator");
        };
        if totals.iter().any(|x| x.eq_int(*t)) {
            return Ok(());
        }
    }
    f.fail("AC4", "the bit-rate example does not use a controller example's byte count (c not the audio 1 152)");
    Ok(())
}
