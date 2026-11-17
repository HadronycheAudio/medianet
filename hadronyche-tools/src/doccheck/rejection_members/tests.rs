//! The proof: every policy against the scripts' own Python, case by case
//! (`data/rejection_corpus.txt`), plus the shape of the API.

use std::collections::BTreeMap;

use super::reasons::*;
use super::*;

const CORPUS: &str = include_str!("../../../data/rejection_corpus.txt");
const WIRE: &str = include_str!("../../../../doc/design/spec/wire.md");

fn unescape(s: &str) -> String {
    let (mut out, mut it) = (String::new(), s.chars());
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(o) => out.push(o),
            None => {}
        }
    }
    out
}

/// (case, variant) → key → values in order.
type Facts = BTreeMap<(String, String), BTreeMap<String, Vec<String>>>;

fn facts() -> Facts {
    let mut out: Facts = BTreeMap::new();
    for line in CORPUS.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.splitn(4, '\t').collect();
        assert_eq!(f.len(), 4, "corpus line {line:?}");
        out.entry((f[0].to_string(), f[1].to_string()))
            .or_default()
            .entry(f[2].to_string())
            .or_default()
            .push(unescape(f[3]));
    }
    out
}

/// The inputs: the corpus records each case's name; the texts are rebuilt
/// here exactly as the generator builds them.
fn section_edit(wire: &str, old: &str, new: &str) -> String {
    let start = wire.find("## Rejection reasons").expect("section");
    let i = start + wire[start..].find(old).unwrap_or_else(|| panic!("{old:?}"));
    format!("{}{new}{}", &wire[..i], &wire[i + old.len()..])
}

fn row(wire: &str, name: &str) -> String {
    let start = wire.find("## Rejection reasons").expect("section");
    let i = start + wire[start..].find(&format!(" | {name} | ")).expect("row");
    let a = wire[..i].rfind('\n').map_or(0, |k| k + 1);
    let b = i + wire[i..].find('\n').expect("eol") + 1;
    wire[a..b].to_string()
}

/// As the generator's `excerpt`: wire.md up to the end of its Rejection
/// reasons section, then its Refusal reasons subsection under a level-2
/// heading.
fn excerpt(wire: &str) -> String {
    let start = wire.find("\n## Rejection reasons\n").expect("section");
    let end = start + 1 + wire[start + 1..].find("\n## ").expect("next section");
    let r0 = wire.find("\n### Refusal reasons\n").expect("refusals");
    let r1 = ["\n### ", "\n## "]
        .iter()
        .filter_map(|h| wire[r0 + 1..].find(h))
        .min()
        .map_or(wire.len(), |k| r0 + 1 + k);
    format!(
        "{}\n\n## Admission messages\n{}\n",
        &wire[..end],
        &wire[r0..r1]
    )
}

fn without(wire: &str, names: &[&str]) -> String {
    let mut t = wire.to_string();
    for n in names {
        t = t.replacen(&row(wire, n), "", 1);
    }
    t
}

fn cases() -> Vec<(&'static str, String)> {
    let excerpt = excerpt(WIRE);
    let w = excerpt.as_str();
    let r = |n: &str| row(w, n);
    let e = |old: &str, new: &str| section_edit(w, old, new);
    vec![
        ("base", WIRE.to_string()),
        ("excerpt", w.to_string()),
        ("delete-1", without(w, &["UnconfiguredDomain"])),
        (
            "delete-3-leaves-10",
            without(w, &["UnconfiguredDomain", "BadChecksum", "WrongEpoch"]),
        ),
        (
            "delete-4-leaves-9",
            without(
                w,
                &[
                    "UnconfiguredDomain",
                    "BadChecksum",
                    "WrongEpoch",
                    "BadMagic",
                ],
            ),
        ),
        (
            "duplicate-name-new-value",
            e(
                &r("ReservedNonZero"),
                &format!(
                    "{}| 13 | TooShort | again | control |\n",
                    r("ReservedNonZero")
                ),
            ),
        ),
        (
            "value-tie-11",
            e("| 12 | ReservedNonZero |", "| 11 | ReservedNonZero |"),
        ),
        ("values-swapped", e("| 0 | TooShort |", "| 1 | TooShort |")),
        (
            "values-swapped-2",
            e("| 1 | BadMagic |", "| 0 | BadMagic |"),
        ),
        ("markup-bold", e("| 1 | BadMagic |", "| 1 | **BadMagic** |")),
        ("markup-code", e("| 0 | TooShort |", "| 0 | `TooShort` |")),
        (
            "leading-zeros",
            e("| 7 | BadLength |", "| 007 | BadLength |"),
        ),
        (
            "value-beyond-i128",
            e(
                "| 7 | BadLength |",
                &format!("| {} | BadLength |", "9".repeat(45)),
            ),
        ),
        (
            "value-beyond-i64-on-the-zero-member",
            e("| 0 | TooShort |", "| 99999999999999999999 | TooShort |"),
        ),
        (
            "three-cells",
            e(
                &r("BadPacketIndex"),
                "| 9 | BadPacketIndex | the packet index |\n",
            ),
        ),
        (
            "lowercase-name",
            e("| 8 | BadChecksum |", "| 8 | badChecksum |"),
        ),
        (
            "hyphen-name",
            e("| 8 | BadChecksum |", "| 8 | Bad-Checksum |"),
        ),
        ("one-letter-name", e("| 8 | BadChecksum |", "| 8 | B |")),
        (
            "value-with-superscript",
            e("| 8 | BadChecksum |", "| 8² | BadChecksum |"),
        ),
        (
            "value-text",
            e("| 8 | BadChecksum |", "| eight | BadChecksum |"),
        ),
        (
            "tabs-in-cells",
            e("| 3 | ForeignDomain |", "|\t3\t|\tForeignDomain\t|"),
        ),
        (
            "escaped-pipe-in-meaning",
            e("fewer bytes than", "fewer \\| bytes than"),
        ),
        (
            "no-layers-control",
            e(
                "dropped before any field is read | control, sync, media, measurement |",
                "dropped before any field is read | sync, media, measurement |",
            ),
        ),
        (
            "layers-uppercase",
            e(
                "a control message exceeds the 1 252-byte UDP-payload bound (C-17) | control |",
                "a control message exceeds the 1 252-byte UDP-payload bound (C-17) | Control |",
            ),
        ),
    ]
    .into_iter()
    .chain(more_cases(w))
    .collect()
}

/// The second half of [`cases`] (split for the function-length gate).
fn more_cases(w: &str) -> Vec<(&'static str, String)> {
    let e = |old: &str, new: &str| section_edit(w, old, new);
    let first = w.find("\n## ").expect("a level-2 heading");
    vec![
        (
            "separator-removed",
            e("|---|---|---|---|\n| 0 | TooShort", "| 0 | TooShort"),
        ),
        (
            "separator-aligned",
            e(
                "|---|---|---|---|\n| 0 | TooShort",
                "|:--|--:|:-:|---|\n| 0 | TooShort",
            ),
        ),
        (
            "separator-no-dash",
            e(
                "|---|---|---|---|\n| 0 | TooShort",
                "|:::|:::|:::|:::|\n| 0 | TooShort",
            ),
        ),
        (
            "table-fenced",
            e("| Value (u8) | Name |", "```\n| Value (u8) | Name |"),
        ),
        (
            "extra-row-in-subsection",
            e(
                "### Counting model\n",
                "### Counting model\n\n| V | N | M | L |\n|---|---|---|---|\n| 13 | Extra | x | control |\n\n",
            ),
        ),
        (
            "header-renamed-code",
            e("| Value (u8) | Name |", "| Code | Name |"),
        ),
        (
            "header-name-first",
            e("| Value (u8) | Name |", "| Value | Label |"),
        ),
        (
            "trailing-space-name",
            e("| 5 | UnknownStream |", "| 5 | UnknownStream  |"),
        ),
        (
            "heading-level-3",
            w.replacen("## Rejection reasons\n", "### Rejection reasons\n", 1),
        ),
        (
            "heading-codes",
            w.replacen("## Rejection reasons\n", "## Rejection reason codes\n", 1),
        ),
        (
            "heading-hyphen",
            w.replacen("## Rejection reasons\n", "## Rejection-reasons\n", 1),
        ),
        (
            "heading-closing-hashes",
            w.replacen("## Rejection reasons\n", "## Rejection reasons ##\n", 1),
        ),
        (
            "heading-upper",
            w.replacen("## Rejection reasons\n", "## REJECTION REASONS\n", 1),
        ),
        (
            "heading-tab",
            w.replacen("## Rejection reasons\n", "##\tRejection\treasons\n", 1),
        ),
        (
            "heading-seven-hashes",
            w.replacen("## Rejection reasons\n", "####### Rejection reasons\n", 1),
        ),
        (
            "heading-fenced",
            w.replacen(
                "## Rejection reasons\n",
                "```\n## Rejection reasons\n```\n",
                1,
            ),
        ),
        (
            "earlier-level-3-match",
            format!(
                "{}\n### Rejection-reason codes\n\n| V | N | M | L |\n|---|---|---|---|\n| 1 | Early | x | control |\n{}",
                &w[..first],
                &w[first..]
            ),
        ),
        (
            "section-removed",
            w.replacen("## Rejection reasons\n", "## Reasons\n", 1),
        ),
        ("crlf", w.replace('\n', "\r\n")),
        ("lone-cr", w.replace('\n', "\r")),
    ]
}

/// The generator writes an empty name-to-value dict as `KEY# 0` (it
/// cannot tell it from an empty name-to-text dict).
fn put_items(out: &mut BTreeMap<String, Vec<String>>, key: &str, e: &Enumeration) {
    if e.members.is_empty() {
        out.entry(format!("{key}#"))
            .or_default()
            .push("0".to_string());
    } else {
        out.entry(key.to_string()).or_default().push(items(e));
    }
}

fn items(e: &Enumeration) -> String {
    e.members
        .iter()
        .map(|m| format!("{}={}", m.name, m.value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn layers(e: &Enumeration) -> Vec<String> {
    e.members
        .iter()
        .map(|m| format!("{}={}", m.name, m.layers()))
        .collect()
}

fn sorted(v: Vec<&str>) -> String {
    let mut v = v;
    v.sort_unstable();
    v.join(" ")
}

/// What the port computes for one (case, variant), in the corpus's keys.
fn port(variant: &str, wire: &str) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    macro_rules! put {
        ($k:expr, $v:expr) => {
            out.entry($k.to_string()).or_default().push($v)
        };
    }
    let fails = |ok: bool, ac: &str| if ok { String::new() } else { ac.to_string() };
    match variant {
        v if v.starts_with("enum-004-1") || v == "enum-004-20" => {
            let e = Enumeration::parse(wire, REJECTION_004_16);
            put_items(&mut out, "MEMBERS", &e);
            let ac = match v {
                "enum-004-19" => "5",
                "enum-004-20" => "AC5",
                _ => "AC4",
            };
            if v == "enum-004-20" {
                for l in layers(&e) {
                    put!("LAYERS", l);
                }
                put!("LAYERS#", e.count().to_string());
            }
            // the section missing is one failure; otherwise the pin
            put!(
                "fails",
                if e.section_found {
                    fails(e.has_exactly(13), ac)
                } else {
                    ac.to_string()
                }
            );
        }
        "enum-004-24" => {
            let e = Enumeration::parse(wire, REJECTION_004_24);
            put_items(&mut out, "MEMBERS", &e);
            for l in layers(&e) {
                put!("MEMBER_LAYERS", l);
            }
            put!("MEMBER_LAYERS#", e.count().to_string());
            put!("fails", fails(e.has_exactly(13), "AC4"));
        }
        "enum-004-25" | "enum-004-28" | "enum-004-29" => {
            let e = Enumeration::parse(wire, REJECTION_004_25);
            put_items(&mut out, "REJ_VALUE", &e);
            for l in layers(&e) {
                put!("REJ_LAYERS", l);
            }
            put!("REJ_LAYERS#", e.count().to_string());
            put!("CONTROL_MEMBERS", sorted(e.control_members()));
            let ok = if variant == "enum-004-29" {
                e.has_control_reasons(13, &CONTROL_REASONS_004_29)
            } else {
                e.matches_fixed(&REJ_FIXED)
            };
            put!("fails", fails(ok, "DoD"));
        }
        "enum-004-26" => {
            let rej = Enumeration::parse(wire, REJECTION_004_26);
            let refu = Enumeration::parse(wire, REFUSAL_004_26);
            put_items(&mut out, "REJECT", &rej);
            put_items(&mut out, "REFUSAL", &refu);
            put!(
                "fails",
                fails(rej.has_exactly(13) && refu.has_exactly(18), "DoD")
            );
        }
        v if v.starts_with("tolerant-") => {
            put!("MEMBERS", tolerant_members(wire, TOLERANT_004_06).join(" "));
            put!("fails", String::new());
        }
        "table-004-11" => {
            let names = value_name_table(wire);
            put!(
                "enum_names",
                names.as_ref().map_or("None".to_string(), |n| n.join(" "))
            );
            let set: std::collections::BTreeSet<&str> =
                names.iter().flatten().map(String::as_str).collect();
            let ok = names.is_some() && set == DEFAULT_13.into_iter().collect();
            put!("fails", fails(ok, "4"));
        }
        other => panic!("no port for {other}"),
    }
    out
}

/// The corpus was generated from these inputs: their lengths (in
/// characters) are recorded, so a changed `wire.md` asks for a
/// regeneration instead of failing obscurely.
#[test]
fn the_corpus_inputs_are_the_current_documents() {
    for (case, wire) in cases() {
        let line = format!("# input {case}: {} chars", wire.chars().count());
        assert!(
            CORPUS.lines().any(|l| l == line),
            "{case}: wire.md changed since the corpus was generated; run \
             `python3 hadronyche-tools/tools/gen_rejection_corpus.py > hadronyche-tools/data/rejection_corpus.txt`"
        );
    }
}

#[test]
fn every_enumeration_variant_equals_its_script() {
    let facts = facts();
    let mut seen = 0;
    for (case, wire) in cases() {
        for ((c, variant), want) in facts.range((case.to_string(), String::new())..) {
            if c != case {
                break;
            }
            if !(variant.starts_with("enum-")
                || variant.starts_with("tolerant-")
                || variant.starts_with("table-"))
            {
                continue;
            }
            seen += 1;
            assert_eq!(&port(variant, &wire), want, "case {case}, {variant}");
        }
    }
    assert_eq!(
        seen,
        cases().len() * 15,
        "every case of every variant is in the corpus"
    );
}

#[test]
fn the_refusal_set_of_c185_equals_004_18() {
    let facts = facts();
    let texts: Vec<(String, String)> = CORPUS
        .lines()
        .filter_map(|l| l.strip_prefix("c185\t"))
        .map(|l| {
            let (case, rest) = l.split_once('\t').expect("case");
            (
                case.to_string(),
                unescape(rest.strip_prefix("text\t").expect("text")),
            )
        })
        .collect();
    assert!(texts.len() >= 9);
    for (case, text) in texts {
        let want = &facts[&(case.clone(), "refusal-004-18".to_string())];
        let got = refusal_set(&text);
        assert_eq!(got.join(" "), want["REFUSAL_SET"][0], "{case}");
        let discovery = ["VersionMismatch", "NameTaken", "ModuleCap"];
        let ok = got.len() == 18 && discovery.iter().all(|d| got.iter().any(|g| g == d));
        // 004-18 fails AC2 once for the count and once per missing reason
        let mut fails = Vec::new();
        if got.len() != 18 {
            fails.push("AC2");
        }
        fails.extend(
            discovery
                .iter()
                .filter(|d| !got.iter().any(|g| g == *d))
                .map(|_| "AC2"),
        );
        assert_eq!(fails.join(" "), want["fails"][0], "{case}");
        let _ = ok;
    }
}

#[test]
fn the_reason_helpers_equal_their_scripts() {
    let facts = facts();
    let outcomes: Vec<String> = CORPUS
        .lines()
        .filter_map(|l| l.strip_prefix("outcome\t"))
        .map(|l| unescape(l.split_once("\ttext\t").expect("text").1))
        .collect();
    assert!(outcomes.len() >= 25);
    let members: Vec<&str> = DEFAULT_13.to_vec();
    let is_member = |w: &str| members.contains(&w);
    let union = |w: &str| members.contains(&w) || w == "PortCeiling" || w == "NameTaken";
    let get =
        |k: usize, v: &str, key: &str| facts[&(format!("o{k}"), v.to_string())][key][0].clone();
    for (k, o) in outcomes.iter().enumerate() {
        for v in ["named-004-16", "named-004-17"] {
            assert_eq!(
                named_reasons(o, is_member, LOOKBACK_004_16).join(" "),
                get(k, v, "found"),
                "{v} {o:?}"
            );
        }
        assert_eq!(
            named_reasons(o, union, LOOKBACK_004_16).join(" "),
            get(k, "named-004-18", "found"),
            "{o:?}"
        );
        for v in ["negated-004-19", "negated-004-20"] {
            assert_eq!(
                negated_tokens(o, CAMEL_LETTERS, LOOKBACK_004_19).join(" "),
                get(k, v, "found"),
                "{v} {o:?}"
            );
            assert_eq!(
                get(k, v, "pattern"),
                format!("{} /{}", LOOKBACK_004_19.pattern, LOOKBACK_004_19.chars),
                "{v}: the script's pattern is the library's"
            );
        }
        for v in [
            "guard-004-06",
            "guard-004-08",
            "guard-004-09",
            "guard-004-10",
        ] {
            let mut got = unnegated_non_members(o, is_member);
            got.sort();
            assert_eq!(got.join(" "), get(k, v, "found"), "{v} {o:?}");
        }
        for (v, p) in [
            ("nonrej-004-16", NON_REJECTION_004_16),
            ("nonrej-004-17", NON_REJECTION_004_16),
            ("nonrej-004-18", NON_REJECTION_004_18),
            ("nonrej-004-20", NON_REJECTION_004_20),
            ("nonrej-004-24", NON_REJECTION_004_24),
            ("nonrej-004-25", NON_REJECTION_004_25),
            ("nonrej-004-28", NON_REJECTION_004_25),
            ("nonrej-004-29", NON_REJECTION_004_29),
        ] {
            assert_eq!(
                if p.is_match(o) { "1" } else { "0" },
                get(k, v, "match"),
                "{v} {o:?}"
            );
        }
    }
    // the patterns are the scripts' own text
    for (v, p) in [
        ("nonrej-004-16", NON_REJECTION_004_16),
        ("nonrej-004-17", NON_REJECTION_004_16),
        ("nonrej-004-18", NON_REJECTION_004_18),
        ("nonrej-004-20", NON_REJECTION_004_20),
        ("nonrej-004-24", NON_REJECTION_004_24),
        ("nonrej-004-25", NON_REJECTION_004_25),
        ("nonrej-004-28", NON_REJECTION_004_25),
        ("nonrej-004-29", NON_REJECTION_004_29),
    ] {
        assert_eq!(
            facts[&("pattern".to_string(), v.to_string())]["source"][0],
            p.0,
            "{v}"
        );
    }
}

#[test]
fn the_api_on_a_small_table() {
    let t = "# W\n\n## Rejection reasons\n\n| Value | Name | Meaning | Layers |\n|---|---|---|---|\n| 1 | `BadMagic` | x | control |\n| 02 | **TooShort** | y | media |\n| x | NotANumber | z | |\n| 3 | lower | z | |\n\n### Notes\n\n| 6 | Deeper |\n|---|---|\n\n## Next\n\n| 7 | Outside |\n|---|---|\n";
    let e = Enumeration::parse(t, REJECTION_004_16);
    assert!(e.section_found);
    assert_eq!(e.names().collect::<Vec<_>>(), ["BadMagic", "TooShort"]);
    assert_eq!(e.get("TooShort").and_then(Member::int), Some(2));
    assert_eq!(e.control_members(), ["BadMagic"]);
    assert!(e.has_exactly(2) && !e.contains("Outside"));
    // Layered does not strip markup: neither name is read
    assert_eq!(Enumeration::parse(t, REJECTION_004_24).count(), 0);
    assert!(
        !Enumeration::parse(t, REJECTION_004_25).section_found
            || Enumeration::parse(t, REJECTION_004_25).count() == 0
    );
    assert_eq!(split_row(r"| a \| b | c |"), ["a | b", "c"]);
    assert_eq!(
        numbered_rows(
            "| 1 | A1 |\n|---|---|\n| 1 | Ab |\n| 2 | Ab |\n",
            RowRule::Loose
        )
        .len(),
        2
    );
}
