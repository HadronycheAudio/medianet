//! The Markdown helpers of the WP7 scripts (004-16 to 004-20), transcribed
//! with Python semantics ([`super::py`]).
//!
//! 004-16, 004-17 and 004-18 share one family of helpers (`split_row`,
//! `parse_tables`, `heading_index`, `strip_fences`, `prose_blocks`,
//! `clean_start`, `is_field_table`, `width_of`, `fields_of`, `subsections`,
//! `find_sub`, `first_table`, `section_span`); 004-19 and 004-20 use the
//! other textual family of `parse_tables`, `heading_index` and
//! `strip_fences`, which computes the same thing (the conditions of
//! `parse_tables` are the same, in another order). Tables are the 004
//! family's: a run of lines starting with `|` whose second line is a
//! separator by the line regex `^\s*\|?[\s:|-]+\|?\s*$` with a `-` in it;
//! a run without one is dropped whole (not the 004-22 rule of the library's
//! `tables`).

use super::py::{self, is_space, lstrip, strip};
use crate::doccheck::Report;

/// A table: its first line's index, header cells and data rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PyTable {
    pub start: usize,
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// `split_row`.
pub(crate) fn split_row(line: &str) -> Vec<String> {
    let mut s = strip(line);
    s = s.strip_prefix('|').unwrap_or(s);
    s = s.strip_suffix('|').unwrap_or(s);
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut prev = None;
    for c in s.chars() {
        if c == '|' && prev != Some('\\') {
            cells.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
        prev = Some(c);
    }
    cells.push(cur);
    cells
        .into_iter()
        .map(|c| strip(&c).replace("\\|", "|"))
        .collect()
}

/// The 004 separator line test.
pub(crate) fn is_separator(line: &str) -> bool {
    py::matches(r"^\s*\|?[\s:|-]+\|?\s*$", line, false).is_some() && line.contains('-')
}

/// `parse_tables`.
pub(crate) fn parse_tables(text: &str) -> Vec<PyTable> {
    let lines = py::splitlines(text);
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut i = 0;
    while i < lines.len() {
        let stripped = lstrip(lines[i]);
        if stripped.starts_with("```") {
            in_fence = !in_fence;
            i += 1;
            continue;
        }
        if in_fence || !stripped.starts_with('|') {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < lines.len() && lstrip(lines[j]).starts_with('|') {
            j += 1;
        }
        let block = &lines[i..j];
        if block.len() >= 2 && is_separator(block[1]) {
            out.push(PyTable {
                start: i,
                header: split_row(block[0]),
                rows: block[2..].iter().map(|l| split_row(l)).collect(),
            });
        }
        i = j;
    }
    out
}

/// A heading: line index, level, title.
pub(crate) type Head = (usize, usize, String);

fn is_fence(line: &str) -> bool {
    lstrip(line).starts_with("```")
}

/// `heading_index`.
pub(crate) fn heading_index<S: AsRef<str>>(lines: &[S]) -> Vec<Head> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for (idx, line) in lines.iter().enumerate() {
        let line = line.as_ref();
        if is_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let level = line.chars().take_while(|&c| c == '#').count();
        if level == 0 || level > 6 {
            continue;
        }
        let rest = &line[level..];
        let body = rest.trim_start_matches(is_space);
        if body.len() == rest.len() {
            continue;
        }
        let t = body.trim_end_matches(is_space).trim_end_matches('#');
        out.push((idx, level, strip(t).to_string()));
    }
    out
}

/// `strip_fences`: fenced lines (and the fences) blanked.
pub(crate) fn strip_fences(text: &str) -> String {
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in py::splitlines(text) {
        if is_fence(line) {
            in_fence = !in_fence;
            out.push("");
            continue;
        }
        out.push(if in_fence { "" } else { line });
    }
    out.join("\n")
}

/// `prose_blocks`: (kind, text), kind `'p'` or `'t'`.
pub(crate) fn prose_blocks(text: &str) -> Vec<(char, String)> {
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    let mut kind: Option<char> = None;
    let plain = strip_fences(text);
    for line in py::splitlines(&plain) {
        if py::matches(r"^#{1,6}\s", line, false).is_some() || strip(line).is_empty() {
            if !cur.is_empty() {
                out.push((kind.unwrap_or('p'), cur.join("\n")));
            }
            cur.clear();
            kind = None;
            continue;
        }
        let k = if lstrip(line).starts_with('|') {
            't'
        } else {
            'p'
        };
        if kind.is_some_and(|x| x != k) {
            out.push((kind.unwrap_or('p'), cur.join("\n")));
            cur.clear();
        }
        kind = Some(k);
        cur.push(line);
    }
    if !cur.is_empty() {
        out.push((kind.unwrap_or('p'), cur.join("\n")));
    }
    out
}

/// `clean_start`: `re.sub(r"^[\s*_`>-]+", "", text)`.
pub(crate) fn clean_start(text: &str) -> &str {
    text.trim_start_matches(|c: char| is_space(c) || "*_`>-".contains(c))
}

const KEY_RULES: [(&str, &str); 7] = [
    ("magic", r"^magic"),
    ("version", r"^(protocol\s+)?version"),
    ("domain", r"^domain"),
    ("type", r"^message\s+type"),
    ("scheme", r"^(authentication\s+)?scheme"),
    ("counter", r"^(message\s+)?counter"),
    ("tag", r"^(authentication\s+)?tag"),
];

/// `key_of(meaning)`.
pub(crate) fn key_of(meaning: &str) -> Option<&'static str> {
    let m = clean_start(meaning).to_lowercase();
    KEY_RULES
        .iter()
        .find(|(_, r)| py::has(r, &m))
        .map(|(k, _)| *k)
}

/// `EVIDENCE.search(text)` (the six tags, any case).
pub(crate) fn has_evidence(text: &str) -> bool {
    py::has_i(
        r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]",
        text,
    )
}

fn norm_head(x: &str) -> String {
    x.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .collect()
}

/// `is_field_table`: Offset, Width, Type, Meaning, Valid range.
pub(crate) fn is_field_table(header: &[String]) -> bool {
    let h: Vec<String> = header.iter().map(|x| norm_head(x)).collect();
    h.len() >= 5
        && h[0].starts_with("offset")
        && h[1].contains("width")
        && h[2].contains("type")
        && h[3].contains("meaning")
        && h[4].contains("valid")
}

/// `width_of`: (bits, max count); `(None, None)` when the cell is neither.
pub(crate) fn width_of(cell: &str) -> (Option<i128>, Option<i128>) {
    let s = strip(cell).replace(['`', '*'], "").replace('X', "x");
    let bits = |n: &str, unit: Option<&str>| {
        let unit = unit.unwrap_or("bytes").to_lowercase();
        py::int(n).map(|n| {
            if unit.starts_with("bit") {
                n
            } else {
                n.saturating_mul(8)
            }
        })
    };
    if let Some(m) = py::fullmatch(r"(\d+)\s*(bytes?|bits?)?\s*(?:x|×)\s*max\s*(\d+)", &s, true) {
        return (bits(m.g(1), m.opt(2)), py::int(m.g(3)));
    }
    if let Some(m) = py::fullmatch(r"(\d+)\s*(bytes?|bits?)?", &s, true) {
        return (bits(m.g(1), m.opt(2)), None);
    }
    (None, None)
}

/// One row of a field table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    pub off: i128,
    pub bits: i128,
    pub maxc: Option<i128>,
    pub ty: String,
    pub meaning: String,
    pub valid: String,
}

impl Field {
    /// `size_bits`: bits times the max count (a zero count counts once).
    pub fn size_bits(&self) -> i128 {
        self.bits
            .saturating_mul(self.maxc.filter(|&m| m != 0).unwrap_or(1))
    }
}

/// `fields_of(table, ac, where)`.
pub(crate) fn fields_of(
    table: Option<&PyTable>,
    ac: &str,
    place: &str,
    report: &mut Report,
) -> Vec<Field> {
    let Some(t) = table else { return Vec::new() };
    if !is_field_table(&t.header) {
        report.fail(ac, format!("{place}: the table does not have the five columns Offset, Width, Type, Meaning, Valid range"));
        return Vec::new();
    }
    let mut out = Vec::new();
    for r in &t.rows {
        if r.len() < 5 {
            report.fail(
                ac,
                format!(
                    "{place}: a row has fewer than five cells: {:?}",
                    &r[..r.len().min(2)]
                ),
            );
            continue;
        }
        if py::fullmatch(r"\d+", strip(&r[0]), false).is_none() {
            report.fail(
                ac,
                format!("{place}: the Offset cell '{}' is not a byte offset", r[0]),
            );
            continue;
        }
        let (Some(bits), maxc) = width_of(&r[1]) else {
            report.fail(
                ac,
                format!(
                    "{place}: the Width cell '{}' is neither a byte count nor 'E × max M'",
                    r[1]
                ),
            );
            continue;
        };
        out.push(Field {
            off: py::int(&r[0]).unwrap_or_default(),
            bits,
            maxc,
            ty: strip(&r[2]).to_string(),
            meaning: strip(&r[3]).to_string(),
            valid: strip(&r[4]).to_string(),
        });
    }
    out
}

/// `subsections(sec_lines)`: the `###` subsections as (title, body), each
/// ending at the next heading of level 3 or higher.
pub(crate) fn subsections<S: AsRef<str>>(lines: &[S]) -> Vec<(String, String)> {
    let hs = heading_index(lines);
    let mut out = Vec::new();
    for (k, (idx, lvl, title)) in hs.iter().enumerate() {
        if *lvl != 3 {
            continue;
        }
        let end = hs[k + 1..]
            .iter()
            .find(|h| h.1 <= 3)
            .map_or(lines.len(), |h| h.0);
        let body: Vec<&str> = lines[(idx + 1).min(end)..end]
            .iter()
            .map(AsRef::as_ref)
            .collect();
        out.push((title.clone(), body.join("\n")));
    }
    out
}

/// `find_sub(subs, pattern)`: titles matching `pattern` (any case).
pub(crate) fn find_sub<'a>(
    subs: &'a [(String, String)],
    pattern: &str,
) -> Vec<&'a (String, String)> {
    subs.iter().filter(|(t, _)| py::has_i(pattern, t)).collect()
}

/// `first_table(text, pred)`.
pub(crate) fn first_table(text: &str, pred: impl Fn(&[String]) -> bool) -> Option<PyTable> {
    parse_tables(text).into_iter().find(|t| pred(&t.header))
}

/// `section_span(pattern, level)`: the first heading matching `pattern`
/// (`re.search`, any case) at `level` (any level when `None`), up to the
/// next heading of the same or a higher level, as a line range without the
/// heading line.
pub(crate) fn section_span(
    lines_len: usize,
    heads: &[Head],
    pattern: &str,
    level: Option<usize>,
) -> Option<(usize, usize)> {
    for (i, (idx, lvl, title)) in heads.iter().enumerate() {
        if py::has_i(pattern, title) && level.is_none_or(|l| l == *lvl) {
            let end = heads[i + 1..]
                .iter()
                .find(|h| h.1 <= *lvl)
                .map_or(lines_len, |h| h.0);
            return Some((idx + 1, end));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_need_the_004_separator_line() {
        let t = "| a | b |\n|-:-|---|\n| 1 | 2 |\n\n| c |\n| 3 |\n";
        let ts = parse_tables(t);
        assert_eq!(ts.len(), 1);
        assert_eq!(ts[0].rows, vec![vec!["1".to_string(), "2".to_string()]]);
        assert_eq!(split_row(r"| a \| b | c |"), vec!["a | b", "c"]);
    }

    #[test]
    fn headings_blocks_and_widths() {
        let lines = ["# A #", "```", "## no", "```", "###  B  ##", "#x"];
        let hs = heading_index(&lines);
        assert_eq!(hs, vec![(0, 1, "A".to_string()), (4, 3, "B".to_string())]);
        let b = prose_blocks("para one\ntwo\n| t |\n\n## H\nnext");
        assert_eq!(
            b,
            vec![
                ('p', "para one\ntwo".to_string()),
                ('t', "| t |".to_string()),
                ('p', "next".to_string())
            ]
        );
        assert_eq!(width_of("2"), (Some(16), None));
        assert_eq!(width_of("`4` X max 3"), (Some(32), Some(3)));
        assert_eq!(width_of("16 bits"), (Some(16), None));
        assert_eq!(width_of("two"), (None, None));
        assert_eq!(clean_start("** > Open"), "Open");
    }
}
