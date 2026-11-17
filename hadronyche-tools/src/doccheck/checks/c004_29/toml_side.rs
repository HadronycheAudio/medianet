//! AC5 (document part): wire.toml declares the two messages, row for row
//! (offset and width in bits), read with the scripts' subset reader (any
//! exception of the reader is an AC5 failure); and the tightened DoD on a
//! stamp section type added here (the 18-entry cap threshold restated).
//! Values are used as the script uses them ([`PyValue`]); the exceptions it
//! does not catch here (`in` or `.get` on a value that is not a dict,
//! `int()` of a non-number, an unhashable reason) stop the run as a crash.

use super::msgs::Msg;
use super::{Merged29, Section};
use crate::doccheck::Report;
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::sentences::{SentenceSplit, split};
use crate::doccheck::text::{has, has_i};
use crate::doccheck::tomlsubset;

/// `str(m.get("name", ""))` of a message (a dict here).
fn name(m: &Dict) -> String {
    m.get("name").map(PyValue::py_str).unwrap_or_default()
}

fn no_get(x: &PyValue) -> Flow<&Dict> {
    x.as_dict().map_or_else(|| crash(format!("AttributeError: '{}' object has no attribute 'get'", x.type_name())), Ok)
}

/// `int(d.get(key, default))`.
fn int_or(d: &Dict, key: &str, default: i128) -> Flow<i128> {
    match d.get(key) {
        None => Ok(default),
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

/// `"section_of" in m and re.search(r"commission", name) and
/// re.search(r"announc|close", name)` for any item of the message list.
fn stamp_message(m: &PyValue) -> Flow<bool> {
    let has_key = match m {
        PyValue::Dict(d) => d.contains_key("section_of"),
        PyValue::Str(s) => s.contains("section_of"),
        PyValue::List(xs) => xs.iter().any(|x| x.eq_str("section_of")),
        other => return crash(format!("TypeError: argument of type '{}' is not iterable", other.type_name())),
    };
    if !has_key {
        return Ok(false);
    }
    let d = no_get(m)?;
    Ok(has_i("commission", &name(d)) && has_i("announc|close", &name(d)))
}

/// The rows `zip(m.get("field", []), md["fl"])` reads: the first `n` items
/// of the field list, each used with `.get`; with `len()` of the list.
fn fields(m: &Dict, n: usize) -> Flow<(usize, Vec<&Dict>)> {
    let items: Vec<&PyValue> = match m.get("field") {
        None => Vec::new(),
        Some(PyValue::List(xs)) => xs.iter().collect(),
        Some(PyValue::Dict(d)) => {
            // iterating a dict gives its keys, strings
            if d.is_empty() || n == 0 {
                return Ok((d.len(), Vec::new()));
            }
            return crash("AttributeError: 'str' object has no attribute 'get'");
        }
        Some(PyValue::Str(s)) => {
            let len = s.chars().count();
            if len == 0 || n == 0 {
                return Ok((len, Vec::new()));
            }
            return crash("AttributeError: 'str' object has no attribute 'get'");
        }
        Some(other) => return crash(format!("TypeError: object of type '{}' has no len()", other.type_name())),
    };
    let len = items.len();
    let used = items.into_iter().take(n).map(no_get).collect::<Flow<Vec<&Dict>>>()?;
    Ok((len, used))
}

fn one(msgs: &[&Dict], label: &str, pat: &str, md: Option<&Msg>, m: &Merged29, report: &mut Report) -> Flow<()> {
    let cand: Vec<&&Dict> = msgs.iter().filter(|x| has_i("commission", &name(x)) && has_i(pat, &name(x)) && !x.contains_key("section_of")).collect();
    report.check("AC5", cand.len() == 1, format!("wire.toml declares exactly one commissioning window {label} message (found {})", cand.len()));
    let (1, Some(md)) = (cand.len(), md) else { return Ok(()) };
    let t = cand[0];
    let ok = t.get("layer").is_some_and(|v| v.eq_str("control")) && Some(int_or(t, "dscp", -1)?) == m.dscp && int_or(t, "max_size", -1)? == md.end;
    report.check("AC5", ok, format!("the {label} message of wire.toml is control, DSCP {:?}, with max_size the table's {} bytes", m.dscp, md.end));
    let reasons = hashable(t, "rejection_reasons")?;
    report.check("AC5", !reasons.is_empty() && reasons.iter().all(|r| m.is_control(r)), format!("the {label} message names only control reasons (found {reasons:?})"));
    let (len, tf) = fields(t, md.fl.len())?;
    let mut agree = len == md.fl.len();
    for (g, f) in tf.iter().zip(&md.fl) {
        if int_or(g, "offset", -1)? != f.off.saturating_mul(8) || int_or(g, "width", -1)? != f.bits {
            agree = false;
        }
    }
    report.check("AC5", agree, format!("the {label} fields of wire.toml agree with its table row for row (offset and width in bits)"));
    Ok(())
}

/// AC5 and the tightened DoD.
pub(crate) fn ac5(text: Option<&str>, s: &Section, m: &Merged29, ann: Option<&Msg>, clo: Option<&Msg>, report: &mut Report) -> Flow<()> {
    let root = match tomlsubset::parse(text.unwrap_or("")) {
        Ok(r) => Some(r),
        Err(e) => {
            report.fail("AC5", format!("wire.toml does not parse: {e}"));
            None
        }
    };
    // `msgs = msgs if isinstance(msgs, list) else []`
    let raw: &[PyValue] = match root.as_ref().and_then(|r| r.get("message")) {
        Some(PyValue::List(xs)) => xs,
        _ => &[],
    };
    let new_type = has(r"(?i)section type (?:code )?(?:[1-9]|[12]\d|3[01])\b", &s.text);
    let mut new_msg = false;
    for x in raw {
        new_msg |= stamp_message(x)?;
    }
    if new_type || new_msg {
        let restated = split(&s.text, SentenceSplit::PARAGRAPHS_004_29).iter().any(|x| x.contains("18 entries"));
        report.check("DoD", restated, "a regime stamp section type added here restates the connection-cap threshold: a sentence with 18 entries per section (Open item 1, connection cap; hand-off from status.md)");
    }
    // the candidate filters call `.get` on every item
    let msgs = raw.iter().map(no_get).collect::<Flow<Vec<&Dict>>>()?;
    one(&msgs, "announcement", "announc", ann, m, report)?;
    one(&msgs, "close", "close", clo, m, report)
}
