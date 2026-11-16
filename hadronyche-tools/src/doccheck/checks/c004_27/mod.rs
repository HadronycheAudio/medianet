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
//! * wire.toml parsed by the crate's TOML parser, its values read as the
//!   script's subset reader and Python saw them ([`TomlReader::SubsetReader`]:
//!   `int()`, `isinstance`, `==` across int and float, iteration over a
//!   string's characters); where the subset reader rejects valid TOML the
//!   difference is classified (see `KNOWN`), and its crashes are reported;
//! * integers checked (`pycompat::parse_int`): a number beyond `i128` is
//!   reported, never wrapped.
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` has 0
//! failed and at least 12 passed) are in `scripts/repo-state.d/004-27.sh`;
//! its "reads the working tree only, runs no other check" is the library
//! test `doccheck::checks::tests::checks_read_only_their_docs`.
//!
//! [`LineSplit::PySplitlines`]: crate::doccheck::policy::LineSplit::PySplitlines
//! [`HeadingMatch::RegexAtLevel`]: crate::doccheck::policy::HeadingMatch::RegexAtLevel
//! [`TableRules::FAMILY_004`]: crate::doccheck::policy::TableRules::FAMILY_004
//! [`TagPolicy::PROSE_004_27`]: crate::doccheck::evidence::TagPolicy::PROSE_004_27
//! [`OpenItemPolicy::LeadingWithTicket`]: crate::doccheck::items::OpenItemPolicy::LeadingWithTicket
//! [`TomlReader::SubsetReader`]: crate::doccheck::policy::TomlReader::SubsetReader

mod denials;
mod dod;
mod landed;
mod messages;
mod prose;
mod toml_side;
mod validity;

use crate::doccheck::pycompat::PyStr;
use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::markdown::{Markdown, Span};
use crate::doccheck::policy::{HeadingMatch, LineSplit, TableRules, TomlReader};
use crate::doccheck::pycompat::{IntError, parse_int, universal_newlines};
use crate::doccheck::tables::{Row, Table};
use crate::doccheck::text::{full, join_digit_groups, rx};
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
    toml: TomlReader::SubsetReader,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[KnownDifference {
    label: r"^denial review_1f_",
    class: DiffClass::SubsetTomlReader,
    reason: "the script's hand-written TOML reader cannot parse a triple-quoted string (`note = \"\"\"valid TOML\"\"\"`), \
             fails AC4 'does not parse' and then AC2 on the empty table; the file is valid TOML and no assertion of the \
             script is aimed at string syntax (lead ruling R3)",
}];

/// One row of a five-column field table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    /// Byte offset.
    pub off: i128,
    pub bits: i128,
    /// `E × max M`: M.
    pub maxc: Option<i128>,
    pub ty: String,
    pub meaning: String,
    pub valid: String,
}

impl Field {
    /// Bits at the maximum count (`bits * (maxc or 1)`), saturating: Python
    /// integers do not overflow, and a saturated size equals no size a
    /// document states.
    pub fn size_bits(&self) -> i128 {
        self.bits.saturating_mul(self.maxc.filter(|&m| m != 0).unwrap_or(1))
    }
}

/// `is_field_table`: Offset, Width, Type, Meaning, Valid range.
pub(crate) fn is_field_table(header: &[String]) -> bool {
    let h: Vec<String> = header
        .iter()
        .map(|x| rx(r"[^a-z0-9]").replace_all(&x.to_lowercase(), "").into_owned())
        .collect();
    h.len() >= 5
        && h[0].starts_with("offset")
        && h[1].contains("width")
        && h[2].contains("type")
        && h[3].contains("meaning")
        && h[4].contains("valid")
}

/// A width: (bits, max count).
pub(crate) type Width = (i128, Option<i128>);

/// `width_of`: the (bits, max count) of a Width cell, `Ok(None)` when it is
/// neither form, `Err` when a number is beyond what the check can compare.
pub(crate) fn width_of(cell: &str) -> Result<Option<Width>, IntError> {
    let s = cell.py_strip().replace(['`', '*'], "").replace('X', "x");
    let bits = |n: &str, unit: Option<regex::Match>| -> Result<i128, IntError> {
        let n = parse_int(n)?;
        let unit = unit.map_or("bytes".to_string(), |u| u.as_str().to_lowercase());
        Ok(if unit.starts_with("bit") { n } else { n.saturating_mul(8) })
    };
    if let Some(c) = rx(r"(?i)^(\d+)\s*(bytes?|bits?)?\s*(?:x|×)\s*max\s*(\d+)$").captures(&s) {
        return Ok(Some((bits(&c[1], c.get(2))?, Some(parse_int(&c[3])?))));
    }
    match rx(r"(?i)^(\d+)\s*(bytes?|bits?)?$").captures(&s) {
        Some(c) => Ok(Some((bits(&c[1], c.get(2))?, None))),
        None => Ok(None),
    }
}

fn field_row(r: &Row, place: &str) -> Result<Field, String> {
    if r.cells.len() < 5 {
        return Err(format!("{place}: a row has fewer than five cells: {:?}", &r.cells[..r.cells.len().min(2)]));
    }
    let off = r.cell(0).py_strip();
    if !full(r"\d+", off) {
        return Err(format!("{place}: the Offset cell '{}' is not a byte offset", r.cell(0)));
    }
    let off = parse_int(off).map_err(|e| format!("{place}: {e}"))?;
    let Some((bits, maxc)) = width_of(r.cell(1)).map_err(|e| format!("{place}: {e}"))? else {
        return Err(format!("{place}: the Width cell '{}' is neither a byte count nor 'E × max M'", r.cell(1)));
    };
    Ok(Field {
        off,
        bits,
        maxc,
        ty: r.cell(2).py_strip().to_string(),
        meaning: r.cell(3).py_strip().to_string(),
        valid: r.cell(4).py_strip().to_string(),
    })
}

/// `fields_of`: the rows of a field table, reporting malformed rows under
/// `ac` (a number beyond `i128` is reported too).
pub(crate) fn fields_of(table: Option<&Table>, ac: &str, place: &str, report: &mut Report) -> Vec<Field> {
    let Some(t) = table else { return Vec::new() };
    if !is_field_table(t.head()) {
        report.fail(ac, format!("{place}: the table does not have the five columns Offset, Width, Type, Meaning, Valid range"));
        return Vec::new();
    }
    let mut out = Vec::new();
    for r in &t.rows {
        match field_row(r, place) {
            Ok(f) => out.push(f),
            Err(e) => report.fail(ac, e),
        }
    }
    out
}

/// The 004 family's `parse_tables` ([`TableRules::FAMILY_004`]).
pub(crate) fn tables(text: &str) -> Vec<Table> {
    let md = Markdown::parse(text);
    md.tables_with(md.all(), TableRules::FAMILY_004)
}

/// `first_table(text, is_field_table)`.
pub(crate) fn first_field_table(text: &str) -> Option<Table> {
    tables(text).into_iter().find(|t| is_field_table(t.head()))
}

/// `subsections`: the `###` subsections of `text` as (title, body without
/// the heading line), each ending at the next heading of level 3 or higher.
pub(crate) fn subsections(text: &str) -> Vec<(String, String)> {
    let md = Markdown::parse(text);
    let mut out = Vec::new();
    for (k, h) in md.heads.iter().enumerate() {
        if h.level != 3 {
            continue;
        }
        let end = md.heads[k + 1..].iter().find(|n| n.level <= 3).map_or(md.lines.len(), |n| n.line);
        out.push((h.text.clone(), md.lines[h.line + 1..end].join("\n")));
    }
    out
}

/// `find_sub`: the subsections whose title matches `pattern` (any case).
pub(crate) fn find_sub<'a>(subs: &'a [(String, String)], pattern: &str) -> Vec<&'a (String, String)> {
    let re = rx(&format!("(?i){pattern}"));
    subs.iter().filter(|(t, _)| re.is_match(t)).collect()
}

/// Sentences: `re.split(r"(?<=[.;])\s+", text)` (plus `\n` when `newline`).
pub(crate) fn sentences(text: &str, newline: bool) -> Vec<String> {
    let cut = rx(r"[.;]\s+|\n");
    let mut out = Vec::new();
    let mut start = 0;
    for m in cut.find_iter(text) {
        let s = m.as_str();
        if s == "\n" {
            if newline {
                out.push(text[start..m.start()].to_string());
                start = m.end();
            }
            continue;
        }
        out.push(text[start..m.start() + 1].to_string());
        start = m.end();
    }
    out.push(text[start..].to_string());
    out
}

/// The documents, read once.
pub(crate) struct Ctx {
    pub md: Markdown,
    pub consts: Constants,
    pub toml_text: Option<String>,
}

impl Ctx {
    /// `span_of(pattern, level)`: the section of the first heading at
    /// `level` matching `pattern` (any case).
    pub fn span_of(&self, pattern: &str, level: usize) -> Option<Span> {
        let h = self.md.find_by(HeadingMatch::RegexAtLevel(level), &format!("(?i){pattern}")).into_iter().next()?;
        Some(self.md.section(h))
    }

    /// The text of a span, or `""`.
    pub fn body(&self, span: Option<Span>) -> String {
        span.map(|s| self.md.text(s)).unwrap_or_default()
    }
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
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
    let constants = docs.read(CONSTANTS_MD).map(|t| t.to_string()).unwrap_or_default();
    let consts = Constants::parse(&LINES.normalize(&join_digit_groups(&universal_newlines(&constants))));
    let ctx = Ctx {
        md: Markdown::parse(&wire),
        consts,
        toml_text,
    };
    let landed = landed::landed(&ctx, &mut report);
    let probe = messages::probe(&ctx, &landed, &mut report);
    prose::ac1_prose(&probe, &landed, &mut report);
    prose::ac2(&probe, &landed, &mut report);
    validity::ac3(&probe, &landed, &mut report);
    dod::dod(&probe, &mut report);
    toml_side::ac4(&probe, &landed, &mut report);
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_and_sentences() {
        assert_eq!(width_of("2"), Ok(Some((16, None))));
        assert_eq!(width_of("16 bits"), Ok(Some((16, None))));
        assert_eq!(width_of("1 × max 65453"), Ok(Some((8, Some(65453)))));
        assert_eq!(width_of("`4` X max 3"), Ok(Some((32, Some(3)))));
        assert_eq!(width_of("two"), Ok(None));
        assert!(width_of(&"9".repeat(60)).is_err());
        assert_eq!(sentences("A b. C; d\ne.", false), vec!["A b.", "C;", "d\ne."]);
        assert_eq!(sentences("A b. C\nd", true), vec!["A b.", "C", "d"]);
        let f = Field { off: 0, bits: 8, maxc: Some(3), ty: String::new(), meaning: String::new(), valid: String::new() };
        assert_eq!(f.size_bits(), 24);
    }
}
