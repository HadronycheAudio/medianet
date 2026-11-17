//! Rules several 002 scripts write the same way, as named variants (the
//! inventory's section 4 a, c, j, k). A port names its script's variant at
//! the call site; messages and criteria stay in the port.
//!
//! | rule | variant | scripts |
//! |---|---|---|
//! | evidence tag (`has_tag`) | [`TagSet::Five`] | 002-03, 05, 06, 07, 08, 10, 11, 12, 14 |
//! | | [`TagSet::Six`] (adds `[standard]`) | 002-04 |
//! | file:line pointer (`has_ptr`) | [`PtrRule::Plain`] | 002-03, 04, 10 |
//! | | [`PtrRule::Range`] (`-N` allowed) | 002-05, 06 |
//! | pointer lines tagged on the line or the next one, clipped at the sub-section's end (`check_tag_discipline`) | [`untagged_pointer_lines`] | 002-03, 04 |
//! | open items owned (`check_open_items`) | [`unowned_open_items`] | 002-03, 04 |
//! | one section verdict line, pending accepted (`verdict:` count, first line, body) | [`section_verdict`] | 002-03, 04, 05, 07, 10, 11, 12, 14 |

use super::ere::{Grep, Sed};
use super::shell::{Lines, tr_lower};

/// The evidence tags a script accepts (case-sensitive, bracketed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagSet {
    /// `[code] [measured] [kernel-doc] [memory] [decision]`.
    Five,
    /// The five and `[standard]` (002-04).
    Six,
}

impl TagSet {
    /// The script's ERE.
    pub fn pattern(self) -> &'static str {
        match self {
            TagSet::Five => r"\[(code|measured|kernel-doc|memory|decision)\]",
            TagSet::Six => r"\[(code|measured|kernel-doc|memory|decision|standard)\]",
        }
    }
}

/// `has_tag`: any line of `text` carries a tag of `tags`.
pub fn has_tag(text: &str, tags: TagSet) -> bool {
    Grep::ere(tags.pattern()).any_line(text)
}

/// The file:line pointer a script recognises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtrRule {
    /// `[A-Za-z0-9_./-]+\.rs:[0-9]+`.
    Plain,
    /// The same with an optional `-N` range (002-05, 002-06).
    Range,
}

impl PtrRule {
    /// The script's ERE.
    pub fn pattern(self) -> &'static str {
        match self {
            PtrRule::Plain => r"[A-Za-z0-9_./-]+\.rs:[0-9]+",
            PtrRule::Range => r"[A-Za-z0-9_./-]+\.rs:[0-9]+(-[0-9]+)?",
        }
    }
}

/// `has_ptr`: any line of `text` carries a pointer. (A letter range makes
/// the pattern one the DFA does not support: the superset DFA and the
/// regex matcher decide, so `x.rs:²` is a pointer only when an ASCII digit
/// follows some `.rs:` on the line.)
pub fn has_ptr(text: &str, rule: PtrRule) -> bool {
    Grep::ere(rule.pattern()).any_line(text)
}

/// `check_tag_discipline` (002-03, 002-04): the lines `k` of
/// `start..=end` that carry a pointer while the block of `k` and `k + 1`
/// (clipped at `end`) carries no tag.
pub fn untagged_pointer_lines(
    lines: &Lines,
    start: usize,
    end: usize,
    ptr: PtrRule,
    tags: TagSet,
) -> Vec<usize> {
    (start..=end)
        .filter(|k| has_ptr(lines.sed_line(*k), ptr))
        .filter(|k| !has_tag(&lines.line_range(*k, (k + 1).min(end)), tags))
        .collect()
}

/// `check_open_items` (002-03, 002-04): the lines `k` of `start..=end`
/// whose ASCII-lowercased text raises an open item (`open item`, `open
/// question`, `todo`, `tbd`, `unresolved` as words) without a negation
/// (`no`/`none`/`not` up to 30 non-dots before `open`), while the block of
/// `k` and `k + 1` (clipped at `end`) lacks `owner` or `falsifier`
/// (case-insensitive substrings).
pub fn unowned_open_items(lines: &Lines, start: usize, end: usize) -> Vec<usize> {
    let raised = Grep::ere(r"open (item|question)|\btodo\b|\btbd\b|\bunresolved\b");
    let negated = Grep::ere(r"(^|[^a-z])(no|none|not)[^.]{0,30}open");
    let (owner, falsifier) = (Grep::bre_i("owner"), Grep::bre_i("falsifier"));
    (start..=end)
        .filter(|k| {
            let low = tr_lower(lines.sed_line(*k));
            raised.matches(&low) && !negated.matches(&low)
        })
        .filter(|k| {
            let block = lines.line_range(*k, (k + 1).min(end));
            !owner.any_line(&block) || !falsifier.any_line(&block)
        })
        .collect()
}

/// The form of a section's one verdict line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerdictForm {
    /// The body is `pending` (case-insensitive, whole).
    Pending,
    /// The body holds `mechanism:` (an instrument-bug finding).
    Mechanism,
    /// The body holds `explanation:` (a not-a-bug finding).
    Explanation,
    /// None of the three.
    Other,
}

/// The verdict lines of a section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionVerdict {
    /// No line matches `^[[:space:]]*(\*\*)?verdict:`.
    Missing,
    /// That many lines match.
    Many(usize),
    /// Exactly one: the line, and the form of its body.
    One { line: String, form: VerdictForm },
}

/// The section-marker rule of 002-03/04/05/07/10/11/12/14 on `section`
/// (the section's text as `$(sed -n …)` gives it): count the lines matching
/// `^[[:space:]]*(\*\*)?verdict:` (case-sensitive `verdict`); with one,
/// strip the marker and a trailing `**` from it (`sed -E`) and classify the
/// body: `^pending$`, then `mechanism:`, then `explanation:` (each
/// case-insensitive).
pub fn section_verdict(section: &str) -> SectionVerdict {
    let marker = Grep::ere(r"^[[:space:]]*(\*\*)?verdict:");
    let count = marker.count_lines(section);
    if count == 0 {
        return SectionVerdict::Missing;
    }
    if count > 1 {
        return SectionVerdict::Many(count);
    }
    let line = section
        .split('\n')
        .find(|l| marker.matches(l))
        .unwrap_or("")
        .to_string();
    let body = Sed::ere(
        r"s/^[[:space:]]*(\*\*)?verdict:[[:space:]]*//; s/[[:space:]]*\*\*[[:space:]]*$//",
    )
    .apply(&line);
    let form = if Grep::ere_i("^pending$").matches(&body) {
        VerdictForm::Pending
    } else if Grep::ere_i("mechanism:").matches(&body) {
        VerdictForm::Mechanism
    } else if Grep::ere_i("explanation:").matches(&body) {
        VerdictForm::Explanation
    } else {
        VerdictForm::Other
    };
    SectionVerdict::One { line, form }
}
