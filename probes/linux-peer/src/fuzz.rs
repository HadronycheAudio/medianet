//! Payload-decoder fuzz target (C-95, ticket 001-01 Work item 4).
//!
//! The target must be runnable for a fixed duration from the command line,
//! e.g. `cargo run --bin fuzz-decode -- --duration-secs 60`. The per-change
//! budget is 60 s and the nightly budget 1 hour; the CI wiring is 001-15.
//! Feeding the decoder arbitrary bytes must never panic.

use std::time::Duration;

/// C-95: per-change fuzz budget, 60 s, for every parser.
pub const FUZZ_SECONDS_PER_CHANGE: u64 = 60;

/// What one fuzz run did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuzzOutcome {
    /// Number of inputs fed to the decoder.
    pub executions: u64,
    /// Wall-clock duration of the run.
    pub elapsed: Duration,
    /// Inputs that made the decoder panic (must be 0).
    pub crashes: u64,
}

impl FuzzOutcome {
    /// Summary line printed by the fuzz target. Must mention the number of
    /// executions so the short CI run can prove it actually fuzzed.
    pub fn render(&self) -> String {
        todo!("render the fuzz outcome")
    }
}

/// Seed corpus committed with the probe: the payloads the unit tests encode.
pub fn seed_corpus() -> Vec<Vec<u8>> {
    todo!("seed corpus from the unit-test payloads")
}

/// Run the payload decoder over generated and mutated inputs for `duration`,
/// catching panics and counting them as crashes. Returns normally only when
/// the run completed; a nonzero [`FuzzOutcome::crashes`] is a failure.
pub fn run(duration: Duration, seed: &[Vec<u8>]) -> FuzzOutcome {
    let _ = (duration, seed);
    todo!("fuzz the payload decoder for a fixed duration")
}

/// Parse `--duration-secs <n>` from the command line; default 60 s (C-95).
pub fn duration_from_args<I, S>(args: I) -> Result<Duration, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let _ = args;
    todo!("parse --duration-secs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::decode;

    #[test]
    fn seeds_are_decodable_probe_payloads() {
        let seeds = seed_corpus();
        assert!(!seeds.is_empty(), "the seed corpus must be committed");
        for seed in seeds {
            assert!(
                decode(&seed).is_ok(),
                "seed corpus must contain valid payloads"
            );
        }
    }

    #[test]
    fn short_fuzz_run_completes_without_a_crash() {
        // The real budget is 60 s (C-95); the unit test uses a short fixed
        // duration so the per-change suite stays fast (AC 5).
        let outcome = run(Duration::from_millis(50), &seed_corpus());
        assert_eq!(outcome.crashes, 0, "decoder panicked on a fuzzed input");
        assert!(outcome.executions > 0, "the fuzz run must execute inputs");
        assert!(!outcome.render().is_empty());
    }

    #[test]
    fn duration_flag_defaults_to_the_c95_budget() {
        assert_eq!(FUZZ_SECONDS_PER_CHANGE, 60);
        assert_eq!(
            duration_from_args(Vec::<String>::new()).expect("default duration"),
            Duration::from_secs(FUZZ_SECONDS_PER_CHANGE)
        );
        assert_eq!(
            duration_from_args(["--duration-secs", "1"]).expect("explicit duration"),
            Duration::from_secs(1)
        );
    }
}
