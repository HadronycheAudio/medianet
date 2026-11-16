//! Port of `checks/006-02.sh`: the "Domain line and position map" section of
//! timeline.md (ticket 006-02).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_006_02.rs`. The script's variants, named where used:
//!
//! * heading: the first heading whose text, with trailing `#` removed, is
//!   exactly `Domain line and position map` (case-sensitive,
//!   [`HeadingMatch::Exact`]-like), found without regard to fences; the
//!   section ends at the next line starting `#`s of its level or higher,
//!   fences not considered; every line is normalised first (`norm_text`);
//! * fences for the tables: an opener of three backticks or tildes closes on
//!   any fence of the same character, whatever its length;
//! * rows: split by walking characters, a table needs a header, a
//!   separator whose cells are `:?-{2,}:?`, and a row;
//! * numbers: `NUM_RE`; integers exactly, decimals that are not whole are
//!   the script's `Fraction` and never match; the map's arithmetic is exact
//!   integer arithmetic (`i128`), floor and ceiling as the script's `//`;
//! * tags: `[decision]` only, in lower case.
//!
//! The script has no repository-level assertion.
//!
//! [`HeadingMatch::Exact`]: crate::doccheck::policy::HeadingMatch::Exact

mod prose;
mod rows;

use super::c006_01::text::norm_text;
use super::{CheckSpec, Target};
use crate::doccheck::text::{has_i, rx, splitlines};
use crate::doccheck::{Docs, Report};

pub use prose::self_check;

/// The specification.
pub const SPEC_MD: &str = "doc/design/spec/timeline.md";
const HEADING: &str = "Domain line and position map";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "006-02",
    run,
    script_env: &[],
    targets: &[Target { file: SPEC_MD, heading: Some("^Domain line and position map$") }],
    keywords: &["floor", "ceil", "decision", "only"],
    known_differences: &[],
};

/// `find_section`: (heading index, end index).
fn find_section(lines: &[String]) -> Option<(usize, usize)> {
    let head = rx(r"^(#{1,6})\s+(.*?)\s*$");
    let (start, level) = lines.iter().enumerate().find_map(|(i, l)| {
        let m = head.captures(l)?;
        (m[2].trim().trim_end_matches('#').trim() == HEADING).then(|| (i, m[1].len()))
    })?;
    let any = rx(r"^(#{1,6})\s+");
    let end = (start + 1..lines.len())
        .find(|&j| any.captures(&lines[j]).is_some_and(|m| m[1].len() <= level))
        .unwrap_or(lines.len());
    Some((start, end))
}

/// `declared_origin`.
fn declared_origin(text: &str) -> i128 {
    let re = rx(r"(?i)origin(?:_ns)?\s*(?:=|is|:)\s*([+\-]?\d[\d _,\u{a0}\u{202f}]*)");
    let Some(m) = re.captures(text) else { return 0 };
    let raw: String = m[1].chars().filter(|c| !matches!(c, ' ' | '_' | ',' | '\u{a0}' | '\u{202f}')).collect();
    raw.parse().unwrap_or(0)
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut r = Report::default();
    let Some(raw) = docs.read(SPEC_MD) else {
        r.fail("AC1", format!("missing {SPEC_MD}"));
        return r;
    };
    let lines: Vec<String> = splitlines(&raw).into_iter().map(norm_text).collect();
    let Some((start, end)) = find_section(&lines) else {
        r.fail("AC1", format!("timeline.md has no '## {HEADING}' heading"));
        for ac in ["AC2", "AC3", "AC4", "AC5", "AC6"] {
            r.fail(ac, "section missing, cannot check");
        }
        return r;
    };
    let section_lines = &lines[start + 1..end];
    let text = norm_text(&section_lines.join("\n"));
    if text.is_empty() || has_i("to be written in 006-02", &text) {
        r.fail("AC1", "section is still the 006-02 placeholder (no formulas, no table)");
        for ac in ["AC2", "AC3", "AC4", "AC5", "AC6"] {
            r.fail(ac, "section is still the placeholder; cannot check");
        }
        return r;
    }
    let origin = declared_origin(&text);
    prose::ac1(&text, &mut r);
    prose::ac2(&text, &mut r);
    prose::ac3(&text, &mut r);
    prose::ac4(&text, &mut r);
    prose::ac5(&rows::tables(section_lines), origin, &mut r);
    prose::ac6(&text, &mut r);
    r
}
