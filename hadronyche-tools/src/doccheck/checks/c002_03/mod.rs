//! Port of `checks/002-03.sh`: the reading notes on `transport.rs` and
//! `control.rs` in section (d) of REPORT-0b (ticket 002-03).
//!
//! Bash semantics from [`crate::doccheck::family002`] (GNU grep, sed and
//! mawk as they ran, proven against the real tools). Criteria are the
//! banners of the script's `bad` calls: `Helpers` (the report is missing),
//! `Locate` (section (d), its (e) bound, the two sub-sections), `AC1` to
//! `AC4`, `DoD`, `Section marker`. The variants, named where used:
//!
//! * `heading_no`, not fence-aware ([`Lines::heading_no`]);
//! * a heading's level is its number of leading `#` (`sed -E
//!   's/^(#+).*/\1/'`; 002-02 counted spaces too);
//! * a sub-section is found on the ASCII-lowercased heading line
//!   (`transport\.rs`, `control\.rs`) and ends before the next heading of
//!   its level or higher, or before (e);
//! * every two-line block (a fact, a pointer line, a constant id, an open
//!   item) is clipped at the sub-section's end (002-02 took the next line of
//!   the file);
//! * the unit check is the script's awk program on bytes (mawk), ported in
//!   [`units`] and proven whole against mawk ([`units::unit_offenders`],
//!   corpus snippet `002-03 unit_offenders`);
//! * a range of constant ids is compared with bash's `test` integers
//!   ([`bash_test_int`]: `030` is 30; a number beyond 64 bits is no number);
//! * tag discipline, open items and the section verdict:
//!   [`family002::rules`] (Tag5, the plain pointer, pending accepted).
//!
//! No repository-level assertion: nothing is relocated.
//!
//! [`bash_test_int`]: crate::doccheck::family002::bash_test_int

mod ac;
mod denials;
pub mod units;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::family002::{self, Grep, Lines, Read, Sed, tr_lower};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::{Docs, Report};

/// The report the script reads.
pub const REPORT: &str = "doc/design/reports/archaeology/REPORT-0b.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "002-03",
    run,
    script_env: &[],
    targets: &[
        Target {
            file: REPORT,
            heading: Some(r"^transport\.rs"),
        },
        Target {
            file: REPORT,
            heading: Some(r"^control\.rs"),
        },
        Target {
            file: REPORT,
            heading: Some(r"^\(d\)"),
        },
    ],
    keywords: &["matches", "differs", "not in these files", "sender", "events"],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    // the script reads no TOML; the field is required
    toml: TomlReader::Subset004_18,
    // the script prints unlabelled `FAIL: message` lines: the criterion is
    // the banner above the `bad` call that printed it
    fail_reader: crate::doccheck::equiv::labels002::READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The two sub-sections, as the script bounds them.
pub(crate) struct Subs<'a> {
    pub lines: &'a Lines<'a>,
    pub d: usize,
    pub e: usize,
    /// transport.rs: start and end (1-based, inclusive).
    pub t: (usize, usize),
    /// control.rs.
    pub c: (usize, usize),
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut r = Report::default();
    let text = match family002::read(docs, REPORT, &mut r) {
        Read::Missing => {
            r.fail("Helpers", format!("report missing: {REPORT}"));
            return r;
        }
        Read::Text(t) => t,
    };
    if !r.passed() {
        return r;
    }
    let lines = Lines::new(&text);
    let Some(subs) = locate(&lines, &mut r) else {
        return r;
    };
    ac::ac1(&subs, &mut r);
    ac::ac2(&subs, &mut r);
    units::ac3(&subs, &mut r);
    ac::ac4(&subs, &mut r);
    ac::dod(&subs, &mut r);
    ac::section_marker(&subs, &mut r);
    r
}

/// `hashes=$(printf '%s' "$line" | sed -E 's/^(#+).*/\1/'); ${#hashes}`:
/// the number of leading `#` of a heading line.
fn hashes(line: &str) -> usize {
    Sed::ere(r"s/^(#+).*/\1/").apply(line).chars().count()
}

/// The headings strictly between (d) and (e) whose lowercased line matches
/// `re`: (line, level).
fn find_headings(lines: &Lines, d: usize, e: usize, re: &str) -> Vec<(usize, usize)> {
    let heading = Grep::ere(r"^#+[[:space:]]");
    let named = Grep::ere(re);
    (d + 1..e)
        .filter(|i| {
            let line = lines.sed_line(*i);
            heading.matches(line) && named.matches(&tr_lower(line))
        })
        .map(|i| (i, hashes(lines.sed_line(i))))
        .collect()
}

/// The end of the sub-section at `start` of level `lvl`.
fn section_end(lines: &Lines, start: usize, lvl: usize, e: usize) -> usize {
    let heading = Grep::ere(r"^#+[[:space:]]");
    (start + 1..e)
        .find(|j| {
            let line = lines.sed_line(*j);
            heading.matches(line) && hashes(line) <= lvl
        })
        .map_or(e - 1, |j| j - 1)
}

/// Section (d), its (e) bound and the two sub-sections; `None` (with the
/// failures recorded) where the script exits.
fn locate<'a>(lines: &'a Lines<'a>, r: &mut Report) -> Option<Subs<'a>> {
    const L: &str = "Locate";
    let d = lines.heading_no(&Grep::ere_i(r"^#+[[:space:]]*\(d\)"));
    let e = lines.heading_no(&Grep::ere_i(r"^#+[[:space:]]*\(e\)"));
    let Some(d) = d else {
        r.fail(L, "section (d) Reading notes not found (heading containing '(d)')");
        return None;
    };
    let Some(e) = e.filter(|e| *e > d) else {
        r.fail(L, "section (e) heading not found after section (d); cannot bound the reading notes");
        return None;
    };
    let d_lvl = hashes(lines.sed_line(d));
    r.check(L, true, format!("section (d) found at line {d} (level {d_lvl}), section (e) at line {e}"));
    let t_list = find_headings(lines, d, e, r"transport\.rs");
    let c_list = find_headings(lines, d, e, r"control\.rs");
    if t_list.is_empty() || c_list.is_empty() {
        if t_list.is_empty() {
            r.fail(L, "no sub-section heading under (d) naming transport.rs");
        }
        if c_list.is_empty() {
            r.fail(L, "no sub-section heading under (d) naming control.rs");
        }
        return None;
    }
    if t_list.len() > 1 {
        r.fail(L, format!("{} headings under (d) name transport.rs; one sub-section per file is required", t_list.len()));
    }
    if c_list.len() > 1 {
        r.fail(L, format!("{} headings under (d) name control.rs; one sub-section per file is required", c_list.len()));
    }
    let ((ts, tl), (cs, cl)) = (t_list[0], c_list[0]);
    if ts == cs {
        r.fail(L, "the transport.rs and control.rs headings are the same heading; two distinct sub-sections are required");
        return None;
    }
    if tl <= d_lvl || cl <= d_lvl {
        r.fail(L, "a sub-section heading is not deeper than section (d); the notes would fall outside (d)");
    }
    let t = (ts, section_end(lines, ts, tl, e));
    let c = (cs, section_end(lines, cs, cl, e));
    r.check(L, true, format!("transport.rs sub-section: lines {}-{}", t.0, t.1));
    r.check(L, true, format!("control.rs sub-section: lines {}-{}", c.0, c.1));
    Some(Subs { lines, d, e, t, c })
}
