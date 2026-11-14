//! Experiment-metrics integrity checker (ticket 002-09, POC archaeology (c)).
//!
//! # Status
//!
//! RED-phase stub: the tester fixed the contract below and every function is
//! `todo!()`. The coder implements it; the frozen POC is read-only (C-130).
//!
//! # Single-writer rule (restated in every report, ticket Work step 1)
//!
//! A metrics file is *single-writer* exactly when all of these hold:
//!
//! 1. the metrics header [`METRICS_HEADER`] appears exactly once, as the
//!    first line of the file;
//! 2. every line after it parses against the header: exactly
//!    [`METRICS_FIELD_COUNT`] comma-separated fields, each of the type the
//!    POC writes (integers, decimals, names at the columns defined by
//!    `module.rs:1292` in the frozen POC);
//! 3. `unix_ms` (the first field) never decreases from one parseable line to
//!    the next; and
//! 4. no line is interleaved, i.e. no two writers' bytes share a line. Two
//!    handles writing from their own offset 0 show up as a field-count or
//!    type failure, or as a truncated row/line.
//!
//! A header-only file is vacuously single-writer (one header, nothing after
//! it). An empty file, a file without the header, a file with a second
//! header, and a file with any unparseable line are not single-writer.
//!
//! # Line and count conventions (tester's contract)
//!
//! - A line ends at `\n`; a trailing final newline does not create an empty
//!   line, while a last line without a newline still counts. A trailing
//!   `\r` (CRLF) is ignored.
//! - `total_lines` counts every line, header included.
//! - `parseable_lines` counts non-header lines that satisfy rule 2, wherever
//!   they appear. Header occurrences are not data lines.
//! - `unix_ms_violations` counts, among parseable lines in file order, each
//!   line whose `unix_ms` is strictly less than the previous parseable
//!   line's. Unparseable lines neither reset nor advance the comparison.
//! - `single_writer` is true iff header occurrences == 1, the first line is
//!   the header, every other line is parseable, and violations == 0.
//!
//! # Directory scanning
//!
//! [`check_dir`] walks `dir` recursively and treats a regular file as a
//! metrics file iff its first line (CR stripped) is exactly
//! [`METRICS_HEADER`]. The POC's `experiments/` directories also hold
//! `runs.csv`, logs and analysis text, which are therefore not reported.
//! Reports are sorted by path. A missing or unreadable directory yields an
//! empty vector: there is no file to report and no error to raise.
//!
//! [`check_bytes`] is the single-file core, shared by tests and the fuzzer;
//! it has no path, so its [`FileReport::path`] is empty.
//!
//! # Report text (C-206)
//!
//! The ticket leaves the checker's exact output line format and the report
//! file names inside `doc/design/reports/archaeology/` to the coder. The
//! tests therefore require only that [`run`] returns one line per metrics
//! file, in [`check_dir`] order, each containing the file's path or name and
//! its verdict. The verdict must be recognisable as the single-writer
//! verdict: the words `single-writer`/`single writer` plus a yes/no-ish
//! token. The text restates the rule. [`run`] writes no files; it returns
//! the text so the caller (CI, the report step) chooses the destination.
//!
//! # Fuzz target (AC 2, C-95)
//!
//! `src/bin/metrics-fuzz.rs` runs the line parser ([`check_bytes`]) for a
//! fixed duration: [`FUZZ_SECONDS_PER_CHANGE`] (60 s) per change and 1 hour
//! nightly. Panics are counted as crashes and fail the run. The seed corpus
//! is kept in `fuzz-corpus/metrics/` with the checker, and the summary line
//! mentions the number of executions so a short CI run can prove it fuzzed.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Number of columns in the POC metrics header (`module.rs:1292`).
pub const METRICS_FIELD_COUNT: usize = 38;

/// The POC metrics header, byte for byte (`module.rs:1292`).
pub const METRICS_HEADER: &str = "unix_ms,net_ns,offset_ns,name,holder,holding,forced,lock,err_ns,drift_ppb,delay_ns,desync,loadavg1,loadavg5,cores,machine_cpu,psi_cpu10,psi_cpu1,psi_io10,psi_mem10,running,blocked,ctxt_s,intr_s,proc_cpu,late_clock_us,late_rt_us,peak_us,turn_mean_us,turn_max_us,served,rtt_mean_us,rtt_min_us,asked,datagrams,connections,band,points";

/// C-95: per-change fuzz budget, in seconds.
pub const FUZZ_SECONDS_PER_CHANGE: u64 = 60;

/// What one metrics file looked like to the checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReport {
    /// The file inspected. Empty for [`check_bytes`], which has no path.
    pub path: PathBuf,
    /// Every line in the file, header included.
    pub total_lines: usize,
    /// Non-header lines that parse against the header.
    pub parseable_lines: usize,
    /// Parseable lines whose `unix_ms` is below the previous parseable line's.
    pub unix_ms_violations: usize,
    /// The single-writer verdict, by the rule restated in the module docs.
    pub single_writer: bool,
}

/// Check one file's bytes. The single-file core, reusable by tests and the
/// fuzzer; never panics, whatever the bytes.
pub fn check_bytes(_content: &[u8]) -> FileReport {
    todo!("002-09: the metrics line parser and verdict are the coder's implementation")
}

/// Check every metrics file under `dir`, recursively, sorted by path.
///
/// A metrics file is a regular file whose first line is exactly
/// [`METRICS_HEADER`]; `runs.csv`, logs and analysis files are not reported.
pub fn check_dir(_dir: &Path) -> Vec<FileReport> {
    todo!("002-09: directory discovery is the coder's implementation")
}

/// Render the human-readable report for `dir`: one line per metrics file,
/// each naming the file and stating its single-writer verdict (C-206 leaves
/// the exact wording to the coder; see the module docs).
pub fn run(_dir: &Path) -> String {
    todo!("002-09: the report text is the coder's implementation")
}

/// What one fuzz run did (same shape as `probes/linux-peer`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuzzOutcome {
    /// Number of inputs fed to the line parser.
    pub executions: u64,
    /// Wall-clock duration of the run.
    pub elapsed: Duration,
    /// Inputs that made the parser panic (must be 0).
    pub crashes: u64,
}

impl FuzzOutcome {
    /// Summary line printed by the fuzz target; must mention the number of
    /// executions so the short CI run can prove it actually fuzzed.
    pub fn render(&self) -> String {
        todo!("002-09: the fuzz summary line is the coder's implementation")
    }
}

/// Parse `--duration-secs <n>` (also `--duration-secs=<n>`); default
/// [`FUZZ_SECONDS_PER_CHANGE`] (C-95).
pub fn duration_from_args<I, S>(_args: I) -> Result<Duration, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    todo!("002-09: the fuzz duration flag is the coder's implementation")
}

/// Feed the line parser generated and mutated bytes for `duration`, counting
/// panics as crashes. Seeds come from `fuzz-corpus/metrics/`, which is kept
/// with the checker (C-95).
pub fn fuzz_run(_duration: Duration) -> FuzzOutcome {
    todo!("002-09: the fixed-duration fuzz runner is the coder's implementation")
}
