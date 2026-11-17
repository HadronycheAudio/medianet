//! The constants the budget needs, and the part sizes computed from the
//! landed toml tables (read by the library's `Line004_14` reader).

use crate::doccheck::pycompat::{PyStr, splitlines};
use crate::doccheck::pynum::record_int_limit;
use crate::doccheck::report::{Earlier, Flow, Report, crash, exit};
use crate::doccheck::tables::{first_int_004_14, split_row};
use crate::doccheck::text::rx;
use crate::doccheck::tomlline::{Doc, Table, Value};

/// The constants of 02-constants.md the budget reads.
#[derive(Debug, Clone)]
pub(crate) struct Consts {
    pub c04: Option<String>,
    pub sample: i128,
    pub audio: i128,
    pub fb_pay: i128,
    pub cap: i128,
    pub pay_def: i128,
    pub pay_fb: i128,
}

/// `const_value(cid)`: the third cell of the first row of `cid`.
pub(crate) fn const_value(src: &str, cid: &str) -> Option<String> {
    for line in splitlines(src) {
        if !line.py_lstrip().starts_with('|') {
            continue;
        }
        let cells = split_row(line);
        if cells.len() >= 3 && cells[0] == cid {
            return Some(cells[2].clone());
        }
    }
    None
}

/// The largest magnitude the budget arithmetic is computed with: the port
/// multiplies up to three such values in `i128` without overflow checks, so
/// a larger number is red by rule (`INT_LIMIT`, recorded) like one beyond
/// `i128`; the documents' values are a few thousand.
const WORK_LIMIT: i128 = 1_000_000_000_000;

pub(crate) fn limited(n: i128) -> i128 {
    if n.unsigned_abs() > WORK_LIMIT.unsigned_abs() {
        record_int_limit(&n.to_string());
        return 0;
    }
    n
}

/// `first_int(const_value(cid))`, `None` when there is no such number.
fn first(src: &str, cid: &str) -> Option<i128> {
    first_int_004_14(&const_value(src, cid)?)
        .ok()
        .flatten()
        .map(limited)
}

/// Reads the constants; `Err` is the script's `FAIL setup` exit.
pub(crate) fn consts(src: &str) -> Flow<Consts> {
    let (ch, frames, sample) = (first(src, "C-04"), first(src, "C-02"), first(src, "C-03"));
    let (fb_mtu, fb_pay) = (first(src, "C-06"), first(src, "C-17"));
    let cap = rx(r"at most\s+(\d+)\s+bytes")
        .captures(&const_value(src, "C-137").unwrap_or_default())
        .and_then(|c| crate::doccheck::pycompat::parse_int(&c[1]).ok())
        .map(limited);
    let (Some(ch), Some(frames), Some(sample), Some(fb_mtu), Some(fb_pay), Some(cap)) =
        (ch, frames, sample, fb_mtu, fb_pay, cap)
    else {
        return exit(
            "setup",
            "a constant the budget needs (C-02, C-03, C-04, C-06, C-17, C-137) is unreadable",
            Earlier::Kept,
        );
    };
    let pay_fb = fb_mtu - 20 - 8;
    if pay_fb != fb_pay {
        return exit(
            "setup",
            format!("C-06 minus IPv4 and UDP headers is {pay_fb}, but C-17 states {fb_pay}"),
            Earlier::Kept,
        );
    }
    let audio = ch * frames * sample;
    Ok(Consts {
        c04: const_value(src, "C-04"),
        sample,
        audio,
        fb_pay,
        cap,
        pay_def: 1500 - 20 - 8,
        pay_fb,
    })
}

/// The part sizes the toml declares (`COMP`).
#[derive(Debug, Clone)]
pub(crate) struct Comp {
    pub pkt: i128,
    pub slice: i128,
    pub ann_count: i128,
    pub entry: i128,
    pub entry_max: i128,
    pub ext: i128,
    pub cksum: i128,
    pub cw: i128,
    /// `None` when the script's value is not an int (an error only where
    /// it is used in arithmetic).
    pub transport: Option<i128>,
    pub sp_entry: Option<i128>,
    pub sp_hdr: i128,
    pub sample_bytes: Option<i128>,
}

impl Comp {
    pub fn ann_max(&self) -> i128 {
        self.ann_count + self.entry * self.entry_max
    }

    pub fn mandatory(&self) -> i128 {
        self.pkt + self.slice + self.ann_count + self.cw + self.cksum
    }

    pub fn max_non_audio(&self) -> i128 {
        self.mandatory() + self.entry * self.entry_max + self.ext
    }

    /// `ALLOWED_ANN_BYTES`.
    pub fn allowed_ann(&self) -> Vec<i128> {
        (0..=self.entry_max)
            .map(|k| self.ann_count + self.entry * k)
            .collect()
    }

    /// `ALLOWED_COUNTED`.
    pub fn allowed_counted(&self) -> Vec<i128> {
        let base = self.pkt + self.slice + self.ann_count + self.cw;
        let mut out = Vec::new();
        for k in 0..=self.entry_max {
            for e in [0, self.ext] {
                out.push(base + self.entry * k + e);
            }
        }
        out
    }
}

/// A value as Python arithmetic and `isinstance(v, int)` see it: an int or
/// a bool; an int beyond `i128` is red by rule (recorded) and reads 0.
fn int_like(v: &Value) -> Option<i128> {
    match v {
        Value::Int(n) => Some(limited(*n)),
        Value::Bool(b) => Some(i128::from(*b)),
        Value::BigInt(s) => {
            record_int_limit(s);
            Some(0)
        }
        _ => None,
    }
}

/// `f.get(key, default)` used in arithmetic: a string, `None`, a list or a
/// dict raises `TypeError`.
fn geti(f: &Table, key: &str, default: i128) -> Flow<i128> {
    match f.get(key) {
        None => Ok(default),
        Some(v) => int_like(v).map_or_else(
            || crash(format!("{key}: {} in arithmetic", v.type_name())),
            Ok,
        ),
    }
}

/// `f["key"]` in arithmetic: a missing key raises `KeyError`.
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
fn fbytes(f: Option<&Table>) -> Option<i128> {
    let w = f?.get("width").and_then(int_like)?;
    Some(w.div_euclid(8))
}

/// A message's `fields` as the script iterates it (`for f in m["fields"]`,
/// then `f.get(...)`): a list of dicts; an empty string iterates nothing; any
/// other value, or a non-dict item, raises.
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

fn field<'a>(m: Option<&'a Table>, name: &str) -> Flow<Option<&'a Table>> {
    let Some(m) = m else { return Ok(None) };
    Ok(fields(m)?
        .into_iter()
        .find(|f| f.get("name").and_then(Value::as_str) == Some(name)))
}

/// `BY_NAME = {m.get("name"): m for m in MESSAGES if m.get("name")}`: a
/// truthy list as a name is an unhashable dict key.
fn by_name_hashable(d: &Doc) -> Flow<()> {
    for m in &d.messages {
        if let Some(Value::List(xs)) = m.get("name")
            && !xs.is_empty() {
                return crash("TypeError: unhashable type: 'list'");
            }
    }
    Ok(())
}

/// `msg(name)`, recording AC 5 when the message is missing (the script's
/// `BY_NAME`: a later message of the same name wins).
fn msg<'a>(d: &'a Doc, name: &str, r: &mut Report) -> Option<&'a Table> {
    let found = d
        .messages
        .iter()
        .rev()
        .find(|m| m.get("name").and_then(Value::as_str) == Some(name));
    if found.is_none() {
        r.fail(
            "AC5",
            format!("wire.toml has no [[message]] named {name:?}"),
        );
    }
    found
}

/// The pieces of `COMP` that may be absent or `None` while still "present".
#[derive(Default)]
struct Raw {
    pkt: Option<i128>,
    slice: Option<i128>,
    ann_count: Option<Option<i128>>,
    entry: Option<Option<i128>>,
    entry_max: Option<Option<i128>>,
    ext: Option<i128>,
    ext_start: Option<i128>,
    cksum: Option<Option<i128>>,
    cw: Option<Option<i128>>,
    transport: Option<Option<i128>>,
    sp_entry: Option<Option<i128>>,
    sp_hdr: Option<i128>,
    sample_bytes: Option<Option<i128>>,
}

fn size_of(m: &Table) -> Option<i128> {
    m.get("max_size").and_then(int_like)
}

fn media_parts(mp: &Table, count: &Table, entry: &Table, raw: &mut Raw) -> Flow<()> {
    let flds = fields(mp)?;
    let mut hdr = 0;
    for f in &flds {
        if geti(f, "offset", 0)? + geti(f, "width", 0)? <= 128 {
            hdr += idx(f, "width")?;
        }
    }
    raw.pkt = Some(hdr.div_euclid(8));
    let mut sl = 0;
    for f in &flds {
        let off = geti(f, "offset", -1)?;
        if 128 <= off && off < idx(count, "offset")? {
            sl += idx(f, "width")?;
        }
    }
    raw.slice = Some(sl.div_euclid(8));
    raw.ann_count = Some(fbytes(Some(count)));
    raw.entry = Some(fbytes(Some(entry)));
    raw.entry_max = Some(entry.get("max_count").and_then(int_like));
    let start = idx(entry, "offset")? + idx(entry, "width")? * idx(entry, "max_count")?;
    let mut ext = 0;
    for f in &flds {
        if geti(f, "offset", -1)? >= start {
            ext += idx(f, "width")?;
        }
    }
    raw.ext = Some(ext.div_euclid(8));
    raw.ext_start = Some(start);
    Ok(())
}

/// Computes `COMP` and runs the toml consistency checks (AC 5).
pub(crate) fn components(d: &Doc, c: &Consts, r: &mut Report) -> Flow<Comp> {
    by_name_hashable(d)?;
    let mp = msg(d, "media packet", r);
    let count_f = field(mp, "announcement_count")?;
    let entry_f = field(mp, "announcement_entry")?;
    let ck = msg(d, "slice checksum", r);
    let cw_m = msg(d, "content-message header", r);
    let audio_m = msg(d, "audio message", r);
    let transport_m = msg(d, "transport state message", r);
    let sp_entry_m = msg(d, "sparse keyed entry", r);
    let sparse_m = msg(d, "sparse keyed controller message", r);
    let sparse_entry_f = field(sparse_m, "entry")?;
    let sample_f = field(audio_m, "sample")?;
    let mut raw = Raw::default();
    if let (Some(mp), Some(count), Some(entry)) = (mp, count_f, entry_f) {
        media_parts(mp, count, entry, &mut raw)?;
    }
    if let Some(ck) = ck {
        raw.cksum = Some(size_of(ck));
        if fbytes(field(Some(ck), "checksum")?) != size_of(ck) {
            r.fail(
                "AC5",
                "wire.toml 'slice checksum' max_size does not equal its field width",
            );
        }
    }
    raw.cw = cw_m.map(size_of);
    raw.transport = transport_m.map(size_of);
    raw.sp_entry = sp_entry_m.map(size_of);
    if let (Some(sm), Some(sef), Some(sp_m)) = (sparse_m, sparse_entry_f, sp_entry_m)
        && sp_m.get("max_size").is_some_and(|v| *v != Value::None) {
            let (Some(max), Some(sp)) = (size_of(sm), size_of(sp_m)) else {
                return crash("TypeError in max_size - sp_entry * max_count");
            };
            raw.sp_hdr = Some(max - sp * geti(sef, "max_count", 0)?);
        }
    raw.sample_bytes = audio_m.and(sample_f).map(|f| fbytes(Some(f)));
    finish(raw, mp, c, r)
}

fn finish(raw: Raw, mp: Option<&Table>, c: &Consts, r: &mut Report) -> Flow<Comp> {
    let missing: Vec<&str> = [
        ("pkt", raw.pkt.is_none()),
        ("slice", raw.slice.is_none()),
        ("ann_count", raw.ann_count.is_none()),
        ("ann_entry", raw.entry.is_none()),
        ("ann_max_n", raw.entry_max.is_none()),
        ("ext", raw.ext.is_none()),
        ("cksum", raw.cksum.is_none()),
        ("cw", raw.cw.is_none()),
        ("transport", raw.transport.is_none()),
        ("sp_entry", raw.sp_entry.is_none()),
        ("sp_hdr", raw.sp_hdr.is_none()),
        ("sample_bytes", raw.sample_bytes.is_none()),
    ]
    .iter()
    .filter(|(_, gone)| *gone)
    .map(|(k, _)| *k)
    .collect();
    if !missing.is_empty() {
        let why = format!(
            "wire.toml does not declare what the budget needs: {}",
            missing.join(", ")
        );
        return exit("AC5", why, Earlier::Dropped);
    }
    let need =
        |v: Option<Option<i128>>| v.flatten().map_or_else(|| crash("None in arithmetic"), Ok);
    if need(raw.entry_max)? > WORK_LIMIT / 1_000_000 {
        return crash("range(ENTRY_MAX + 1) does not finish");
    }
    let comp = Comp {
        pkt: raw.pkt.unwrap_or_default(),
        slice: raw.slice.unwrap_or_default(),
        ann_count: need(raw.ann_count)?,
        entry: need(raw.entry)?,
        entry_max: need(raw.entry_max)?,
        ext: raw.ext.unwrap_or_default(),
        cksum: need(raw.cksum)?,
        cw: need(raw.cw)?,
        transport: raw.transport.flatten(),
        sp_entry: raw.sp_entry.flatten(),
        sp_hdr: raw.sp_hdr.unwrap_or_default(),
        sample_bytes: raw.sample_bytes.flatten(),
    };
    if comp.sample_bytes != Some(c.sample) {
        let got = comp
            .sample_bytes
            .map_or("None".to_string(), |n| n.to_string());
        r.fail(
            "AC5",
            format!(
                "wire.toml audio sample is {got} bytes, C-03 says {}",
                c.sample
            ),
        );
    }
    if comp.ann_count != 1 {
        r.fail("AC5", "wire.toml announcement count is not one byte");
    }
    let start = raw.ext_start.unwrap_or_default();
    let want = comp.pkt * 8 + comp.slice * 8 + comp.ann_count * 8 + comp.entry * 8 * comp.entry_max;
    if start != want {
        r.fail(
            "AC5",
            "the measurement extension does not start where the maximal container ends",
        );
    }
    let total = comp.pkt + comp.slice + comp.ann_count + comp.entry * comp.entry_max + comp.ext;
    if mp.and_then(size_of) != Some(total) {
        r.fail(
            "AC5",
            "wire.toml 'media packet' max_size is not header + slice + container max + extension",
        );
    }
    Ok(comp)
}
