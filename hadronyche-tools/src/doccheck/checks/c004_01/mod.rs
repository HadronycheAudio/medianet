//! Port of `checks/004-01.sh`: the wire.md skeleton and encoding conventions
//! (ticket 004-01).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_01.rs`. The script's variants, named where used:
//!
//! * headings outside `^\s*```` fences, `^(#{1,6})\s+(.*?)\s*#*\s*$`, a
//!   section running to the next heading of the same or a higher level; a
//!   heading is found by a predicate over its title, the first in order;
//! * tables: a run of `|` lines (fenced runs skipped by toggling on
//!   `` ``` ``) with a dash-only second line, cells split on unescaped
//!   pipes ([`RowSplit::Escaped`]);
//! * tags: the literal `[decision]` in the byte-order section only;
//! * windows (320 to 500 characters) in characters, as Python slices them;
//! * the constants of the Constants subsection compared with
//!   02-constants.md by their numbers (thousands separators dropped), the
//!   table of 02-constants.md winning.
//!
//! The script has no repository-level assertion. Other scripts
//! (`checks/004-03.sh` to `004-09.sh`) still run `checks/004-01.sh` as a
//! gate, so it stays in the tree until they are ported.
//!
//! [`RowSplit::Escaped`]: crate::doccheck::policy::RowSplit::Escaped

mod ac_a;
mod ac_b;
mod doc;

use super::{CheckSpec, Target};
use crate::doccheck::{Docs, Report};

/// The documents.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
const CONSTANTS_MD: &str = "doc/design/02-constants.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-01",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("CONSTANTS_MD", CONSTANTS_MD)],
    targets: &[
        Target { file: WIRE_MD, heading: Some("^Byte order") },
        Target { file: WIRE_MD, heading: Some("(?i)alignment") },
        Target { file: WIRE_MD, heading: Some("(?i)bit-level encoding") },
        Target { file: WIRE_MD, heading: Some("(?i)field table") },
        Target { file: WIRE_MD, heading: Some("^Layer overview") },
        Target { file: WIRE_MD, heading: Some("^Constants used") },
        Target { file: CONSTANTS_MD, heading: None },
    ],
    keywords: &["endian", "guaranteed", "multicast"],
    known_differences: &[],
};

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut r = Report::default();
    let Some(wire) = docs.read(WIRE_MD) else {
        r.fail("AC1", format!("{WIRE_MD} does not exist"));
        for ac in 2..8 {
            r.fail(&format!("AC{ac}"), format!("cannot check \u{2014} {WIRE_MD} does not exist"));
        }
        return r;
    };
    let d = doc::Doc::new(&wire);
    ac_a::ac1(&d, &mut r);
    ac_a::ac2(&d, &mut r);
    ac_a::ac3(&d, &mut r);
    ac_a::ac4(&d, &mut r);
    ac_b::ac5(&d, &mut r);
    ac_b::ac6(&wire, &d, &mut r);
    let consts = docs.read(CONSTANTS_MD);
    ac_b::ac7(&d, consts.as_deref(), CONSTANTS_MD, &mut r);
    r
}
