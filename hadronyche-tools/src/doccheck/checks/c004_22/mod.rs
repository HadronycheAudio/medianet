//! Port of `checks/004-22.sh`: the traceability table and the review
//! against the architecture specification (ticket 004-22).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_22.rs`; the denial list is `denials.rs`. This module
//! reproduces the script's variant of every shared rule, named where it is
//! used:
//!
//! * text read as `open()` does, split with `split("\n")`
//!   ([`LineSplit::Nl`], [`LINES`]);
//! * tables by [`TableRules::T004_22`] (plain `split("|")`, cell-wise
//!   separator, headerless runs kept as rows);
//! * [`TagPolicy::UNITS_004_22`] (six tags, exact case, units split at list
//!   markers, `\S+` words after links are reduced to text);
//! * [`OpenItemPolicy::Mention`];
//! * heading references resolved by normalised title (with and without the
//!   section number), links by GitHub anchor with Python's `\w`, link paths
//!   by [`PathNorm::PyNormpath`] (a leading `..` is kept);
//! * wire.toml as `tomllib` read it ([`TomlReader::Tomllib`]), widths
//!   compared with Python's `==`, the script's crashes (a field without a
//!   name, a message that is not a table) reported under `WIRE`.
//!
//! Its repository-level assertions (`cargo run --bin wire-check` exits 0;
//! `cargo test -p hadronyche-tools --test wire_tables` reports at least 12
//! passed and 0 failed) are in `scripts/repo-state.d/004-22.sh`.
//!
//! [`LineSplit::Nl`]: crate::doccheck::policy::LineSplit::Nl
//! [`TableRules::T004_22`]: crate::doccheck::policy::TableRules::T004_22
//! [`TagPolicy::UNITS_004_22`]: crate::doccheck::evidence::TagPolicy::UNITS_004_22
//! [`OpenItemPolicy::Mention`]: crate::doccheck::items::OpenItemPolicy::Mention
//! [`PathNorm::PyNormpath`]: crate::doccheck::policy::PathNorm::PyNormpath
//! [`TomlReader::Tomllib`]: crate::doccheck::policy::TomlReader::Tomllib

mod audit;
mod denials;
mod dod;
mod record;
mod refs;
mod trace;
mod work3;

use std::collections::BTreeMap;

use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::markdown::{Markdown, Span, norm_title};
use crate::doccheck::policy::{LineSplit, TableRules, TomlReader};
use crate::doccheck::tables::Row;
use crate::doccheck::ticket::Ticket;
use crate::doccheck::wire_toml::{WireToml, display};
use toml::Value;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
/// The machine-readable tables.
pub const WIRE_TOML: &str = "doc/design/spec/wire.toml";
/// The architecture specification.
pub const ARCH_MD: &str = "doc/design/01-architecture.md";
/// Where a separate review record may live.
pub const REPORTS_DIR: &str = "doc/design/reports";

/// How 004-22 read its documents: `open()` (universal newlines), then
/// `split("\n")`.
pub const LINES: LineSplit = LineSplit::Nl;

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
    denials: denials::DENIALS,
    toml: TomlReader::Tomllib,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[KnownDifference {
    label: r"^wire\.toml:",
    class: DiffClass::ScriptCrashed,
    reason: "a mutation that makes wire.toml invalid TOML: the script's tomllib.load raises TOMLDecodeError, which it \
             does not catch, so it crashes with a traceback and no FAIL line; the port reports WIRE \
             'wire.toml does not parse (tomllib raises)'",
}];

/// Everything the check reads, parsed once.
pub(crate) struct Inputs {
    pub wire: Markdown,
    pub wire_text: String,
    /// wire.toml field names (fields of every message, then reserved
    /// entries) with their `width` values (`None` when an entry has none);
    /// compared to a stated width with Python's `==` (`16.0 == 16`).
    pub widths: BTreeMap<String, Vec<Option<Value>>>,
    /// The media-layer messages: (name, field names); a field name that is
    /// not a string is kept as a name no backticked token can equal.
    pub media: Vec<(String, Vec<String>)>,
    pub arch: Markdown,
    pub ticket: String,
}

/// `for x in v` over a TOML value whose items the script then indexes as
/// dicts: an array's tables; an empty table or string iterates nothing;
/// anything else is the script's crash (`AttributeError`, `TypeError`).
fn py_items<'a>(v: Option<&'a Value>, what: &str) -> Result<Vec<&'a toml::Table>, String> {
    let items: Vec<&Value> = match v {
        None => return Ok(Vec::new()),
        Some(Value::Array(a)) => a.iter().collect(),
        Some(Value::Table(t)) if t.is_empty() => return Ok(Vec::new()),
        Some(Value::String(s)) if s.is_empty() => return Ok(Vec::new()),
        Some(other) => return Err(format!("{what} is a {}, which the script iterates and indexes", other.type_str())),
    };
    items
        .into_iter()
        .map(|x| x.as_table().ok_or_else(|| format!("an item of {what} is a {}, not a table", x.type_str())))
        .collect()
}

/// `f["name"]` as a dict key: missing or unhashable is the script's crash;
/// a string is the key; another scalar is a key no string equals.
fn py_name(f: &toml::Table, what: &str) -> Result<String, String> {
    match f.get("name") {
        None => Err(format!("an entry of {what} has no name (KeyError)")),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(v @ (Value::Array(_) | Value::Table(_))) => Err(format!("a name of {what} is a {} (unhashable)", v.type_str())),
        Some(v) => Ok(format!("\u{0}{}", display(v))),
    }
}

/// The script's reading of wire.toml (`FIELD_INFO`, `MEDIA_MSGS`,
/// `EXPECTED`); `Err` where its Python crashed.
/// What the script read from wire.toml: (widths by field name, media messages).
type TomlRead = (BTreeMap<String, Vec<Option<Value>>>, Vec<(String, Vec<String>)>);

fn read_toml(root: &toml::Table) -> Result<TomlRead, String> {
    let mut widths: BTreeMap<String, Vec<Option<Value>>> = BTreeMap::new();
    let messages = py_items(root.get("message"), "message")?;
    for m in &messages {
        for f in py_items(m.get("field"), "a message's field")? {
            widths.entry(py_name(f, "field")?).or_default().push(f.get("width").cloned());
        }
    }
    for r in py_items(root.get("reserved"), "reserved")? {
        widths.entry(py_name(r, "reserved")?).or_default().push(r.get("width").cloned());
    }
    let pick = crate::doccheck::text::rx(r"content|audio message|dense controller|sparse keyed|transport state");
    let mut media = Vec::new();
    for m in messages {
        let name = m.get("name").ok_or("a message has no name (KeyError)")?;
        let is_packet = matches!(name, Value::String(n) if n == "media packet");
        let section = matches!(m.get("section_of"), Some(Value::String(s)) if s == "media packet");
        if !is_packet && !section {
            let Value::String(n) = name else {
                return Err(format!("a message name is a {} (re.search needs a string)", name.type_str()));
            };
            if !pick.is_match(n) {
                continue;
            }
        }
        let fields = py_items(m.get("field"), "a message's field")?
            .into_iter()
            .map(|f| py_name(f, "field"))
            .collect::<Result<Vec<_>, _>>()?;
        media.push((display(name), fields));
    }
    Ok((widths, media))
}

/// The Traceability section.
pub(crate) struct Trace {
    pub span: Span,
    pub slug: String,
    pub rows: Vec<Row>,
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
    let toml = match WireToml::parse(&LINES.normalize(&docs.read(WIRE_TOML)?)) {
        Ok(t) => t,
        Err(e) => {
            report.fail("WIRE", format!("wire.toml does not parse (tomllib raises): {e}"));
            return None;
        }
    };
    let (widths, media) = match read_toml(&toml.root) {
        Ok(x) => x,
        Err(e) => {
            report.fail("WIRE", format!("wire.toml: the script's reader crashes here: {e}"));
            return None;
        }
    };
    let wire_text = LINES.normalize(&docs.read(WIRE_MD)?);
    Some(Inputs {
        wire: Markdown::parse(&wire_text),
        wire_text,
        widths,
        media,
        arch: Markdown::parse(&LINES.normalize(&docs.read(ARCH_MD)?)),
        ticket: LINES.normalize(&ticket.text),
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
        .tables_with(span, TableRules::T004_22)
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
