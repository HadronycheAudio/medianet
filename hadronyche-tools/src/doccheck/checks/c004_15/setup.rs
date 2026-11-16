//! The constants and the part sizes the check computes everything from.

use super::num::{Dec, Fails, R, Stop, crash, floats, ints, nrm, to_int};
use crate::doccheck::checks::c004_14::pytoml::{Dict, Flavor, Message, Other, Parsed, V, parse};
use crate::doccheck::tables::split_row;
use crate::doccheck::text::{rx, splitlines};

/// Everything computed from the constants and wire.toml.
#[derive(Debug, Clone)]
pub(crate) struct Calc {
    pub slice_rate: i64,
    pub c20: i64,
    pub c21: f64,
    pub c21_ch: f64,
    pub c22: Vec<(i64, i64, i64, i64)>,
    pub cap: i64,
    pub ch: i64,
    pub audio: i64,
    pub def_pay: i64,
    pub ann_max_pay: i64,
    pub v1_largest_pay: i64,
    pub wire: i64,
    pub wire_max: i64,
    pub ovh: i64,
    pub eth: i64,
    pub bits_stream: i64,
    pub rate_stream: f64,
    pub rate_ch: f64,
    pub diff_b: i64,
    pub pct_b: f64,
    pub diff_r: f64,
    pub pct_r: f64,
    pub exceeds: bool,
    pub hdr_allowed: Vec<i64>,
    pub parsed: Parsed,
}

/// `const_cell(cid)`: the third cell of the first row of `cid`, normalised.
fn const_cell(src: &str, cid: &str) -> Option<String> {
    for line in splitlines(src) {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let cells = split_row(line);
        if cells.len() >= 3 && cells[0] == cid {
            return Some(nrm(&cells[2]));
        }
    }
    None
}

fn first<T: Copy>(v: &[T], what: &str) -> R<T> {
    v.first().copied().map_or_else(|| crash(what), Ok)
}

/// The constants the check reads (`CELL` and what is parsed from it).
struct Consts {
    slice_rate: i64,
    c20: i64,
    c21: f64,
    c21_ch: f64,
    c22: Vec<(i64, i64, i64, i64)>,
    cap: i64,
    ch: i64,
    audio: i64,
}

fn consts(src: &str) -> R<Consts> {
    let mut cell = Vec::new();
    for cid in ["C-02", "C-03", "C-04", "C-20", "C-21", "C-22", "C-137"] {
        cell.push(const_cell(src, cid));
    }
    let [Some(c02), Some(c03), Some(c04), Some(c20), Some(c21), Some(c22), Some(c137)] = &cell[..] else {
        return Err(Stop::Exit("setup", "a constant row the wire-size check needs (C-02, C-03, C-04, C-20, C-21, C-22) is missing".into()));
    };
    let search = |p: &str, t: &str| -> R<String> {
        rx(p).captures(t).map(|c| c[1].to_string()).map_or_else(|| crash("re.search is None"), Ok)
    };
    let slice_rate = to_int(&search(r"(\d[\d ]*) slices/s", c02)?)?;
    let frames: i64 = search(r"(\d+) frames", c02)?.parse().unwrap_or(0);
    let sample = first(&ints(c03)?, "IndexError")?;
    let ch = first(&ints(c04)?, "IndexError")?;
    let c20v = first(&ints(c20)?, "IndexError")?;
    let c21v = first(&floats(c21), "IndexError")?.0;
    let mut c22v = Vec::new();
    for c in rx(r"(\d+) ch: (\d+) streams?, (\d+) Mbit/s, (\d[\d ]*\d|\d+) packets/s").captures_iter(c22) {
        c22v.push((to_int(&c[1])?, to_int(&c[2])?, to_int(&c[3])?, to_int(&c[4])?));
    }
    if c22v.is_empty() {
        return Err(Stop::Exit("setup", "C-22 does not parse as 'NN ch: N streams, N Mbit/s, N packets/s'".into()));
    }
    let cap: i64 = search(r"at most (\d+) bytes", c137)?.parse().unwrap_or(0);
    let c21_ch: f64 = search(r"([\d.]+) Mbit/s per channel", c21)?.parse().or_else(|_| crash("float()"))?;
    Ok(Consts { slice_rate, c20: c20v, c21: c21v, c21_ch, c22: c22v, cap, ch, audio: ch * frames * sample })
}

fn geti(f: &Dict, key: &str, default: i64) -> R<i64> {
    match f.get(key) {
        None => Ok(default),
        Some(v) => v.as_int().map_or_else(|| crash("not a number"), Ok),
    }
}

fn idx(f: &Dict, key: &str) -> R<i64> {
    f.get(key).and_then(V::as_int).map_or_else(|| crash("KeyError"), Ok)
}

fn fbytes(f: &Dict) -> Option<i64> {
    f.get("width").and_then(V::as_int).map(|w| w.div_euclid(8))
}

fn field<'a>(m: &'a Message, name: &str) -> Option<&'a Dict> {
    m.fields.iter().find(|f| f.text("name") == Some(name))
}

fn message<'a>(p: &'a Parsed, name: &str) -> Option<&'a Message> {
    p.messages.iter().rev().find(|m| m.keys.text("name") == Some(name))
}

/// The part sizes of the toml: (pkt, slice, ann_count, entry, entry_max, ext).
fn parts(mp: &Message, count: &Dict, entry: &Dict) -> R<[i64; 6]> {
    let mut pkt = 0;
    for f in &mp.fields {
        if geti(f, "offset", 0)? + geti(f, "width", 0)? <= 128 {
            pkt += idx(f, "width")?;
        }
    }
    let count_off = idx(count, "offset")?;
    let mut slice = 0;
    for f in &mp.fields {
        let off = geti(f, "offset", -1)?;
        if 128 <= off && off < count_off {
            slice += idx(f, "width")?;
        }
    }
    let Some(ann_count) = fbytes(count) else { return crash("None in arithmetic") };
    let Some(ent) = fbytes(entry) else { return crash("None in arithmetic") };
    let entry_max = geti(entry, "max_count", 0)?;
    let start = idx(entry, "offset")? + idx(entry, "width")? * entry_max;
    let mut ext = 0;
    for f in &mp.fields {
        if geti(f, "offset", -1)? >= start {
            ext += idx(f, "width")?;
        }
    }
    Ok([pkt.div_euclid(8), slice.div_euclid(8), ann_count, ent, entry_max, ext.div_euclid(8)])
}

/// The default total of a `[budget]`-like table (`budget_default`).
fn budget_default(p: &Parsed) -> Option<i64> {
    let mut found = None;
    for (name, o) in &p.others {
        if let (true, Other::One(d)) = (name.to_lowercase().contains("budget"), o)
            && let Some(V::Int(n)) = d.get("default_total") {
                found = Some(*n);
            }
    }
    found
}

/// Reads everything; records the `[budget]` disagreement (AC 1).
pub(crate) fn calc(const_src: &str, toml_src: &str, f: &mut Fails) -> R<Calc> {
    let c = consts(const_src)?;
    let parsed = parse(toml_src, Flavor::Capacity).map_err(|b| match b {
        crate::doccheck::checks::c004_14::pytoml::Bad::Value => Stop::Exit("setup", "wire.toml does not parse".into()),
        crate::doccheck::checks::c004_14::pytoml::Bad::Crash => Stop::Crash("AttributeError".into()),
    })?;
    let (mp, ck, cw) = (message(&parsed, "media packet"), message(&parsed, "slice checksum"), message(&parsed, "content-message header"));
    let (count, entry) = match mp {
        Some(m) => (field(m, "announcement_count"), field(m, "announcement_entry")),
        None => (None, None),
    };
    let (Some(mp), Some(ck), Some(cw), Some(count), Some(entry)) = (mp, ck, cw, count, entry) else {
        return Err(Stop::Exit("setup", "wire.toml lacks the media packet, checksum or content-message header tables".into()));
    };
    let [pkt, slice, ann_count, ent, entry_max, ext] = parts(mp, count, entry)?;
    let size = |m: &Message| m.keys.get("max_size").and_then(V::as_int).map_or_else(|| crash("None in arithmetic"), Ok);
    let (cksum, cwv) = (size(ck)?, size(cw)?);
    let def_pay = pkt + slice + ann_count + cwv + cksum + c.audio;
    let ann_max_pay = pkt + slice + ann_count + ent * entry_max + ext + cwv + cksum + c.audio;
    let v1 = pkt + slice + ann_count + ent + ext + cwv + cksum + c.audio;
    let (ovh, eth) = (28 + 38, 38);
    let wire = def_pay + ovh;
    let bits_stream = wire * 8 * c.slice_rate;
    if c.ch == 0 || c.c20 == 0 || c.c21 == 0.0 {
        return crash("ZeroDivisionError");
    }
    let rate_stream = bits_stream as f64 / 1e6;
    let diff_b = wire - c.c20;
    let pct_b = diff_b as f64 / c.c20 as f64 * 100.0;
    let diff_r = rate_stream - c.c21;
    let pct_r = diff_r / c.c21 * 100.0;
    let mut hdr = vec![pkt + slice + ann_count + cwv + cksum, pkt + slice + ann_count + ent + ext + cksum, pkt + slice + ann_count + ent + cksum, ann_max_pay - c.audio, c.cap];
    for k in 0..=entry_max {
        for e in [0, 1] {
            hdr.push(pkt + slice + ann_count + ent * k + ext * e + cwv + cksum);
        }
        hdr.push(cwv + cksum + pkt + slice + ann_count + ent * k);
    }
    if let Some(b) = budget_default(&parsed)
        && b != def_pay {
            f.add("AC1", format!("wire.toml [budget] default_total is {b}, the 004-14 components sum to {def_pay}"));
        }
    Ok(Calc {
        slice_rate: c.slice_rate, c20: c.c20, c21: c.c21, c21_ch: c.c21_ch, c22: c.c22, cap: c.cap, ch: c.ch,
        audio: c.audio, def_pay, ann_max_pay, v1_largest_pay: v1, wire, wire_max: ann_max_pay + ovh, ovh, eth,
        bits_stream, rate_stream, rate_ch: rate_stream / c.ch as f64, diff_b, pct_b, diff_r, pct_r,
        exceeds: pct_b.abs().max(pct_r.abs()) > 5.0, hdr_allowed: hdr, parsed,
    })
}

/// Keeps `Dec` in scope for the sibling modules.
#[allow(dead_code)]
pub(crate) type Decimal = Dec;
