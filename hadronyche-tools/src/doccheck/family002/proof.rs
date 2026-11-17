//! The proof of `family002` against the real tools: every helper is
//! compared with the committed tables `data/family002/` (written by
//! `tools/gen_family002.py`, which ran GNU grep, sed, tr, cut, mawk and bash
//! in the scripts' locale; versions in the tables' headers). No tool runs
//! here.

use std::collections::{BTreeMap, BTreeSet};

use super::awk::{AwkRe, substr, tolower, v_assign};
use super::ere::{Grep, Sed, unescape_bytes, unescape_field};
use super::shell::*;
use super::tables::{CORPUS, compare_digests, inputs, rows};
use super::translate::{Syntax, is_space, is_word, posix_class, translate};

const CHARS: &str = include_str!("../../../data/family002/chars.tsv");
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

fn ids(spec: &str) -> BTreeSet<usize> {
    super::tables::ids(spec).into_iter().collect()
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
    let mut onlys = 0;
    for r in rows(CORPUS).filter(|r| r[0] == "OD") {
        let (kind, pattern) = (r[1], unescape_field(r[2]));
        let g = grep_of(kind.trim_end_matches('o'), &pattern);
        let outputs: Vec<Vec<u8>> = input
            .iter()
            .map(|l| g.only(l).join("\x1f").into_bytes())
            .collect();
        assert_digests(&format!("grep -o {pattern:?}"), &input, &outputs, r[3]);
        onlys += 1;
    }
    assert!(onlys >= 5);
    let mut seds = 0;
    for r in rows(CORPUS).filter(|r| r[0] == "SD") {
        let s = Sed::ere(&unescape_field(r[2]));
        let outputs: Vec<Vec<u8>> = input.iter().map(|l| s.apply(l).into_bytes()).collect();
        assert_digests(&format!("sed -E {:?}", s.script()), &input, &outputs, r[3]);
        seds += 1;
    }
    assert!(seds >= 2);
}

fn assert_digests(what: &str, inputs: &[impl AsRef<str>], outputs: &[Vec<u8>], digests: &str) {
    let inputs: Vec<&str> = inputs.iter().map(|s| s.as_ref()).collect();
    if let Err(e) = compare_digests(what, &inputs, outputs, digests) {
        panic!("{e}");
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
        .filter(|r| matches!(r[0], "Q" | "OD" | "SD" | "AT" | "AMD" | "AG"))
        .map(|r| (r[1].to_string(), unescape_field(r[2])))
        .collect();
    for (kind, p, origin) in proven_list() {
        let found = corpus.contains(&(kind.clone(), p.clone()));
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
        "word_count" => format!("{}\n", alnum_word_count(line)).into_bytes(),
        "terminal_count" => format!("{}\n", Grep::ere("[.!?]").only(line).len()).into_bytes(),
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
        "awk_v_assign" => v_assign(line).into_bytes(),
        "bash_test_le_30" => {
            let yes = bash_test_int(line).is_some_and(|n| n <= 30);
            (if yes { "yes" } else { "no" }).as_bytes().to_vec()
        }
        other => panic!("snippet {other} has no Rust counterpart"),
    }
}

/// Snippets named after a script (`002-03 unit_offenders`) are that
/// script's own code: its port's test proves them (`check_snippet`).
fn port_snippet(name: &str) -> bool {
    name.starts_with("002-")
}

#[test]
fn every_bash_snippet_gives_the_shells_output() {
    let input = inputs();
    let mut seen = 0;
    for r in rows(CORPUS) {
        match r[0] {
            "HD" if port_snippet(r[1]) => {}
            "HD" => {
                let subset: Vec<&str> = ids(r[2]).into_iter().map(|i| input[i].as_str()).collect();
                let outputs: Vec<Vec<u8>> = subset.iter().map(|l| snippet(r[1], l)).collect();
                assert_digests(r[1], &subset, &outputs, r[3]);
                seen += 1;
            }
            _ => {}
        }
    }
    assert!(seen >= 11, "every snippet has rows");
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
        "any_ptr_distinct" => {
            let sub = all();
            format!(
                "{}\n",
                distinct_count(Grep::ere(r"[A-Za-z0-9_./-]+\.rs:[0-9]+").only_lines(&sub))
            )
        }
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

/// Every awk pattern on every input: `~` (rows AT), `match` with RSTART
/// and RLENGTH (AM), `gsub` with an empty replacement (AG, digests).
#[test]
fn awk_patterns_give_mawks_answers() {
    let input = inputs();
    let bytes: Vec<&[u8]> = input.iter().map(|s| s.as_bytes()).collect();
    let mut n = 0;
    for r in rows(CORPUS) {
        let pattern = || unescape_field(r[2]);
        match r[0] {
            "AT" => {
                let re = AwkRe::new(&pattern());
                let want = ids(r[3]);
                for (i, b) in bytes.iter().enumerate() {
                    assert_eq!(
                        re.test(b),
                        want.contains(&i),
                        "awk {:?} ~ on {:?}",
                        pattern(),
                        input[i]
                    );
                }
            }
            "AMD" => {
                let re = AwkRe::new(&pattern());
                let outputs: Vec<Vec<u8>> = bytes
                    .iter()
                    .map(|b| {
                        re.find(b)
                            .map_or("-".to_string(), |(s, l)| format!("{s}:{l}"))
                            .into_bytes()
                    })
                    .collect();
                assert_digests(
                    &format!("awk match {:?}", pattern()),
                    &input,
                    &outputs,
                    r[3],
                );
            }
            "AG" => {
                let re = AwkRe::new(&pattern());
                let outputs: Vec<Vec<u8>> =
                    bytes.iter().map(|b| re.replace(b, b"", true)).collect();
                assert_digests(&format!("awk gsub {:?}", pattern()), &input, &outputs, r[3]);
            }
            _ => continue,
        }
        n += 1;
    }
    assert!(n >= 16, "the awk rows are there");
}

#[test]
fn awk_strings_are_bytes() {
    assert_eq!(substr(b"abc", 0, Some(1)), b"");
    assert_eq!(substr(b"abc", -1, Some(3)), b"a");
    assert_eq!(substr(b"abc", 2, None), b"bc");
    assert_eq!(substr(b"abc", 3, Some(24)), b"c");
    assert_eq!(substr(b"abc", 4, Some(1)), b"");
    assert_eq!(substr("aµb".as_bytes(), 2, Some(1)), &[0xC2]);
    assert_eq!(tolower("AΔB".as_bytes()), "aΔb".as_bytes());
    assert_eq!(AwkRe::new("x*").replace(b"abxd", b"-", true), b"-a-b-d-");
    assert_eq!(AwkRe::new("b").replace(b"abcb", b"[&]", false), b"a[b]cb");
}
