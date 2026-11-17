//! DoD: evidence tags on prose and on the subsection of every table; open
//! items name an owner and a falsifier (script lines 1298-1323).

use crate::doccheck::wp7::common::{is_open_item, words};
use crate::doccheck::wp7::md::{has_evidence, prose_blocks};
use super::{Ctx, py};
use crate::doccheck::Report;

/// The DoD tag and open-item rules over the section's top text and every
/// `###` subsection.
pub(crate) fn tags(ctx: &Ctx, report: &mut Report) {
    let top: Vec<&str> = ctx
        .hello_lines
        .iter()
        .take_while(|l| py::matches(r"###\s", l, false).is_none())
        .map(String::as_str)
        .collect();
    let top = top.join("\n");
    let parts = std::iter::once(("(section)", top.as_str())).chain(ctx.subs.iter().map(|(t, x)| (t.as_str(), x.as_str())));
    for (title, text) in parts {
        let tagged = has_evidence(text);
        for (kind, body) in prose_blocks(text) {
            let first = py::head(py::splitlines(&body).first().copied().unwrap_or(""), 80);
            if kind == 'p' {
                if words(&body) >= 5 && !has_evidence(&body) {
                    report.fail("DoD", format!("a prose paragraph of '{title}' carries no evidence tag: {first}"));
                }
                if is_open_item(&body) && (!py::has_i("owner", &body) || !py::has_i("falsifier", &body)) {
                    report.fail("DoD", format!("an open item in '{title}' names no owner or no falsifier: {first}"));
                }
            } else if !tagged {
                report.fail("DoD", format!("a table of '{title}' follows no evidence-tagged paragraph"));
            }
        }
    }
}
