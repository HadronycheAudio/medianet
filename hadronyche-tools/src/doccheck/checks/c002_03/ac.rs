//! AC1, AC2, AC4, the DoD and the section marker of 002-03, each as the
//! script computes it.

use super::Subs;
use crate::doccheck::Report;
use crate::doccheck::family002::rules::{
    PtrRule, SectionVerdict, TagSet, VerdictForm, has_ptr, has_tag, section_verdict,
    unowned_open_items, untagged_pointer_lines,
};
use crate::doccheck::family002::{Grep, Sed, bash_test_int, distinct_count, tr_lower};

const TAGS: TagSet = TagSet::Five;
const PTR: PtrRule = PtrRule::Plain;

/// `has_marker`: the ASCII-lowercased block says "not in these files" (or a
/// variant).
fn has_marker(block: &str) -> bool {
    Grep::ere(r"not in (these|the two|either) files?|not found in (these|either)|absent from (these|both)")
        .any_line(&tr_lower(block))
}

fn check(r: &mut Report, c: &str, ok: bool, pass: String, fail: String) {
    r.check(c, ok, if ok { pass } else { fail });
}

fn pass(r: &mut Report, c: &str, msg: impl Into<String>) {
    r.check(c, true, msg);
}

/// AC1, per sub-section: at most 30 non-empty lines; at least 3 distinct
/// pointers; at least 2 distinct pointers to its own file.
pub(crate) fn ac1(s: &Subs, r: &mut Report) {
    for (label, (start, end), own) in [("transport.rs", s.t, "transport"), ("control.rs", s.c, "control")] {
        let sec = s.lines.line_range(start, end);
        let nonempty = Grep::bre("[^[:space:]]").count_lines(&sec);
        check(
            r,
            "AC1",
            nonempty <= 30,
            format!("{label}: compact ({nonempty} non-empty lines, half-page proxy limit 30)"),
            format!("{label}: too long ({nonempty} non-empty lines, half-page proxy limit 30)"),
        );
        let nptr = distinct_count(Grep::ere(PTR.pattern()).only_lines(&sec));
        check(
            r,
            "AC1",
            nptr >= 3,
            format!("{label}: {nptr} distinct file:line pointers (need >= 3)"),
            format!("{label}: {nptr} distinct file:line pointer(s), need at least 3"),
        );
        let nown = distinct_count(Grep::ere(&format!(r"{own}\.rs:[0-9]+")).only_lines(&sec));
        check(
            r,
            "AC1",
            nown >= 2,
            format!("{label}: {nown} distinct {own}.rs pointers (need >= 2 to its own file)"),
            format!("{label}: {nown} distinct {own}.rs pointer(s), need at least 2 to its own file"),
        );
    }
}

/// The outcome of `fact_scan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fact {
    None,
    Answer(usize),
    Notin(usize),
    AnswerNotag(usize),
    NotinNotag(usize),
}

/// `fact_scan`: over both sub-sections, each line matching the anchor: its
/// block (it and the next line, clipped) answers when it has a pointer
/// and every needle (tagged: done; untagged: remembered, scan on);
/// otherwise a not-in-files marker (tagged: done; untagged: remembered).
fn fact_scan(s: &Subs, anchor: &str, needles: &[&str]) -> Fact {
    let anchor = Grep::ere_i(anchor);
    let mut state = Fact::None;
    for (start, end) in [s.t, s.c] {
        for k in start..=end {
            if !anchor.matches(s.lines.sed_line(k)) {
                continue;
            }
            let block = s.lines.line_range(k, (k + 1).min(end));
            if has_ptr(&block, PTR) && needles.iter().all(|n| Grep::ere_i(n).any_line(&block)) {
                if has_tag(&block, TAGS) {
                    return Fact::Answer(k);
                }
                state = Fact::AnswerNotag(k);
                continue;
            }
            if has_marker(&block) {
                if has_tag(&block, TAGS) {
                    return Fact::Notin(k);
                }
                state = Fact::NotinNotag(k);
            }
        }
    }
    state
}

const FACTS: [(&str, &str, &[&str]); 4] = [
    (
        "who acts as transport reference",
        r"who acts[^.]*reference|transport[ -]reference|reference (role|holder)|(sender|synth)[^.]*(is|acts as|serves as|defaults? to)[^.]*reference|reference[^.]{0,30}(is|equals)[^.]{0,20}(sender|synth)([^[:alnum:]_.]|$)",
        &[r"(^|[^[:alnum:]_])sender([^[:alnum:]_.]|$)"],
    ),
    ("how ctx.is_reference() is decided", "is_reference", &[r"module\.rs|elected|election|routes"]),
    (
        "listen-before-claim delay, its value and what it guards",
        "listen|claim",
        &[
            r"(1[.,]5[^0-9]{0,3}(s|sec|secs|second|seconds))|(1500[[:space:]]*ms)|(1[ _]?500[ _]?000[ _]?000[[:space:]]*ns)",
            "before|hear|guard|already|join|wait",
        ],
    ),
    (
        "how transport datagrams are sent and on which output connection",
        "transport|datagram|events",
        &["events", r"output|connection|port|\bsends?\b|\bsent\b|\bsending\b"],
    ),
];

/// AC2: the four facts, answered with a pointer or marked not in these
/// files, with an evidence tag.
pub(crate) fn ac2(s: &Subs, r: &mut Report) {
    for (desc, anchor, needles) in FACTS {
        match fact_scan(s, anchor, needles) {
            Fact::Answer(k) => pass(r, "AC2", format!("fact answered with pointer and [code]: {desc} (line {k})")),
            Fact::Notin(k) => pass(
                r,
                "AC2",
                format!("fact explicitly marked not in these files, with [code]: {desc} (line {k})"),
            ),
            Fact::AnswerNotag(k) => {
                r.fail("AC2", format!("fact {desc} is answered (line {k}) but carries no evidence tag"))
            }
            Fact::NotinNotag(k) => r.fail(
                "AC2",
                format!("fact {desc} is marked not in these files (line {k}) but carries no evidence tag"),
            ),
            Fact::None => r.fail(
                "AC2",
                format!("fact not stated, not answered with a pointer, and not marked 'not in these files': {desc}"),
            ),
        }
    }
}

/// `line_covers_id`: the line names `C-<id>` alone, or a range `C-a to
/// C-b` (`through`, `..`, `-`) with `a <= id <= b` as bash's `test` reads
/// the two numbers.
fn line_covers_id(line: &str, id: u32) -> bool {
    if Grep::ere(&format!("(^|[^A-Za-z0-9])C-{id}([^0-9]|$)")).matches(line) {
        return true;
    }
    let ranges = Grep::ere(r"C-[0-9]+[[:space:]]*(to|through|\.\.|-)[[:space:]]*C-[0-9]+");
    ranges.only(line).into_iter().any(|range| {
        let a = Sed::ere(r"s/^C-([0-9]+).*/\1/").apply(range);
        let b = Sed::ere(r"s/.*C-([0-9]+)$/\1/").apply(range);
        let id = i64::from(id);
        bash_test_int(&a).is_some_and(|a| a <= id) && bash_test_int(&b).is_some_and(|b| b >= id)
    })
}

/// `check_const_id`.
fn const_id(s: &Subs, id: u32, r: &mut Report) {
    let differ = Grep::ere_i(r"(^|[^[:alnum:]_])differ(s|ed|ing)?([^[:alnum:]_]|$)");
    let verdict = Grep::ere_i(
        r"(^|[^[:alnum:]_])(match|matches|matched|matching|differ|differs|differed|differing)([^[:alnum:]_]|$)",
    );
    let (mut state, mut first) = ("unaccounted", None);
    for (start, end) in [s.t, s.c] {
        for k in start..=end {
            if !line_covers_id(s.lines.sed_line(k), id) {
                continue;
            }
            let block = s.lines.line_range(k, (k + 1).min(end));
            if differ.any_line(&block) && !Grep::bre("002-13").any_line(&block) {
                r.fail("AC4", format!("C-{id} differs but is not flagged for 002-13 (line {k})"));
                return;
            }
            if has_marker(&block) {
                if has_tag(&block, TAGS) {
                    first.get_or_insert(k);
                } else {
                    state = "notin_notag";
                }
            } else if verdict.any_line(&block) {
                if !has_ptr(&block, PTR) {
                    state = "verdict_noptr";
                } else if !has_tag(&block, TAGS) {
                    state = "verdict_notag";
                } else {
                    first.get_or_insert(k);
                }
            }
        }
    }
    if let Some(k) = first {
        r.check("AC4", true, format!("C-{id} accounted for with a pointer or not-in-files marker, with [code] (line {k})"));
        return;
    }
    r.fail(
        "AC4",
        match state {
            "notin_notag" => format!("C-{id} is marked not in these files but carries no evidence tag"),
            "verdict_noptr" => format!("C-{id} is marked matches/differs without a file:line pointer"),
            "verdict_notag" => format!("C-{id} is marked matches/differs but carries no evidence tag"),
            _ => format!("C-{id} is not accounted for (no matches/differs and no 'not in these files' marker)"),
        },
    );
}

/// AC4: C-30 to C-33 accounted for.
pub(crate) fn ac4(s: &Subs, r: &mut Report) {
    for id in [30, 31, 32, 33] {
        const_id(s, id, r);
    }
}

/// DoD: pointer lines tagged; open items owned.
pub(crate) fn dod(s: &Subs, r: &mut Report) {
    for (label, (start, end)) in [("transport.rs", s.t), ("control.rs", s.c)] {
        for k in untagged_pointer_lines(s.lines, start, end, PTR, TAGS) {
            r.fail("DoD", format!("{label} line {k} carries a file:line pointer but no evidence tag"));
        }
    }
    for (label, (start, end)) in [("transport.rs", s.t), ("control.rs", s.c)] {
        for k in unowned_open_items(s.lines, start, end) {
            r.fail("DoD", format!("{label} line {k} raises an open item without an owner and a falsifier"));
        }
    }
}

/// The section marker: exactly one verdict line in section (d), pending or
/// a final form.
pub(crate) fn section_marker(s: &Subs, r: &mut Report) {
    const C: &str = "Section marker";
    let section = s.lines.sed_range_text(s.d + 1, s.e - 1);
    match section_verdict(&section) {
        SectionVerdict::Missing => r.fail(C, "section (d) lost its 'verdict:' line"),
        SectionVerdict::Many(n) => r.fail(
            C,
            format!("section (d) has {n} 'verdict:' lines; exactly one section-level verdict is required"),
        ),
        SectionVerdict::One { line, form } => match form {
            VerdictForm::Pending => pass(r, C, "section (d) has exactly one verdict line (pending)"),
            VerdictForm::Mechanism => {
                pass(r, C, "section (d) has exactly one verdict line (instrument bug, mechanism)")
            }
            VerdictForm::Explanation => {
                pass(r, C, "section (d) has exactly one verdict line (not a bug, explanation)")
            }
            VerdictForm::Other => {
                r.fail(C, format!("section (d) has one verdict line but it is not in an allowed form: {line}"))
            }
        },
    }
}
