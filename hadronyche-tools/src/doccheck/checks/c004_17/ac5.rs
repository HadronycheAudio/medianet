//! AC5 (the open item owned by 005 and the hand-off to the hello) and the
//! behaviour restated as fact, with no timer as a field (script lines
//! 713-743).

use super::Ctx;
use crate::doccheck::Report;
use crate::doccheck::family004::md_name;
use crate::doccheck::text::{has, has_i, rx};
use crate::doccheck::units::clean_start;

/// 004-17's `TIMER` (any case).
pub(crate) const TIMER: &str = r"timer|interval|timeout|time[\s-]*out|listen|window|silence|period|expir|deadline|backoff|delay|\bms\b|seconds?\b";

/// `re.search(r"(?<![\d.])" + re.escape(num) + r"\s*" + unit + r"\b", text)`.
fn duration_stated(num: &str, unit: &str, text: &str) -> bool {
    has(&format!(r"(?<![\d.]){}\s*{unit}\b", regex::escape(num)), text)
}

/// AC5 and Behaviour.
pub(crate) fn open_item_and_behaviour(ctx: &Ctx, report: &mut Report) {
    let opens = ctx.el_paras.iter().filter(|t| rx(r"(?i)^\s*\*{0,2}\s*open\s+items?\b").is_match(&clean_start(t)));
    let owned = opens.into_iter().any(|t| {
        has_i(r"owner[^.;]*?\b005\b", t) && has_i("falsifier", t) && has_i(r"needs\s+a\s+message\s+or\s+field", t)
    });
    if !owned {
        report.fail("AC5", "no paragraph starting 'Open item' names epic 005 as the confirming owner (the 'Owner' label) and the falsifier 'epic 005's election state machine needs a message or field the set lacks'");
    }
    if ctx.el_para(&["hello", r"election\s+epoch", r"time\s+state", "reuse|confirm|differ"]).is_none() {
        report.fail("AC5", "no sentence says that the election messages reuse or confirm the hello's election epoch width and time-state encoding, or states the difference");
    }
    let p = &ctx.el_plain;
    if !(has(r"TTL\s+1\b", p) && p.contains("C-100") && has_i("unicast", p) && has("IPv4", p)) {
        report.fail("Behaviour", "multicast with TTL 1 and unicast over IPv4 (C-100) is not restated as fact");
    }
    if let Some(d) = ctx.dscp_control
        && !(has(&format!(r"CS5\s*\({d}\)"), p) && p.contains("C-13"))
    {
        report.fail("Behaviour", "CS5 (40) with C-13 is not restated as fact");
    }
    for (_, cid, d) in &ctx.dur {
        let Some((num, unit)) = d else { continue };
        if !(duration_stated(num, unit, p) && p.contains(cid)) {
            report.fail("Behaviour", format!("{cid} ({num} {unit}) is not restated as behaviour in the section"));
        }
    }
    if p.contains("C-34")
        && let Some(n) = ctx.peers_max
        && !has(&format!(r"\b{n}\b"), p)
    {
        report.fail("Behaviour", "C-34 is cited but its bound is not restated");
    }
    for m in &ctx.msgs {
        for f in &m.fields {
            let name = md_name(&f.meaning);
            if has_i(TIMER, name) {
                report.fail("Behaviour", format!("'{}': the field '{name}' is a timer; a timer is behaviour, not a field", m.title));
            }
        }
    }
}
