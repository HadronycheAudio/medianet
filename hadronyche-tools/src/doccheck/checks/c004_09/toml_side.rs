//! AC6 of 004-09: the announcement fields of wire.toml (media packet fields
//! at offset 304 or more, or the fields of a message whose name holds
//! "announcement"), with type, length, a 64-bit u64 position and a count
//! whose maximum is the stated one; the carrying messages' reasons and
//! max_size. The fields are read whatever the state of the section (the
//! DEFECT detector reads their strings).

use std::collections::BTreeSet;

use super::Ctx;
use super::wp5::big::Big;
use super::wp5::md::norm_name;
use super::wp5::pytoml::{Dict, Value};
use super::wp5::toml::{Crash, items, load, repr};
use crate::doccheck::{Docs, Report};

/// (index of the carrying message, the field).
pub(crate) type AnnField = (usize, Dict);

fn name_norm(d: &Dict) -> String {
    norm_name(&d.get("name").map_or(String::new(), Value::py_str))
}

/// The announcement fields and the carrying messages, in order.
/// The announcement fields and their carrying messages.
type Collected = (Vec<AnnField>, Vec<(usize, Dict)>);

fn collect(doc: &Dict) -> Result<Collected, Crash> {
    let mut fields = Vec::new();
    let mut carrying: Vec<(usize, Dict)> = Vec::new();
    for (k, m) in items(doc.get("message"))?.into_iter().enumerate() {
        let Value::Dict(msg) = m else { continue };
        let nm = name_norm(&msg);
        let (media, ann) = (nm.contains("mediapacket"), nm.contains("announcement"));
        for f in items(msg.get("field"))? {
            let Value::Dict(fd) = f else { continue };
            let Some(off) = fd.get("offset").and_then(Value::int_like) else { continue };
            if (media && off >= Big::from(304)) || (ann && off >= Big::default()) {
                fields.push((k, fd));
                if !carrying.iter().any(|(j, _)| *j == k) {
                    carrying.push((k, msg.clone()));
                }
            }
        }
    }
    Ok((fields, carrying))
}

/// The first field of each kind (by its lower-cased name), among those with
/// an integer width.
fn by_kind(fields: &[AnnField]) -> [Option<Dict>; 4] {
    let mut out: [Option<Dict>; 4] = [None, None, None, None];
    for (_, f) in fields {
        if f.get("width").and_then(Value::int_like).is_none() {
            continue;
        }
        let name = f.get("name").map_or(String::new(), Value::py_str).to_lowercase();
        let kind = if name.contains("position") {
            2
        } else if name.contains("length") {
            1
        } else if name.contains("type") && !crate::doccheck::text::rx(r"type.?code.?table").is_match(&name) {
            0
        } else if name.contains("count") {
            3
        } else {
            continue;
        };
        if out[kind].is_none() {
            out[kind] = Some(f.clone());
        }
    }
    out
}

/// The count field's maximum: its own `max_count`, else that of the first
/// field it counts.
fn count_max(count: &Dict, fields: &[AnnField]) -> Option<Big> {
    if let Some(m) = count.get("max_count").and_then(Value::strict_int) {
        return Some(m.clone());
    }
    let name = count.get("name");
    fields.iter().find_map(|(_, f2)| {
        let same = match (f2.get("count"), name) {
            (None, None) => true,
            (Some(a), Some(b)) => a.py_eq(b),
            _ => false,
        };
        if same { f2.get("max_count").and_then(Value::strict_int).cloned() } else { None }
    })
}

/// The field checks.
fn kinds(fields: &[AnnField], max_count: Option<&Big>, report: &mut Report) {
    let found = by_kind(fields);
    for (k, kind) in ["type", "length", "position", "count"].iter().enumerate() {
        if found[k].is_none() {
            report.fail("AC6", format!("the announcement fields in doc/design/spec/wire.toml have no {kind} field"));
        }
    }
    if let Some(p) = &found[2]
        && !(p.get("width").is_some_and(|w| w.eq_int(64)) && p.get("type").is_some_and(|t| t.py_eq(&Value::Str("u64".into()))))
    {
        report.fail("AC6", "the announcement position field in wire.toml is not a 64-bit u64 (the slice header's position form)");
    }
    if let Some(c) = &found[3] {
        match count_max(c, fields) {
            None => report.fail("AC6", "the announcement count field has no fixed maximum (every counted list states one; this is how the maximum size is computed)"),
            Some(mc) => {
                if let Some(n) = max_count
                    && mc != *n
                {
                    report.fail("AC6", format!("the count field's maximum ({mc}) disagrees with the stated maximum number of announcements per packet ({n})"));
                }
            }
        }
    }
}

/// The carrying messages' reasons and max_size.
fn carriers(carrying: &[(usize, Dict)], members: &[String], report: &mut Report) -> Result<(), Crash> {
    let set: BTreeSet<&str> = members.iter().map(String::as_str).collect();
    for (_, msg) in carrying {
        let name = repr(msg.get("name"));
        let Some(Value::List(l)) = msg.get("rejection_reasons").filter(|v| v.truthy()) else {
            report.fail("AC6", format!("message {name}: no rejection_reasons declared for the announcement-carrying message"));
            continue;
        };
        if !l.iter().any(|r| r.py_eq(&Value::Str("BadLength".into()))) {
            report.fail("AC6", format!("message {name}: rejection_reasons do not include BadLength (length overrun and count-above-maximum are announcement rejections)"));
        }
        for r in l {
            if !r.hashable() {
                return Err("TypeError: unhashable type in a set membership test".into());
            }
            if !r.as_str().is_some_and(|x| set.contains(x)) {
                report.fail("AC6", format!("message {name}: rejection reason {} is not a member of the closed enumeration", r.repr()));
            }
        }
        match msg.get("max_size").and_then(Value::strict_int) {
            None => report.fail("AC6", format!("message {name}: max_size is not an integer")),
            Some(m) if name_norm(msg).contains("mediapacket") && *m <= Big::from(38) => {
                report.fail("AC6", format!("message {name}: max_size {m} does not grow past the 38-byte packet + slice headers the announcements extend"))
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// AC6; returns the announcement fields (for the DEFECT detector).
pub(crate) fn ac6(docs: &Docs, ctx: &Ctx, max_count: Option<&Big>, report: &mut Report) -> Result<Vec<AnnField>, Crash> {
    let doc = load(docs, "AC6", report)?;
    let (fields, carrying) = match &doc {
        Some(d) => collect(d)?,
        None => (Vec::new(), Vec::new()),
    };
    if !ctx.open {
        return Ok(fields);
    }
    if fields.is_empty() {
        report.fail("AC6", "doc/design/spec/wire.toml declares no announcement fields (neither media-packet fields at offset >= 304 bits nor an announcement sub-table message)");
    } else {
        kinds(&fields, max_count, report);
    }
    carriers(&carrying, &ctx.members, report)?;
    Ok(fields)
}
