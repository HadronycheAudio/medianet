//! AC4 (document part): the probe messages of wire.toml and their agreement
//! with the Markdown tables; the toml type code inside the probe block
//! (reported under AC1, as the script does).
//!
//! Values are read as the script's Python saw them (lead ruling R3): `int()`
//! of a value ([`py_int`]: a float truncates, a string parses, anything else
//! is the script's crash), `isinstance(x, int)` ([`py_is_int`]), `==`
//! across int and float ([`py_eq_int`]), and iteration over a string's
//! characters ([`py_iter`]: `rejection_reasons = "BadLength"` names the
//! "reasons" `B`, `a`, `d`, ..., none a member, so it fails as it did).

use toml::{Table, Value};

use super::Field;
use super::landed::Landed;
use super::messages::Probe;
use crate::doccheck::Report;
use crate::doccheck::pycompat::{py_dicts, py_eq_int, py_int, py_is_int, py_iter};
use crate::doccheck::text::{first_int, has_i, rx};
use crate::doccheck::wire_toml::display;

fn name_of(m: &Table) -> String {
    m.get("name").map(display).unwrap_or_default()
}

/// `int(f.get(key, default))`, the script's crash as `Err`.
fn int_or(f: &Table, key: &str, default: i128) -> Result<i128, String> {
    f.get(key).map_or(Ok(default), |v| py_int(v).map_err(|e| format!("{key}: {e}")))
}

fn sizes(tm: &Table, kind: &str, landed: &Landed, report: &mut Report) -> Result<(), String> {
    let name = name_of(tm);
    let layer = matches!(tm.get("layer"), Some(Value::String(l)) if l == "control");
    let dscp = match (tm.get("dscp"), landed.dscp) {
        (Some(v), Some(d)) => py_eq_int(v, d),
        (None, None) => true,
        _ => false,
    };
    report.check("AC4", layer && dscp, format!("toml '{name}' is layer control at dscp {:?} (C-13)", landed.dscp));
    let ms = tm.get("max_size").filter(|v| py_is_int(v)).map(py_int).transpose().map_err(|e| e.to_string())?;
    if kind == "request" {
        let above = matches!((ms, landed.bound), (Some(m), Some(b)) if m > b);
        report.check("AC4", above, format!("toml '{name}' declares a max_size above the {:?}-byte bound, so its exemption is not vacuous", landed.bound));
    } else {
        let within = matches!((ms, landed.bound), (Some(m), Some(b)) if m <= b);
        report.check("AC4", within, format!("toml '{name}' declares a max_size at most the {:?}-byte bound: it is not exempt", landed.bound));
        let plain = !py_dicts(tm.get("field"))?
            .iter()
            .any(|f| f.contains_key("max_count") || f.contains_key("count") || matches!(f.get("type"), Some(Value::String(t)) if t == "pad"));
        report.check("AC4", plain, format!("toml '{name}' has no padding and no counted field"));
    }
    Ok(())
}

/// The bit signature (offset, size) of each toml field, and the tiling
/// problems; `Err` where `int()` would crash.
/// (bit position after the last field, tiling problems, (offset, size) per field).
type TomlLayout = (i128, Vec<String>, Vec<(i128, i128)>);

fn toml_layout(tf: &[&Table]) -> Result<TomlLayout, String> {
    let (mut cur, mut probs, mut sig) = (0i128, Vec::new(), Vec::new());
    for f in tf {
        let mult = if f.contains_key("max_count") { int_or(f, "max_count", 1)? } else { 1 };
        let off = int_or(f, "offset", -1)?;
        if off != cur {
            probs.push(format!("'{}' at bit {off}, expected {cur}", f.get("name").map(display).unwrap_or_default()));
        }
        let size = int_or(f, "width", 0)?.saturating_mul(mult);
        cur = int_or(f, "offset", 0)?.saturating_add(size);
        sig.push((off, size));
    }
    Ok((cur, probs, sig))
}

fn layout(tm: &Table, fl: &[Field], kind: &str, landed: &Landed, report: &mut Report) -> Result<(), String> {
    let name = name_of(tm);
    let ms = tm.get("max_size").filter(|v| py_is_int(v)).map(py_int).transpose().map_err(|e| e.to_string())?;
    let tf = py_dicts(tm.get("field"))?;
    let (cur, probs, tsig) = toml_layout(&tf)?;
    let tiles = probs.is_empty() && ms.is_some_and(|m| m.checked_mul(8) == Some(cur));
    report.check("AC4", tiles, format!("toml '{name}' fields tile from bit 0 to max_size ({cur} bits against {ms:?} bytes): {probs:?}"));
    let reasons = match tm.get("rejection_reasons") {
        Some(v) => py_iter(v)?,
        None => Vec::new(),
    };
    let unknown: Vec<&String> = reasons.iter().filter(|r| !landed.members.contains_key(*r)).collect();
    report.check("AC4", unknown.is_empty(), format!("toml '{name}' rejection_reasons are members of the closed Reject enumeration; not members: {unknown:?}"));
    let md: Vec<(i128, i128)> = fl.iter().map(|f| (f.off.saturating_mul(8), f.size_bits())).collect();
    report.check("AC4", !fl.is_empty() && md == tsig, format!("the {kind} rows of wire.md and of toml '{name}' agree row for row in offset and size"));
    let label = |f: &&&Table| format!("{} {}", f.get("name").map(display).unwrap_or_default(), f.get("meaning").map(display).unwrap_or_default());
    let tt = tf.iter().find(|f| has_i(r"message[_\s]type", &label(f)));
    let code = tt.and_then(|f| first_int(&f.get("valid").map(display).unwrap_or_default()).ok().flatten());
    report.check("AC1", landed.in_block(code), format!("toml '{name}' carries a message type inside the probe block"));
    Ok(())
}

/// AC4 for the two probe messages.
pub(crate) fn ac4(probe: &Probe, landed: &Landed, report: &mut Report) {
    let probes: Vec<&Table> = landed
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
            report.fail("AC4", format!("wire.toml: the script's reader crashes here: {e}"));
            return;
        }
    }
    if !ok {
        report.fail("AC4", "wire.toml does not declare one probe request and one probe reply");
    }
}
