# 027-01 — Crate skeleton and skew types
**Epic:** 027-L3-cross-sink-rig · **Effort:** XS · **Depends on:** epic 020

## Context
- Epic 027 builds the L3 cross-sink rig: the certified Linux ALSA reference sink and the embedded reference device play the same marker stream; both analogue outputs go into two channels of one capture device; the tool measures the skew between the channels. L3 is the top rung of the certification ladder `[decision]` (L0 loopback, L1 two machines, L2 one reference sink per platform alone, L3 a hosted sink and an embedded sink together).
- All L3 analysis code lives in crate `hadronyche-tools`, binary `hadronyche-l3`, in the design set's repository (C-133). Modules: `skew.rs` (marker location and skew series), `run.rs` (scenarios), `stamp.rs` (regime stamp fetch, ticket 027-22), `report.rs` (statistics and pass/fail).
- This ticket only creates the binary, the modules and the public types the epic fixes, so the following tickets can be developed in parallel.
- Sign convention: `skew_ns` is **channel 2 minus channel 1**. Channel 1 carries the Linux sink output, channel 2 the embedded device output, so a positive skew means the device emits later.
- `MarkerPlan` (the description of where markers sit in the stream) is provided by the marker analysis code of epic 020 / the marker stream of epic 017; this ticket imports it, it does not redefine it.
- Pass rule fixed by C-79: `pass` is `p999_abs_ns <= limit_ns`, with the maximum reported beside it.

## Work
1. In crate `hadronyche-tools`, add binary target `hadronyche-l3` (`src/bin/hadronyche-l3/main.rs` or the equivalent layout used by the crate) with modules `skew`, `run`, `stamp`, `report`.
2. In `skew.rs` declare, exactly:
   ```rust
   pub struct SkewSample { pub position: u64, pub skew_ns: i64 }   // channel 2 minus channel 1
   pub fn measure_skew(capture: &[[f32; 2]], rate: u32, markers: &MarkerPlan) -> Vec<SkewSample>;
   ```
   The `measure_skew` body is `unimplemented!()` with a clear message until 027-03.
3. In `report.rs` declare, exactly:
   ```rust
   pub struct SkewReport { pub p50_ns: i64, pub p999_abs_ns: i64, pub max_abs_ns: i64, pub limit_ns: i64, pub pass: bool }
   ```
4. In `run.rs` declare the scenario names as an enum or CLI subcommands, with no behaviour yet: `steady`, `restart-linux`, `restart-device`, `power-cycle-device`, `handover`, `loss`, `interrupt`, `epoch`, `holdover`, `admission`.
5. The CLI prints usage listing the scenarios; unimplemented scenarios exit non-zero with "not implemented".
6. Doc comments on each type stating the sign convention and units (nanoseconds; `position` = timeline position of the marker), and on `SkewReport::pass` the C-79 rule.

## Acceptance criteria
1. `cargo build -p hadronyche-tools --bin hadronyche-l3` succeeds in CI.
2. `hadronyche-l3 --help` lists all 10 scenario names given above.
3. `SkewSample`, `measure_skew` and `SkewReport` signatures match the ones above character for character (field names, types, order).
4. A unit test constructs a `SkewSample` and a `SkewReport` (compile-level check of the public API).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Public types documented with the sign convention (channel 2 minus channel 1, device minus Linux)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched are complete and documented" — no stamp field; "Design documents updated where behaviour differs" — no design behaviour; "No new open item without an owner and a falsifier" — opens none; "Every new factual claim carries an evidence tag" — makes no factual claim; "The skew analysis runs in CI on recorded two-channel files" — no analysis yet (027-08); "The rig is photographed or diagrammed" — no rig work; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Marker detection (027-03), capture file reading (027-02), statistics (027-05, 027-06), stamp fetch (027-22), scenarios (027-11 onward).
- Defining `MarkerPlan` (epics 017 / 020).
