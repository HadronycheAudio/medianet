//! wire.toml for 004-06: the `[versioning]` values (AC1), the `[[reject]]`
//! enumeration and the media packet `[[message]]` (AC6), read with the
//! scripts' own reader. An exception the script would have raised is
//! returned as `Err`: the script crashed (red, no FAIL line).

use std::collections::BTreeSet;

use super::wp5::big::Big;
use super::header::Size;
use super::wp5::md::norm_name;
use super::wp5::pytoml::{Dict, Value};
use super::wp5::toml::{Crash, WIRE_TOML, items, load, message, repr};
use super::wp5::py::head;
use super::{Ctx, FieldRow};
use crate::doccheck::{Docs, Report};




/// The `[[reject]]` names of wire.toml.
pub(crate) fn toml_members(doc: &Dict) -> Result<BTreeSet<String>, Crash> {
    Ok(items(doc.get("reject"))?
        .iter()
        .filter_map(|e| e.as_dict().and_then(|d| d.get("name")).and_then(Value::as_str).map(str::to_string))
        .collect())
}


const NO_MEDIA: &str = "doc/design/spec/wire.toml has no [[message]] for the media packet (a message whose name normalises to something containing 'media packet')";

/// The rest of the check: versioning, enumeration, the media message.
pub(crate) fn check(docs: &Docs, ctx: &Ctx, size: Option<&Size>, report: &mut Report) -> Result<(), Crash> {
    let Some(doc) = load(docs, "AC6", report)? else {
        if docs.read(WIRE_TOML).is_some() {
            report.fail("AC6", NO_MEDIA);
        }
        return Ok(());
    };
    if let Some(Value::Dict(v)) = doc.get("versioning")
        && !v.0.is_empty()
    {
        if !v.get("magic").is_some_and(|m| m.eq_int(0x4D4E)) {
            report.fail("AC1", format!("{WIRE_TOML} [versioning] magic is {}, not 0x4d4e (004-04)", repr(v.get("magic"))));
        }
        if !v.get("version").is_some_and(|m| m.eq_int(0x0100)) {
            report.fail("AC1", format!("{WIRE_TOML} [versioning] version is {}, not 0x0100 (004-04)", repr(v.get("version"))));
        }
    }
    let member_set: BTreeSet<String> = ctx.members.iter().cloned().collect();
    let declared = toml_members(&doc)?;
    if !declared.is_empty() && declared != member_set {
        report.fail("AC6", "the [[reject]] enumeration in wire.toml disagrees with wire.md's Rejection reasons section");
    }
    let Some(media) = message(&doc, "mediapacket")? else {
        report.fail("AC6", NO_MEDIA);
        return Ok(());
    };
    message_rules(ctx, &media, size, &member_set, report)?;
    if ctx.field_table_found {
        fields_agree(ctx, media.get("field"), report)?;
    }
    Ok(())
}


/// Layer, dscp, max_size and the rejection reasons of the media message.
fn message_rules(ctx: &Ctx, m: &Dict, size: Option<&Size>, members: &BTreeSet<String>, report: &mut Report) -> Result<(), Crash> {
    let name = repr(m.get("name"));
    if !m.get("layer").is_some_and(|l| l.py_eq(&Value::Str("media".into()))) {
        report.fail("AC6", format!("message {name}: layer is {}, not \"media\"", repr(m.get("layer"))));
    }
    if !m.get("dscp").is_some_and(|d| d.eq_int(46)) {
        report.fail("AC6", format!("message {name}: dscp is {}, not 46 (EF, C-13)", repr(m.get("dscp"))));
    }
    match m.get("max_size").and_then(Value::strict_int) {
        None => report.fail("AC6", format!("message {name}: max_size is not stated as an integer")),
        Some(max) => {
            if let Some((bits, bytes)) = size
                && ctx.field_table_found
                && bits.divmod_small(8).1 == 0
                && max < bytes
            {
                report.fail("AC6", format!("message {name}: max_size {max} is smaller than the header's {bytes} bytes"));
            }
        }
    }
    match m.get("rejection_reasons") {
        Some(Value::List(l)) if !l.is_empty() => {
            for reason in l {
                if !reason.hashable() {
                    return Err("TypeError: unhashable type in a set membership test".into());
                }
                if !reason.as_str().is_some_and(|r| members.contains(r)) {
                    report.fail("AC6", format!("message {name}: rejection reason {} is not a member of the closed enumeration", reason.repr()));
                }
            }
        }
        _ => report.fail("AC6", format!("message {name}: rejection_reasons is missing or empty")),
    }
    Ok(())
}

/// `off * 8 <= toff < off * 8 + 8` (a sub-byte field inside its byte).
fn inside_byte(off: &Big, toff: &Big) -> bool {
    let lo = off.mul(&Big::from(8));
    lo <= *toff && *toff < lo.add(&Big::from(8))
}

/// The toml field a wire.md row agrees with: by name (a normalised name of
/// 3+ characters inside the row, not yet matched), else by offset and width.
fn match_row(row: &FieldRow, by_name: &[(String, Dict)], fields: &[Value], matched: &BTreeSet<String>) -> Option<Dict> {
    let row_norm = norm_name(&row.text);
    if let Some((_, f)) = by_name.iter().find(|(n, _)| n.chars().count() >= 3 && row_norm.contains(n.as_str()) && !matched.contains(n)) {
        return Some(f.clone());
    }
    for f in fields {
        let Value::Dict(d) = f else { continue };
        let fname = norm_name(&d.get("name").map_or(String::new(), Value::py_str));
        if matched.contains(&fname) {
            continue;
        }
        let (Some(toff), Some(tw)) = (d.get("offset").and_then(Value::int_like), d.get("width").and_then(Value::int_like)) else {
            continue;
        };
        let hit = if row.bits < Big::from(8) {
            inside_byte(&row.off, &toff) && tw == row.bits
        } else {
            toff == row.off.mul(&Big::from(8)) && tw == row.bits
        };
        if hit {
            return Some(d.clone());
        }
    }
    None
}

/// The `[[message.field]]` rows agree with wire.md name by name, offset by
/// offset.
fn fields_agree(ctx: &Ctx, field: Option<&Value>, report: &mut Report) -> Result<(), Crash> {
    let fields = items(field)?;
    let mut by_name: Vec<(String, Dict)> = Vec::new();
    for f in &fields {
        if let Value::Dict(d) = f
            && let Some(n) = d.get("name").and_then(Value::as_str)
        {
            let key = norm_name(n);
            match by_name.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = d.clone(),
                None => by_name.push((key, d.clone())),
            }
        }
    }
    let mut matched = BTreeSet::new();
    for row in &ctx.field_rows {
        let Some(m) = match_row(row, &by_name, &fields, &matched) else {
            report.fail(
                "AC6",
                format!("no [[message.field]] row agrees with the wire.md field {:?} (offset {}, width {} bits)", head(&row.text, 60), row.off, row.bits),
            );
            continue;
        };
        let mname = repr(m.get("name"));
        matched.insert(norm_name(&m.get("name").map_or(String::new(), Value::py_str)));
        let (Some(toff), Some(tw)) = (m.get("offset").and_then(Value::int_like), m.get("width").and_then(Value::int_like)) else {
            report.fail("AC6", format!("field {mname}: offset/width are not integers"));
            continue;
        };
        if row.bits < Big::from(8) {
            if !inside_byte(&row.off, &toff) {
                report.fail("AC6", format!("field {mname}: offset {toff} is not inside byte {} where wire.md places it", row.off));
            }
        } else if toff != row.off.mul(&Big::from(8)) {
            report.fail("AC6", format!("field {mname}: toml offset {toff} bits vs wire.md offset {} bytes", row.off));
        }
        if tw != row.bits {
            report.fail("AC6", format!("field {mname}: toml width {tw} bits vs wire.md width {} bits", row.bits));
        }
    }
    Ok(())
}
