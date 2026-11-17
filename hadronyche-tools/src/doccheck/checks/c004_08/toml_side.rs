//! AC5 of 004-08: wire.toml's slice-header fields (fields at offset 128 or
//! more of a message whose name holds "media packet" or "slice header")
//! agree with wire.md; the carrying messages' max_size covers both headers;
//! the media packet can produce BadPacketIndex.

use super::Ctx;
use super::table::Row;
use super::wp5::big::Big;
use super::wp5::md::norm_name;
use super::wp5::py::head;
use super::wp5::pytoml::{Dict, Value};
use super::wp5::toml::{Crash, items, load, message, repr};
use crate::doccheck::{Docs, Report};

/// A slice-header field: (index of its message, the message, the field).
type Slice = (usize, Dict, Dict);

fn name_of(d: &Dict) -> String {
    norm_name(&d.get("name").map_or(String::new(), Value::py_str))
}

/// The fields at offset >= 128 of the media packet and slice-header
/// messages.
fn slice_fields(doc: &Dict) -> Result<Vec<Slice>, Crash> {
    let mut out = Vec::new();
    for (k, m) in items(doc.get("message"))?.into_iter().enumerate() {
        let Value::Dict(msg) = m else { continue };
        let nm = name_of(&msg);
        if !(nm.contains("mediapacket") || nm.contains("sliceheader")) {
            continue;
        }
        for f in items(msg.get("field"))? {
            if let Value::Dict(fd) = f
                && fd.get("offset").and_then(Value::int_like).is_some_and(|o| o >= Big::from(128))
            {
                out.push((k, msg.clone(), fd));
            }
        }
    }
    Ok(out)
}

fn inside_byte(off: &Big, toff: &Big) -> bool {
    let lo = off.mul(&Big::from(8));
    lo <= *toff && *toff < lo.add(&Big::from(8))
}

fn find_match(row: &Row, by_name: &[(String, Slice)], fields: &[Slice], matched: &std::collections::BTreeSet<String>) -> Option<Slice> {
    let row_norm = norm_name(&row.text);
    if let Some((_, e)) = by_name.iter().find(|(n, _)| n.len() >= 3 && row_norm.contains(n.as_str()) && !matched.contains(n)) {
        return Some(e.clone());
    }
    for e in fields {
        if matched.contains(&name_of(&e.2)) {
            continue;
        }
        let (Some(toff), Some(tw)) = (e.2.get("offset").and_then(Value::int_like), e.2.get("width").and_then(Value::int_like)) else {
            continue;
        };
        let hit = if row.bits < Big::from(8) {
            inside_byte(&row.off, &toff) && tw == row.bits
        } else {
            toff == row.off.mul(&Big::from(8)) && tw == row.bits
        };
        if hit {
            return Some(e.clone());
        }
    }
    None
}

/// Row-by-row agreement; returns the carrying messages in first-match order.
fn agree(rows: &[Row], fields: &[Slice], report: &mut Report) -> Vec<(usize, Dict)> {
    let mut by_name: Vec<(String, Slice)> = Vec::new();
    for e in fields {
        if let Some(n) = e.2.get("name").and_then(Value::as_str) {
            let key = norm_name(n);
            match by_name.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = e.clone(),
                None => by_name.push((key, e.clone())),
            }
        }
    }
    let mut matched = std::collections::BTreeSet::new();
    let mut carrying: Vec<(usize, Dict)> = Vec::new();
    for row in rows {
        let Some((k, msg, f)) = find_match(row, &by_name, fields, &matched) else {
            let label = row.cells.get(3).map_or_else(|| head(&row.text, 60), |c| head(c, 60));
            report.fail("AC5", format!("no [[message.field]] row agrees with the wire.md slice-header field {label:?} (offset {}, width {} bits)", row.off, row.bits));
            continue;
        };
        matched.insert(name_of(&f));
        if !carrying.iter().any(|(j, _)| *j == k) {
            carrying.push((k, msg));
        }
        let fname = repr(f.get("name"));
        let (Some(toff), Some(tw)) = (f.get("offset").and_then(Value::int_like), f.get("width").and_then(Value::int_like)) else {
            report.fail("AC5", format!("field {fname}: offset/width are not integers"));
            continue;
        };
        if row.bits < Big::from(8) {
            if !inside_byte(&row.off, &toff) {
                report.fail("AC5", format!("field {fname}: offset {toff} is not inside byte {} where wire.md places it", row.off));
            }
        } else if toff != row.off.mul(&Big::from(8)) {
            report.fail("AC5", format!("field {fname}: toml offset {toff} bits vs wire.md offset {} bytes", row.off));
        }
        if tw != row.bits {
            report.fail("AC5", format!("field {fname}: toml width {tw} bits vs wire.md width {} bits", row.bits));
        }
    }
    carrying
}

/// AC5.
pub(crate) fn ac5(docs: &Docs, ctx: &Ctx, slice_bytes: Option<Big>, report: &mut Report) -> Result<(), Crash> {
    let doc = load(docs, "AC5", report)?;
    let media = match &doc {
        Some(d) => message(d, "mediapacket")?,
        None => None,
    };
    let fields = match &doc {
        Some(d) => slice_fields(d)?,
        None => Vec::new(),
    };
    if !ctx.open {
        return Ok(());
    }
    if media.is_none() && fields.is_empty() {
        report.fail("AC5", "doc/design/spec/wire.toml has no [[message]] carrying the slice header (neither the media packet message nor a slice-header field group)");
    }
    if let Some(rows) = &ctx.field {
        if fields.is_empty() {
            report.fail("AC5", "doc/design/spec/wire.toml declares no slice-header fields (no [[message.field]] at offset >= 128 bits in the media packet or a slice-header group)");
        } else {
            let carrying = agree(rows, &fields, report);
            if let Some(sb) = &slice_bytes {
                let need = Big::from(16).add(sb);
                for (_, msg) in carrying {
                    let name = repr(msg.get("name"));
                    match msg.get("max_size").and_then(Value::strict_int) {
                        None => report.fail("AC5", format!("message {name}: max_size is not stated as an integer")),
                        Some(m) if *m < need => report.fail("AC5", format!("message {name}: max_size {m} does not cover the packet header plus the slice header ({need} bytes)")),
                        Some(_) => {}
                    }
                }
            }
        }
    }
    if let Some(m) = media {
        let has = match m.get("rejection_reasons") {
            Some(Value::List(l)) => l.iter().any(|r| r.py_eq(&Value::Str("BadPacketIndex".into()))),
            _ => false,
        };
        if !has {
            report.fail("AC5", "the media packet message's rejection_reasons do not include BadPacketIndex (the slice header adds it, per the wire.toml note)");
        }
    }
    Ok(())
}
