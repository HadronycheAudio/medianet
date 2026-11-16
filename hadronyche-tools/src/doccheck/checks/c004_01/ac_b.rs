//! AC 5 to AC 7.

use std::collections::{BTreeMap, BTreeSet};

use super::doc::{Doc, norm_cell, parse_tables};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_15::num::{guarded, is_word};
use crate::doccheck::tables::split_row;
use crate::doccheck::text::{full, has, has_i, rx, splitlines};

const PLACEHOLDERS: [(&str, &str); 22] = [
    ("rejection reasons", r"rejection[-\s]+reasons?"),
    ("versioning", r"version(?:ing)?\b"),
    ("reserved fields", r"reserved[-\s]+fields?"),
    ("media packet header", r"media[-\s]+packet[-\s]+header"),
    ("slice header", r"slice[-\s]+header"),
    ("announcements", r"announcements?\b"),
    ("measurement extension", r"measurement[-\s]+extension"),
    ("content messages", r"content[-\s]+messages?"),
    ("checksum", r"checksum"),
    ("size budget", r"size[-\s]+budget"),
    ("hello", r"\bhello\b"),
    ("election messages", r"election\b"),
    ("admission messages", r"admission\b"),
    ("manifest", r"\bmanifest\b"),
    ("path-MTU probe pair", r"path[-\s]*mtu|probe\s+pair"),
    ("regime stamp", r"regime[-\s]+stamp"),
    ("commissioning window messages", r"commissioning[-\s]+window"),
    ("sync request and reply", r"sync\s+requests?[^|]{0,20}repl(?:y|ies)|sync\s+request\s*/\s*reply"),
    ("receiver report", r"receiver[-\s]+reports?"),
    ("one-way probe", r"one[-\s]+way[-\s]+probe"),
    ("validity rules", r"validity[-\s]+rules?"),
    ("traceability table", r"traceability"),
];

/// AC 5: the 22 placeholder sections are present as distinct headings.
pub(crate) fn ac5(d: &Doc, r: &mut Report) {
    let mut used: BTreeMap<usize, &str> = BTreeMap::new();
    for (name, pat) in PLACEHOLDERS {
        let Some(i) = (0..d.heads.len()).find(|&i| has_i(pat, &d.heads[i].2)) else {
            r.fail("AC5", format!("placeholder section missing: {name}"));
            continue;
        };
        if let Some(other) = used.get(&i) {
            r.fail("AC5", format!("placeholder section {name:?} shares its heading with {other:?}: {:?}", d.heads[i].2));
            continue;
        }
        used.insert(i, name);
    }
}

/// AC 6: the four-layer table, its DSCP values and the IPv4 / TTL text.
pub(crate) fn ac6(wire: &str, d: &Doc, r: &mut Report) {
    let tables = parse_tables(wire);
    let found = tables.iter().find_map(|(start, header, rows)| {
        let hl: Vec<String> = header.iter().map(|h| h.to_lowercase()).collect();
        let l = hl.iter().position(|h| h.contains("layer"))?;
        let c = hl.iter().position(|h| h.contains("dscp"))?;
        Some((*start, rows, l, c))
    });
    let Some((start, rows, lcol, dcol)) = found else {
        r.fail("AC6", "no layer table with Layer and DSCP columns");
        return;
    };
    let cats: [(&str, &str); 4] = [("control", r"(?i)control|discovery"), ("sync", r"(?i)sync|clock"), ("media", r"(?i)media"), ("measurement", r"(?i)measurement")];
    let mut seen: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for row in rows.iter().filter(|row| row.len() > lcol.max(dcol)) {
        if let Some((cat, _)) = cats.iter().find(|(_, p)| has(p, &row[lcol])) {
            seen.entry(cat).or_default().push(&row[dcol]);
        }
    }
    if rows.len() != 4 {
        r.fail("AC6", format!("layer table has {} rows, expected four layers", rows.len()));
    }
    for (cat, _) in cats {
        let Some(v) = seen.get(cat) else {
            r.fail("AC6", format!("layer table has no {cat} row"));
            continue;
        };
        if v.len() != 1 {
            r.fail("AC6", format!("layer table has {} {cat} rows, expected one", v.len()));
            continue;
        }
        let ints: BTreeSet<i64> = rx(r"\d+").find_iter(v[0]).filter_map(|m| m.as_str().parse().ok()).collect();
        let (ok, want) = match cat {
            "control" => (ints.contains(&40) && !ints.contains(&46) && !ints.contains(&48), "CS5 (40)"),
            "sync" => (ints.contains(&48) && !ints.contains(&40) && !ints.contains(&46), "CS6 (48)"),
            "media" => (ints.contains(&46) && !ints.contains(&40) && !ints.contains(&48), "EF (46)"),
            _ => (ints.contains(&40) && ints.contains(&46) && !ints.contains(&48), "reports CS5 (40), probes EF (46)"),
        };
        if !ok {
            r.fail("AC6", format!("{cat} layer DSCP is {:?}, expected {want}", v[0]));
        }
    }
    let before = d.heads.iter().rposition(|h| h.0 < start);
    let section = match before {
        Some(k) => {
            let end = d.heads.get(k + 1).map_or(d.lines.len(), |n| n.0);
            d.lines[d.heads[k].0..end].join("\n")
        }
        None => d.lines[..(start + 10).min(d.lines.len())].join("\n"),
    };
    if !has_i(r"\bIPv4\b", &section) {
        r.fail("AC6", "layer overview does not state IPv4");
    }
    if !has_i(r"\bmulticast\b", &section) {
        r.fail("AC6", "layer overview does not state multicast");
    }
    if !has_i(r"\bTTL\s*(?:=|of\s*)?1\b", &section) {
        r.fail("AC6", "layer overview does not state multicast TTL 1");
    }
}

fn required() -> Vec<String> {
    let mut v: Vec<String> = (1..14).map(|n| format!("C-{n:02}")).collect();
    v.extend((17..23).map(|n| format!("C-{n:02}")));
    v.extend(["C-34", "C-40", "C-43", "C-44", "C-100", "C-137"].map(String::from));
    v
}

/// `numbers(text)`: the numbers of a constants cell, thousands separators
/// removed.
fn numbers(text: &str) -> Vec<String> {
    let t = rx(r"\bC-\d+\b").replace_all(text, " ").replace(['\u{a0}', '\u{202f}'], " ");
    let chars: Vec<char> = t.chars().collect();
    let mut joined = String::new();
    for (i, c) in chars.iter().enumerate() {
        let sep = *c == ','
            && i > 0
            && chars[i - 1].is_ascii_digit()
            && chars.get(i + 1..i + 4).is_some_and(|d| d.iter().all(char::is_ascii_digit))
            && !chars.get(i + 4).is_some_and(char::is_ascii_digit);
        if !sep {
            joined.push(*c);
        }
    }
    let re = rx(r"\d+(?:[ ]\d{3})*(?:[.]\d+)?");
    let mut out = Vec::new();
    for m in guarded(re, &joined, |c| is_word(c) || c == '.') {
        let w = m.get(0).map_or(0..0, |w| w.range());
        if !joined[w.end..].chars().next().is_some_and(is_word) {
            out.push(m[0].replace(' ', ""));
        }
    }
    out
}

/// AC 7: the Constants subsection lists every required id with the values of
/// 02-constants.md.
pub(crate) fn ac7(d: &Doc, consts: Option<&str>, const_path: &str, r: &mut Report) {
    let Some(consts) = consts else {
        r.fail("AC7", format!("{const_path} does not exist; cannot compare constants"));
        return;
    };
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    for line in splitlines(consts) {
        let s = line.trim();
        if !s.starts_with('|') {
            continue;
        }
        let cells = split_row(s);
        if cells.len() >= 3 && full(r"C-\d+", &cells[0]) {
            values.insert(cells[0].clone(), cells[2].clone());
        }
    }
    let pred = |t: &str| full(r"(?i)(?:\d+[.)]?\s*)?(?:design\s+)?constants?(?:\s+subsection)?", t.trim());
    let Some(i) = d.find(pred).or_else(|| d.find(|t| has_i(r"\bconstants?\b", t))) else {
        r.fail("AC7", "no Constants subsection");
        return;
    };
    let sec = d.section(i);
    let req = required();
    let mut entries: BTreeMap<String, String> = BTreeMap::new();
    for (_, header, rows) in parse_tables(&sec) {
        let idcol = header.iter().position(|h| norm_cell(h) == "id");
        let valcol = header.iter().position(|h| norm_cell(h).contains("value"));
        let Some(idcol) = idcol else { continue };
        for row in rows.iter().filter(|r| r.len() > idcol) {
            let cid = row[idcol].trim().trim_matches(['*', '`']).to_string();
            if req.contains(&cid) && !entries.contains_key(&cid) {
                let v = match valcol {
                    Some(v) if row.len() > v => row[v].clone(),
                    _ => row.join(" | "),
                };
                entries.insert(cid, v);
            }
        }
    }
    for line in splitlines(&sec) {
        for cid in &req {
            if !entries.contains_key(cid) && has(&format!(r"\b{}\b", regex::escape(cid)), line) {
                entries.insert(cid.clone(), line.trim().to_string());
            }
        }
    }
    for cid in &req {
        let Some(expected) = values.get(cid) else {
            r.fail("AC7", format!("{cid} is not present in {const_path} (test is stale)"));
            continue;
        };
        let Some(actual) = entries.get(cid) else {
            r.fail("AC7", format!("{cid} missing from the Constants subsection"));
            continue;
        };
        let (en, an) = (numbers(expected), numbers(actual).into_iter().collect::<BTreeSet<_>>());
        let missing: Vec<&String> = en.iter().filter(|n| !an.contains(*n)).collect();
        if !missing.is_empty() {
            r.fail("AC7", format!("{cid} mismatch: constants table says {expected:?} but wire.md says {actual:?} (missing value(s): {})", missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
        } else if en.is_empty() && actual.trim().chars().count() < 6 {
            r.fail("AC7", format!("{cid} has no value in the Constants subsection"));
        }
    }
}
