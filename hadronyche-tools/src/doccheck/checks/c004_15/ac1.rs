//! AC 1: the on-wire size of one default packet, the Ethernet framing, the
//! bit rates per stream, channel and source, and the wire-size table.

use super::num::{Fails, R, close, floats, ints, mbit_list, tol_of, to_int, window};
use super::setup::Calc;
use super::units::{Eq, Sub, Unit, equations, mbits};
use crate::doccheck::text::{has_i, rx};

fn row_val(u: &Unit) -> R<Option<i64>> {
    for c in u.cells.iter().skip(1) {
        if let Some(m) = rx(r"\d[\d ]*\d|\d").find(c) {
            return Ok(Some(to_int(m.as_str())?));
        }
    }
    Ok(None)
}

/// The on-wire size of one default packet, with its arithmetic.
pub(crate) fn wire_size(sub: &Sub, eqs: &[Eq], k: &Calc, f: &mut Fails) -> R<()> {
    let wire_eq: Vec<&Eq> = eqs.iter().filter(|e| e.terms.contains(&k.def_pay)).collect();
    let wire = k.wire as f64;
    if let Some(bad) = wire_eq.iter().find(|e| e.rhs != wire) {
        f.add("AC1", format!("the default packet's on-wire size is stated as {}, but UDP payload {} + {} = {} (the 004-14 default total and the 66 bytes of overhead)", bad.rhs_raw, k.def_pay, k.ovh, k.wire));
    } else if !wire_eq.iter().any(|e| e.rhs == wire) {
        let (mut payload, mut total) = (false, false);
        for u in sub.units.iter().filter(|u| u.row) {
            let v = row_val(u)?;
            payload |= has_i("payload", &u.cells[0]) && v == Some(k.def_pay);
            total |= has_i("wire|total|on-the-wire", &u.cells[0]) && v == Some(k.wire);
        }
        if !(payload && total) {
            f.add("AC1", format!("no arithmetic or table states the on-wire size of one default packet: {} (UDP payload, the 004-14 default) + 8 + 20 + 38 = {}", k.def_pay, k.wire));
        }
    }
    Ok(())
}

/// The four Ethernet framing parts beside their names, and the total.
pub(crate) fn ethernet(sub: &Sub, k: &Calc, f: &mut Fails) -> R<()> {
    let keys: [(&str, i64, &str); 4] = [
        ("header", 14, r"(?i)Ethernet[^.;|]{0,30}\bheader\b|\bheader\b[^.;|]{0,30}Ethernet"),
        ("FCS", 4, r"(?i)\bFCS\b|frame check|check sequence"),
        ("preamble and start-of-frame delimiter", 8, r"(?i)preamble"),
        ("inter-frame gap", 12, r"(?i)inter-?frame|\bIFG\b"),
    ];
    let text = &sub.joined;
    let near = |m: regex::Match, v: i64| -> R<bool> {
        Ok(ints(&window(text, m.start(), m.end(), 80, 80))?.contains(&v))
    };
    for (name, value, pat) in keys {
        let mut hit = false;
        for m in rx(pat).find_iter(text) {
            if near(m, value)? {
                hit = true;
                break;
            }
        }
        if !hit {
            f.add("AC1", format!("the Ethernet {name} of {value} bytes is not stated beside its name"));
        }
    }
    let mut total = false;
    for m in rx(r"(?i)Ethernet").find_iter(text) {
        if near(m, k.eth)? {
            total = true;
            break;
        }
    }
    if !total {
        f.add("AC1", format!("the Ethernet framing total of {} bytes is not stated beside 'Ethernet'", k.eth));
    }
    Ok(())
}

fn has_rate(sub: &Sub, pred: impl Fn(&Unit) -> bool, value: f64) -> bool {
    sub.units.iter().any(|u| pred(u) && mbits(u).iter().any(|&(v, d)| close(v, value, tol_of(d))))
}

/// The bit rates per stream, per channel and per source (C-22).
pub(crate) fn rates(sub: &Sub, k: &Calc, f: &mut Fails) -> R<()> {
    if !has_rate(sub, |u| u.text.to_lowercase().contains("stream"), k.rate_stream) {
        f.add("AC1", format!("no bit rate per stream of {:.1} Mbit/s (wire {} bytes x 8 x {}) is stated", k.rate_stream, k.wire, k.slice_rate));
    }
    if !has_rate(sub, |u| u.text.to_lowercase().contains("channel"), k.rate_ch) {
        f.add("AC1", format!("no bit rate per channel of {:.2} Mbit/s (per stream / {}, C-04) is stated", k.rate_ch, k.ch));
    }
    for &(ch, streams, c22_rate, pps) in &k.c22 {
        let expect = streams as f64 * k.rate_stream;
        let mut ok = false;
        for u in &sub.units {
            let n = ints(&u.text)?;
            if n.contains(&ch) && n.contains(&streams) && n.contains(&pps) && n.contains(&c22_rate)
                && mbits(u).iter().any(|&(v, d)| close(v, expect, tol_of(d)))
            {
                ok = true;
                break;
            }
        }
        if !ok {
            f.add("AC1", format!("no unit gives {ch} channels: {streams} streams, {expect:.1} Mbit/s, {pps} packets/s beside C-22's {c22_rate} Mbit/s (streams = channels / {}, rate = streams x per-stream rate)", k.ch));
        }
    }
    Ok(())
}

fn col_of(header: &[String], word: &str) -> Option<usize> {
    header.iter().position(|h| h.to_lowercase().contains(word))
}

fn size_row(u: &Unit, k: &Calc, f: &mut Fails) -> R<()> {
    let (cells, header) = (&u.cells, &u.header);
    let name = &cells[0];
    let cols: Vec<Option<usize>> = ["payload", "wire", "stream", "channel"].iter().map(|w| col_of(header, w)).collect();
    let [Some(ip), Some(iw), Some(ist), Some(ich)] = cols[..] else {
        f.add("AC1", format!("wire-size row {name:?} lacks a payload, wire, stream or channel cell"));
        return Ok(());
    };
    if ip.max(iw).max(ist).max(ich) >= cells.len() {
        f.add("AC1", format!("wire-size row {name:?} lacks a payload, wire, stream or channel cell"));
        return Ok(());
    }
    let pay = ints(&cells[ip])?;
    let eqw = equations(&cells[iw])?;
    let (Some(&payload), Some(first)) = (pay.first(), eqw.first()) else {
        f.add("AC1", format!("wire-size row {name:?} states no payload or no on-the-wire arithmetic"));
        return Ok(());
    };
    let wire_row = first.rhs;
    let low = name.to_lowercase();
    let (expected, label) = if low.contains("default") {
        (Some(k.def_pay), "the 004-14 default total")
    } else if low.contains("largest unshared") {
        (Some(k.v1_largest_pay), "the 004-14 fallback one-announcement-and-extension total")
    } else if low.contains("format maximum") {
        (Some(k.ann_max_pay), "the 004-14 format-maximum total")
    } else {
        (None, "")
    };
    if expected.is_some_and(|e| e != payload) {
        f.add("AC1", format!("wire-size row {name:?}: payload {payload} is not {label} ({})", expected.unwrap_or(0)));
    }
    if wire_row != (payload + k.ovh) as f64 {
        f.add("AC1", format!("wire-size row {name:?}: on the wire {wire_row} is not payload {payload} + {}", k.ovh));
    }
    size_rates(u, [ist, ich], wire_row, k, f)
}

fn size_rates(u: &Unit, cols: [usize; 2], wire_row: f64, k: &Calc, f: &mut Fails) -> R<()> {
    let (cells, name) = (&u.cells, &u.cells[0]);
    let rate_mbit = wire_row * 8.0 * k.slice_rate as f64 / 1e6;
    let vals = mbit_list(&cells[cols[0]]);
    if vals.is_empty() {
        f.add("AC1", format!("wire-size row {name:?} states no Mbit/s rate"));
    }
    for (v, d) in vals {
        if !close(v, rate_mbit, tol_of(d)) {
            f.add("AC1", format!("wire-size row {name:?}: rate {v} Mbit/s is not {wire_row} x 8 x {} = {rate_mbit:.3} Mbit/s", k.slice_rate));
        }
    }
    for e in equations(&cells[cols[0]])? {
        let bits = wire_row * 8.0 * k.slice_rate as f64;
        if !close(e.rhs, bits, e.tol.max(1e-9)) {
            f.add("AC1", format!("wire-size row {name:?}: {:?} does not give {bits} bit/s", e.raw));
        }
    }
    let fl = floats(&cells[cols[1]]);
    match fl.first() {
        None => f.add("AC1", format!("wire-size row {name:?} states no per-channel rate")),
        Some(&(v, d)) if !close(v, rate_mbit / k.ch as f64, tol_of(d)) => {
            f.add("AC1", format!("wire-size row {name:?}: per-channel {v} Mbit/s is not {rate_mbit:.3} / {} = {:.3} Mbit/s (C-04)", k.ch, rate_mbit / k.ch as f64));
        }
        Some(_) => {}
    }
    Ok(())
}

/// Every row of the wire-size table (a header naming the payload and the
/// wire).
pub(crate) fn size_table(sub: &Sub, k: &Calc, f: &mut Fails) -> R<()> {
    for u in &sub.units {
        let named = |w: &str| u.header.iter().any(|h| h.to_lowercase().contains(w));
        if u.row && named("payload") && named("wire") {
            size_row(u, k, f)?;
        }
    }
    Ok(())
}
