//! AC3 (the multi-packet regime and the hand-off to 004-25), WS5 (the
//! receive-timestamp note) and DoD (tags, the regime-stamp line) of 004-08.

use super::Ctx;
use crate::doccheck::Report;
use super::wp5::md::untagged_blocks;
use super::wp5::py::{head, py_strip, s, si};
use crate::doccheck::pycompat::splitlines;

/// AC3.
pub(crate) fn ac3(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    let p = &ctx.prose;
    if !si(r"packet\s+count\s*>\s*1|more\s+than\s+one\s+packet|multi-?packet", p) || !si("regime", p) {
        report.fail("AC3", "the multi-packet regime is not defined on the media side (packet count > 1 in the slice header, C-05)");
    }
    let mut seen: Option<String> = None;
    let mut ok = false;
    for (k, b) in ctx.blks.iter().enumerate() {
        if !s("004-25", b) {
            continue;
        }
        let window = ctx.blks[k.saturating_sub(1)..(k + 2).min(ctx.blks.len())].join("\n");
        let named = si(r"regime\s+stamp", &window)
            && si(r"stream\s+number", &window)
            && si(r"packets?\s+per\s+slice|packet\s+count", &window)
            && si(r"frames?\s+per\s+slice|slice\s+length", &window);
        if named {
            ok = true;
            break;
        }
        seen = Some(window);
    }
    if !ok {
        let found = seen.map_or(String::new(), |w| {
            format!("; the 004-25 mention found does not name them: {}", head(splitlines(py_strip(&w)).first().copied().unwrap_or(""), 80))
        });
        report.fail(
            "AC3",
            format!("no hand-off to 004-25 naming the regime-stamp values the multi-packet regime needs (at least stream number, packets per slice, frames per slice — or the author's documented set){found}"),
        );
    }
}

/// WS5.
pub(crate) fn ws5(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    let note = ctx.blks.iter().filter(|b| si(r"receive[\s_-]*timestamp", b)).cloned().collect::<Vec<_>>().join("\n");
    if py_strip(&note).is_empty() {
        report.fail("WS5", "no receive-timestamp note in the Slice header section (Work step 5)");
        return;
    }
    for (source, name) in [("kernel", "kernel"), ("userspace", "userspace"), (r"mac[\s_-]*hardware", "mac-hardware"), ("interrupt", "interrupt")] {
        if !si(source, &note) {
            report.fail("WS5", format!("the receive-timestamp note does not name the C-131 source {name:?}"));
        }
    }
    if !s("C-129", &note) || !si("capabilit", &note) {
        report.fail("WS5", "the receive-timestamp note does not tie the sources to C-129's capability states");
    }
    if !s("004-25", &note) || !si(r"regime\s+stamp", &note) {
        report.fail("WS5", "the receive-timestamp note does not point at the regime stamp (004-25) as the place the source and its capability state travel");
    }
    let not_in = r"not\s+(?:in|carried\s+in|part\s+of|the)\s+(?:the\s+)?slice\s+header|not\s+in\s+the\s+slice\s+header|never\s+in\s+the\s+slice\s+header|(?:not|never)[^.;\n]{0,60}slice\s+header";
    if !si(not_in, &note) {
        report.fail("WS5", "the receive-timestamp note does not state that the source is NOT in the slice header (it travels in the regime stamp, 004-25)");
    }
}

/// DoD: every block tagged; the regime-stamp-fields line covered or marked
/// not applicable with C-125.
pub(crate) fn dod(ctx: &Ctx, report: &mut Report) {
    if !ctx.open {
        return;
    }
    for first in untagged_blocks(&ctx.section) {
        let more = if first.chars().count() > 90 { " …" } else { "" };
        report.fail(
            "DoD",
            format!("this block of the Slice header section carries no evidence tag ([code] [measured] [kernel-doc] [standard] [memory] [decision]): {}{more}", head(&first, 90)),
        );
    }
    let blks = &ctx.blks;
    let covered = blks.iter().enumerate().any(|(k, b)| {
        if !si(r"regime\s+stamp", b) {
            return false;
        }
        let window = match blks.get(k + 1) {
            Some(n) => format!("{b}\n{n}"),
            None => b.clone(),
        };
        si(r"complet|documented|hand(?:ed|over|ing)|suppli", &window)
    });
    let na = blks.iter().any(|b| si(r"not\s+applicable|\bN/?A\b", b) && s("C-125", b));
    if !covered && !na {
        report.fail("DoD", "the regime-stamp-fields-touched line is neither covered (the handed fields stated complete and documented) nor marked not applicable with the reason (C-125)");
    }
}
