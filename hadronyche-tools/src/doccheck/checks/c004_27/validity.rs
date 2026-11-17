//! AC3: the invalid forms of the probe's validity rules and their
//! outcomes.

use crate::doccheck::pycompat::PyStr;
use super::landed::Landed;
use super::messages::Probe;
use crate::doccheck::family004::{sentences, tables};
use crate::doccheck::Report;
use crate::doccheck::text::{has, has_i, rx};

type Form = (String, String);

/// The (invalid form, outcome) rows of every validity subsection: tables
/// whose first header cell matches "form" or "invalid".
fn rows(probe: &Probe) -> Vec<Form> {
    let mut out = Vec::new();
    for (_, x) in probe.subs.iter().filter(|(t, _)| has_i("validity", t)) {
        for t in tables(x) {
            if t.head().len() >= 2 && has_i("form|invalid", &t.head()[0]) {
                for r in &t.rows {
                    if r.cells.len() >= 2 && !r.cell(0).py_strip().is_empty() {
                        out.push((r.cell(0).py_strip().to_string(), r.cell(1).py_strip().to_string()));
                    }
                }
            }
        }
    }
    out
}

/// The reason tokens of an outcome (004-27's pattern, which differs from
/// most scripts': `[a-z0-9]` after the capital, any letters after).
fn tokens(s: &str) -> Vec<String> {
    rx(r"\b[A-Z][a-z0-9]+(?:[A-Z][A-Za-z0-9]*)+\b").find_iter(s).map(|m| m.as_str().to_string()).collect()
}

fn ctrl_reason(o: &str, landed: &Landed) -> bool {
    tokens(o).iter().any(|t| landed.layers.get(t).is_some_and(|l| l.contains("control")))
}

fn names_and_layers(forms: &[Form], landed: &Landed, report: &mut Report) {
    let (mut bad_names, mut bad_layer) = (Vec::new(), Vec::new());
    for (_, o) in forms {
        for t in tokens(o) {
            match landed.layers.get(&t) {
                Some(l) if !l.contains("control") => bad_layer.push(format!("{t} (Layers: {l})")),
                Some(_) => {}
                None if !landed.refusals.contains_key(&t) => bad_names.push(t),
                None => {}
            }
        }
    }
    report.check("AC3", !forms.is_empty() && bad_names.is_empty(), format!("every outcome names a member of the Rejection reasons or the refusal enumeration; not members: {bad_names:?}"));
    report.check("AC3", !forms.is_empty() && bad_layer.is_empty(), format!("every packet reason named by an outcome lists the control layer in its Layers cell; {bad_layer:?}"));
}

fn classes(forms: &[Form], probe: &Probe, landed: &Landed, report: &mut Report) {
    let size: Vec<&Form> = forms.iter().filter(|(f, _)| has_i("intended", f) && has_i("size", f) && !has_i("pad", f)).collect();
    let pad: Vec<&Form> = forms.iter().filter(|(f, _)| has_i("pad", f) && has_i(r"non-?\s?zero|not\s+zero|other\s+than\s+zero|\bnonzero\b", f)).collect();
    let reply: Vec<&Form> = forms.iter().filter(|(f, _)| has_i("reply", f) && has_i(r"outstanding|match|no\s+probe|unmatched", f)).collect();
    let generic_re = r"magic|message\s+type|domain|scheme|\btag\b|oversize|too\s+short|version|counter|datagram|unassigned";
    let generic: Vec<&Form> = forms
        .iter()
        .filter(|r| has_i(generic_re, &r.0) && !size.contains(r) && !pad.contains(r) && !reply.contains(r))
        .collect();
    report.check("AC3", !size.is_empty() && size.iter().all(|(_, o)| ctrl_reason(o, landed)), "a probe size different from its declared intended size has an outcome naming a control-layer reason");
    report.check("AC3", !pad.is_empty() && pad.iter().all(|(_, o)| o.contains("ReservedNonZero")), "non-zero padding is dropped and counted under ReservedNonZero");
    let reply_ok = |o: &str| {
        ctrl_reason(o, landed) || (has_i("not a parse rejection|not a rejection", o) && has_i(r"\d{3}-\d{2}|epic", o))
    };
    report.check("AC3", !reply.is_empty() && reply.iter().all(|(_, o)| reply_ok(o)), "a reply that matches no outstanding probe has an outcome: a control-layer reason, or not a rejection with its owner");
    let len_row = forms.iter().any(|(f, o)| {
        has_i("reply", f) && has_i("size|length|datagram|bytes", f) && o.contains("BadLength") && has_i(r"other\s+than|differs|not\s+the|does\s+not", f)
    });
    let len_sent = sentences(&probe.text, true).iter().any(|x| {
        has_i("reply", x)
            && x.contains("BadLength")
            && has_i(r"fixed\s+size|length", x)
            && has(r"shared|Validity\s+rules|Hello|Admission\s+messages|Rejection\s+reasons", x)
    });
    report.check("AC3", len_row || len_sent, "a reply whose length differs from its fixed size is dropped and counted under BadLength, by a row or by the shared rule the section cites");
    report.check("AC3", !generic.is_empty() && generic.iter().all(|(_, o)| ctrl_reason(o, landed)), "the generic control violations (magic, message type, domain, scheme or tag, oversize, too short) have control-layer outcomes");
}

/// AC3.
pub(crate) fn ac3(probe: &Probe, landed: &Landed, report: &mut Report) {
    let forms = rows(probe);
    report.check("AC3", forms.len() >= 4 && forms.iter().all(|(_, o)| !o.is_empty()), format!("the probe's validity rules give at least 4 invalid forms, each with an outcome (found {})", forms.len()));
    names_and_layers(&forms, landed, report);
    classes(&forms, probe, landed, report);
}
