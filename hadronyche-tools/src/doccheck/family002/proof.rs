//! The proof of `family002` against the real tools: every helper is
//! compared with the committed tables `data/family002/` (written by
//! `tools/gen_family002.py`, which ran GNU grep, sed, tr, cut, mawk and bash
//! in the scripts' locale; versions in the tables' headers). No tool runs
//! here.

use std::collections::{BTreeMap, BTreeSet};

use super::ere::{Grep, Sed, unescape_bytes, unescape_field};
use super::shell::*;
use super::translate::{Syntax, is_space, is_word, posix_class, translate};
use crate::doccheck::pycompat::parse_int;

const CHARS: &str = include_str!("../../../data/family002/chars.tsv");
const CORPUS: &str = include_str!("../../../data/family002/corpus.tsv");
const PATTERNS: &str = include_str!("../../../data/family002/patterns.tsv");
const RANGES: &str = include_str!("../../../data/family002/ranges.tsv");

fn members(g: &Grep) -> String {
    let all: Vec<char> = rows(CHARS).map(|r| cp(r[0])).collect();
    let m: Vec<String> = all
        .into_iter()
        .filter(|c| g.matches(&c.to_string()))
        .map(|c| format!("{:04X}", u32::from(c)))
        .collect();
    if m.is_empty() {
        "-".into()
    } else {
        m.join(" ")
    }
}

#[test]
fn ranges_follow_each_matcher() {
    let mut n = 0;
    for r in rows(RANGES) {
        let (lo, hi) = r[0].split_once('-').expect("lo-hi");
        let digits = lo.chars().all(|c| c.is_ascii_digit());
        let alone = Grep::ere(&format!("^[{lo}-{hi}]$"));
        let forced = Grep::ere(&format!("^([{lo}-{hi}]|[^[:print:][:cntrl:]])$"));
        if digits {
            assert_eq!(members(&alone), r[1], "DFA range {lo}-{hi}");
        } else {
            let paired = Grep::ere(&format!("^([{lo}-{hi}]x|[{lo}-{hi}])$"));
            assert_eq!(members(&paired), r[1], "range {lo}-{hi}");
        }
        assert_eq!(members(&forced), r[2], "regex range {lo}-{hi}");
        n += 1;
    }
    assert_eq!(n, 57, "every digit range and a-z, A-Z");
}

const CLASS_COLUMNS: [&str; 18] = [
    "alnum", "alpha", "digit", "space", "upper", "lower", "punct", "blank", "print", "graph",
    "cntrl", "xdigit", "w", "s", "dot", "09", "az", "AZ",
];

fn rows(table: &str) -> impl Iterator<Item = Vec<&str>> {
    table
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split('\t').collect())
}

fn ids(spec: &str) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    if spec == "-" {
        return out;
    }
    for part in spec.split(',') {
        let (a, b) = part.split_once('-').unwrap_or((part, part));
        let (a, b) = (num(a), num(b));
        out.extend(a..=b);
    }
    out
}

fn num(s: &str) -> usize {
    usize::try_from(parse_int(s).expect("a number")).expect("an index")
}

fn cp(hex: &str) -> char {
    char::from_u32(u32::from_str_radix(hex, 16).expect("hex")).expect("char")
}

/// The grep pattern the generator used for class column `k`, whole line.
fn class_pattern(k: &str) -> String {
    match k {
        "w" => r"^\w$".into(),
        "s" => r"^\s$".into(),
        "dot" => "^.$".into(),
        "09" => "^[0-9]$".into(),
        "az" => "^[a-z]$".into(),
        "AZ" => "^[A-Z]$".into(),
        _ => format!("^[[:{k}:]]$"),
    }
}

#[test]
fn every_character_has_the_tools_classes() {
    let mut n = 0;
    for r in rows(CHARS) {
        let c = cp(r[0]);
        let s = c.to_string();
        for (icase, bits) in [(false, r[1]), (true, r[2])] {
            for (k, bit) in CLASS_COLUMNS.iter().zip(bits.chars()) {
                let g = Grep::new(Syntax::Ere, icase, &class_pattern(k));
                assert_eq!(
                    g.matches(&s),
                    bit == '1',
                    "grep -E{} {k} on U+{}",
                    if icase { "i" } else { "" },
                    r[0]
                );
            }
        }
        assert_eq!(is_word(c), r[1].as_bytes()[12] == b'1', "\\w U+{}", r[0]);
        assert_eq!(is_space(c), r[1].as_bytes()[3] == b'1', "space U+{}", r[0]);
        // grep -iF: the other repertoire characters that match c
        let want: BTreeSet<char> = if r[3] == "-" {
            BTreeSet::new()
        } else {
            r[3].split(' ').map(cp).collect()
        };
        let got: BTreeSet<char> = rows(CHARS)
            .map(|x| cp(x[0]))
            .filter(|d| *d != c && Grep::fixed_i(&s).matches(&d.to_string()))
            .collect();
        assert_eq!(got, want, "grep -iF partners of U+{}", r[0]);
        // bash [[:class:]] patterns
        let bash: BTreeSet<&str> = if r[4] == "-" {
            BTreeSet::new()
        } else {
            r[4].split(',').collect()
        };
        for k in [
            "alnum", "alpha", "digit", "space", "upper", "lower", "punct", "blank",
        ] {
            assert_eq!(
                posix_class(k, c, false) == Some(true),
                bash.contains(k),
                "bash [[:{k}:]] U+{}",
                r[0]
            );
        }
        assert_eq!(r[5], "1", "bash counts one character");
        assert_eq!(
            tr_lower(&s).as_bytes(),
            unescape_bytes(r[6]),
            "tr lower U+{}",
            r[0]
        );
        assert_eq!(
            tr_delete_space(&s).is_empty(),
            r[7] == "1",
            "tr -d space U+{}",
            r[0]
        );
        assert_eq!(
            &s.as_bytes()[..1],
            unescape_bytes(r[8]),
            "cut -c1-1 U+{}",
            r[0]
        );
        // mawk is byte-based: case maps ASCII only, length counts bytes,
        // a class or `.` never matches a multi-byte character whole
        assert_eq!(
            s.to_ascii_lowercase().as_bytes(),
            unescape_bytes(r[9]),
            "awk tolower U+{}",
            r[0]
        );
        assert_eq!(
            s.to_ascii_uppercase().as_bytes(),
            unescape_bytes(r[10]),
            "awk toupper U+{}",
            r[0]
        );
        assert_eq!(r[11], s.len().to_string(), "awk length U+{}", r[0]);
        let single = c.is_ascii();
        let want_awk = format!(
            "{}{}{}",
            u8::from(single && c.is_ascii_alphanumeric()),
            u8::from(single && is_space(c)),
            u8::from(single)
        );
        assert_eq!(r[12], want_awk, "awk classes U+{}", r[0]);
        n += 1;
    }
    assert_eq!(
        n + 1,
        crate::doccheck::repertoire::table().len(),
        "every repertoire character but newline"
    );
}

fn inputs() -> Vec<String> {
    rows(CORPUS)
        .filter(|r| r[0] == "I")
        .map(|r| unescape_field(r.get(2).copied().unwrap_or("")))
        .collect()
}

fn grep_of(kind: &str, pattern: &str) -> Grep {
    let flags = kind.trim_start_matches("grep-");
    let syntax = match &flags[..1] {
        "E" => Syntax::Ere,
        "G" => Syntax::Bre,
        _ => Syntax::Fixed,
    };
    Grep::new(syntax, flags[1..].contains('i'), pattern)
}

#[test]
fn every_pattern_matches_the_lines_grep_matched() {
    let input = inputs();
    assert!(input.len() > 3000, "the corpus has its inputs");
    let mut proven = 0;
    let mut wrong = Vec::new();
    for r in rows(CORPUS).filter(|r| r[0] == "Q") {
        let (kind, pattern) = (r[1], unescape_field(r[2]));
        let g = grep_of(kind, &pattern);
        let want = ids(r[3]);
        let got: BTreeSet<usize> = (0..input.len()).filter(|i| g.matches(&input[*i])).collect();
        for i in want.symmetric_difference(&got) {
            wrong.push(format!(
                "{kind} {pattern:?} on {:?}: grep {}",
                input[*i],
                want.contains(i)
            ));
        }
        proven += 1;
    }
    assert!(
        wrong.is_empty(),
        "{} differences:\n{}",
        wrong.len(),
        wrong[..wrong.len().min(40)].join("\n")
    );
    assert!(proven >= 30);
}

#[test]
fn grep_o_and_sed_give_the_tools_output() {
    let input = inputs();
    type Matches = BTreeMap<usize, Vec<Vec<u8>>>;
    let mut only: BTreeMap<(String, String), Matches> = BTreeMap::new();
    for r in rows(CORPUS).filter(|r| r[0] == "O") {
        let ms = r
            .get(4)
            .copied()
            .unwrap_or("")
            .split("\\x1f")
            .map(unescape_bytes)
            .collect();
        only.entry((r[1].to_string(), unescape_field(r[2])))
            .or_default()
            .insert(num(r[3]), ms);
    }
    for (_, p, _) in proven_list()
        .into_iter()
        .filter(|(k, _, _)| k.ends_with('o'))
    {
        only.entry((format!("grep-{}o", "E"), p)).or_default();
    }
    for ((kind, pattern), want) in &only {
        let g = grep_of(kind.trim_end_matches('o'), pattern);
        for (i, line) in input.iter().enumerate() {
            let got: Vec<Vec<u8>> = g.only(line).iter().map(|m| m.as_bytes().to_vec()).collect();
            let want = want.get(&i).cloned().unwrap_or_default();
            assert_eq!(got, want, "grep -o {pattern:?} on {line:?}");
        }
    }
    let mut seds = 0;
    for r in rows(CORPUS).filter(|r| r[0] == "SD") {
        let s = Sed::ere(&unescape_field(r[2]));
        let outputs: Vec<Vec<u8>> = input.iter().map(|l| s.apply(l).into_bytes()).collect();
        assert_digests(&format!("sed -E {:?}", s.script()), &input, &outputs, r[3]);
        seds += 1;
    }
    assert!(seds >= 2);
}

/// FNV-1a, 64 bits (the generator's digest).
fn fnv(data: &[u8]) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in data {
        h = (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// Compares `outputs` (one per input of `inputs`, in order) with the
/// generator's per-chunk digests; a mismatch names the chunk's inputs.
fn assert_digests(what: &str, inputs: &[impl AsRef<str>], outputs: &[Vec<u8>], digests: &str) {
    let want: Vec<&str> = digests.split(',').collect();
    let chunks: Vec<&[Vec<u8>]> = outputs.chunks(64).collect();
    assert_eq!(chunks.len(), want.len(), "{what}: chunk count");
    for (k, (chunk, want)) in chunks.iter().zip(want).enumerate() {
        let mut data = Vec::new();
        for o in *chunk {
            data.extend_from_slice(o);
            data.push(0x1e);
        }
        let got = format!("{:016x}", fnv(&data));
        if got != want {
            let lines: Vec<String> = (k * 64..(k * 64 + chunk.len()))
                .map(|i| {
                    format!(
                        "{:?} -> {:?}",
                        inputs[i].as_ref(),
                        String::from_utf8_lossy(&outputs[i])
                    )
                })
                .collect();
            panic!(
                "{what}: chunk {k} differs from the tool (run the snippet or sed on these lines to see the tool's output); ours:\n{}",
                lines.join("\n")
            );
        }
    }
}

fn proven_list() -> Vec<(String, String, String)> {
    rows(PATTERNS)
        .map(|r| {
            (
                r[0].to_string(),
                unescape_field(r[1]),
                r.get(2).unwrap_or(&"").to_string(),
            )
        })
        .collect()
}

#[test]
fn every_listed_pattern_is_in_the_corpus() {
    let corpus: BTreeSet<(String, String)> = rows(CORPUS)
        .filter(|r| matches!(r[0], "Q" | "O" | "SD"))
        .map(|r| (r[1].to_string(), unescape_field(r[2])))
        .collect();
    for (kind, p, origin) in proven_list() {
        let found = corpus.contains(&(kind.clone(), p.clone())) || kind.ends_with('o');
        assert!(found, "{kind} {p:?} ({origin}) has no row in corpus.tsv");
    }
}

/// The 002-02 `normalise()`, composed from the helpers as the port does.
fn normalise(s: &str) -> String {
    let mut s = lstrip_space(s);
    for p in ["- ", "* ", "+ ", "> "] {
        s = strip_prefix_once(s, p);
    }
    s = lstrip_space(s);
    s = strip_suffix_once(strip_prefix_once(s, "**"), "**");
    lstrip_space(s).to_string()
}

fn snippet(name: &str, line: &str) -> Vec<u8> {
    match name {
        "normalise" => normalise(line).into_bytes(),
        "tr_lower" => tr_lower(line).into_bytes(),
        "rstrip_space" => rstrip_space(line).as_bytes().to_vec(),
        "lstrip_space" => lstrip_space(line).as_bytes().to_vec(),
        "hash_run_len" => leading_run(line, " #")
            .chars()
            .count()
            .to_string()
            .into_bytes(),
        "tr_delete_space" => tr_delete_space(line).into_bytes(),
        "word_count" => alnum_word_count(line).to_string().into_bytes(),
        "terminal_count" => Grep::ere("[.!?]").only(line).len().to_string().into_bytes(),
        "cut_110" => {
            // cut ends every line it prints with a newline; empty input
            // (`printf '%s' ""`) has no line at all
            if line.is_empty() {
                return Vec::new();
            }
            let mut b = cut_bytes(line, 110).into_bytes();
            if b.ends_with("\u{FFFD}".as_bytes()) {
                b = line.as_bytes()[..line.len().min(110)].to_vec();
            }
            b.push(b'\n');
            b
        }
        other => panic!("snippet {other} has no Rust counterpart"),
    }
}

#[test]
fn every_bash_snippet_gives_the_shells_output() {
    let input = inputs();
    let mut seen = 0;
    for r in rows(CORPUS) {
        match r[0] {
            "HD" => {
                let subset: Vec<&str> = ids(r[2]).into_iter().map(|i| input[i].as_str()).collect();
                let outputs: Vec<Vec<u8>> = subset.iter().map(|l| snippet(r[1], l)).collect();
                assert_digests(r[1], &subset, &outputs, r[3]);
                seen += 1;
            }
            "HC" => {
                let values: Vec<&str> = r[3].split(',').collect();
                for (i, v) in ids(r[2]).into_iter().zip(values) {
                    assert_eq!(
                        snippet(r[1], &input[i]),
                        v.as_bytes(),
                        "{} on {:?}",
                        r[1],
                        input[i]
                    );
                }
                seen += 1;
            }
            _ => {}
        }
    }
    assert!(seen >= 9, "every snippet has rows");
}

fn records(v: &[&str]) -> String {
    v.iter().map(|l| format!("{l}\x1e")).collect()
}

fn probe(name: &str, text: &str) -> String {
    let lines = Lines::new(text);
    let all = || lines.sed_range_text(1, 99);
    let nl = |o: Option<usize>| o.map(|n| format!("{n}\n")).unwrap_or_default();
    match name {
        "grep_lines" => (1..=lines.len())
            .map(|n| format!("{n}:{}\x1e", lines.sed_line(n)))
            .collect(),
        "line_count" => format!("{}\n", lines.wc_l()),
        "read_loop_from_2" => records(&lines.read_loop_from(2)),
        "read_loop_from_1" => records(&lines.read_loop_from(1)),
        "subst_range_1_3" => lines.sed_range_text(1, 3),
        "subst_range_2_9" => lines.sed_range_text(2, 9),
        "subst_line_2" => lines.sed_line(2).to_string(),
        "heredoc_lines_all" => records(&heredoc_lines(&all())),
        "heading_no_d" => nl(lines.heading_no(&Grep::ere_i(r"^#+[[:space:]]*\(d\)"))),
        "heading_no_open" => {
            nl(lines.heading_no(&Grep::ere_i(r"^#+[[:space:]]*Open sub-questions")))
        }
        "count_nonempty_sub" => format!("{}\n", Grep::bre("[^[:space:]]").count_lines(&all())),
        "ptr_distinct" => {
            let sub = all();
            format!(
                "{}\n",
                distinct_count(Grep::ere(r"audio\.rs:[0-9]+").only_lines(&sub))
            )
        }
        other => panic!("file probe {other} has no Rust counterpart"),
    }
}

#[test]
fn file_level_probes_give_the_tools_output() {
    let fixtures: BTreeMap<String, String> = rows(CORPUS)
        .filter(|r| r[0] == "F")
        .map(|r| {
            (
                r[1].to_string(),
                unescape_field(r.get(2).copied().unwrap_or("")),
            )
        })
        .collect();
    let mut n = 0;
    for r in rows(CORPUS).filter(|r| r[0] == "R") {
        let text = &fixtures[r[2]];
        let want = unescape_field(r.get(3).copied().unwrap_or(""));
        assert_eq!(probe(r[1], text), want, "{} on fixture {text:?}", r[1]);
        n += 1;
    }
    assert!(n >= 200);
}

/// A construct the translation does not reproduce is refused, never
/// guessed: the pattern becomes `Rx::refused`, whose every use records an
/// engine failure (R19, tested in `rx`), so the check turns red under
/// `REGEX_LIMIT`.
#[test]
fn refused_constructs_turn_red_never_guess() {
    for (s, p) in [
        (Syntax::Ere, r"\d"),
        (Syntax::Ere, "*a"),
        (Syntax::Ere, "a{x}"),
        (Syntax::Ere, r"(a)\1"),
        (Syntax::Bre, r"\1"),
        (Syntax::Ere, "(a"),
        (Syntax::Ere, "[a-9]"),
        (Syntax::Ere, "[[=a=]]"),
    ] {
        assert!(translate(s, false, p).is_err(), "{p:?} is refused");
        assert!(!Grep::new(s, false, p).matches("a d 1"));
    }
    assert_eq!(
        Sed::ere("y/a/b/").apply("a"),
        "a",
        "an unsupported sed command changes nothing"
    );
}

#[test]
fn the_line_model_reads_like_sed_and_read() {
    let l = Lines::new("a\nb\nc");
    assert_eq!((l.len(), l.wc_l()), (3, 2));
    assert_eq!(
        l.read_loop_from(1),
        vec!["a", "b"],
        "read loses an unterminated last line"
    );
    assert_eq!(
        l.sed_range(3, 1),
        3..=3,
        "a second address before the first: one line"
    );
    assert!(l.sed_range(0, 2).is_empty() && l.sed_range(4, 9).is_empty());
    assert_eq!(l.line_range(2, 1), "");
    assert_eq!(Lines::new("x\n\n\n").sed_range_text(1, 3), "x");
    assert_eq!(cut_bytes("a—b", 2), "a\u{FFFD}");
}
