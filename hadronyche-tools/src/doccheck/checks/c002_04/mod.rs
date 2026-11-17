//! Port of `checks/002-04.sh`: the reading notes on `sender.rs`,
//! `stats.rs` and `tui.rs` in section (d) of REPORT-0b (ticket 002-04).
//!
//! Bash semantics from [`crate::doccheck::family002`]. Criteria are the
//! banners of the script's `bad` calls: `Helpers`, `Locate`, `AC1`,
//! `AC2`, `Context fact`, `AC3`, `DoD`, `Section marker`. The variants,
//! named where used:
//!
//! * `heading_no`, not fence-aware; a level is the count of leading `#`;
//! * a per-file heading names exactly one of `sender.rs`, `stats.rs`,
//!   `tui.rs` (substrings of the ASCII-lowercased line): a heading naming
//!   two or three is an umbrella and is skipped;
//! * two-line blocks clipped at the sub-section's end;
//! * the evidence tags are Tag6 (`[standard]` added,
//!   [`TagSet::Six`](crate::doccheck::family002::rules::TagSet::Six)); the
//!   plain pointer; tag discipline, open items and the section verdict from
//!   [`family002::rules`] (pending accepted);
//! * a hop is described by the FIRST pointer line whose block holds every
//!   required ERE; the failure reports the missing EREs of the first
//!   pointer line (a multi-line message, as the script prints it).
//!
//! No repository-level assertion: nothing is relocated.

mod ac;
mod denials;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::family002::{self, Grep, Lines, Read, Sed, tr_lower};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::{Docs, Report};

/// The report the script reads.
pub const REPORT: &str = "doc/design/reports/archaeology/REPORT-0b.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "002-04",
    run,
    script_env: &[],
    targets: &[
        Target {
            file: REPORT,
            heading: Some(r"^sender\.rs"),
        },
        Target {
            file: REPORT,
            heading: Some(r"^stats\.rs"),
        },
        Target {
            file: REPORT,
            heading: Some(r"^tui\.rs"),
        },
        Target {
            file: REPORT,
            heading: Some(r"^\(d\)"),
        },
    ],
    keywords: &["latest", "pooled", "streams", "packets", "unchanged"],
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

/// The three sub-sections (start, end), as the script bounds them.
pub(crate) struct Subs<'a> {
    pub lines: &'a Lines<'a>,
    pub d: usize,
    pub e: usize,
    pub s: (usize, usize),
    pub st: (usize, usize),
    pub u: (usize, usize),
}

impl Subs<'_> {
    /// The sub-sections with their labels and own-file stems.
    pub fn all(&self) -> [(&'static str, (usize, usize), &'static str); 3] {
        [("sender.rs", self.s, "sender"), ("stats.rs", self.st, "stats"), ("tui.rs", self.u, "tui")]
    }
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
    ac::context_fact(&subs, &mut r);
    ac::ac3(&subs, &mut r);
    ac::dod(&subs, &mut r);
    ac::section_marker(&subs, &mut r);
    r
}

/// The number of leading `#` (`sed -E 's/^(#+).*/\1/'`, `${#hashes}`).
fn hashes(line: &str) -> usize {
    Sed::ere(r"s/^(#+).*/\1/").apply(line).chars().count()
}

/// `find_file_headings want`: the headings strictly between (d) and (e)
/// that name exactly one of the three files, that one being `want`.
fn find_file_headings(lines: &Lines, d: usize, e: usize, want: &str) -> Vec<(usize, usize)> {
    let heading = Grep::ere(r"^#+[[:space:]]");
    (d + 1..e)
        .filter(|i| {
            let line = lines.sed_line(*i);
            if !heading.matches(line) {
                return false;
            }
            let low = tr_lower(line);
            let names = ["sender", "stats", "tui"].iter().filter(|t| low.contains(&format!("{t}.rs"))).count();
            names == 1 && low.contains(&format!("{want}.rs"))
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

/// `check_disjoint`.
fn disjoint(r: &mut Report, (an, (a_s, a_e)): (&str, (usize, usize)), (bn, (b_s, b_e)): (&str, (usize, usize))) {
    if a_s < b_s && b_s <= a_e {
        r.fail(
            "Locate",
            format!("{bn} sub-section (line {b_s}) lies inside the {an} sub-section (lines {a_s}-{a_e}); sub-sections must be disjoint"),
        );
    }
    if b_s < a_s && a_s <= b_e {
        r.fail(
            "Locate",
            format!("{an} sub-section (line {a_s}) lies inside the {bn} sub-section (lines {b_s}-{b_e}); sub-sections must be disjoint"),
        );
    }
}

/// Section (d), its (e) bound and the three sub-sections; `None` (with the
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
    let lists = ["sender", "stats", "tui"].map(|w| find_file_headings(lines, d, e, w));
    for (list, name) in lists.iter().zip(["sender.rs", "stats.rs", "tui.rs"]) {
        if list.is_empty() {
            r.fail(L, format!("no sub-section heading under (d) naming only {name}"));
        }
    }
    if lists.iter().any(Vec::is_empty) {
        return None;
    }
    for (list, name) in lists.iter().zip(["sender.rs", "stats.rs", "tui.rs"]) {
        if list.len() > 1 {
            r.fail(L, format!("{} headings under (d) name only {name}; one sub-section per file is required", list.len()));
        }
    }
    let [(ss, sl), (sts, stl), (us, ul)] = [lists[0][0], lists[1][0], lists[2][0]];
    if ss == sts || ss == us || sts == us {
        r.fail(L, "two of the sender.rs/stats.rs/tui.rs headings are the same heading; three distinct sub-sections are required");
        return None;
    }
    if sl <= d_lvl || stl <= d_lvl || ul <= d_lvl {
        r.fail(L, "a sub-section heading is not deeper than section (d); the notes would fall outside (d)");
    }
    let s = (ss, section_end(lines, ss, sl, e));
    let st = (sts, section_end(lines, sts, stl, e));
    let u = (us, section_end(lines, us, ul, e));
    disjoint(r, ("sender.rs", s), ("stats.rs", st));
    disjoint(r, ("sender.rs", s), ("tui.rs", u));
    disjoint(r, ("stats.rs", st), ("tui.rs", u));
    for (name, (a, b)) in [("sender.rs", s), ("stats.rs", st), ("tui.rs", u)] {
        r.check(L, true, format!("{name} sub-section: lines {a}-{b}"));
    }
    Some(Subs { lines, d, e, s, st, u })
}
