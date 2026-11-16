//! Port of `checks/004-07.sh`: the timeline epoch wrap analysis (C-10, 16
//! bits) beside the epoch field of the media packet header (ticket 004-07).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_07.rs`. The script's variants, named where used:
//!
//! * the analysis is found by a heading (`^#{1,6}\s+...`, NOT fence-aware,
//!   lower-cased) about the epoch and its wrap, bump, rate, width or
//!   falsifier, else by the Media packet header heading; its section ends at
//!   the next heading of the same or a higher level, again not fence-aware;
//! * text is normalised (`NBSP`, figure and narrow spaces to a space, the
//!   minus sign to `-`, blanks squashed) before anything is read;
//! * windows (`+-500`, `+-350`, `+-300` characters) and positions are in
//!   characters, as Python slices them;
//! * the evidence rule is not per paragraph: at least three of the six tags
//!   (lower case) anywhere in the section, a `[decision]` or `[memory]` tag
//!   within 500 characters of each rate marker, any tag within 500 of the
//!   verdict;
//! * arithmetic in floating point (`f64`), 65 536 times the interval, a 30 %
//!   tolerance, one year of 31 557 600 s, a 1.9 s election floor;
//! * an open item: a row of 05-risks-and-open.md mentioning epoch or wrap
//!   with at least five `|` cells whose third and second last are not empty,
//!   or the words owner and falsif in the analysis.
//!
//! `CONST_MD` is accepted by the script and never read; so here. Its
//! repository-level assertions (the design checker prints OK; `checks/004-01.sh`
//! still passes) are in `scripts/repo-state.d/004-07.sh`.

mod analysis;
mod rates;

use super::{CheckSpec, Target};
use crate::doccheck::text::{has_i, rx, splitlines};
use crate::doccheck::{Docs, Report};

/// The documents.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
const OPEN_MD: &str = "doc/design/05-risks-and-open.md";

/// The harness description of this port. The override variables are given as
/// `./`-paths so that the script takes them for overrides and skips its
/// repository-level runs, which are in the repository-state script.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-07",
    run,
    script_env: &[
        ("WIRE_MD", "./doc/design/spec/wire.md"),
        ("CONST_MD", "./doc/design/02-constants.md"),
        ("OPEN_MD", "./doc/design/05-risks-and-open.md"),
    ],
    targets: &[
        Target { file: WIRE_MD, heading: Some("(?i)epoch.*(wrap|bump|rate|width|falsif)") },
        Target { file: OPEN_MD, heading: None },
    ],
    keywords: &["falsified", "holds", "stale", "wrap", "pathological"],
    known_differences: &[],
};

/// `norm`.
pub(crate) fn norm(text: &str) -> String {
    let t: String = text
        .chars()
        .map(|c| match c {
            '\u{a0}' | '\u{2007}' | '\u{202f}' => ' ',
            '\u{2212}' => '-',
            c => c,
        })
        .collect();
    rx(r"[ \t]+").replace_all(&t, " ").into_owned()
}

/// `find_heading`: (line index, level) of the first heading whose lower-cased
/// text satisfies `pred`.
fn find_heading(lines: &[&str], pred: impl Fn(&str) -> bool) -> Option<(usize, usize)> {
    let re = rx(r"^(#{1,6})\s+(.*?)\s*#*\s*$");
    lines.iter().enumerate().find_map(|(i, l)| {
        let m = re.captures(l)?;
        pred(&m[2].to_lowercase()).then(|| (i, m[1].len()))
    })
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut r = Report::default();
    let Some(raw) = docs.read(WIRE_MD) else {
        r.fail("AC1", format!("missing {WIRE_MD}"));
        for ac in ["AC2", "AC3", "DoD"] {
            r.fail(ac, "wire.md missing, cannot check");
        }
        return r;
    };
    let wire_text = norm(&raw);
    let lines = splitlines(&wire_text);
    let bounds = find_heading(&lines, |h| h.contains("epoch") && rx(r"wrap|bump|rate|width|falsif").is_match(h))
        .or_else(|| find_heading(&lines, |h| h.contains("media packet header")));
    let Some((start, level)) = bounds else {
        r.fail("AC1", "wire.md has neither an epoch wrap/bump section nor the Media packet header section");
        r.fail("AC2", "no analysis section to read the comparison rule from");
        r.fail("AC3", "no analysis section; the falsifier is unanswered");
        r.fail("DoD", "no analysis section; no evidence tags to check");
        return r;
    };
    let any = rx(r"^(#{1,6})\s+");
    let end = (start + 1..lines.len())
        .find(|&j| any.captures(lines[j]).is_some_and(|m| m[1].len() <= level))
        .unwrap_or(lines.len());
    let section = norm(&lines[start + 1..end].join("\n"));
    if has_i("to be written", &section) || section.trim().is_empty() {
        for ac in ["AC1", "AC2", "AC3", "DoD"] {
            r.fail(ac, "the analysis section is still a placeholder or empty");
        }
        return r;
    }
    let open_text = docs.read(OPEN_MD).map(|t| norm(&t)).unwrap_or_default();
    analysis::run(&section, &wire_text, &open_text, &mut r);
    r
}
