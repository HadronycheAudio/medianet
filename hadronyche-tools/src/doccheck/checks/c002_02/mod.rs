//! Port of `checks/002-02.sh`: the reading notes on `audio.rs` (`analyse`,
//! `analyse_voices`) in section (d) of REPORT-0b (ticket 002-02).
//!
//! The script is bash; its semantics come from [`crate::doccheck::family002`]
//! (GNU grep and sed in the scripts' locale, bash expansions, `tr`, `cut`,
//! proven against the real tools). The criterion map, with the ticket
//! sentences, is on the tests in `tests/doc_002_02.rs`; the denial list is
//! `denials.rs`. The script prints unlabelled `FAIL: message` lines; its
//! criteria are the banners its `bad` calls sit under (the harness reads
//! them the same way, `equiv::labels002`): `Helpers` (the report is
//! missing), `Locate` (section (d), its (e) bound, the sub-section),
//! `AC1` to `AC5`, `DoD`. The variants it uses, named where used:
//!
//! * `heading_no`: the first `#` line matching a case-insensitive ERE over
//!   the whole line, NOT fence-aware ([`Lines::heading_no`]);
//! * the sub-section: the first heading line (`^#+[[:space:]]`) strictly
//!   between (d) and (e) whose ASCII-lowercased text holds `audio.rs` and
//!   `analyse` or `analysis`; its level is the length of its leading run of
//!   `#` and spaces (`${line%%[! #]*}`: `### x` is level 4); it ends before
//!   the next heading of that level or lower, or before (e);
//! * the sub-section text is `$(sed -n …)`: trailing empty lines dropped;
//!   it is read back line by line through a here-document;
//! * a fact's block is its line and the NEXT line of the file, even past the
//!   end of the sub-section;
//! * the DoD row is searched from the Open sub-questions heading to the END
//!   OF THE FILE through `read`, which loses a last line without a newline.
//!
//! The script has no repository-level assertion: nothing is relocated.

mod ac;
mod denials;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::family002::{self, Grep, Lines, Read, leading_run, tr_lower};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::{Docs, Report};

/// The report the script reads.
pub const REPORT: &str = "doc/design/reports/archaeology/REPORT-0b.md";

/// The criteria, as the script's banners name them.
pub const HELPERS: &str = "Helpers";
pub const LOCATE: &str = "Locate";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "002-02",
    run,
    script_env: &[],
    targets: &[
        Target {
            file: REPORT,
            heading: Some(r"^audio\.rs"),
        },
        Target {
            file: REPORT,
            heading: Some("^Open sub-questions$"),
        },
        Target {
            file: REPORT,
            heading: Some(r"^\(d\)"),
        },
    ],
    keywords: &["confirmed", "corrected", "answered", "because", "complete", "yes"],
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

/// The sub-section, as the script bounds it.
pub(crate) struct Sub<'a> {
    pub lines: &'a Lines<'a>,
    /// Its heading line and its last line (1-based, inclusive).
    pub start: usize,
    pub end: usize,
    /// `$(line_range "$REPORT" "$sub_no" "$sub_end")`.
    pub text: String,
}

/// Records `ok` as a pass (with `pass`) or a failure (with `fail`).
pub(crate) fn verdict(r: &mut Report, criterion: &str, ok: bool, pass: String, fail: String) {
    r.check(criterion, ok, if ok { pass } else { fail });
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut r = Report::default();
    let text = match family002::read(docs, REPORT, &mut r) {
        Read::Missing => {
            r.fail(HELPERS, format!("report missing: {REPORT}"));
            return r;
        }
        Read::Text(t) => t,
    };
    if !r.passed() {
        return r;
    }
    let lines = Lines::new(&text);
    let Some(sub) = locate(&lines, &mut r) else {
        return r;
    };
    ac::ac1(&sub, &mut r);
    ac::ac2(&sub, &mut r);
    ac::ac3(&sub, &mut r);
    ac::ac4(&sub, &mut r);
    ac::ac5(&sub, &mut r);
    ac::dod(&lines, &mut r);
    r
}

/// Section (d), its (e) bound and the audio.rs sub-section; `None` (with
/// the failure recorded) where the script exits.
fn locate<'a>(lines: &'a Lines<'a>, r: &mut Report) -> Option<Sub<'a>> {
    let d_no = lines.heading_no(&Grep::ere_i(r"^#+[[:space:]]*\(d\)"));
    let e_no = lines.heading_no(&Grep::ere_i(r"^#+[[:space:]]*\(e\)"));
    let Some(d) = d_no else {
        r.fail(LOCATE, "section (d) Reading notes not found (heading containing '(d)')");
        return None;
    };
    let e = match e_no {
        Some(e) if e > d => e,
        _ => {
            r.fail(
                LOCATE,
                "section (e) heading not found after section (d); cannot bound the reading notes",
            );
            return None;
        }
    };
    r.check(LOCATE, true, format!("section (d) found at line {d}, section (e) at line {e}"));
    let heading = Grep::ere(r"^#+[[:space:]]");
    let level = |line: &str| leading_run(line, " #").chars().count();
    let found = (d + 1..e).find_map(|i| {
        let line = lines.sed_line(i);
        let low = tr_lower(line);
        let named = low.contains("audio.rs") && (low.contains("analyse") || low.contains("analysis"));
        (heading.matches(line) && named).then(|| (i, level(line)))
    });
    let Some((start, lvl)) = found else {
        r.fail(
            LOCATE,
            "no sub-section heading under (d) containing both 'audio.rs' and 'analyse'/'analysis'",
        );
        return None;
    };
    r.check(LOCATE, true, format!("sub-section found at line {start}: {}", lines.sed_line(start)));
    let end = (start + 1..e)
        .find(|j| {
            let line = lines.sed_line(*j);
            heading.matches(line) && level(line) <= lvl
        })
        .map_or(e - 1, |j| j - 1);
    let text = lines.line_range(start, end);
    if text.is_empty() {
        // unreachable: the text holds the heading line itself
        r.fail(LOCATE, "sub-section is empty");
        return None;
    }
    Some(Sub {
        lines,
        start,
        end,
        text,
    })
}
