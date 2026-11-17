//! The closed rejection-reason enumeration of `wire.md`, read the way the
//! 004 ticket scripts read it.
//!
//! The scripts of epic 004 that judge invalid-form outcomes (004-16 to
//! 004-20, and later 004-11, 004-24 to 004-29) first parse the members of
//! the "Rejection reasons" enumeration from `doc/design/spec/wire.md`, then
//! each fails unless it found an exact number of members (13 today; a design
//! change will raise it). This module is that parse, once. The pin is **not**
//! here: each check keeps its own `const REJECTION_PIN: usize` next to its
//! own failure message and calls [`Enumeration::has_exactly`], so a change of
//! the pin is one obvious line per check.
//!
//! The parse, as in `checks/004-16.sh` lines 363-387 (and 004-17:350-373,
//! 004-18:345-368, 004-19:624-650, 004-20:675-702, which are the same code):
//!
//! 1. the text is read as Python's `open()` reads it (universal newlines)
//!    and split with `str.splitlines()`;
//! 2. headings are `^(#{1,6})\s+(.*?)\s*#*\s*$` outside fences, a fence
//!    being a line that starts, after whitespace, with three backticks;
//! 3. the section is the first heading **of any level** whose title
//!    matches `rejection[-\s]+reasons?` (`re.search`, any case), up to the
//!    next heading of the same or a higher level;
//! 4. its tables are runs of lines starting (after whitespace) with `|`,
//!    outside fences, whose second line is a separator
//!    (`^\s*\|?[\s:|-]+\|?\s*$` and containing `-`); cells are split on
//!    unescaped `|`;
//! 5. a member is a row of at least two cells whose first cell is a decimal
//!    number (`\d+`, any Unicode decimal digit, as Python's `re` and `int`
//!    read it) and whose second cell, stripped of `` ` ``, `*`, `_` and
//!    spaces, is `[A-Z][A-Za-z0-9]+`. A later row with the same name
//!    overwrites the earlier one's value (a Python dict) and keeps its
//!    place.
//!
//! "Whitespace" is Python's (`str.isspace`, `re`'s `\s`): Rust's
//! `White_Space` plus U+001C to U+001F.
//!
//! What the scripts do with the members afterwards (reason-token patterns,
//! negation look-backs, the Layers cell) differs per script and stays in
//! each port.

/// One member of the enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// The name, e.g. `BadMagic`.
    pub name: String,
    /// The value cell as Python's `int()` reads it, in ASCII decimal without
    /// leading zeros (Python's integers are unbounded, so it stays text).
    pub value: String,
    /// Every cell of the member's row, stripped (cell 3 is the Layers cell
    /// that 004-20 reads; the scripts read `row[3]` only when the row has at
    /// least four cells).
    pub cells: Vec<String>,
}

/// The enumeration as parsed from one `wire.md` text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Enumeration {
    /// A heading matching `rejection[-\s]+reasons?` exists.
    pub section_found: bool,
    /// The members in first-insertion order.
    pub members: Vec<Member>,
}

impl Enumeration {
    /// Parses the enumeration from the text of `wire.md`.
    pub fn parse(wire_md: &str) -> Enumeration {
        let text = wire_md.replace("\r\n", "\n").replace('\r', "\n");
        let lines = crate::doccheck::text::splitlines(&text);
        let heads = headings(&lines);
        let Some(k) = heads.iter().position(|h| is_rejection_title(&h.2)) else {
            return Enumeration::default();
        };
        let (start, level) = (heads[k].0 + 1, heads[k].1);
        let end = heads[k + 1..]
            .iter()
            .find(|h| h.1 <= level)
            .map_or(lines.len(), |h| h.0);
        let mut out = Enumeration {
            section_found: true,
            members: Vec::new(),
        };
        let body = lines[start.min(end)..end].join("\n");
        for rows in tables(&body) {
            for row in rows {
                out.consider(row);
            }
        }
        out
    }

    fn consider(&mut self, row: Vec<String>) {
        if row.len() < 2 {
            return;
        }
        let Some(value) = decimal(strip(&row[0])) else {
            return;
        };
        let name = strip(&row[1]).trim_matches(|c| "`*_ ".contains(c));
        let mut chars = name.chars();
        let first_upper = chars.next().is_some_and(|c| c.is_ascii_uppercase());
        let rest: Vec<char> = chars.collect();
        if !first_upper || rest.is_empty() || !rest.iter().all(char::is_ascii_alphanumeric) {
            return;
        }
        let member = Member {
            name: name.to_string(),
            value,
            cells: row,
        };
        match self.members.iter_mut().find(|m| m.name == member.name) {
            Some(m) => *m = member,
            None => self.members.push(member),
        }
    }

    /// The number of distinct member names (`len(MEMBERS)`).
    pub fn count(&self) -> usize {
        self.members.len()
    }

    /// True when the enumeration has exactly `pin` members: the scripts'
    /// `len(MEMBERS) != 13` failure is `!has_exactly(REJECTION_PIN)`.
    pub fn has_exactly(&self, pin: usize) -> bool {
        self.count() == pin
    }

    /// The member names, in first-insertion order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.members.iter().map(|m| m.name.as_str())
    }

    /// True when `name` is a member (case-sensitive, as `name in MEMBERS`).
    pub fn contains(&self, name: &str) -> bool {
        self.members.iter().any(|m| m.name == name)
    }

    /// The member named `name`.
    pub fn get(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.name == name)
    }
}

/// Python's `str.isspace` for one character.
fn is_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python's `str.strip()`.
fn strip(s: &str) -> &str {
    s.trim_matches(is_space)
}

/// `re.fullmatch(r"\d+", s)` then `int(s)`: the value in ASCII decimal
/// without leading zeros, for any Unicode decimal digits.
fn decimal(s: &str) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    let mut out = String::new();
    for c in s.chars() {
        let d = digit_value(c)?;
        if !(out.is_empty() && d == 0) {
            out.push(char::from(b'0' + d));
        }
    }
    if out.is_empty() {
        out.push('0');
    }
    Some(out)
}

/// The value of a Unicode decimal digit (general category Nd), as Python's
/// `int` reads it. Every Nd block is a run of ten consecutive code points
/// from 0 to 9, and adjacent runs are whole multiples of ten.
pub(crate) fn digit_value(c: char) -> Option<u8> {
    if c.is_ascii_digit() {
        return Some(c as u8 - b'0');
    }
    let nd = crate::doccheck::text::rx(r"^\p{Nd}$");
    if c.is_ascii() || !nd.is_match(c.encode_utf8(&mut [0; 4])) {
        return None;
    }
    let mut start = c as u32;
    while let Some(prev) = char::from_u32(start - 1) {
        if !nd.is_match(prev.encode_utf8(&mut [0; 4])) {
            break;
        }
        start -= 1;
    }
    Some(((c as u32 - start) % 10) as u8)
}

/// `heading_index`: (line, level, title) of each heading outside fences.
fn headings(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim_start_matches(is_space).starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some((level, title)) = heading(line) {
            out.push((idx, level, title));
        }
    }
    out
}

/// `re.match(r"^(#{1,6})\s+(.*?)\s*#*\s*$", line)`: the level and the
/// stripped title.
fn heading(line: &str) -> Option<(usize, String)> {
    let level = line.chars().take_while(|&c| c == '#').count().min(6);
    if level == 0 {
        return None;
    }
    let rest = &line[level..];
    let body = rest.trim_start_matches(is_space);
    if body.len() == rest.len() {
        return None;
    }
    // (.*?)\s*#*\s*$ : the lazy title ends where only a run of spaces, then
    // hashes, then spaces remain; the first such point wins.
    let t = body.trim_end_matches(is_space);
    let t = t.trim_end_matches('#');
    Some((level, strip(t).to_string()))
}

/// `re.search(r"rejection[-\s]+reasons?", title, re.I)`. Under `re.I`
/// Python also matches `i` with the dotless `ı` and the dotted `İ`, which
/// Rust's case folding does not: they are mapped to `i` first (the result is
/// only a boolean, so the length change does not matter).
fn is_rejection_title(title: &str) -> bool {
    let folded = title.replace(['\u{131}', '\u{130}'], "i");
    crate::doccheck::text::rx(r"(?i)rejection[-\s\x1c-\x1f]+reasons?").is_match(&folded)
}

/// The data rows of every table of `text` (the 004 family's
/// `parse_tables`).
fn tables(text: &str) -> Vec<Vec<Vec<String>>> {
    let lines = crate::doccheck::text::splitlines(text);
    let sep = crate::doccheck::text::rx(r"^[\s\x1c-\x1f]*\|?[\s\x1c-\x1f:|-]+\|?[\s\x1c-\x1f]*$");
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut i = 0;
    while i < lines.len() {
        let stripped = lines[i].trim_start_matches(is_space);
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
        while j < lines.len() && lines[j].trim_start_matches(is_space).starts_with('|') {
            j += 1;
        }
        let block = &lines[i..j];
        if block.len() >= 2 && sep.is_match(block[1]) && block[1].contains('-') {
            out.push(block[2..].iter().map(|l| split_row(l)).collect());
        }
        i = j;
    }
    out
}

/// `split_row`: strip, drop one leading and one trailing `|`, split on `|`
/// not preceded by a backslash, strip each cell and unescape `\|`.
fn split_row(line: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const WIRE: &str = "# Wire\n\n## Rejection reasons\n\nText.\n\n| Value | Name | Meaning | Layers |\n|---|---|---|---|\n| 1 | `BadMagic` | x | control |\n| 02 | **TooShort** | y | media |\n| x | NotANumber | z | |\n| 3 | lower | z | |\n| 4 | B | z | |\n\n### Notes\n\n| 5 | InSubsection |\n|---|---|\n| 6 | Deeper |\n\n## Next\n\n| 7 | Outside |\n|---|---|\n| 8 | Outside |\n";

    #[test]
    fn parses_the_members_the_way_the_scripts_do() {
        let e = Enumeration::parse(WIRE);
        assert!(e.section_found);
        let names: Vec<&str> = e.names().collect();
        assert_eq!(names, ["BadMagic", "TooShort", "Deeper"]);
        assert_eq!(e.get("TooShort").map(|m| m.value.as_str()), Some("2"));
        assert_eq!(
            e.get("BadMagic").map(|m| m.cells[3].as_str()),
            Some("control")
        );
        assert!(e.has_exactly(3));
        assert!(!e.has_exactly(13));
        assert!(e.contains("Deeper") && !e.contains("Outside"));
    }

    #[test]
    fn python_semantics_digits_whitespace_and_newlines() {
        // Arabic-indic digits are decimal for Python's \d and int().
        let t = "## Rejection reasons\n| V | N |\n|---|---|\n| \u{663} | Arabic |\n| 1\u{1f} | Unit |\n";
        let e = Enumeration::parse(t);
        assert_eq!(e.get("Arabic").map(|m| m.value.as_str()), Some("3"));
        // U+001F is whitespace for str.strip().
        assert!(e.contains("Unit"));
        // U+2028 starts a line for str.splitlines(): the row is two lines.
        let t = "## Rejection reasons\n| V | N |\n|---|---|\n| 1 | A1\u{2028}| 2 | Bb |\n";
        let names: Vec<String> = Enumeration::parse(t).names().map(String::from).collect();
        assert_eq!(names, ["A1", "Bb"]);
        // A carriage return alone ends a line (universal newlines).
        let t = "## Rejection reasons\r| V | N |\r|---|---|\r| 1 | Cr |\r";
        assert!(Enumeration::parse(t).contains("Cr"));
    }

    #[test]
    fn the_section_is_the_first_matching_heading_of_any_level() {
        let t = "### Rejection-reason codes\n| V | N |\n|--|--|\n| 1 | First |\n## Rejection reasons\n| V | N |\n|--|--|\n| 2 | Second |\n";
        let names: Vec<String> = Enumeration::parse(t).names().map(String::from).collect();
        assert_eq!(names, ["First"]);
        assert!(!Enumeration::parse("## Rejections\n").section_found);
        // Python's re.I matches i with the dotless and dotted capitals.
        assert!(Enumeration::parse("## Reject\u{131}on reasons\n").section_found);
        assert!(Enumeration::parse("## REJECT\u{130}ON\u{1f}REASONS\n").section_found);
        // A table without a separator row is not a table.
        let t = "## Rejection reasons\n| V | N |\n| 1 | Kept |\n";
        assert_eq!(Enumeration::parse(t).count(), 0);
        // Fenced headings do not count.
        let t = "```\n## Rejection reasons\n```\n## Other\n";
        assert!(!Enumeration::parse(t).section_found);
    }

    #[test]
    fn digit_values() {
        assert_eq!(digit_value('7'), Some(7));
        assert_eq!(digit_value('\u{669}'), Some(9));
        assert_eq!(digit_value('\u{1d7d9}'), Some(1));
        assert_eq!(digit_value('x'), None);
        assert_eq!(digit_value('\u{b2}'), None);
    }
}
