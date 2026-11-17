//! AC5 (document part): wire.toml declares the two messages, row for row
//! (offset and width in bits), read with the scripts' subset reader; and the
//! tightened DoD on a stamp section type added here (the 18-entry cap
//! threshold restated).

use toml::{Table, Value};

use super::msgs::Msg;
use super::{Merged29, Section};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_24::py::py_int;
use crate::doccheck::checks::c004_24::pytoml::{names, parse_toml, tables_strict};
use crate::doccheck::checks::c004_25::review::split_pieces_with;
use crate::doccheck::checks::c004_25::toml_side::name;
use crate::doccheck::text::{has, has_i};

fn one(msgs: &[Table], label: &str, pat: &str, md: Option<&Msg>, m: &Merged29, report: &mut Report) -> Result<(), String> {
    let cand: Vec<&Table> = msgs.iter().filter(|x| has_i("commission", &name(x)) && has_i(pat, &name(x)) && !x.contains_key("section_of")).collect();
    report.check("AC5", cand.len() == 1, format!("wire.toml declares exactly one commissioning window {label} message (found {})", cand.len()));
    let (1, Some(md)) = (cand.len(), md) else { return Ok(()) };
    let t = cand[0];
    let ok = t.get("layer").and_then(Value::as_str) == Some("control") && Some(py_int(t, "dscp", -1)?) == m.dscp && py_int(t, "max_size", -1)? == md.end;
    report.check("AC5", ok, format!("the {label} message of wire.toml is control, DSCP {:?}, with max_size the table's {} bytes", m.dscp, md.end));
    let reasons = names(t.get("rejection_reasons"))?;
    report.check("AC5", !reasons.is_empty() && reasons.iter().all(|r| m.is_control(r)), format!("the {label} message names only control reasons (found {reasons:?})"));
    let tf = tables_strict(t.get("field"))?;
    let mut agree = tf.len() == md.fl.len();
    for (g, f) in tf.iter().zip(&md.fl) {
        if py_int(g, "offset", -1)? != f.off * 8 || py_int(g, "width", -1)? != f.bits {
            agree = false;
        }
    }
    report.check("AC5", agree, format!("the {label} fields of wire.toml agree with its table row for row (offset and width in bits)"));
    Ok(())
}

/// AC5; `Err` where the script stops.
pub(crate) fn ac5(text: Option<&str>, s: &Section, m: &Merged29, ann: Option<&Msg>, clo: Option<&Msg>, report: &mut Report) -> Result<(), String> {
    let (root, ok) = match parse_toml(text.unwrap_or("")) {
        Ok(r) => (r, true),
        Err(e) => {
            report.fail("AC5", format!("wire.toml does not parse: {e}"));
            (Table::new(), false)
        }
    };
    let msgs = match root.get("message") {
        Some(Value::Array(_)) => tables_strict(root.get("message"))?,
        _ => Vec::new(),
    };
    let new_type = has(r"(?i)section type (?:code )?(?:[1-9]|[12]\d|3[01])\b", &s.text);
    let new_msg = ok && msgs.iter().any(|x| x.contains_key("section_of") && has_i("commission", &name(x)) && has_i("announc|close", &name(x)));
    if new_type || new_msg {
        let restated = split_pieces_with(&s.text, false).iter().any(|x| x.contains("18 entries"));
        report.check("DoD", restated, "a regime stamp section type added here restates the connection-cap threshold: a sentence with 18 entries per section (Open item 1, connection cap; hand-off from status.md)");
    }
    one(&msgs, "announcement", "announc", ann, m, report)?;
    one(&msgs, "close", "close", clo, m, report)
}
