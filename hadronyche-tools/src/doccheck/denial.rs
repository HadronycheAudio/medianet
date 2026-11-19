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

/// The pseudo-criterion of an expected crash.
pub const CRASH: &str = "CRASH";

/// How a denial changes its file.
#[derive(Debug, Clone, Copy)]
pub enum DenialEdit {
    /// Replace the file's text by `f(text)`.
    Text(fn(&str) -> String),
    /// Delete the file.
    Delete,
    /// Edit several documents together (a multi-file denial): each
    /// `(path, edit)` applies in order; [`Denial::file`] names the first,
    /// for messages only. Nested `Files` are not allowed.
    Files(&'static [(&'static str, FileEdit)]),
}

/// One file's edit inside [`DenialEdit::Files`].
#[derive(Debug, Clone, Copy)]
pub enum FileEdit {
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
    /// What the port must report, exactly: its failed criteria are the
    /// criteria named here (each pair is a criterion and a substring one of
    /// its messages must contain; `""` matches any). Empty: the port passes.
    /// The pseudo-criterion `CRASH` means the port models the script's
    /// crash ([`crate::doccheck::Report::crash`]). The criteria are the old
    /// script's verdict on the same documents, or `REPERTOIRE` when the edit
    /// brings in a character outside the repertoire.
    pub expect: &'static [(&'static str, &'static str)],
}

impl Denial {
    /// `docs` with the denial applied; `Err` when a file to edit is
    /// missing (a stale denial must fail, not pass vacuously).
    pub fn apply(&self, docs: &Docs) -> Result<Docs, String> {
        let mut out = docs.clone();
        for (path, text) in self.edits(docs)? {
            out = match text {
                Some(t) => out.with(&path, t),
                None => out.without(&path),
            };
        }
        Ok(out)
    }

    /// Every file the denial changes, with its new text (`None`: deleted),
    /// in order. A later edit of the same file sees the earlier one.
    pub fn edits(&self, docs: &Docs) -> Result<Vec<(String, Option<String>)>, String> {
        let single = [(self.file, self.file_edit())];
        let list: &[(&str, FileEdit)] = match self.edit {
            DenialEdit::Files(list) => list,
            _ => &single,
        };
        let mut cur = docs.clone();
        let mut out: Vec<(String, Option<String>)> = Vec::new();
        for (path, edit) in list {
            let text = match edit {
                FileEdit::Delete => None,
                FileEdit::Text(f) => {
                    let t = cur
                        .read(path)
                        .ok_or_else(|| format!("denial {}: {path} does not exist", self.name))?;
                    Some(f(&t))
                }
            };
            cur = match &text {
                Some(t) => cur.with(path, t.clone()),
                None => cur.without(path),
            };
            out.retain(|(p, _)| p != path);
            out.push((path.to_string(), text));
        }
        Ok(out)
    }

    fn file_edit(&self) -> FileEdit {
        match self.edit {
            DenialEdit::Text(f) => FileEdit::Text(f),
            _ => FileEdit::Delete,
        }
    }

    /// The edited text of [`Denial::file`] (`None` for a deletion); for a
    /// multi-file denial use [`Denial::edits`].
    pub fn edited(&self, docs: &Docs) -> Result<Option<String>, String> {
        let edits = self.edits(docs)?;
        Ok(edits
            .into_iter()
            .find(|(p, _)| p == self.file)
            .and_then(|(_, t)| t))
    }

    /// The expectations the report does not meet, as messages.
    pub fn unmet(&self, report: &Report) -> Vec<String> {
        let mut out = Vec::new();
        let want = self.criteria();
        let got = report.failed_criteria();
        if got != want {
            out.push(format!(
                "expected the failed criteria {want:?}, got {got:?}"
            ));
        }
        let want_crash = self.expect.iter().any(|(c, _)| *c == CRASH);
        if report.crash.is_some() != want_crash {
            out.push(format!(
                "expected a crash: {want_crash}, got {:?}",
                report.crash
            ));
        }
        for (c, needle) in self
            .expect
            .iter()
            .filter(|(c, n)| *c != CRASH && !n.is_empty())
        {
            if !report.of(c).iter().any(|f| f.message.contains(needle)) {
                out.push(format!("expected a failure of {c} containing {needle:?}"));
            }
        }
        out
    }

    /// The criteria the denial expects (without `CRASH`).
    pub fn criteria(&self) -> std::collections::BTreeSet<String> {
        self.expect
            .iter()
            .filter(|(c, _)| *c != CRASH)
            .map(|(c, _)| c.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOTH: Denial = Denial {
        name: "two_documents_together",
        file: "doc/a.md",
        edit: DenialEdit::Files(&[
            ("doc/a.md", FileEdit::Text(|t| t.replace("one", "uno"))),
            ("doc/b.md", FileEdit::Delete),
            ("doc/a.md", FileEdit::Text(|t| format!("{t}+"))),
        ]),
        expect: &[],
    };

    #[test]
    fn a_multi_file_denial_edits_every_document_in_order() {
        let docs = crate::doccheck::testing::repo()
            .with("doc/a.md", "one")
            .with("doc/b.md", "two");
        let edits = BOTH.edits(&docs).expect("both exist");
        assert_eq!(
            edits,
            vec![
                ("doc/b.md".to_string(), None),
                ("doc/a.md".to_string(), Some("uno+".to_string()))
            ]
        );
        let after = BOTH.apply(&docs).expect("applies");
        assert_eq!(after.read("doc/a.md").as_deref(), Some("uno+"));
        assert!(after.read("doc/b.md").is_none());
        assert_eq!(BOTH.edited(&docs).expect("ok"), Some("uno+".to_string()));
        assert!(
            BOTH.apply(&docs.without("doc/a.md")).is_err(),
            "a stale denial fails"
        );
    }
}
