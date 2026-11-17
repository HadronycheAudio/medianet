//! The Markdown helpers the four WP5 scripts share, character for character
//! (`split_row`, `parse_tables`, `heading_index`, `strip_fences`, `blocks`,
//! `has_evidence`, `norm_name`, `parse_int_token`, `width_to_bits`,
//! `bit_positions`, `is_field_table`), and their every-block tag rule. Text
//! is the shadowed text of [`super::py`]; lines are `str.splitlines()`.
//!
//! The variants these helpers fix, against the library's:
//! * fences: a line whose `lstrip()` starts with three backticks toggles;
//!   tildes do not ([`FencePolicy::Backticks`] in the library);
//! * a table is a run of lines starting with `|` (after `lstrip`) whose
//!   second line contains `-` and matches `^\s*\|?[\s:|-]+\|?\s*$`; a run
//!   without such a separator is dropped whole (no headerless table);
//! * a row is split on `|` not preceded by a backslash, and `\|` restored;
//! * headings: `^(#{1,6})\s+(.*?)\s*#*\s*$` outside fences, any level.
//!
//! [`FencePolicy::Backticks`]: crate::doccheck::policy::FencePolicy::Backticks

use super::big::Big;
use super::py::{py_int, py_strip, si};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::rx;

/// A table: (index of its first line, header cells, data rows).
pub(crate) type MdTable = (usize, Vec<String>, Vec<Vec<String>>);

/// `split_row(line)`.
pub(crate) fn split_row(line: &str) -> Vec<String> {
    let mut s = py_strip(line);
    if let Some(r) = s.strip_prefix('|') {
        s = r;
    }
    if let Some(r) = s.strip_suffix('|') {
        s = r;
    }
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut prev: Option<char> = None;
    for c in s.chars() {
        if c == '|' && prev != Some('\\') {
            cells.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
        prev = Some(c);
    }
    cells.push(cur);
    cells.into_iter().map(|c| py_strip(&c).replace("\\|", "|")).collect()
}

fn lstrip(s: &str) -> &str {
    s.trim_start_matches(super::py::py_space)
}

/// `parse_tables(text)`.
pub(crate) fn parse_tables(text: &str) -> Vec<MdTable> {
    let lines = splitlines(text);
    let mut tables = Vec::new();
    let mut in_fence = false;
    let mut i = 0;
    while i < lines.len() {
        let stripped = lstrip(lines[i]);
        if stripped.starts_with("```") {
            in_fence = !in_fence;
            i += 1;
            continue;
        }
        if !in_fence && stripped.starts_with('|') {
            let start = i;
            let mut j = i;
            while j < lines.len() && lstrip(lines[j]).starts_with('|') {
                j += 1;
            }
            let block = &lines[i..j];
            if block.len() >= 2 && block[1].contains('-') && rx(r"^\s*\|?[\s:|-]+\|?\s*$").is_match(block[1]) {
                tables.push((start, split_row(block[0]), block[2..].iter().map(|r| split_row(r)).collect()));
            }
            i = j;
        } else {
            i += 1;
        }
    }
    tables
}

/// `heading_index(lines)`: (line index, level, title) outside fences.
pub(crate) fn heading_index(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for (idx, line) in lines.iter().enumerate() {
        if rx(r"^\s*```").is_match(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(m) = rx(r"^(#{1,6})\s+(.*?)\s*#*\s*$").captures(line) {
            out.push((idx, m[1].len(), py_strip(&m[2]).to_string()));
        }
    }
    out
}

/// The section of heading `k` of `heads`: lines after it up to the next
/// heading of the same or a higher level, joined with `\n`.
pub(crate) fn section_of(lines: &[&str], heads: &[(usize, usize, String)], k: usize) -> String {
    let (start, level) = (heads[k].0 + 1, heads[k].1);
    let end = heads[k + 1..].iter().find(|h| h.1 <= level).map_or(lines.len(), |h| h.0);
    lines[start..end.max(start)].join("\n")
}

/// `strip_fences(text)`.
pub(crate) fn strip_fences(text: &str) -> String {
    let mut in_fence = false;
    let mut out = Vec::new();
    for line in splitlines(text) {
        if rx(r"^\s*```").is_match(line) {
            in_fence = !in_fence;
            out.push("");
            continue;
        }
        out.push(if in_fence { "" } else { line });
    }
    out.join("\n")
}

/// `blocks(text)`: runs of non-blank lines.
pub(crate) fn blocks(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for line in splitlines(text) {
        if !py_strip(line).is_empty() {
            cur.push(line);
        } else if !cur.is_empty() {
            out.push(cur.join("\n"));
            cur.clear();
        }
    }
    if !cur.is_empty() {
        out.push(cur.join("\n"));
    }
    out
}

/// `EVIDENCE.search(block)` (the six tags, `re.I`).
pub(crate) fn has_evidence(block: &str) -> bool {
    si(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", block)
}

/// `norm_name(s)`: lower case, then only `[a-z0-9]` kept.
pub(crate) fn norm_name(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit()).collect()
}

/// `int(m.replace(" ", "").replace("_", ""))` of a `(\d[\d _]*)` match.
fn int_of(m: &str) -> Big {
    py_int(&m.replace([' ', '_'], "")).expect("digits")
}

/// `parse_int_token(text)`: the first integer, spaces and `_` removed.
pub(crate) fn parse_int_token(text: &str) -> Option<Big> {
    rx(r"(\d[\d _]*)").captures(text).map(|c| int_of(&c[1]))
}

/// `width_to_bits(cell)`: `N bits` reads N, else the first integer × 8.
pub(crate) fn width_to_bits(cell: &str) -> Option<Big> {
    let s = cell.to_lowercase();
    if let Some(c) = rx(r"(\d[\d _]*)\s*bits?").captures(&s) {
        return Some(int_of(&c[1]));
    }
    rx(r"(\d[\d _]*)").captures(&s).map(|c| int_of(&c[1]).mul(&Big::from(8)))
}

/// `bit_positions(row_text)`: (lo, hi) of `bits hi..lo` or `bit N`.
pub(crate) fn bit_positions(row_text: &str) -> Option<(i64, i64)> {
    let small = |s: &Option<String>| py_int(s.as_deref().unwrap_or("")).and_then(|b| b.to_i64());
    if let Some((_, _, g)) = super::py::find_i(r"bits?\s*(\d+)\s*\.\.\s*(\d+)", row_text) {
        let (hi, lo) = (small(&g[0]), small(&g[1]));
        if let (Some(hi), Some(lo)) = (hi, lo)
            && 0 <= lo
            && lo <= hi
            && hi <= 7
        {
            return Some((lo, hi));
        }
    }
    if let Some((_, _, g)) = super::py::find_i(r"\bbit\s*(\d+)\b", row_text)
        && let Some(b) = small(&g[0])
        && (0..=7).contains(&b)
    {
        return Some((b, b));
    }
    None
}

/// `is_field_table(header)`: Offset, Width, Type, Meaning, Valid range.
pub(crate) fn is_field_table(header: &[String]) -> bool {
    let h: Vec<String> = header.iter().map(|x| norm_name(x)).collect();
    h.len() >= 5
        && h[0].starts_with("offset")
        && h[1].contains("width")
        && h[2].contains("type")
        && h[3].contains("meaning")
        && h[4].contains("valid")
}

/// `PLACEHOLDER.search(section) and "|" not in section`, or an empty
/// section.
pub(crate) fn is_placeholder(section: &str) -> bool {
    py_strip(section).is_empty() || (si(r"to\s+be\s+filled", section) && !section.contains('|'))
}

/// The every-block tag rule of the four scripts: each block of the section
/// (`blocks`, fenced code included) loses its `#` lines; a block with a tag
/// passes; a table or code block (all lines start with `|`, or the first
/// starts with three backticks) passes when the previous block passed;
/// a block of headings only resets to passed. Returns the first line of
/// each failing block.
pub(crate) fn untagged_blocks(section: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut prev_tagged = true;
    for block in blocks(section) {
        let stripped = py_strip(&block);
        if stripped.is_empty() {
            continue;
        }
        let content: Vec<&str> = splitlines(stripped).into_iter().filter(|l| !lstrip(l).starts_with('#')).collect();
        if content.is_empty() {
            prev_tagged = true;
            continue;
        }
        let body = py_strip(&content.join("\n")).to_string();
        let is_table = content.iter().filter(|l| !py_strip(l).is_empty()).all(|l| py_strip(l).starts_with('|'));
        let is_code = lstrip(content[0]).starts_with("```");
        if has_evidence(&body) {
            prev_tagged = true;
            continue;
        }
        if (is_table || is_code) && prev_tagged {
            continue;
        }
        out.push(splitlines(&body).first().map(|l| l.to_string()).unwrap_or_default());
        prev_tagged = false;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_as_the_scripts_wrote_them() {
        assert_eq!(split_row(r"| a \| b | c |"), vec!["a | b", "c"]);
        assert_eq!(split_row("|"), vec![""]);
        let t = parse_tables("x\n| A | B |\n|---|---|\n| 1 | 2 |\n\n| C |\n| D |\n");
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].2, vec![vec!["1".to_string(), "2".to_string()]]);
        let lines = splitlines("# T #\n```\n# no\n```\n## U\u{2028}### V");
        let h = heading_index(&lines);
        assert_eq!(h.iter().map(|x| x.2.as_str()).collect::<Vec<_>>(), ["T", "U", "V"]);
        assert_eq!(width_to_bits("2 bytes"), Some(Big::from(16)));
        assert_eq!(width_to_bits("2 bits"), Some(Big::from(2)));
        assert_eq!(width_to_bits("1 2"), Some(Big::from(96)));
        assert_eq!(bit_positions("bits 7..6 x"), Some((6, 7)));
        assert_eq!(bit_positions("bit 9"), None);
        assert_eq!(untagged_blocks("A `[code]`.\n\n| t |\n\nB.\n\n| u |\n"), vec!["B.", "| u |"]);
    }
}
