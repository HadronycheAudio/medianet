//! The constants and the part sizes the check computes everything from.

use super::num::{Dec, floats, ints, nrm, to_int};
use crate::doccheck::pycompat::{PyStr, splitlines};
use crate::doccheck::pynum::{py_float, record_int_limit};
use crate::doccheck::report::{Earlier, Flow, crash, exit};
use crate::doccheck::tables::split_row;
use crate::doccheck::text::rx;
use crate::doccheck::tomlline::{Doc, Table, Value, Variant, parse};

/// Everything computed from the constants and wire.toml.
#[derive(Debug, Clone)]
pub(crate) struct Calc {
    pub slice_rate: i128,
    pub c20: i128,
    pub c21: f64,
    pub c21_ch: f64,
    pub c22: Vec<(i128, i128, i128, i128)>,
    pub cap: i128,
    pub ch: i128,
    pub audio: i128,
    pub def_pay: i128,
    pub ann_max_pay: i128,
    pub v1_largest_pay: i128,
    pub wire: i128,
    pub wire_max: i128,
    pub ovh: i128,
    pub eth: i128,
    pub bits_stream: i128,
    pub rate_stream: f64,
    pub rate_ch: f64,
    pub diff_b: i128,
    pub pct_b: f64,
    pub diff_r: f64,
    pub pct_r: f64,
    pub exceeds: bool,
    pub hdr_allowed: Vec<i128>,
    pub doc: Doc,
}

/// The largest magnitude the arithmetic is computed with: the port
/// multiplies a few such values in `i128` without overflow checks, so a
/// larger number is red by rule (`INT_LIMIT`, recorded) like one beyond
/// `i128`; the documents' values are a few thousand.
const WORK_LIMIT: i128 = 1_000_000_000_000;

/// `const_cell(cid)`: the third cell of the first row of `cid`, normalised.
fn const_cell(src: &str, cid: &str) -> Option<String> {
    for line in splitlines(src) {
        if !line.py_lstrip().starts_with('|') {
            continue;
        }
        let cells = split_row(line);
        if cells.len() >= 3 && cells[0] == cid {
            return Some(nrm(&cells[2]));
        }
    }
    None
}

/// `[0]` of a list: an empty one raises `IndexError`.
fn first<T: Copy>(v: &[T], what: &str) -> Flow<T> {
    v.first().copied().map_or_else(|| crash(what), Ok)
}

/// A value the arithmetic can use (see `WORK_LIMIT`).
fn small(n: Option<i128>, what: &str) -> Flow<i128> {
    match n {
        Some(n) if n.unsigned_abs() <= WORK_LIMIT.unsigned_abs() => Ok(n),
        Some(n) => {
            record_int_limit(&n.to_string());
            crash(format!("{what} beyond the work limit"))
        }
        None => crash(format!("{what}: int() failed")),
    }
}

/// The constants the check reads (`CELL` and what is parsed from it).
struct Consts {
    slice_rate: i128,
    c20: i128,
    c21: f64,
    c21_ch: f64,
    c22: Vec<(i128, i128, i128, i128)>,
    cap: i128,
    ch: i128,
    audio: i128,
}

fn consts(src: &str) -> Flow<Consts> {
    let cell =
        ["C-02", "C-03", "C-04", "C-20", "C-21", "C-22", "C-137"].map(|c| const_cell(src, c));
    let [
        Some(c02),
        Some(c03),
        Some(c04),
        Some(c20),
        Some(c21),
        Some(c22),
        Some(c137),
    ] = &cell
    else {
        return exit(
            "setup",
            "a constant row the wire-size check needs (C-02, C-03, C-04, C-20, C-21, C-22) is missing",
            Earlier::Kept,
        );
    };
    let search = |p: &str, t: &str| -> Flow<String> {
        rx(p)
            .captures(t)
            .map(|c| c[1].to_string())
            .map_or_else(|| crash("re.search is None"), Ok)
    };
    let slice_rate = small(to_int(&search(r"(\d[\d ]*) slices/s", c02)?), "SLICE_RATE")?;
    let frames = small(to_int(&search(r"(\d+) frames", c02)?), "FRAMES")?;
    let sample = small(Some(first(&ints(c03), "IndexError")?), "SAMPLE")?;
    let ch = small(Some(first(&ints(c04), "IndexError")?), "CH")?;
    let c20v = small(Some(first(&ints(c20), "IndexError")?), "C20")?;
    let c21v = first(&floats(c21), "IndexError")?.0;
    let mut c22v = Vec::new();
    for c in
        rx(r"(\d+) ch: (\d+) streams?, (\d+) Mbit/s, (\d[\d ]*\d|\d+) packets/s").captures_iter(c22)
    {
        let n: Vec<i128> = (1..=4)
            .map(|k| small(to_int(&c[k]), "C-22"))
            .collect::<Flow<_>>()?;
        c22v.push((n[0], n[1], n[2], n[3]));
    }
    if c22v.is_empty() {
        return exit(
            "setup",
            "C-22 does not parse as 'NN ch: N streams, N Mbit/s, N packets/s'",
            Earlier::Kept,
        );
    }
    let cap = small(to_int(&search(r"at most (\d+) bytes", c137)?), "CAP")?;
    let c21_ch = match py_float(&search(r"([\d.]+) Mbit/s per channel", c21)?) {
        Some(f) => f,
        None => return crash("ValueError: float()"),
    };
    Ok(Consts {
        slice_rate,
        c20: c20v,
        c21: c21v,
        c21_ch,
        c22: c22v,
        cap,
        ch,
        audio: ch * frames * sample,
    })
}

/// A value as Python arithmetic sees it: an int or a bool (`True + 1`).
fn int_like(v: &Value) -> Option<i128> {
    match v {
        Value::Int(n) => small(Some(*n), "toml int").ok(),
        Value::Bool(b) => Some(i128::from(*b)),
        Value::BigInt(s) => {
            record_int_limit(s);
            Some(0)
        }
        _ => None,
    }
}

/// `f.get(key, default)` in arithmetic.
fn geti(f: &Table, key: &str, default: i128) -> Flow<i128> {
    match f.get(key) {
        None => Ok(default),
        Some(v) => int_like(v).map_or_else(
            || crash(format!("{key}: {} in arithmetic", v.type_name())),
            Ok,
        ),
    }
}

/// `f["key"]` in arithmetic.
fn idx(f: &Table, key: &str) -> Flow<i128> {
    match f.get(key) {
        None => crash(format!("KeyError: {key}")),
        Some(v) => int_like(v).map_or_else(
            || crash(format!("{key}: {} in arithmetic", v.type_name())),
            Ok,
        ),
    }
}

/// `fbytes(f)`: `f["width"] // 8` when the width is an int.
fn fbytes(f: &Table) -> Option<i128> {
    f.get("width").and_then(int_like).map(|w| w.div_euclid(8))
}

/// A message's `fields` as the script iterates it.
fn fields(m: &Table) -> Flow<Vec<&Table>> {
    match m.get("fields") {
        None => Ok(Vec::new()),
        Some(Value::List(xs)) => xs
            .iter()
            .map(|x| {
                x.as_table()
                    .map_or_else(|| crash(format!("{}.get()", x.type_name())), Ok)
            })
            .collect(),
        Some(Value::Str(s)) if s.is_empty() => Ok(Vec::new()),
        Some(v) => crash(format!("iterating a {}", v.type_name())),
    }
}

fn field<'a>(m: &'a Table, name: &str) -> Flow<Option<&'a Table>> {
    Ok(fields(m)?
        .into_iter()
        .find(|f| f.get("name").and_then(Value::as_str) == Some(name)))
}

/// `BY_NAME.get(name)`: the last message of that name (a truthy list as a
/// name is an unhashable dict key).
fn message<'a>(d: &'a Doc, name: &str) -> Flow<Option<&'a Table>> {
    for m in &d.messages {
        if let Some(Value::List(xs)) = m.get("name")
            && !xs.is_empty()
        {
            return crash("TypeError: unhashable type: 'list'");
        }
    }
    Ok(d.messages
        .iter()
        .rev()
        .find(|m| m.get("name").and_then(Value::as_str) == Some(name)))
}

/// The part sizes of the toml: (pkt, slice, ann_count, entry, entry_max, ext).
fn parts(mp: &Table, count: &Table, entry: &Table) -> Flow<[i128; 6]> {
    let flds = fields(mp)?;
    let mut pkt = 0;
    for f in &flds {
        if geti(f, "offset", 0)? + geti(f, "width", 0)? <= 128 {
            pkt += idx(f, "width")?;
        }
    }
    let mut slice = 0;
    for f in &flds {
        let off = geti(f, "offset", -1)?;
        if 128 <= off && off < idx(count, "offset")? {
            slice += idx(f, "width")?;
        }
    }
    let (Some(ann_count), Some(ent)) = (fbytes(count), fbytes(entry)) else {
        return crash("None in arithmetic");
    };
    let entry_max = geti(entry, "max_count", 0)?;
    if entry_max > WORK_LIMIT / 1_000_000 {
        return crash("range(ENTRY_MAX + 1) does not finish");
    }
    let start = idx(entry, "offset")? + idx(entry, "width")? * entry_max;
    let mut ext = 0;
    for f in &flds {
        if geti(f, "offset", -1)? >= start {
            ext += idx(f, "width")?;
        }
    }
    Ok([
        pkt.div_euclid(8),
        slice.div_euclid(8),
        ann_count,
        ent,
        entry_max,
        ext.div_euclid(8),
    ])
}

/// The default total of a `[budget]`-like table (`budget_default`): the
/// last table whose name has "budget", a dict with an int `default_total`.
fn budget_default(d: &Doc) -> Option<i128> {
    let mut found = None;
    for (name, v) in &d.others.0 {
        if let (true, Value::Dict(t)) = (name.to_lowercase().contains("budget"), v) {
            match t.get("default_total") {
                Some(Value::Int(n)) => found = Some(*n),
                Some(Value::Bool(b)) => found = Some(i128::from(*b)),
                Some(Value::BigInt(s)) => {
                    record_int_limit(s);
                    found = Some(0);
                }
                _ => {}
            }
        }
    }
    found
}

/// `float` of a size entry: `max_size` as an int.
fn size(m: &Table) -> Flow<i128> {
    m.get("max_size")
        .and_then(int_like)
        .map_or_else(|| crash("None in arithmetic"), Ok)
}

/// `HDR_ALLOWED`: the header figures of the 004-14 tables a worked example
/// may use as its header term.
fn header_figures(p: [i128; 6], cwv: i128, cksum: i128, max_hdr: i128, cap: i128) -> Vec<i128> {
    let [pkt, slice, ann_count, ent, entry_max, ext] = p;
    let mut hdr = vec![
        pkt + slice + ann_count + cwv + cksum,
        pkt + slice + ann_count + ent + ext + cksum,
        pkt + slice + ann_count + ent + cksum,
        max_hdr,
        cap,
    ];
    for k in 0..=entry_max {
        for e in [0, 1] {
            hdr.push(pkt + slice + ann_count + ent * k + ext * e + cwv + cksum);
        }
        hdr.push(cwv + cksum + pkt + slice + ann_count + ent * k);
    }
    hdr
}

/// Reads everything; records the `[budget]` disagreement (AC 1).
pub(crate) fn calc(
    const_src: &str,
    toml_src: &str,
    f: &mut crate::doccheck::report::Report,
) -> Flow<Calc> {
    let c = consts(const_src)?;
    let doc = match parse(toml_src, Variant::V004_15) {
        Ok(d) => d,
        Err(e) if e.caught_by(Variant::V004_15) => {
            return exit(
                "setup",
                format!("wire.toml does not parse: {}", e.message),
                Earlier::Kept,
            );
        }
        Err(e) => return crash(format!("{}: {}", e.kind, e.message)),
    };
    let (mp, ck, cw) = (
        message(&doc, "media packet")?,
        message(&doc, "slice checksum")?,
        message(&doc, "content-message header")?,
    );
    let (count, entry) = match mp {
        Some(m) => (
            field(m, "announcement_count")?,
            field(m, "announcement_entry")?,
        ),
        None => (None, None),
    };
    let (Some(mp), Some(ck), Some(cw), Some(count), Some(entry)) = (mp, ck, cw, count, entry)
    else {
        return exit(
            "setup",
            "wire.toml lacks the media packet, checksum or content-message header tables",
            Earlier::Kept,
        );
    };
    let [pkt, slice, ann_count, ent, entry_max, ext] = parts(mp, count, entry)?;
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
    #[allow(clippy::cast_precision_loss)]
    let (rate_stream, diff_b) = (bits_stream as f64 / 1e6, wire - c.c20);
    #[allow(clippy::cast_precision_loss)]
    let (pct_b, rate_ch) = (
        diff_b as f64 / c.c20 as f64 * 100.0,
        rate_stream / c.ch as f64,
    );
    let diff_r = rate_stream - c.c21;
    let pct_r = diff_r / c.c21 * 100.0;
    let hdr = header_figures(
        [pkt, slice, ann_count, ent, entry_max, ext],
        cwv,
        cksum,
        ann_max_pay - c.audio,
        c.cap,
    );
    if let Some(b) = budget_default(&doc)
        && b != def_pay
    {
        f.fail(
            "AC1",
            format!(
                "wire.toml [budget] default_total is {b}, the 004-14 components sum to {def_pay}"
            ),
        );
    }
    Ok(Calc {
        slice_rate: c.slice_rate,
        c20: c.c20,
        c21: c.c21,
        c21_ch: c.c21_ch,
        c22: c.c22,
        cap: c.cap,
        ch: c.ch,
        audio: c.audio,
        def_pay,
        ann_max_pay,
        v1_largest_pay: v1,
        wire,
        wire_max: ann_max_pay + ovh,
        ovh,
        eth,
        bits_stream,
        rate_stream,
        rate_ch,
        diff_b,
        pct_b,
        diff_r,
        pct_r,
        exceeds: pct_b.abs().max(pct_r.abs()) > 5.0,
        hdr_allowed: hdr,
        doc,
    })
}

/// Keeps `Dec` in scope for the sibling modules.
#[allow(dead_code)]
pub(crate) type Decimal = Dec;
