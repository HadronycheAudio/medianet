//! AC 2: the differences from C-20 and C-21, the C-22 comparison, and the
//! figures stated beside them.

use super::num::{
    Fails, close, dec_value, floats, has_number, ints, mbit_list, pcts, sentences, tol_of,
};
use super::setup::Calc;
use super::units::{Sub, Unit};
use crate::doccheck::pynum::format_float;
use crate::doccheck::report::Flow;
use crate::doccheck::text::{has_i, rx};

fn sentences_of(u: &Unit) -> Vec<String> {
    if u.row {
        vec![u.own.clone()]
    } else {
        sentences(&u.text)
    }
}

fn pct_ok(v: f64, d: usize, want: f64) -> bool {
    (v - want.abs()).abs() <= 0.1_f64.max(tol_of(d))
}

fn head(x: &str, n: usize) -> String {
    x.chars().take(n).collect()
}

/// The differences from C-20 (bytes) and C-21 (rate), absolute and percent.
pub(crate) fn differences(sub: &Sub, k: &Calc, f: &mut Fails) -> Flow<()> {
    let units: Vec<String> = sub.units.iter().flat_map(sentences_of).collect();
    let (mut ok_bytes, mut ok_rate) = (false, false);
    let c21 = format_float(k.c21, "g").unwrap_or_default();
    for x in &units {
        let n = ints(x);
        let about_c20 = n.contains(&k.c20) && n.contains(&k.diff_b.abs());
        if about_c20 {
            ok_bytes |= pcts(x).iter().any(|&(v, d)| pct_ok(v, d, k.pct_b));
            for (v, _) in pcts(x).into_iter().filter(|&(v, d)| !pct_ok(v, d, k.pct_b)) {
                f.fail("AC2", format!("a percent beside C-20 and the {}-byte difference is {v} %, but the computed share is {:.2} %: {}", k.diff_b.abs(), k.pct_b.abs(), head(x, 100)));
            }
        }
        let fl = floats(x);
        if has_number(x, "21.1") && fl.iter().any(|&(v, d)| close(v, k.diff_r.abs(), tol_of(d))) {
            for (v, _) in pcts(x).into_iter().filter(|&(v, d)| !pct_ok(v, d, k.pct_r)) {
                f.fail("AC2", format!("a percent beside C-21 and the {:.3} Mbit/s difference is {v} %, but the computed share is {:.2} %: {}", k.diff_r.abs(), k.pct_r.abs(), head(x, 100)));
            }
        }
        if has_number(x, &c21)
            && fl.iter().any(|&(v, d)| close(v, k.diff_r.abs(), tol_of(d)))
            && pcts(x).iter().any(|&(v, d)| pct_ok(v, d, k.pct_r))
        {
            ok_rate = true;
        }
    }
    if !ok_bytes {
        f.fail("AC2", format!("no unit gives the byte difference {:+} bytes against {} (C-20) and {:+.1} % in one sentence or row", k.diff_b, k.c20, k.pct_b));
    }
    if !ok_rate {
        f.fail("AC2", format!("no unit gives the rate difference {:+.2} Mbit/s against {} (C-21) and {:+.1} % in one sentence or row", k.diff_r, k.c21, k.pct_r));
    }
    Ok(())
}

/// A signed figure: (magnitude, decimals, sign).
fn signed(pattern: &str, t: &str) -> Vec<(f64, usize, f64)> {
    rx(pattern)
        .captures_iter(t)
        .filter_map(|m| {
            let (v, d) = dec_value(&m[2], m.get(3).map(|x| x.as_str()))?;
            Some((
                v,
                d,
                if &m[1] == "\u{2212}" || &m[1] == "-" {
                    -1.0
                } else {
                    1.0
                },
            ))
        })
        .collect()
}

const SIGNED_MBIT: &str = r"(?<![\w\-])([−+-])\s*(\d+)(?:\.(\d+))?\s*Mbit/s";
const SIGNED_PCT: &str = r"(?<![\w\-])([−+-])\s*(\d+)(?:\.(\d+))?\s*%";

/// Every stated difference and percent against C-22 is evaluated.
pub(crate) fn against_c22(sub: &Sub, k: &Calc, f: &mut Fails) -> Flow<()> {
    for &(ch, streams, c22_rate, _) in &k.c22 {
        let exp_diff = streams as f64 * k.rate_stream - c22_rate as f64;
        let base2 = streams as f64 * k.c21;
        for t in sub.units.iter().flat_map(sentences_of) {
            let n = ints(&t);
            if !n.contains(&ch) || !n.contains(&c22_rate) {
                continue;
            }
            for (v, d, sg) in signed(SIGNED_MBIT, &t) {
                if !close(sg * v, exp_diff, tol_of(d)) {
                    f.fail("AC2", format!("the {ch}-channel difference from C-22 is stated as {} Mbit/s, but the computed difference is {exp_diff:+.3} Mbit/s ({streams} x {:.3} - {c22_rate}): {}", sg * v, k.rate_stream, head(&t, 100)));
                }
            }
            let unrounded = floats(&t).iter().any(|&(v, _)| (v - base2).abs() <= 0.05);
            let bases: Vec<f64> = if unrounded {
                vec![base2]
            } else {
                vec![c22_rate as f64, base2]
            };
            for (p, d, _) in signed(SIGNED_PCT, &t) {
                if !bases
                    .iter()
                    .any(|b| (p - exp_diff.abs() / b * 100.0).abs() <= 0.1_f64.max(tol_of(d)))
                {
                    f.fail("AC2", format!("the {ch}-channel share of the C-22 difference is stated as {p} %, but {:.3} over {bases:?} is a different share: {}", exp_diff.abs(), head(&t, 100)));
                }
            }
        }
    }
    Ok(())
}

/// A percent stated against "C-21 x streams" is the per-stream share.
pub(crate) fn share_against_c21(sub: &Sub, k: &Calc, f: &mut Fails) {
    let trigger = rx(r"(?i)C-21 ?[\u{d7}x] ?streams|C-21 times|before C-22");
    for t in sub
        .units
        .iter()
        .flat_map(sentences_of)
        .filter(|t| trigger.is_match(t))
    {
        for (p, d, sg) in signed(SIGNED_PCT, &t) {
            if !close(sg * p, k.pct_r, 0.1_f64.max(tol_of(d))) {
                f.fail("AC2", format!("the share against C-21 x streams is stated as {} %, but the per-stream share is {:+.2} % ((per-stream - C-21) / C-21): {}", sg * p, k.pct_r, head(&t, 100)));
            }
        }
        for m in rx(r"(?<![\w.\-−+])(\d+)(?:\.(\d+))?\s*%").captures_iter(&t) {
            let Some((p, d)) = dec_value(&m[1], m.get(2).map(|x| x.as_str())) else {
                continue;
            };
            if !close(p, k.pct_r.abs(), 0.1_f64.max(tol_of(d))) {
                f.fail("AC2", format!("the share against C-21 x streams is stated as {p} %, but the per-stream share is {:.2} %: {}", k.pct_r.abs(), head(&t, 100)));
            }
        }
    }
}

/// A per-channel figure in a sentence is the computed rate, C-21's, or
/// their difference.
pub(crate) fn per_channel(sub: &Sub, k: &Calc, f: &mut Fails) {
    for u in sub.units.iter().filter(|u| !u.row) {
        for t in sentences(&u.text)
            .iter()
            .filter(|t| has_i("per channel", t))
        {
            let mut allowed = vec![
                (k.rate_ch, 2),
                (k.c21_ch, 1),
                ((k.rate_ch - k.c21_ch).abs(), 2),
            ];
            if t.to_lowercase().contains("stream") {
                allowed.push((k.rate_stream, 2));
            }
            for (v, d) in mbit_list(t) {
                if !allowed
                    .iter()
                    .any(|&(a, dd)| close(v, a, tol_of(d).max(tol_of(dd))))
                {
                    f.fail("AC2", format!("a per-channel figure of {v} Mbit/s is neither the computed {:.3} (per stream / {}, C-04) nor C-21's {} or their difference: {}", k.rate_ch, k.ch, k.c21_ch, head(t, 100)));
                }
            }
        }
    }
}
