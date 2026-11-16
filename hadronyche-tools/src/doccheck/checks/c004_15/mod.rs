//! Port of `checks/004-15.sh`: the on-wire size of one default packet, the
//! bit rates computed from the final layout, the difference from the
//! design's derived figures and the controller-stream wire cost of C-219
//! (ticket 004-15).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_15.rs`. The script's variants, named where used:
//!
//! * the subsection: the first heading, inside the level-2 `Size budget`
//!   section or the first level-2 heading after it, whose title matches
//!   `wire size|capacity|recomput|bit rate|on the wire`, ending at the next
//!   heading of its own or a higher level; lines are Python's `splitlines`;
//!   fences are [`FencePolicy::Backticks`];
//! * units: paragraphs (a blank line, a heading, a table or a list marker
//!   ends one) and table rows, each row carrying its caption, header and
//!   cells; tables need the 004 separator test (a dash and only
//!   `\s : | -`), cells split on unescaped pipes ([`RowSplit::Escaped`]);
//! * numbers: `INT_RE`, `FLOAT_RE`, `PCT_RE`, `MBIT_RE` with their
//!   lookarounds ([`num`]), tolerances `tol_of` (half the last shown
//!   decimal, 0.5 for an integer) plus `1e-9`, floats as the script's
//!   Python floats;
//! * equations: `a + b = c`, `x`, `×`, `*` and `−` read as operators,
//!   evaluated with precedence, the right side scaled by its unit;
//! * wire.toml: the script's own line reader (the 004-14 reader with
//!   strings kept as written and an uncaught error for `[[x]]` after
//!   `[x]`), not the crate's parser;
//! * the evidence rule: units of at least 5 `[A-Za-z0-9]+` words
//!   ([`TagPolicy::ALNUM_004_14`]'s words and tag set, the units built
//!   here), any tag of the six in any case.
//!
//! Where the script would raise (a missing key, a `None` in arithmetic),
//! the port reports criterion `CRASH`; where it prints `FAIL` lines and
//! exits, the port reports those failures.
//!
//! The script has no repository-level assertion.
//!
//! [`FencePolicy::Backticks`]: crate::doccheck::policy::FencePolicy::Backticks
//! [`RowSplit::Escaped`]: crate::doccheck::policy::RowSplit::Escaped
//! [`TagPolicy::ALNUM_004_14`]: crate::doccheck::evidence::TagPolicy::ALNUM_004_14

mod ac1;
mod ac2;
mod ac3;
mod ac4;
mod ac5;
mod dod;
pub(crate) mod num;
mod setup;
mod units;

use super::c004_14::util::Stop;
use super::{CheckSpec, Target};
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};
use num::{Fails, R};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
const CONSTANTS: &str = "doc/design/02-constants.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-15",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Wire size and capacity recompute$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Budget on a 1 472-byte path$"),
        },
        Target {
            file: WIRE_TOML,
            heading: None,
        },
        Target {
            file: CONSTANTS,
            heading: None,
        },
    ],
    keywords: &["exceed", "within", "declared", "stand"],
    known_differences: &[],
};

fn read_all(docs: &Docs) -> R<(String, String, String)> {
    let read = |path: &str, label: &str| {
        docs.read(path)
            .map(|t| t.to_string())
            .ok_or_else(|| Stop::Exit("setup", format!("cannot read {label} at {path}")))
    };
    Ok((read(WIRE_MD, "wire.md")?, read(WIRE_TOML, "wire.toml")?, read(CONSTANTS, "02-constants.md")?))
}

fn body(docs: &Docs) -> R<Fails> {
    let (wire, toml_src, const_src) = read_all(docs)?;
    let mut f = Fails::default();
    let k = setup::calc(&const_src, &toml_src, &mut f)?;
    let Some(sub) = units::read(&wire) else {
        f.add("AC1", "no heading naming the wire size or the capacity recompute, inside 'Size budget' or directly after it");
        return Ok(f);
    };
    let eqs = units::all_equations(&sub, &mut f)?;
    dod::dod(&sub, &mut f)?;
    ac1::wire_size(&sub, &eqs, &k, &mut f)?;
    ac1::ethernet(&sub, &k, &mut f)?;
    ac1::rates(&sub, &k, &mut f)?;
    ac2::differences(&sub, &k, &mut f)?;
    ac2::against_c22(&sub, &k, &mut f)?;
    ac2::share_against_c21(&sub, &k, &mut f);
    ac2::per_channel(&sub, &k, &mut f);
    ac1::size_table(&sub, &k, &mut f)?;
    let sents = ac3::all_sentences(&sub);
    ac3::rule(&sub, &sents, &k, &mut f);
    ac3::declared_rate(&sents, &k, &mut f)?;
    ac4::controller_cost(&sub, &eqs, &k, &mut f)?;
    ac5::toml_keys(&k, &mut f);
    Ok(f)
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    match body(docs) {
        Ok(f) => {
            for (ac, msg) in f.0 {
                report.fail(ac, msg);
            }
            if report.passed() {
                report.check("OK", true, "004-15 wire size, rates, differences and the C-219 formula");
            }
        }
        Err(Stop::Exit(ac, msg)) => report.fail(ac, msg),
        Err(Stop::Crash(why)) => report.fail("CRASH", format!("the script raises: {why}")),
    }
    report
}
