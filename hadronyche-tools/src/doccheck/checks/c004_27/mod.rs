//! Port of `checks/004-27.sh`: the path-MTU probe pair — the probe request
//! and reply, their exemption and their validity rules (ticket 004-27).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_27.rs`; the denial list is `denials.rs`. The script's
//! variants, named where used:
//!
//! * text read as `open()` does, digit groups joined ("1 280" reads 1280,
//!   Python's `\d`), then `splitlines()` ([`LineSplit::PySplitlines`],
//!   [`LINES`]): U+2028, form feed and the other Python boundaries start a
//!   line;
//! * headings found by `re.search` at an exact level
//!   ([`HeadingMatch::RegexAtLevel`]); `###` subsections ending at the next
//!   heading of level 3 or higher;
//! * tables by [`TableRules::FAMILY_004`] (escaped pipes, the line-regex
//!   separator, headerless runs dropped);
//! * [`TagPolicy::PROSE_004_27`]; [`OpenItemPolicy::LeadingWithTicket`];
//! * wire.toml read through the faithful port of the script's own subset
//!   reader ([`TomlReader::Subset004_18`], `doccheck::tomlsubset`), its
//!   values queried as Python did (`int()`, `isinstance`, `==` across int
//!   and float, iteration over a string's characters); the script's
//!   uncaught exceptions are modelled as crashes (`Report::crash`);
//! * the repertoire guard first, on wire.md, wire.toml and the constants
//!   (R13);
//! * integers checked (`pycompat::parse_int`): a number beyond `i128` is
//!   reported, never wrapped.
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` has 0
//! failed and at least 12 passed) are in `scripts/repo-state.d/004-27.sh`;
//! its "reads the working tree only, runs no other check" is the library
//! test `the clippy configuration (hadronyche-tools/clippy.toml) and doccheck::testing::source_rules`.
//!
//! [`LineSplit::PySplitlines`]: crate::doccheck::policy::LineSplit::PySplitlines
//! [`HeadingMatch::RegexAtLevel`]: crate::doccheck::policy::HeadingMatch::RegexAtLevel
//! [`TableRules::FAMILY_004`]: crate::doccheck::policy::TableRules::FAMILY_004
//! [`TagPolicy::PROSE_004_27`]: crate::doccheck::evidence::TagPolicy::PROSE_004_27
//! [`OpenItemPolicy::LeadingWithTicket`]: crate::doccheck::items::OpenItemPolicy::LeadingWithTicket
//! [`TomlReader::Subset004_18`]: crate::doccheck::policy::TomlReader::Subset004_18

mod denials;
mod dod;
mod landed;
mod messages;
mod prose;
mod toml_side;
mod validity;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::family004::Ctx;
use crate::doccheck::pyvalue::Dict;
use crate::doccheck::{repertoire, tomlsubset};
use crate::doccheck::markdown::Markdown;
use crate::doccheck::policy::{LineSplit, TomlReader};
use crate::doccheck::pycompat::universal_newlines;
use crate::doccheck::text::join_digit_groups;
use crate::doccheck::ticket::{CONSTANTS_MD, Constants};
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// How the 004 family read its documents: `open()` (universal newlines),
/// then `splitlines()`.
pub const LINES: LineSplit = LineSplit::PySplitlines;

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-27",
    run,
    script_env: &[("WIRE_MD", WIRE_MD)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Path-MTU probe pair$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Common control header$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Control message type codes$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Refusal reasons$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Admission request$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Rejection reasons$"),
        },
        Target {
            file: WIRE_TOML,
            heading: None,
        },
        Target {
            file: CONSTANTS_MD,
            heading: None,
        },
    ],
    keywords: &["exempt", "padded", "zero"],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    if !repertoire::guard(docs, &[WIRE_MD, WIRE_TOML, CONSTANTS_MD], &mut report) {
        return report;
    }
    // read_text (universal newlines), norm (digit groups), splitlines
    let wire = match docs.read(WIRE_MD) {
        Some(t) => LINES.normalize(&join_digit_groups(&universal_newlines(&t))),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let toml_text = docs.read(WIRE_TOML).map(|t| universal_newlines(&t));
    if toml_text.is_none() {
        report.fail("AC4", format!("{WIRE_TOML} does not exist"));
    }
    // TOML = parse_toml(TOML_TEXT), any exception caught as an AC4 failure
    let toml = tomlsubset::parse(toml_text.as_deref().unwrap_or("")).unwrap_or_else(|e| {
        report.fail("AC4", format!("wire.toml does not parse with the subset reader: {e}"));
        Dict::default()
    });
    let constants = docs.read(CONSTANTS_MD).map(|t| t.to_string()).unwrap_or_default();
    let consts = Constants::parse(&LINES.normalize(&join_digit_groups(&universal_newlines(&constants))));
    let ctx = Ctx {
        md: Markdown::parse(&wire),
        consts,
    };
    let landed = landed::landed(&ctx, &toml, &mut report);
    let probe = messages::probe(&ctx, &landed, &mut report);
    prose::ac1_prose(&probe, &landed, &mut report);
    prose::ac2(&probe, &landed, &mut report);
    validity::ac3(&probe, &landed, &mut report);
    dod::dod(&probe, &mut report);
    toml_side::ac4(&probe, &landed, &mut report);
    report
}

