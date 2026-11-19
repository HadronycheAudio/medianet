//! AC4, Work 3: every subsection 3.N accounted for, the known
//! contradictions recorded, and no over-claiming scope sentence.

use crate::doccheck::pycompat::{PyStr, is_space, is_word};
use super::record::Record;
use super::{ARCH_MD, Inputs, Trace};
use crate::doccheck::Report;
use crate::doccheck::markdown::norm_title;
use crate::doccheck::text::{rx, split_nl};

fn is_item(line: &str) -> bool {
    line.py_strip().starts_with('|') || rx(r"^\s*([-*]|\d+\.)\s+").is_match(line)
}

fn no_diff(line: &str) -> bool {
    rx(r"(?i)no\s+(discrepanc|difference)").is_match(line)
}

/// `(?<![\d.])3\.N(?!\d)`.
pub fn names_sec(line: &str, num: &str) -> bool {
    let needle = format!("3.{num}");
    line.match_indices(&needle).any(|(i, m)| {
        let before = line[..i].chars().next_back();
        let after = line[i + m.len()..].chars().next();
        !before.is_some_and(|c| c.is_numeric() || c == '.') && !after.is_some_and(char::is_numeric)
    })
}

fn discrepancy_line(line: &str, num: &str, must: Option<&str>) -> bool {
    is_item(line) && !no_diff(line) && names_sec(line, num) && must.is_none_or(|m| rx(&format!("(?i){m}")).is_match(line))
}

/// The lines under a heading that names § 3 (the record's own § 3 parts).
fn s3_lines(rec_lines: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_s3 = false;
    for line in rec_lines {
        if line.py_lstrip().starts_with('#') {
            in_s3 = rx(r"§\s*3\b").is_match(line);
            continue;
        }
        if in_s3 {
            out.push(line.to_string());
        }
    }
    out
}

fn subsections(inp: &Inputs, rec_lines: &[&str], s3: &[String], report: &mut Report) {
    let arch = &inp.arch;
    let Some(proto) = arch.heads.iter().find(|h| {
        let n = norm_title(&h.text);
        n.contains("protocol") && n.contains("packetis")
    }) else {
        return;
    };
    let span = arch.section(proto);
    let subs: Vec<(String, String)> = arch
        .heads
        .iter()
        .filter(|h| h.line > span.lo && h.line < span.hi)
        .filter_map(|h| rx(r"^3\.(\d+)\b").captures(&h.text).map(|c| (c[1].to_string(), h.text.clone())))
        .collect();
    if subs.is_empty() {
        report.fail("AC4", format!("{ARCH_MD}: no subsection 3.N under the protocol section, so Work 3 cannot be checked"));
    }
    for (num, title) in subs {
        let covered = s3.iter().any(|l| discrepancy_line(l, &num, None))
            || rec_lines.iter().any(|l| names_sec(l, &num) && no_diff(l));
        if !covered {
            report.fail("AC4", format!("Work 3: the architecture subsection 3.{num} ('{title}') is neither named by a discrepancy row or item of the review record nor stated to have no difference (quote the 'no difference' statement for it)"));
        }
    }
}

/// `re.sub(r"\s+", " ", text)` (Python's white space, [`is_space`]).
fn flat(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_space = false;
    for c in text.chars() {
        if is_space(c) {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

fn contradictions(inp: &Inputs, rec_lines: &[&str], report: &mut Report) {
    let w = &inp.wire;
    if let Some(ci) = w.heads.iter().find(|h| norm_title(&h.text).starts_with("copy index"))
        && w.text(w.section(ci)).to_lowercase().contains("retransmi")
        && !rec_lines.iter().any(|l| discrepancy_line(l, "7", Some("retransmi")))
    {
        report.fail("AC4", "the review record has no discrepancy row naming § 3.7 and 'retransmi' for the 'retransmitting' wording of wire.md's Copy index section");
    }
    let a = &inp.arch;
    if let Some(s5) = a.heads.iter().find(|h| rx(r"^3\.5\b").is_match(&h.text))
        && rx(r"(?i)verified by the receiving peer").is_match(&flat(&a.text(a.section(s5))))
        && !rec_lines.iter().any(|l| discrepancy_line(l, "5", Some("verif")))
    {
        report.fail("AC4", "the review record has no discrepancy row naming § 3.5 and 'verified' for the 'verified by the receiving peer' marking the architecture states");
    }
    let wflat = flat(&inp.wire_text);
    let row = |l: &&str| {
        is_item(l)
            && !no_diff(l)
            && rx(r"(?i)residence").is_match(l)
            && rx(r"(?i)contradic").is_match(l)
            && rx(r"(?i)owner").is_match(l)
    };
    if rx(r"(?i)send timestamp of the output packet").is_match(&wflat)
        && rx(r"(?i)same for every consumer").is_match(&wflat)
        && !rec_lines.iter().any(row)
    {
        report.fail("AC4", "wire.md says the residence term runs to 'the send timestamp of the output packet' and that the extension's values are 'the same for every consumer': the review record has no discrepancy row naming residence_time as a contradiction for the design owner");
    }
}

/// `[re.sub(r"[^\w/.\-]", "", t).lower() for t in re.findall(r"\S+", text)]`
/// (Python's `\w`, [`is_word`]).
fn tokens(text: &str) -> Vec<String> {
    text.py_split()
        .map(|t| t.chars().filter(|&c| is_word(c) || "/.-".contains(c)).collect::<String>().to_lowercase())
        .collect()
}

fn window(tk: &[String], i: usize, back: usize, fwd: usize) -> &[String] {
    // Python: tk[max(0, i - back):i + fwd]
    &tk[i.max(back) - back..(i + fwd).min(tk.len())]
}

fn scope_claims(label: &str, text: &str, report: &mut Report) {
    let tk = tokens(text);
    for i in 1..tk.len() {
        let i = i - 1;
        if tk[i] == "every"
            && tk[i + 1] == "known"
            && window(&tk, i, 12, 14).iter().any(|t| ["difference", "discrepanc", "item"].iter().any(|p| t.starts_with(p)))
        {
            report.fail("AC4", format!("the {label} claims to hold every known difference ('every known ...'); the record may state only the items this review found and the earlier reports it names"));
            break;
        }
    }
    let wide = ["section", "placement", "here", "where", "file", "record", "kept"];
    for (i, t) in tk.iter().enumerate() {
        if t.starts_with("004-22")
            && window(&tk, i, 8, 9).iter().any(|x| x == "allows" || x == "permits")
            && window(&tk, i, 16, 17).iter().any(|x| wide.iter().any(|p| x.starts_with(p)))
        {
            report.fail("AC4", format!("the {label} says ticket 004-22 allows or permits the record's placement; the ticket names no placement"));
            break;
        }
    }
}

/// AC4, Work 3.
pub fn ac4_work3(inp: &Inputs, trace: Option<&Trace>, rec: &Record, report: &mut Report) {
    let rec_lines = split_nl(&rec.text);
    let s3 = s3_lines(&rec_lines);
    subsections(inp, &rec_lines, &s3, report);
    contradictions(inp, &rec_lines, report);
    let trace_text = trace.map(|t| inp.wire.text(t.span)).unwrap_or_default();
    scope_claims("review record", &rec.text, report);
    scope_claims("Traceability section", &trace_text, report);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_numbers_are_whole() {
        assert!(names_sec("| R-14 | 3.7 Redundancy |", "7"));
        assert!(!names_sec("13.7", "7"));
        assert!(!names_sec("3.71", "7"));
        assert!(!names_sec("§ 4.3.7", "7"));
        assert!(names_sec("(3.5)", "5"));
    }

    #[test]
    fn scope_windows() {
        let mut r = Report::default();
        scope_claims("x", "it holds every known difference of the epic", &mut r);
        scope_claims("x", "ticket 004-22 allows the record to be kept here", &mut r);
        assert_eq!(r.failures.len(), 2);
    }
}
