//! Readers for tickets and the constants table.
//!
//! A ticket is `doc/design/tickets/<epic>/<id>-<Title>.md`: a title, a
//! `**Depends on:**` line, numbered acceptance criteria and a definition of
//! done checklist. The constants table is the `| C-NN | name | value |` table
//! of `doc/design/02-constants.md`.

use std::collections::{BTreeMap, BTreeSet};

use super::docs::Docs;
use super::markdown::Markdown;
use super::text::rx;

/// Where the constants table lives.
pub const CONSTANTS_MD: &str = "doc/design/02-constants.md";

/// A parsed ticket.
#[derive(Debug, Clone)]
pub struct Ticket {
    /// Repository-relative path of the ticket file.
    pub path: String,
    pub text: String,
}

impl Ticket {
    /// The ticket `id` (`004-22`) from `docs`: the first file of its epic
    /// directory whose name starts with `<id>-` (the scripts'
    /// `ls doc/design/tickets/004/004-22-*.md | head -n 1`).
    pub fn load(docs: &Docs, id: &str) -> Option<Ticket> {
        let epic = id.split('-').next()?;
        let dir = format!("doc/design/tickets/{epic}");
        let path = docs
            .glob(&dir, &format!("{id}-"))
            .into_iter()
            .find(|p| p.ends_with(".md"))?;
        let text = docs.read(&path)?.to_string();
        Some(Ticket { path, text })
    }

    /// The rest of the first line that contains `label` (`Constants
    /// traced:`), after the label.
    pub fn sentence_after(&self, label: &str) -> Option<String> {
        let at = self.text.find(label)?;
        let rest = &self.text[at + label.len()..];
        Some(rest.split('\n').next().unwrap_or("").to_string())
    }

    /// The tickets named on the `**Depends on:**` line.
    pub fn depends_on(&self) -> Vec<String> {
        let Some(line) = self.text.lines().find(|l| l.contains("Depends on:")) else {
            return Vec::new();
        };
        let after = &line[line.find("Depends on:").unwrap_or(0)..];
        let after = after.split('·').next().unwrap_or("");
        rx(r"\b\d{3}[a-z]?-\d{2}\b")
            .find_iter(after)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn numbered_items(&self, heading: &str) -> Vec<(usize, String)> {
        let md = Markdown::parse(&self.text);
        let Some(h) = md.find(heading).into_iter().next() else {
            return Vec::new();
        };
        let span = md.section(h);
        let re = rx(r"^\s*(?:(\d+)\.|-\s*\[[ xX]\])\s+(.*)$");
        let mut out = Vec::new();
        for line in &md.lines[span.lo + 1..span.hi] {
            if let Some(c) = re.captures(line) {
                let n = c
                    .get(1)
                    .map_or(out.len() + 1, |m| m.as_str().parse().unwrap_or(0));
                out.push((n, c[2].trim().to_string()));
            }
        }
        out
    }

    /// The acceptance criteria, numbered as written.
    pub fn acceptance_criteria(&self) -> Vec<(usize, String)> {
        self.numbered_items(r"(?i)^acceptance criteria")
    }

    /// The definition-of-done checklist items, numbered from 1.
    pub fn definition_of_done(&self) -> Vec<(usize, String)> {
        self.numbered_items(r"(?i)^definition of done")
    }
}

/// The constant ids of `text`, ranges expanded: `C-01 to C-13`, `C-103–C-107`
/// (004-22's `expand_ids`; a reversed range adds nothing).
pub fn expand_constant_ids(text: &str) -> BTreeSet<u32> {
    let mut ids = BTreeSet::new();
    for c in rx(r"C-(\d{1,3})(?:\s*(?:to|–)\s*C-(\d{1,3}))?").captures_iter(text) {
        let a: u32 = c[1].parse().unwrap_or(0);
        let b: u32 = c.get(2).map_or(a, |m| m.as_str().parse().unwrap_or(0));
        if b >= a {
            ids.extend(a..=b);
        }
    }
    ids
}

/// A constant id as the design set writes it: `C-01`, `C-137`.
pub fn cid(n: u32) -> String {
    if n < 100 {
        format!("C-{n:02}")
    } else {
        format!("C-{n}")
    }
}

/// One row of the constants table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constant {
    /// 1-based line in the constants file.
    pub line: usize,
    pub name: String,
    pub value: String,
}

/// The constants table by id (`C-17`).
#[derive(Debug, Clone, Default)]
pub struct Constants {
    pub rows: BTreeMap<String, Constant>,
}

impl Constants {
    /// Parses the rows that start with `| C-NN |`.
    pub fn parse(text: &str) -> Constants {
        let mut rows = BTreeMap::new();
        for (i, line) in text.split('\n').enumerate() {
            let Some(c) = rx(r"^\| (C-\d+) \|").captures(line) else {
                continue;
            };
            let cells: Vec<&str> = line.split('|').collect();
            let cell = |k: usize| cells.get(k).map_or("", |c| c.trim()).to_string();
            rows.entry(c[1].to_string()).or_insert(Constant {
                line: i + 1,
                name: cell(2),
                value: cell(3),
            });
        }
        Constants { rows }
    }

    /// The constants table of `docs`.
    pub fn load(docs: &Docs) -> Constants {
        docs.read(CONSTANTS_MD)
            .map_or_else(Constants::default, |t| Constants::parse(&t))
    }

    /// The value cell of `id`, or `""`.
    pub fn value(&self, id: &str) -> &str {
        self.rows.get(id).map_or("", |c| c.value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_expand_ranges_with_to_and_en_dash() {
        let ids = expand_constant_ids("C-01 to C-03, C-17, C-103–C-105, C-9 to C-2");
        assert_eq!(
            ids.into_iter().collect::<Vec<_>>(),
            vec![1, 2, 3, 17, 103, 104, 105]
        );
        assert_eq!(cid(7), "C-07");
        assert_eq!(cid(137), "C-137");
    }

    #[test]
    fn the_real_ticket_reads() {
        let t = Ticket::load(&Docs::repo(), "004-22").expect("ticket 004-22");
        assert_eq!(t.depends_on(), vec!["004-07", "004-14", "004-15", "004-21"]);
        let acs = t.acceptance_criteria();
        assert_eq!(acs.len(), 4);
        assert!(acs[0].1.contains("22 numbered rules"));
        assert_eq!(t.definition_of_done().len(), 3);
        let traced = t.sentence_after("Constants traced:").unwrap();
        assert!(expand_constant_ids(&traced).contains(&137));
    }

    #[test]
    fn constants_by_id() {
        let c = Constants::load(&Docs::repo());
        assert!(c.value("C-132").contains("[standard]"));
        assert!(c.rows["C-132"].line > 1);
        assert_eq!(c.value("C-999"), "");
        let one = Constants::parse("| C-06 | Fallback | 1 280 bytes |\n");
        assert_eq!(one.value("C-06"), "1 280 bytes");
    }
}
