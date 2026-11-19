//! Helpers for the ports' test files (`tests/doc_<id>.rs`).
//!
//! A check reads only the `Docs` it is given; a test file gets its documents
//! from here and nowhere else: clippy (`hadronyche-tools/clippy.toml`)
//! forbids `Docs::repo`, `Docs::dir`, `at_commit`, processes, files and the
//! environment everywhere else, and `source_rules` (below) covers what clippy
//! cannot see. This module is the explicit allow-list: the repository's
//! working tree for the criterion tests, the red-phase tree, and the source
//! tests.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)] // clippy.toml allow-list: the test helpers and the source tests

use std::collections::HashMap;
use std::io::Write as _;
use std::sync::{Mutex, OnceLock};

use super::checks::CheckSpec;
use super::docs::{Docs, commit_exists, repo_root};
use super::report::Report;

/// The environment variable that lets a red-phase test skip in a clone
/// without the commit (a shallow checkout). Unset, a missing commit fails.
pub const ALLOW_SHALLOW: &str = "DOCCHECK_ALLOW_SHALLOW";

/// The repository's working tree, shared by the tests of one binary.
pub fn repo() -> &'static Docs {
    static DOCS: OnceLock<Docs> = OnceLock::new();
    DOCS.get_or_init(Docs::repo)
}

/// The check on the repository's working tree, computed once per check and
/// test binary.
pub fn report_of(spec: &'static CheckSpec) -> Report {
    static REPORTS: OnceLock<Mutex<HashMap<&'static str, Report>>> = OnceLock::new();
    let cache = REPORTS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(r) = cache.lock().expect("report cache").get(spec.id) {
        return r.clone();
    }
    let r = spec.evaluate(repo());
    cache
        .lock()
        .expect("report cache")
        .insert(spec.id, r.clone());
    r
}

/// Writes past the test harness's output capture, so a line is visible in
/// `cargo test` output even when the test passes.
fn visible(line: &str) {
    let mut err = std::io::stderr().lock();
    let _ = writeln!(err, "{line}");
}

/// The red-phase test: the tree at `commit` (the commit that added the
/// script) fails exactly `criteria`. Fails when the commit is missing,
/// unless `DOCCHECK_ALLOW_SHALLOW=1`, which prints a visible SKIPPED line.
/// Prints a visible EXECUTED line when it ran, so a CI log shows it.
pub fn red_phase(spec: &CheckSpec, commit: &str, criteria: &[&str]) {
    let root = repo_root();
    if !commit_exists(&root, commit) {
        if std::env::var(ALLOW_SHALLOW).is_ok_and(|v| v == "1") {
            visible(&format!(
                "SKIPPED red-phase {} at {commit}: the commit is not in this clone ({ALLOW_SHALLOW}=1)",
                spec.id
            ));
            return;
        }
        panic!(
            "red-phase {}: commit {commit} is not in this clone; fetch the full history \
             (CI: actions/checkout with fetch-depth: 0) or set {ALLOW_SHALLOW}=1 to skip visibly",
            spec.id
        );
    }
    let docs =
        Docs::at_commit(root, commit).unwrap_or_else(|e| panic!("red-phase {}: {e}", spec.id));
    let r = spec.evaluate(&docs);
    let failed: Vec<String> = r.failed_criteria().into_iter().collect();
    visible(&format!(
        "EXECUTED red-phase {} at {commit}: FAIL {}",
        spec.id,
        failed.join(",")
    ));
    assert_eq!(failed, criteria, "{}", r.render());
}

/// Runs the denial `name` of `spec` and panics, printing the report, unless
/// every expectation holds.
pub fn check_denial(spec: &CheckSpec, name: &str) {
    let d = spec
        .denials
        .iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("{}: no denial named {name}", spec.id));
    let docs = d.apply(repo()).unwrap_or_else(|e| panic!("{e}"));
    let r = spec.evaluate(&docs);
    let unmet = d.unmet(&r);
    assert!(
        unmet.is_empty(),
        "denial {name}: {}\nthe report:\n{}",
        unmet.join("; "),
        if r.passed() {
            "(no failure)".to_string()
        } else {
            r.render()
        }
    );
}

/// Panics unless the tested names and the spec's denial list are the same
/// set (no denial without a test, no test of a removed denial).
pub fn assert_all_denials_tested(spec: &CheckSpec, tested: &[&str]) {
    let mut names: Vec<&str> = spec.denials.iter().map(|d| d.name).collect();
    names.sort_unstable();
    let dup: Vec<_> = names
        .windows(2)
        .filter(|w| w[0] == w[1])
        .map(|w| w[0])
        .collect();
    assert!(dup.is_empty(), "{}: denials listed twice {dup:?}", spec.id);
    let listed: std::collections::BTreeSet<&str> = names.iter().copied().collect();
    let tested: std::collections::BTreeSet<&str> = tested.iter().copied().collect();
    let untested: Vec<_> = listed.difference(&tested).collect();
    let unknown: Vec<_> = tested.difference(&listed).collect();
    assert!(
        untested.is_empty() && unknown.is_empty(),
        "{}: denials without a test {untested:?}; tests without a denial {unknown:?}",
        spec.id
    );
}

/// One `#[test]` per denial of a check, plus a test that the list and the
/// tests agree one to one.
///
/// Every denial test is ignored in a debug build and runs in a release
/// build (`#[cfg_attr(debug_assertions, ignore)]`): the default debug
/// `cargo test` keeps the criterion, red-phase and library tests; the
/// release step `cargo test --release -p hadronyche-tools --test doc` runs
/// every denial, exactly once. A denial is never skipped: the agreement
/// test runs in both builds and fails on a denial without a test or a test
/// without a denial. A `slow:` list (backtracking inputs) is accepted and
/// runs in the same release step.
///
/// ```ignore
/// hadronyche_tools::denial_tests!(c004_27::SPEC; ac1_denial_no_probe_section, ac3_denial_fewer_than_four_forms);
/// ```
#[macro_export]
macro_rules! denial_tests {
    ($spec:path; $($name:ident),+ $(,)? ; slow: $($slow:ident),+ $(,)?) => {
        $crate::denial_tests!($spec; $($name),+ , $($slow),+);
    };
    ($spec:path; $($name:ident),+ $(,)?) => {
        $(
            #[test]
            #[cfg_attr(
                debug_assertions,
                ignore = "denial: run by the release step (cargo test --release -p hadronyche-tools --test doc)"
            )]
            fn $name() {
                $crate::doccheck::testing::check_denial(&$spec, stringify!($name));
            }
        )+
        #[test]
        fn every_denial_of_the_list_has_exactly_one_test() {
            $crate::doccheck::testing::assert_all_denials_tested(&$spec, &[$(stringify!($name)),+]);
        }
    };
}

/// Source tests (lead ruling R16). Clippy (`hadronyche-tools/clippy.toml`)
/// bans files, processes, the environment and every other document source
/// in the whole crate except the explicit allow-list; these textual rules
/// cover what clippy cannot see, on code with comments and string contents
/// blanked:
///
/// * a check module (`src/doccheck/checks/c<id>/`) or a check test file
///   (`tests/doc_<id>.rs`) uses no `include!`/`include_str!`/
///   `include_bytes!`, no `#[path]`, and does not opt out of the clippy rule
///   (`disallowed_methods`, `disallowed_types`);
/// * NO PORT IMPORTS ANOTHER PORT: `checks::c<other id>` appears in no
///   check module or test file but its own; shared code lives in the
///   library;
/// * a check module does not use `testing::`, `.parse` or `unwrap_or(0)`:
///   a digit capture goes through `pycompat::parse_int`;
/// * the library uses no `unwrap_or(0)`, and `.parse` only in the files
///   listed in `PARSE_ALLOWED` (none reads a digit capture of a document).
#[cfg(test)]
mod source_rules {
    use std::path::{Path, PathBuf};

    /// Library files that may call `.parse`: TOML text into the crate's
    /// parser (tomllib, wire_toml), already-validated digits inside the
    /// subset reader (tomlsubset), Rust's own float formatting (pyvalue),
    /// a replacement's group number (rx), Python's `float()` on validated
    /// ASCII (pynum).
    const PARSE_ALLOWED: &[&str] = &[
        "tomllib.rs",
        "wire_toml.rs",
        "tomlsubset.rs",
        "pyvalue.rs",
        "rx.rs",
        "pynum.rs",
    ];

    fn files(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).expect("readable").flatten() {
            let p = e.path();
            if p.is_dir() {
                files(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }

    /// The code of a Rust source: comments removed, string and char
    /// literal contents blanked (a `//` inside a string is not a comment).
    pub(super) fn code_of(src: &str) -> String {
        let c: Vec<char> = src.chars().collect();
        let (mut out, mut i) = (String::with_capacity(src.len()), 0);
        while i < c.len() {
            if c[i] == '/' && c.get(i + 1) == Some(&'/') {
                while i < c.len() && c[i] != '\n' {
                    i += 1;
                }
            } else if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                let mut depth = 0;
                while i < c.len() {
                    if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                        depth += 1;
                        i += 2;
                    } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                        depth -= 1;
                        i += 2;
                        if depth == 0 {
                            break;
                        }
                    } else {
                        i += 1;
                    }
                }
            } else if c[i] == 'r'
                && (c.get(i + 1) == Some(&'"') || c.get(i + 1) == Some(&'#'))
                && !c[..i]
                    .last()
                    .is_some_and(|p| p.is_alphanumeric() || *p == '_')
            {
                let mut j = i + 1;
                let mut hashes = 0;
                while c.get(j) == Some(&'#') {
                    hashes += 1;
                    j += 1;
                }
                if c.get(j) != Some(&'"') {
                    out.push(c[i]);
                    i += 1;
                    continue;
                }
                let close: String = std::iter::once('"')
                    .chain(std::iter::repeat_n('#', hashes))
                    .collect();
                let rest: String = c[j + 1..].iter().collect();
                let end = rest.find(&close).map_or(c.len(), |k| {
                    j + 1 + rest[..k].chars().count() + close.chars().count()
                });
                out.push_str("\"\"");
                i = end;
            } else if c[i] == '"' {
                i += 1;
                while i < c.len() && c[i] != '"' {
                    i += if c[i] == '\\' { 2 } else { 1 };
                }
                i += 1;
                out.push_str("\"\"");
            } else {
                out.push(c[i]);
                i += 1;
            }
        }
        out
    }

    fn own_module(path: &Path) -> Option<String> {
        let s = path.to_string_lossy();
        let r = regex::Regex::new(
            r"(?:checks|tests/doc)/(c\d+_\d+[a-z]?)|tests/doc_(\d+)_(\d+[a-z]?)\.rs",
        )
        .expect("pattern");
        let c = r.captures(&s)?;
        Some(c.get(1).map_or_else(
            || format!("c{}_{}", &c[2], &c[3]),
            |m| m.as_str().to_string(),
        ))
    }

    fn violations(path: &Path, code: &str, is_check: bool) -> Vec<String> {
        let mut out = Vec::new();
        let mut bad = vec![
            "include!",
            "include_str!",
            "include_bytes!",
            "#[path",
            "disallowed_methods",
            "disallowed_types",
            // R19: a check runs only through CheckSpec::evaluate, and never
            // sees or clears the regex failure record
            "run)(",
            "take_failures",
            "restore_failures",
            "take_int_limits",
            "restore_int_limits",
        ];
        if is_check {
            // whitespace-stripped: `.parse :: <usize>()`, `unwrap_or( 0 )`
            bad.extend([
                ".parse(",
                "parse::<",
                "from_str(",
                "unwrap_or(0)",
                ".evaluate(",
                // R19 holes of review round 4: a regex run on another
                // thread, or a result cached across evaluations, escapes
                // the per-thread failure record
                "thread::",
                "OnceLock",
                "LazyLock",
                "OnceCell",
                "thread_local!",
                "Mutex",
                "RwLock",
                "atomic::",
                // R21: no silent saturation or wrapping on document numbers
                // (doccheck::checked records INT_LIMIT instead)
                "saturating_",
                "wrapping_",
            ]);
        }
        let flat: String = code.chars().filter(|c| !c.is_whitespace()).collect();
        for b in bad {
            if flat.contains(b) {
                out.push(format!("{}: uses `{b}`", path.display()));
            }
        }
        if is_check
            && regex::Regex::new(r"(?:^|[^'A-Za-z0-9_])static\b")
                .expect("pattern")
                .is_match(code)
        {
            out.push(format!(
                "{}: declares a `static` (a check keeps no state across evaluations, R19)",
                path.display()
            ));
        }
        if is_check
            && regex::Regex::new(r"\bas\s+(?:i|u|f)(?:8|16|32|64|128|size)\b")
                .expect("pattern")
                .is_match(code)
        {
            out.push(format!(
                "{}: a numeric `as` cast (R21: use doccheck::checked or parse_int)",
                path.display()
            ));
        }
        if is_check {
            // an aliased import (`use … testing as t`) still names `testing`
            for (word, why) in [
                ("testing", "the test helpers are not for checks"),
                (
                    "rx_raw",
                    "a port uses Python's semantics (text::rx), never rx_raw",
                ),
                ("thread", "a check runs on the evaluating thread (R19)"),
            ] {
                if regex::Regex::new(&format!(r"\b{word}\b"))
                    .expect("pattern")
                    .is_match(code)
                {
                    out.push(format!("{}: names `{word}` ({why})", path.display()));
                }
            }
        }
        let own = own_module(path);
        // any path to another port: `checks::c004_27::`, `super::c004_27::`,
        // `super::super::c004_27::`, `crate::…::c004_27::`
        let port = regex::Regex::new(r"\b(c\d+_\d+[a-z]?)\s*::").expect("pattern");
        for c in port.captures_iter(code) {
            if Some(&c[1]) != own.as_deref() {
                out.push(format!(
                    "{}: imports another port, `{}::` (shared code belongs in the library)",
                    path.display(),
                    &c[1]
                ));
            }
        }
        out
    }

    /// The string literal (plain or raw) at the start of `s`, unescaped
    /// enough for a pattern (`\\` and `\"`), or `None`.
    fn literal_at(s: &str) -> Option<String> {
        let s = s.trim_start();
        if let Some(rest) = s.strip_prefix('r') {
            let hashes = rest.chars().take_while(|c| *c == '#').count();
            let body = rest[hashes..].strip_prefix('"')?;
            let close = format!("\"{}", "#".repeat(hashes));
            return body.find(&close).map(|e| body[..e].to_string());
        }
        let body = s.strip_prefix('"')?;
        let (mut out, mut chars) = (String::new(), body.chars());
        while let Some(c) = chars.next() {
            match c {
                '"' => return Some(out),
                '\\' => match chars.next()? {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    other => out.push(other),
                },
                c => out.push(c),
            }
        }
        None
    }

    /// Every literal pattern passed to the pattern helpers in `src`.
    fn literal_patterns(src: &str) -> Vec<String> {
        let call = regex::Regex::new(r"\b(?:rx|has|has_i|starts|full)\(").expect("pattern");
        call.find_iter(src)
            .filter_map(|m| literal_at(&src[m.end()..]))
            .collect()
    }

    /// R19: a pattern with a refused construct (`\Z`, `{,n}`,
    /// back-references, look-behind) or that does not compile fails here,
    /// at test time, for every literal pattern of the ports and the library
    /// (a computed pattern that fails turns its check red, `REGEX_LIMIT`).
    #[test]
    fn every_literal_pattern_builds() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/doccheck");
        let mut all = Vec::new();
        files(&root, &mut all);
        let (mut seen, mut hits) = (0, Vec::new());
        // testing.rs holds this test's own refused example
        for p in all.iter().filter(|p| !p.ends_with("testing.rs")) {
            let src = std::fs::read_to_string(p).expect("readable");
            for pat in literal_patterns(&src) {
                seen += 1;
                if let Some(e) = crate::doccheck::text::rx(&pat).compile_error() {
                    hits.push(format!("{}: {pat:?}: {e}", p.display()));
                }
            }
        }
        assert!(seen > 100, "the scan found only {seen} patterns");
        assert!(
            hits.is_empty(),
            "patterns that do not build:\n{}",
            hits.join("\n")
        );
        assert_eq!(
            literal_patterns(r#"has(r"a\Z", t)"#),
            vec![r"a\Z".to_string()]
        );
        assert!(crate::doccheck::text::rx(r"a\Z").compile_error().is_some());
    }

    /// R19: the only caller of a check's `run` is `CheckSpec::evaluate`
    /// (docs.rs), and only rx.rs and docs.rs touch the failure record.
    #[test]
    fn no_code_runs_a_check_around_evaluate() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut all = Vec::new();
        files(&root.join("src"), &mut all);
        files(&root.join("tests"), &mut all);
        let mut hits = Vec::new();
        for p in &all {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if matches!(
                name.as_str(),
                "docs.rs" | "rx.rs" | "pynum.rs" | "testing.rs" | "checked.rs"
            ) || p.ends_with("pynum/tests.rs")
            {
                continue;
            }
            let flat: String = code_of(&std::fs::read_to_string(p).expect("readable"))
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            for b in [
                "run)(",
                "take_failures",
                "restore_failures",
                "take_int_limits",
                "restore_int_limits",
            ] {
                if flat.contains(b) {
                    hits.push(format!("{}: `{b}`", p.display()));
                }
            }
        }
        assert!(hits.is_empty(), "{}", hits.join("\n"));
    }

    #[test]
    fn check_modules_and_test_files_follow_the_source_rules() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut checks = Vec::new();
        files(&root.join("src/doccheck/checks"), &mut checks);
        checks.retain(|p| own_module(p).is_some());
        let mut tests = Vec::new();
        files(&root.join("tests"), &mut tests);
        tests.retain(|p| own_module(p).is_some());
        assert!(
            checks.len() >= 2 && tests.len() >= 2,
            "the scan found no port"
        );
        let mut hits = Vec::new();
        for (p, is_check) in checks
            .iter()
            .map(|p| (p, true))
            .chain(tests.iter().map(|p| (p, false)))
        {
            hits.extend(violations(
                p,
                &code_of(&std::fs::read_to_string(p).expect("readable")),
                is_check,
            ));
        }
        assert!(hits.is_empty(), "source rules broken:\n{}", hits.join("\n"));
    }

    #[test]
    fn the_library_reads_digits_through_parse_int() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/doccheck");
        let mut lib = Vec::new();
        files(&root, &mut lib);
        let mut hits = Vec::new();
        for p in lib
            .iter()
            .filter(|p| own_module(p).is_none() && !p.to_string_lossy().contains("/equiv/"))
        {
            let code = code_of(&std::fs::read_to_string(p).expect("readable"));
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if code.contains("unwrap_or(0)") {
                hits.push(format!("{}: unwrap_or(0)", p.display()));
            }
            if (code.contains(".parse(") || code.contains(".parse::<"))
                && !PARSE_ALLOWED.contains(&name.as_str())
                && name != "testing.rs"
            {
                hits.push(format!(
                    "{}: .parse (route digits through pycompat::parse_int)",
                    p.display()
                ));
            }
        }
        assert!(hits.is_empty(), "{}", hits.join("\n"));
    }

    #[test]
    fn the_rules_see_through_strings_and_catch_imports() {
        let code = code_of(
            "let a = \"// not a comment\"; std::fs::read(x); // include!(\"x\")\n/* checks::c004_27 */ r#\"include!\"#;",
        );
        assert!(
            code.contains("std::fs::read")
                && !code.contains("include!")
                && !code.contains("c004_27")
        );
        let p = Path::new("src/doccheck/checks/c004_22/x.rs");
        assert!(
            violations(p, "use crate::doccheck::checks::c004_27::Field;", true)
                .iter()
                .any(|v| v.contains("another port"))
        );
        assert!(violations(p, "use crate::doccheck::checks::c004_22::X;", true).is_empty());
        assert!(
            violations(
                p,
                "const N: &'static str = \"x\"; fn f(s: &'static str) {}",
                true
            )
            .is_empty(),
            "a 'static lifetime is not a static"
        );
        assert!(!violations(p, "let n = x.parse::<i64>();", true).is_empty());
        for bypass in [
            "use super::super::c004_27::SPEC;",
            "use super::c004_27::Field;",
            "use crate::doccheck::testing as t; t::repo();",
            "let n = str::parse::<u32>(\"7\");",
            "let n = u32::from_str(\"7\");",
            "let n = \"7\".parse :: <usize>();",
            "let n = x.unwrap_or( 0 );",
            "use crate::doccheck::text::rx_raw as rx;",
            "let r = (SPEC.run)(docs);",
            "let f = crate::doccheck::rx::take_failures();",
            "let f = crate::doccheck::pynum::take_int_limits();",
            // review round 4, item 1: the two green-with-failure cases
            "let h = std::thread::spawn(move || has(PAT, &t)); h.join();",
            "use std::thread as th; th::scope(|s| { s.spawn(|| has(PAT, &t)); });",
            "static SEEN: OnceLock<bool> = OnceLock::new();\nfn f() -> bool { *SEEN.get_or_init(|| has(PAT, &t)) }",
            "    pub(crate) static CACHE: std::sync::Mutex<Vec<u8>> = Mutex::new(Vec::new());",
            "thread_local! { static C: Cell<u8> = Cell::new(0); }",
            // review round 5, finding 1: a static inside a function body
            "pub fn cached(t: &str) -> bool { static mut K: Option<bool> = None; unsafe { K.unwrap_or(false) } }",
            // review round 5, finding 2 (R21)
            "let s = w.saturating_mul(8);",
            "let s = w.wrapping_add(1);",
            "let n = x as i64;",
            "let n = (x) as usize;",
        ] {
            assert!(!violations(p, bypass, true).is_empty(), "missed: {bypass}");
        }
        assert!(
            !violations(
                Path::new("tests/doc_004_22.rs"),
                "include_str!(\"a\")",
                false
            )
            .is_empty()
        );
    }
}
