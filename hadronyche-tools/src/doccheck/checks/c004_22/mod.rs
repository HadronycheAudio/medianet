//! Port of `checks/004-22.sh`: the traceability table and the review
//! against the architecture specification (ticket 004-22).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_22.rs`. This module reproduces the script's variant of
//! every shared rule, named where it is used: [`RowSplit::Plain`] tables
//! (an escaped pipe breaks a cell), [`TagPolicy::UNITS_004_22`] (six tags,
//! exact case, units split at list markers, `\S+` words after links are
//! reduced to text), [`OpenItemPolicy::Mention`], heading references
//! resolved by normalised title (with and without the section number) and
//! links by GitHub anchor.
//!
//! The script's repository-level assertions (`cargo run --bin wire-check`
//! exits 0; `cargo test -p hadronyche-tools --test wire_tables` reports at
//! least 12 passed and 0 failed) are in `scripts/repo-state-checks.sh`.
//!
//! [`RowSplit::Plain`]: crate::doccheck::policy::RowSplit::Plain
//! [`TagPolicy::UNITS_004_22`]: crate::doccheck::evidence::TagPolicy::UNITS_004_22
//! [`OpenItemPolicy::Mention`]: crate::doccheck::items::OpenItemPolicy::Mention

mod audit;
mod dod;
mod record;
mod refs;
mod trace;
mod work3;

use std::collections::BTreeMap;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::markdown::{Markdown, Span, norm_title};
use crate::doccheck::policy::RowSplit;
use crate::doccheck::tables::Row;
use crate::doccheck::ticket::Ticket;
use crate::doccheck::wire_toml::{WireToml, fields, int, text};
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
/// The machine-readable tables.
pub const WIRE_TOML: &str = "doc/design/spec/wire.toml";
/// The architecture specification.
pub const ARCH_MD: &str = "doc/design/01-architecture.md";
/// Where a separate review record may live.
pub const REPORTS_DIR: &str = "doc/design/reports";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-22",
    run,
    script_env: &[("WIRE_MD", WIRE_MD)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Traceability table$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Review record$"),
        },
        Target {
            file: WIRE_TOML,
            heading: None,
        },
        Target {
            file: ARCH_MD,
            heading: Some("Protocol and packetisation"),
        },
    ],
    keywords: &["destination-specific", "every known", "allows"],
    known_differences: KNOWN,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// Everything the check reads, parsed once.
pub(crate) struct Inputs {
    pub wire: Markdown,
    pub wire_text: String,
    /// wire.toml field names (fields of every message, then reserved
    /// entries) with their widths (`None` when an entry has no width).
    pub widths: BTreeMap<String, Vec<Option<i64>>>,
    /// The media-layer messages: (name, field names).
    pub media: Vec<(String, Vec<String>)>,
    pub arch: Markdown,
    pub ticket: String,
}

/// The Traceability section.
pub(crate) struct Trace {
    pub span: Span,
    pub slug: String,
    pub rows: Vec<Row>,
}

fn media_messages(toml: &WireToml) -> Vec<(String, Vec<String>)> {
    let pick = crate::doccheck::text::rx(r"content|audio message|dense controller|sparse keyed|transport state");
    toml.messages()
        .into_iter()
        .filter(|m| {
            let name = text(m, "name");
            name == "media packet" || text(m, "section_of") == "media packet" || pick.is_match(name)
        })
        .map(|m| {
            let names = fields(m).iter().map(|f| text(f, "name").to_string()).collect();
            (text(m, "name").to_string(), names)
        })
        .collect()
}

fn load(docs: &Docs, report: &mut Report) -> Option<Inputs> {
    let ticket = Ticket::load(docs, "004-22");
    for (path, what) in [(WIRE_MD, "wire.md"), (WIRE_TOML, "wire.toml"), (ARCH_MD, "01-architecture.md")] {
        if docs.read(path).is_none() {
            report.fail("WIRE", format!("cannot read {what} at '{path}'"));
            return None;
        }
    }
    let Some(ticket) = ticket else {
        report.fail("WIRE", "cannot read the ticket at ''");
        return None;
    };
    let toml = match WireToml::load(docs) {
        Ok(t) => t,
        Err(e) => {
            report.fail("WIRE", format!("wire.toml does not parse: {e}"));
            return None;
        }
    };
    let widths = toml
        .field_info()
        .into_iter()
        .map(|(k, v)| (k, v.iter().map(|f| int(f, "width")).collect()))
        .collect();
    let wire_text = docs.read(WIRE_MD)?.to_string();
    Some(Inputs {
        wire: Markdown::parse(&wire_text),
        wire_text,
        widths,
        media: media_messages(&toml),
        arch: Markdown::parse(&docs.read(ARCH_MD)?),
        ticket: ticket.text,
    })
}

fn find_trace(inp: &Inputs) -> Option<Trace> {
    let mut heads: Vec<_> = inp
        .wire
        .heads
        .iter()
        .filter(|h| norm_title(&h.text).starts_with("traceability"))
        .collect();
    heads.sort_by_key(|h| h.level);
    let h = heads.first()?;
    let span = inp.wire.section(h);
    let rows = inp
        .wire
        .tables_with(span, RowSplit::Plain)
        .into_iter()
        .flat_map(|t| t.rows)
        .collect();
    Some(Trace {
        span,
        slug: h.slug.clone(),
        rows,
    })
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let Some(inp) = load(docs, &mut report) else {
        return report;
    };
    let Some(lists) = trace::constant_lists(&inp.ticket) else {
        report.fail("AC2", "the ticket has no 'Constants traced:' sentence");
        return report;
    };
    let trace = find_trace(&inp);
    if trace.is_none() {
        report.fail(
            "AC1",
            format!("{WIRE_MD} has no Traceability section (a heading starting 'Traceability')"),
        );
    }
    trace::ac1(&inp, trace.as_ref(), docs, &mut report);
    trace::ac2(&inp, trace.as_ref(), &lists, &mut report);
    let audit = audit::ac3(&inp, &mut report);
    let rec = record::ac4(&inp, trace.as_ref(), &audit, docs, &mut report);
    if let Some(rec) = &rec {
        work3::ac4_work3(&inp, trace.as_ref(), rec, &mut report);
    }
    dod::dod(&inp, trace.as_ref(), &audit, rec.as_ref(), &mut report);
    report
}
