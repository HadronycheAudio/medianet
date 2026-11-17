//! Port of `checks/006-01.sh`: the timeline specification skeleton and its
//! constants table (ticket 006-01).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_006_01.rs`. The script's variants, named where used:
//!
//! * fences: [`FencePolicy::Matching`] (opener `^[ \t]*` three or more
//!   backticks or tildes, closed only by a bare marker of the same
//!   character at least as long);
//! * rows and tables: [`TableRules::T006_01`] (character-walking row split,
//!   dash-only separator line), the header and separator lines kept;
//! * headings: matched by stemmed token groups (`markdown::stem_match`), level 2 or
//!   deeper, the first match; a section runs to the line before the next
//!   heading of the same or a higher level;
//! * text: `norm_text` with `pycompat::nfkc`;
//! * tags: `[code]` only, in lower case, in the retired-behaviour note.
//!
//! The script has no repository-level assertion.

//! [`FencePolicy::Matching`]: crate::doccheck::policy::FencePolicy::Matching
//! [`TableRules::T006_01`]: crate::doccheck::policy::TableRules::T006_01

mod constants;
mod denials;
mod scan;
mod sections;

use super::{CheckSpec, Target};
use crate::doccheck::policy::TomlReader;
use crate::doccheck::pycompat::{splitlines, universal_newlines};
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
        Target {
            file: SPEC_MD,
            heading: None,
        },
        Target {
            file: CONSTANTS,
            heading: None,
        },
    ],
    keywords: &["wins", "differ", "pending"],
    known_differences: &[],
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: crate::doccheck::equiv::script::DEFAULT_READER,
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
    let spec = universal_newlines(&spec);
    let lines = splitlines(&spec);
    let sc = scan::scan(&lines);
    let tables = scan::tables(&lines);
    let source = docs
        .read(CONSTANTS)
        .map(|t| scan::constants_source(&universal_newlines(&t)));
    constants::ac1(&lines, &sc, &tables, source.as_ref(), &mut r);
    sections::ac2(&lines, &sc, &mut r);
    sections::ac3(&lines, &sc, &mut r);
    sections::ac4(&sc, &tables, &mut r);
    r
}
