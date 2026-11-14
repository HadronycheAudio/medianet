//! Payload-decoder fuzz target, runnable for a fixed duration (C-95).
//!
//! Usage: `cargo run --bin fuzz-decode -- --duration-secs 60`
//!
//! Default budget: 60 s per change (C-95); nightly runs 1 hour
//! (`--duration-secs 3600`, wired in CI by ticket 001-15). The seed corpus
//! lives in `corpus/` in this crate and is kept.

use linux_peer::fuzz;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let duration = fuzz::duration_from_args(&args).unwrap_or_else(|err| {
        eprintln!("fuzz-decode: {err}");
        eprintln!("usage: fuzz-decode [--duration-secs <n>]");
        std::process::exit(2);
    });

    let outcome = fuzz::run(duration, &fuzz::seed_corpus());
    println!("{}", outcome.render());
    if outcome.crashes != 0 {
        eprintln!(
            "fuzz-decode: the payload decoder panicked on {} fuzzed input(s); failing (C-95)",
            outcome.crashes
        );
        std::process::exit(1);
    }
}
