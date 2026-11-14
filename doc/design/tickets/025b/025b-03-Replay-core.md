# 025b-03 — Replay core
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** M · **Depends on:** 025b-01
## Context
`hadronyche-replay` in crate `hadronyche-tools`: **record and replay**. Reads a recording of the slices delivered to a module (with gap flags) and the hashes of what it emitted; reruns the module's code offline; reports the excluded-window count and the first differing slice. Shape given by the epic (names may be refined):
```rust
pub enum ReplayOutcome {
    Identical { slices: u64, excluded_windows: u64 },
    Differs { first_slice: u64, output: usize, excluded_windows: u64 },
}
pub fn replay<M: Module>(recording: &Recording) -> ReplayOutcome;
```
Exact tier and loss `[decision]`: **windows of 1 024 samples touched by loss or by a degraded slice are excluded from exact comparison and counted.** The degraded flag follows the content: an output at p is degraded if its input at p − L was gapped (C-122), so the recorded degraded marks already sit at the shifted positions.

Applies to exact-class modules (`exact-fixed`; `exact-float` where the fingerprint matches). A recording whose header says `bounded` (C-123) is not compared bit for bit; replay reports it as not applicable.

Still open, owned by this ticket: **window alignment** (aligned to the domain timeline, or relative to the recording start). Falsifier: the same gaps give a different excluded count depending on where the recording started. This ticket chooses, implements and records the choice in `doc/design/spec/` (C-130).
## Work
1. `hadronyche-tools` library module `replay` implementing `replay<M: Module>`.
2. Feed recorded inputs and gap flags to a fresh instance, hash each output slice, compare with the recorded hash.
3. Compute excluded windows: 1 024-sample windows touched by a gapped input or a degraded output slice; skip their comparison; count them into `excluded_windows`.
4. On the first non-excluded mismatch, return `Differs { first_slice, output, excluded_windows }`.
5. Record the window-alignment choice in the spec.
## Acceptance criteria
1. Replaying a recording of a corpus gain module made by the same code returns `Identical` with the recorded slice count and `excluded_windows: 0`.
2. Corrupting the hash of slice 5 000 output 2 returns `Differs { first_slice: 5000, output: 2, .. }`.
3. A recording with a gap at slice 100 (48-frame slices) excludes exactly the 1 024-sample window(s) covering that slice, reports their count, and still returns `Identical` if a mismatch is planted only inside that window.
4. A recording whose header class is `bounded` is reported as not comparable, not as identical.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- CLI and docs (025b-04); the one-minute echo acceptance run (025b-05).
