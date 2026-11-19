//! Acceptance tests for ticket 002-09 — experiment metrics integrity.
//!
//! # The tester's contract (fixed here; the coder implements it)
//!
//! The tests drive the smallest API that covers the ticket's checker:
//!
//! ```ignore
//! hadronyche_tools::metrics_check::check_bytes(&[u8]) -> FileReport
//! hadronyche_tools::metrics_check::check_dir(&Path) -> Vec<FileReport>
//! hadronyche_tools::metrics_check::run(&Path) -> String
//! ```
//!
//! with `FileReport { path, total_lines, parseable_lines, unix_ms_violations,
//! single_writer }`, `METRICS_FIELD_COUNT == 38` and `METRICS_HEADER` the
//! byte-for-byte POC header at `module.rs:1292`. The line and count
//! conventions, the directory-scan rule and the restated single-writer rule
//! are documented in `src/metrics_check.rs`; this file pins them with the
//! ticket's three synthetic files (Work step 2).
//!
//! | fixture | ticket's case | expected verdict |
//! |---|---|---|
//! | `clean.csv` | clean: multi-line, monotonic, header once | single-writer |
//! | `two-writer.csv` | truncated/interleaved by two writers | not single-writer |
//! | `non-monotonic.csv` | `unix_ms` decreases twice | not single-writer, 2 violations |
//! | `header-only.csv` | edge: header, no rows | single-writer (vacuous) |
//!
//! AC 1 is covered by `check_dir_reports_every_metrics_file_once_and_sorted`:
//! exactly one report per metrics file, sorted, decoys (`runs.csv`, logs)
//! excluded. AC 3/4 are report-content criteria; C-206 leaves the wording to
//! the coder, so `run_lists_each_file_name_with_its_verdict` checks only that
//! every file is named with a recognisable single-writer verdict. AC 2's fuzz
//! obligation is covered by `fuzz_target_runs_one_second_without_crashing`
//! and `fuzz_corpus_is_kept_with_the_checker`, mirroring ticket 001-01's
//! fixed-duration fuzz target.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)] // clippy.toml allow-list: the metrics checker's own tests

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use hadronyche_tools::metrics_check::{
    FUZZ_SECONDS_PER_CHANGE, METRICS_FIELD_COUNT, METRICS_HEADER, check_bytes, check_dir,
    duration_from_args, run,
};

/// Path of one tiny fixture under `tests/fixtures/metrics/`.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("metrics")
        .join(name)
}

/// Bytes of one fixture; a missing fixture is a test bug, not a checker bug.
fn read(name: &str) -> Vec<u8> {
    fs::read(fixture(name)).unwrap_or_else(|err| panic!("cannot read fixture {name}: {err}"))
}

/// A scratch directory under the system temporary directory, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Scratch {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the wall clock is after 1970")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "medianet-002-09-{tag}-{}-{unique}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the scratch directory");
        Scratch(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Copy a fixture into the scratch directory under a new name.
fn place(scratch: &Scratch, fixture_name: &str, target: &str) {
    let dest = scratch.path().join(target);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).expect("create the fixture's parent directory");
    }
    fs::copy(fixture(fixture_name), &dest)
        .unwrap_or_else(|err| panic!("cannot place {fixture_name} as {target}: {err}"));
}

/// Whether a report line states the single-writer verdict. C-206 leaves the
/// exact wording to the coder, so this accepts any yes/no-ish verdict token
/// next to the rule's name; it does not fix the format.
fn states_single_writer_verdict(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let names_the_rule = lower.contains("single-writer") || lower.contains("single writer");
    let states_a_verdict = [
        "yes",
        "no",
        "true",
        "false",
        "intact",
        "corrupt",
        "clean",
        "broken",
        "pass",
        "fail",
        "ok",
        "violation",
    ]
    .iter()
    .any(|word| lower.contains(word));
    names_the_rule && states_a_verdict
}

// ---------------------------------------------------------------------------
// The header contract (ticket Work step 1: columns defined at module.rs:1292)
// ---------------------------------------------------------------------------

#[test]
fn metrics_header_matches_the_poc_definition() {
    assert_eq!(
        METRICS_FIELD_COUNT, 38,
        "the POC metrics header has 38 columns (module.rs:1292)"
    );
    assert_eq!(
        METRICS_HEADER.split(',').count(),
        METRICS_FIELD_COUNT,
        "METRICS_HEADER must have one name per column"
    );
    let clean = String::from_utf8(read("clean.csv")).expect("the clean fixture is UTF-8");
    assert_eq!(
        clean
            .lines()
            .next()
            .expect("the clean fixture has a header"),
        METRICS_HEADER,
        "fixtures must carry the byte-for-byte POC header"
    );
}

// ---------------------------------------------------------------------------
// Work step 2: the three synthetic files, plus the ticket's edge cases
// ---------------------------------------------------------------------------

#[test]
fn clean_file_is_single_writer() {
    let report = check_bytes(&read("clean.csv"));
    assert_eq!(report.total_lines, 6, "header plus five rows");
    assert_eq!(report.parseable_lines, 5, "every data row parses");
    assert_eq!(report.unix_ms_violations, 0, "unix_ms increases");
    assert!(
        report.single_writer,
        "a clean file with one header, five parseable rows and monotonic unix_ms is single-writer"
    );
}

#[test]
fn two_writer_file_is_not_single_writer() {
    let report = check_bytes(&read("two-writer.csv"));
    assert_eq!(report.total_lines, 5, "header plus four data lines");
    assert_eq!(
        report.parseable_lines, 2,
        "only the two whole rows survive the interleaving"
    );
    assert!(
        !report.single_writer,
        "a truncated/interleaved file must not be single-writer"
    );
    assert!(
        report.parseable_lines < report.total_lines - 1,
        "at least one line after the header must fail to parse"
    );
}

#[test]
fn non_monotonic_unix_ms_is_counted() {
    let report = check_bytes(&read("non-monotonic.csv"));
    assert_eq!(report.total_lines, 6, "header plus five rows");
    assert_eq!(report.parseable_lines, 5, "types and columns still parse");
    assert_eq!(
        report.unix_ms_violations, 2,
        "1000,2000,1500,1600,1400 decreases twice"
    );
    assert!(
        !report.single_writer,
        "a decreasing unix_ms breaks the single-writer rule"
    );
}

#[test]
fn header_only_file_is_vacuously_single_writer() {
    let report = check_bytes(&read("header-only.csv"));
    assert_eq!(report.total_lines, 1);
    assert_eq!(report.parseable_lines, 0);
    assert_eq!(report.unix_ms_violations, 0);
    assert!(
        report.single_writer,
        "one header and nothing after it satisfies the rule vacuously"
    );
}

#[test]
fn empty_file_has_no_header_and_is_not_single_writer() {
    let report = check_bytes(b"");
    assert_eq!(report.total_lines, 0);
    assert_eq!(report.parseable_lines, 0);
    assert_eq!(report.unix_ms_violations, 0);
    assert!(
        !report.single_writer,
        "the header does not appear exactly once"
    );
}

#[test]
fn duplicate_header_is_not_single_writer() {
    let mut content = read("clean.csv");
    content.extend_from_slice(METRICS_HEADER.as_bytes());
    content.push(b'\n');
    let report = check_bytes(&content);
    assert_eq!(report.total_lines, 7, "header, five rows, a second header");
    assert_eq!(report.parseable_lines, 5);
    assert!(
        !report.single_writer,
        "the header must appear exactly once, as the first line"
    );
}

#[test]
fn wrong_column_type_is_not_parseable() {
    let clean = String::from_utf8(read("clean.csv")).expect("the clean fixture is UTF-8");
    let first_data = clean
        .lines()
        .nth(1)
        .expect("the clean fixture has a data row");
    let unix_ms = first_data
        .split(',')
        .next()
        .expect("the row has a first column");
    let mangled = first_data.replacen(unix_ms, "not-a-number", 1);
    let content = format!("{METRICS_HEADER}\n{mangled}\n");
    let report = check_bytes(content.as_bytes());
    assert_eq!(report.total_lines, 2);
    assert_eq!(
        report.parseable_lines, 0,
        "a non-numeric unix_ms violates the column types at module.rs:1292"
    );
    assert!(!report.single_writer);
}

#[test]
fn a_line_without_a_trailing_newline_still_counts() {
    let mut content = read("clean.csv");
    assert_eq!(
        content.last(),
        Some(&b'\n'),
        "the fixture ends with a newline"
    );
    content.pop();
    let report = check_bytes(&content);
    assert_eq!(
        report.total_lines, 6,
        "dropping the final newline must not drop the last row"
    );
    assert_eq!(report.parseable_lines, 5);
    assert!(report.single_writer);
}

// ---------------------------------------------------------------------------
// AC 1: every metrics file under experiments/, once, with a stable order
// ---------------------------------------------------------------------------

#[test]
fn check_dir_reports_every_metrics_file_once_and_sorted() {
    let scratch = Scratch::new("dir");
    place(&scratch, "clean.csv", "clean.csv");
    place(&scratch, "header-only.csv", "header-only.csv");
    place(&scratch, "non-monotonic.csv", "nested/broken.csv");
    // Decoys the POC's experiments/ directories really hold: runs.csv, a log
    // and analysis text are not metrics files.
    fs::write(scratch.path().join("runs.csv"), "run,start,end\n1,0,10\n")
        .expect("write the runs.csv decoy");
    fs::write(scratch.path().join("sender.1.log"), "not a metrics file\n")
        .expect("write the log decoy");

    let reports = check_dir(scratch.path());
    assert_eq!(
        reports.len(),
        3,
        "exactly the three metrics files, decoys excluded"
    );
    let names: Vec<String> = reports
        .iter()
        .map(|report| {
            report
                .path
                .file_name()
                .expect("every report names a file")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        names.join(","),
        "clean.csv,header-only.csv,broken.csv",
        "reports are sorted by path; the nested file sorts last"
    );

    for report in &reports {
        let bytes = fs::read(&report.path).expect("read the reported file");
        let single = check_bytes(&bytes);
        assert_eq!(
            (
                report.total_lines,
                report.parseable_lines,
                report.unix_ms_violations,
                report.single_writer,
            ),
            (
                single.total_lines,
                single.parseable_lines,
                single.unix_ms_violations,
                single.single_writer,
            ),
            "check_dir must delegate each file to the one parser and verdict"
        );
    }
}

#[test]
fn check_dir_on_missing_directory_is_empty() {
    let scratch = Scratch::new("missing");
    let missing = scratch.path().join("does-not-exist");
    assert!(
        check_dir(&missing).is_empty(),
        "a missing directory has no metrics file to report"
    );
}

// ---------------------------------------------------------------------------
// AC 3/4: the report names every file and states its verdict (C-206 wording)
// ---------------------------------------------------------------------------

#[test]
fn run_lists_each_file_name_with_its_verdict() {
    let scratch = Scratch::new("run");
    place(&scratch, "clean.csv", "clean.csv");
    place(&scratch, "non-monotonic.csv", "broken.csv");

    let text = run(scratch.path());
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(
        lines.len(),
        2,
        "one report line per metrics file; C-206 leaves the wording open:\n{text}"
    );
    for name in ["clean.csv", "broken.csv"] {
        let line = lines
            .iter()
            .find(|line| line.contains(name))
            .unwrap_or_else(|| panic!("run() must name {name}:\n{text}"));
        assert!(
            states_single_writer_verdict(line),
            "run() must state the single-writer verdict for {name}:\n{line}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC 2 / C-95: the fuzz target and the corpus kept with the checker
// ---------------------------------------------------------------------------

#[test]
fn fuzz_budget_defaults_to_sixty_seconds() {
    assert_eq!(FUZZ_SECONDS_PER_CHANGE, 60, "C-95 per-change budget");
    let default =
        duration_from_args(Vec::<String>::new()).expect("the empty argument list uses the default");
    assert_eq!(default.as_secs(), FUZZ_SECONDS_PER_CHANGE);
    let explicit = duration_from_args(["--duration-secs", "1"]).expect("an explicit duration");
    assert_eq!(explicit.as_secs(), 1);
}

#[test]
fn fuzz_target_runs_one_second_without_crashing() {
    let exe = env!("CARGO_BIN_EXE_metrics-fuzz");
    let output = Command::new(exe)
        .args(["--duration-secs", "1"])
        .output()
        .unwrap_or_else(|err| panic!("cannot run {exe}: {err}"));
    assert!(
        output.status.success(),
        "metrics-fuzz failed: status {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    assert!(
        stdout.contains("execution"),
        "the summary must mention executions so the short CI run proves it fuzzed: {stdout}"
    );
}

#[test]
fn fuzz_corpus_is_kept_with_the_checker() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz-corpus")
        .join("metrics");
    let seeds: Vec<PathBuf> = fs::read_dir(&corpus)
        .unwrap_or_else(|err| panic!("fuzz-corpus/metrics/ must be committed (C-95): {err}"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    assert!(
        !seeds.is_empty(),
        "the corpus is kept with the checker (C-95)"
    );
}
