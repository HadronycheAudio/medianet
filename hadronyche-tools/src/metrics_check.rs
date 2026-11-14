//! Experiment-metrics integrity checker (ticket 002-09, POC archaeology (c)).
//!
//! # Status
//!
//! Implemented (GREEN phase). The frozen POC is read-only (C-130): the
//! checker only reads `experiments/` and writes nothing there; its own
//! output goes to stdout.
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
//! # Column types (the POC's row format at `module.rs:1292`)
//!
//! The POC formats every row with one fixed `format!` (`module.rs:1408`):
//! columns 3 (`name`) and 4 (`holder`) are names; columns 15
//! (`machine_cpu`), 17 (`psi_cpu1`), 22 (`ctxt_s`), 23 (`intr_s`), 28
//! (`turn_mean_us`), 29 (`turn_max_us`), 31 (`rtt_mean_us`) and 32
//! (`rtt_min_us`) are decimals (printed with `{:.1}`, `{:.2}` or `{:.0}`);
//! every other column is an integer. A line parses iff it has exactly
//! [`METRICS_FIELD_COUNT`] fields and each field is of its column's type
//! (a decimal column also accepts an integer, as `{:.0}` prints one).
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
//! tests require one line per metrics file, in [`check_dir`] order, each
//! containing the file's path or name and its verdict; the text restates the
//! rule. [`run`] writes no files; it returns the text so the caller (CI, the
//! report step) chooses the destination.
//!
//! The line format, fixed here and used by both [`run`] and the
//! `metrics-check` binary (one line per metrics file, `\n`-terminated):
//!
//! ```text
//! <path>: single-writer <yes|no> (lines <total>, parseable <parseable>, unix_ms violations <count>)
//! ```
//!
//! `<yes|no>` is the single-writer verdict of the rule restated above; the
//! parenthesised counts are the raw evidence. The report file for this
//! ticket's table is `doc/design/reports/archaeology/REPORT-0b.md`, section
//! (c) — the only fixed file name.
//!
//! # Fuzz target (AC 2, C-95)
//!
//! `src/bin/metrics-fuzz.rs` runs the line parser ([`check_bytes`]) for a
//! fixed duration: [`FUZZ_SECONDS_PER_CHANGE`] (60 s) per change and 1 hour
//! nightly. Panics are counted as crashes and fail the run. The seed corpus
//! is kept in `fuzz-corpus/metrics/` with the checker, and the summary line
//! mentions the number of executions so a short CI run can prove it fuzzed.
//!
//! The read-only checker is also usable directly:
//!
//! ```text
//! cargo run --release --bin metrics-check -- <directory-or-file>
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Number of columns in the POC metrics header (`module.rs:1292`).
pub const METRICS_FIELD_COUNT: usize = 38;

/// The POC metrics header, byte for byte (`module.rs:1292`).
pub const METRICS_HEADER: &str = "unix_ms,net_ns,offset_ns,name,holder,holding,forced,lock,err_ns,drift_ppb,delay_ns,desync,loadavg1,loadavg5,cores,machine_cpu,psi_cpu10,psi_cpu1,psi_io10,psi_mem10,running,blocked,ctxt_s,intr_s,proc_cpu,late_clock_us,late_rt_us,peak_us,turn_mean_us,turn_max_us,served,rtt_mean_us,rtt_min_us,asked,datagrams,connections,band,points";

/// Columns the POC prints as decimals (`module.rs:1408`: `{:.1}`, `{:.2}`
/// or `{:.0}`); a decimal column also accepts an integer, as `{:.0}` prints
/// one. Every other column except the two names is an integer.
const DECIMAL_COLUMNS: [usize; 8] = [15, 17, 22, 23, 28, 29, 31, 32];

/// Columns the POC prints as names: `name` and `holder`.
const NAME_COLUMNS: [usize; 2] = [3, 4];

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

/// Whether one field is of the type the POC writes at this column.
fn field_parses(column: usize, field: &str) -> bool {
    if field.is_empty() {
        return false;
    }
    if NAME_COLUMNS.contains(&column) {
        return true;
    }
    if DECIMAL_COLUMNS.contains(&column) {
        return field.parse::<f64>().is_ok();
    }
    field.parse::<i128>().is_ok()
}

/// Split file content into lines: a line ends at `\n`, a trailing final
/// newline does not create an empty line, and a last line without a newline
/// still counts.
fn split_lines(content: &[u8]) -> Vec<&[u8]> {
    let body = if content.last() == Some(&b'\n') {
        &content[..content.len() - 1]
    } else {
        content
    };
    if body.is_empty() {
        return Vec::new();
    }
    body.split(|byte| *byte == b'\n').collect()
}

/// Drop one trailing `\r`, so CRLF line endings are accepted.
fn without_cr(line: &[u8]) -> &[u8] {
    match line.split_last() {
        Some((b'\r', rest)) => rest,
        _ => line,
    }
}

/// Check one file's bytes. The single-file core, reusable by tests and the
/// fuzzer; never panics, whatever the bytes.
pub fn check_bytes(content: &[u8]) -> FileReport {
    let mut report = FileReport {
        path: PathBuf::new(),
        total_lines: 0,
        parseable_lines: 0,
        unix_ms_violations: 0,
        single_writer: false,
    };
    let lines = split_lines(content);
    report.total_lines = lines.len();
    let mut header_occurrences = 0usize;
    let mut first_line_is_header = false;
    let mut previous_unix_ms: Option<i128> = None;
    let mut every_data_line_parses = true;
    for (index, raw) in lines.iter().enumerate() {
        let line = without_cr(raw);
        // A non-UTF-8 line is not the header and does not parse; from_utf8
        // never panics and the lossless fallback keeps the count stable.
        let text = std::str::from_utf8(line).unwrap_or("");
        if text == METRICS_HEADER {
            header_occurrences += 1;
            first_line_is_header |= index == 0;
            continue;
        }
        let fields: Vec<&str> = text.split(',').collect();
        let parseable = fields.len() == METRICS_FIELD_COUNT
            && fields
                .iter()
                .enumerate()
                .all(|(column, field)| field_parses(column, field));
        if parseable {
            report.parseable_lines += 1;
            if let Ok(unix_ms) = fields[0].parse::<i128>() {
                if let Some(previous) = previous_unix_ms
                    && unix_ms < previous
                {
                    report.unix_ms_violations += 1;
                }
                previous_unix_ms = Some(unix_ms);
            }
        } else {
            every_data_line_parses = false;
        }
    }
    report.single_writer = header_occurrences == 1
        && first_line_is_header
        && every_data_line_parses
        && report.unix_ms_violations == 0;
    report
}

/// Whether a file's first line (CR stripped) is exactly [`METRICS_HEADER`].
fn first_line_is_header(content: &[u8]) -> bool {
    let end = content
        .iter()
        .position(|byte| *byte == b'\n')
        .unwrap_or(content.len());
    without_cr(&content[..end]) == METRICS_HEADER.as_bytes()
}

/// Walk `dir` recursively, appending a report for every regular metrics
/// file. Unreadable entries are skipped: the checker never fails a run over
/// one unreadable file, it just does not report what it could not read.
fn visit(dir: &Path, reports: &mut Vec<FileReport>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            visit(&path, reports);
        } else if file_type.is_file()
            && let Ok(content) = fs::read(&path)
            && first_line_is_header(&content)
        {
            let mut report = check_bytes(&content);
            report.path = path;
            reports.push(report);
        }
    }
}

/// Check every metrics file under `dir`, recursively, sorted by path.
///
/// A metrics file is a regular file whose first line is exactly
/// [`METRICS_HEADER`]; `runs.csv`, logs and analysis files are not reported.
pub fn check_dir(dir: &Path) -> Vec<FileReport> {
    let mut reports = Vec::new();
    visit(dir, &mut reports);
    reports.sort_by(|left, right| left.path.cmp(&right.path));
    reports
}

/// Render one file's report in the line format documented in the module
/// docs (C-206): the path, the single-writer verdict, the counts.
pub fn render_line(report: &FileReport) -> String {
    let verdict = if report.single_writer { "yes" } else { "no" };
    format!(
        "{}: single-writer {} (lines {}, parseable {}, unix_ms violations {})",
        report.path.display(),
        verdict,
        report.total_lines,
        report.parseable_lines,
        report.unix_ms_violations,
    )
}

/// Render the human-readable report for `dir`: one line per metrics file,
/// each naming the file and stating its single-writer verdict (C-206 leaves
/// the exact wording to the coder; see the module docs).
pub fn run(dir: &Path) -> String {
    let mut text = String::new();
    for report in check_dir(dir) {
        text.push_str(&render_line(&report));
        text.push('\n');
    }
    text
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
        format!(
            "fuzz: {} executions in {:.3} s, {} crashes",
            self.executions,
            self.elapsed.as_secs_f64(),
            self.crashes,
        )
    }
}

/// A tiny deterministic PRNG (xorshift64*), as in `probes/linux-peer`, so
/// runs are reproducible without pulling in a dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

/// The committed seed corpus (`fuzz-corpus/metrics/`, C-95), sorted for
/// determinism. An unreadable corpus yields an empty list: the fuzzer then
/// feeds generated noise only.
fn seed_corpus() -> Vec<Vec<u8>> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz-corpus")
        .join("metrics");
    let mut seeds: Vec<Vec<u8>> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter_map(|path| fs::read(path).ok())
        .collect();
    seeds.sort();
    seeds
}

/// Feed the line parser generated and mutated bytes for `duration`, counting
/// panics as crashes. Seeds come from `fuzz-corpus/metrics/`, which is kept
/// with the checker (C-95).
pub fn fuzz_run(duration: Duration) -> FuzzOutcome {
    let seed = seed_corpus();
    let start = std::time::Instant::now();
    // Seed from the wall clock; the mutations stay deterministic within a
    // run thanks to the xorshift state.
    let mut rng = Rng(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15)
            | 1,
    );
    let mut executions: u64 = 0;
    let mut crashes: u64 = 0;

    while start.elapsed() < duration {
        // Build one input: mostly a mutated seed, sometimes fresh noise.
        let mut buf = if rng.below(4) == 0 || seed.is_empty() {
            let len = rng.below(2_500);
            (0..len).map(|_| (rng.next() & 0xff) as u8).collect::<Vec<u8>>()
        } else {
            let mut buf = seed[rng.below(seed.len())].clone();
            // A few mutations per input: truncation, byte pokes, bit flips,
            // insertions.
            let mutations = 1 + rng.below(8);
            for _ in 0..mutations {
                match rng.below(4) {
                    0 => {
                        let len = rng.below(buf.len() + 1);
                        buf.truncate(len);
                    }
                    1 if !buf.is_empty() => {
                        let index = rng.below(buf.len());
                        buf[index] = (rng.next() & 0xff) as u8;
                    }
                    2 if !buf.is_empty() => {
                        let index = rng.below(buf.len());
                        buf[index] ^= 1 << rng.below(8);
                    }
                    _ => {
                        let index = rng.below(buf.len() + 1);
                        buf.insert(index, (rng.next() & 0xff) as u8);
                    }
                }
            }
            buf
        };
        if buf.len() > 4_096 {
            buf.truncate(4_096);
        }

        executions += 1;
        // The parser must never panic, whatever the bytes.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = check_bytes(&buf);
        }));
        if result.is_err() {
            crashes += 1;
        }
    }

    FuzzOutcome {
        executions,
        elapsed: start.elapsed(),
        crashes,
    }
}

/// Parse `--duration-secs <n>` (also `--duration-secs=<n>`); default
/// [`FUZZ_SECONDS_PER_CHANGE`] (C-95).
pub fn duration_from_args<I, S>(args: I) -> Result<Duration, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    const FLAG: &str = "--duration-secs";
    let mut args = args.into_iter().peekable();
    let mut duration = Duration::from_secs(FUZZ_SECONDS_PER_CHANGE);
    while let Some(arg) = args.next() {
        let arg = arg.as_ref().to_string();
        let arg = arg.as_str();
        let value: Option<String> = if arg == FLAG {
            args.next().map(|value| value.as_ref().to_string())
        } else {
            arg.strip_prefix(FLAG)
                .and_then(|rest| rest.strip_prefix('='))
                .map(str::to_string)
        };
        let Some(value) = value else { continue };
        let secs: u64 = value
            .parse()
            .map_err(|_| format!("{FLAG}: {value:?} is not a number of seconds"))?;
        duration = Duration::from_secs(secs);
    }
    Ok(duration)
}
