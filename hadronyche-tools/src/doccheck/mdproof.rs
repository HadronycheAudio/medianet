//! The proof of the markdown, table, cell and look-around helpers against
//! the scripts' own Python (`tools/gen_markdown_corpus.py`): every record
//! of `data/markdown_corpus.txt` and `data/lookaround_corpus.txt` is
//! rebuilt from the library and compared in the generator's canonical form.

use std::fmt::Write as _;

use super::markdown::{Markdown, STOPWORDS_006_01, code_mask, stem_tokens};
use super::numwin::{lookaround, window};
use super::policy::{FencePolicy, LineSplit, TableRules};
use super::pycompat::{IntError, splitlines};
use super::tables::{
    caption_above, cell_label, cell_number_004_14, first_int_004_14, parse_tables, split_row,
    split_row_char_walk, split_row_with,
};

const MARKDOWN: &str = include_str!("../../data/markdown_corpus.txt");
const LOOKAROUND: &str = include_str!("../../data/lookaround_corpus.txt");
const LOOK_DOCS: &[&str] = &[
    "doc/design/01-architecture.md",
    "doc/design/02-constants.md",
    "doc/design/spec/timeline.md",
    "doc/design/spec/wire.md",
];

/// A value in the generator's canonical form (`q`).
enum Q {
    None,
    Int(i128),
    Str(String),
    List(Vec<Q>),
}

fn esc(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                let _ = write!(out, "\\x{{{:02x}}}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn unesc(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('x') => {
                let hex: String = it.by_ref().skip(1).take_while(|c| *c != '}').collect();
                let c = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
                out.push(c.unwrap_or('?'));
            }
            Some(o) => out.push(o),
            None => {}
        }
    }
    out
}

fn q(v: &Q) -> String {
    match v {
        Q::None => "None".into(),
        Q::Int(i) => i.to_string(),
        Q::Str(s) => format!("\"{}\"", esc(s)),
        Q::List(l) => format!("[{}]", l.iter().map(q).collect::<Vec<_>>().join(", ")),
    }
}

fn strs(v: &[String]) -> Q {
    Q::List(v.iter().map(|s| Q::Str(s.clone())).collect())
}

fn int(n: usize) -> Q {
    Q::Int(n as i128)
}

fn fnv(s: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn tables(
    text: &str,
    fences: FencePolicy,
    rules: TableRules,
) -> (Vec<String>, Vec<super::tables::Table>) {
    let lines: Vec<String> = splitlines(text).into_iter().map(String::from).collect();
    let mask = code_mask(&lines, fences);
    let t = parse_tables(&lines, &mask, 0, lines.len(), rules);
    (lines, t)
}

fn heads(text: &str, fences: FencePolicy) -> Vec<super::markdown::Heading> {
    Markdown::parse_with(&splitlines(text).join("\n"), fences).heads
}

fn mask_str(text: &str, fences: FencePolicy) -> String {
    code_mask(&splitlines(text), fences)
        .iter()
        .map(|f| if *f { '1' } else { '0' })
        .collect()
}

fn canon_t006_01(text: &str) -> String {
    let (lines, t) = tables(text, FencePolicy::Matching, TableRules::T006_01);
    q(&Q::List(
        t.iter()
            .map(|t| {
                let sep = split_row_with(&lines[t.line + 1], TableRules::T006_01.split);
                let mut block = vec![
                    Q::List(vec![int(t.line + 1), strs(t.head())]),
                    Q::List(vec![int(t.line + 2), strs(&sep)]),
                ];
                block.extend(
                    t.rows
                        .iter()
                        .map(|r| Q::List(vec![int(r.line + 1), strs(&r.cells)])),
                );
                Q::List(block)
            })
            .collect(),
    ))
}

fn canon_caption(text: &str) -> String {
    let (lines, t) = tables(text, FencePolicy::Backticks, TableRules::FAMILY_004);
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let mask = code_mask(&refs, FencePolicy::Backticks);
    q(&Q::List(
        t.iter()
            .map(|t| {
                let rows = t.rows.iter().map(|r| strs(&r.cells)).collect();
                Q::List(vec![
                    int(t.line),
                    int(t.line + 2 + t.rows.len()),
                    strs(t.head()),
                    Q::List(rows),
                    Q::Str(caption_above(&refs, &mask, t.line)),
                ])
            })
            .collect(),
    ))
}

/// The canonical result of a document-level kind, rebuilt from the library.
fn canon(kind: &str, text: &str) -> String {
    let n = splitlines(text).len();
    match kind {
        "tables_a" | "tables_b" => {
            let (_, t) = tables(text, FencePolicy::Backticks, TableRules::FAMILY_004);
            q(&Q::List(
                t.iter()
                    .map(|t| {
                        let rows = t.rows.iter().map(|r| strs(&r.cells)).collect();
                        Q::List(vec![int(t.line), strs(t.head()), Q::List(rows)])
                    })
                    .collect(),
            ))
        }
        "t006_01" => canon_t006_01(text),
        "t006_02" => {
            let (_, t) = tables(text, FencePolicy::SameChar, TableRules::T006_02);
            q(&Q::List(
                t.iter()
                    .map(|t| {
                        let rows = t
                            .rows
                            .iter()
                            .map(|r| Q::List(vec![int(r.line + 1), strs(&r.cells)]));
                        Q::List(vec![strs(t.head()), Q::List(rows.collect())])
                    })
                    .collect(),
            ))
        }
        "heads_a" | "heads_b" | "heads_c" | "heads_006_01" => {
            let (fences, base) = if kind == "heads_006_01" {
                (FencePolicy::Matching, 1)
            } else {
                (FencePolicy::Backticks, 0)
            };
            q(&Q::List(
                heads(text, fences)
                    .iter()
                    .map(|h| {
                        Q::List(vec![
                            int(h.line + base),
                            int(h.level),
                            Q::Str(h.text.clone()),
                        ])
                    })
                    .collect(),
            ))
        }
        "fence_004_14" => mask_str(text, FencePolicy::Backticks),
        "fence_006_01" => mask_str(text, FencePolicy::Matching),
        "fence_006_02" => mask_str(text, FencePolicy::SameChar),
        "find_006_02" => {
            let hs = heads(text, FencePolicy::NoFences);
            let mut seen: Vec<&str> = Vec::new();
            let mut out = Vec::new();
            for h in &hs {
                if seen.contains(&h.text.as_str()) {
                    continue;
                }
                seen.push(&h.text);
                let end = hs
                    .iter()
                    .find(|o| o.line > h.line && o.level <= h.level)
                    .map_or(n, |o| o.line);
                out.push(Q::List(vec![
                    Q::Str(h.text.clone()),
                    Q::List(vec![int(h.line), int(end)]),
                ]));
            }
            q(&Q::List(out))
        }
        "caption_004_14" | "caption_004_15" => canon_caption(text),
        other => panic!("unknown kind {other}"),
    }
}

fn int_result(r: Result<Option<i128>, IntError>) -> Option<Q> {
    match r {
        Ok(Some(v)) => Some(Q::Int(v)),
        Ok(None) => Some(Q::None),
        Err(_) => None,
    }
}

fn records(corpus: &str) -> impl Iterator<Item = (&str, &str, &str, &str)> {
    corpus.lines().filter(|l| !l.starts_with('#')).map(|l| {
        let f: Vec<&str> = l.splitn(4, '\t').collect();
        assert_eq!(f.len(), 4, "bad record {l}");
        (f[0], f[1], f[2], f[3])
    })
}

/// The documents of the corpus, read now; `None` for one whose text has
/// changed since the corpus was generated.
fn document(path: &str, hash: &str) -> Option<String> {
    let text = crate::doccheck::testing::repo().read(path)?;
    let text = LineSplit::Nl.normalize(&text);
    (format!("fnv:{}", fnv(&text)) == hash).then_some(text)
}

#[test]
fn markdown_helpers_equal_the_scripts_python() {
    let (mut checked, mut skipped, mut overflow, mut bad) = (0, 0, 0, Vec::new());
    for (kind, id, input, expected) in records(MARKDOWN) {
        let input_s = unesc(input);
        let got = match kind {
            "row_family" | "row_004_14" => q(&strs(&split_row(&input_s))),
            "row_006" => q(&split_row_char_walk(&input_s).map_or(Q::None, |c| strs(&c))),
            "stem_006_01" => q(&Q::List(
                stem_tokens(&input_s, STOPWORDS_006_01)
                    .into_iter()
                    .map(Q::Str)
                    .collect(),
            )),
            "cell_004_14" => {
                let (pn, fi) = (cell_number_004_14(&input_s), first_int_004_14(&input_s));
                match (int_result(pn), int_result(fi)) {
                    (Some(pn), Some(fi)) => q(&Q::List(vec![Q::Str(cell_label(&input_s)), pn, fi])),
                    _ => {
                        // beyond i128: the library reports IntError::Overflow
                        overflow += 1;
                        continue;
                    }
                }
            }
            "window" => {
                let f: Vec<&str> = input_s.split('\u{1f}').collect();
                let num = |k: usize| {
                    f[k].bytes()
                        .fold(0usize, |a, b| a * 10 + usize::from(b - b'0'))
                };
                let (t, s, e) = (f[0], num(1), num(2));
                let byte = |c: usize| t.char_indices().nth(c).map_or(t.len(), |(b, _)| b);
                q(&Q::Str(
                    window(t, byte(s), byte(e), num(3), num(4)).to_string(),
                ))
            }
            k if k.starts_with("doc:") => {
                let Some(text) = document(id, input) else {
                    skipped += 1;
                    continue;
                };
                format!("fnv:{}", fnv(&canon(&k[4..], &text)))
            }
            k => canon(k, &input_s),
        };
        checked += 1;
        if got != expected {
            bad.push(format!(
                "{kind} {id} {input}\n  python: {expected}\n  rust:   {got}"
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "{} of {checked} differ:\n{}",
        bad.len(),
        bad.join("\n")
    );
    assert!(
        checked > 1000 && skipped * 10 < checked,
        "checked {checked}, skipped {skipped}"
    );
    assert_eq!(
        overflow, 1,
        "the one cell beyond i128 (Python: an int; here IntError::Overflow)"
    );
}

fn look(pattern: &str, s: &str) -> String {
    let re = lookaround(pattern);
    let chars = |b: usize| int(s[..b].chars().count());
    q(&Q::List(
        re.captures_iter(s)
            .map(|c| {
                let m = c.get(0).expect("group 0");
                let groups = (1..c.len())
                    .map(|i| c.get(i).map_or(Q::None, |g| Q::Str(g.as_str().to_string())));
                Q::List(vec![
                    chars(m.start()),
                    chars(m.end()),
                    Q::List(groups.collect()),
                ])
            })
            .collect(),
    ))
}

#[test]
fn lookaround_patterns_equal_the_scripts_python() {
    // the documents of the generator's LOOK_DOCS
    let docs: Vec<String> = records(MARKDOWN)
        .filter(|r| r.0 == "doc:tables_a" && LOOK_DOCS.contains(&r.1))
        .filter_map(|(_, id, hash, _)| document(id, hash))
        .collect();
    assert_eq!(
        docs.len(),
        LOOK_DOCS.len(),
        "a look-around document changed: regenerate"
    );
    let all_lines: Vec<&str> = docs.iter().flat_map(|d| splitlines(d)).collect();
    let joined = all_lines.join("\n");
    let (mut checked, mut patterns, mut bad) = (0, std::collections::BTreeSet::new(), Vec::new());
    for (kind, id, input, expected) in records(LOOKAROUND) {
        let pattern = unesc(id);
        patterns.insert(pattern.clone());
        let got = match kind {
            "look" => look(&pattern, &unesc(input)),
            "split" => {
                if pattern.replace("\\(", "").replace("(?", "").contains('(') {
                    continue; // Python's split also returns the groups
                }
                q(&strs(
                    &lookaround(&pattern)
                        .split(&unesc(input))
                        .map(String::from)
                        .collect::<Vec<_>>(),
                ))
            }
            "doc:look" => {
                if format!("fnv:{}", fnv(&joined)) != input {
                    continue;
                }
                let body: Vec<String> = all_lines.iter().map(|l| look(&pattern, l)).collect();
                format!("fnv:{}", fnv(&body.join("\n")))
            }
            other => panic!("unknown kind {other}"),
        };
        checked += 1;
        if got != expected {
            bad.push(format!(
                "{kind} {pattern:?} {input}\n  python: {expected}\n  rust:   {got}"
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "{} of {checked} differ:\n{}",
        bad.len(),
        bad.join("\n")
    );
    assert!(
        patterns.len() > 40 && checked > 800,
        "{} patterns, {checked} records",
        patterns.len()
    );
}
