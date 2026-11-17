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
    let r = (spec.run)(repo());
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
    let r = (spec.run)(&docs);
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
    let r = (spec.run)(&docs);
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
    let listed: std::collections::BTreeSet<&str> = spec.denials.iter().map(|d| d.name).collect();
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
/// tests agree:
///
/// ```ignore
/// hadronyche_tools::denial_tests!(c004_27::SPEC; ac1_denial_no_probe_section, ac3_denial_fewer_than_four_forms);
/// ```
#[macro_export]
macro_rules! denial_tests {
    ($spec:path; $($name:ident),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                $crate::doccheck::testing::check_denial(&$spec, stringify!($name));
            }
        )+
        #[test]
        fn every_denial_of_the_list_has_a_test() {
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
    /// a replacement's group number (rx).
    const PARSE_ALLOWED: &[&str] = &[
        "tomllib.rs",
        "wire_toml.rs",
        "tomlsubset.rs",
        "pyvalue.rs",
        "rx.rs",
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
        let r = regex::Regex::new(r"checks/(c\d+_\d+[a-z]?)|tests/doc_(\d+)_(\d+[a-z]?)\.rs")
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
        ];
        if is_check {
            bad.extend(["testing::", ".parse(", ".parse::<", "unwrap_or(0)"]);
        }
        for b in bad {
            if code.contains(b) {
                out.push(format!("{}: uses `{b}`", path.display()));
            }
        }
        let own = own_module(path);
        let port = regex::Regex::new(r"checks::(c\d+_\d+[a-z]?)").expect("pattern");
        for c in port.captures_iter(code) {
            if Some(&c[1]) != own.as_deref() {
                out.push(format!(
                    "{}: imports another port, `checks::{}` (shared code belongs in the library)",
                    path.display(),
                    &c[1]
                ));
            }
        }
        out
    }

    #[test]
    fn check_modules_and_test_files_follow_the_source_rules() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut checks = Vec::new();
        files(&root.join("src/doccheck/checks"), &mut checks);
        checks.retain(|p| own_module(p).is_some());
        let tests: Vec<PathBuf> = std::fs::read_dir(root.join("tests"))
            .expect("tests/")
            .flatten()
            .map(|e| e.path())
            .filter(|p| own_module(p).is_some())
            .collect();
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
        assert!(!violations(p, "let n = x.parse::<i64>();", true).is_empty());
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
