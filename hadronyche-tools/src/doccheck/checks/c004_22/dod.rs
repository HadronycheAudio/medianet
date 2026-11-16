//! DoD (document part): evidence tags, owned open items and no main ref in
//! the Traceability section, the audit and the review record.

use super::audit::Audit;
use super::record::Record;
use super::{Inputs, Trace};
use crate::doccheck::Report;
use crate::doccheck::evidence::{TagPolicy, untagged};
use crate::doccheck::items::{OpenItemPolicy, unowned_items};
use crate::doccheck::text::{head, rx};

/// The DoD of the three new scopes.
pub fn dod(inp: &Inputs, trace: Option<&Trace>, audit: &Audit, rec: Option<&Record>, report: &mut Report) {
    let mut scopes: Vec<(&str, String)> = Vec::new();
    if let Some(t) = trace {
        scopes.push(("Traceability", inp.wire.text(t.span)));
    }
    if audit.found {
        scopes.push(("audit", audit.text.clone()));
    }
    if let Some(r) = rec {
        scopes.push(("review record", r.text.clone()));
    }
    for (label, text) in scopes {
        if rx(r"`(origin/)?main`|origin/main|\bmain branch\b").is_match(&text) {
            report.fail("DoD", format!("the {label} text refers to a main ref"));
        }
        for u in untagged(&text, &TagPolicy::UNITS_004_22) {
            report.fail("DoD", format!("an untagged paragraph of 5+ words in the {label}: '{}'", head(&u.text, 80)));
        }
        for i in unowned_items(&text, OpenItemPolicy::Mention) {
            report.fail("DoD", format!("an open item in the {label} names no owner or no falsifier: '{}'", head(&i.text, 80)));
        }
    }
}
