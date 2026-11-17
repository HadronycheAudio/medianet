//! Port of `checks/004-26.sh`: the manifest encoding in the admission
//! handshake (ticket 004-26).
//!
//! The acceptance-criterion map, with the ticket sentences, is on the tests
//! in `tests/doc_004_26.rs`. 004-26 is a rewrite of the shared helpers with
//! its own conventions, each reproduced here and named where used:
//! wire.md and the constants table read as written (no digit-group join:
//! numbers are read with [`NUM`], which takes groups of three after a
//! space, a no-break space or a narrow no-break space, and `0x` hex, and
//! [`to_int`], which also drops commas); sections by
//! [`section_lines`] (`re.search` at an exact level, the heading line
//! excluded); field-table rows by [`model_rows`] (malformed rows collected,
//! reported only for the manifest's own tables) and their tiling by
//! [`tile`] (bit offsets from the first row, whole-byte rows byte-aligned);
//! a counted row is one whose width has a `max` (`maxc is not None`, a
//! count of 0 included); tables of the 004-16 to 004-29 family
//! ([`crate::doccheck::checks::c004_24::py::tables`]); the validity table
//! is any table with "form" and "outcome" in its first two header cells
//! that is not a field table; reasons are matched as exact `\bNAME\b`
//! against the names read from the Rejection reasons (unanchored
//! `rejection[-\s]+reasons?`) and the refusal reasons (level 3), 13 and 18;
//! the untagged-claim rule per prose block with 5 `\S+` words
//! ([`TagPolicy::PROSE_004_26`]); an open item is a prose block whose first
//! 40 characters after `*_`` are removed contain "open item", and it needs
//! `\bowner\b` and `\bfalsifier\b`; every subsection that holds a table
//! needs a tag. wire.toml is read with the scripts' subset reader
//! ([`crate::doccheck::checks::c004_24::pytoml`]) and its values compared
//! as Python compares them.
//!
//! Its repository-level assertions (`cargo run --bin wire-check` reports
//! "checks clean"; `cargo test -p hadronyche-tools --test wire_tables` has 0
//! failed and at least 12 passed; the checker names no ref containing main
//! and runs no other check) are in `scripts/repo-state.d/004-26.sh`.
//!
//! [`TagPolicy::PROSE_004_26`]: crate::doccheck::evidence::TagPolicy::PROSE_004_26

mod encoding;
mod fields;
mod forms;
mod landed;
mod rest;
mod size;
mod toml_side;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::checks::c004_24::Stop;
use crate::doccheck::checks::c004_24::py::{py_chars, py_text, stopped, tables};
use crate::doccheck::checks::c004_27::width_of;
use crate::doccheck::checks::c004_27::subsections;
use crate::doccheck::tables::{Table, split_row};
use crate::doccheck::text::{full, rx, splitlines};
use crate::doccheck::ticket::CONSTANTS_MD;
use crate::doccheck::wire_toml::WIRE_TOML;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-26",
    run,
    script_env: &[("WIRE_MD", WIRE_MD)],
    targets: &[
        Target {
            file: WIRE_MD,
            heading: Some("^Manifest$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Admission messages$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Common control header$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Rejection reasons$"),
        },
        Target {
            file: WIRE_MD,
            heading: Some("^Reserved fields$"),
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
    keywords: &["falsified", "exceeds", "distinct", "reserved", "device"],
    known_differences: KNOWN,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// 004-26's number: `0x` hex, digit groups after a space or a (narrow)
/// no-break space, or digits.
pub(crate) const NUM: &str = "(?:0x[0-9a-fA-F]+|\\d{1,3}(?:[ \u{a0}\u{202f}]\\d{3})+|\\d+)";
/// `HEXNUM`.
pub(crate) const HEXNUM: &str = "0x[0-9a-fA-F]+";

/// 004-26's `to_int`: hex after `0x`, else the digits with spaces, no-break
/// spaces and commas dropped. `None` where Python's `int()` raises (or the
/// value does not fit 128 bits).
pub(crate) fn to_int(s: &str) -> Option<i128> {
    let s = s.trim();
    if s.to_lowercase().starts_with("0x") {
        return i128::from_str_radix(&s[2..].replace('_', ""), 16).ok();
    }
    let t: String = s.chars().filter(|c| !matches!(c, ' ' | '\u{a0}' | '\u{202f}' | ',')).collect();
    t.replace('_', "").parse().ok()
}

/// One row of a field table (`model_rows`).
#[derive(Debug, Clone)]
pub(crate) struct MRow {
    pub off: i64,
    pub bits: i64,
    /// `maxc`: `Some` for a counted row (a count of 0 included).
    pub maxc: Option<i64>,
    pub ty: String,
    pub meaning: String,
    pub valid: String,
    pub wcell: String,
}

impl MRow {
    /// `txt(r)`: meaning and valid range, lowered.
    pub fn txt(&self) -> String {
        format!("{} {}", self.meaning, self.valid).to_lowercase()
    }
}

/// `model_rows`: the rows and the problems of the malformed ones.
pub(crate) fn model_rows(rows: &[Vec<String>]) -> (Vec<MRow>, Vec<String>) {
    let (mut out, mut bad) = (Vec::new(), Vec::new());
    for r in rows {
        if r.len() < 5 {
            bad.push(format!("a row has fewer than five cells: {:?}", &r[..r.len().min(2)]));
            continue;
        }
        let c0 = r[0].trim();
        let Some(off) = full(r"\d+", c0).then(|| c0.parse::<i64>().ok()).flatten() else {
            bad.push(format!("the Offset cell '{}' is not a byte offset", r[0]));
            continue;
        };
        let Some((bits, maxc)) = width_of(&r[1]) else {
            bad.push(format!("the Width cell '{}' is neither a byte count, a bit count nor 'W × max M'", r[1]));
            continue;
        };
        out.push(MRow {
            off,
            bits,
            maxc,
            ty: r[2].trim().to_string(),
            meaning: r[3].trim().to_string(),
            valid: r[4].trim().to_string(),
            wcell: r[1].trim().to_string(),
        });
    }
    (out, bad)
}

/// The rows of a table as Python's lists.
pub(crate) fn cells(t: &Table) -> Vec<Vec<String>> {
    t.rows.iter().map(|r| r.cells.clone()).collect()
}

/// `tile`: the bit offset of each row, the total bits, the tiling errors.
pub(crate) fn tile(rows: &[MRow]) -> (Vec<i64>, i64, Vec<String>) {
    let (mut errors, mut offsets) = (Vec::new(), Vec::new());
    let Some(first) = rows.first() else { return (offsets, 0, errors) };
    let mut cursor = first.off * 8;
    let start = cursor;
    for r in rows {
        if r.off != cursor.div_euclid(8) {
            errors.push(format!("offset {} does not follow the previous row, which ends at byte {}", r.off, cursor.div_euclid(8)));
        }
        if r.bits % 8 == 0 && cursor.rem_euclid(8) != 0 {
            errors.push(format!("a whole-byte row at offset {} is not byte-aligned", r.off));
        }
        offsets.push(cursor);
        cursor += r.bits * r.maxc.filter(|&m| m != 0).unwrap_or(1);
    }
    if (cursor - start).rem_euclid(8) != 0 {
        errors.push("the table does not cover a whole number of bytes".into());
    }
    (offsets, cursor - start, errors)
}

/// `is_field_table`.
pub(crate) fn is_field_table(h: &[String]) -> bool {
    crate::doccheck::checks::c004_27::is_field_table(h)
}

/// `is_validity_table`.
pub(crate) fn is_validity_table(h: &[String]) -> bool {
    let h: Vec<String> = h.iter().map(|x| x.to_lowercase()).collect();
    h.len() >= 2 && h[..2].iter().any(|x| x.contains("form")) && h[..2].iter().any(|x| x.contains("outcome"))
}

/// The wire text as 004-26 holds it.
pub(crate) struct Wire {
    /// The raw text, word and space classes as Python's.
    pub raw: String,
    /// `wire.splitlines()`.
    pub lines: Vec<String>,
}

impl Wire {
    /// `section_lines(wire, pattern, level)`: the lines after the first
    /// heading at `level` matching `pattern` (any case), to the next heading
    /// of that level or higher.
    pub fn section_lines(&self, pattern: &str, level: usize) -> Option<Vec<String>> {
        section_lines(&self.lines, pattern, level)
    }
}

/// `heading_index` over `lines`.
pub(crate) fn heading_index(lines: &[String]) -> Vec<(usize, usize, String)> {
    let (mut out, mut fence) = (Vec::new(), false);
    let re = rx(r"^(#{1,6})\s+(.*?)\s*#*\s*$");
    for (i, l) in lines.iter().enumerate() {
        if rx(r"^\s*```").is_match(l) {
            fence = !fence;
            continue;
        }
        if fence {
            continue;
        }
        if let Some(c) = re.captures(l) {
            out.push((i, c[1].len(), c[2].trim().to_string()));
        }
    }
    out
}

/// `section_lines` over `lines`.
pub(crate) fn section_lines(lines: &[String], pattern: &str, level: usize) -> Option<Vec<String>> {
    let hs = heading_index(lines);
    let re = rx(&format!("(?i){pattern}"));
    let (i, (idx, _, _)) = hs.iter().enumerate().find(|(_, (_, l, t))| *l == level && re.is_match(t))?;
    let end = hs[i + 1..].iter().find(|(_, l, _)| *l <= level).map_or(lines.len(), |(j, _, _)| *j);
    Some(lines[idx + 1..end].to_vec())
}

/// `subsections(lines)`.
pub(crate) fn subs(lines: &[String]) -> Vec<(String, String)> {
    subsections(&lines.join("\n"))
}

/// The rows of every line of `text` that starts with `|`, after the first
/// two (how the script reads the Hello's common control header).
pub(crate) fn pipe_rows(text: &str) -> Vec<Vec<String>> {
    splitlines(text).into_iter().filter(|l| l.trim_start().starts_with('|')).skip(2).map(split_row).collect()
}

/// The field tables of `text`.
pub(crate) fn field_tables(text: &str) -> Vec<Table> {
    tables(text).into_iter().filter(|t| is_field_table(t.head())).collect()
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let consts = py_text(&docs.read(CONSTANTS_MD).map(|t| t.to_string()).unwrap_or_default());
    let raw = match docs.read(WIRE_MD) {
        Some(t) => py_chars(&t, false),
        None => {
            report.fail("AC3", format!("{WIRE_MD} does not exist"));
            String::new()
        }
    };
    let wire = Wire {
        lines: splitlines(&py_chars(&raw, true)).into_iter().map(String::from).collect(),
        raw,
    };
    let landed = landed::landed(&consts, &wire, &mut report);
    let man = fields::manifest(&wire, &mut report);
    fields::ac1(&man, &mut report);
    encoding::ac2(&man, &mut report);
    let top = size::ac3(&man, &landed, &mut report);
    let join_open = forms::ac4(&man, &mut report);
    forms::ac5(&man, &landed, &mut report);
    size::t1(&man, &landed, top.as_ref(), &join_open, &mut report);
    rest::tags(&man, &mut report);
    let after = |report: &mut Report| -> Result<(), Stop> {
        let mut toml = toml_side::parse(docs.read(WIRE_TOML).as_deref(), report)?;
        toml_side::models(&man, &landed, &toml, report)?;
        rest::point2(&man, report);
        toml_side::fixed(&man, top.as_ref(), &mut toml, report)?;
        fields::point4(&man, &toml, report)?;
        fields::t3a(&man, &wire, report);
        encoding::t3b(&man, &toml, report)?;
        rest::t3c(&man, report);
        rest::reserved(&man, &wire, report);
        fields::t3f(&man, &wire, report);
        rest::t3g(&man, report);
        landed::unchanged(&wire, &landed, report);
        Ok(())
    };
    if let Err(Stop(e)) = after(&mut report) {
        stopped(&mut report, "AC6", &e);
    }
    report
}
