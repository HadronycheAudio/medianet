//! The closed rejection-reason enumeration of `wire.md`, read the way each
//! 004 script reads it, with every variant a named policy.
//!
//! The scripts that judge invalid-form outcomes parse the members of the
//! "Rejection reasons" enumeration (and 004-26 also the "Refusal reasons")
//! from `doc/design/spec/wire.md`. They share one table reader and one
//! heading index (their `split_row`, `parse_tables` and `heading_index` are
//! the same code in every script, docstrings aside), and differ in three
//! places, each a policy here:
//!
//! * which section ([`SectionRule`]): the first heading of any level whose
//!   title matches `rejection[-\s]+reasons?` (004-16 to 004-20), or the first
//!   at level 2 (004-24, 004-26) or 3 (004-26's refusals), with the pattern
//!   unanchored (004-24, 004-26) or anchored `^rejection\s+reasons$` (004-25,
//!   004-28, 004-29);
//! * which rows ([`RowRule`]): at least two cells, a `\d+` first cell and a
//!   second cell that, stripped of `` ` ``, `*`, `_` and spaces, is
//!   `[A-Z][A-Za-z0-9]+` ([`RowRule::Loose`]: 004-16 to 004-20, 004-26); or
//!   at least four cells and a bare `[A-Z][A-Za-z0-9]+` second cell
//!   ([`RowRule::Layered`]: 004-24, 004-25, 004-28, 004-29, which also read
//!   the Layers cell);
//! * what the check then demands, which stays in the check as a named
//!   constant beside its message: an exact count ([`Enumeration::has_exactly`]
//!   with the check's own pin; 004-16 to 004-20, 004-24, 004-27), 13 and 18
//!   (004-26, [`REJECTION_004_26`] and [`REFUSAL_004_26`]), the fixed names
//!   and values ([`Enumeration::matches_fixed`] with [`REJ_FIXED`]; 004-25,
//!   004-28), 13 members with seven named control reasons
//!   ([`CONTROL_REASONS_004_29`]).
//!
//! Three more readings are not a section parse:
//! * 004-06, 004-08, 004-09 and 004-10 start from [`DEFAULT_13`] and replace
//!   it with the default names found in any cell of the section's tables
//!   when at least ten are found ([`tolerant_members`],
//!   [`TOLERANT_004_06`]); their headers say "13-name", their code does not
//!   pin the count (the port follows the code);
//! * 004-11 reads the first table of the whole file whose header is
//!   `value…` and `name` ([`value_name_table`]) and compares the set with
//!   [`DEFAULT_13`];
//! * 004-18 reads the refusal set from the constant C-185
//!   ([`refusal_set`]) and the numbered rows of its Admission messages
//!   section ([`numbered_rows`]).
//!
//! Every variant is proven against the scripts' own Python on
//! `data/rejection_corpus.txt` (`tools/gen_rejection_corpus.py`, which runs
//! the code taken from each script with `ast`).
//!
//! Values are Python's unbounded `int`: [`Member::value`] is the canonical
//! decimal text, [`Member::int`] the value when it fits `i128`.

pub mod reasons;

use super::pycompat::{PyStr, parse_int, splitlines, universal_newlines};
use super::text::{full, has_i, rx};

/// The 13 names, in value order (0 to 12): 004-06/08/09/10's default
/// `MEMBERS`, 004-11's `MEMBERS_EXPECTED`.
pub const DEFAULT_13: [&str; 13] = [
    "TooShort",
    "BadMagic",
    "VersionMismatch",
    "ForeignDomain",
    "UnconfiguredDomain",
    "UnknownStream",
    "UnadmittedSource",
    "BadLength",
    "BadChecksum",
    "BadPacketIndex",
    "WrongEpoch",
    "OversizeControl",
    "ReservedNonZero",
];

/// 004-25's and 004-28's `REJ_FIXED`: the 13 names with their values.
pub const REJ_FIXED: [(&str, i128); 13] = [
    ("TooShort", 0),
    ("BadMagic", 1),
    ("VersionMismatch", 2),
    ("ForeignDomain", 3),
    ("UnconfiguredDomain", 4),
    ("UnknownStream", 5),
    ("UnadmittedSource", 6),
    ("BadLength", 7),
    ("BadChecksum", 8),
    ("BadPacketIndex", 9),
    ("WrongEpoch", 10),
    ("OversizeControl", 11),
    ("ReservedNonZero", 12),
];

/// The seven control reasons 004-29 requires among the members whose
/// Layers cell contains `control`.
pub const CONTROL_REASONS_004_29: [&str; 7] = [
    "TooShort",
    "BadMagic",
    "VersionMismatch",
    "ForeignDomain",
    "OversizeControl",
    "ReservedNonZero",
    "BadLength",
];

/// Which heading opens the section (its title searched with `re.I`); the
/// section runs to the next heading of the same or a higher level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionRule {
    /// The first heading of any level (`section_span(pattern)`).
    AnyLevel(&'static str),
    /// The first heading of exactly this level (`span_of(pattern, level)`,
    /// `section_lines(text, pattern, level)`).
    AtLevel(&'static str, usize),
}

/// Which table rows are members.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowRule {
    /// `len(row) >= 2`, `fullmatch(\d+, row[0].strip())`, name
    /// `row[1].strip().strip("`*_ ")` fullmatching `[A-Z][A-Za-z0-9]+`.
    Loose,
    /// `len(row) >= 4`, `fullmatch(\d+, row[0].strip())`, name
    /// `row[1].strip()` fullmatching `[A-Z][A-Za-z0-9]+` (no markup
    /// stripped).
    Layered,
}

/// A section rule with a row rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumPolicy {
    pub section: SectionRule,
    pub rows: RowRule,
}

const REJECTION: &str = r"rejection[-\s]+reasons?";

/// 004-16, 004-17, 004-18, 004-19, 004-20.
pub const REJECTION_004_16: EnumPolicy = EnumPolicy {
    section: SectionRule::AnyLevel(REJECTION),
    rows: RowRule::Loose,
};
/// 004-24.
pub const REJECTION_004_24: EnumPolicy = EnumPolicy {
    section: SectionRule::AtLevel(REJECTION, 2),
    rows: RowRule::Layered,
};
/// 004-25, 004-28, 004-29.
pub const REJECTION_004_25: EnumPolicy = EnumPolicy {
    section: SectionRule::AtLevel(r"^rejection\s+reasons$", 2),
    rows: RowRule::Layered,
};
/// 004-26's Reject names.
pub const REJECTION_004_26: EnumPolicy = EnumPolicy {
    section: SectionRule::AtLevel(REJECTION, 2),
    rows: RowRule::Loose,
};
/// 004-26's refusal names.
pub const REFUSAL_004_26: EnumPolicy = EnumPolicy {
    section: SectionRule::AtLevel(r"refusal[-\s]+reasons?", 3),
    rows: RowRule::Loose,
};

/// One member of the enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// The name, e.g. `BadMagic`.
    pub name: String,
    /// The value as Python's `int()` reads it, in ASCII decimal without
    /// leading zeros (Python's integers are unbounded).
    pub value: String,
    /// Every cell of the row, stripped (cell 3 is the Layers cell).
    pub cells: Vec<String>,
}

impl Member {
    /// The value when it fits `i128`.
    pub fn int(&self) -> Option<i128> {
        parse_int(&self.value).ok()
    }

    /// The Layers cell (`row[3]`), or `""` when the row has fewer than four
    /// cells (004-20's `row[3] if len(row) >= 4 else ""`).
    pub fn layers(&self) -> &str {
        self.cells.get(3).map_or("", String::as_str)
    }
}

/// The enumeration as one policy reads it from one `wire.md` text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Enumeration {
    /// A heading matching the policy's section rule exists.
    pub section_found: bool,
    /// The members in first-insertion order; a later row with the same name
    /// replaces the earlier one's value and cells and keeps its place (a
    /// Python dict).
    pub members: Vec<Member>,
}

impl Enumeration {
    /// Parses the enumeration from the text of `wire.md` (read as `open()`
    /// reads it: universal newlines).
    pub fn parse(wire_md: &str, policy: EnumPolicy) -> Enumeration {
        let text = universal_newlines(wire_md);
        let lines = splitlines(&text);
        let Some(body) = section(&lines, policy.section) else {
            return Enumeration::default();
        };
        let mut out = Enumeration {
            section_found: true,
            members: Vec::new(),
        };
        for m in rows_of(&body, policy.rows) {
            match out.members.iter_mut().find(|x| x.name == m.name) {
                Some(x) => *x = m,
                None => out.members.push(m),
            }
        }
        out
    }

    /// The number of distinct names (`len(MEMBERS)`).
    pub fn count(&self) -> usize {
        self.members.len()
    }

    /// `len(MEMBERS) == pin`: the pin is the check's own constant.
    pub fn has_exactly(&self, pin: usize) -> bool {
        self.count() == pin
    }

    /// The names, in first-insertion order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.members.iter().map(|m| m.name.as_str())
    }

    /// `name in MEMBERS` (case-sensitive).
    pub fn contains(&self, name: &str) -> bool {
        self.members.iter().any(|m| m.name == name)
    }

    /// The member named `name`.
    pub fn get(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.name == name)
    }

    /// `CONTROL_MEMBERS`: the names whose Layers cell contains `control`
    /// (case-sensitive substring), in insertion order.
    pub fn control_members(&self) -> Vec<&str> {
        self.members
            .iter()
            .filter(|m| m.layers().contains("control"))
            .map(|m| m.name.as_str())
            .collect()
    }

    /// 004-25/28's DoD: `[(n, v) for …] and sorted(REJ_VALUE.items(),
    /// key=lambda x: x[1]) == fixed` (a stable sort by value: members with
    /// equal values keep their insertion order).
    pub fn matches_fixed(&self, fixed: &[(&str, i128)]) -> bool {
        if self.members.is_empty() {
            return false;
        }
        let mut items: Vec<&Member> = self.members.iter().collect();
        items.sort_by(|a, b| (a.value.len(), &a.value).cmp(&(b.value.len(), &b.value)));
        items.len() == fixed.len()
            && items
                .iter()
                .zip(fixed)
                .all(|(m, (n, v))| m.name == *n && m.value == v.to_string())
    }

    /// 004-29's DoD: 13 members, and every name of `required` among the
    /// control members.
    pub fn has_control_reasons(&self, pin: usize, required: &[&str]) -> bool {
        let control = self.control_members();
        self.has_exactly(pin) && required.iter().all(|r| control.contains(r))
    }
}

/// The default-replacing reading of 004-06/08/09/10.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tolerant {
    /// The names the script starts from.
    pub default: &'static [&'static str],
    /// How many default names found in the section replace the default.
    pub at_least: usize,
}

/// 004-06, 004-08, 004-09, 004-10 (004-06 has a `>= 13` branch that does
/// the same as its `>= 10` one).
pub const TOLERANT_004_06: Tolerant = Tolerant {
    default: &DEFAULT_13,
    at_least: 10,
};

/// `MEMBERS` of 004-06/08/09/10: the default, replaced by the sorted set of
/// default names found as a whole cell (`[`*_\s]*NAME[`*_\s]*`) in the
/// tables of the first section of any level titled `rejection[-\s]+reasons?`
/// when at least `policy.at_least` are found. The names can only shrink.
pub fn tolerant_members(wire_md: &str, policy: Tolerant) -> Vec<String> {
    let default = || policy.default.iter().map(|s| s.to_string()).collect();
    let text = universal_newlines(wire_md);
    let lines = splitlines(&text);
    let Some(body) = section(&lines, SectionRule::AnyLevel(REJECTION)) else {
        return default();
    };
    let cell_name = rx(r"^[`*_\s]*([A-Z][A-Za-z0-9]+)[`*_\s]*\z");
    let mut found = std::collections::BTreeSet::new();
    for table in parse_tables(&body) {
        for cell in table.rows.iter().flatten() {
            if let Some(c) = cell_name.captures(cell.py_strip())
                && policy.default.contains(&&c[1])
            {
                found.insert(c[1].to_string());
            }
        }
    }
    if found.len() >= policy.at_least {
        found.into_iter().collect()
    } else {
        default()
    }
}

/// 004-11's `enum_names`: the second cells (rows of at least two cells) of
/// the first table of the whole text whose normalised header (`[^a-z0-9]`
/// removed from the lowercased cell) starts `value` and then is `name`.
pub fn value_name_table(wire_md: &str) -> Option<Vec<String>> {
    let norm = |s: &str| -> String {
        s.to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            .collect()
    };
    for t in parse_tables(&universal_newlines(wire_md)) {
        let h: Vec<String> = t.header.iter().map(|c| norm(c)).collect();
        if h.len() >= 2 && h[0].starts_with("value") && h[1] == "name" {
            return Some(
                t.rows
                    .iter()
                    .filter(|r| r.len() >= 2)
                    .map(|r| r[1].clone())
                    .collect(),
            );
        }
    }
    None
}

/// 004-18's refusal set from the value of C-185: the CamelCase words
/// (`\b[A-Z][a-z]+(?:[A-Z][A-Za-z]*)+\b`) of the text before the first
/// `". "`, parenthetical glosses (`\([^)]*\)`) removed, first occurrence
/// kept.
pub fn refusal_set(c185: &str) -> Vec<String> {
    let seg = c185.split(". ").next().unwrap_or("");
    let seg = rx(r"\([^)]*\)").replace_all_literal(seg, "");
    let mut out: Vec<String> = Vec::new();
    for m in rx(r"\b[A-Z][a-z]+(?:[A-Z][A-Za-z]*)+\b").find_iter(&seg) {
        if !out.iter().any(|x| x == m.as_str()) {
            out.push(m.as_str().to_string());
        }
    }
    out
}

/// Every member row of every table of `text` under `rule`, in order and
/// with duplicates (004-18 reads its refusal enumeration this way and fails
/// on a name listed twice).
pub fn numbered_rows(text: &str, rule: RowRule) -> Vec<Member> {
    rows_of(&universal_newlines(text), rule)
}

fn rows_of(text: &str, rule: RowRule) -> Vec<Member> {
    let mut out = Vec::new();
    for t in parse_tables(text) {
        for row in t.rows {
            if let Some(m) = member(row, rule) {
                out.push(m);
            }
        }
    }
    out
}

fn member(row: Vec<String>, rule: RowRule) -> Option<Member> {
    let min = match rule {
        RowRule::Loose => 2,
        RowRule::Layered => 4,
    };
    if row.len() < min {
        return None;
    }
    let digits = row[0].py_strip();
    if !full(r"\d+", digits) {
        return None;
    }
    let name = match rule {
        RowRule::Loose => row[1].py_strip().trim_matches(|c| "`*_ ".contains(c)),
        RowRule::Layered => row[1].py_strip(),
    };
    if !full(r"[A-Z][A-Za-z0-9]+", name) {
        return None;
    }
    Some(Member {
        name: name.to_string(),
        value: canonical(digits),
        cells: row.clone(),
    })
}

/// `str(int(s))` for a string of Unicode decimal digits.
fn canonical(digits: &str) -> String {
    let mut out = String::new();
    for c in digits.chars() {
        // every character matched `\d`: one decimal digit each
        let Ok(d) = parse_int(c.encode_utf8(&mut [0; 4])) else {
            continue;
        };
        if !(out.is_empty() && d == 0) {
            out.push_str(&d.to_string());
        }
    }
    if out.is_empty() {
        out.push('0');
    }
    out
}

/// The section's text (`"\n".join(lines[start:end])`), or `None` when no
/// heading matches.
fn section(lines: &[&str], rule: SectionRule) -> Option<String> {
    let heads = heading_index(lines);
    let (pattern, level) = match rule {
        SectionRule::AnyLevel(p) => (p, None),
        SectionRule::AtLevel(p, l) => (p, Some(l)),
    };
    let k = heads
        .iter()
        .position(|h| level.is_none_or(|l| h.1 == l) && has_i(pattern, &h.2))?;
    let (start, lvl) = (heads[k].0 + 1, heads[k].1);
    let end = heads[k + 1..]
        .iter()
        .find(|h| h.1 <= lvl)
        .map_or(lines.len(), |h| h.0);
    Some(lines[start.min(end)..end].join("\n"))
}

/// One heading: (line index, level, title).
pub type Heading = (usize, usize, String);

/// The 004 family's `heading_index`: `^(#{1,6})\s+(.*?)\s*#*\s*$` on every
/// line outside fences (a fence is a line matching `^\s*```` and toggles).
pub fn heading_index(lines: &[&str]) -> Vec<Heading> {
    let head = rx(r"^(#{1,6})\s+(.*?)\s*#*\s*$");
    let mut out = Vec::new();
    let mut in_fence = false;
    for (idx, line) in lines.iter().enumerate() {
        if line.py_lstrip().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(c) = head.captures(line) {
            out.push((idx, c[1].chars().count(), c[2].py_strip().to_string()));
        }
    }
    out
}

/// One table of the 004 family's `parse_tables`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// The 0-based line of the header in the text parsed.
    pub start: usize,
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// The 004 family's `parse_tables` (identical in every script that reads
/// the enumeration): runs of lines starting, after whitespace, with `|`,
/// outside fences (a line starting, after whitespace, with three
/// backticks toggles); the second line must contain `-` and match
/// `^\s*\|?[\s:|-]+\|?\s*$`.
pub fn parse_tables(text: &str) -> Vec<Table> {
    let lines = splitlines(text);
    let sep = rx(r"^\s*\|?[\s:|-]+\|?\s*$");
    let mut out = Vec::new();
    let (mut in_fence, mut i) = (false, 0);
    while i < lines.len() {
        let stripped = lines[i].py_lstrip();
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
        while j < lines.len() && lines[j].py_lstrip().starts_with('|') {
            j += 1;
        }
        let block = &lines[i..j];
        if block.len() >= 2 && block[1].contains('-') && sep.is_match(block[1]) {
            out.push(Table {
                start: i,
                header: split_row(block[0]),
                rows: block[2..].iter().map(|l| split_row(l)).collect(),
            });
        }
        i = j;
    }
    out
}

/// The 004 family's `split_row`: strip, drop one leading and one trailing
/// `|`, split on `|` not preceded by a backslash (`(?<!\\)\|`), strip each
/// cell and unescape `\|`.
pub fn split_row(line: &str) -> Vec<String> {
    let mut s = line.py_strip();
    s = s.strip_prefix('|').unwrap_or(s);
    s = s.strip_suffix('|').unwrap_or(s);
    let mut cells = Vec::new();
    let (mut cur, mut prev) = (String::new(), None);
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
        .map(|c| c.py_strip().replace("\\|", "|"))
        .collect()
}

#[cfg(test)]
mod tests;
