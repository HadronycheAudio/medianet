//! AC4 (document part): the probe messages of wire.toml and their agreement
//! with the Markdown tables; the toml type code inside the probe block
//! (reported under AC1, as the script does).

use toml::Table;

use super::Field;
use super::landed::Landed;
use super::messages::Probe;
use crate::doccheck::Report;
use crate::doccheck::text::{first_int, has_i, rx};
use crate::doccheck::wire_toml::{display, int, tables_of, text};

fn fields(m: &Table) -> Vec<&Table> {
    tables_of(m.get("field"))
}

fn mult(f: &Table) -> i64 {
    if f.contains_key("max_count") { int(f, "max_count").unwrap_or(1) } else { 1 }
}

fn sizes(tm: &Table, report: &mut Report, kind: &str, landed: &Landed) {
    let name = text(tm, "name");
    let ms = int(tm, "max_size");
    let ok = text(tm, "layer") == "control" && int(tm, "dscp") == landed.dscp;
    report.check("AC4", ok, format!("toml '{name}' is layer control at dscp {:?} (C-13)", landed.dscp));
    if kind == "request" {
        let above = matches!((ms, landed.bound), (Some(m), Some(b)) if m > b);
        report.check("AC4", above, format!("toml '{name}' declares a max_size above the {:?}-byte bound, so its exemption is not vacuous", landed.bound));
    } else {
        let within = matches!((ms, landed.bound), (Some(m), Some(b)) if m <= b);
        report.check("AC4", within, format!("toml '{name}' declares a max_size at most the {:?}-byte bound: it is not exempt", landed.bound));
        let plain = !fields(tm).iter().any(|f| f.contains_key("max_count") || f.contains_key("count") || text(f, "type") == "pad");
        report.check("AC4", plain, format!("toml '{name}' has no padding and no counted field"));
    }
}

fn layout(tm: &Table, fl: &[Field], kind: &str, landed: &Landed, report: &mut Report) {
    let name = text(tm, "name");
    let ms = int(tm, "max_size");
    let tf = fields(tm);
    let (mut cur, mut probs) = (0, Vec::new());
    for f in &tf {
        let off = int(f, "offset").unwrap_or(-1);
        if off != cur {
            probs.push(format!("'{}' at bit {off}, expected {cur}", text(f, "name")));
        }
        cur = int(f, "offset").unwrap_or(0) + int(f, "width").unwrap_or(0) * mult(f);
    }
    report.check("AC4", probs.is_empty() && ms.is_some_and(|m| cur == m * 8), format!("toml '{name}' fields tile from bit 0 to max_size ({cur} bits against {ms:?} bytes): {probs:?}"));
    let reasons = tm.get("rejection_reasons").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let unknown: Vec<String> = reasons.iter().map(display).filter(|r| !landed.members.contains_key(r)).collect();
    report.check("AC4", unknown.is_empty(), format!("toml '{name}' rejection_reasons are members of the closed Reject enumeration; not members: {unknown:?}"));
    let md: Vec<(i64, i64)> = fl.iter().map(|f| (f.off * 8, f.size_bits())).collect();
    let tm_sig: Vec<(i64, i64)> = tf.iter().map(|f| (int(f, "offset").unwrap_or(-1), int(f, "width").unwrap_or(0) * mult(f))).collect();
    report.check("AC4", !fl.is_empty() && md == tm_sig, format!("the {kind} rows of wire.md and of toml '{name}' agree row for row in offset and size"));
    let tt = tf.iter().find(|f| has_i(r"message[_\s]type", &format!("{} {}", text(f, "name"), text(f, "meaning"))));
    let code = tt.and_then(|f| first_int(&f.get("valid").map(display).unwrap_or_default()));
    report.check("AC1", landed.in_block(code), format!("toml '{name}' carries a message type inside the probe block"));
}

/// AC4 for the two probe messages.
pub(crate) fn ac4(probe: &Probe, landed: &Landed, report: &mut Report) {
    let probes: Vec<&Table> = landed
        .tmsgs
        .iter()
        .filter(|m| {
            let n = text(m, "name");
            n.to_lowercase().contains("probe") && !rx(r"(?i)one[\s_-]*way").is_match(n)
        })
        .collect();
    let mut ok = probes.len() >= 2;
    for (kind, fl) in [("request", &probe.req_fl), ("reply", &probe.rep_fl)] {
        let Some(tm) = probes.iter().find(|m| text(m, "name").to_lowercase().contains(kind)) else {
            ok = false;
            report.fail("AC4", format!("wire.toml has no probe {kind} message named with '{kind}'"));
            continue;
        };
        sizes(tm, report, kind, landed);
        layout(tm, fl, kind, landed, report);
    }
    if !ok {
        report.fail("AC4", "wire.toml does not declare one probe request and one probe reply");
    }
}
