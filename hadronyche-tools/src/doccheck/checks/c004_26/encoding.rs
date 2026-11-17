//! AC2: the none encoding of required E and the unknown encoding of
//! declared E, as a sentinel outside the stated E range or as a flag field;
//! and review T3 (b), every sentinel stated in prose equals the table row
//! and the wire.toml field.

use std::collections::BTreeSet;

use super::fields::{Man, e_rows, numeric_range};
use super::toml_side::Toml;
use super::{HEXNUM, MRow, NUM, to_int};
use crate::doccheck::Report;
use crate::doccheck::report::Flow;
use crate::doccheck::text::{has, has_i, rx};

const NOT_INTERP: &str = r"not (?:interpreted|read|used)|ignored|disregard|does not apply|no meaning";
const DISTINCT_TXT: &str = r"distinct|never|cannot|not a valid|no valid|differ";

/// `code_for(text, word)`: the number before the word (within 40
/// characters, no `;.,` between), else the one after it.
pub(crate) fn code_for(text: &str, word: &str) -> Option<i128> {
    let n = format!("({HEXNUM}|{NUM})");
    let before = rx(&format!(r"(?i){n}[^;.,]{{0,40}}?\b{word}\b")).captures(text);
    let c = before.or_else(|| rx(&format!(r"(?i)\b{word}\b[^;.,]{{0,40}}?{n}")).captures(text))?;
    to_int(&c[1])
}

/// `2 ** bits` compared with `code`.
fn fits(code: i128, bits: i128) -> bool {
    bits >= 127 || code < (1i128 << bits.max(0))
}

/// How an encoding is stated.
#[derive(Clone)]
pub(crate) struct Enc {
    pub sentinel: bool,
    pub code: i128,
    pub row: MRow,
}

fn encoding_of(man: &Man, kind: &str, e: &[&MRow]) -> Option<Enc> {
    for r in e {
        if let (Some(code), Some((lo, hi))) = (code_for(&r.valid, kind), numeric_range(&r.valid))
            && !(lo <= code && code <= hi) && fits(code, r.bits) {
                return Some(Enc { sentinel: true, code, row: (*r).clone() });
            }
    }
    for r in man.all_rows() {
        if r.maxc.is_none() && numeric_range(&r.valid).is_none()
            && let Some(code) = code_for(&format!("{} {}", r.meaning, r.valid), kind) {
                return Some(Enc { sentinel: false, code, row: r.clone() });
            }
    }
    None
}

fn per_row(e: &[&MRow], label: &str, report: &mut Report) {
    for r in e {
        if let (Some(c), Some((lo, hi))) = (code_for(&r.valid, label), numeric_range(&r.valid)) {
            report.check("AC2", !(lo <= c && c <= hi), format!("the {label} code {c} lies outside the E range {lo} to {hi} that the same row states"));
        }
    }
}

fn form_checks(man: &Man, label: &str, enc: &Enc, e: &[&MRow], report: &mut Report) {
    if enc.sentinel {
        report.check("AC2", fits(enc.code, enc.row.bits), format!("the {label} sentinel {} fits the {}-bit E field", enc.code, enc.row.bits));
    } else {
        report.check("AC2", e.iter().any(|r| has_i(NOT_INTERP, &format!("{} {}", r.valid, r.meaning))), format!("the E value is stated as not interpreted when the flag says {label}"));
        let distinct = man.flat_sentences().iter().any(|s| {
            let l = s.to_lowercase();
            l.contains(label) && l.contains("valid") && has_i(DISTINCT_TXT, s)
        });
        report.check("AC2", distinct, format!("a sentence states that {label} is distinct from every valid E value"));
    }
}

/// AC2.
pub(crate) fn ac2(man: &Man, report: &mut Report) {
    let (req, dec) = (e_rows(man, "required"), e_rows(man, "declared"));
    per_row(&req, "none", report);
    per_row(&dec, "unknown", report);
    let none = encoding_of(man, "none", &req);
    let unknown = encoding_of(man, "unknown", &dec);
    report.check("AC2", none.is_some(), "the none encoding of required E is stated (a sentinel outside the E range, or a flag field)");
    report.check("AC2", unknown.is_some(), "the unknown encoding of declared E is stated (a sentinel outside the E range, or a flag field)");
    for (label, enc, e) in [("none", &none, &req), ("unknown", &unknown, &dec)] {
        if let Some(enc) = enc {
            form_checks(man, label, enc, e, report);
        }
    }
    if let (Some(n), Some(u)) = (&none, &unknown)
        && n.sentinel && u.sentinel {
            report.check("AC2", n.code != u.code, "the none and unknown sentinels are distinct numbers");
            for (label, enc) in [("none", n), ("unknown", u)] {
                let ok = numeric_range(&enc.row.valid).is_some_and(|(lo, hi)| !(lo <= enc.code && enc.code <= hi));
                report.check("AC2", ok, format!("the {label} sentinel {} lies outside its stated E range", enc.code));
            }
        }
}

/// T3 (b).
pub(crate) fn t3b(man: &Man, toml: &Toml, report: &mut Report) -> Flow<()> {
    let mut table_code = std::collections::BTreeMap::new();
    for (kind, rows) in [("none", e_rows(man, "required")), ("unknown", e_rows(man, "declared"))] {
        if let Some(c) = rows.iter().find_map(|r| code_for(&r.valid, kind)) {
            table_code.insert(kind, c);
        }
    }
    for (kind, other) in [("none", "unknown"), ("unknown", "none")] {
        let Some(&code) = table_code.get(kind) else { continue };
        let mut prose_hex = BTreeSet::new();
        for s in man.para_sentences() {
            if has_i(&format!(r"\b{kind}\b"), &s) && has(r"\bE\b", &s) {
                prose_hex.extend(rx("0x[0-9a-fA-F]+").find_iter(&s).filter_map(|m| i128::from_str_radix(&m.as_str()[2..], 16).ok()));
            }
        }
        let allowed: BTreeSet<i128> = [Some(code), table_code.get(other).copied()].into_iter().flatten().collect();
        let stray: Vec<&i128> = prose_hex.difference(&allowed).collect();
        if !prose_hex.is_empty() {
            report.check("AC2", prose_hex.contains(&code) && stray.is_empty(), format!("the prose sentinel for {kind} agrees with the table row ({code:#x}); prose hex values {prose_hex:?}"));
        }
        toml.sentinel_checks(kind, code, report)?;
    }
    Ok(())
}
