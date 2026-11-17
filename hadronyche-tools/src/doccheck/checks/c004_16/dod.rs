//! DoD: evidence tags on prose and on the subsection of every table; open
//! items name an owner and a falsifier (script lines 1298-1323).

use super::md::{clean_start, has_evidence, prose_blocks};
use super::{Ctx, py};
use crate::doccheck::Report;

/// The words of a paragraph as 004-16 to 004-18 count them: `[A-Za-z]{2,}`
/// outside code spans.
pub(crate) fn words(body: &str) -> usize {
    py::finditer("[A-Za-z]{2,}", &py::sub("`[^`]*`", "", body, false), false).len()
}

/// `re.match(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(body), re.I)`.
pub(crate) fn is_open_item(body: &str) -> bool {
    py::matches(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(body), true).is_some()
}

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
