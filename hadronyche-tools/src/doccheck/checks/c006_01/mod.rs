//! Port of `checks/006-01.sh`: the timeline specification skeleton and its
//! constants table (ticket 006-01).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_006_01.rs`. The script's variants, named where used:
//!
//! * fences: an opener of three or more backticks or tildes, a closer of the
//!   same character, at least as long, with nothing after it
//!   ([`scan::scan`]; the library's `FencePolicy::Matching` differs on an
//!   info string after the closer);
//! * rows: split by walking the characters ([`text::split_row`]), a table
//!   needs at least two visible lines with a dash-only second line;
//! * headings: matched by stemmed token groups, level 2 or deeper, the first
//!   match ([`sections`]); a section runs to the line before the next
//!   heading of the same or a higher level;
//! * text: `norm_text` with NFKC ([`text`]);
//! * tags: `[code]` only, in lower case, in the retired-behaviour note.
//!
//! The script has no repository-level assertion.

mod constants;
mod scan;
mod sections;
pub(crate) mod text;

use super::{CheckSpec, Target};
use crate::doccheck::{Docs, Report};

/// The specification.
pub const SPEC_MD: &str = "doc/design/spec/timeline.md";
const CONSTANTS: &str = "doc/design/02-constants.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "006-01",
    run,
    script_env: &[],
    targets: &[
        Target { file: SPEC_MD, heading: None },
        Target { file: CONSTANTS, heading: None },
    ],
    keywords: &["wins", "differ", "pending"],
    known_differences: &[],
};

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut r = Report::default();
    let Some(spec) = docs.read(SPEC_MD) else {
        for ac in ["AC1", "AC2", "AC3", "AC4"] {
            r.fail(ac, "doc/design/spec/timeline.md does not exist");
        }
        return r;
    };
    let lines = crate::doccheck::text::splitlines(&spec);
    let sc = scan::scan(&lines);
    let tables = scan::tables(&lines, &sc.visible);
    let source = docs.read(CONSTANTS).map(|t| scan::constants_source(&t));
    constants::ac1(&lines, &sc, &tables, source.as_ref(), &mut r);
    sections::ac2(&lines, &sc, &mut r);
    sections::ac3(&lines, &sc, &mut r);
    sections::ac4(&sc, &tables, &mut r);
    r
}
