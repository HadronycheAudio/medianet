//! Port of `checks/004-08.sh`: the slice header and the send-timestamp
//! format (ticket 004-08). The criterion map, with the ticket sentences, is
//! on the tests in `tests/doc_004_08.rs`.
//!
//! It uses the WP5 shared module (`checks/wp5`, compiled in with `#[path]`) (Python text semantics,
//! the scripts' TOML reader, the Markdown helpers and the every-block tag
//! rule). Its own variants: the section is the first heading whose title
//! contains `slice\s+header` (`re.I`); durations are Python floats with the
//! look-behind `(?<![a-zA-Zµ])` under `re.I` ([`send_ts`]); the validity
//! units are the rows of every table of the section and its bullet lines,
//! leaving out the tables with a field-table header only when a field table
//! was found (004-08:1209-1217).
//!
//! **Header and code disagree** on the enumeration: the header says "the
//! closed 13-name enumeration"; the code replaces the 13 names with those
//! found in the Rejection reasons tables when at least 10 are found
//! (004-08:783-784). The port follows the code.
//!
//! The repository-level assertions (`wire-check` reports "checks clean";
//! `wire_tables` stays 12 passed, 0 failed, 0 ignored; the design checker
//! prints OK) are in `scripts/repo-state.d/004-08.sh`. The script does not
//! run checks/004-01.sh.

// The WP5 shared module, compiled into each of the four ports (no port
// imports another, lead ruling R3) until the library takes it.
#[allow(dead_code, clippy::duplicate_mod)]
#[path = "../wp5/mod.rs"]
mod wp5;
mod denials;
mod forms;
mod regime;
mod send_ts;
mod table;
mod toml_side;

use wp5::md::{blocks, heading_index, is_placeholder, section_of, strip_fences};
use wp5::py::{read_universal, shadow, si};
use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
/// The machine-readable tables.
pub const WIRE_TOML: &str = "doc/design/spec/wire.toml";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-08",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target { file: WIRE_MD, heading: Some("^Slice header$") },
        Target { file: WIRE_MD, heading: Some("^Rejection reasons$") },
        Target { file: WIRE_TOML, heading: None },
    ],
    keywords: &["wrap", "clock", "default", "regime", "complete"],
    known_differences: KNOWN,
    denials: DENIALS,
    toml: crate::doccheck::policy::TomlReader::SubsetReader,
};

use denials::DENIALS;

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[KnownDifference {
    label: r"_crashes$",
    class: DiffClass::ScriptCrashed,
    reason: "the script raises on these inputs (OverflowError: 2^N with N >= 1024, or a frame count beyond a float, converted to float; AttributeError: `message` a number under its TOML reader) and prints no FAIL line; the port reports the same input as a CRASH failure",
}];

/// wire.md as the script holds it.
pub(crate) struct Ctx {
    pub open: bool,
    pub section: String,
    pub prose: String,
    /// `blocks(section_prose)`.
    pub blks: Vec<String>,
    pub field: Option<table::Fields>,
    pub members: Vec<String>,
}

fn load(wire: &str, report: &mut Report) -> Ctx {
    let owned: Vec<String> = splitlines(&shadow(wire)).into_iter().map(str::to_string).collect();
    let lines: Vec<&str> = owned.iter().map(String::as_str).collect();
    let heads = heading_index(&lines);
    let sec = heads.iter().position(|h| si(r"slice\s+header", &h.2));
    let mut section = String::new();
    let mut open = false;
    match sec {
        None => {
            for ac in ["AC1", "AC2", "AC3", "AC4", "AC5", "WS5", "DoD"] {
                report.fail(ac, "no 'Slice header' section heading in wire.md");
            }
        }
        Some(k) => {
            section = section_of(&lines, &heads, k);
            open = !is_placeholder(&section);
            if !open {
                for ac in ["AC1", "AC2", "AC3", "AC4", "AC5", "WS5", "DoD"] {
                    report.fail(ac, "the Slice header section is still the 004-08 placeholder");
                }
            }
        }
    }
    let prose = strip_fences(&section);
    let field = table::field_table(&section);
    if open && field.is_none() {
        report.fail("AC2", "no field table (Offset/Width/Type/Meaning/Valid range) in the Slice header section");
    }
    Ctx {
        open,
        blks: blocks(&prose),
        prose,
        field,
        members: wp5::members::members(&lines, &heads),
        section,
    }
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let Some(wire) = read_universal(docs, WIRE_MD) else {
        for ac in ["AC1", "AC2", "AC3", "AC4", "AC5"] {
            report.fail(ac, format!("{WIRE_MD} does not exist"));
        }
        report.fail("WS5", "cannot check — wire.md does not exist");
        report.fail("DoD", "cannot check — wire.md does not exist");
        return report;
    };
    let ctx = load(&wire, &mut report);
    let outcome = send_ts::ac1(&ctx, &mut report).and_then(|()| {
        let slice_bytes = table::ac2(&ctx, &mut report);
        regime::ac3(&ctx, &mut report);
        regime::ws5(&ctx, &mut report);
        forms::ac4(&ctx, &mut report);
        toml_side::ac5(docs, &ctx, slice_bytes, &mut report)
    });
    if let Err(crash) = outcome {
        let mut crashed = Report::default();
        crashed.fail("CRASH", format!("the script raised an exception: {crash}"));
        return crashed;
    }
    regime::dod(&ctx, &mut report);
    report
}
