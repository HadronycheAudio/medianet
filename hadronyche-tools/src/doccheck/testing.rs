//! Helpers for the ports' test files (`tests/doc_<id>.rs`).
//!
//! A check reads only the `Docs` it is given; a test file gets its documents
//! from here and nowhere else (the source scan of
//! `checks::tests::checks_read_only_their_docs` forbids `Docs::repo`,
//! `Docs::dir`, `at_commit`, processes, files and the environment in test
//! files too). This module is the explicit allow-list: the repository's
//! working tree for the criterion tests, and the red-phase tree.

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
