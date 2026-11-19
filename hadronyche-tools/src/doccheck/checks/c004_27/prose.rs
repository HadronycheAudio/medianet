//! AC1 (the rule sentences of the section) and AC2 (the exemption).

use std::collections::BTreeSet;

use super::landed::Landed;
use super::messages::Probe;
use crate::doccheck::family004::sentences;
use crate::doccheck::Report;
use crate::doccheck::text::{has, has_i, rx};
use crate::doccheck::units::{BlockKind, prose_blocks};
use crate::doccheck::pyvalue::{Dict, PyValue};

fn matching(probe: &Probe, report: &mut Report) {
    let ok = probe.subs.iter().filter(|(t, _)| has_i("match|binding|identif", t)).any(|(_, x)| {
        has_i("reply", x) && has_i(r"request\s+id|probe\s+identifier|intended\s+packet\s+size", x)
    });
    report.check("AC1", ok, "a matching subsection says how a reply identifies its probe, naming the reply and an identifier of the probe (request id, probe identifier or intended packet size)");
}

fn rules(probe: &Probe, report: &mut Report) {
    let t = &probe.text;
    let paras: Vec<String> = prose_blocks(t).into_iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text).collect();
    let size_rule = paras.iter().any(|p| has_i("intended", p) && has_i("probe", p) && has_i(r"packet\s+size", p));
    report.check("AC1", size_rule, "a prose paragraph states the probe size rule: the probe is sent at the intended media packet size");
    let df = has_i(r"don\W{0,2}t[\s-]+fragment|\bDF\b", t) && has_i(r"rule\s+11", t) && has_i("fragmentation", t);
    report.check("AC1", df, "the DF requirement is stated: every probe is sent with don't-fragment set and fragmentation is forbidden (rule 11)");
    let ip = t.contains("IPv4") && t.contains("C-100") && has(r"CS5\s*\(40\)", t) && t.contains("C-13");
    report.check("AC1", ip, "the probe pair is IPv4 only (C-100) and travels at CS5 (40) (C-13)");
    let ruling = paras.iter().any(|p| {
        has_i(r"small\s+acknowledg", p)
            && has_i(r"only\s+the\s+probe\s+is\s+padded|the\s+probe\s+alone\s+is\s+padded", p)
            && has_i("exempt", p)
            && p.contains("[decision]")
    });
    report.check("AC1", ruling, "a [decision] paragraph states the owner's ruling: the reply is a small acknowledgement, and only the probe is padded and exempt from C-17");
    let rep = &probe.rep_text;
    report.check("AC1", !rep.is_empty() && !has_i(r"\bis\s+padded\b|as\s+long\s+as\s+the\s+probe", rep), "the probe reply subsection does not say the reply is padded or as long as the probe");
    let silence = r"no\s+reply|does\s+not\s+arrive|no\s+answer|no\s+response|not\s+answered|never\s+arrives|silence";
    report.check("AC1", has_i(silence, t), "the outcome when no reply arrives is stated");
}

/// The script's `(FALLBACK_MTU or 0)`: Python's `or`, not a parse default.
fn or_zero(n: Option<i128>) -> i128 {
    n.unwrap_or_default()
}

fn fallback(probe: &Probe, landed: &Landed, report: &mut Report) {
    let t = &probe.text;
    let at = |re: &str| rx(re).find(t).map(|m| m.start());
    let p_int = at("(?i)intended");
    let p_fb = at(&format!(r"\b{}\b", or_zero(landed.fallback)));
    let p_ref = at("PathMtuTooSmall");
    let ordered = matches!((p_int, p_fb, p_ref), (Some(a), Some(b), Some(c)) if a < b && b < c);
    report.check("AC1", ordered, format!("the fallback sequence is stated in order: the intended size, then the {:?}-byte fallback, then the refusal PathMtuTooSmall (C-173)", landed.fallback));
    let cites = has(&format!(r"\b{}\b", or_zero(landed.bound)), t) && t.contains("C-06") && t.contains("C-173");
    report.check("AC1", cites, format!("the section cites C-06 and C-173 and states the {:?}-byte UDP payload of the fallback", landed.bound));
}

/// AC1's sentence checks.
pub(crate) fn ac1_prose(probe: &Probe, landed: &Landed, report: &mut Report) {
    matching(probe, report);
    rules(probe, report);
    fallback(probe, landed, report);
}

/// AC2: the exemption cites C-116 and is carried by exactly the probe
/// request; every sentence that says "exempt" cites C-116 only.
pub(crate) fn ac2(probe: &Probe, landed: &Landed, report: &mut Report) {
    // str(m.get("name", "")) for the filters; the raw value for EXEMPT_ALL
    let name = |m: &Dict| m.get("name").map(PyValue::py_str).unwrap_or_default();
    let probes: Vec<&Dict> = landed
        .tmsgs
        .iter()
        .filter(|m| name(m).to_lowercase().contains("probe") && !has_i(r"one[\s_-]*way", &name(m)))
        .collect();
    let exempt: Vec<Option<&PyValue>> = landed
        .tmsgs
        .iter()
        .filter(|m| m.contains_key("exempt_from_control_bound"))
        .map(|m| m.get("name"))
        .collect();
    let req: Vec<&&Dict> = probes.iter().filter(|m| name(m).to_lowercase().contains("request")).collect();
    let rep: Vec<&&Dict> = probes.iter().filter(|m| name(m).to_lowercase().contains("reply")).collect();
    let same = |a: Option<&PyValue>, b: Option<&PyValue>| match (a, b) {
        (Some(x), Some(y)) => x.py_eq(y),
        (None, None) => true,
        _ => false,
    };
    let only_request = req.len() == 1 && exempt.len() == 1 && same(exempt[0], req[0].get("name"));
    let shown: Vec<String> = exempt.iter().map(|n| n.map_or("None".into(), PyValue::py_repr)).collect();
    report.check("AC2", only_request, format!("exactly one message of wire.toml carries the exemption from C-17, the probe request (exempt messages {shown:?})"));
    let cites = req.len() == 1
        && req[0].get("exempt_from_control_bound").is_some_and(|v| v.eq_str("C-116"))
        && rep.len() == 1
        && !rep[0].contains_key("exempt_from_control_bound");
    report.check("AC2", cites, "the probe request's exemption cites C-116, and the probe reply carries no exemption");
    let no_code = rx(r"(?s)```.*?```").replace_all(&probe.text, "");
    let ex: Vec<String> = sentences(&no_code, false).into_iter().filter(|s| has_i("exempt", s)).collect();
    let allowed: BTreeSet<&str> = ["C-116", "C-17"].into();
    let only_116 = ex.iter().all(|s| s.contains("C-116") && rx(r"C-\d+").find_iter(s).all(|m| allowed.contains(m.as_str())));
    let ok = probe.found && !probe.text.to_lowercase().contains("to be filled") && !ex.is_empty() && only_116;
    report.check("AC2", ok, "every sentence of the section that says exempt cites C-116 and no other exemption");
}
