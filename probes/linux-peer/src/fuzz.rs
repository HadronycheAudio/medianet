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
        format!(
            "fuzz: {} executions in {:.3} s, {} crashes",
            self.executions,
            self.elapsed.as_secs_f64(),
            self.crashes,
        )
    }
}

/// Seed corpus committed with the probe: the payloads the unit tests encode.
pub fn seed_corpus() -> Vec<Vec<u8>> {
    use crate::payload::{encode, HEADER_LEN, DEFAULT_UDP_PAYLOAD_LEN};
    vec![
        encode(0, 0, DEFAULT_UDP_PAYLOAD_LEN),
        encode(1_234, 5_000_000_123, DEFAULT_UDP_PAYLOAD_LEN),
        encode(u64::MAX, u64::MAX, DEFAULT_UDP_PAYLOAD_LEN),
        encode(7, 8, HEADER_LEN),
        encode(u64::MAX - 1, 1 << 32, HEADER_LEN),
        encode(0, 1, 64),
        encode(1, 2, 512),
    ]
}

/// A tiny deterministic PRNG (xorshift64*) so runs are reproducible without
/// pulling in a dependency.
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

/// Run the payload decoder over generated and mutated inputs for `duration`,
/// catching panics and counting them as crashes. Returns normally only when
/// the run completed; a nonzero [`FuzzOutcome::crashes`] is a failure.
pub fn run(duration: Duration, seed: &[Vec<u8>]) -> FuzzOutcome {
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
            let len = rng.below(1_400);
            (0..len).map(|_| (rng.next() & 0xff) as u8).collect::<Vec<u8>>()
        } else {
            let mut buf = seed[rng.below(seed.len())].clone();
            // A few mutations per input: truncation, byte pokes, bit flips.
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
        // The decoder must never panic, whatever the bytes.
        let result = std::panic::catch_unwind(|| crate::payload::decode(&buf));
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

/// Parse `--duration-secs <n>` from the command line; default 60 s (C-95).
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
            args.next().map(|v| v.as_ref().to_string())
        } else if let Some(rest) = arg.strip_prefix(FLAG).filter(|rest| rest.starts_with('=')) {
            Some(rest[1..].to_string())
        } else {
            continue;
        };
        let value = value.ok_or_else(|| format!("{FLAG} needs a number of seconds"))?;
        let secs: u64 = value
            .parse()
            .map_err(|_| format!("{FLAG}: {value:?} is not a number of seconds"))?;
        duration = Duration::from_secs(secs);
    }
    Ok(duration)
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
