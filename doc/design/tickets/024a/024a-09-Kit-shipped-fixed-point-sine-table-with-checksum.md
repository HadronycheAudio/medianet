# 024a-09 — Kit shipped fixed-point sine table with checksum
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** 024a-05, 024a-07

## Context
POC lesson `[code]`: `src/music.rs:311` builds the fixed-point sine table at start with libm `f64::sin` and rounds it to i32. A libm differing by one unit in the last place could flip an entry, so that path is not strictly cross-machine deterministic. Rule `[decision]`: **ship tables as constants** (or generate them with add/multiply only) and pin them with a checksum test. Every shipped table has a checksum test and a note on how it was generated. Settled by the epic: table size, checksum algorithm and interpolation rule are chosen here and recorded in the generation note.

## Work
- In `hadronyche-kit/src/tables.rs`: `pub static SINE_Q1_31: [i32; N]` as a literal constant (choose N, e.g. the POC's size; record it).
- A generator binary `gen-tables` in the `hadronyche-tools` crate (std allowed; C-133) that produced the table; commit its source and the exact command. If it uses `f64::sin`, record the host, libm version and rounding rule; prefer an add/multiply-only generator.
- `pub const SINE_Q1_31_CHECKSUM: u64` (choose and name the hash, e.g. FNV-1a 64 over little-endian bytes) and a test that recomputes it.
- A fixed-point sine lookup function using the table and Q0.32 phase (e.g. `sine_q(phase: Q0_32) -> Q1_31`) with a documented index/interpolation rule.
- A note (doc comment at the table) stating size, format, generator, command and checksum.

## Acceptance criteria
1. The table is a compile-time constant; no code computes it at start.
2. The checksum test passes on the hosted target; changing any one entry by 1 makes it fail (tested via a modified copy).
3. Re-running the generator reproduces the committed table byte for byte.
4. `sine_q` at phases 0, 1/4, 1/2, 3/4 returns 0, MAX, 0, −MAX (or the documented values) exactly.
5. The generation note is present at the table definition.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every shipped table has a checksum test and a note on how it was generated
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Resampler coefficient tables (024a-11).
- Running the checksum on all CI targets (024a-14).
