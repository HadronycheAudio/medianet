//! Port of `checks/004-10.sh`: the media measurement extension layout
//! (ticket 004-10). The criterion map, with the ticket sentences, is on the
//! tests in `tests/doc_004_10.rs`.
//!
//! It uses the WP5 shared module (`checks/wp5`, compiled in with `#[path]`). Its own variants: the
//! section is the first heading whose whole title is
//! `measurement[\s-]+extensions?` (`re.fullmatch`, `re.I`); the three fields
//! are found by keyword in the Meaning cell of any field table of the
//! section (one key per row, first free key); the unit test has a
//! look-behind and a look-ahead `[A-Za-z]` under `re.I` ([`fields`]); the
//! validity units are the rows of the non-field tables and the bullet
//! lines, and the reason-token scan also reads every prose block
//! ([`forms`]); AC5 runs whatever the section's state, and `message` is
//! iterated without `or []` ([`toml_side`]).
//!
//! **Header and code disagree** on the enumeration: the header says "the
//! closed 13-name enumeration"; the code replaces the 13 names with those
//! found in the Rejection reasons tables when at least 10 are found
//! (004-10:605-606). The port follows the code.
//!
//! The repository-level assertions (`wire-check` reports "checks clean";
//! `wire_tables` stays 12 passed, 0 failed, 0 ignored; the design checker
//! prints OK) are in `scripts/repo-state.d/004-10.sh`.

// The WP5 shared module, compiled into each of the four ports (no port
// imports another, lead ruling R3) until the library takes it.
#[allow(dead_code, clippy::duplicate_mod)]
#[path = "../wp5/mod.rs"]
mod wp5;
mod denials;
mod fields;
mod forms;
mod size;
mod toml_side;

use wp5::md::{blocks, heading_index, is_placeholder, section_of, strip_fences, untagged_blocks};
use wp5::py::{head, read_universal, shadow};
use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::rx;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
/// The machine-readable tables.
pub const WIRE_TOML: &str = "doc/design/spec/wire.toml";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-10",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target { file: WIRE_MD, heading: Some("^Measurement extension$") },
        Target { file: WIRE_MD, heading: Some("^Rejection reasons$") },
        Target { file: WIRE_TOML, heading: None },
    ],
    keywords: &["saturate", "wrap", "fits", "whole", "receiver"],
    known_differences: KNOWN,
    denials: DENIALS,
    toml: crate::doccheck::policy::TomlReader::SubsetReader,
};

use denials::DENIALS;

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[KnownDifference {
    label: r"_crashes$",
    class: DiffClass::ScriptCrashed,
    reason: "the script raises on these inputs (TypeError: an extension row without an offset digit meets `None - int` at 004-10:1081; `message` a number is not iterable; a list as a rejection reason is unhashable) and prints no FAIL line; the port reports the same input as a CRASH failure",
}];

/// wire.md as the script holds it.
pub(crate) struct Ctx {
    pub open: bool,
    pub section: String,
    pub prose: String,
    /// The prose blocks not starting with `|`.
    pub prose_blks: Vec<String>,
    /// hop, residence, delay.
    pub ext: [Option<fields::Ext>; 3],
    pub members: Vec<String>,
}

impl Ctx {
    /// The three fields when all were found.
    pub fn all(&self) -> Option<[&fields::Ext; 3]> {
        Some([self.ext[0].as_ref()?, self.ext[1].as_ref()?, self.ext[2].as_ref()?])
    }
}

fn load(wire: &str, report: &mut Report) -> Ctx {
    let owned: Vec<String> = splitlines(&shadow(wire)).into_iter().map(str::to_string).collect();
    let lines: Vec<&str> = owned.iter().map(String::as_str).collect();
    let heads = heading_index(&lines);
    let sec = heads.iter().position(|h| rx(r"(?i)^(?:measurement[\s-]+extensions?)$").is_match(&wp5::py::fold_i(&h.2)));
    let mut section = String::new();
    let mut open = false;
    match sec {
        None => {
            for ac in ["AC1", "AC2", "AC3", "AC4", "DoD"] {
                report.fail(ac, "no 'Measurement extension' section heading in wire.md");
            }
        }
        Some(k) => {
            section = section_of(&lines, &heads, k);
            open = !is_placeholder(&section);
            if !open {
                for ac in ["AC1", "AC2", "AC3", "AC4", "DoD"] {
                    report.fail(ac, "the Measurement extension section is still the 004-10 placeholder");
                }
            }
        }
    }
    let prose = strip_fences(&section);
    let prose_blks = blocks(&prose).into_iter().filter(|b| !b.trim_start_matches(wp5::py::py_space).starts_with('|')).collect();
    let (ext, found) = fields::extension_rows(&section);
    if open && !found {
        report.fail("AC1", "no field table (Offset/Width/Type/Meaning/Valid range) in the Measurement extension section");
    }
    Ctx { open, prose, prose_blks, ext, members: wp5::members::members(&lines, &heads), section }
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let Some(wire) = read_universal(docs, WIRE_MD) else {
        for ac in ["AC1", "AC2", "AC3", "AC4", "AC5"] {
            report.fail(ac, format!("{WIRE_MD} does not exist"));
        }
        report.fail("DoD", "cannot check — wire.md does not exist");
        return report;
    };
    let ctx = load(&wire, &mut report);
    fields::ac1(&ctx, &mut report);
    let stated = size::ac2(&ctx, &mut report);
    forms::ac3(&ctx, &mut report);
    forms::ac4(&ctx, &mut report);
    if let Err(crash) = toml_side::ac5(docs, &ctx, stated.as_ref(), &mut report) {
        let mut crashed = Report::default();
        crashed.fail("CRASH", format!("the script raised an exception: {crash}"));
        return crashed;
    }
    if ctx.open {
        for first in untagged_blocks(&ctx.section) {
            let more = if first.chars().count() > 90 { " …" } else { "" };
            report.fail(
                "DoD",
                format!(
                    "this block of the Measurement extension section carries no evidence tag ([code] [measured] [kernel-doc] [standard] [memory] [decision]): {}{more}",
                    head(&first, 90)
                ),
            );
        }
    }
    report
}
