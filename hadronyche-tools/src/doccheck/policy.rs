//! Named policies for the rules the scripts implement differently.
//!
//! The lead's ruling for the migration is equivalence first: a port
//! reproduces its own script's variant of each rule, named explicitly at the
//! call site. Unifying variants is a separate, later, reviewed step. The
//! inventory of variants is `swarm/notes/checks-inventory-2026-11-16.md`
//! section 4; the policies here cover the ones the library implements so
//! far, and a porter adds a variant (never changes one) when their script
//! needs another.
//!
//! The other policy types live with their rule: [`super::evidence::TagPolicy`]
//! (tag set, case, unit, word count, minimum), [`super::items::OpenItemPolicy`].

/// How fenced code is recognised (inventory 4f).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FencePolicy {
    /// A line whose trimmed start is `` ``` `` toggles code (the 004 family,
    /// `^\s*```).
    #[default]
    Backticks,
    /// `` ``` `` or `~~~` toggles, any length, no matching (003-02).
    BackticksOrTildes,
    /// An opener of 3 or more backticks or tildes closes only on the same
    /// character with at least its length (006-01, CommonMark).
    Matching,
}

/// How a heading is found by a pattern (inventory 4d).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingMatch {
    /// `re.search(pattern, title)`; add `(?i)` in the pattern for `re.I`.
    Regex,
    /// `re.search` restricted to one level (the scripts' `span_of`).
    RegexAtLevel(usize),
    /// `re.fullmatch(pattern, title, re.I)` (004-09, 004-10, 004-11).
    FullMatch,
    /// Case-insensitive substring of the title (003-02, 003-05).
    Substring,
    /// Exact, case-sensitive title equality (006-02).
    Exact,
    /// Equality after [`super::markdown::norm_title`] (004-21).
    NormEq,
    /// The heading's GitHub anchor equals the pattern (004-22 links).
    Slug,
}

/// Where a section ends (inventory 4e).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SectionEnd {
    /// At the next heading of the same or a higher level.
    #[default]
    SameOrHigher,
    /// At the next heading of any level (003-08).
    AnyHeading,
    /// At the next heading of level 2 or higher (`^## `, 002-10 to 002-14).
    NextLevel2,
}

/// How a table line is split into cells (inventory 4i).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowSplit {
    /// Split on `|` not preceded by `\`, then restore `\|` (most 004
    /// scripts, and GitHub).
    #[default]
    Escaped,
    /// Plain `split("|")`: an escaped pipe breaks the cell (004-22).
    Plain,
}

/// How a document's text becomes lines (inventory 4f; review finding 1c).
/// Every script read with `open()` in text mode, so `\r\n` and a lone `\r`
/// are already `\n` (universal newlines, [`super::pycompat::universal_newlines`]);
/// the variants differ in how they split after that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineSplit {
    /// `text.split("\n")` (004-22): U+2028, form feed and the other Python
    /// boundaries stay inside a line.
    #[default]
    Nl,
    /// `text.splitlines()` (the 004 family): every Python line boundary
    /// ([`super::pycompat::splitlines`]) starts a line.
    PySplitlines,
}

impl LineSplit {
    /// `text` as the script saw it, rejoined with `\n`: universal newlines,
    /// then for [`LineSplit::PySplitlines`] every Python boundary as `\n`.
    pub fn normalize(self, text: &str) -> String {
        let text = super::pycompat::universal_newlines(text);
        match self {
            LineSplit::Nl => text,
            LineSplit::PySplitlines => super::pycompat::splitlines(&text).join("\n"),
        }
    }
}

/// Which second line makes a run of `|` lines a table with a header
/// (inventory 4i; review finding 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeparatorRule {
    /// Every non-empty cell is `:?-+:?` and one cell is not empty (004-22).
    #[default]
    CellWise,
    /// The line matches `^\s*\|?[\s:|-]+\|?\s*$` and holds a `-` (the 004
    /// family's `parse_tables`, 23 scripts).
    LineRegex,
}

/// What a run of `|` lines without a separator row is (review finding 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeaderlessRuns {
    /// A table with no header whose every line is a row (004-22).
    #[default]
    Keep,
    /// Not a table (the 004 family).
    Drop,
}

/// How a table is read: row splitting, separator rule, headerless runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TableRules {
    pub split: RowSplit,
    pub separator: SeparatorRule,
    pub headerless: HeaderlessRuns,
}

impl TableRules {
    /// 004-22's `tables_in`.
    pub const T004_22: TableRules = TableRules {
        split: RowSplit::Plain,
        separator: SeparatorRule::CellWise,
        headerless: HeaderlessRuns::Keep,
    };
    /// The 004 family's `parse_tables` (004-01, 03-06, 08-13, 16-21, 24-29).
    pub const FAMILY_004: TableRules = TableRules {
        split: RowSplit::Escaped,
        separator: SeparatorRule::LineRegex,
        headerless: HeaderlessRuns::Drop,
    };
}

/// How a relative link path is resolved (review finding 1a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PathNorm {
    /// `os.path.normpath(os.path.join(base, rel))`: a leading `..` is kept,
    /// so a path that climbs above the repository names no document of the
    /// [`super::Docs`] (the only variant any script used).
    #[default]
    PyNormpath,
}

/// Which TOML reader the script used (lead ruling R14). A port reads
/// `wire.toml` through a faithful port of ITS script's reader, so the
/// script's verdict on any input is reproduced, not excused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TomlReader {
    /// `tomllib` (004-22): the crate's TOML parser, values as Python's
    /// ([`super::tomllib`]); its differences from `tomllib` on the corpus are
    /// listed in `tomllib::KNOWN_DIFFERENCES`.
    Tomllib,
    /// The statement-scanner subset reader of 004-03, 05, 06, 08-13, 16-20
    /// and 24-29, ported in [`super::tomlsubset`]. The corpus generator runs
    /// all twenty scripts' copies and they give the same result on every
    /// snippet (004-03 and 004-05 differ only in formatting: a dict literal
    /// on several lines, `%`-formatted messages that print the same).
    Subset004_18,
    /// The line reader of 004-14 ([`super::tomlline`], `Variant::V004_14`):
    /// `(messages, others)`, strings unescaped (`\"`, `\\`).
    Line004_14,
    /// The line reader of 004-15 ([`super::tomlline`], `Variant::V004_15`):
    /// strings kept as written; a field before any message is an
    /// `IndexError`, a `[[x]]` after `[x]` an uncaught `AttributeError`.
    Line004_15,
}

impl TomlReader {
    /// The [`super::tomlline::Variant`] of a line reader.
    pub fn line_variant(self) -> Option<super::tomlline::Variant> {
        match self {
            TomlReader::Line004_14 => Some(super::tomlline::Variant::V004_14),
            TomlReader::Line004_15 => Some(super::tomlline::Variant::V004_15),
            TomlReader::Tomllib | TomlReader::Subset004_18 => None,
        }
    }
}

/// The regex word class (lead ruling R1). Not a parameter: every pattern
/// given to [`super::text::rx`] has Python's `\w`/`\s`
/// ([`super::pycompat::translate`]); [`super::text::rx_raw`] is for the
/// library's own Unicode-property patterns only. Listed here so the policy
/// list is complete.
pub const WORD_CLASS: &str = "python";
