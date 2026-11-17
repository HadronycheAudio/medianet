//! AC5 (Work 4, document part): wire.toml declares the two section types
//! and their entry tables, row for row, read with the scripts' subset
//! reader (the comparisons are 004-25's,
//! [`crate::doccheck::checks::c004_25::toml_side`]; 004-28 also requires a
//! non-empty reason list).

use toml::{Table, Value};

use super::budget::Types;
use crate::doccheck::Report;
use crate::doccheck::checks::c004_24::py::py_int;
use crate::doccheck::checks::c004_24::pytoml::{names, parse_toml, tables_strict};
use crate::doccheck::checks::c004_25::merged::Merged;
use crate::doccheck::checks::c004_25::toml_side::{agree_entry, agree_main, name};
use crate::doccheck::checks::c004_25::tree::entry_nodes;
use crate::doccheck::text::has_i;

fn one(msgs: &[Table], key: &'static str, t: &Types, m: &Merged, dscp: Option<i64>, report: &mut Report) -> Result<(), String> {
    let cand: Vec<&Table> = msgs.iter().filter(|x| has_i(key, &name(x)) && x.get("section_of").and_then(Value::as_str) == Some("regime stamp")).collect();
    report.check("AC5", cand.len() == 1, format!("wire.toml declares exactly one regime stamp message for the {key} section type (found {})", cand.len()));
    let Some(a) = t.new.get(key).filter(|a| a.ok) else { return Ok(()) };
    let (1, Some(root)) = (cand.len(), a.root.as_ref()) else { return Ok(()) };
    let msg = cand[0];
    let ok = msg.get("layer").and_then(Value::as_str) == Some("control") && Some(py_int(msg, "dscp", -1)?) == dscp && Some(py_int(msg, "max_size", -1)?) == a.sec_max;
    report.check("AC5", ok, format!("the {key} message of wire.toml is control, DSCP {dscp:?}, and its max_size is the table's {:?} bytes", a.sec_max));
    let reasons = names(msg.get("rejection_reasons"))?;
    report.check("AC5", !reasons.is_empty() && reasons.iter().all(|r| m.is_control(r)), format!("the {key} message names only control reasons (found {reasons:?})"));
    report.check("AC5", agree_main(msg, a.body_rel, root)?, format!("the {key} fields of wire.toml agree with its table row for row (offset, width, count)"));
    let ent_msgs: Vec<&Table> = msgs.iter().filter(|x| has_i(key, &name(x)) && name(x).to_lowercase().contains("entry") && !x.contains_key("section_of")).collect();
    let ents = entry_nodes(root);
    report.check("AC5", ent_msgs.len() == ents.len(), format!("wire.toml declares one entry message per entry table of the {key} section (tables {}, messages {})", ents.len(), ent_msgs.len()));
    for (n, em) in ents.iter().zip(&ent_msgs) {
        report.check("AC5", agree_entry(em, n)?, format!("the entry table of {} agrees with wire.toml row for row and tiles to its size ({} bits)", name(em), n.size));
    }
    Ok(())
}

/// AC5 (document part).
pub(crate) fn ac5(text: Option<&str>, t: &Types, m: &Merged, dscp: Option<i64>, report: &mut Report) -> Result<(), String> {
    let root = match parse_toml(text.unwrap_or("")) {
        Ok(r) => r,
        Err(e) => {
            report.fail("AC5", format!("wire.toml does not parse: {e}"));
            Table::new()
        }
    };
    let mut go = || -> Result<(), String> {
        let msgs = match root.get("message") {
            Some(Value::Array(_)) => tables_strict(root.get("message"))?,
            _ => Vec::new(),
        };
        for key in ["dscp", "alignment"] {
            one(&msgs, key, t, m, dscp, report)?;
        }
        Ok(())
    };
    go()
}
