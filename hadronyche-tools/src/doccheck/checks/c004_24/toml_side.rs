//! AC5 (document part): the stamp messages of wire.toml and their agreement
//! with the Markdown tables; their rejection reasons (reported under AC4,
//! as the script does). The values are used as Python uses them; an
//! exception the script does not catch here (a non-iterable value, `int()`
//! of a non-number, an unhashable reason) stops the run as a crash.

use std::collections::BTreeSet;

use super::hello::Landed;
use super::stamp::Stamp;
use crate::doccheck::checked;
use crate::doccheck::Report;
use crate::doccheck::family004::Field;
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::report::{Flow, crash};

type Sig = Vec<(i128, i128, i128)>;

fn name(m: &Dict) -> String {
    m.get("name").map_or("None".to_string(), PyValue::py_str)
}

/// `[x for x in d.get(key, []) if isinstance(x, dict)]`.
fn dicts<'a>(d: &'a Dict, key: &str) -> Flow<Vec<&'a Dict>> {
    match d.get(key) {
        None => Ok(Vec::new()),
        Some(v) => v.py_dicts().or_else(|e| crash(format!("TypeError: {e}"))),
    }
}

/// `int(d.get(key, default))`.
fn int_or(d: &Dict, key: &str, default: i128) -> Flow<i128> {
    match d.get(key) {
        None => Ok(default),
        Some(v) => v.py_int().or_else(|e| crash(format!("ValueError: {e}"))),
    }
}

/// The items of `d.get(key, [])` used as dict keys or set members
/// (`r not in MEMBERS`, `set(...)`): `str()` of each, a `TypeError` for an
/// unhashable item (a list or a dict) or a non-iterable value.
fn hashable(d: &Dict, key: &str) -> Flow<Vec<String>> {
    match d.get(key) {
        None => Ok(Vec::new()),
        Some(PyValue::List(items)) => items
            .iter()
            .map(|x| match x {
                PyValue::List(_) | PyValue::Dict(_) => crash(format!("TypeError: unhashable type: '{}'", x.type_name())),
                other => Ok(other.py_str()),
            })
            .collect(),
        Some(v) => v.py_iter_str().or_else(|e| crash(format!("TypeError: {e}"))),
    }
}

/// `toml_sig`.
fn toml_sig(m: &Dict) -> Flow<Sig> {
    dicts(m, "field")?
        .iter()
        .map(|f| Ok((int_or(f, "offset", -1)?, int_or(f, "width", 0)?, int_or(f, "max_count", 1)?)))
        .collect()
}

fn md_sig(fl: &[Field]) -> Sig {
    fl.iter().map(|f| (checked::mul(f.off, 8), f.bits, f.maxc.filter(|&m| m != 0).unwrap_or(1))).collect()
}

/// `isinstance(ms, int) and ms <= CTRL_BOUND and not probs and cur == ms * 8`.
fn tiles(ms: Option<&PyValue>, cur: i128, probs: &[String], landed: &Landed) -> Flow<bool> {
    let Some(v) = ms.filter(|v| v.is_int()) else { return Ok(false) };
    let Some(b) = landed.ctrl_bound else {
        return crash("TypeError: '<=' not supported between instances of 'int' and 'NoneType'");
    };
    // a value beyond i128 is never <= the bound when positive, and never
    // tiles when negative: no comparison of it decides the verdict
    let Ok(s) = (match v {
        PyValue::BigInt(_) => Err(()),
        other => other.py_int().map_err(|_| ()),
    }) else {
        return Ok(false);
    };
    Ok(s <= b && probs.is_empty() && cur == checked::mul(s, 8))
}

/// `check_toml_message`.
fn one_message(m: &Dict, md_fl: &[Field], md_total: i128, landed: &Landed, report: &mut Report) -> Flow<bool> {
    let name = name(m);
    let (mut cur, mut probs) = (0i128, Vec::new());
    for f in dicts(m, "field")? {
        let mult = int_or(f, "max_count", 1)?;
        let off = int_or(f, "offset", -1)?;
        if off != cur {
            probs.push(format!("'{}' at bit {off}, expected {cur}", f.get("name").map_or("None".to_string(), PyValue::py_str)));
        }
        cur = checked::add(int_or(f, "offset", 0)?, checked::mul(int_or(f, "width", 0)?, mult));
    }
    let ms = m.get("max_size");
    let ok = tiles(ms, cur, &probs, landed)?;
    report.check("AC5", ok, format!("toml '{name}' tiles from bit 0 to max_size, and max_size is at most the control bound ({cur} bits): {probs:?}"));
    let dscp_ok = match (m.get("dscp"), landed.dscp) {
        (None, None) => true,
        (Some(v), Some(d)) => v.eq_int(d),
        _ => false,
    };
    report.check("AC5", m.get("layer").is_some_and(|v| v.eq_str("control")) && dscp_ok, format!("toml '{name}' is layer control at dscp {:?} (C-13)", landed.dscp));
    report.check("AC5", !m.contains_key("exempt_from_control_bound"), format!("toml '{name}' carries no exemption from the control bound (C-116 is the probe pair's alone)"));
    let rs = hashable(m, "rejection_reasons")?;
    let unknown: Vec<&String> = rs.iter().filter(|r| !landed.is_member(r)).collect();
    let ok = unknown.is_empty() && rs.iter().all(|r| landed.layer(r).contains("control"));
    report.check("AC4", ok, format!("toml '{name}' rejection_reasons are members whose Layers include control; not members: {unknown:?}"));
    let rows_ok = !md_fl.is_empty() && md_sig(md_fl) == toml_sig(m)? && ms.is_some_and(|v| v.eq_int(md_total));
    report.check("AC5", rows_ok, format!("the markdown table and toml '{name}' agree row for row in offset, width and count, and in size"));
    Ok(rows_ok)
}

/// `_md_matches(m, fl)`.
fn matches(m: &Dict, fl: &[Field]) -> Flow<bool> {
    if fl.is_empty() {
        return Ok(false);
    }
    Ok(md_sig(fl) == toml_sig(m)?)
}

/// AC5 (document part).
pub(crate) fn ac5(toml: &Dict, st: &Stamp, landed: &Landed, reasons_md: &BTreeSet<String>, report: &mut Report) -> Flow<()> {
    let msgs = dicts(toml, "message")?;
    let truthy = |m: &Dict| m.get("section_of").is_some_and(PyValue::truthy);
    let mut sec = Vec::new();
    for m in &msgs {
        if truthy(m) && matches(m, &st.sec_fl)? {
            sec.push(*m);
        }
    }
    let mut req = Vec::new();
    for m in &msgs {
        if !truthy(m) && matches(m, &st.req_fl)? {
            req.push(*m);
        }
    }
    report.check("AC5", !sec.is_empty(), "a wire.toml message with section_of (the 004-02 attribute) carries the markdown section table row for row");
    report.check("AC5", !req.is_empty(), "a wire.toml message without section_of carries the markdown request table row for row");
    for m in &sec {
        let ok = m.get("section_of").and_then(PyValue::as_str).is_some_and(|s| !crate::doccheck::pycompat::PyStr::py_strip(s).is_empty());
        report.check("AC5", ok, format!("the section_of of '{}' names its parent (a non-empty name; the parent need not be declared)", name(m)));
    }
    let mut agree = false;
    for m in &sec {
        agree |= one_message(m, &st.sec_fl, st.sec_tile, landed, report)?;
    }
    if !sec.is_empty() {
        report.check("AC5", agree, "a stamp section message of wire.toml is the markdown section table");
    }
    let mut toml_reasons = BTreeSet::new();
    for m in sec.iter().chain(req.iter()) {
        toml_reasons.extend(hashable(m, "rejection_reasons")?);
    }
    for m in &req {
        one_message(m, &st.req_fl, st.req_tile, landed, report)?;
    }
    let missing: Vec<&String> = reasons_md.difference(&toml_reasons).collect();
    report.check("AC4", !reasons_md.is_empty() && missing.is_empty(), format!("every reason the stamp's validity table names is in the rejection_reasons of a stamp message of wire.toml (missing {missing:?})"));
    Ok(())
}
