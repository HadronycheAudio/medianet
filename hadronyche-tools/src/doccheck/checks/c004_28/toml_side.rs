//! AC5 (Work 4, document part): wire.toml declares the two section types
//! and their entry tables, row for row, read with the scripts' subset
//! reader. 004-28's own copy of the comparisons it shares with 004-25 (no
//! port imports another); 004-28 also requires a non-empty reason list,
//! and catches every exception of the reader as an AC5 failure. Values are
//! used as the script uses them ([`PyValue`]); the exceptions it does not
//! catch here (`.get` on a non-dict item, `int()` of a non-number, a
//! missing key, an unhashable reason) stop the run as a crash.

use super::budget::Types;
use super::merged::Merged;
use super::tree::{Node, cdiv, entry_nodes, maxc};
use crate::doccheck::checked;
use crate::doccheck::Report;
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::has_i;
use crate::doccheck::tomlsubset;

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
        Some(PyValue::Str(_) | PyValue::Dict(_)) => crash("AttributeError: 'str' object has no attribute 'get'"),
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

fn one(msgs: &[&Dict], key: &'static str, t: &Types, m: &Merged, dscp: Option<i128>, report: &mut Report) -> Flow<()> {
    let cand: Vec<&&Dict> = msgs.iter().filter(|x| has_i(key, &name(x)) && x.get("section_of").is_some_and(|v| v.eq_str("regime stamp"))).collect();
    report.check("AC5", cand.len() == 1, format!("wire.toml declares exactly one regime stamp message for the {key} section type (found {})", cand.len()));
    let Some(a) = t.new.get(key).filter(|a| a.ok) else { return Ok(()) };
    let (1, Some(root)) = (cand.len(), a.root.as_ref()) else { return Ok(()) };
    let msg = cand[0];
    let ok = msg.get("layer").is_some_and(|v| v.eq_str("control")) && Some(int_or(msg, "dscp", -1)?) == dscp && Some(int_or(msg, "max_size", -1)?) == a.sec_max;
    report.check("AC5", ok, format!("the {key} message of wire.toml is control, DSCP {dscp:?}, and its max_size is the table's {:?} bytes", a.sec_max));
    let reasons = hashable(msg, "rejection_reasons")?;
    report.check("AC5", !reasons.is_empty() && reasons.iter().all(|r| m.is_control(r)), format!("the {key} message names only control reasons (found {reasons:?})"));
    let agree = agree_main(msg, a.body_rel, root)?;
    report.check("AC5", agree, format!("the {key} fields of wire.toml agree with its table row for row (offset, width, count)"));
    let ent_msgs: Vec<&&Dict> = msgs.iter().filter(|x| has_i(key, &name(x)) && name(x).to_lowercase().contains("entry") && !x.contains_key("section_of")).collect();
    let ents = entry_nodes(root);
    report.check("AC5", ent_msgs.len() == ents.len(), format!("wire.toml declares one entry message per entry table of the {key} section (tables {}, messages {})", ents.len(), ent_msgs.len()));
    for (n, em) in ents.iter().zip(&ent_msgs) {
        let ok = agree_entry(em, n)?;
        report.check("AC5", ok, format!("the entry table of {} agrees with wire.toml row for row and tiles to its size ({} bits)", name(em), n.size));
    }
    Ok(())
}

/// AC5 (document part): `TOML = parse_toml(TOML_TEXT)`, any exception of
/// the reader caught as an AC5 failure.
pub(crate) fn ac5(text: Option<&str>, t: &Types, m: &Merged, dscp: Option<i128>, report: &mut Report) -> Flow<()> {
    let root = match tomlsubset::parse(text.unwrap_or("")) {
        Ok(r) => Some(r),
        Err(e) => {
            report.fail("AC5", format!("wire.toml does not parse: {e}"));
            None
        }
    };
    // `msgs = msgs if isinstance(msgs, list) else []`; every element is then
    // used with `.get`
    let msgs: Vec<&Dict> = match root.as_ref().and_then(|r| r.get("message")) {
        Some(PyValue::List(_)) => items(root.as_ref().expect("present"), "message")?,
        _ => Vec::new(),
    };
    for key in ["dscp", "alignment"] {
        one(&msgs, key, t, m, dscp, report)?;
    }
    Ok(())
}
