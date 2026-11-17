//! AC3: per message, the sender's module id, nonce and election epoch;
//! across the set, the time state, holder eligibility, believed holder,
//! winner's timeline epoch and the sentences about them (script lines
//! 549-620).

use std::collections::BTreeSet;

use crate::doccheck::wp7::md::Field;
use crate::doccheck::wp7::py;
use super::{Ctx, Msg};
use crate::doccheck::Report;

/// `row_in(fl, pat, bits)`.
fn row_in<'a>(fl: &'a [Field], pat: &str, bits: Option<i128>) -> Option<&'a Field> {
    fl.iter().find(|f| py::matches(&format!(r"\s*{pat}"), &f.meaning, true).is_some() && bits.is_none_or(|b| f.bits == b))
}

/// `{int(x) for x in re.findall(r"(?<![\w.])(\d{1,3})\s*=", text)}`.
pub(crate) fn stated_values(text: &str) -> BTreeSet<i128> {
    let chars: Vec<char> = text.chars().collect();
    let digit = |c: char| crate::doccheck::rejection_members::digit_value(c).is_some();
    let mut out = BTreeSet::new();
    let mut p = 0;
    while p < chars.len() {
        let free = p == 0 || !(py::py_word(chars[p - 1]) || chars[p - 1] == '.');
        if !digit(chars[p]) || !free {
            p += 1;
            continue;
        }
        let run = chars[p..].iter().take_while(|&&c| digit(c)).count();
        let mut q = p + run;
        while q < chars.len() && py::is_space(chars[q]) {
            q += 1;
        }
        if run <= 3 && chars.get(q) == Some(&'=') {
            let s: String = chars[p..p + run].iter().collect();
            out.extend(py::int(&s));
            p = q + 1;
        } else {
            p += 1;
        }
    }
    out
}

fn per_message(ctx: &Ctx, report: &mut Report) {
    for m in &ctx.msgs {
        let (t, fl) = (&m.title, &m.fields);
        if row_in(fl, r"module\s+id\b", Some(64)).is_none() {
            report.fail("AC3", format!("'{t}': no module id row of 64 bits (C-11)"));
        }
        if row_in(fl, r"instance\s+nonce\b", Some(64)).is_none() {
            report.fail("AC3", format!("'{t}': no instance nonce row of 64 bits (C-11)"));
        }
        match (row_in(fl, r"election\s+epoch\b", None), ctx.hello_epoch_bits) {
            (None, _) => report.fail("AC3", format!("'{t}': no election epoch row")),
            (Some(ep), Some(h)) if ep.bits != h => report.fail("AC3", format!("'{t}': the election epoch is {} bits; the hello's election epoch is {h} bits, and the election messages take the same width", ep.bits)),
            _ => {}
        }
    }
}

fn rows_of(ctx: &Ctx, pred: impl Fn(&Field) -> bool) -> Vec<(&Msg, &Field)> {
    ctx.msgs.iter().flat_map(|m| m.fields.iter().map(move |f| (m, f))).filter(|(_, f)| pred(f)).collect()
}

fn time_state(ctx: &Ctx, report: &mut Report) {
    let ts = rows_of(ctx, |f| py::matches(r"\s*time\s+state\b", &f.meaning, true).is_some());
    if ts.is_empty() {
        report.fail("AC3", "no election message carries a time state: the candidate's usable time (C-213) cannot be decided");
    }
    let mut kinds = BTreeSet::new();
    for (m, f) in &ts {
        if f.bits != 8 {
            report.fail("AC3", format!("'{}': the time state is {} bits; it is a one-byte field", m.title, f.bits));
        }
        let vals = stated_values(&format!("{} {}", f.meaning, f.valid));
        if vals == BTreeSet::from([0, 1, 2]) {
            kinds.insert("three");
        } else if [0, 1, 2, 3].iter().all(|v| vals.contains(v)) {
            kinds.insert("four");
        } else {
            report.fail("AC3", format!("'{}': the time state row does not state 'N = meaning' for the hello's values 0, 1, 2 or for four values 0 to 3", m.title));
        }
    }
    if kinds.len() > 1 {
        report.fail("AC3", "the election messages encode the time state two different ways; one encoding serves them all");
    }
    if !ts.is_empty() && ctx.el_para(&[r"time\s+state", "hello", "C-213"]).is_none() {
        report.fail("AC3", "no sentence relates the election time state to the hello's time state and cites C-213");
    }
    if kinds.contains("three") && ctx.el_para(&["C-162", "hello|three"]).is_none() {
        report.fail("AC3", "the hello's three-value time state is reused, but no sentence argues why three values suffice (C-162: the lapsed claim is Unlocked)");
    }
    if kinds.contains("four") && ctx.el_para(&[r"\bmap(s|ping|ped)?\b|\bread\s+as\b", "hello", "C-213"]).is_none() {
        report.fail("AC3", "the four-value time state has no sentence that maps it to the hello's three values");
    }
}

fn holder_rows(ctx: &Ctx, report: &mut Report) {
    let elig = rows_of(ctx, |f| py::matches(r"\s*holder[\s-]*eligib", &f.meaning, true).is_some());
    if elig.is_empty() {
        report.fail("AC3", "no election message carries holder eligibility (a bit consistent with the hello's capability bit 0)");
    }
    for (m, f) in &elig {
        if !format!("{} {}", f.meaning, f.valid).to_lowercase().contains("bit 0") {
            report.fail("AC3", format!("'{}': holder eligibility does not name bit 0, the hello's holder-eligible bit", m.title));
        }
        if f.bits != 1 && f.bits != 8 {
            report.fail("AC3", format!("'{}': holder eligibility is {} bits; a bit or a one-byte field", m.title, f.bits));
        }
    }
    if rows_of(ctx, |f| py::matches(r"\s*believed\s+holder\b", &f.meaning, true).is_some() && f.bits == 64).is_empty() {
        report.fail("AC3", "no election message carries the believed holder (module id, 64 bits)");
    }
    let winner = rows_of(ctx, |f| py::has_i(r"\bwinner", &f.meaning) && f.meaning.to_lowercase().contains("timeline epoch") && f.bits == 16);
    if winner.is_empty() {
        report.fail("AC3", "no election message carries the winner's timeline epoch (16 bits, C-10)");
    }
}

fn sentences(ctx: &Ctx, report: &mut Report) {
    if ctx.el_para(&[r"timeline\s+epoch", "C-213", "kept|keeps", "bump", "C-113", "lowest"]).is_none() {
        report.fail("AC3", "no sentence restates as fact that the winner's timeline epoch is kept when the winner has usable time (C-213) and bumped when no candidate has usable time, the lowest module id winning (C-113)");
    }
    if ctx.el_para(&[r"election\s+epoch", "compared", "unsigned", "hello"]).is_none() {
        report.fail("AC3", "no sentence says that the election epoch is compared unsigned, as in the hello");
    }
    let names_all = |p: &str| ctx.msgs.iter().all(|m| py::has_i(&regex::escape(&m.title), p));
    if !ctx.el_paras.iter().any(|p| py::has_i("carr", p) && names_all(p)) {
        report.fail("AC3", "no sentence says which election message carries which field: a paragraph with 'carries' must name every election message");
    }
}

/// AC3.
pub(crate) fn fields(ctx: &Ctx, report: &mut Report) {
    per_message(ctx, report);
    if ctx.msgs.is_empty() {
        return;
    }
    time_state(ctx, report);
    holder_rows(ctx, report);
    sentences(ctx, report);
}
