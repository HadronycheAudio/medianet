//! AC 2 (every section present, naming its ticket), AC 3 (the retired POC
//! behaviour note) and AC 4 (the Markdown parses).

use std::collections::BTreeSet;

use super::scan::{Block as Table, Head, Scan, norm_text, section_body};
use crate::doccheck::Report;
use crate::doccheck::markdown::{Heading, stem_match};
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::text::rx;

type Groups = &'static [&'static [&'static str]];

const SECTIONS: [(&str, &[&str], Groups); 15] = [
    (
        "Domain line and position map",
        &["006-02"],
        &[&["domain"], &["line"], &["position"], &["map"]],
    ),
    (
        "Epoch width and wrap",
        &["006-03"],
        &[&["epoch"], &["width"], &["wrap"]],
    ),
    (
        "Sync reply and line reconstruction",
        &["006-04"],
        &[
            &["sync"],
            &["reply"],
            &["line"],
            &["reconstruction", "reconstruct"],
        ],
    ),
    (
        "Epoch-bump causes",
        &["006-05"],
        &[&["epoch"], &["bump"], &["cause"]],
    ),
    (
        "Discontinuity event",
        &["006-06"],
        &[&["discontinuity"], &["event"]],
    ),
    (
        "Handover without a bump",
        &["006-07"],
        &[&["handover"], &["without", "no"], &["bump"]],
    ),
    (
        "Source-side switch procedure and session numbering",
        &["006-09"],
        &[
            &["source"],
            &["switch"],
            &["procedure"],
            &["session"],
            &["number"],
        ],
    ),
    (
        "Modules in Locking or holdover",
        &["006-10"],
        &[&["module"], &["lock"], &["holdover"]],
    ),
    (
        "Consumer-side procedures",
        &["006-11"],
        &[&["consumer"], &["procedure"]],
    ),
    (
        "Health of an input on another epoch",
        &["006-21"],
        &[&["health"], &["input"], &["other", "another"], &["epoch"]],
    ),
    (
        "Announcement ordering and loss",
        &["006-12"],
        &[&["announcement"], &["order"], &["loss"]],
    ),
    (
        "Sequence diagrams",
        &["006-13", "006-14", "006-15"],
        &[&["sequence"], &["diagram"]],
    ),
    (
        "Scenarios",
        &["006-16", "006-17", "006-18"],
        &[&["scenario"]],
    ),
    (
        "Regime stamp epoch field",
        &["006-19"],
        &[&["regime"], &["stamp"], &["epoch"], &["field"]],
    ),
    ("Open items", &["006-20"], &[&["open"], &["item"]]),
];

/// `matching_heading`: the library's stemmed-token match over the headings.
fn matching(headings: &[Head], groups: Groups) -> Option<&Head> {
    let hs: Vec<Heading> = headings
        .iter()
        .map(|h| Heading {
            line: h.0,
            level: h.1,
            text: h.2.clone(),
            slug: String::new(),
        })
        .collect();
    let found = stem_match(&hs, groups, norm_text)?.line;
    headings.iter().find(|h| h.0 == found)
}

/// `ids_in`: every `006-NN`, and the ranges written `006-NN..NN`, `-`, `to`.
fn ids_in(text: &str) -> BTreeSet<String> {
    let mut found: BTreeSet<String> = rx(r"006-(\d{2})")
        .captures_iter(text)
        .map(|m| format!("006-{}", &m[1]))
        .collect();
    let range =
        rx("(?i)006-(\\d{2})\\s*(?:\\.\\.+|[\\-\u{2013}\u{2014}]|to|through)\\s*(?:006-)?(\\d{2})");
    for m in range.captures_iter(text) {
        let (Ok(a), Ok(b)) = (parse_int(&m[1]), parse_int(&m[2])) else {
            continue;
        };
        if a <= b && b - a <= 30 {
            found.extend((a..=b).map(|k| format!("006-{k:02}")));
        }
    }
    found
}

/// AC 2.
pub(crate) fn ac2(lines: &[&str], sc: &Scan, r: &mut Report) {
    for (title, ids, groups) in SECTIONS {
        let Some(h) = matching(&sc.headings, groups) else {
            r.fail("AC2", format!("missing section heading: {title}"));
            continue;
        };
        let combined = format!("{}\n{}", h.2, section_body(lines, &sc.headings, h));
        if combined.py_strip().is_empty() {
            r.fail("AC2", format!("section {title:?} is empty"));
            continue;
        }
        let found = ids_in(&combined);
        let missing: Vec<&&str> = ids.iter().filter(|i| !found.contains(**i)).collect();
        if !missing.is_empty() {
            let m: Vec<&str> = missing.iter().map(|s| **s).collect();
            let w: Vec<&str> = found.iter().map(String::as_str).collect();
            r.fail("AC2", format!("section {title:?} does not name the ticket id that fills it (missing {}; written in the section: {})", m.join(", "), if w.is_empty() { "no 006-TT id".to_string() } else { w.join(", ") }));
        }
    }
}

const NOTE: &str = r"(?i)retired\s+(poc|proof[- ]of[- ]concept)\s+behaviou?r";

/// AC 3.
pub(crate) fn ac3(lines: &[&str], sc: &Scan, r: &mut Report) {
    let heading = sc.headings.iter().find(|h| rx(NOTE).is_match(&h.2));
    let note_line = match heading {
        Some(h) => Some(h.0),
        None => lines
            .iter()
            .position(|l| rx(NOTE).is_match(l))
            .map(|i| i + 1),
    };
    let Some(at) = note_line else {
        r.fail("AC3", "no 'Retired POC behaviour' note found");
        return;
    };
    let region = match heading {
        Some(h) => section_body(lines, &sc.headings, h),
        None => {
            let mut end = lines.len().min(at + 60);
            if let Some(n) = sc.headings.iter().find(|h| h.0 > at) {
                end = end.min(n.0 - 1);
            }
            lines[at - 1..end.max(at - 1)].join("\n")
        }
    };
    if !norm_text(&region).contains("[code]") {
        r.fail(
            "AC3",
            "the retired POC behaviour note does not carry [code]",
        );
    }
    if !rx("module\\.rs\\s*:\\s*1630\\s*[-\u{2013}\u{2014}]\\s*1634").is_match(&region) {
        r.fail(
            "AC3",
            "the retired POC behaviour note does not cite module.rs:1630-1634",
        );
    }
}

/// AC 4.
pub(crate) fn ac4(sc: &Scan, tables: &[Table], r: &mut Report) {
    for e in &sc.errors {
        r.fail("AC4", e.clone());
    }
    for t in tables {
        let width = t[0].1.len();
        for (line, cells) in &t[1..] {
            if cells.len() != width {
                r.fail(
                    "AC4",
                    format!(
                        "line {line}: table row has {} columns; its table header has {width}",
                        cells.len()
                    ),
                );
            }
        }
    }
}
