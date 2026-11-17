//! The bash semantics of the epic-002 check scripts (`checks/002-*.sh`).
//!
//! The rest of the library reproduces Python. The 002 scripts are bash:
//! GNU grep (ERE, BRE, fixed strings, `-i`, `-o`, `-c`, `-n`), GNU sed,
//! awk (mawk 1.3.4 here), GNU tr and cut, `sort -u`, `wc -l` and bash's own
//! parameter expansion, run in the locale of the environment (no script
//! sets `LC_ALL` or `LANG`; this machine runs them in `fr_FR.UTF-8`). This
//! module gives those semantics as named helpers, for every 002 port:
//!
//! * [`ere`]: [`Grep`] (`grep -E`/`grep`/`grep -F`, each with `-i`; `-q`,
//!   `-c`, `-o` over one line or a `printf '%s\n'` text) and [`Sed`]
//!   (`sed -E 's/…/…/g'`), translated to the library's regex engine with
//!   the locale's classes (`[[:alnum:]]` holds `µ Δ Σ α π τ` but not `²`),
//!   GNU's `\b \< \> \w \s`, POSIX brackets (a backslash inside is
//!   literal) and `-i` spelled out per character;
//! * [`shell`]: [`Lines`] (a file as grep and sed number it: a final line
//!   without a newline is a line, a CR stays in its line; `sed -n` ranges,
//!   `$(…)` stripping trailing newlines, `read` losing an unterminated last
//!   line, the scripts' `heading_no` and `line_range`), here-document
//!   lines, `${s#…}`/`${s%…}` with `[[:space:]]`, `${s%%[!set]*}`, `tr`
//!   (byte-based: only ASCII changes case), `cut -c` (bytes), `sort -u |
//!   grep -c .`, the alnum word count.
//!
//! **Proof.** `tools/gen_family002.py` runs the real tools on a corpus (the
//! lines of REPORT-0b.md plus case, CR, tab and spacing variants, every
//! pattern keyword between 15 context characters, and hand-written edge
//! lines) and writes what they answered to `data/family002/` with the tool
//! versions and the locale. The unit tests in `proof` compare every helper
//! with those tables without running a tool: the per-character classes
//! (grep with and without `-i`, bash, tr, cut), every proven pattern on
//! every input (`-q`, `-o`, `sed`), every bash snippet on every input, and
//! the file-level probes on fixtures (final line without newline, CRLF,
//! trailing blank lines, a heading inside a fence). A port may only use a
//! pattern the tables prove: its test asserts [`unproven_patterns`] is
//! empty after running the check and its denials.
//!
//! **The repertoire.** The helpers are defined on the repertoire
//! (`data/repertoire.tsv`), which [`crate::doccheck::CheckSpec::evaluate`]
//! enforces. A carriage return IS in the repertoire, and unlike the Python
//! ports these scripts see it (no universal newlines): it is a
//! `[[:space:]]` character inside its line. A file that is not UTF-8 is
//! read by the tools but not by [`crate::doccheck::Docs::read`]:
//! [`read`] reports it under `REPERTOIRE`.
//!
//! What is not here: package-private rules (a script's own keyword lists,
//! its messages, its criteria). Those stay in the ports.

pub mod ere;
#[cfg(test)]
mod proof;
pub mod shell;
pub mod translate;

pub use ere::{Grep, Sed, unproven_patterns};
pub use shell::{
    Lines, alnum_word_count, command_substitution, cut_bytes, distinct_count, heredoc_lines,
    leading_run, lstrip_space, rstrip_space, strip_prefix_once, strip_suffix_once, tr_delete_space,
    tr_lower,
};
pub use translate::{Engine, Syntax, is_space, is_word, posix_class};

use super::{Docs, Report};
use std::sync::Arc;

/// A file as `[ -f "$f" ]` and the tools see it.
#[derive(Debug, Clone)]
pub enum Read {
    /// The file does not exist.
    Missing,
    /// Its text.
    Text(Arc<str>),
}

/// Reads `path` for a bash script: [`Read::Missing`] when it does not
/// exist; when it exists but is not UTF-8 (the tools read it, the port
/// cannot), a `REPERTOIRE` failure is recorded on `report` and the text is
/// empty (outside the repertoire the port is red by rule).
pub fn read(docs: &Docs, path: &str, report: &mut Report) -> Read {
    match docs.read(path) {
        Some(t) => Read::Text(t),
        None if docs.is_file(path) => {
            report.fail_at(
                crate::doccheck::repertoire::CRITERION,
                path,
                1,
                format!(
                    "{path} is not UTF-8: outside the repertoire of the ported checks, red by rule"
                ),
            );
            Read::Text(Arc::from(""))
        }
        None => Read::Missing,
    }
}
