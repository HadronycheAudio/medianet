//! Metrics line-parser fuzz target, runnable for a fixed duration (C-95).
//!
//! Usage: `cargo run --bin metrics-fuzz -- --duration-secs 60`
//!
//! Mirrors `probes/linux-peer/src/bin/fuzz-decode.rs` (ticket 001-01): the
//! per-change budget is 60 s, the nightly budget 1 hour
//! (`--duration-secs 3600`), and the seed corpus is kept in
//! `fuzz-corpus/metrics/` in this crate.
//!
//! RED-phase stub (tester's contract): the coder implements
//! `metrics_check::duration_from_args` and `metrics_check::fuzz_run`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)] // clippy.toml allow-list: a command-line tool

use hadronyche_tools::metrics_check;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let duration = metrics_check::duration_from_args(&args).unwrap_or_else(|err| {
        eprintln!("metrics-fuzz: {err}");
        eprintln!("usage: metrics-fuzz [--duration-secs <n>]");
        std::process::exit(2);
    });

    let outcome = metrics_check::fuzz_run(duration);
    println!("{}", outcome.render());
    if outcome.crashes != 0 {
        eprintln!(
            "metrics-fuzz: the metrics line parser panicked on {} fuzzed input(s); failing (C-95)",
            outcome.crashes
        );
        std::process::exit(1);
    }
}
