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
//!
//! M1-lib adds the other helpers every 004 script carries a copy of, each
//! variant named: [`heading_index`], [`section_span`] ([`SpanRule`]),
//! [`norm_name`] ([`NameNorm`]), [`width_to_bits`] ([`WidthRule`]),
//! [`parse_int_token`], [`bit_positions`], [`to_int`] ([`IntRule`]), the
//! constants-table readers [`const_cell`], [`const_line`], [`const_int`]
//! ([`ConstInt`]), [`md_name`], [`key_of`], [`tname`]. Each is
//! proven against the scripts' own definitions, run by Python 3.10 and 3.13
//! on `data/family004_corpus.txt` (`tools/gen_family004_corpus.py`; the test
//! is `doccheck::proofs004`), which also records which script's copy is
//! which variant.

use super::markdown::{Markdown, Span};
use super::policy::{HeadingMatch, TableRules};
use super::pycompat::{IntError, PyStr, is_space, parse_int, splitlines};
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

/// A heading of [`heading_index`]: (0-based line, level, title).
pub type Head = (usize, usize, String);

/// `heading_index(lines)` of every 004 script (one variant, 004-03 to
/// 004-29): headings `^(#{1,6})\s+(.*?)\s*#*\s*$` outside fences, a fence
/// being a line matching `^\s*```` (tildes are not fences), the title
/// stripped. Proven on `data/family004_corpus.txt`.
pub fn heading_index<S: AsRef<str>>(lines: &[S]) -> Vec<Head> {
    let fence = rx(r"^\s*```");
    let head = rx(r"^(#{1,6})\s+(.*?)\s*#*\s*$");
    let mut out = Vec::new();
    let mut in_fence = false;
    for (idx, line) in lines.iter().enumerate() {
        let line = line.as_ref();
        if fence.is_match(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(m) = head.captures(line) {
            out.push((idx, m[1].chars().count(), m[2].py_strip().to_string()));
        }
    }
    out
}

/// Which section span a script computes from [`heading_index`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanRule {
    /// `section_span(pattern, level=None)` of 004-16 to 004-18 (and
    /// `section_span(pattern)` of 004-19/004-20, the same with no level):
    /// the lines after the first heading whose title matches `pattern`
    /// (`re.search`, any case) at `level`, up to the next heading of the
    /// same or a higher level.
    AfterHeading004_16,
    /// `span_of(pattern, level)` of 004-24, 004-25, 004-28, 004-29: the
    /// level is required and tested first; the span STARTS AT the heading
    /// line itself.
    WithHeading004_24,
}

/// The (start, end) line range of the section named by `pattern` under
/// `rule`, or `None` (see [`SpanRule`]).
pub fn section_span(
    n_lines: usize,
    heads: &[Head],
    pattern: &str,
    level: Option<usize>,
    rule: SpanRule,
) -> Option<(usize, usize)> {
    let re = rx(&format!("(?i){pattern}"));
    for (i, (idx, lvl, title)) in heads.iter().enumerate() {
        if level.is_none_or(|l| l == *lvl) && re.is_match(title) {
            let end = heads[i + 1..]
                .iter()
                .find(|h| h.1 <= *lvl)
                .map_or(n_lines, |h| h.0);
            return Some(match rule {
                SpanRule::AfterHeading004_16 => (idx + 1, end),
                SpanRule::WithHeading004_24 => (*idx, end),
            });
        }
    }
    None
}

/// The name normalisers of the 004 scripts, each a named variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameNorm {
    /// `norm_name` of 004-06, 08-13, 19: `re.sub(r"[^a-z0-9]", "",
    /// s.lower())`.
    Alnum004_06,
    /// `norm_name` of 004-16 to 004-18: `-` to a space, lower case, runs
    /// of `\s` and `_` to one space, stripped.
    Spaced004_16,
    /// `norm` of 004-20: `re.sub(r"[^a-z]", "", s.lower())`.
    Letters004_20,
}

/// `s` normalised under `rule`.
pub fn norm_name(s: &str, rule: NameNorm) -> String {
    let lower = s.to_lowercase();
    match rule {
        NameNorm::Alnum004_06 => lower
            .chars()
            .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            .collect(),
        NameNorm::Spaced004_16 => {
            let mut out = String::with_capacity(lower.len());
            let mut gap = false;
            for c in lower.replace('-', " ").chars() {
                if is_space(c) || c == '_' {
                    gap = true;
                    continue;
                }
                if gap {
                    out.push(' ');
                    gap = false;
                }
                out.push(c);
            }
            if gap {
                out.push(' ');
            }
            out.py_strip().to_string()
        }
        NameNorm::Letters004_20 => lower.chars().filter(char::is_ascii_lowercase).collect(),
    }
}

/// The `width_to_bits(cell)` variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidthRule {
    /// 004-06, 08, 09, 10, 13, 19: `(\d[\d _]*)\s*bits?` reads bits (spaces
    /// and `_` dropped from the number); else the first `(\d[\d _]*)` × 8.
    DigitGroups004_06,
    /// 004-20: the first `\d+`; bits when `\bbits?\b` occurs and
    /// `\bbytes?\b` does not, else × 8.
    FirstNumber004_20,
}

/// `width_to_bits(cell)` under `rule`: `Ok(None)` when the cell has no
/// number, `Err` beyond `i128` (Python would have kept the number).
pub fn width_to_bits(cell: &str, rule: WidthRule) -> Result<Option<i128>, IntError> {
    let s = cell.to_lowercase();
    match rule {
        WidthRule::DigitGroups004_06 => {
            let int = |m: &str| parse_int(&m.replace([' ', '_'], ""));
            if let Some(c) = rx(r"(\d[\d _]*)\s*bits?").captures(&s) {
                return int(&c[1]).map(Some);
            }
            match rx(r"(\d[\d _]*)").captures(&s) {
                Some(c) => Ok(Some(int(&c[1])?.saturating_mul(8))),
                None => Ok(None),
            }
        }
        WidthRule::FirstNumber004_20 => {
            let Some(c) = rx(r"(\d+)").captures(&s) else {
                return Ok(None);
            };
            let n = parse_int(&c[1])?;
            let bits = rx(r"\bbits?\b").is_match(&s) && !rx(r"\bbytes?\b").is_match(&s);
            Ok(Some(if bits { n } else { n.saturating_mul(8) }))
        }
    }
}

/// `parse_int_token(text)` of 004-06, 08, 09: the first `(\d[\d _]*)`, spaces
/// and `_` removed, as an integer; `Err` beyond `i128`.
pub fn parse_int_token(text: &str) -> Result<Option<i128>, IntError> {
    match rx(r"(\d[\d _]*)").captures(text) {
        Some(c) => parse_int(&c[1].replace([' ', '_'], "")).map(Some),
        None => Ok(None),
    }
}

/// `bit_positions(row_text)` of 004-06 and 004-08: the (lo, hi) bits of a
/// sub-byte field, `bits hi..lo` (any case) with `0 <= lo <= hi <= 7`, else
/// a single `bit N` with `0 <= N <= 7`. A number too long for `i128` is
/// out of range, as it is for Python.
pub fn bit_positions(row_text: &str) -> Option<(i128, i128)> {
    let small = |d: &str| parse_int(d).ok().filter(|n| (0..=7).contains(n));
    if let Some(c) = rx(r"(?i)bits?\s*(\d+)\s*\.\.\s*(\d+)").captures(row_text) {
        let (hi, lo) = (parse_int(&c[1]).ok(), small(&c[2]));
        if let (Some(hi), Some(lo)) = (hi, lo)
            && lo <= hi
            && hi <= 7
        {
            return Some((lo, hi));
        }
    }
    let c = rx(r"(?i)\bbit\s*(\d+)\b").captures(row_text)?;
    small(&c[1]).map(|b| (b, b))
}

/// `NUM` of 004-16, 17, 18, 24, 25, 28, 29: an integer with digit groups
/// separated by a space or a no-break space.
pub const NUM_004_16: &str = "(?:\\d{1,3}(?:[ \u{a0}]\\d{3})+|\\d+)";
/// `NUM` of 004-26: also a `0x` hex literal; groups also by U+202F.
pub const NUM_004_26: &str = "(?:0x[0-9a-fA-F]+|\\d{1,3}(?:[ \u{a0}\u{202f}]\\d{3})+|\\d+)";

/// The `to_int(s)` variants of the 004 family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntRule {
    /// 004-16, 17, 18, 24, 25, 28, 29: `int(re.sub(r"[ \xa0]", "", s))`.
    Spaces004_16,
    /// 004-26: stripped; `0x…` read as hex (`int(s, 16)`); else
    /// `int(re.sub(r"[ \xa0\u202f,]", "", s))` (commas dropped too).
    HexCommas004_26,
}

/// `to_int(s)` under `rule`: `Err(NotInt)` where Python raises
/// `ValueError`, `Err(Overflow)` beyond `i128`.
pub fn to_int(s: &str, rule: IntRule) -> Result<i128, IntError> {
    match rule {
        IntRule::Spaces004_16 => parse_int(&s.replace([' ', '\u{a0}'], "")),
        IntRule::HexCommas004_26 => {
            let t = s.py_strip();
            if t.to_lowercase().starts_with("0x") {
                let digits = &t[2..];
                if !full("(?:_?[0-9a-fA-F])+", digits) {
                    return Err(IntError::NotInt(s.to_string()));
                }
                return i128::from_str_radix(&digits.replace('_', ""), 16)
                    .map_err(|_| IntError::Overflow(s.to_string()));
            }
            parse_int(&t.replace([' ', '\u{a0}', '\u{202f}', ','], ""))
        }
    }
}

/// The third cell (`Value`) of the constants row `| cid |`, stripped, or
/// `""`: `const_val(cid)` of 004-16, 17, 18, 24, 26 and `const_row(cid)` of
/// 004-25, 28, 29 (the same function).
pub fn const_cell(text: &str, cid: &str) -> String {
    let lead = format!("| {cid} |");
    splitlines(text)
        .into_iter()
        .find(|l| l.starts_with(&lead))
        .map(|l| l.split('|').nth(3).map_or("", |c| c.py_strip()).to_string())
        .unwrap_or_default()
}

/// The whole constants row `| cid |`, or `""`: `const_row(cid)` of 004-19
/// and 004-20.
pub fn const_line(text: &str, cid: &str) -> String {
    let lead = format!("| {cid} |");
    splitlines(text)
        .into_iter()
        .find(|l| l.starts_with(&lead))
        .unwrap_or_default()
        .to_string()
}

/// The `const_int(cid, pattern)` variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstInt {
    /// 004-16, 17, 18: `re.search(pattern, const_val(cid), re.I)`, group 1
    /// read by [`to_int`] ([`IntRule::Spaces004_16`]).
    AnyCase004_16,
    /// 004-28, 004-29: `re.search(pattern, const_row(cid))` (case-sensitive),
    /// group 1 read by plain `int()` (a digit group with a space raises).
    Exact004_28,
}

/// `const_int(cid, pattern)` under `rule`: `Ok(None)` when the pattern does
/// not match, `Err` where Python raises.
pub fn const_int(
    text: &str,
    cid: &str,
    pattern: &str,
    rule: ConstInt,
) -> Result<Option<i128>, IntError> {
    let cell = const_cell(text, cid);
    let re = match rule {
        ConstInt::AnyCase004_16 => rx(&format!("(?i){pattern}")),
        ConstInt::Exact004_28 => rx(pattern),
    };
    let Some(c) = re.captures(&cell) else {
        return Ok(None);
    };
    let group = c.get(1).map_or("", |m| m.as_str());
    match rule {
        ConstInt::AnyCase004_16 => to_int(group, IntRule::Spaces004_16).map(Some),
        ConstInt::Exact004_28 => parse_int(group).map(Some),
    }
}

/// `md_name(meaning)` of 004-16 to 004-18: the leading run of letters and
/// spaces of a Meaning cell (ending on a letter), or the cell itself.
pub fn md_name(meaning: &str) -> &str {
    rx(r"^\s*([A-Za-z][A-Za-z ]*[A-Za-z]|[A-Za-z])")
        .captures(meaning)
        .and_then(|c| c.get(1))
        .map_or(meaning, |m| m.as_str())
}

/// `KEY_RULES` of 004-16 to 004-18, in order.
pub const KEY_RULES_004_16: [(&str, &str); 7] = [
    ("magic", r"^magic"),
    ("version", r"^(protocol\s+)?version"),
    ("domain", r"^domain"),
    ("type", r"^message\s+type"),
    ("scheme", r"^(authentication\s+)?scheme"),
    ("counter", r"^(message\s+)?counter"),
    ("tag", r"^(authentication\s+)?tag"),
];

/// `key_of(meaning)` of 004-16 to 004-18: the first [`KEY_RULES_004_16`] key
/// whose pattern matches the lower-cased Meaning after `clean_start`.
pub fn key_of(meaning: &str) -> Option<&'static str> {
    let m = super::units::clean_start(meaning).to_lowercase();
    KEY_RULES_004_16
        .iter()
        .find(|(_, r)| rx(r).is_match(&m))
        .map(|(k, _)| *k)
}

/// `tname(text)` of 004-17, 004-18: [`NameNorm::Spaced004_16`] without a
/// trailing "message(s)".
pub fn tname(text: &str) -> String {
    rx(r"\s+messages?$").replace(&norm_name(text, NameNorm::Spaced004_16), "")
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
