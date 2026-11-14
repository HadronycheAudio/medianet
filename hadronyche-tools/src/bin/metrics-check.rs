//! `metrics-check` — run the ticket 002-09 experiment-metrics integrity
//! checker from the command line (read-only; the checker writes nothing).
//!
//! Usage: `cargo run --release --bin metrics-check -- <directory-or-file>`
//!
//! With a directory, prints [`hadronyche_tools::metrics_check::run`]'s
//! report: one line per metrics file found under the directory (recursively,
//! sorted by path), in the line format documented in
//! [`hadronyche_tools::metrics_check`] (C-206):
//!
//! ```text
//! <path>: single-writer <yes|no> (lines <total>, parseable <parseable>, unix_ms violations <count>)
//! ```
//!
//! With a single file, prints the same line for that file. The exit status
//! is 0 either way — a verdict is a finding, not a failure; a missing or
//! unreadable path exits 1. The captured output for the POC's `experiments/`
//! is recorded in section (c) of `doc/design/reports/archaeology/REPORT-0b.md`.

use std::path::PathBuf;

use hadronyche_tools::metrics_check::{check_bytes, render_line, run};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!("usage: metrics-check <directory-or-file>");
        std::process::exit(2);
    }
    let path = PathBuf::from(&args[0]);
    if path.is_dir() {
        print!("{}", run(&path));
        return;
    }
    let content = std::fs::read(&path).unwrap_or_else(|err| {
        eprintln!("metrics-check: cannot read {}: {err}", path.display());
        std::process::exit(1);
    });
    let mut report = check_bytes(&content);
    report.path = path;
    println!("{}", render_line(&report));
}
