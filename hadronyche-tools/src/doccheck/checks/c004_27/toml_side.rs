//! AC4 (document part): the probe messages of wire.toml and their agreement
//! with the Markdown tables; the toml type code inside the probe block
//! (reported under AC1, as the script does).
//!
//! wire.toml is read by the faithful port of the script's own reader
//! ([`crate::doccheck::tomlsubset`]), and its values are queried as the
//! script's Python did ([`PyValue`]: `int()`, `isinstance`, `==` across int
//! and float, iteration over a string's characters). Where the script would
//! crash (`int()` of a list, iterating an int, an unhashable member test),
//! the port reports the crash ([`Report::crash`]).

use crate::doccheck::family004::Field;
use super::landed::Landed;
use super::messages::Probe;
use crate::doccheck::Report;
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::text::{first_int, has_i, rx};

fn name_of(m: &Dict) -> String {
    m.get("name").map(PyValue::py_str).unwrap_or_default()
}

/// `int(f.get(key, default))`, the script's crash as `Err`.
fn int_or(f: &Dict, key: &str, default: i128) -> Result<i128, String> {
    f.get(key).map_or(Ok(default), |v| v.py_int().map_err(|e| format!("int() of {key}: {e}")))
}

/// `ms` as an int when `isinstance(ms, int)`; an int beyond `i128` is red
/// by rule (`INT_LIMIT`, recorded by `py_int`), never saturated.
fn max_size(tm: &Dict) -> Option<i128> {
    match tm.get("max_size") {
        Some(v) if v.is_int() => v.py_int().ok(),
        _ => None,
    }
}

fn fields(tm: &Dict) -> Result<Vec<&Dict>, String> {
    tm.get("field").map_or(Ok(Vec::new()), PyValue::py_dicts)
}

fn sizes(tm: &Dict, kind: &str, landed: &Landed, report: &mut Report) -> Result<(), String> {
    let name = name_of(tm);
    let layer = tm.get("layer").is_some_and(|l| l.eq_str("control"));
    let dscp = match (tm.get("dscp"), landed.dscp) {
        (Some(v), Some(d)) => v.eq_int(d),
        (None, None) => true,
        _ => false,
    };
    report.check("AC4", layer && dscp, format!("toml '{name}' is layer control at dscp {:?} (C-13)", landed.dscp));
    let ms = max_size(tm);
    if kind == "request" {
        let above = matches!((ms, landed.bound), (Some(m), Some(b)) if m > b);
        report.check("AC4", above, format!("toml '{name}' declares a max_size above the {:?}-byte bound, so its exemption is not vacuous", landed.bound));
    } else {
        let within = matches!((ms, landed.bound), (Some(m), Some(b)) if m <= b);
        report.check("AC4", within, format!("toml '{name}' declares a max_size at most the {:?}-byte bound: it is not exempt", landed.bound));
        let plain = !fields(tm)?
            .iter()
            .any(|f| f.contains_key("max_count") || f.contains_key("count") || f.get("type").is_some_and(|t| t.eq_str("pad")));
        report.check("AC4", plain, format!("toml '{name}' has no padding and no counted field"));
    }
    Ok(())
}

/// (bit position after the last field, tiling problems, (offset, size) per field).
type TomlLayout = (i128, Vec<String>, Vec<(i128, i128)>);

/// The bit signature of each toml field and the tiling problems; `Err`
/// where `int()` would crash.
fn toml_layout(tf: &[&Dict]) -> Result<TomlLayout, String> {
    let (mut cur, mut probs, mut sig) = (0i128, Vec::new(), Vec::new());
    for f in tf {
        let mult = if f.contains_key("max_count") { int_or(f, "max_count", 1)? } else { 1 };
        let off = int_or(f, "offset", -1)?;
        if off != cur {
            probs.push(format!("'{}' at bit {off}, expected {cur}", f.get("name").map(PyValue::py_str).unwrap_or_default()));
        }
        let size = int_or(f, "width", 0)?.saturating_mul(mult);
        cur = int_or(f, "offset", 0)?.saturating_add(size);
        sig.push((off, size));
    }
    Ok((cur, probs, sig))
}

/// `[r for r in tm.get("rejection_reasons", []) if r not in MEMBERS]`.
fn unknown_reasons(tm: &Dict, landed: &Landed) -> Result<Vec<String>, String> {
    let Some(v) = tm.get("rejection_reasons") else { return Ok(Vec::new()) };
    if let PyValue::List(items) = v
        && let Some(bad) = items.iter().find(|x| matches!(x, PyValue::List(_) | PyValue::Dict(_)))
    {
        return Err(format!("TypeError: unhashable type: '{}'", bad.type_name()));
    }
    Ok(v.py_iter_str()?.into_iter().filter(|r| !landed.members.contains_key(r)).collect())
}

fn layout(tm: &Dict, fl: &[Field], kind: &str, landed: &Landed, report: &mut Report) -> Result<(), String> {
    let name = name_of(tm);
    let ms = max_size(tm);
    let tf = fields(tm)?;
    let (cur, probs, tsig) = toml_layout(&tf)?;
    let tiles = probs.is_empty() && ms.is_some_and(|m| m.checked_mul(8) == Some(cur));
    report.check("AC4", tiles, format!("toml '{name}' fields tile from bit 0 to max_size ({cur} bits against {ms:?} bytes): {probs:?}"));
    let unknown = unknown_reasons(tm, landed)?;
    report.check("AC4", unknown.is_empty(), format!("toml '{name}' rejection_reasons are members of the closed Reject enumeration; not members: {unknown:?}"));
    let md: Vec<(i128, i128)> = fl.iter().map(|f| (f.off.saturating_mul(8), f.size_bits())).collect();
    report.check("AC4", !fl.is_empty() && md == tsig, format!("the {kind} rows of wire.md and of toml '{name}' agree row for row in offset and size"));
    let label = |f: &&&Dict| {
        let s = |k: &str| f.get(k).map(PyValue::py_str).unwrap_or_default();
        format!("{} {}", s("name"), s("meaning"))
    };
    let tt = tf.iter().find(|f| has_i(r"message[_\s]type", &label(f)));
    // a code beyond i128 is in no block, as Python's big int would not be
    let code = tt.and_then(|f| first_int(&f.get("valid").map(PyValue::py_str).unwrap_or_default()).ok().flatten());
    report.check("AC1", landed.in_block(code), format!("toml '{name}' carries a message type inside the probe block"));
    Ok(())
}

/// AC4 for the two probe messages.
pub(crate) fn ac4(probe: &Probe, landed: &Landed, report: &mut Report) {
    let probes: Vec<&Dict> = landed
        .tmsgs
        .iter()
        .filter(|m| {
            let n = name_of(m);
            n.to_lowercase().contains("probe") && !rx(r"(?i)one[\s_-]*way").is_match(&n)
        })
        .collect();
    let mut ok = probes.len() >= 2;
    for (kind, fl) in [("request", &probe.req_fl), ("reply", &probe.rep_fl)] {
        let Some(tm) = probes.iter().find(|m| name_of(m).to_lowercase().contains(kind)) else {
            ok = false;
            report.fail("AC4", format!("wire.toml has no probe {kind} message named with '{kind}'"));
            continue;
        };
        let read = sizes(tm, kind, landed, report).and_then(|()| layout(tm, fl, kind, landed, report));
        if let Err(e) = read {
            report.crash(format!("wire.toml: {e}"));
            return;
        }
    }
    if !ok {
        report.fail("AC4", "wire.toml does not declare one probe request and one probe reply");
    }
}
