//! AC 5 (keys): every toml budget key is bound to its case by name tokens
//! and must equal the computed figure.

use std::collections::BTreeSet;

use super::allow::AllowRow;
use super::cases::Case;
use super::comp::{Comp, Consts};
use crate::doccheck::checked::sub;
use crate::doccheck::pynum::record_int_limit;
use crate::doccheck::report::Report;
use crate::doccheck::text::rx;
use crate::doccheck::tomlline::{Doc, Table, Value};

type Set = BTreeSet<String>;

fn set(words: &[&str]) -> Set {
    words.iter().map(|w| (*w).to_string()).collect()
}

/// `key_tokens`.
fn key_tokens(name: &str) -> BTreeSet<String> {
    rx(r"[^a-z0-9]+")
        .split(&name.to_lowercase())
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

/// `tok_match`: every group met, no forbidden token.
fn tok_match(tokens: &BTreeSet<String>, groups: &[Set], forb: &Set) -> bool {
    groups.iter().all(|g| g.iter().any(|w| tokens.contains(w)))
        && !forb.iter().any(|w| tokens.contains(w))
}

/// `BUDGET_FLAT`: the int keys (not bools) of every table whose name has
/// "budget", a later table's value replacing an earlier one's.
pub(crate) fn budget_flat(d: &Doc) -> Option<Vec<(String, i128)>> {
    let tables: Vec<&Value> = d
        .others
        .0
        .iter()
        .filter(|(n, _)| n.to_lowercase().contains("budget"))
        .map(|(_, v)| v)
        .collect();
    if tables.is_empty() {
        return None;
    }
    let mut flat: Vec<(String, i128)> = Vec::new();
    for v in tables {
        let blocks: Vec<&Table> = match v {
            Value::List(xs) => xs.iter().filter_map(Value::as_table).collect(),
            Value::Dict(t) => vec![t],
            _ => Vec::new(),
        };
        for t in blocks {
            for (k, v) in &t.0 {
                let n = match v {
                    Value::Int(n) => *n,
                    Value::BigInt(s) => {
                        record_int_limit(s);
                        0
                    }
                    _ => continue,
                };
                match flat.iter_mut().find(|(key, _)| key == k) {
                    Some(slot) => slot.1 = n,
                    None => flat.push((k.clone(), n)),
                }
            }
        }
    }
    Some(flat)
}

fn ann_words(k: i128) -> Set {
    match k {
        0 => set(&["mandatory", "bare", "plain", "none"]),
        1 => set(&["one", "1"]),
        3 => set(&["three", "3"]),
        4 => set(&["four", "4", "all", "maximum", "full"]),
        _ => BTreeSet::from([k.to_string()]),
    }
}

/// `case_groups`.
fn case_groups(c: &Case, k: &Comp) -> (Vec<Set>, Set) {
    match c.kind {
        "default" => (
            vec![set(&["default", "mandatory", "nominal", "1472"])],
            set(&["fallback", "worst"]),
        ),
        "worst" if c.ext != 0 => (
            vec![set(&["worst", "maximum", "max"])],
            set(&["fallback", "default", "no", "without"]),
        ),
        "worst" => (
            vec![
                set(&["worst", "maximum", "max"]),
                set(&["no", "without", "absent"]),
            ],
            set(&["fallback", "default"]),
        ),
        _ => {
            let n = if k.entry != 0 {
                sub(c.ann, k.ann_count).div_euclid(k.entry)
            } else {
                -1
            };
            let mut groups = vec![set(&["fallback"]), ann_words(n)];
            let mut forb = set(&["default", "worst"]);
            if c.ext != 0 {
                groups.push(set(&["extension", "measurement"]));
            } else {
                forb.extend(set(&["extension", "measurement"]));
            }
            (groups, forb)
        }
    }
}

type Part = (&'static str, Vec<Set>, Set, i128);

fn part_specs(k: &Comp, c: &Consts) -> Vec<Part> {
    let s = set;
    vec![
        (
            "packet header",
            vec![s(&["packet"]), s(&["header"])],
            s(&["content", "total", "spare", "allowance", "fallback"]),
            k.pkt,
        ),
        (
            "slice header",
            vec![s(&["slice"]), s(&["header"])],
            s(&["total", "spare", "allowance"]),
            k.slice,
        ),
        (
            "announcement count",
            vec![s(&["announcement"]), s(&["count"])],
            s(&["total", "spare", "allowance", "container", "max", "maximum"]),
            k.ann_count,
        ),
        (
            "announcement container maximum",
            vec![s(&["announcement"]), s(&["container", "max", "maximum"])],
            s(&["count", "total", "spare", "allowance"]),
            k.ann_max(),
        ),
        (
            "measurement extension",
            vec![s(&["measurement", "extension"])],
            s(&["allowance", "total", "spare"]),
            k.ext,
        ),
        (
            "content-message header",
            vec![s(&["content"]), s(&["header"])],
            s(&["packet", "slice", "total", "spare", "allowance"]),
            k.cw,
        ),
        (
            "checksum",
            vec![s(&["checksum", "crc"])],
            s(&["total", "spare", "allowance"]),
            k.cksum,
        ),
        (
            "audio",
            vec![s(&["audio"])],
            s(&["total", "spare", "allowance", "mandatory", "maximum"]),
            c.audio,
        ),
        (
            "path payload",
            vec![s(&["payload"])],
            s(&["fallback", "allowance", "total", "spare"]),
            c.pay_def,
        ),
        (
            "fallback path payload",
            vec![s(&["payload"]), s(&["fallback"])],
            s(&["allowance", "total", "spare"]),
            c.pay_fb,
        ),
        (
            "mandatory beside audio",
            vec![s(&["mandatory"])],
            s(&[
                "total",
                "spare",
                "allowance",
                "fallback",
                "default",
                "worst",
            ]),
            k.mandatory(),
        ),
        (
            "maximum beside audio",
            vec![s(&["maximum"])],
            s(&[
                "container",
                "announcement",
                "total",
                "spare",
                "allowance",
                "worst",
                "fallback",
                "default",
            ]),
            k.max_non_audio(),
        ),
    ]
}

fn case_keys(flat: &[(String, i128)], cases: &[Case], k: &Comp, f: &mut Report) {
    for c in cases {
        let (groups, forb) = case_groups(c, k);
        for (role, expected, words) in [
            ("total", c.total, set(&["total"])),
            ("spare", c.spare, set(&["spare", "margin"])),
        ] {
            let mut gs = groups.clone();
            gs.push(words);
            let cands: Vec<&(String, i128)> = flat
                .iter()
                .filter(|(key, _)| tok_match(&key_tokens(key), &gs, &forb))
                .collect();
            if cands.is_empty() {
                f.fail("AC5", format!("no wire.toml budget key for the {} case (announcements {}, extension {}) and its {role}", c.kind, c.ann, c.ext));
            }
            for (key, v) in cands {
                if *v != expected {
                    f.fail("AC5", format!("wire.toml key {key:?} = {v}, but the {} case (announcements {}, extension {}) has {role} {expected}", c.kind, c.ann, c.ext));
                }
            }
        }
    }
}

fn allowance_keys(flat: &[(String, i128)], rows: &[AllowRow], f: &mut Report) {
    let all: BTreeSet<i128> = rows.iter().map(|(_, a)| *a).collect();
    for (key, v) in flat {
        let toks = key_tokens(key);
        if !toks.contains("allowance") {
            continue;
        }
        let has_any = |ws: &[&str]| ws.iter().any(|w| toks.contains(*w));
        let pick = |pred: &dyn Fn(&str) -> bool| -> BTreeSet<i128> {
            rows.iter()
                .filter(|(l, _)| pred(l))
                .map(|(_, a)| *a)
                .collect()
        };
        let (want, role) = if has_any(&["not", "without", "non", "no"]) {
            (
                pick(&|l| l.contains("could not")),
                "could not carry the extension",
            )
        } else if has_any(&["could", "carry", "with", "measuring", "measure"]) {
            (
                pick(&|l| l.contains("could carry") && !l.contains("could not")),
                "could carry the extension",
            )
        } else {
            (all.clone(), "any computed allowance")
        };
        if want.is_empty() {
            f.fail("AC5", format!("wire.toml key {key:?} names an allowance for a stream that {role}, but wire.md has no such row"));
        } else if !want.contains(v) {
            f.fail(
                "AC5",
                format!("wire.toml key {key:?} = {v}, but the row for {role} is {want:?}"),
            );
        }
    }
    if rows.is_empty()
        && flat
            .iter()
            .any(|(key, _)| key_tokens(key).contains("allowance"))
    {
        f.fail(
            "AC5",
            "wire.toml names an allowance but wire.md has no computed allowance table",
        );
    }
}

/// AC 5: the keys of the toml budget tables.
pub(crate) fn budget_keys(
    flat: &[(String, i128)],
    cases: &[Case],
    rows: &[AllowRow],
    k: &Comp,
    c: &Consts,
    f: &mut Report,
) {
    case_keys(flat, cases, k, f);
    for (label, groups, forb, expected) in part_specs(k, c) {
        for (key, v) in flat {
            if tok_match(&key_tokens(key), &groups, &forb) && *v != expected {
                f.fail(
                    "AC5",
                    format!(
                        "wire.toml key {key:?} = {v}, but the {label} is {expected} (landed table)"
                    ),
                );
            }
        }
    }
    allowance_keys(flat, rows, f);
}
