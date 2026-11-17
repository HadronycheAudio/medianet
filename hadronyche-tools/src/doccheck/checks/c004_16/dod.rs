//! DoD: evidence tags on prose and on the subsection of every table; open
//! items name an owner and a falsifier (script lines 1298-1323).

use super::Ctx;
use crate::doccheck::Report;
use crate::doccheck::evidence::{TableTagRule, TagPolicy, untagged, untagged_tables};
use crate::doccheck::items::{OpenItemPolicy, unowned_items};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::{head, starts};

fn first_line(text: &str) -> String {
    head(splitlines(text).first().copied().unwrap_or(""), 80)
}

/// The DoD tag and open-item rules over the section's top text and every
/// `###` subsection.
pub(crate) fn tags(ctx: &Ctx, report: &mut Report) {
    let top: Vec<&str> = ctx.hello_lines.iter().take_while(|l| !starts(r"###\s", l)).map(String::as_str).collect();
    let top = top.join("\n");
    let parts = std::iter::once(("(section)", top.as_str())).chain(ctx.subs.iter().map(|(t, x)| (t.as_str(), x.as_str())));
    for (title, text) in parts {
        for u in untagged(text, &TagPolicy::ALPHA_004_16) {
            report.fail("DoD", format!("a prose paragraph of '{title}' carries no evidence tag: {}", first_line(&u.text)));
        }
        for item in unowned_items(text, OpenItemPolicy::LeadingClean) {
            report.fail("DoD", format!("an open item in '{title}' names no owner or no falsifier: {}", first_line(&item.text)));
        }
        for _ in untagged_tables(text, TableTagRule::SectionTagged004_16) {
            report.fail("DoD", format!("a table of '{title}' follows no evidence-tagged paragraph"));
        }
    }
}
