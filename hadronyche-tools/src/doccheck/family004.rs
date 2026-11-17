//! The 004 family's shared Markdown helpers (lead ruling R18).
//!
//! The scripts 004-16 to 004-18 and 004-24 to 004-29 share these functions
//! (`fields_of`, `width_of`, `is_field_table`, `subsections`, `find_sub`,
//! `first_table`, `span_of`, `size_bits`; inventory section 2). They lived in
//! the 004-27 port; two packages imported them from there, and a port may not
//! import another port (R16), so they are library code now, unchanged in
//! behaviour (the 004-27 equivalence proofs did not move). Each reproduces
//! the scripts' Python: digit captures through `parse_int`, tables by
//! [`TableRules::FAMILY_004`], Python's `strip`.

use super::markdown::{Markdown, Span};
use super::policy::{HeadingMatch, TableRules};
use super::pycompat::{IntError, PyStr, parse_int};
use super::report::Report;
use super::tables::{Row, Table};
use super::text::{full, rx};
use super::ticket::Constants;

/// One row of a five-column field table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
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
        self.bits
            .saturating_mul(self.maxc.filter(|&m| m != 0).unwrap_or(1))
    }
}

/// `is_field_table`: Offset, Width, Type, Meaning, Valid range.
pub fn is_field_table(header: &[String]) -> bool {
    let h: Vec<String> = header
        .iter()
        .map(|x| rx(r"[^a-z0-9]").replace_all(&x.to_lowercase(), ""))
        .collect();
    h.len() >= 5
        && h[0].starts_with("offset")
        && h[1].contains("width")
        && h[2].contains("type")
        && h[3].contains("meaning")
        && h[4].contains("valid")
}

/// A width: (bits, max count).
pub type Width = (i128, Option<i128>);

/// `width_of`: the (bits, max count) of a Width cell, `Ok(None)` when it is
/// neither form, `Err` when a number is beyond what the check can compare.
pub fn width_of(cell: &str) -> Result<Option<Width>, IntError> {
    let s = cell.py_strip().replace(['`', '*'], "").replace('X', "x");
    let bits = |n: &str, unit: Option<crate::doccheck::rx::M>| -> Result<i128, IntError> {
        let n = parse_int(n)?;
        let unit = unit.map_or("bytes".to_string(), |u| u.as_str().to_lowercase());
        Ok(if unit.starts_with("bit") {
            n
        } else {
            n.saturating_mul(8)
        })
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
        return Err(format!(
            "{place}: a row has fewer than five cells: {:?}",
            &r.cells[..r.cells.len().min(2)]
        ));
    }
    let off = r.cell(0).py_strip();
    if !full(r"\d+", off) {
        return Err(format!(
            "{place}: the Offset cell '{}' is not a byte offset",
            r.cell(0)
        ));
    }
    let off = parse_int(off).map_err(|e| format!("{place}: {e}"))?;
    let Some((bits, maxc)) = width_of(r.cell(1)).map_err(|e| format!("{place}: {e}"))? else {
        return Err(format!(
            "{place}: the Width cell '{}' is neither a byte count nor 'E × max M'",
            r.cell(1)
        ));
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
pub fn fields_of(table: Option<&Table>, ac: &str, place: &str, report: &mut Report) -> Vec<Field> {
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
pub fn tables(text: &str) -> Vec<Table> {
    let md = Markdown::parse(text);
    md.tables_with(md.all(), TableRules::FAMILY_004)
}

/// `first_table(text, is_field_table)`.
pub fn first_field_table(text: &str) -> Option<Table> {
    tables(text).into_iter().find(|t| is_field_table(t.head()))
}

/// `subsections`: the `###` subsections of `text` as (title, body without
/// the heading line), each ending at the next heading of level 3 or higher.
pub fn subsections(text: &str) -> Vec<(String, String)> {
    let md = Markdown::parse(text);
    let mut out = Vec::new();
    for (k, h) in md.heads.iter().enumerate() {
        if h.level != 3 {
            continue;
        }
        let end = md.heads[k + 1..]
            .iter()
            .find(|n| n.level <= 3)
            .map_or(md.lines.len(), |n| n.line);
        out.push((h.text.clone(), md.lines[h.line + 1..end].join("\n")));
    }
    out
}

/// `find_sub`: the subsections whose title matches `pattern` (any case).
pub fn find_sub<'a>(subs: &'a [(String, String)], pattern: &str) -> Vec<&'a (String, String)> {
    let re = rx(&format!("(?i){pattern}"));
    subs.iter().filter(|(t, _)| re.is_match(t)).collect()
}

/// Sentences: `re.split(r"(?<=[.;])\s+", text)` (plus `\n` when `newline`).
pub fn sentences(text: &str, newline: bool) -> Vec<String> {
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

/// A document and the constants table, as the family reads them.
pub struct Ctx {
    pub md: Markdown,
    pub consts: Constants,
}

impl Ctx {
    /// `span_of(pattern, level)`: the section of the first heading at
    /// `level` matching `pattern` (any case).
    pub fn span_of(&self, pattern: &str, level: usize) -> Option<Span> {
        let h = self
            .md
            .find_by(HeadingMatch::RegexAtLevel(level), &format!("(?i){pattern}"))
            .into_iter()
            .next()?;
        Some(self.md.section(h))
    }

    /// The text of a span, or `""`.
    pub fn body(&self, span: Option<Span>) -> String {
        span.map(|s| self.md.text(s)).unwrap_or_default()
    }
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
        assert_eq!(
            sentences("A b. C; d\ne.", false),
            vec!["A b.", "C;", "d\ne."]
        );
        assert_eq!(sentences("A b. C\nd", true), vec!["A b.", "C", "d"]);
        let f = Field {
            off: 0,
            bits: 8,
            maxc: Some(3),
            ty: String::new(),
            meaning: String::new(),
            valid: String::new(),
        };
        assert_eq!(f.size_bits(), 24);
    }
}
