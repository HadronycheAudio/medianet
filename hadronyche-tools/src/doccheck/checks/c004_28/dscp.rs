//! AC2 (Work 1): the observed DSCP of peers section.

use super::Consts;
use super::analyse::Analysis;
use crate::doccheck::checked;
use crate::doccheck::Report;
use crate::doccheck::family004::Field;
use crate::doccheck::pycompat::parse_int;
use crate::doccheck::text::{has, has_i, rx};

pub(super) fn not_observed_codes(valid: &str) -> Vec<i128> {
    let v = rx(r"C-\d+").replace_all(valid, "");
    v.split(';').filter(|cl| has_i("not observed", cl)).flat_map(|cl| rx(r"\b\d+\b").find_iter(cl).filter_map(|m| parse_int(m.as_str()).ok()).collect::<Vec<i128>>()).collect()
}

fn encoding(ent: &[&Field], report: &mut Report) -> bool {
    let notobs: Vec<&&Field> = ent.iter().filter(|f| has_i("not observed", &format!("{} {}", f.meaning, f.valid))).collect();
    let sentinel = notobs.iter().any(|f| {
        let c = not_observed_codes(&f.valid);
        !c.is_empty() && c.iter().all(|&x| x > 63)
    });
    let collide = notobs.iter().any(|f| not_observed_codes(&f.valid).iter().any(|&x| x <= 63));
    let flag = ent.iter().any(|f| has_i("observed", &f.meaning) && f.bits == 1 && has(r"\b0 or 1\b|\b0 to 1\b|\b0/1\b", &f.valid));
    let state: Vec<&&Field> = ent.iter().filter(|f| has("C-129", &f.meaning) && has_i(r"\bstate", &f.meaning)).collect();
    for f in &state {
        let counts: Vec<i128> = rx(r"(\d+) to (\d+)")
            .captures_iter(&f.valid)
            .filter_map(|c| Some((parse_int(&c[1]).ok()?, parse_int(&c[2]).ok()?)))
            .filter(|(a, _)| *a >= 1)
            .map(|(a, b)| checked::add(checked::sub(b, a), 1))
            .collect();
        report.check("AC2", counts == [4], format!("the C-129 state field enumerates four states (C-129), found {counts:?}"));
    }
    let state_dscp = ent.iter().any(|f| has("DSCP", &f.meaning) && has_i("zero|present", &format!("{} {}", f.meaning, f.valid)));
    !collide && (sentinel || flag || (!state.is_empty() && state_dscp))
}

/// AC2; returns `DSCP_WIDE_FLAG`.
pub(crate) fn ac2(d: Option<&Analysis>, consts: &Consts, report: &mut Report) -> bool {
    let Some(d) = d.filter(|d| d.ok) else {
        report.fail("AC2", "the observed DSCP section is missing: the peer id, the hello and sync DSCP, the not-observed encoding and the dropped-request counter cannot be checked");
        return false;
    };
    let ent = d.entry_fields();
    let like = |pat: &str| -> Vec<&Field> { ent.iter().copied().filter(|f| has_i(pat, &f.meaning)).collect() };
    report.check("AC2", like("module id").iter().any(|f| f.bits == 64), "the observed DSCP entry names its peer by a 64-bit module id (Work 1, C-11)");
    let hello = like("hello");
    report.check("AC2", !hello.is_empty() && hello.iter().any(|f| has(r"\b63\b", &f.valid)), "the observed DSCP entry carries the DSCP observed on the peer's hellos, valid 0 to 63 (C-152, every module)");
    report.check("AC2", like("sync").iter().any(|f| has_i("holder", &f.meaning)), "the observed DSCP entry carries the DSCP observed on sync requests, holder only (C-152)");
    let dropped = like("drop");
    let hi = consts.accept_hi.map_or("None".to_string(), |v| v.to_string());
    let rate = format!(r"\b{hi}\b|C-204");
    report.check("AC2", dropped.iter().any(|f| has(&rate, &format!("{} {}", f.meaning, f.valid))), format!("the dropped-request counter names the accepted rate of {hi} per second (C-204, C-112)"));
    report.check("AC2", dropped.iter().any(|f| f.bits != 0), "the dropped-request counter has a width (C-204)");
    report.check("AC2", dropped.iter().any(|f| has_i("request|per second", &f.meaning)), "the dropped-request counter has a unit: requests (C-204)");
    let one_rule = dropped.iter().any(|f| {
        let mut kinds: Vec<String> = rx(r"(?i)wrap|saturat").find_iter(&format!("{} {}", f.meaning, f.valid)).map(|m| m.as_str().to_lowercase().chars().take(4).collect()).collect();
        kinds.sort();
        kinds.dedup();
        kinds.len() == 1
    });
    report.check("AC2", one_rule, "the dropped-request counter states exactly one wrap or saturation rule (Work 1)");
    let ok = encoding(&ent, report);
    report.check("AC2", ok, "the not-observed encoding is stated and told from an observed DSCP 0 to 63 (a sentinel, a one-bit flag, or a C-129 state field)");
    report.check("AC2", ent.iter().any(|f| has("DSCP", &f.meaning) && f.valid.contains("0 to 63")), "the DSCP proper is 0 to 63, six bits (C-13), in the observed DSCP entry");
    let wide = ent.iter().any(|f| has("DSCP", &f.meaning) && f.bits >= 7);
    report.check("AC2", has_i("order", &d.text), "the entry order of the observed DSCP section is stated");
    report.check("AC2", has("C-34|C-114|C-217", &d.text), "the observed DSCP entry count derives from the constants (C-34, C-114)");
    if let Some(&f) = d.lists().first()
        && let Some(s) = d.stmts.iter().find(|s| s.1 == f) {
            report.check("AC2", Some(s.0) == consts.max_peers, format!("the observed DSCP section holds at most {} entries, one per peer (C-34, C-114): the text states {}", consts.max_peers.map_or("None".to_string(), |p| p.to_string()), s.0));
        }
    report.check("DoD", d.text.contains("sender rule 3"), "hand-off 004-28: the dropped-request counter falls under sender rule 3 (any byte change is a new version)");
    wide
}
