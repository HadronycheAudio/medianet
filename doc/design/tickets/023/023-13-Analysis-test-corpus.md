# 023-13 — Analysis test corpus with hand-derived values
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-10, 023-11

## Context
- 023-AC-2 (contributes to AC-33): for a corpus of **at least 20 programs** — pure gain, explicit delay of N, lookahead limiter, linear-phase FIR of length 2k+1, block buffer of B, resampler 96→48 in an edge module, compositions of these including at least one join of unequal paths — the reported structural L equals the hand-derived value exactly. A program with a recursive filter reports its structural L and sets `non_constant_delay`.
- 023-AC-3 (join padding, C-104) and 023-AC-4 (cost) use the same corpus.
- L is **always a whole number of domain-rate samples** (C-105); it counts explicit delays, lookahead, block buffering and linear-phase filter centres `[decision]`. A resampler's delay is an edge's conversion delay, counted only in D and never in L (C-134, C-207); it is recorded separately. A linear-phase FIR of length 2k+1 has centre delay k; a block buffer of B samples delays by B. Where paths of different latency meet, the shorter path is padded with a whole-sample delay line and L is the longest path; a path with frequency-dependent delay is padded on its structural part only and the join is flagged (C-104).
- Only edge modules may contain a resampler or a non-domain rate (C-01, C-134); the resampler exists only at 2:1 and 1:2 (C-105).

## Work
1. Create `hadronyche-dsl/tests/corpus/` with ≥ 20 programs, each with a sidecar file of expected values: per-output L (whole domain-rate samples), `non_constant_delay`, per-join inserted delay, opt-out or frequency-dependent flag, per device-facing port conversion delay (edge modules), and the hand derivation as a comment.
2. Cover: pure gain (L = 0); delay N for several N; lookahead limiter with lookahead N; FIR 2k+1 for several k; block buffer B for several B; resampler 96→48 and 48→96 in edge modules (conversion delay from 023-11, excluded from L); at least 6 compositions (series and parallel; outputs with different L); at least 2 joins of unequal paths, one with the opt-out; at least 1 join fed by a recursive-filter path (C-104 flag); at least 1 recursive filter (IIR) with `non_constant_delay = true`; at least 1 multi-output program.
3. A test harness that parses and lowers each program (`check()` pipeline up to IR) and exposes expected values to 023-15, 023-16 and 023-28.

## Acceptance criteria
1. Corpus contains ≥ 20 programs, all of which lower without diagnostics; float and fixed variants of one program count as one program (C-192).
2. Each of the 7 categories listed in 023-AC-2 appears at least once; compositions ≥ 6; joins of unequal paths ≥ 2 (≥ 1 opted out); join with a recursive-filter path ≥ 1; recursive filter ≥ 1.
3. Every expected L and inserted delay has a written derivation reviewed by a second engineer.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The latency analysis itself (023-15), join padding (023-28) and cost analysis (023-16).
