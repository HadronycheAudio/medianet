//! Block segmentations of the 004 family other than [`super::units`]'s
//! prose blocks, each as the scripts wrote it, and the helpers that read
//! tables of invalid forms.
//!
//! | Function | Script function | Scripts |
//! |---|---|---|
//! | [`blank_blocks`] | `blocks(text)`: runs of lines that are not blank (`line.strip()`) | 004-03, 05, 06, 08-13, 19 |
//! | [`section_blocks`], [`block_role`] | `section_blocks(lines)`, `block_kind(block)`: each heading its own block, fenced code kept inside its block, blank lines separating the rest | 004-19, 004-20 |
//! | [`form_rows`] | `form_rows(text)` ([`FormRows::AnyCell004_17`]); `Sec.form_rows` ([`FormRows::FirstCell004_20`]) | 004-17, 004-18; 004-20 |
//! | [`is_placeholder`] | `is_placeholder(text)` (004-20); the inline `not section.strip() or (PLACEHOLDER.search(section) and "\|" not in section)` (004-06, 08, 09, 10) | |
//!
//! Each is proven against the scripts' own definitions on
//! `data/family004_corpus.txt` (`doccheck::proofs004`).

use super::family004::{NameNorm, norm_name, tables};
use super::pycompat::{PyStr, splitlines};
use super::text::rx;

/// `blocks(text)`: the runs of non-blank lines of `text.splitlines()`, each
/// joined with `\n` (a line of spaces or tabs is blank).
pub fn blank_blocks(text: &str) -> Vec<String> {
    blank_blocks_at(text).into_iter().map(|(_, b)| b).collect()
}

/// [`blank_blocks`] with the 0-based index of each block's first line.
pub fn blank_blocks_at(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    let mut start = 0;
    for (i, line) in splitlines(text).into_iter().enumerate() {
        if !line.py_strip().is_empty() {
            if cur.is_empty() {
                start = i;
            }
            cur.push(line);
        } else if !cur.is_empty() {
            out.push((start, cur.join("\n")));
            cur.clear();
        }
    }
    if !cur.is_empty() {
        out.push((start, cur.join("\n")));
    }
    out
}

fn is_fence(line: &str) -> bool {
    rx(r"^\s*```").is_match(line)
}

fn is_heading(line: &str) -> bool {
    rx(r"^#{1,6}\s").is_match(line.py_lstrip())
}

/// `section_blocks(src_lines)` of 004-19 and 004-20: each heading line is a
/// block of its own; a fence line and the lines up to the closing fence stay
/// in the current block; a blank line ends a block.
pub fn section_blocks<S: AsRef<str>>(lines: &[S]) -> Vec<Vec<String>> {
    section_blocks_at(lines)
        .into_iter()
        .map(|(_, b)| b)
        .collect()
}

/// [`section_blocks`] with the 0-based index of each block's first line.
pub fn section_blocks_at<S: AsRef<str>>(lines: &[S]) -> Vec<(usize, Vec<String>)> {
    let mut out = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut start = 0;
    let mut in_fence = false;
    let flush = |out: &mut Vec<(usize, Vec<String>)>, cur: &mut Vec<String>, start: usize| {
        if !cur.is_empty() {
            out.push((start, std::mem::take(cur)));
        }
    };
    for (i, ln) in lines.iter().enumerate() {
        let ln = ln.as_ref();
        let fence = is_fence(ln);
        if fence || in_fence {
            in_fence ^= fence;
        } else if is_heading(ln) {
            flush(&mut out, &mut cur, start);
            out.push((i, vec![ln.to_string()]));
            continue;
        } else if ln.py_strip().is_empty() {
            flush(&mut out, &mut cur, start);
            continue;
        }
        if cur.is_empty() {
            start = i;
        }
        cur.push(ln.to_string());
    }
    flush(&mut out, &mut cur, start);
    out
}

/// What `block_kind(block)` says a [`section_blocks`] block is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockRole {
    Heading,
    Code,
    Table,
    Prose,
}

impl BlockRole {
    /// The script's name for it.
    pub fn name(self) -> &'static str {
        match self {
            BlockRole::Heading => "heading",
            BlockRole::Code => "code",
            BlockRole::Table => "table",
            BlockRole::Prose => "prose",
        }
    }
}

/// `block_kind(block)`: a single heading line; a block whose first line
/// starts with three backticks; a block whose every line starts with `|`
/// (after `lstrip`); else prose.
pub fn block_role(block: &[String]) -> BlockRole {
    if block.len() == 1 && is_heading(&block[0]) {
        BlockRole::Heading
    } else if block
        .first()
        .is_some_and(|l| l.py_lstrip().starts_with("```"))
    {
        BlockRole::Code
    } else if block.iter().all(|l| l.py_lstrip().starts_with('|')) {
        BlockRole::Table
    } else {
        BlockRole::Prose
    }
}

/// Which tables hold invalid forms, and what a row is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormRows {
    /// `form_rows(text)` of 004-17 and 004-18: a table whose lower-cased
    /// header has a cell containing "form" or "invalid" and one containing
    /// "outcome"; each row long enough gives `[form cell, outcome cell]`
    /// (the first such columns).
    AnyCell004_17,
    /// `Sec.form_rows` of 004-20: a table of two or more columns whose
    /// FIRST header cell, normalised to its letters ([`NameNorm::Letters004_20`]),
    /// contains form, invalid, rule, case or condition, and some header cell
    /// "outcome"; each row of two or more cells whose first cell is not
    /// blank, whole.
    FirstCell004_20,
}

/// The rows of the invalid-form tables of `text` under `rule`.
pub fn form_rows(text: &str, rule: FormRows) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for t in tables(text) {
        let rows = t.rows.iter().map(|r| &r.cells);
        match rule {
            FormRows::AnyCell004_17 => {
                let h: Vec<String> = t.head().iter().map(|c| c.to_lowercase()).collect();
                let fi = h
                    .iter()
                    .position(|c| c.contains("form") || c.contains("invalid"));
                let oi = h.iter().position(|c| c.contains("outcome"));
                let (Some(fi), Some(oi)) = (fi, oi) else {
                    continue;
                };
                for r in rows.filter(|r| r.len() > fi.max(oi)) {
                    out.push(vec![r[fi].clone(), r[oi].clone()]);
                }
            }
            FormRows::FirstCell004_20 => {
                const FORM_KEYS: [&str; 5] = ["form", "invalid", "rule", "case", "condition"];
                let heads: Vec<String> = t
                    .head()
                    .iter()
                    .map(|h| norm_name(h, NameNorm::Letters004_20))
                    .collect();
                if heads.len() >= 2
                    && FORM_KEYS.iter().any(|k| heads[0].contains(k))
                    && heads.iter().any(|h| h.contains("outcome"))
                {
                    out.extend(
                        rows.filter(|r| r.len() >= 2 && !r[0].py_strip().is_empty())
                            .cloned(),
                    );
                }
            }
        }
    }
    out
}

/// True when a section is still a placeholder: absent, blank, or saying
/// "to be filled" (any case, any spacing) with no table pipe in it.
pub fn is_placeholder(text: Option<&str>) -> bool {
    text.is_none_or(|t| {
        t.py_strip().is_empty() || (rx(r"(?i)to\s+be\s+filled").is_match(t) && !t.contains('|'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_as_the_scripts_cut_them() {
        assert_eq!(blank_blocks("a\nb\n  \nc\n\n\nd"), vec!["a\nb", "c", "d"]);
        let lines: Vec<&str> = "p\n## H\n```\n\nx\n```\n| t |\n\nq".lines().collect();
        let b = section_blocks(&lines);
        let roles: Vec<&str> = b.iter().map(|b| block_role(b).name()).collect();
        assert_eq!(roles, ["prose", "heading", "code", "prose"]);
        assert_eq!(b[2], vec!["```", "", "x", "```", "| t |"]);
        assert!(is_placeholder(None) && is_placeholder(Some(" To  be\tfilled. ")));
        assert!(!is_placeholder(Some("To be filled | x |")));
        let t = "| Invalid form | Outcome |\n|---|---|\n| short | TooShort |\n\n| Case | x | Outcome |\n|---|---|---|\n|  | a | b |\n| c | d | e |";
        assert_eq!(
            form_rows(t, FormRows::AnyCell004_17),
            vec![vec!["short", "TooShort"]]
        );
        assert_eq!(
            form_rows(t, FormRows::FirstCell004_20),
            vec![vec!["short", "TooShort"], vec!["c", "d", "e"]]
        );
    }
}
