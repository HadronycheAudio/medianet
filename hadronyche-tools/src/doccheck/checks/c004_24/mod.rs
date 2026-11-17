//! Port of `checks/004-24.sh`: the regime stamp message — its sections, its
//! version and its fetch request (ticket 004-24).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_24.rs`. The script's variants, named where used:
//! wire.md and the constants table read with digit groups joined ("1 252"
//! reads 1252); headings found by `re.search` at an exact level
//! ([`HeadingMatch::RegexAtLevel`], through 004-27's `span_of`), the
//! Rejection reasons by the unanchored `rejection[-\s]+reasons?`; `###`
//! subsections ending at the next heading of level 3 or higher; tables of the
//! 004-16 to 004-29 family ([`py::tables`]: cells split on unescaped pipes,
//! [`RowSplit::Escaped`], and only runs with the scripts' separator row);
//! sentences cut after `.`, `;` or `:`; reason tokens
//! `[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+`; the non-rejection outcome pattern
//! without "no reason"; the untagged-claim rule per line
//! ([`dod::LINES_004_24`]); open items as any prose or table block that
//! mentions "open item" ([`dod::MENTION_BLOCKS_004_24`]); the Rejection
//! reasons only counted (13), not pinned by name.
//!
//! The family helpers the script shares verbatim with 004-27
//! (`is_field_table`, `width_of`, `fields_of`, `subsections`, `find_sub`,
//! `span_of`) are 004-27's port's, reused unchanged.
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` has 0
//! failed and at least 12 passed; the checker reads the working tree only
//! and runs no other check) are in `scripts/repo-state-checks.sh`.
//!
//! [`HeadingMatch::RegexAtLevel`]: crate::doccheck::policy::HeadingMatch::RegexAtLevel
//! [`RowSplit::Escaped`]: crate::doccheck::policy::RowSplit::Escaped

pub(crate) mod dod;
pub(crate) mod landed;
pub(crate) mod py;
pub(crate) mod pytoml;
mod rules;
mod stamp;
mod toml_side;
mod validity;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::checks::c004_27::Ctx;
use crate::doccheck::markdown::Markdown;
use crate::doccheck::text::join_digit_groups;
use crate::doccheck::ticket::{CONSTANTS_MD, Constants};
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-24",
    run,
    script_env: &[("WIRE_MD", WIRE_MD)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Regime stamp$"),
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
            heading: Some("^Hello layout$"),
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
    keywords: &["unicast", "multicast", "stale", "identical", "control"],
    known_differences: KNOWN,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// Where the script stops with a Python exception: the checks after it
/// never run, so the port stops there too (the run is red either way).
#[derive(Debug)]
pub(crate) struct Stop(pub String);

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let wire = match docs.read(WIRE_MD) {
        Some(t) => join_digit_groups(&py::py_text(&t)),
        None => {
            report.fail("AC1", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let toml_text = docs.read(WIRE_TOML).map(|t| t.to_string());
    if toml_text.is_none() {
        report.fail("AC5", format!("{WIRE_TOML} does not exist"));
    }
    let consts = Constants::parse(&join_digit_groups(&py::py_text(&docs.read(CONSTANTS_MD).map(|t| t.to_string()).unwrap_or_default())));
    let ctx = Ctx {
        md: Markdown::parse(&wire),
        consts,
        toml_text,
    };
    if let Err(Stop(e)) = checks(&ctx, &mut report) {
        py::stopped(&mut report, "AC5", &e);
    }
    report
}

fn checks(ctx: &Ctx, report: &mut Report) -> Result<(), Stop> {
    let landed = landed::landed(ctx, report);
    let st = stamp::stamp(ctx, &landed, report)?;
    stamp::ac2(ctx, &st, &landed, report)?;
    rules::ac3(&st, &landed, report);
    let reasons = validity::ac4(&st, &landed, report);
    toml_side::ac5(ctx.toml_text.as_deref(), &st, &landed, &reasons, report)?;
    dod::dod(&st, report);
    Ok(())
}
