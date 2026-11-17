//! AC5 (document part): the stamp messages of wire.toml and their agreement
//! with the Markdown tables; their rejection reasons (reported under AC4,
//! as the script does). wire.toml is read with the scripts' own subset
//! reader ([`super::pytoml`]), and its values compared as Python compares
//! them.

use std::collections::BTreeSet;

use toml::{Table, Value};

use super::Stop;
use super::landed::Landed;
use super::py::{py_int, truthy};
use super::pytoml::{as_int, dicts, eq_int, names, parse_toml};
use super::stamp::Stamp;
use crate::doccheck::Report;
use crate::doccheck::checks::c004_27::Field;
use crate::doccheck::wire_toml::display;

type Sig = Vec<(i64, i64, i64)>;

fn name(m: &Table) -> String {
    m.get("name").map(display).unwrap_or_else(|| "None".into())
}

/// `toml_fields`: the dict elements of `m.get("field", [])`.
fn fields(m: &Table) -> Vec<Table> {
    dicts(m.get("field"))
}

/// `toml_sig`; `Err` where Python's `int()` raises.
fn toml_sig(m: &Table) -> Result<Sig, String> {
    fields(m)
        .iter()
        .map(|f| Ok((py_int(f, "offset", -1)?, py_int(f, "width", 0)?, py_int(f, "max_count", 1)?)))
        .collect()
}

fn md_sig(fl: &[Field]) -> Sig {
    fl.iter().map(|f| (f.off * 8, f.bits, f.maxc.filter(|&m| m != 0).unwrap_or(1))).collect()
}

/// `check_toml_message`; `Err` where the script would crash.
fn one_message(m: &Table, md_fl: &[Field], md_total: i64, landed: &Landed, report: &mut Report) -> Result<bool, String> {
    let name = name(m);
    let ms = as_int(m.get("max_size"));
    let (mut cur, mut probs) = (0, Vec::new());
    for f in fields(m) {
        let mult = py_int(&f, "max_count", 1)?;
        let off = py_int(&f, "offset", -1)?;
        if off != cur {
            probs.push(format!("'{}' at bit {off}, expected {cur}", f.get("name").map(display).unwrap_or_default()));
        }
        cur = py_int(&f, "offset", 0)? + py_int(&f, "width", 0)? * mult;
    }
    let within = match (ms, landed.ctrl_bound) {
        (Some(s), Some(b)) => s <= b,
        (Some(_), None) => return Err("TypeError: int <= None".into()),
        _ => false,
    };
    report.check("AC5", within && probs.is_empty() && Some(cur) == ms.map(|s| s * 8), format!("toml '{name}' tiles from bit 0 to max_size, and max_size is at most the control bound ({cur} bits against {ms:?} bytes): {probs:?}"));
    let layer = m.get("layer").and_then(Value::as_str) == Some("control");
    report.check("AC5", layer && eq_int(m.get("dscp"), landed.dscp), format!("toml '{name}' is layer control at dscp {:?} (C-13)", landed.dscp));
    report.check("AC5", !m.contains_key("exempt_from_control_bound"), format!("toml '{name}' carries no exemption from the control bound (C-116 is the probe pair's alone)"));
    let rs = names(m.get("rejection_reasons"))?;
    let unknown: Vec<&String> = rs.iter().filter(|r| !landed.members.contains_key(*r)).collect();
    let ok = unknown.is_empty() && rs.iter().all(|r| landed.layer(r).contains("control"));
    report.check("AC4", ok, format!("toml '{name}' rejection_reasons are members whose Layers include control; not members: {unknown:?}"));
    let rows_ok = !md_fl.is_empty() && Ok(md_sig(md_fl)) == toml_sig(m) && eq_int(m.get("max_size"), Some(md_total));
    report.check("AC5", rows_ok, format!("the markdown table and toml '{name}' agree row for row in offset, width and count, and in size"));
    Ok(rows_ok)
}

fn matches(m: &Table, fl: &[Field]) -> Result<bool, String> {
    if fl.is_empty() {
        return Ok(false);
    }
    Ok(md_sig(fl) == toml_sig(m)?)
}

fn messages(st: &Stamp, landed: &Landed, msgs: &[Table], reasons_md: &BTreeSet<String>, report: &mut Report) -> Result<(), String> {
    let mut sec = Vec::new();
    let mut req = Vec::new();
    for m in msgs {
        if truthy(m.get("section_of")) && matches(m, &st.sec_fl)? {
            sec.push(m);
        }
    }
    for m in msgs {
        if !truthy(m.get("section_of")) && matches(m, &st.req_fl)? {
            req.push(m);
        }
    }
    report.check("AC5", !sec.is_empty(), "a wire.toml message with section_of (the 004-02 attribute) carries the markdown section table row for row");
    report.check("AC5", !req.is_empty(), "a wire.toml message without section_of carries the markdown request table row for row");
    for m in &sec {
        let ok = m.get("section_of").and_then(Value::as_str).is_some_and(|s| !s.trim().is_empty());
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
        toml_reasons.extend(names(m.get("rejection_reasons"))?);
    }
    for m in &req {
        one_message(m, &st.req_fl, st.req_tile, landed, report)?;
    }
    let missing: Vec<&String> = reasons_md.difference(&toml_reasons).collect();
    report.check("AC4", !reasons_md.is_empty() && missing.is_empty(), format!("every reason the stamp's validity table names is in the rejection_reasons of a stamp message of wire.toml (missing {missing:?})"));
    Ok(())
}

/// AC5 (document part).
pub(crate) fn ac5(toml_text: Option<&str>, st: &Stamp, landed: &Landed, reasons_md: &BTreeSet<String>, report: &mut Report) -> Result<(), Stop> {
    let root = match parse_toml(toml_text.unwrap_or("")) {
        Ok(t) => t,
        Err(e) => {
            report.fail("AC5", format!("wire.toml does not parse with the subset reader: {e}"));
            Table::new()
        }
    };
    super::pytoml::py_iter(root.get("message")).map_err(Stop)?;
    let msgs = dicts(root.get("message"));
    messages(st, landed, &msgs, reasons_md, report).map_err(Stop)
}
