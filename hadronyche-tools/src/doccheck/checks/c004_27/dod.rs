//! DoD (document part): evidence tags on prose and on every subsection,
//! owned open items.

use super::messages::Probe;
use crate::doccheck::Report;
use crate::doccheck::evidence::{TagPolicy, subsections_untagged, untagged};
use crate::doccheck::items::{OpenItemPolicy, open_items, unowned_items};
use crate::doccheck::text::head;
use crate::doccheck::units::clean_start;

/// The DoD of the probe section.
pub(crate) fn dod(probe: &Probe, report: &mut Report) {
    let bad: Vec<String> = untagged(&probe.text, &TagPolicy::PROSE_004_27)
        .into_iter()
        .map(|u| head(&clean_start(&u.text), 60))
        .collect();
    let first: Vec<&String> = bad.iter().take(3).collect();
    report.check("DoD", probe.found && bad.is_empty(), format!("every prose paragraph of 5 or more words in the probe section carries an evidence tag; untagged: {first:?}"));
    let subs = subsections_untagged(&probe.subs);
    report.check("DoD", !probe.subs.is_empty() && subs.is_empty(), format!("every subsection of the probe section carries an evidence tag; untagged: {subs:?}"));
    let all = open_items(&probe.text, OpenItemPolicy::LeadingWithTicket).len();
    let unowned: Vec<String> = unowned_items(&probe.text, OpenItemPolicy::LeadingWithTicket)
        .into_iter()
        .map(|i| head(&i.text, 50))
        .collect();
    report.check("DoD", unowned.is_empty(), format!("every open item of the probe section names an owner and a falsifier ({all} found); without: {unowned:?}"));
}
