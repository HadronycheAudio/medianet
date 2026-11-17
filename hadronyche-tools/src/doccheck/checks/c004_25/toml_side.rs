//! AC6 (document part): each section type is one regime stamp message of
//! wire.toml that agrees with its table row for row, with one entry message
//! per entry table, bit for bit.

use toml::{Table, Value};

use super::merged::{HDR_STAMP, Merged};
use super::tree::{Node, cdiv, entry_nodes, maxc};
use super::types::{SType, TYPE_SPECS};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_24::Stop;
use crate::doccheck::checks::c004_24::py::py_int;
use crate::doccheck::checks::c004_24::pytoml::{names, parse_toml, tables_strict};
use crate::doccheck::text::has_i;
use crate::doccheck::wire_toml::display;

/// `m.get("field", [])`, each element used as a dict.
pub(crate) fn fields(m: &Table) -> Result<Vec<Table>, String> {
    tables_strict(m.get("field"))
}

/// `int(t[key])`: `Err` where Python raises (a missing key, or `int()`).
pub(crate) fn must(t: &Table, key: &str) -> Result<i64, String> {
    if !t.contains_key(key) {
        return Err(format!("KeyError: {key}"));
    }
    py_int(t, key, 0)
}

/// The name as Python's `str(m.get("name", ""))`.
pub(crate) fn name(m: &Table) -> String {
    m.get("name").map(display).unwrap_or_default()
}

fn legacy(msgs: &[Table], report: &mut Report) -> Result<(), String> {
    let Some(m) = msgs.iter().find(|m| m.get("name").and_then(Value::as_str) == Some("regime stamp section")) else { return Ok(()) };
    let mut tf = Vec::new();
    for f in fields(m)? {
        if py_int(&f, "offset", 0)? < 464 {
            tf.push(f);
        }
    }
    let mut ok = tf.len() == 14;
    if ok {
        for (f, (o, b)) in tf.iter().zip(HDR_STAMP) {
            if must(f, "offset")? != o * 8 || must(f, "width")? != b * 8 {
                ok = false;
                break;
            }
        }
    }
    report.check("AC6", ok, "the merged regime stamp section message of wire.toml is unchanged (header fields)");
    Ok(())
}

/// The section message's body fields agree with the root table row for
/// row (offset, width, count); `Err` where the script crashes.
pub(crate) fn agree_main(m: &Table, body_rel: bool, root: &Node) -> Result<bool, String> {
    let mut tf = Vec::new();
    for f in fields(m)? {
        if py_int(&f, "offset", 0)? >= 464 {
            tf.push(f);
        }
    }
    let mut agree = tf.len() == root.rows.len();
    for (f, g) in tf.iter().zip(&root.rows) {
        let exp_off = g.start + if body_rel { 464 } else { 0 };
        if must(f, "offset")? != exp_off || must(f, "width")? != g.f.bits {
            agree = false;
        }
        let gm = maxc(&g.f);
        if gm.is_some() != f.contains_key("count") {
            agree = false;
        } else if let Some(c) = gm
            && py_int(f, "max_count", -1)? != c {
                agree = false;
            }
    }
    Ok(agree)
}

/// An entry message agrees with its entry table bit for bit and tiles to
/// its size; `Err` where the script crashes.
pub(crate) fn agree_entry(em: &Table, n: &Node) -> Result<bool, String> {
    let efs = fields(em)?;
    let mut ok = efs.len() == n.rows.len() && py_int(em, "max_size", -1)? == cdiv(n.size, 8);
    for (r, g) in n.rows.iter().zip(&efs) {
        if py_int(g, "offset", -1)? != r.start || py_int(g, "width", -1)? != r.f.bits {
            ok = false;
        }
        let fm = maxc(&r.f);
        if fm.is_some() != g.contains_key("count") {
            ok = false;
        } else if let Some(c) = fm
            && py_int(g, "max_count", -1)? != c {
                ok = false;
            }
    }
    let mut ends = Vec::new();
    for g in &efs {
        let mult = if g.contains_key("count") { py_int(g, "max_count", 1)? } else { 1 };
        ends.push(must(g, "offset")? + must(g, "width")? * mult);
    }
    if ends.last().is_some_and(|e| *e != n.size) {
        ok = false;
    }
    Ok(ok)
}

fn one_type(msgs: &[Table], t: &SType, merged: &Merged, dscp: Option<i64>, report: &mut Report) -> Result<(), String> {
    let pat = t.pat;
    let cand: Vec<&Table> = msgs
        .iter()
        .filter(|m| has_i(pat, &name(m)) && m.get("section_of").and_then(Value::as_str) == Some("regime stamp"))
        .collect();
    report.check("AC6", cand.len() == 1, format!("wire.toml declares exactly one regime stamp message for the {} section type (found {})", t.key, cand.len()));
    let (Some(root), 1) = (t.root.as_ref(), cand.len()) else { return Ok(()) };
    let m = cand[0];
    let ok = m.get("layer").and_then(Value::as_str) == Some("control") && Some(py_int(m, "dscp", -1)?) == dscp && Some(py_int(m, "max_size", -1)?) == t.sec_max;
    report.check("AC6", ok, format!("the {} message of wire.toml is control, DSCP {dscp:?} (C-13), and its max_size is the table's {:?} bytes", t.key, t.sec_max));
    let reasons = names(m.get("rejection_reasons"))?;
    report.check("AC6", reasons.iter().all(|r| merged.is_control(r)), format!("the {} message names only control reasons (found {reasons:?})", t.key));
    let agree = agree_main(m, t.body_rel, root)?;
    report.check("AC6", agree, format!("the {} fields of wire.toml agree with its table row for row (offset, width, count)", t.key));
    let ent_msgs: Vec<&Table> = msgs
        .iter()
        .filter(|x| has_i(pat, &name(x)) && name(x).to_lowercase().contains("entry") && !x.contains_key("section_of"))
        .collect();
    let ents = entry_nodes(root);
    report.check("AC6", ent_msgs.len() == ents.len(), format!("wire.toml declares one entry message per entry table of the {} section (tables {}, messages {})", t.key, ents.len(), ent_msgs.len()));
    for (n, em) in ents.iter().zip(&ent_msgs) {
        let ok = agree_entry(em, n)?;
        report.check("AC6", ok, format!("the entry table of {} agrees with wire.toml row for row (offset, width, count) and tiles to its size ({} bits)", name(em), n.size));
    }
    Ok(())
}

/// AC6 (document part).
pub(crate) fn ac6(toml_text: Option<&str>, types: &[SType], merged: &Merged, dscp: Option<i64>, report: &mut Report) -> Result<(), Stop> {
    let root = match parse_toml(toml_text.unwrap_or("")) {
        Ok(t) => t,
        Err(e) => {
            report.fail("AC6", format!("wire.toml does not parse: {e}"));
            Table::new()
        }
    };
    let mut run = || -> Result<(), String> {
        // `msgs = TOML.get("message", [])`, kept only when it is a list; each
        // element is then used as a dict.
        let msgs = match root.get("message") {
            Some(Value::Array(_)) => tables_strict(root.get("message"))?,
            _ => Vec::new(),
        };
        legacy(&msgs, report)?;
        for (key, _) in TYPE_SPECS {
            if let Some(t) = types.iter().find(|t| t.key == key) {
                one_type(&msgs, t, merged, dscp, report)?;
            }
        }
        Ok(())
    };
    run().map_err(Stop)
}
