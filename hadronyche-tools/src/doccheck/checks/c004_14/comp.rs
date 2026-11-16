//! The constants the budget needs, and the part sizes computed from the
//! landed toml tables.

use super::pytoml::{Dict, Message, Parsed, V};
use super::util::{Fails, R, Stop, crash, first_int};
use crate::doccheck::tables::split_row;
use crate::doccheck::text::{rx, splitlines};

/// The constants of 02-constants.md the budget reads.
#[derive(Debug, Clone)]
pub(crate) struct Consts {
    pub c04: Option<String>,
    pub sample: i64,
    pub audio: i64,
    pub fb_pay: i64,
    pub cap: i64,
    pub pay_def: i64,
    pub pay_fb: i64,
}

/// `const_value(cid)`: the third cell of the first row of `cid`.
pub(crate) fn const_value(src: &str, cid: &str) -> Option<String> {
    for line in splitlines(src) {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let cells = split_row(line);
        if cells.len() >= 3 && cells[0] == cid {
            return Some(cells[2].clone());
        }
    }
    None
}

/// Reads the constants; `Err` is the script's `FAIL setup` exit.
pub(crate) fn consts(src: &str) -> R<Consts> {
    let v = |id: &str| const_value(src, id);
    let (ch, frames, sample) = (
        first_int(v("C-04").as_deref()),
        first_int(v("C-02").as_deref()),
        first_int(v("C-03").as_deref()),
    );
    let (fb_mtu, fb_pay) = (first_int(v("C-06").as_deref()), first_int(v("C-17").as_deref()));
    let cap = rx(r"at most\s+(\d+)\s+bytes")
        .captures(&v("C-137").unwrap_or_default())
        .and_then(|c| c[1].parse::<i64>().ok());
    let (Some(ch), Some(frames), Some(sample), Some(fb_mtu), Some(fb_pay), Some(cap)) =
        (ch, frames, sample, fb_mtu, fb_pay, cap)
    else {
        return Err(Stop::Exit(
            "setup",
            "a constant the budget needs (C-02, C-03, C-04, C-06, C-17, C-137) is unreadable".into(),
        ));
    };
    let pay_fb = fb_mtu - 20 - 8;
    if pay_fb != fb_pay {
        return Err(Stop::Exit(
            "setup",
            format!("C-06 minus IPv4 and UDP headers is {pay_fb}, but C-17 states {fb_pay}"),
        ));
    }
    Ok(Consts {
        c04: v("C-04"),
        sample,
        audio: ch * frames * sample,
        fb_pay,
        cap,
        pay_def: 1500 - 20 - 8,
        pay_fb,
    })
}

/// The part sizes the toml declares (`COMP`).
#[derive(Debug, Clone)]
pub(crate) struct Comp {
    pub pkt: i64,
    pub slice: i64,
    pub ann_count: i64,
    pub entry: i64,
    pub entry_max: i64,
    pub ext: i64,
    pub cksum: i64,
    pub cw: i64,
    pub transport: i64,
    pub sp_entry: i64,
    pub sp_hdr: i64,
    pub sample_bytes: Option<i64>,
}

impl Comp {
    pub fn ann_max(&self) -> i64 {
        self.ann_count + self.entry * self.entry_max
    }

    pub fn mandatory(&self) -> i64 {
        self.pkt + self.slice + self.ann_count + self.cw + self.cksum
    }

    pub fn max_non_audio(&self) -> i64 {
        self.mandatory() + self.entry * self.entry_max + self.ext
    }

    /// `ALLOWED_ANN_BYTES`.
    pub fn allowed_ann(&self) -> Vec<i64> {
        (0..=self.entry_max).map(|k| self.ann_count + self.entry * k).collect()
    }

    /// `ALLOWED_COUNTED`.
    pub fn allowed_counted(&self) -> Vec<i64> {
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

/// `f.get(key, default)` used in arithmetic: a string raises `TypeError`.
fn geti(f: &Dict, key: &str, default: i64) -> R<i64> {
    match f.get(key) {
        None => Ok(default),
        Some(v) => v.as_int().map_or_else(|| crash("not a number"), Ok),
    }
}

/// `f["key"]`: a missing key raises `KeyError`.
fn idx(f: &Dict, key: &str) -> R<i64> {
    match f.get(key) {
        None => crash("KeyError"),
        Some(v) => v.as_int().map_or_else(|| crash("not a number"), Ok),
    }
}

/// `fbytes`: the width in bytes when it is an integer.
fn fbytes(f: Option<&Dict>) -> Option<i64> {
    f.and_then(|f| f.get("width")).and_then(V::as_int).map(|w| w.div_euclid(8))
}

fn field<'a>(m: Option<&'a Message>, name: &str) -> Option<&'a Dict> {
    m?.fields.iter().find(|f| f.text("name") == Some(name))
}

/// `msg(name)`, recording AC 5 when the message is missing.
fn msg<'a>(p: &'a Parsed, name: &str, fails: &mut Fails) -> Option<&'a Message> {
    let found = p.messages.iter().rev().find(|m| m.keys.text("name") == Some(name));
    if found.is_none() {
        fails.add("AC5", format!("wire.toml has no [[message]] named {name:?}"));
    }
    found
}

/// The pieces of `COMP` that may be absent or `None` while still "present".
#[derive(Default)]
struct Raw {
    pkt: Option<i64>,
    slice: Option<i64>,
    ann_count: Option<Option<i64>>,
    entry: Option<Option<i64>>,
    entry_max: Option<Option<i64>>,
    ext: Option<i64>,
    ext_start: Option<i64>,
    cksum: Option<Option<i64>>,
    cw: Option<Option<i64>>,
    transport: Option<Option<i64>>,
    sp_entry: Option<Option<i64>>,
    sp_hdr: Option<i64>,
    sample_bytes: Option<Option<i64>>,
}

fn size_of(m: &Message) -> Option<i64> {
    m.keys.get("max_size").and_then(V::as_int)
}

fn media_parts(mp: &Message, count: &Dict, entry: &Dict, raw: &mut Raw) -> R<()> {
    let mut hdr = 0;
    for f in &mp.fields {
        if geti(f, "offset", 0)? + geti(f, "width", 0)? <= 128 {
            hdr += idx(f, "width")?;
        }
    }
    raw.pkt = Some(hdr.div_euclid(8));
    let count_off = idx(count, "offset")?;
    let mut sl = 0;
    for f in &mp.fields {
        let off = geti(f, "offset", -1)?;
        if 128 <= off && off < count_off {
            sl += idx(f, "width")?;
        }
    }
    raw.slice = Some(sl.div_euclid(8));
    raw.ann_count = Some(fbytes(Some(count)));
    raw.entry = Some(fbytes(Some(entry)));
    raw.entry_max = Some(entry.get("max_count").and_then(V::as_int));
    let start = idx(entry, "offset")? + idx(entry, "width")? * idx(entry, "max_count")?;
    let mut ext = 0;
    for f in &mp.fields {
        if geti(f, "offset", -1)? >= start {
            ext += idx(f, "width")?;
        }
    }
    raw.ext = Some(ext.div_euclid(8));
    raw.ext_start = Some(start);
    Ok(())
}

/// Computes `COMP` and runs the toml consistency checks (AC 5).
pub(crate) fn components(p: &Parsed, c: &Consts, fails: &mut Fails) -> R<Comp> {
    let mp = msg(p, "media packet", fails);
    let count_f = field(mp, "announcement_count");
    let entry_f = field(mp, "announcement_entry");
    let ck = msg(p, "slice checksum", fails);
    let cw_m = msg(p, "content-message header", fails);
    let audio_m = msg(p, "audio message", fails);
    let transport_m = msg(p, "transport state message", fails);
    let sp_entry_m = msg(p, "sparse keyed entry", fails);
    let sparse_m = msg(p, "sparse keyed controller message", fails);
    let sparse_entry_f = field(sparse_m, "entry");
    let sample_f = field(audio_m, "sample");
    let mut raw = Raw::default();
    if let (Some(mp), Some(count), Some(entry)) = (mp, count_f, entry_f) {
        media_parts(mp, count, entry, &mut raw)?;
    }
    if let Some(ck) = ck {
        raw.cksum = Some(size_of(ck));
        if fbytes(field(Some(ck), "checksum")) != size_of(ck) {
            fails.add("AC5", "wire.toml 'slice checksum' max_size does not equal its field width");
        }
    }
    raw.cw = cw_m.map(size_of);
    raw.transport = transport_m.map(size_of);
    raw.sp_entry = sp_entry_m.map(size_of);
    if let (Some(sm), Some(sef), Some(Some(sp))) = (sparse_m, sparse_entry_f, raw.sp_entry) {
        let Some(max) = size_of(sm) else { return crash("None minus int") };
        raw.sp_hdr = Some(max - sp * geti(sef, "max_count", 0)?);
    }
    raw.sample_bytes = audio_m.and(sample_f).map(|f| fbytes(Some(f)));
    finish(raw, mp, c, fails)
}

fn finish(raw: Raw, mp: Option<&Message>, c: &Consts, fails: &mut Fails) -> R<Comp> {
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
        let why = format!("wire.toml does not declare what the budget needs: {}", missing.join(", "));
        return Err(Stop::Exit("AC5", why));
    }
    let need = |v: Option<Option<i64>>| v.flatten().map_or_else(|| crash("None in arithmetic"), Ok);
    let comp = Comp {
        pkt: raw.pkt.unwrap_or(0),
        slice: raw.slice.unwrap_or(0),
        ann_count: need(raw.ann_count)?,
        entry: need(raw.entry)?,
        entry_max: need(raw.entry_max)?,
        ext: raw.ext.unwrap_or(0),
        cksum: need(raw.cksum)?,
        cw: need(raw.cw)?,
        transport: need(raw.transport)?,
        sp_entry: need(raw.sp_entry)?,
        sp_hdr: raw.sp_hdr.unwrap_or(0),
        sample_bytes: raw.sample_bytes.flatten(),
    };
    if comp.sample_bytes != Some(c.sample) {
        let got = comp.sample_bytes.map_or("None".to_string(), |n| n.to_string());
        fails.add("AC5", format!("wire.toml audio sample is {got} bytes, C-03 says {}", c.sample));
    }
    if comp.ann_count != 1 {
        fails.add("AC5", "wire.toml announcement count is not one byte");
    }
    let start = raw.ext_start.unwrap_or(0);
    let want = comp.pkt * 8 + comp.slice * 8 + comp.ann_count * 8 + comp.entry * 8 * comp.entry_max;
    if start != want {
        fails.add("AC5", "the measurement extension does not start where the maximal container ends");
    }
    let total = comp.pkt + comp.slice + comp.ann_count + comp.entry * comp.entry_max + comp.ext;
    if mp.and_then(size_of) != Some(total) {
        fails.add("AC5", "wire.toml 'media packet' max_size is not header + slice + container max + extension");
    }
    Ok(comp)
}
