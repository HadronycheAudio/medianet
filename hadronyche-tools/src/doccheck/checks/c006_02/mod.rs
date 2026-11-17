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

mod denials;
mod prose;
mod rows;

use super::{CheckSpec, Target};
use crate::doccheck::family006::{norm_lines, norm_text};
use crate::doccheck::markdown::Markdown;
use crate::doccheck::policy::{FencePolicy, HeadingMatch, TomlReader};
use crate::doccheck::pycompat::{parse_int, universal_newlines};
use crate::doccheck::text::{has_i, rx};
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
    targets: &[Target {
        file: SPEC_MD,
        heading: Some("^Domain line and position map$"),
    }],
    keywords: &["floor", "ceil", "decision", "only"],
    known_differences: &[],
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: crate::doccheck::equiv::script::DEFAULT_READER,
};

/// `declared_origin`: `int(raw)` of the number after `origin =`; a failed
/// `int` is the script's `(0, False)`.
fn declared_origin(text: &str) -> i128 {
    let re = rx(r"(?i)origin(?:_ns)?\s*(?:=|is|:)\s*([+\-]?\d[\d _,\u00a0\u202f]*)");
    let Some(m) = re.captures(text) else { return 0 };
    let raw = rx(r"[ _,\u00a0\u202f]").replace_all(&m[1], "");
    parse_int(&raw).unwrap_or_default()
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut r = Report::default();
    let Some(raw) = docs.read(SPEC_MD) else {
        r.fail("AC1", format!("missing {SPEC_MD}"));
        return r;
    };
    let raw = universal_newlines(&raw);
    let lines: Vec<String> = norm_lines(&raw);
    let md = Markdown::parse_with(&lines.join("\n"), FencePolicy::NoFences);
    let found = md
        .find_by(HeadingMatch::Exact, HEADING)
        .into_iter()
        .next()
        .map(|h| md.section(h));
    let Some((start, end)) = found.map(|s| (s.lo, s.hi)) else {
        r.fail("AC1", format!("timeline.md has no '## {HEADING}' heading"));
        for ac in ["AC2", "AC3", "AC4", "AC5", "AC6"] {
            r.fail(ac, "section missing, cannot check");
        }
        return r;
    };
    let section_lines = &lines[start + 1..end];
    let text = norm_text(&section_lines.join("\n"));
    if text.is_empty() || has_i("to be written in 006-02", &text) {
        r.fail(
            "AC1",
            "section is still the 006-02 placeholder (no formulas, no table)",
        );
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
