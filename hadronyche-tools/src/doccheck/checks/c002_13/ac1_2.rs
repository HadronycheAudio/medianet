//! AC 1 (the checklist of the seven epic criteria before section (a)) and
//! AC 2 (one final verdict per section (a)..(e)).

use super::{Rep, TAG, bad, cut};
use crate::doccheck::Report;
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::text::{full, has, has_i, rx, starts};

/// `re.search(r'002-AC-%d(?!\d)' % n, l)`.
fn names_criterion(l: &str, n: usize) -> bool {
    has(&format!(r"002-AC-{n}(?!\d)"), l)
}

/// The heading texts of `names_section`: each heading's title without a
/// trailing parenthesis, stripped, lowercased, of 6 code points or more.
fn head_texts(rep: &Rep) -> Vec<String> {
    let paren = rx(r"\s*\([^)]*\)\s*$");
    (0..rep.lines.len())
        .filter(|&i| rep.level[i] > 0)
        .map(|i| paren.replace_all(&rep.title[i], "").py_strip().to_string())
        .filter(|t| t.chars().count() >= 6)
        .map(|t| t.to_lowercase())
        .collect()
}

/// AC 1.
pub(super) fn ac1(rep: &Rep, report: &mut Report) {
    let first = (0..rep.lines.len())
        .find(|&i| rep.level[i] > 0 && starts(r"^#+\s*\([a-e]\)", &rep.lines[i]));
    let pre: &[String] = match first {
        Some(i) => &rep.lines[..i],
        None => {
            bad(
                report,
                "AC 1: no lettered section heading (a)..(e) found".into(),
            );
            &rep.lines
        }
    };
    let heads = head_texts(rep);
    for n in 1..=7 {
        let ticked: Vec<&String> = pre
            .iter()
            .filter(|l| names_criterion(l, n) && starts(r"^\s*[-*]\s*\[[xX]\]", l))
            .collect();
        let unticked = pre
            .iter()
            .any(|l| names_criterion(l, n) && starts(r"^\s*[-*]\s*\[ \]", l));
        if ticked.len() != 1 {
            let extra = if unticked {
                " (an unticked line exists)"
            } else {
                ""
            };
            bad(report, format!("AC 1: 002-AC-{n} needs exactly one ticked \"[x]\" checklist line before section (a); found {}{extra}", ticked.len()));
            continue;
        }
        let line = ticked[0];
        let lower = line.to_lowercase();
        let names_section =
            has(r"\([a-e]\)", line) || heads.iter().any(|t| lower.contains(t.as_str()));
        if !names_section {
            bad(report, format!("AC 1: checklist line for 002-AC-{n} names no section (a)..(e) and no heading of the report"));
        } else if !has(TAG, line) {
            bad(
                report,
                format!("AC 1: checklist line for 002-AC-{n} carries no evidence tag"),
            );
        } else {
            report.passes.push(("AC1".into(), format!("002-AC-{n} ticked")));
        }
    }
}

/// The verdict body: the line without its `verdict:` lead and a trailing
/// `**`, stripped.
fn verdict_body(line: &str) -> String {
    let body = rx(r"^\s*(\*\*)?verdict:\s*").replace_all(line, "");
    rx(r"\s*\*\*\s*$")
        .replace_all(&body, "")
        .py_strip()
        .to_string()
}

/// AC 2.
pub(super) fn ac2(rep: &Rep, report: &mut Report) {
    for l in ['a', 'b', 'c', 'd', 'e'] {
        let Some(i) = rep.heading_index(&format!(r"^#+\s*\({l}\)")) else {
            bad(report, format!("AC 2: section ({l}) heading not found"));
            continue;
        };
        let j = rep.section_end(i, 2);
        let sec = &rep.lines[i + 1..j];
        let vl: Vec<(usize, &String)> = sec
            .iter()
            .enumerate()
            .filter(|(_, s)| starts(r"^\s*(\*\*)?verdict:", s))
            .collect();
        if vl.len() != 1 {
            bad(
                report,
                format!(
                    "AC 2: section ({l}) must carry exactly one \"verdict:\" line; found {}",
                    vl.len()
                ),
            );
            continue;
        }
        let (k, line) = vl[0];
        let body = verdict_body(line);
        if let Some(msg) = verdict_fault(l, k, sec, &body) {
            bad(report, msg);
        } else {
            report.passes.push(("AC2".into(), format!("section ({l})")));
        }
    }
}

/// The first rule the verdict line of section `l` breaks, as the script's
/// message.
fn verdict_fault(l: char, k: usize, sec: &[String], body: &str) -> Option<String> {
    if full("(?i)pending", body) {
        return Some(format!("AC 2: section ({l}) verdict is still pending"));
    }
    let last = sec.iter().rposition(|s| !s.py_strip().is_empty());
    if last != Some(k) {
        return Some(format!(
            "AC 2: section ({l}) verdict is not the last non-blank line of the section"
        ));
    }
    if !has_i("mechanism:|explanation:", body) {
        return Some(format!(
            "AC 2: section ({l}) verdict has neither \"mechanism:\" nor \"explanation:\""
        ));
    }
    if !has(r"\[(code|measured)\]", body) {
        return Some(format!(
            "AC 2: section ({l}) verdict carries no [code] or [measured] tag"
        ));
    }
    if l == 'd' && !starts("(instrument bug, mechanism:|not a bug, explanation:)", body) {
        return Some("AC 2: section (d) verdict must use one literal form: \"instrument bug, mechanism:\" or \"not a bug, explanation:\"".into());
    }
    if l == 'e' && !starts("instrument bug, mechanism:", body) {
        return Some(format!("AC 2: section (e) verdict must use the form \"instrument bug, mechanism:\" (the sync pacing is an instrument bug in the design vocabulary: ADR 21, 00-context.md, appendix 0c); found: {}", cut(body, 120)));
    }
    None
}
