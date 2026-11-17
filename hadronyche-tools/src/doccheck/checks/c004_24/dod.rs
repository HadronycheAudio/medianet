//! DoD (document part): evidence tags on every prose line
//! ([`LinePolicy::LINES_004_24`]) and every `###` subsection, owned open
//! items ([`OpenItemPolicy::MentionBlocks`]).

use super::stamp::Stamp;
use crate::doccheck::Report;
use crate::doccheck::evidence::{LinePolicy, subsections_untagged, untagged_lines};
use crate::doccheck::items::{OpenItemPolicy, open_items, unowned_items};
use crate::doccheck::text::head;

/// The DoD of the regime stamp section.
pub(crate) fn dod(st: &Stamp, report: &mut Report) {
    let bad: Vec<String> = untagged_lines(&st.text, &LinePolicy::LINES_004_24).iter().map(|t| head(t, 60)).collect();
    let first: Vec<&String> = bad.iter().take(3).collect();
    report.check("DoD", bad.is_empty(), format!("every prose paragraph of 5 or more words in the regime stamp section carries an evidence tag; untagged: {first:?}"));
    let subs = subsections_untagged(&st.subs);
    report.check("DoD", subs.is_empty(), format!("every subsection of the regime stamp section carries an evidence tag; untagged: {subs:?}"));
    let n = open_items(&st.text, OpenItemPolicy::MentionBlocks).len();
    let unowned: Vec<String> = unowned_items(&st.text, OpenItemPolicy::MentionBlocks).iter().map(|i| head(&i.text, 50)).collect();
    report.check("DoD", unowned.is_empty(), format!("every Open item paragraph of the regime stamp section names an owner and a falsifier ({n} found); without: {unowned:?}"));
}
