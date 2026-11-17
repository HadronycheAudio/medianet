//! Port of `checks/004-24.sh`: the regime stamp message — its sections, its
//! version and its fetch request (ticket 004-24).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc/c004_24.rs`; the denial list is `denials.rs`. The script's
//! variants, named where used:
//!
//! * text read as `open()` does, digit groups joined, then `splitlines()`
//!   ([`LineSplit::PySplitlines`]);
//! * the 004 family's helpers ([`crate::doccheck::family004`]): `span_of`
//!   at an exact level, `###` subsections, [`TableRules::FAMILY_004`],
//!   `fields_of`, `width_of`;
//! * the Rejection reasons by [`REJECTION_004_24`] (unanchored
//!   `rejection[-\s]+reasons?`, four-cell rows), only counted (13);
//! * sentences by [`SentenceSplit::COLON_004_24`]; reason tokens
//!   [`CAMEL_DIGITS`]; the non-rejection pattern [`NON_REJECTION_004_24`];
//! * the untagged-claim rule per line ([`LinePolicy::LINES_004_24`]); open
//!   items as any prose or table block mentioning "open item"
//!   ([`OpenItemPolicy::MentionBlocks`]);
//! * wire.toml through the script's own subset reader
//!   ([`TomlReader::Subset004_18`]), every exception of it caught as an AC5
//!   failure (`except Exception`); the values used as Python uses them
//!   ([`crate::doccheck::pyvalue::PyValue`]); the script's uncaught
//!   exceptions (`TypeError` on a missing C-17 bound, a non-iterable TOML
//!   value, `int()` of a non-number) end the run as a crash
//!   ([`crate::doccheck::report::Stop::Crash`]).
//!
//! Its repository-level assertions are in `scripts/repo-state.d/004-24.sh`.
//!
//! [`LineSplit::PySplitlines`]: crate::doccheck::policy::LineSplit::PySplitlines
//! [`TableRules::FAMILY_004`]: crate::doccheck::policy::TableRules::FAMILY_004
//! [`REJECTION_004_24`]: crate::doccheck::rejection_members::REJECTION_004_24
//! [`SentenceSplit::COLON_004_24`]: crate::doccheck::sentences::SentenceSplit::COLON_004_24
//! [`CAMEL_DIGITS`]: crate::doccheck::rejection_members::reasons::CAMEL_DIGITS
//! [`NON_REJECTION_004_24`]: crate::doccheck::rejection_members::reasons::NON_REJECTION_004_24
//! [`LinePolicy::LINES_004_24`]: crate::doccheck::evidence::LinePolicy::LINES_004_24
//! [`OpenItemPolicy::MentionBlocks`]: crate::doccheck::items::OpenItemPolicy::MentionBlocks
//! [`TomlReader::Subset004_18`]: crate::doccheck::policy::TomlReader::Subset004_18

mod denials;
mod dod;
mod hello;
mod rules;
mod stamp;
mod toml_side;
mod validity;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::family004::Ctx;
use crate::doccheck::markdown::Markdown;
use crate::doccheck::policy::{LineSplit, TomlReader};
use crate::doccheck::pycompat::universal_newlines;
use crate::doccheck::pyvalue::Dict;
use crate::doccheck::report::Flow;
use crate::doccheck::text::join_digit_groups;
use crate::doccheck::ticket::{CONSTANTS_MD, Constants};
use crate::doccheck::tomlsubset;
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// How the script reads its documents.
const LINES: LineSplit = LineSplit::PySplitlines;

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-24",
    run,
    script_env: &[("WIRE_MD", WIRE_MD)],
    targets: &[
        Target { file: WIRE_MD, heading: Some("^Regime stamp$") },
        Target { file: WIRE_MD, heading: Some("^Common control header$") },
        Target { file: WIRE_MD, heading: Some("^Control message type codes$") },
        Target { file: WIRE_MD, heading: Some("^Hello layout$") },
        Target { file: WIRE_MD, heading: Some("^Rejection reasons$") },
        Target { file: WIRE_TOML, heading: None },
        Target { file: CONSTANTS_MD, heading: None },
    ],
    keywords: &["unicast", "multicast", "stale", "identical", "control"],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    toml: TomlReader::Subset004_18,
    fail_reader: crate::doccheck::equiv::script::DEFAULT_READER,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// The documents as the script reads them.
pub(crate) struct Input {
    pub ctx: Ctx,
    /// `WIRE`: the normalised wire.md.
    pub wire: String,
    pub toml: Dict,
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let wire = match docs.read(WIRE_MD) {
        Some(t) => LINES.normalize(&join_digit_groups(&universal_newlines(&t))),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let toml_text = docs.read(WIRE_TOML).map(|t| universal_newlines(&t));
    if toml_text.is_none() {
        report.fail("AC5", format!("{WIRE_TOML} does not exist"));
    }
    let constants = docs.read(CONSTANTS_MD).map(|t| t.to_string()).unwrap_or_default();
    let consts = Constants::parse(&LINES.normalize(&join_digit_groups(&universal_newlines(&constants))));
    let input = Input {
        ctx: Ctx { md: Markdown::parse(&wire), consts },
        wire,
        toml: Dict::default(),
    };
    let flow = checks(input, toml_text.as_deref(), &mut report);
    report.finish(flow)
}

fn checks(mut input: Input, toml_text: Option<&str>, report: &mut Report) -> Flow<()> {
    let landed = hello::landed(&input, report)?;
    let st = stamp::stamp(&input.ctx, &landed, report)?;
    stamp::ac2(&input.ctx, &st, &landed, report)?;
    rules::ac3(&st, &landed, report);
    let reasons = validity::ac4(&st, &landed, report);
    // TOML = parse_toml(TOML_TEXT), any exception caught as an AC5 failure
    input.toml = tomlsubset::parse(toml_text.unwrap_or("")).unwrap_or_else(|e| {
        report.fail("AC5", format!("wire.toml does not parse with the subset reader: {e}"));
        Dict::default()
    });
    toml_side::ac5(&input.toml, &st, &landed, &reasons, report)?;
    dod::dod(&st, report);
    Ok(())
}
