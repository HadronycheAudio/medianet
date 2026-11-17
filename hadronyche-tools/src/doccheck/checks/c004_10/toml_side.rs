//! AC5 of 004-10: wire.toml carries the extension's three fields (media
//! form: in the media packet, inside the window after the announcement
//! container's maximum end; section form: a "measurement" message with
//! `section_of = "media packet"`), agreeing with wire.md; the max_size
//! invariant (container end + extension <= max_size <= 100); members-only
//! reasons including BadLength and ReservedNonZero. Runs whatever the state
//! of the wire.md section, as the script did.

use std::collections::BTreeSet;

use super::Ctx;
use super::fields::KEYS;
use super::wp5::big::Big;
use super::wp5::md::norm_name;
use super::wp5::pytoml::{Dict, Value};
use super::wp5::toml::{Crash, items, load, repr};
use crate::doccheck::{Docs, Report};

const NO_FIELDS: &str = "doc/design/spec/wire.toml declares no measurement-extension fields (hop count, residence time, declared delay): neither media packet fields after the announcement container nor a measurement section message";

/// `[m for m in document.get("message", []) if isinstance(m, dict)]`
/// (no `or []`: a number or a bool raised).
fn messages(doc: &Dict) -> Result<Vec<Dict>, Crash> {
    let list = match doc.get("message") {
        None => Vec::new(),
        Some(v @ (Value::Int(_) | Value::Float(_) | Value::Bool(_))) => {
            return Err(format!("TypeError: '{}' object is not iterable", v.repr()));
        }
        Some(v) => items(Some(v))?,
    };
    Ok(list.into_iter().filter_map(|m| if let Value::Dict(d) = m { Some(d) } else { None }).collect())
}

fn named(msgs: &[Dict], needle: &str) -> Option<Dict> {
    msgs.iter().find(|m| norm_name(&m.get("name").map_or(String::new(), Value::py_str)).contains(needle)).cloned()
}

/// The counted region's maximum end: max of `offset + width * max_count`.
fn container_end(media: &Dict) -> Result<Option<Big>, Crash> {
    let mut end: Option<Big> = None;
    for f in items(media.get("field"))? {
        let Value::Dict(d) = f else { continue };
        let int = |k| d.get(k).and_then(Value::int_like);
        if let (Some(c), Some(o), Some(w)) = (int("max_count"), int("offset"), int("width")) {
            let e = o.add(&w.mul(&c));
            if end.as_ref().is_none_or(|x| e > *x) {
                end = Some(e);
            }
        }
    }
    Ok(end)
}

/// `toml_key(name)`.
fn toml_key(name: &str) -> Option<usize> {
    let n = norm_name(name);
    (0..3).find(|&k| n.contains(KEYS[k].0))
}

/// The extension fields in the toml: key → field, and the form.
type Found = ([Option<Dict>; 3], Option<&'static str>);

fn find_fields(ctx: &Ctx, media: Option<&Dict>, meas: Option<&Dict>, end: Option<&Big>) -> Result<Found, Crash> {
    let mut found: [Option<Dict>; 3] = [None, None, None];
    let mut form = None;
    let window = ctx.all().and_then(|a| a.iter().map(|e| e.bits.clone().filter(|b| !b.is_zero())).collect::<Option<Vec<Big>>>());
    if let (Some(m), Some(end), Some(w)) = (media, end, window) {
        let width = w.iter().fold(Big::default(), |a, b| a.add(b));
        for f in items(m.get("field"))? {
            let Value::Dict(d) = f else { continue };
            let Some(off) = d.get("offset").and_then(Value::int_like) else { continue };
            let key = toml_key(&d.get("name").map_or(String::new(), Value::py_str));
            if let Some(k) = key
                && *end <= off
                && off < end.add(&width)
                && found[k].is_none()
            {
                found[k] = Some(d);
                form = Some("media");
            }
        }
    }
    if let Some(m) = meas
        && found.iter().all(Option::is_none)
    {
        for f in items(m.get("field"))? {
            let Value::Dict(d) = f else { continue };
            if d.get("offset").and_then(Value::int_like).is_none() {
                continue;
            }
            if let Some(k) = toml_key(&d.get("name").map_or(String::new(), Value::py_str))
                && found[k].is_none()
            {
                found[k] = Some(d);
                form = Some("section");
            }
        }
    }
    Ok((found, form))
}

/// Offsets, widths and types against wire.md; a missing wire.md offset is
/// the script's crash (`min()` of nothing, `None - int`).
fn agreement(ctx: &Ctx, found: &[Dict; 3], form: &str, end: Option<&Big>, report: &mut Report) -> Result<(), Crash> {
    let Some(all) = ctx.all() else { return Ok(()) };
    let offs: Vec<Big> = found.iter().map(|f| f.get("offset").and_then(Value::int_like).expect("filtered")).collect();
    let base = match (form, end) {
        ("media", Some(e)) => e.clone(),
        _ => offs.iter().min().expect("three").clone(),
    };
    let md: Option<Vec<Big>> = all.iter().map(|e| e.off.clone()).collect();
    let Some(md) = md else {
        return Err("TypeError: unsupported operand type(s) for -: 'NoneType' and 'int'".into());
    };
    let md_base = md.iter().min().expect("three").clone();
    for k in 0..3 {
        let f = &found[k];
        let name = repr(f.get("name"));
        let md_rel = md[k].sub(&md_base).mul(&Big::from(8));
        let toml_rel = offs[k].sub(&base);
        if toml_rel != md_rel {
            report.fail("AC5", format!("field {name}: wire.toml places it {toml_rel} bits into the extension, wire.md {md_rel} bits"));
        }
        let width = f.get("width");
        let same = match (&all[k].bits, width) {
            (None, None) => true,
            (Some(b), Some(w)) => w.py_eq(&Value::Int(b.clone())),
            _ => false,
        };
        if !same {
            report.fail("AC5", format!("field {name}: wire.toml width {} bits vs wire.md width {:?} bits", repr(width), all[k].bits.as_ref().map(Big::to_string)));
        }
        let ty = f.get("type").map_or(String::new(), Value::py_str).to_lowercase();
        if all[k].ty != ty {
            report.fail("AC5", format!("field {name}: wire.toml type {} vs wire.md type {:?}", repr(f.get("type")), all[k].ty));
        }
    }
    Ok(())
}

/// Widths of the toml fields: whole bytes, summed (`width // 8`).
fn toml_bytes(found: &[Option<Dict>; 3], report: &mut Report) -> Big {
    let mut total = Big::default();
    for f in found.iter().flatten() {
        let Some(w) = f.get("width").and_then(Value::int_like) else {
            report.fail("AC5", format!("field {}: width is not an integer", repr(f.get("name"))));
            continue;
        };
        let (q, r) = w.divmod_small(8);
        total = total.add(&q);
        if r != 0 {
            report.fail("AC5", format!("field {}: width {w} bits is not whole bytes", repr(f.get("name"))));
        }
    }
    total
}

/// The max_size invariant of the carrying message (and of the media packet
/// in the section form).
fn sizes(carrying: &Dict, media: Option<&Dict>, form: &str, end: &Big, bytes: &Big, report: &mut Report) {
    let base = end.divmod_small(8).0;
    let need = base.add(bytes);
    let cap = Big::from(100);
    let name = repr(carrying.get("name"));
    let max = carrying.get("max_size").and_then(Value::strict_int);
    match max {
        None => report.fail("AC5", format!("message {name}: max_size is not an integer")),
        Some(m) if form == "section" && m != bytes => report.fail("AC5", format!("message {name}: max_size {m} != extension bytes {bytes}")),
        Some(m) if form == "media" && *m < need => {
            report.fail("AC5", format!("media packet max_size {m} does not hold the container's maximum end ({base} bytes) plus the extension ({bytes} bytes)"))
        }
        Some(_) => {}
    }
    let media_max = media.and_then(|m| m.get("max_size")).and_then(Value::strict_int);
    if form == "section" && media_max.is_none_or(|m| *m < need) {
        report.fail("AC5", format!("media packet max_size does not hold the container's maximum end plus the extension ({need} bytes)"));
    }
    if let Some(m) = max
        && *m > cap
    {
        report.fail("AC5", format!("message {name}: max_size {m} exceeds the C-137 header cap of 100 bytes"));
    }
    if form == "section"
        && let Some(m) = media_max
        && *m > cap
    {
        report.fail("AC5", format!("media packet max_size {m} exceeds the C-137 header cap of 100 bytes"));
    }
}

/// Members only, BadLength and ReservedNonZero among them.
fn reasons(carrying: &Dict, members: &[String], report: &mut Report) -> Result<(), Crash> {
    let Some(Value::List(l)) = carrying.get("rejection_reasons").filter(|v| v.truthy()) else {
        report.fail("AC5", "the extension-carrying message declares no rejection_reasons");
        return Ok(());
    };
    let set: BTreeSet<&str> = members.iter().map(String::as_str).collect();
    for r in l {
        if !r.hashable() {
            return Err("TypeError: unhashable type in a set membership test".into());
        }
        if !r.as_str().is_some_and(|x| set.contains(x)) {
            report.fail("AC5", format!("rejection reason {} is not a member of the closed enumeration", r.repr()));
        }
    }
    for need in ["BadLength", "ReservedNonZero"] {
        if !l.iter().any(|r| r.py_eq(&Value::Str(need.into()))) {
            report.fail("AC5", format!("the extension-carrying message's rejection_reasons do not include {need}"));
        }
    }
    Ok(())
}

/// AC5.
pub(crate) fn ac5(docs: &Docs, ctx: &Ctx, stated: Option<&Big>, report: &mut Report) -> Result<(), Crash> {
    let doc = load(docs, "AC5", report)?;
    let msgs = match &doc {
        Some(d) => messages(d)?,
        None => Vec::new(),
    };
    let media = named(&msgs, "mediapacket");
    let meas = named(&msgs, "measurement");
    let end = match &media {
        Some(m) => container_end(m)?,
        None => None,
    };
    let (found, form) = find_fields(ctx, media.as_ref(), meas.as_ref(), end.as_ref())?;
    if end.is_none() {
        report.fail("AC5", "wire.toml's media packet declares no counted announcement region to place the extension after");
    }
    let Some(form) = form else {
        report.fail("AC5", NO_FIELDS);
        return Ok(());
    };
    for (k, (_, _, label)) in KEYS.iter().enumerate() {
        if found[k].is_none() {
            report.fail("AC5", format!("doc/design/spec/wire.toml has no field for the {label}"));
        }
    }
    let carrying = if form == "media" { media.clone() } else { meas.clone() }.expect("the form names a message");
    if form == "section" && !carrying.get("section_of").is_some_and(|v| v.py_eq(&Value::Str("media packet".into()))) {
        report.fail("AC5", "the measurement section message does not declare section_of = \"media packet\"");
    }
    let bytes = toml_bytes(&found, report);
    if let [Some(a), Some(b), Some(c)] = &found {
        agreement(ctx, &[a.clone(), b.clone(), c.clone()], form, end.as_ref(), report)?;
    }
    if let Some(n) = stated
        && *n != bytes
    {
        report.fail("AC5", format!("wire.toml's extension fields total {bytes} bytes; wire.md states {n}"));
    }
    if let Some(e) = &end {
        sizes(&carrying, media.as_ref(), form, e, &bytes, report);
    }
    reasons(&carrying, &ctx.members, report)
}
