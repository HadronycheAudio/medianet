//! AC 5: the P-02 row of the constants table, and the "Documents changed"
//! list of the report.

use std::collections::BTreeSet;

use super::{BULLET, Rep, TAG, bad, cells, cut, is_table_row};
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::text::{has, has_i, rx, starts};
use crate::doccheck::docs::{norm, outside};
use crate::doccheck::{Docs, Report};

/// `has_num(s, n)`: `re.search(r'(?<![\d.])%s(?!\d)' % re.escape(n), s)`
/// (`n` is digits and at most one dot, which both escapes write `\.`).
fn has_num(s: &str, n: &str) -> bool {
    has(&format!(r"(?<![\d.]){}(?!\d)", regex::escape(n)), s)
}

/// The "x / y" pair of the report's P-02 replacement line, or the failure.
fn p02_pair(rep: &Rep, report: &mut Report) -> Option<(String, String)> {
    let line = rep
        .lines
        .iter()
        .find(|l| starts(r"^\s*\**P-02 replacement\*\*", l))
        .or_else(|| {
            rep.lines
                .iter()
                .find(|l| l.contains("P-02 replacement") && has_i("replac", l))
        });
    let Some(line) = line else {
        bad(
            report,
            "AC 5: the report has no \"P-02 replacement\" line to parse the figures from".into(),
        );
        return None;
    };
    let pair = rx(r"\*\*\s*(\d+(?:\.\d+)?)\s*/\s*(\d+(?:\.\d+)?)\s*\*\*")
        .captures(line)
        .or_else(|| rx(r"(\d+(?:\.\d+)?)\s*/\s*(\d+(?:\.\d+)?)").captures(line))
        .map(|m| (m[1].to_string(), m[2].to_string()));
    if pair.is_none() {
        bad(
            report,
            "AC 5: no \"x / y\" figure pair on the P-02 replacement line".into(),
        );
    }
    pair
}

/// AC 5, the P-02 row.
pub(super) fn p02(rep: &Rep, consts: &[String], report: &mut Report) {
    let pair = p02_pair(rep, report);
    let row = consts
        .iter()
        .find(|l| is_table_row(l) && cells(l)[0] == "P-02");
    let Some(row) = row else {
        bad(report, "AC 5: row P-02 not found in the constants table".into());
        return;
    };
    let Some((x, y)) = pair else {
        return;
    };
    let msg = if !(has_num(row, &x) && has_num(row, &y)) {
        format!("AC 5: P-02 row does not hold the clean 50 / 1000 Hz figures {x} / {y} from the report")
    } else if !has(r"\[measured\]", row) {
        "AC 5: P-02 row is not tagged [measured]".into()
    } else if !has_i("strict|C-147", row) {
        "AC 5: P-02 row does not say the figures come from runs without strict mode (C-147)".into()
    } else if !has_i("unprivileged|POC", row) {
        "AC 5: P-02 row does not say the figures are POC / unprivileged figures".into()
    } else if row.contains("37 / 77") {
        "AC 5: P-02 row still reads \"37 / 77\"".into()
    } else {
        report.passes.push(("AC5".into(), format!("P-02 holds {x} / {y}")));
        return;
    };
    bad(report, msg);
}

/// `os.path.exists(os.path.join(REPO, p))`: a file, or a directory holding
/// one (git trees have no empty directories).
fn exists(docs: &Docs, p: &str) -> bool {
    let key = norm(p);
    !outside(&key) && (docs.is_file(&key) || !docs.files_under(&key).is_empty())
}

/// AC 5, the "Documents changed" list.
pub(super) fn documents_changed(docs: &Docs, rep: &Rep, report: &mut Report) {
    let Some(dc) = rep.heading_index("Documents changed") else {
        bad(
            report,
            "AC 5: no \"Documents changed\" section in the report".into(),
        );
        return;
    };
    let dj = rep.section_end(dc, rep.level[dc]);
    let path_rx = rx(r"\b((?:doc|checks|src|tools)/[A-Za-z0-9_./-]*[A-Za-z0-9_])");
    let mut listed: BTreeSet<String> = BTreeSet::new();
    let mut bad_tag = false;
    for l in &rep.lines[dc + 1..dj] {
        let paths: Vec<String> = path_rx
            .captures_iter(l)
            .map(|c| c[1].to_string())
            .collect();
        if paths.is_empty() {
            continue;
        }
        for p in paths {
            if !exists(docs, &p) {
                bad(
                    report,
                    format!("AC 5: \"Documents changed\" lists a path that does not exist: {p}"),
                );
            }
            listed.insert(p);
        }
        if has(BULLET, l) && !has(TAG, l) {
            bad(
                report,
                format!(
                    "AC 5: \"Documents changed\" entry carries no evidence tag: {}",
                    cut(l.py_strip(), 120)
                ),
            );
            bad_tag = true;
        }
    }
    for must in [
        "doc/design/02-constants.md",
        "doc/design/06-evidence-appendix.md",
    ] {
        if !listed.contains(must) {
            bad(
                report,
                format!("AC 5: \"Documents changed\" does not list {must}"),
            );
        }
    }
    if !bad_tag && listed.contains("doc/design/02-constants.md") {
        report
            .passes
            .push(("AC5".into(), "documents changed listed".into()));
    }
}
