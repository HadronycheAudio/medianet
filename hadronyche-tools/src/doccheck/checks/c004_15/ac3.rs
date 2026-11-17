//! AC 3 (the 5 % rule) and the audio declared rate.

use super::num::{Fails, close, mbit_list, sentences, to_int, tol_of};
use super::setup::Calc;
use super::units::Sub;
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::{has, has_i, rx};

const WITHIN: &str = r"(?i)\bwithin\b|\bbelow\s+(?:the\s+)?5|\bunder\s+(?:the\s+)?5|\bless than\s+(?:the\s+)?5|\bsmaller than\s+(?:the\s+)?5|\bnot exceed|\bdoes not exceed|\bdo not exceed|\bno more than";
const EXCEED: &str = r"(?i)exceed|above|\bmore than\s+(?:the\s+)?5|\bgreater than\s+(?:the\s+)?5|\bover\s+(?:the\s+)?5";
const STAND: &str = r"(?i)no constant[^.;]{0,40}(?:recomput|changed|updated|re-?derived)|\bconstants?\b[^.;]{0,60}\bstand\b|not recomputed|constants? (?:are|is|remain|remains) unchanged";

/// Every sentence of every unit.
pub(crate) fn all_sentences(sub: &Sub) -> Vec<String> {
    sub.units.iter().flat_map(|u| sentences(&u.text)).collect()
}

fn head(x: &str, n: usize) -> String {
    x.chars().take(n).collect()
}

fn five_percent(sents: &[String], k: &Calc, f: &mut Fails) {
    let five: Vec<&String> = sents.iter().filter(|s| has(r"(?<![\w.])5 ?%", s)).collect();
    if five.is_empty() {
        f.fail("AC3", "no sentence states the 5 % rule (a sentence with '5 %' and exceed / within / below / above)");
    }
    for s in five {
        let within = has(WITHIN, s);
        let exceed = has(EXCEED, s) && !within;
        if !(within || exceed) {
            f.fail(
                "AC3",
                format!("a '5 %' sentence states no direction: {}", head(s, 90)),
            );
            continue;
        }
        if exceed != k.exceeds {
            f.fail("AC3", format!("the section says the difference {} 5 %, but the computed difference is {:+.2} % bytes and {:+.2} % rate: {}", if exceed { "exceeds" } else { "is within" }, k.pct_b, k.pct_r, head(s, 90)));
        }
    }
}

/// AC 3: the 5 % rule, against the computed difference.
pub(crate) fn rule(sub: &Sub, sents: &[String], k: &Calc, f: &mut Fails) {
    five_percent(sents, k, f);
    let text = &sub.sub_text;
    if !k.exceeds {
        if !has(STAND, text) {
            f.fail("AC3", "the difference is within 5 %, but the section does not say that no constant is recomputed (the constants stand)");
        }
        return;
    }
    for cid in [
        "C-20", "C-21", "C-22", "C-24", "C-25", "C-26", "C-90", "C-92", "C-118",
    ] {
        if !text.contains(cid) {
            f.fail("AC3", format!("the difference exceeds 5 %, but the section does not name {cid} as marked for recompute"));
        }
    }
    if has(STAND, text) {
        f.fail("AC3", "the difference exceeds 5 %, but the section still says the constants stand or that no constant is recomputed");
    }
    let capacity_item = sub.units.iter().filter(|u| !u.row).any(|u| {
        has_i(r"^[\s*_]*open item", &u.text)
            && has(r"C-20\b|capacity constants", &u.text)
            && has_i("owner", &u.text)
            && has_i("falsifier", &u.text)
    });
    if !has_i("constants table updated", text) && !capacity_item {
        f.fail("AC3", "the difference exceeds 5 %, but neither 'constants table updated' nor an open item that itself names C-20 (or capacity constants) with owner and falsifier");
    }
}

/// If a sentence states an audio stream's declared rate, it is the cap form:
/// cap + audio + 66 bytes at 2 000 packets/s.
pub(crate) fn declared_rate(sents: &[String], k: &Calc, f: &mut Fails) -> Flow<()> {
    let aud_bytes = k.cap + k.audio + k.ovh;
    let aud_bits = aud_bytes * 8 * k.slice_rate;
    for t in sents
        .iter()
        .filter(|t| has_i("declared", t) && has_i("audio", t))
    {
        let mut bits = Vec::new();
        for m in rx(r"(\d[\d ]*\d|\d)\s*bit/s").captures_iter(t) {
            match to_int(&m[1]) {
                Some(v) => bits.push(v),
                None => return crash("beyond i128 (INT_LIMIT)"),
            }
        }
        let mb = mbit_list(t);
        if !mb.is_empty() || !bits.is_empty() {
            let allowed: Vec<f64> = [
                k.def_pay + k.ovh,
                k.v1_largest_pay + k.ovh,
                k.ann_max_pay + k.ovh,
                aud_bytes,
            ]
            .iter()
            .map(|w| (*w * 8 * k.slice_rate) as f64 / 1e6)
            .collect();
            let all_known = mb
                .iter()
                .all(|&(v, d)| allowed.iter().any(|&a| close(v, a, tol_of(d))))
                && bits
                    .iter()
                    .all(|&b| allowed.iter().any(|&a| (b as f64 - a * 1e6).abs() <= 0.5));
            let cap_form = mb
                .iter()
                .any(|&(v, d)| close(v, aud_bits as f64 / 1e6, tol_of(d)))
                || bits
                    .iter()
                    .any(|&b| (b as f64 - aud_bits as f64).abs() <= 0.5);
            if !all_known {
                f.fail("AC5", format!("an audio declared rate is stated as a figure the layout does not compute (the cap form is {aud_bytes} bytes, {aud_bits} bit/s): {}", head(t, 100)));
            } else if !cap_form {
                f.fail("AC5", format!("the audio declared rate is not the cap form, {aud_bytes} bytes at {aud_bits} bit/s: {}", head(t, 100)));
            }
        }
        let mut big = Vec::new();
        for m in rx(r"(\d[\d ]*\d|\d)\s*bytes").captures_iter(t) {
            let Some(n) = to_int(&m[1]) else {
                return crash("beyond i128 (INT_LIMIT)");
            };
            if n >= 1000 {
                big.push(n);
            }
        }
        if !big.is_empty() && !big.contains(&aud_bytes) {
            f.fail(
                "AC5",
                format!(
                    "the audio declared rate's bytes are not {aud_bytes} (cap + audio + 66): {}",
                    head(t, 100)
                ),
            );
        }
    }
    Ok(())
}
