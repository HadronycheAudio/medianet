//! AC 4: the controller-stream wire cost of C-219, its formula and its
//! worked examples.

use super::num::{Fails, R, crash, guarded};
use super::setup::Calc;
use super::units::{Eq, Sub};
use crate::doccheck::text::{has, has_i, rx};

/// `(?<![\d.])<core>(?![\d])`-style search for a literal core.
fn lit(core: &str, text: &str, next_bad: &str) -> bool {
    guarded(rx(core), text, |c| c.is_ascii_digit() || c == '.')
        .iter()
        .any(|m| {
            let end = m.get(0).map_or(0, |w| w.end());
            !text[end..].chars().next().is_some_and(|c| next_bad.contains(c))
        })
}

const PLUS: &str = r"28 ?\+ ?38";

/// `=\s*(?:H|c|B)\b[^=.;]*(?<![\d.])28 ?\+ ?38(?![\d])`.
fn symbolic_formula(text: &str) -> bool {
    let head = rx(r"=\s*(?:H|c|B)\b");
    let tail = rx(&format!("^{PLUS}"));
    for m in head.find_iter(text) {
        let rest = &text[m.end()..];
        let stop = rest.find(['=', '.', ';']).unwrap_or(rest.len());
        for (j, _) in rest[..stop].char_indices() {
            let prev = if j == 0 { text[..m.end()].chars().next_back() } else { rest[..j].chars().next_back() };
            if prev.is_some_and(|c| c.is_ascii_digit() || c == '.') {
                continue;
            }
            if let Some(t) = tail.find(&rest[j..])
                && !rest[j + t.end()..].chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    return true;
                }
        }
    }
    false
}

fn formula(btxt: &str, k: &Calc, f: &mut Fails) {
    if !lit(PLUS, btxt, "0123456789") || !lit("66", btxt, "0123456789.") {
        f.add("AC4", "the 66 bytes of Ethernet, IP and UDP overhead are not shown as 28 + 38 with 66");
    }
    if !symbolic_formula(btxt) {
        f.add("AC4", "no symbolic formula (H, c or B on the left, 28 + 38 on the right of an '=') gives the bytes");
    }
    if !has(r"2 ?000", btxt) || !has(r"packets?/s|packets? per second|per second", btxt) {
        f.add("AC4", "the controller stream is not stated at 2 000 packets/s");
    }
    if !has(r"8 ?[\u{d7}x*] ?2 ?000", btxt) {
        f.add("AC4", "no bit-rate formula of the form bytes x 8 x 2 000 for the controller stream");
    }
    if !(lit(&k.cap.to_string(), btxt, "0123456789.") && has_i("cap|budget", btxt)) {
        f.add("AC4", "the header term of the formula is not named as the 100-byte cap or a Size budget figure");
    }
}

/// The worked examples `H + c + 28 + 38 = N`: (N, c).
fn examples(eqs: &[Eq], k: &Calc, f: &mut Fails) -> Vec<(f64, i64)> {
    let mut out = Vec::new();
    for e in eqs {
        if e.ops.is_empty() || !e.ops.iter().all(|o| o == "+") || !e.terms.contains(&28) || !e.terms.contains(&38) || !e.ok {
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
            _ => f.add("AC4", format!("worked example {:?}: the header term is not a figure of the budget (the cap 100 or a 004-14 table figure) and two terms remain: {rest:?}", e.raw)),
        }
    }
    out
}

/// AC 4.
pub(crate) fn controller_cost(sub: &Sub, eqs: &[Eq], k: &Calc, f: &mut Fails) -> R<()> {
    let block: Vec<&str> = sub
        .units
        .iter()
        .filter(|u| has("C-219", &u.text) || (has_i("controller", &u.text) && has_i(r"per slice|controller bytes", &u.text)))
        .map(|u| u.text.as_str())
        .collect();
    if block.is_empty() {
        f.add("AC4", "no paragraph states the C-219 wire cost of a controller stream");
        return Ok(());
    }
    formula(&block.join(" "), k, f);
    let ex = examples(eqs, k, f);
    if ex.is_empty() {
        f.add("AC4", "no worked example 'H + c + 28 + 38 = N' of the C-219 formula (verified by evaluation)");
    }
    let totals: Vec<f64> = ex.iter().filter(|(_, c)| *c != k.audio).map(|(t, _)| *t).collect();
    if totals.is_empty() {
        f.add("AC4", format!("the C-219 section has no controller example (every worked example carries the audio {} bytes as c); a transport state or sparse message example is required", k.audio));
    }
    let bit_ex: Vec<&Eq> = eqs.iter().filter(|e| e.ok && e.terms.len() == 3 && e.terms.contains(&8) && e.terms.contains(&2000)).collect();
    if bit_ex.is_empty() {
        f.add("AC4", "no worked bit-rate example 'N x 8 x 2 000 = R' of the controller stream");
        return Ok(());
    }
    for e in &bit_ex {
        let Some(t) = e.terms.iter().find(|t| **t != 8 && **t != 2000) else { return crash("StopIteration in a generator") };
        if totals.contains(&(*t as f64)) {
            return Ok(());
        }
    }
    f.add("AC4", "the bit-rate example does not use a controller example's byte count (c not the audio 1 152)");
    Ok(())
}
