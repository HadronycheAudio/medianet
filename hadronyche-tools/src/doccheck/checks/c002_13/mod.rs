//! Port of `checks/002-13.sh`: the close-out of epic 002 — the report
//! audit (checklist, verdicts, open sub-questions), the evidence-appendix
//! entries, the P-02 row, the stale sync-rate and loss-accounting guards
//! over the design set, the regime-stamp line, the frozen-tree re-check,
//! the hand-offs and the open items (ticket 002-13).
//!
//! The acceptance-criterion map is on the tests in `tests/doc_002_13.rs`;
//! the denial list is `denials.rs`. The script was bash around embedded
//! Python; its variants, reproduced as written (not through the shared
//! rules, which differ):
//!
//! * every file read with `open(encoding='utf-8').read().splitlines()`
//!   (universal newlines, then every Python line boundary); a file that is
//!   not UTF-8 is an uncaught exception, modelled as a crash;
//! * headings: `^(#{1,6})\s+(.*\S)\s*$` outside ```` ``` ```` fences
//!   (a fence line is any line whose `lstrip()` starts with three
//!   backticks, and is itself masked); `heading_index` is the first heading
//!   line `re.search`ed by a pattern with `re.I`; `section_end(i, level)` is
//!   the next heading of level `<= level` (the AC 2 and AC 3 sections end at
//!   the next level-2 heading, the hand-offs and "Documents changed" at
//!   their own level, the 002-10 subsection at level 3);
//! * `blocks` (blank lines and bullet starts separate) and `para_blocks`
//!   (blank lines only); `cells` is `strip().strip('|').split('|')`, each
//!   cell stripped (no escaped pipes); a separator row is
//!   `^\s*\|?\s*:?-{3,}`; a table row is any line whose `lstrip()` starts
//!   with `|`;
//! * the five-tag set `[code] [measured] [kernel-doc] [memory]
//!   [decision]`, case-sensitive;
//! * every pattern copied as written, look-arounds included
//!   (`(?<=\.)` in the clause split, `(?<![\w.,/-])` in the stale-rate
//!   pattern, `(?<![\d.])` and `(?!\d)` in the P-02 number test), on
//!   Python's classes (`text::rx`).
//!
//! FAIL lines are the script's colon form (`FAIL: <message>`); each failure
//! is named by [`colon_criterion`] of the script's exact message, as the
//! harness names the script's.
//!
//! AC 7 reads the proof of concept: the recomputed `poc-untouched`
//! tag-tree fingerprint and the 27-file frozen-data manifest. Those reads
//! are in `scripts/repo-state.d/002-13.sh`, the script's own code. The
//! port keeps the report's side: the recorded fingerprint must equal the
//! frozen tag's value [`POC_TAG_TREE_FINGERPRINT`] (the value the
//! repo-state file recomputes from git and compares with the report), and
//! the re-check paragraph must quote it or the frozen-tree fingerprint.
//! Header and code disagree (followed: the code): the header says checks
//! are made "never by comparison with any git ref", while AC 7 reads the
//! proof of concept's tag with `git ls-tree`, `cat-file` and `rev-parse`.
//!
//! The script's override variables (`REPORT_0B`, `CONSTANTS_MD`,
//! `EVIDENCE_MD`, `DESIGN_ROOT`) served its fixture harness; the port reads
//! the default paths, as the script did when they were not set.

mod ac1_2;
mod ac3_4;
mod ac5;
mod close;
mod denials;
mod guards;

use super::{CheckSpec, KnownDifference, Target};
use crate::doccheck::fail_form::colon_criterion;
use crate::doccheck::policy::TomlReader;
use crate::doccheck::pycompat::{PyStr, splitlines, universal_newlines};
use crate::doccheck::text::{has, has_i, rx, starts};
use crate::doccheck::{Docs, Report};

/// The report.
pub const REPORT: &str = "doc/design/reports/archaeology/REPORT-0b.md";
/// The constants table.
pub const CONSTANTS: &str = "doc/design/02-constants.md";
/// The evidence appendix.
pub const EVIDENCE: &str = "doc/design/06-evidence-appendix.md";
/// The root the stale-statement and loss guards walk.
pub const DESIGN_DIR: &str = "doc/design";

/// The `poc-untouched` tag-tree fingerprint of the frozen proof of concept,
/// as the script's `tag_fingerprint` computes it from git (sha256 over
/// `path:<path> NUL <blob> NUL` of the tracked files under `src/`,
/// `tools/`, `experiments/` plus `Cargo.toml`, `Cargo.lock`, `README.md`).
/// The proof of concept is frozen, so this is a constant;
/// `scripts/repo-state.d/002-13.sh` recomputes it from the tag and fails
/// when it differs from this value or from the report's.
pub const POC_TAG_TREE_FINGERPRINT: &str =
    "ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "002-13",
    run,
    script_env: &[
        ("DOCCHECK_FAIL_FORM", "colon"),
        // AC 7's manifest is read at a fixed path beside the script; the
        // proof of concept itself is read in place, without optional locks
        ("DOCCHECK_EXPORT", "checks/002-01-frozen-data.sha256"),
        ("GIT_OPTIONAL_LOCKS", "0"),
    ],
    targets: &[
        Target {
            file: REPORT,
            heading: None,
        },
        Target {
            file: REPORT,
            heading: Some("^Close-out checklist"),
        },
        Target {
            file: REPORT,
            heading: Some("^Open sub-questions$"),
        },
        Target {
            file: REPORT,
            heading: Some("^Close-out \\(002-13\\)$"),
        },
        Target {
            file: REPORT,
            heading: Some("^002-10 "),
        },
        Target {
            file: EVIDENCE,
            heading: None,
        },
        Target {
            file: CONSTANTS,
            heading: None,
        },
        Target {
            file: "doc/design/adr/24-Numbers-governance.md",
            heading: None,
        },
        Target {
            file: "doc/design/implementation/002-POC-archaeology.md",
            heading: None,
        },
        Target {
            file: "doc/design/05-risks-and-open.md",
            heading: None,
        },
    ],
    keywords: &[
        "answered",
        "superseded",
        "corrupt",
        "owner",
        "falsifier",
        "verdict",
        "fixed",
    ],
    known_differences: KNOWN,
    denials: denials::DENIALS,
    // the script reads no TOML; the field is required
    toml: TomlReader::Subset004_18,
};

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[];

/// `TAG` of the script: the five evidence tags, case-sensitive.
pub(crate) const TAG: &str = r"\[(code|measured|kernel-doc|memory|decision)\]";

/// A bullet start (`blocks` and the "Documents changed" entries).
pub(crate) const BULLET: &str = r"^\s*([-*]|\d+\.)\s";

/// `bad(msg)`: a failure named as the harness names the script's line.
pub(crate) fn bad(report: &mut Report, msg: String) {
    report.fail(&colon_criterion(&msg), msg);
}

/// `s[:n]` (code points).
pub(crate) fn cut(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// `' '.join(s.split())`.
pub(crate) fn flat(s: &str) -> String {
    s.py_split().collect::<Vec<_>>().join(" ")
}

/// A file as `read_lines` gave it: `None` when the script would crash on it
/// (it exists but is not UTF-8) or it does not exist.
pub(crate) fn read_lines(docs: &Docs, path: &str) -> Option<Vec<String>> {
    let text = docs.read(path)?;
    Some(
        splitlines(&universal_newlines(&text))
            .into_iter()
            .map(str::to_string)
            .collect(),
    )
}

/// `cells(row)`.
pub(crate) fn cells(row: &str) -> Vec<String> {
    row.py_strip()
        .trim_matches('|')
        .split('|')
        .map(|c| c.py_strip().to_string())
        .collect()
}

/// `is_table_row(l)`.
pub(crate) fn is_table_row(l: &str) -> bool {
    l.py_lstrip().starts_with('|')
}

/// `is_sep(l)`.
pub(crate) fn is_sep(l: &str) -> bool {
    starts(r"\s*\|?\s*:?-{3,}", l)
}

/// `blocks(lines)`: blank lines and bullet starts separate blocks.
pub(crate) fn blocks(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for l in lines {
        if l.py_strip().is_empty() {
            if !cur.is_empty() {
                out.push(cur.join("\n"));
            }
            cur.clear();
            continue;
        }
        if has(BULLET, l) && !cur.is_empty() {
            out.push(cur.join("\n"));
            cur.clear();
        }
        cur.push(l);
    }
    if !cur.is_empty() {
        out.push(cur.join("\n"));
    }
    out
}

/// `para_blocks(lines)`: blank lines separate blocks.
pub(crate) fn para_blocks(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for l in lines {
        if l.py_strip().is_empty() {
            if !cur.is_empty() {
                out.push(cur.join("\n"));
            }
            cur.clear();
        } else {
            cur.push(l);
        }
    }
    if !cur.is_empty() {
        out.push(cur.join("\n"));
    }
    out
}

/// The report's lines with the script's heading levels.
pub(crate) struct Rep {
    pub lines: Vec<String>,
    /// `heading_level(i)`: 0 for a non-heading or fenced line.
    pub level: Vec<usize>,
    /// `HEAD.match(rep[i]).group(2)` of each heading line.
    pub title: Vec<String>,
}

impl Rep {
    fn new(lines: Vec<String>) -> Rep {
        let head = rx(r"^(#{1,6})\s+(.*\S)\s*$");
        let mut inside = false;
        let mut level = Vec::with_capacity(lines.len());
        let mut title = Vec::with_capacity(lines.len());
        for l in &lines {
            // fenced_mask: a fence line is masked and toggles
            let masked = if l.py_lstrip().starts_with("```") {
                inside = !inside;
                true
            } else {
                inside
            };
            match head.captures(l).filter(|_| !masked) {
                Some(c) => {
                    level.push(c[1].chars().count());
                    title.push(c[2].to_string());
                }
                None => {
                    level.push(0);
                    title.push(String::new());
                }
            }
        }
        Rep {
            lines,
            level,
            title,
        }
    }

    /// `heading_index(pattern)` (`re.I`), `None` for -1.
    pub fn heading_index(&self, pattern: &str) -> Option<usize> {
        (0..self.lines.len()).find(|&i| self.level[i] > 0 && has_i(pattern, &self.lines[i]))
    }

    /// `section_end(i, level)`.
    pub fn section_end(&self, i: usize, level: usize) -> usize {
        (i + 1..self.lines.len())
            .find(|&j| self.level[j] > 0 && self.level[j] <= level)
            .unwrap_or(self.lines.len())
    }
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    if !docs.is_file(REPORT) {
        bad(&mut report, format!("report missing: {REPORT}"));
        return report;
    }
    for f in [CONSTANTS, EVIDENCE] {
        if !docs.is_file(f) {
            bad(&mut report, format!("missing file: {f}"));
            return report;
        }
    }
    // DESIGN_ROOT holds the report, so it exists whenever the report does
    // (the script's "design root missing" cannot fire at the default paths)
    let (Some(rep), Some(ev), Some(consts)) = (
        read_lines(docs, REPORT),
        read_lines(docs, EVIDENCE),
        read_lines(docs, CONSTANTS),
    ) else {
        report.crash("UnicodeDecodeError: a file the script opens is not UTF-8");
        return report;
    };
    let rep = Rep::new(rep);
    ac1_2::ac1(&rep, &mut report);
    ac1_2::ac2(&rep, &mut report);
    ac3_4::ac3(&rep, &mut report);
    ac3_4::ac4(&ev, &mut report);
    ac5::p02(&rep, &consts, &mut report);
    if !guards::stale(docs, &mut report) {
        return report;
    }
    ac5::documents_changed(docs, &rep, &mut report);
    if !guards::loss(docs, &rep, &ev, &mut report) {
        return report;
    }
    close::loss_open_item(&rep, &mut report);
    close::ac6(&rep, &mut report);
    close::ac7(&rep, &mut report);
    close::hand_offs(&rep, &mut report);
    close::open_002_10(&rep, &mut report);
    close::dod(&rep, &mut report);
    report
}
