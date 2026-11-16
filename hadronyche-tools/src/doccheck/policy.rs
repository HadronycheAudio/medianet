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
