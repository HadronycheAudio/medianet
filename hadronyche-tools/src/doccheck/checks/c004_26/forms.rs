//! AC4 (the join-delay bound is an open item with an owner and a
//! falsifier) and AC5 (the invalid forms, their categories, the repeat and
//! disagreement rows).

use super::fields::Man;
use super::landed::Landed;
use crate::doccheck::Report;
use crate::doccheck::text::{has_i, head, rx};
use crate::doccheck::units::BlockKind;

/// `OPEN_PARAS`: prose blocks whose first 40 characters, after `*_`` are
/// removed and the text stripped, contain "open item".
pub(crate) fn open_paras(man: &Man) -> Vec<String> {
    man.prose
        .iter()
        .filter(|b| b.kind == BlockKind::Prose)
        .filter(|b| {
            let t = rx(r"[*_`]").replace_all(&b.text, "");
            has_i("open item", &head(t.trim(), 40))
        })
        .map(|b| b.text.clone())
        .collect()
}

/// AC4; returns the join-delay open items.
pub(crate) fn ac4(man: &Man, report: &mut Report) -> Vec<String> {
    let join: Vec<String> = open_paras(man).into_iter().filter(|p| has_i(r"\bjoin", p) && has_i(r"bound|maximum|\bmax\b", p)).collect();
    report.check("AC4", !join.is_empty(), "an open item records the join-delay bound");
    report.check("AC4", join.iter().any(|p| has_i(r"\b(epic\s*)?004\b", p)), "the join-delay open item is owned by epic 004");
    join
}

fn is_repeat_row(f: &str) -> bool {
    has_i("section index", f) && has_i("identical|same|repeat|duplicate", f)
}

/// The members of `names` that `outcome` names as a whole word.
fn named<'a>(names: impl Iterator<Item = &'a String>, outcome: &str) -> Vec<&'a String> {
    names.filter(|n| rx(&format!(r"\b{}\b", regex::escape(n))).is_match(outcome)).collect()
}

/// AC5.
pub(crate) fn ac5(man: &Man, landed: &Landed, report: &mut Report) {
    let forms = &man.forms;
    report.check("AC5", forms.len() >= 5, format!("at least five invalid forms are tabled ({} found)", forms.len()));
    let all = landed.all_reasons();
    for (f, o) in forms.iter().filter(|(f, _)| !is_repeat_row(f)) {
        let ok = !named(all.iter().copied(), o).is_empty();
        report.check("AC5", ok, format!("the invalid form '{}' names its outcome as a member of the Reject or refusal enumeration", head(f, 50)));
    }
    let cat = |p: &dyn Fn(&str, &str) -> bool| forms.iter().any(|(f, o)| p(f, o));
    report.check("AC5", cat(&|f, _| has_i("count", f) && has_i("above|more than|exceed|maximum|greater", f)), "a count above its maximum is an invalid form");
    report.check("AC5", cat(&|f, o| has_i("length", f) && has_i("inconsistent|does not fill|not fill", f) && o.contains("BadLength")), "a length inconsistent with the sections is an invalid form with outcome BadLength");
    report.check("AC5", cat(&|f, _| has_i("channel[- ]map", f) && has_i("range|duplicat|distinct|twice", f)), "a channel-map entry out of range or duplicated is an invalid form");
    report.check("AC5", cat(&|f, _| has_i("device", f) && has_i("conversion", f)), "a conversion delay on a port that is not device-facing is an invalid form");
    report.check("AC5", cat(&|f, _| has_i("reserved", f)), "a non-zero reserved field is an invalid form (Work item 3)");
    let dup: Vec<&(String, String)> = forms.iter().filter(|(f, _)| is_repeat_row(f)).collect();
    report.check("AC5", !dup.is_empty(), "an identical repeated section has an invalid-form row (its outcome is stated)");
    for (_, o) in &dup {
        let n = named(landed.reject.keys(), o);
        let ok = n.is_empty() && has_i("not a rejection|never a rejection|ignored|valid|kept", o);
        report.check("AC5", ok, format!("the identical repeated section is not a rejection: its outcome names no Reject reason and keeps the section already held (names found: {n:?})"));
    }
    let dis: Vec<&(String, String)> = forms.iter().filter(|(f, _)| has_i("disagree|differ|different|conflict", f) && has_i("section count|manifest length", f)).collect();
    report.check("AC5", dis.iter().any(|(f, _)| has_i("manifest length", f)), "sections that disagree on the manifest length have an invalid-form row");
    report.check("AC5", dis.iter().any(|(f, _)| has_i("section count", f)), "sections that disagree on the section count have an invalid-form row");
    for (f, o) in &dis {
        report.check("AC5", named(landed.reject.keys(), o).len() == 1, format!("the disagreement row '{}' names exactly one Reject reason", head(f, 40)));
    }
}
