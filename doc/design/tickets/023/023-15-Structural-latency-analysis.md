# 023-15 — Structural latency analysis
**Epic:** 023-DSL-compiler-front-end · **Effort:** M · **Depends on:** 023-11, 023-13

## Context
- `hadronyche-dsl/analysis/latency.rs` — structural L per output; flags non-constant delay; path latencies at each join.
- What the compiler owes the runtime `[decision]`: **structural latency L as a compile-time constant**: explicit delays, lookahead, block buffering and linear-phase filter centres. Recursive filters have frequency-dependent delay; the compiler reports that as a *property*, not a number. The runtime stamps a module's output at position p − L while it processes input p.
- Unit (C-105, decided): **always a whole number of domain-rate samples** in the manifest and ABI; the earlier rational-L rule is withdrawn, so no rational arithmetic or rounding is needed. Every network port is at the domain rate (C-01); inside an edge module, latencies on the device-facing side are converted to domain-rate samples (a 48 kHz sample is 2 domain-rate samples, so the result stays whole).
- Resampler delay (C-134, C-207): an edge's conversion delay is counted only in its plumbing delay D, never in L; L is the module's structural delay in the domain-rate timeline. This analysis reports it per device-facing port, in whole domain-rate samples, and leaves it out of every L.
- Manifest shape: `pub struct Samples(pub u64)` and `pub struct OutputLatency { pub output: PortId, pub samples: Samples, pub non_constant_delay: bool }`, one per output.
- Join rule (C-104, decided): where paths of different latency meet, the shorter path is padded (pass in 023-28) and **the output's L is the longest path**. This ticket provides the per-path latencies at every join that 023-28 needs.
- Per node: `Delay(n)` → n samples at its rate; lookahead N → N; linear-phase FIR length 2k+1 → k (an even-tap linear-phase FIR never reaches this analysis: it is rejected in 023-10, C-183); block buffer B → B; `Resample` → 0 toward L, its conversion delay from 023-11 reported separately; arithmetic, table lookup, linear ramp, mix → 0; feedback through `StateCell` forming a recursive filter → contributes its non-recursive structural part and sets `non_constant_delay`; a path through such a filter is marked frequency-dependent in `join_paths` so 023-28 can flag the join (C-104).
- 023-AC-2 (contributes to AC-33): over the ≥ 20-program corpus of 023-13, reported L equals the hand-derived value **exactly**; a recursive-filter program reports its structural L and sets `non_constant_delay`.

## Work
1. `analysis::latency::analyse(&Graph) -> Vec<OutputLatency>` using checked integer arithmetic (overflow → diagnostic).
2. Per-node latency function; path accumulation converting each segment to domain-rate samples; `analysis::latency::conversion_delays(&Graph) -> Vec<(PortId, Samples)>` for device-facing ports (C-134).
3. `analysis::latency::join_paths(&Graph) -> Vec<(JoinId, Vec<(InputEdge, Samples, frequency_dependent: bool)>)>` giving the latency of every path into every join; L per output = maximum over paths.
4. Detect recursive filters (cycles through `StateCell` that feed back arithmetic) and set `non_constant_delay`.
5. Fill the latency column of the node table.

## Acceptance criteria
1. All ≥ 20 corpus programs: reported L == expected L (whole domain-rate samples) for every output (0 mismatches).
2. The 96→48 and 48→96 edge programs report the conversion delay documented in 023-11 on their device-facing port, and their L excludes it (C-134, C-207).
3. For every corpus join, `join_paths` returns the hand-derived latency of each path, and L of the downstream output equals the longest.
4. Recursive-filter corpus program: `non_constant_delay == true` and `samples` equals its expected structural L; non-recursive programs: `non_constant_delay == false`.
5. Result unchanged under reformatting of any corpus program.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The closed node set is listed with, per node, its latency, cost and determinism contribution (latency column)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Inserting padding delays (023-28); verifying L against measurement (epic 025 probe suite, AC-33); output stamping at runtime (epics 024/025).
