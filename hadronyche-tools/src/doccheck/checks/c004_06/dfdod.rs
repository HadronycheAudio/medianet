//! The don't-fragment statement (DF) and the evidence tags (DoD) of 004-06.

use super::Ctx;
use super::wp5::md::untagged_blocks;
use super::wp5::py::{head, s, si};
use crate::doccheck::Report;

/// DF: the don't-fragment requirement as an IPv4 DF-bit sender obligation
/// outside the header, citing C-100.
pub(crate) fn df(ctx: &Ctx, report: &mut Report) {
    if !ctx.open() {
        return;
    }
    let p = &ctx.prose;
    if !si(r"don'?t-?fragment|do\s+not\s+fragment|\bDF\b", p) {
        report.fail("DF", "the section does not state the don't-fragment requirement");
    }
    if !s("IPv4", p) {
        report.fail("DF", "the don't-fragment statement does not name IPv4 (C-100: IPv4 only in v1)");
    }
    if !si("sender", p) || !si(r"obligat|must|sets?|require|duty", p) {
        report.fail("DF", "the don't-fragment statement does not name it as a sender obligation");
    }
    let outside = r"outside\s+the\s+header|not\s+(?:a\s+)?(?:part\s+of\s+the\s+)?header\s+field|not\s+in\s+the\s+header|beyond\s+the\s+header|header\s+does\s+not\s+carry|not\s+part\s+of\s+the\s+header|outside\s+this\s+header";
    if !si(outside, p) {
        report.fail("DF", "the don't-fragment statement does not place the DF bit outside the header");
    }
    if !s("C-100", p) {
        report.fail("DF", "the don't-fragment statement does not cite C-100");
    }
}

/// DoD: every block of the section carries a tag (a table or code block
/// inherits the previous block's).
pub(crate) fn dod(ctx: &Ctx, report: &mut Report) {
    if !ctx.open() {
        return;
    }
    for first in untagged_blocks(&ctx.section) {
        let more = if first.chars().count() > 90 { " …" } else { "" };
        report.fail(
            "DoD",
            format!(
                "this block of the Media packet header section carries no evidence tag ([code] [measured] [kernel-doc] [standard] [memory] [decision]): {}{more}",
                head(&first, 90)
            ),
        );
    }
}
