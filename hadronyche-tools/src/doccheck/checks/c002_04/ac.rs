//! The criteria of 002-04 after the layout, each as the script computes it.

use super::Subs;
use crate::doccheck::Report;
use crate::doccheck::family002::rules::{
    PtrRule, SectionVerdict, TagSet, VerdictForm, has_ptr, has_tag, section_verdict,
    unowned_open_items, untagged_pointer_lines,
};
use crate::doccheck::family002::{Grep, distinct_count};

const TAGS: TagSet = TagSet::Six;
const PTR: PtrRule = PtrRule::Plain;

const TYPE32: &str = r"(^|[^[:alnum:]_])(u32|32[ -]?bit|4[ -]?byte)([^[:alnum:]_]|$)";
const TYPE64: &str = r"(^|[^[:alnum:]_])(u64|64[ -]?bit|8[ -]?byte)([^[:alnum:]_]|$)";
const UNIT: &str = r"(^|[^[:alnum:]_])(packets?|counts?|dimensionless|unitless)([^[:alnum:]_]|$)|no (unit|conversion|scaling)|unchanged";
const LATEST: &str = r"(^|[^[:alnum:]_])(latest|last|newest|overwrit(e|es|ten|ing)|replac(e|es|ed|ing)|wins?|supersed(e|es|ed))([^[:alnum:]_]|$)";
const POOLED: &str = r"no per[ -]?stream|not per[ -]?stream|across all|without (per[ -]?stream|separat)|no (per[ -]?stream )?separat|pooled|merged|single (counter|value|number|lost)|one (counter|value|number)|regardless of (the )?stream|indifferent to (the )?stream";
const TX_UNIT: &str = r"(^|[^[:alnum:]_])(ns|nanoseconds?|µs|us|microseconds?|ms|milliseconds?)([^[:alnum:]_]|$)";

fn check(r: &mut Report, c: &str, ok: bool, pass: String, fail: String) {
    r.check(c, ok, if ok { pass } else { fail });
}

/// The block of `k` and `k + 1`, clipped at `end`.
fn block(s: &Subs, k: usize, end: usize) -> String {
    s.lines.line_range(k, (k + 1).min(end))
}

/// AC1, per sub-section: at most 30 non-empty lines; at least 3 distinct
/// pointers; at least 2 to its own file.
pub(crate) fn ac1(s: &Subs, r: &mut Report) {
    for (label, (start, end), own) in s.all() {
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

/// `hop_check`: the first pointer line whose block holds every required ERE
/// (case-insensitive); else the missing EREs of the first pointer line.
fn hop(s: &Subs, r: &mut Report, label: &str, (start, end): (usize, usize), ptr: &str, required: &[&str]) {
    let ptr_re = Grep::ere(ptr);
    let mut first: Option<(usize, String)> = None;
    for k in start..=end {
        if !ptr_re.matches(s.lines.sed_line(k)) {
            continue;
        }
        let b = block(s, k, end);
        let missing: String = required
            .iter()
            .filter(|re| !Grep::ere_i(re).any_line(&b))
            .map(|re| format!("\n    - /{re}/"))
            .collect();
        if missing.is_empty() {
            r.check("AC2", true, format!("{label} described (line {k}) with field, width, unit and [code]"));
            return;
        }
        first.get_or_insert((k, missing));
    }
    match first {
        None => r.fail(
            "AC2",
            format!("{label} is not mentioned: no line in its sub-section carries a pointer matching /{ptr}/"),
        ),
        Some((k, missing)) => {
            r.fail("AC2", format!("{label} is mentioned at line {k} but the description lacks:{missing}"))
        }
    }
}

/// AC2: the three hops.
pub(crate) fn ac2(s: &Subs, r: &mut Report) {
    hop(
        s,
        r,
        "sender.rs:435 hop",
        s.s,
        r"sender\.rs:435([^0-9]|$)",
        &["rx_lost", TYPE32, UNIT, "header|echo|datagram|reply|returned", r"\[code\]"],
    );
    hop(s, r, "stats.rs:324 hop", s.st, r"stats\.rs:324([^0-9]|$)", &["lost_fwd", TYPE64, UNIT, r"\[code\]"]);
    hop(
        s,
        r,
        "tui.rs:107-114 hop",
        s.u,
        r"tui\.rs:(10[7-9]|11[0-4])([^0-9]|$)",
        &["lost_fwd", TYPE64, UNIT, "one[- ]line|display|print|status|format", "sent|lost_total|received", r"\[code\]"],
    );
}

/// The context fact: in the stats.rs note, a line naming lost_fwd/rx_lost
/// and stream(s) whose block has a pointer, a tag, a latest-record marker
/// and a no-per-stream marker.
pub(crate) fn context_fact(s: &Subs, r: &mut Report) {
    const C: &str = "Context fact";
    let (start, end) = s.st;
    let (field, streams) = (Grep::ere_i("lost_fwd|rx_lost"), Grep::ere_i("streams?"));
    let mut first: Option<(usize, String)> = None;
    for k in start..=end {
        let line = s.lines.sed_line(k);
        if !(field.matches(line) && streams.matches(line)) {
            continue;
        }
        let b = block(s, k, end);
        let mut missing = String::new();
        if !has_ptr(&b, PTR) {
            missing.push_str(" pointer");
        }
        if !has_tag(&b, TAGS) {
            missing.push_str(" evidence-tag");
        }
        if !Grep::ere_i(LATEST).any_line(&b) {
            missing.push_str(" latest-record-marker");
        }
        if !Grep::ere_i(POOLED).any_line(&b) {
            missing.push_str(" no-per-stream-marker");
        }
        if missing.is_empty() {
            r.check(C, true, format!("stats.rs note states that lost_fwd is the latest record's value pooled across streams (line {k})"));
            return;
        }
        first.get_or_insert((k, missing));
    }
    match first {
        None => r.fail(
            C,
            "stats.rs note does not say what the loss display takes across return streams (need a line naming lost_fwd/rx_lost together with stream(s))",
        ),
        Some((k, missing)) => r.fail(C, format!("stats.rs note names lost_fwd across streams (line {k}) but lacks:{missing}")),
    }
}

/// AC3: a TX pacing lateness line, in any of the three notes, with a
/// sender.rs pointer, a time unit and a tag.
pub(crate) fn ac3(s: &Subs, r: &mut Report) {
    let named = Grep::ere_i("tx pacing lateness|tx_late");
    let mut first: Option<(usize, String)> = None;
    for (_, (start, end), _) in s.all() {
        for k in start..=end {
            if !named.matches(s.lines.sed_line(k)) {
                continue;
            }
            let b = block(s, k, end);
            let mut missing = String::new();
            if !Grep::ere(r"sender\.rs:[0-9]+").any_line(&b) {
                missing.push_str(" sender.rs-computation-pointer");
            }
            if !Grep::ere_i(TX_UNIT).any_line(&b) {
                missing.push_str(" time-unit");
            }
            if !has_tag(&b, TAGS) {
                missing.push_str(" evidence-tag");
            }
            if missing.is_empty() {
                r.check("AC3", true, format!("TX pacing lateness computation site (sender.rs) and unit given (line {k})"));
                return;
            }
            first.get_or_insert((k, missing));
        }
    }
    match first {
        None => r.fail("AC3", "no line in the three notes mentions 'TX pacing lateness' or tx_late"),
        Some((k, missing)) => r.fail("AC3", format!("TX pacing lateness is mentioned (line {k}) but the statement lacks:{missing}")),
    }
}

/// DoD: pointer lines tagged; open items owned.
pub(crate) fn dod(s: &Subs, r: &mut Report) {
    for (label, (start, end), _) in s.all() {
        for k in untagged_pointer_lines(s.lines, start, end, PTR, TAGS) {
            r.fail("DoD", format!("{label} line {k} carries a file:line pointer but no evidence tag"));
        }
    }
    for (label, (start, end), _) in s.all() {
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
    let msg = match section_verdict(&section) {
        SectionVerdict::Missing => Err("section (d) lost its 'verdict:' line".to_string()),
        SectionVerdict::Many(n) => Err(format!(
            "section (d) has {n} 'verdict:' lines; exactly one section-level verdict is required"
        )),
        SectionVerdict::One { line, form } => match form {
            VerdictForm::Pending => Ok("section (d) has exactly one verdict line (pending)"),
            VerdictForm::Mechanism => Ok("section (d) has exactly one verdict line (instrument bug, mechanism)"),
            VerdictForm::Explanation => Ok("section (d) has exactly one verdict line (not a bug, explanation)"),
            VerdictForm::Other => Err(format!("section (d) has one verdict line but it is not in an allowed form: {line}")),
        },
    };
    match msg {
        Ok(m) => {
            r.check(C, true, m);
        }
        Err(m) => r.fail(C, m),
    }
}
