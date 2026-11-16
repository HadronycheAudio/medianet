//! AC4: the review record — reviewer, date, the architecture heading, the
//! discrepancy list with resolutions, the attached table, its links.

use crate::doccheck::pycompat::PyStr;
use super::audit::Audit;
use super::refs::{base_of, dangling, rule_number};
use super::{ARCH_MD, Inputs, REPORTS_DIR, Trace, WIRE_MD};
use crate::doccheck::docs::{dirname, join};
use crate::doccheck::markdown::{norm_title, unnumbered};
use crate::doccheck::tables::split_row_with;
use crate::doccheck::policy::RowSplit;
use crate::doccheck::text::{full, head, rx, split_nl};
use crate::doccheck::{Docs, Report};

/// The review record found.
pub struct Record {
    pub text: String,
    pub path: String,
    /// True when it is a section of wire.md (not a separate file).
    pub in_wire: bool,
}

fn find_record(inp: &Inputs, trace: Option<&Trace>, docs: &Docs) -> Option<Record> {
    if let Some(t) = trace {
        let section = inp.wire.text(t.span);
        for c in rx(r"\[[^\]]*\]\(([^)\s#]+)(?:#[^)\s]*)?\)").captures_iter(&section) {
            let target = &c[1];
            if target.starts_with("http") {
                continue;
            }
            let p = join(dirname(WIRE_MD), target);
            if docs.is_file(&p) && p.starts_with(&format!("{REPORTS_DIR}/")) {
                let text = super::LINES.normalize(&docs.read(&p)?);
                return Some(Record { text, path: p, in_wire: false });
            }
        }
    }
    let h = inp.wire.heads.iter().find(|h| {
        rx(r"(?i)^\W*review\b").is_match(&h.text) || rx(r"(?i)review record").is_match(&h.text)
    })?;
    Some(Record {
        text: inp.wire.text(inp.wire.section(h)),
        path: WIRE_MD.to_string(),
        in_wire: true,
    })
}

fn reviewer_and_date(rt: &str, report: &mut Report) {
    let reviewer = split_nl(rt).iter().any(|line| {
        let Some(m) = rx(r"(?i)reviewer").find(line) else { return false };
        let after = rx(r"[*_|:`]").replace_all(&line[m.end()..], " ").py_strip().to_string();
        after.chars().count() >= 2 && !rx(r"(?i)^(tbd|todo|unknown|\?)").is_match(&after)
    });
    if !reviewer {
        report.fail("AC4", "the review record names no reviewer (a line 'Reviewer: <name>' with a name)");
    }
    let iso = rx(r"\b20\d\d-\d\d-\d\d\b");
    let month = rx(r"(?i)\b\d{1,2}\s+(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*\.?,?\s+20\d\d\b");
    let date = split_nl(rt).iter().any(|line| {
        rx(r"(?i)\bdate\b|reviewed on|reviewed:").is_match(line) && (iso.is_match(line) || month.is_match(line))
    });
    if !date {
        report.fail("AC4", "the review record states no date (a 'Date:' line with an ISO date such as 2026-11-16)");
    }
}

fn arch_heading(inp: &Inputs, rt: &str, report: &mut Report) {
    let cands: Vec<_> = inp
        .arch
        .heads
        .iter()
        .filter(|h| {
            let n = norm_title(&h.text);
            n.contains("protocol") && n.contains("packetis")
        })
        .collect();
    if cands.is_empty() {
        report.fail("AC4", format!("{ARCH_MD} has no heading on protocol and packetisation (cannot review against it)"));
        return;
    }
    let rec = norm_title(rt);
    if !cands.iter().any(|h| rec.contains(&norm_title(&unnumbered(&h.text)))) {
        let names: Vec<&str> = cands.iter().map(|h| h.text.as_str()).collect();
        report.fail("AC4", format!("the review record names no architecture heading on protocol and packetisation that exists in 01-architecture.md (candidates: {})", names.join(", ")));
    }
}

/// `\b0\d-[a-z-]+\.md\b|doc/design/(?!spec/wire)`: a resolution that
/// changes a design document other than wire.md.
fn names_other_doc(s: &str) -> bool {
    rx(r"\b0\d-[a-z-]+\.md\b").is_match(s)
        || s.match_indices("doc/design/").any(|(i, m)| !s[i + m.len()..].starts_with("spec/wire"))
}

fn owned(s: &str) -> bool {
    s.contains("004-23") || rx(r"(?i)owner").is_match(s)
}

fn discrepancy_table(tbl: &[&str], report: &mut Report) {
    let grp: Vec<Vec<String>> = tbl.iter().map(|l| split_row_with(l, RowSplit::Plain)).collect();
    let header: Vec<String> = grp[0].iter().map(|c| norm_title(c)).collect();
    let Some(ridx) = header.iter().position(|h| h == "resolution") else {
        report.fail("AC4", "a discrepancy table has no 'Resolution' column");
        return;
    };
    let data: Vec<&Vec<String>> = grp[1..]
        .iter()
        .filter(|r| !r.iter().filter(|c| !c.is_empty()).all(|c| full(":?-+:?", c)))
        .collect();
    if data.is_empty() {
        report.fail("AC4", "a discrepancy table has no discrepancy row and no explicit 'no discrepancy' statement");
    }
    for r in data {
        let res = r.get(ridx).map_or("", String::as_str);
        let joined = r.join(" | ");
        if res.is_empty() || ["-", "tbd", "TBD", "todo"].contains(&res) {
            report.fail("AC4", format!("a discrepancy has no resolution: '{}'", head(&joined, 100)));
            continue;
        }
        if names_other_doc(res) && !owned(&r.join(" ")) {
            report.fail("AC4", format!("a resolution changes a design document other than wire.md without naming 004-23 or an owner: '{}'", head(res, 100)));
        }
    }
}

/// The discrepancy section's body (from the first "discrepanc" line to the
/// next heading), checked; `None` when there is no such line.
fn discrepancies(rt: &str, report: &mut Report) -> String {
    let lines = split_nl(rt);
    let Some(at) = lines.iter().position(|l| rx(r"(?i)discrepanc").is_match(l)) else {
        report.fail("AC4", "the review record has no discrepancy list (a 'Discrepancies' heading or table)");
        return String::new();
    };
    let end = (at + 1..lines.len()).find(|&j| rx(r"^#{1,6}\s").is_match(lines[j])).unwrap_or(lines.len());
    let body = lines[at..end].join("\n");
    if rx(r"(?i)no discrepanc|none found").is_match(&body) {
        return body;
    }
    let mut tables: Vec<Vec<&str>> = Vec::new();
    let mut k = at + 1;
    while k < end {
        if lines[k].py_strip().starts_with('|') {
            let lo = k;
            while k < end && lines[k].py_strip().starts_with('|') {
                k += 1;
            }
            tables.push(lines[lo..k].to_vec());
        } else {
            k += 1;
        }
    }
    let marker = rx(r"^\s*([-*]|\d+\.)\s+");
    let items: Vec<&str> = lines[at + 1..end].iter().copied().filter(|l| marker.is_match(l)).collect();
    if tables.is_empty() && items.is_empty() {
        report.fail("AC4", "the discrepancy list has no rows, items or explicit 'no discrepancy' statement");
    }
    for t in &tables {
        discrepancy_table(t, report);
    }
    for li in items {
        if !rx(r"(?i)resolution\s*[:→—–-]\s*\S").is_match(li) {
            report.fail("AC4", format!("a discrepancy list item has no resolution: '{}'", head(li.py_strip(), 100)));
        } else if names_other_doc(li) && !owned(li) {
            report.fail("AC4", format!("a resolution changes a design document other than wire.md without naming 004-23 or an owner: '{}'", head(li.py_strip(), 100)));
        }
    }
    body
}

fn attached(rt: &str, trace: Option<&Trace>) -> bool {
    let linked = trace.is_some_and(|t| {
        rx(r"\]\(([^)\s]*)#([^)\s]+)\)").captures_iter(rt).any(|c| c[2] == *t.slug)
    });
    let rules: std::collections::BTreeSet<u32> = split_nl(rt)
        .iter()
        .filter(|l| l.py_strip().starts_with('|'))
        .filter_map(|l| rule_number(split_row_with(l, RowSplit::Plain).first().map_or("", String::as_str)))
        .collect();
    linked || (1..=22).all(|n| rules.contains(&n))
}

/// AC4 (all but Work 3); returns the record for Work 3 and DoD.
pub fn ac4(inp: &Inputs, trace: Option<&Trace>, audit: &Audit, docs: &Docs, report: &mut Report) -> Option<Record> {
    let Some(rec) = find_record(inp, trace, docs) else {
        report.fail("AC4", "no review record: neither a wire.md section headed 'Review record' nor a file under the reports directory linked from the Traceability section");
        return None;
    };
    if rec.text.is_empty() {
        return None;
    }
    reviewer_and_date(&rec.text, report);
    arch_heading(inp, &rec.text, report);
    let body = discrepancies(&rec.text, report);
    if !attached(&rec.text, trace) {
        report.fail("AC4", "the traceability table is not attached to the review record (no link to the Traceability anchor and no 22-row rule table)");
    }
    let base = if rec.in_wire { base_of(WIRE_MD) } else { dirname(&rec.path) };
    for d in dangling(docs, &rec.text, base, &inp.wire, WIRE_MD, "review record") {
        report.fail("AC4", d);
    }
    let mut seen = std::collections::BTreeSet::new();
    for name in &audit.bad_fields {
        if seen.insert(name) && !body.contains(name.as_str()) {
            report.fail("AC3", format!("destination-specific or per-receiver field {name} is not recorded as a discrepancy in the review record"));
        }
    }
    Some(rec)
}
