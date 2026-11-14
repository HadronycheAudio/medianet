//! Payload-decoder fuzz target, runnable for a fixed duration (C-95).
//!
//! Usage: `cargo run --bin fuzz-decode -- --duration-secs 60`
//!
//! RED-phase skeleton: the coder wires `fuzz::duration_from_args` and
//! `fuzz::run` here.

fn main() {
    todo!("parse --duration-secs, run fuzz::run, print FuzzOutcome, fail on a crash")
}
