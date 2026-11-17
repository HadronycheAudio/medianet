//! AC6 (document part): each section type is one regime stamp message of
//! wire.toml that agrees with its table row for row, with one entry message
//! per entry table, bit for bit. Values are used as the script uses them
//! ([`PyValue`]); the exceptions it does not catch here (`.get` on a
//! non-dict item, `int()` of a non-number, a missing key, an unhashable
//! reason) stop the run as a crash.

use super::merged::Merged;
use super::tree::{Node, cdiv, entry_nodes, maxc};
use super::types::{SType, TYPE_SPECS};
use crate::doccheck::checked;
use crate::doccheck::Report;
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::has_i;

/// `str(m.get("name", ""))`.
fn name(m: &Dict) -> String {
    m.get("name").map(PyValue::py_str).unwrap_or_default()
}

/// The items of `d.get(key, [])` each used with `.get` (an item that is not
/// a dict raises).
fn items<'a>(d: &'a Dict, key: &str) -> Flow<Vec<&'a Dict>> {
    match d.get(key) {
        None => Ok(Vec::new()),
        Some(PyValue::List(xs)) => xs
            .iter()
            .map(|x| x.as_dict().map_or_else(|| crash(format!("AttributeError: '{}' object has no attribute 'get'", x.type_name())), Ok))
            .collect(),
        Some(PyValue::Str(s)) if s.is_empty() => Ok(Vec::new()),
        Some(PyValue::Dict(x)) if x.is_empty() => Ok(Vec::new()),
        Some(PyValue::Str(_)) => crash("AttributeError: 'str' object has no attribute 'get'"),
        Some(PyValue::Dict(_)) => crash("AttributeError: 'str' object has no attribute 'get'"),
        Some(other) => crash(format!("TypeError: '{}' object is not iterable", other.type_name())),
    }
}

/// `int(d.get(key, default))`.
fn int_or(d: &Dict, key: &str, default: i128) -> Flow<i128> {
    match d.get(key) {
        None => Ok(default),
        Some(v) => v.py_int().or_else(|e| crash(format!("ValueError: {e}"))),
    }
}

/// `int(d[key])`.
fn must(d: &Dict, key: &str) -> Flow<i128> {
    match d.get(key) {
        None => crash(format!("KeyError: '{key}'")),
        Some(v) => v.py_int().or_else(|e| crash(format!("ValueError: {e}"))),
    }
}

/// `set(d.get(key, []))` as `str()`s; an unhashable item raises.
fn hashable(d: &Dict, key: &str) -> Flow<Vec<String>> {
    match d.get(key) {
        None => Ok(Vec::new()),
        Some(PyValue::List(xs)) => xs
            .iter()
            .map(|x| match x {
                PyValue::List(_) | PyValue::Dict(_) => crash(format!("TypeError: unhashable type: '{}'", x.type_name())),
                other => Ok(other.py_str()),
            })
            .collect(),
        Some(v) => v.py_iter_str().or_else(|e| crash(format!("TypeError: {e}"))),
    }
}

fn legacy(msgs: &[&Dict], report: &mut Report) -> Flow<()> {
    const HDR: [(i128, i128); 14] = [(0, 2), (2, 2), (4, 2), (6, 1), (7, 1), (8, 8), (16, 16), (32, 8), (40, 8), (48, 4), (52, 1), (53, 1), (54, 2), (56, 2)];
    let Some(m) = msgs.iter().find(|m| m.get("name").is_some_and(|v| v.eq_str("regime stamp section"))) else { return Ok(()) };
    let mut tf = Vec::new();
    for f in items(m, "field")? {
        if int_or(f, "offset", 0)? < 464 {
            tf.push(f);
        }
    }
    let mut ok = tf.len() == 14;
    if ok {
        for (f, (o, b)) in tf.iter().zip(HDR) {
            if must(f, "offset")? != o * 8 || must(f, "width")? != b * 8 {
                ok = false;
                break;
            }
        }
    }
    report.check("AC6", ok, "the merged regime stamp section message of wire.toml is unchanged (header fields)");
    Ok(())
}

/// `bool(maxc) != ("count" in f) or (maxc and int(f.get("max_count", -1)) != maxc)`.
fn count_differs(f: &Dict, m: Option<i128>) -> Flow<bool> {
    if m.is_some() != f.contains_key("count") {
        return Ok(true);
    }
    match m {
        Some(c) => Ok(int_or(f, "max_count", -1)? != c),
        None => Ok(false),
    }
}

/// The section message's body fields agree with the root table row for row.
fn agree_main(m: &Dict, body_rel: bool, root: &Node) -> Flow<bool> {
    let mut tf = Vec::new();
    for f in items(m, "field")? {
        if int_or(f, "offset", 0)? >= 464 {
            tf.push(f);
        }
    }
    let mut agree = tf.len() == root.rows.len();
    for (f, g) in tf.iter().zip(&root.rows) {
        let exp_off = checked::add(g.start, if body_rel { 464 } else { 0 });
        if must(f, "offset")? != exp_off || must(f, "width")? != g.f.bits {
            agree = false;
        }
        if count_differs(f, maxc(&g.f))? {
            agree = false;
        }
    }
    Ok(agree)
}

/// An entry message agrees with its entry table bit for bit and tiles to
/// its size.
fn agree_entry(em: &Dict, n: &Node) -> Flow<bool> {
    let efs = items(em, "field")?;
    let mut ok = efs.len() == n.rows.len() && int_or(em, "max_size", -1)? == cdiv(n.size, 8);
    for (r, g) in n.rows.iter().zip(&efs) {
        if int_or(g, "offset", -1)? != r.start || int_or(g, "width", -1)? != r.f.bits {
            ok = false;
        }
        if count_differs(g, maxc(&r.f))? {
            ok = false;
        }
    }
    let mut ends = Vec::new();
    for g in &efs {
        let mult = if g.contains_key("count") { int_or(g, "max_count", 1)? } else { 1 };
        ends.push(checked::add(must(g, "offset")?, checked::mul(must(g, "width")?, mult)));
    }
    if ends.last().is_some_and(|e| *e != n.size) {
        ok = false;
    }
    Ok(ok)
}

fn one_type(msgs: &[&Dict], t: &SType, merged: &Merged, dscp: Option<i128>, report: &mut Report) -> Flow<()> {
    let cand: Vec<&&Dict> = msgs
        .iter()
        .filter(|m| has_i(t.pat, &name(m)) && m.get("section_of").is_some_and(|v| v.eq_str("regime stamp")))
        .collect();
    report.check("AC6", cand.len() == 1, format!("wire.toml declares exactly one regime stamp message for the {} section type (found {})", t.key, cand.len()));
    let (Some(root), 1) = (t.root.as_ref(), cand.len()) else { return Ok(()) };
    let m = cand[0];
    let ok = m.get("layer").is_some_and(|v| v.eq_str("control")) && Some(int_or(m, "dscp", -1)?) == dscp && Some(int_or(m, "max_size", -1)?) == t.sec_max;
    report.check("AC6", ok, format!("the {} message of wire.toml is control, DSCP {dscp:?} (C-13), and its max_size is the table's {:?} bytes", t.key, t.sec_max));
    let reasons = hashable(m, "rejection_reasons")?;
    report.check("AC6", reasons.iter().all(|r| merged.is_control(r)), format!("the {} message names only control reasons (found {reasons:?})", t.key));
    let agree = agree_main(m, t.body_rel, root)?;
    report.check("AC6", agree, format!("the {} fields of wire.toml agree with its table row for row (offset, width, count)", t.key));
    let ent_msgs: Vec<&&Dict> = msgs
        .iter()
        .filter(|x| has_i(t.pat, &name(x)) && name(x).to_lowercase().contains("entry") && !x.contains_key("section_of"))
        .collect();
    let ents = entry_nodes(root);
    report.check("AC6", ent_msgs.len() == ents.len(), format!("wire.toml declares one entry message per entry table of the {} section (tables {}, messages {})", t.key, ents.len(), ent_msgs.len()));
    for (n, em) in ents.iter().zip(&ent_msgs) {
        let ok = agree_entry(em, n)?;
        report.check("AC6", ok, format!("the entry table of {} agrees with wire.toml row for row (offset, width, count) and tiles to its size ({} bits)", name(em), n.size));
    }
    Ok(())
}

/// AC6 (document part): `toml` is `None` when it did not parse.
pub(crate) fn ac6(toml: Option<&Dict>, types: &[SType], merged: &Merged, dscp: Option<i128>, report: &mut Report) -> Flow<()> {
    // `msgs = TOML.get("message", [])`, kept only when it is a list; every
    // element is then used with `.get`
    let msgs: Vec<&Dict> = match toml.and_then(|t| t.get("message")) {
        Some(PyValue::List(_)) => items(toml.expect("present"), "message")?,
        _ => Vec::new(),
    };
    legacy(&msgs, report)?;
    for (key, _) in TYPE_SPECS {
        if let Some(t) = types.iter().find(|t| t.key == key) {
            one_type(&msgs, t, merged, dscp, report)?;
        }
    }
    Ok(())
}
