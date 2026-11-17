//! Denials: targeted negative fixtures as data.
//!
//! A denial is the real documents with one edit applied in memory and the
//! failures the check must report on them. Each port keeps ONE list
//! (`CheckSpec::denials`, usually in `c<id>/denials.rs`). The list is
//! consumed twice: by the permanent tests (one `#[test]` per denial, through
//! [`crate::denial_tests!`]) and by the harness (`doccheck-equiv <id>
//! --denials`), which runs every denial against the old script as well
//! while it exists. A denial that turns the script red and leaves the port
//! green is "port weaker", the failure the random corpus misses.

use super::Docs;
use super::report::Report;

/// How a denial changes its file.
#[derive(Debug, Clone, Copy)]
pub enum DenialEdit {
    /// Replace the file's text by `f(text)`.
    Text(fn(&str) -> String),
    /// Delete the file.
    Delete,
}

/// One denial.
#[derive(Debug, Clone, Copy)]
pub struct Denial {
    /// The test name (`ac1_denial_a_missing_rule_row`).
    pub name: &'static str,
    /// The repository-relative file it edits.
    pub file: &'static str,
    pub edit: DenialEdit,
    /// What the port must report: (criterion, a substring of the message).
    /// The script must at least fail the same criteria.
    pub expect: &'static [(&'static str, &'static str)],
}

impl Denial {
    /// `docs` with the denial applied; `Err` when the file to edit is
    /// missing (a stale denial must fail, not pass vacuously).
    pub fn apply(&self, docs: &Docs) -> Result<Docs, String> {
        match self.edit {
            DenialEdit::Delete => Ok(docs.without(self.file)),
            DenialEdit::Text(f) => {
                let text = docs
                    .read(self.file)
                    .ok_or_else(|| format!("denial {}: {} does not exist", self.name, self.file))?;
                Ok(docs.with(self.file, f(&text)))
            }
        }
    }

    /// The edited text of the file (`None` for a deletion).
    pub fn edited(&self, docs: &Docs) -> Result<Option<String>, String> {
        match self.edit {
            DenialEdit::Delete => Ok(None),
            DenialEdit::Text(f) => {
                let text = docs
                    .read(self.file)
                    .ok_or_else(|| format!("denial {}: {} does not exist", self.name, self.file))?;
                Ok(Some(f(&text)))
            }
        }
    }

    /// The expectations the report does not meet, as messages.
    pub fn unmet(&self, report: &Report) -> Vec<String> {
        self.expect
            .iter()
            .filter(|(c, needle)| !report.of(c).iter().any(|f| f.message.contains(needle)))
            .map(|(c, needle)| format!("expected a failure of {c} containing {needle:?}"))
            .collect()
    }

    /// The criteria the denial expects.
    pub fn criteria(&self) -> std::collections::BTreeSet<String> {
        self.expect.iter().map(|(c, _)| c.to_string()).collect()
    }
}
