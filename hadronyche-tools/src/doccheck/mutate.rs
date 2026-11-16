//! Mutations of documents: the helpers tests use to build denial fixtures
//! from the real documents, and the structured mutation corpus the
//! equivalence harness generates.
//!
//! The text helpers (`delete_row`, `delete_unit`, ...) locate their target
//! by content, so a test keeps pointing at the same row when lines move;
//! each panics when its target is not found or not unique enough to mean
//! anything, because a fixture that mutates nothing proves nothing.

use super::docs::Docs;
use super::markdown::Markdown;
use super::text::{rx, split_nl};
use super::units::{UnitKind, units};

/// One structured edit of a file, by 0-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Delete lines `lo..=hi`.
    DeleteLines { lo: usize, hi: usize },
    /// Remove the tag `[tag]` (with its backticks, if any) from a line.
    StripTag { line: usize, tag: String },
    /// Replace the bytes `start..end` of a line by `to`.
    ReplaceSpan {
        line: usize,
        start: usize,
        end: usize,
        to: String,
    },
    /// Replace the `nth` (0-based) occurrence of `from` on a line by `to`.
    Replace {
        line: usize,
        from: String,
        to: String,
        nth: usize,
    },
}

/// A named edit of one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mutation {
    pub file: String,
    pub edit: Edit,
    /// What it does, for the harness table (`delete row 3364: | C-17 | ...`).
    pub label: String,
}

impl Mutation {
    /// The edit applied to `text`.
    pub fn apply_text(&self, text: &str) -> String {
        let mut lines: Vec<String> = split_nl(text).into_iter().map(String::from).collect();
        match &self.edit {
            Edit::DeleteLines { lo, hi } => {
                lines.drain(*lo..=(*hi).min(lines.len() - 1));
            }
            Edit::StripTag { line, tag } => {
                let l = &lines[*line];
                lines[*line] = l
                    .replace(&format!("`[{tag}]`"), "")
                    .replace(&format!("[{tag}]"), "");
            }
            Edit::ReplaceSpan {
                line,
                start,
                end,
                to,
            } => {
                let l = &lines[*line];
                lines[*line] = format!("{}{}{}", &l[..*start], to, &l[*end..]);
            }
            Edit::Replace {
                line,
                from,
                to,
                nth,
            } => {
                lines[*line] = replace_nth(&lines[*line], from, to, *nth);
            }
        }
        lines.join("\n")
    }

    /// `docs` with the mutation applied in memory.
    pub fn apply(&self, docs: &Docs) -> Docs {
        docs.mutate(&self.file, |t| self.apply_text(t))
    }
}

fn replace_nth(line: &str, from: &str, to: &str, nth: usize) -> String {
    match line.match_indices(from).nth(nth) {
        Some((at, _)) => format!("{}{}{}", &line[..at], to, &line[at + from.len()..]),
        None => line.to_string(),
    }
}

fn first_line(text: &str, pattern: &str, what: &str) -> usize {
    let re = rx(pattern);
    split_nl(text)
        .iter()
        .position(|l| re.is_match(l))
        .unwrap_or_else(|| panic!("{what}: no line matches {pattern:?}"))
}

fn drop_lines(text: &str, lo: usize, hi: usize) -> String {
    let lines = split_nl(text);
    let mut out: Vec<&str> = lines[..lo].to_vec();
    out.extend_from_slice(&lines[hi + 1..]);
    out.join("\n")
}

/// Deletes the first table row (a line starting with `|`) matching `pattern`.
pub fn delete_row(text: &str, pattern: &str) -> String {
    let re = rx(pattern);
    let i = split_nl(text)
        .iter()
        .position(|l| l.trim_start().starts_with('|') && re.is_match(l))
        .unwrap_or_else(|| panic!("delete_row: no table row matches {pattern:?}"));
    drop_lines(text, i, i)
}

/// Deletes the paragraph or list item ([`units`]) whose text matches
/// `pattern`.
pub fn delete_unit(text: &str, pattern: &str) -> String {
    let re = rx(pattern);
    let u = units(text)
        .into_iter()
        .find(|u| {
            matches!(u.kind, UnitKind::Paragraph | UnitKind::ListItem) && re.is_match(&u.text)
        })
        .unwrap_or_else(|| panic!("delete_unit: no unit matches {pattern:?}"));
    drop_lines(text, u.lo, u.hi)
}

/// Deletes the section of the first heading matching `pattern` (heading
/// included).
pub fn delete_section(text: &str, pattern: &str) -> String {
    let md = Markdown::parse(text);
    let h = md
        .find(pattern)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("delete_section: no heading matches {pattern:?}"));
    let span = md.section(h);
    drop_lines(text, span.lo, span.hi - 1)
}

/// Applies `edit` to the section of the first heading matching `pattern`
/// only (heading line included), so a fixture cannot hit the same words in
/// an earlier section.
pub fn within_section(text: &str, pattern: &str, edit: impl FnOnce(&str) -> String) -> String {
    let md = Markdown::parse(text);
    let h = md
        .find(pattern)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("within_section: no heading matches {pattern:?}"));
    let span = md.section(h);
    let lines = split_nl(text);
    let mut out: Vec<String> = lines[..span.lo].iter().map(|l| l.to_string()).collect();
    out.push(edit(&lines[span.lo..span.hi].join("\n")));
    out.extend(lines[span.hi..].iter().map(|l| l.to_string()));
    out.join("\n")
}

/// Renames the first heading matching `pattern` to `to` (level kept).
pub fn rename_heading(text: &str, pattern: &str, to: &str) -> String {
    let md = Markdown::parse(text);
    let h = md
        .find(pattern)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("rename_heading: no heading matches {pattern:?}"));
    let mut lines = split_nl(text);
    let new = format!("{} {to}", "#".repeat(h.level));
    lines[h.line] = &new;
    lines.join("\n")
}

/// Removes every evidence tag from the unit matching `pattern`.
pub fn strip_tags(text: &str, pattern: &str) -> String {
    let re = rx(pattern);
    let u = units(text)
        .into_iter()
        .find(|u| re.is_match(&u.text))
        .unwrap_or_else(|| panic!("strip_tags: no unit matches {pattern:?}"));
    let tag = rx(r"`?\[(?:code|measured|kernel-doc|standard|memory|decision)\]`?");
    let mut lines: Vec<String> = split_nl(text).into_iter().map(String::from).collect();
    for l in &mut lines[u.lo..=u.hi] {
        *l = tag.replace_all(l, "").into_owned();
    }
    lines.join("\n")
}

/// Replaces the first match of `pattern` on the first line matching
/// `line_pattern` by `to` (for numbers: `replace_in_line(t, "^\\| C-17", "1252", "1253")`).
pub fn replace_in_line(text: &str, line_pattern: &str, pattern: &str, to: &str) -> String {
    let i = first_line(text, line_pattern, "replace_in_line");
    let mut lines = split_nl(text);
    let re = rx(pattern);
    assert!(
        re.is_match(lines[i]),
        "replace_in_line: {pattern:?} not on line {i}"
    );
    let new = re.replace(lines[i], regex::NoExpand(to)).into_owned();
    lines[i] = &new;
    lines.join("\n")
}

/// Negates `keyword` on the first line matching `line_pattern`: inserts
/// "not " before it.
pub fn negate(text: &str, line_pattern: &str, keyword: &str) -> String {
    replace_in_line(
        text,
        line_pattern,
        &regex::escape(keyword),
        &format!("not {keyword}"),
    )
}

/// Replaces every match of `pattern` in the whole text.
pub fn replace_all(text: &str, pattern: &str, to: &str) -> String {
    let re = rx(pattern);
    assert!(re.is_match(text), "replace_all: {pattern:?} not found");
    re.replace_all(text, regex::NoExpand(to)).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: &str = "## A\n\nPara one [decision].\n\n| x | y |\n|---|---|\n| r1 | 7 |\n| r2 | 8 |\n\n## B\n\nis it\n";

    #[test]
    fn content_helpers() {
        assert!(!delete_row(T, "r1").contains("r1"));
        assert!(delete_row(T, "r1").contains("r2"));
        assert!(!delete_unit(T, "Para one").contains("Para"));
        let s = delete_section(T, "^A$");
        assert!(s.starts_with("## B"));
        assert!(rename_heading(T, "^B$", "C").contains("## C\n"));
        assert!(!strip_tags(T, "Para").contains("[decision]"));
        assert!(replace_in_line(T, r"\| r2", "8", "9").contains("| r2 | 9 |"));
        assert!(negate(T, "^is", "it").contains("is not it"));
        assert_eq!(replace_all("a1a1", "1", "2"), "a2a2");
        let w = within_section(T, "^B$", |s| s.replace("is", "was"));
        assert!(w.contains("was it") && w.starts_with("## A"));
    }

    #[test]
    #[should_panic(expected = "no table row matches")]
    fn a_missing_target_panics() {
        delete_row(T, "r9");
    }

    #[test]
    fn structured_edits() {
        let m = Mutation {
            file: "f".into(),
            edit: Edit::Replace {
                line: 6,
                from: "7".into(),
                to: "70".into(),
                nth: 0,
            },
            label: String::new(),
        };
        assert!(m.apply_text(T).contains("| r1 | 70 |"));
        let d = Mutation {
            edit: Edit::DeleteLines { lo: 6, hi: 7 },
            ..m.clone()
        };
        assert!(!d.apply_text(T).contains("r2"));
        let s = Mutation {
            edit: Edit::StripTag {
                line: 2,
                tag: "decision".into(),
            },
            ..m
        };
        assert_eq!(split_nl(&s.apply_text(T))[2], "Para one .");
    }
}
