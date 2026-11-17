//! AC5 (the open item owned by 005 and the hand-off to the hello) and the
//! behaviour restated as fact, with no timer as a field (script lines
//! 713-743).

use crate::doccheck::wp7::common::md_name;
use crate::doccheck::wp7::common::is_open_item;
use crate::doccheck::wp7::py;
use super::Ctx;
use crate::doccheck::Report;

/// 004-17's `TIMER` (any case).
pub(crate) const TIMER: &str = r"timer|interval|timeout|time[\s-]*out|listen|window|silence|period|expir|deadline|backoff|delay|\bms\b|seconds?\b";

/// `re.search(r"(?<![\d.])" + re.escape(num) + r"\s*" + unit + r"\b",
/// text)`: every start is tried, and the look-behind refuses a start after
/// a digit or a point.
fn duration_stated(num: &str, unit: &str, text: &str) -> bool {
    let pat = format!(r"{}\s*{unit}\b", regex::escape(num));
    let mut from = 0;
    while from < text.len() {
        let Some(h) = py::search(&pat, &text[from..], false) else { return false };
        let start = from + h.start;
        let prev = text[..start].chars().next_back();
        if !prev.is_some_and(|c| c == '.' || crate::doccheck::rejection_members::digit_value(c).is_some()) {
            return true;
        }
        from = start + text[start..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// AC5 and Behaviour.
pub(crate) fn open_item_and_behaviour(ctx: &Ctx, report: &mut Report) {
    let owned = ctx.el_paras.iter().filter(|t| is_open_item(t)).any(|t| {
        py::has_i(r"owner[^.;]*?\b005\b", t) && py::has_i("falsifier", t) && py::has_i(r"needs\s+a\s+message\s+or\s+field", t)
    });
    if !owned {
        report.fail("AC5", "no paragraph starting 'Open item' names epic 005 as the confirming owner (the 'Owner' label) and the falsifier 'epic 005's election state machine needs a message or field the set lacks'");
    }
    if ctx.el_para(&["hello", r"election\s+epoch", r"time\s+state", "reuse|confirm|differ"]).is_none() {
        report.fail("AC5", "no sentence says that the election messages reuse or confirm the hello's election epoch width and time-state encoding, or states the difference");
    }
    let p = &ctx.el_plain;
    if !(py::has(r"TTL\s+1\b", p) && p.contains("C-100") && py::has_i("unicast", p) && py::has("IPv4", p)) {
        report.fail("Behaviour", "multicast with TTL 1 and unicast over IPv4 (C-100) is not restated as fact");
    }
    if let Some(d) = ctx.dscp_control
        && !(py::has(&format!(r"CS5\s*\({d}\)"), p) && p.contains("C-13"))
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
        && !py::has(&format!(r"\b{n}\b"), p)
    {
        report.fail("Behaviour", "C-34 is cited but its bound is not restated");
    }
    for m in &ctx.msgs {
        for f in &m.fields {
            let name = md_name(&f.meaning);
            if py::has_i(TIMER, name) {
                report.fail("Behaviour", format!("'{}': the field '{name}' is a timer; a timer is behaviour, not a field", m.title));
            }
        }
    }
}
