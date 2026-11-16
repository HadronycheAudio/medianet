//! Port of `checks/004-14.sh`: the media packet size budget at 1 472 and
//! 1 280 bytes (ticket 004-14).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_14.rs`. The script's variants, named where used:
//!
//! * the section: the level-2 heading `Size budget` by `re.fullmatch` (any
//!   case), ending at the next heading of level 2 or higher; the lines of
//!   the file are Python's `splitlines`; fences are the `Backticks`
//!   policy ([`FencePolicy::Backticks`]);
//! * tables: a run of lines starting with `|` whose second line holds a
//!   dash and only `\s : | -` is a table (the 004 separator test, not the
//!   library's cell-wise one), cells split on unescaped pipes
//!   ([`RowSplit::Escaped`]), and each table carries the two non-table
//!   lines within five lines above it as its caption;
//! * numbers: `\d{1,3}([ NBSP NNBSP]\d{3})+|\d+`, commas dropped, a cell
//!   with `=` read after its last `=`, parentheses ignored, a dash or
//!   "omitted" read as 0, a leading minus or "deficit" negating;
//! * wire.toml: the script's own line reader ([`pytoml`]), not the crate's
//!   parser: it reads files a real parser rejects and its verdict is what
//!   has been passing;
//! * the evidence rule: units where `+` also starts a list item, words
//!   `[A-Za-z0-9]+`, at least 5, any tag of the six in any case
//!   ([`TagPolicy::ALNUM_004_14`], whose segmentation the library does not
//!   implement: the units are built here);
//! * an open item: a unit starting with `open item` after `* _` and blanks,
//!   naming `owner` and `falsifier` anywhere in it.
//!
//! Where the script would raise (a `None` in arithmetic, a missing key) the
//! port reports criterion `CRASH`; where it prints one `FAIL` and exits, the
//! port reports only that failure.
//!
//! Its repository-level assertion (when wire.toml carries no `[budget]`
//! table, `cargo run --bin wire-check` must pass and print the default and
//! fallback totals) is in `scripts/repo-state-checks.sh`.
//!
//! [`FencePolicy::Backticks`]: crate::doccheck::policy::FencePolicy::Backticks
//! [`RowSplit::Escaped`]: crate::doccheck::policy::RowSplit::Escaped
//! [`TagPolicy::ALNUM_004_14`]: crate::doccheck::evidence::TagPolicy::ALNUM_004_14

mod allow;
mod cases;
mod comp;
mod keys;
pub(crate) mod pytoml;
pub(crate) mod section;
mod tail;
pub(crate) mod util;

use super::{CheckSpec, Target};
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};
use util::{Fails, R, Stop};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
const CONSTANTS: &str = "doc/design/02-constants.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-14",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Size budget$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Budget on a 1 472-byte path$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Budget on a fallback path"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Announcement schedule and omission rule$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Shared packets and the controller allowance$"),
        },
        Target {
            file: WIRE_TOML,
            heading: None,
        },
        Target {
            file: "doc/design/02-constants.md",
            heading: None,
        },
    ],
    keywords: &["holds", "falsified", "omitted", "upper bound"],
    known_differences: &[],
};

/// The three documents, or the script's `FAIL setup` exit.
fn read_all(docs: &Docs) -> R<(String, String, String)> {
    let read = |path: &str, label: &str| {
        docs.read(path).map(|t| t.to_string()).ok_or_else(|| {
            Stop::Exit("setup", format!("cannot read {label} at {path}"))
        })
    };
    Ok((
        read(WIRE_MD, "wire.md")?,
        read(WIRE_TOML, "wire.toml")?,
        read(CONSTANTS, "02-constants.md")?,
    ))
}

fn body(docs: &Docs) -> R<Fails> {
    let (wire, toml_src, const_src) = read_all(docs)?;
    let c = comp::consts(&const_src)?;
    let parsed = pytoml::parse(&toml_src, pytoml::Flavor::Budget)
        .map_err(|_| Stop::Exit("setup", "wire.toml does not parse".into()))?;
    let mut f = Fails::default();
    let k = comp::components(&parsed, &c, &mut f)?;
    let sec = section::Sec::read(&wire);
    if !sec.found {
        f.add("AC1", "wire.md has no level-2 heading 'Size budget'");
    }
    let found = cases::cases(&sec, &k, &c, &mut f);
    cases::coverage(&found, &k, &c, &sec, &mut f);
    cases::omission(&sec, &found, &k, &mut f);
    allow::verdict(&sec, &k, &c, &mut f);
    let flat = keys::budget_flat(&parsed);
    let rows = allow::allowance(&sec, &k, &c, &mut f)?;
    if let Some(flat) = &flat {
        keys::budget_keys(flat, &found, &rows, &k, &c, &mut f);
    }
    tail::dod(&sec, &mut f);
    tail::pointers(&wire, &sec, &mut f);
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
                report.check("OK", true, "6/6 acceptance criteria for 004-14");
            }
        }
        Err(Stop::Exit(ac, msg)) => report.fail(ac, msg),
        Err(Stop::Crash(why)) => report.fail("CRASH", format!("the script raises: {why}")),
    }
    report
}
