# 023-23 — `check()` pipeline and 023-AC-1 rejection suite
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-05, 023-10, 023-19, 023-20, 023-28, 023-29, 023-31, 023-32

## Context
- Public entry point (names may be refined): `pub fn check(source: &str) -> Result<(Ir, Manifest), Vec<Diagnostic>>;`
- Pipeline: lex (023-03) → parse (023-04) → type/rate check (023-08) and edge-module check (023-29) → lower to IR with kit rejections (023-10) → latency (023-15) → join padding (023-28, C-104) → remaining analyses (023-14, 023-16 to 023-18) → manifest (023-19, 023-20) with the controller placement check of a single-stream source (023-31, C-181) and the controller member stream of a bundle source (023-32, C-205).
- The front end hands checked IR, per-output dependency sets and manifest to the code generator (epic 024) and the analyses to the conformance suite (epic 025).
- Rule (C-218, decided): **the DSL front end (epic 023) implements every compile-time rejection; the code generator (epic 024b) only asserts it never receives such a program; the conformance corpus (epic 025a) tests it.** So every 023-AC-1 case is rejected here by `check()`, none is left to 024b; the rejection programs of Work 2 are the cases 025a's corpus tests.
- 023-AC-1 — **rejections with a clear diagnostic** (span, stable code, message naming the rule): (a) an unbounded loop or recursion; (b) a call outside the kit (any transcendental); (c) allocation; (d) a cross-rate connection without the resampler node; (e) a network port at a rate other than the domain rate (C-01); (f) a resampler node outside an edge module (C-134); (g) a transport-authoring construct; (h) a port wider than 32 × 6 = 192 channels, or an explicit bundle layout with more than 32 members or a member wider than 6 channels (C-04, C-08); (i) a resampler ratio other than 2:1 or 1:2 (C-105); (j) a linear-phase FIR with an even number of taps, the diagnostic suggesting an odd-tap design (C-183); (k) a join on the device side of an edge module whose paths differ by an odd number of domain-rate samples, without the opt-out, the diagnostic suggesting the per-join opt-out (C-190); (l) a source with controllers whose audio needs more than 31 members (more than 186 channels), since the controller member counts towards the 32 of C-08 (C-209, 023-32).

## Work
1. Replace the 023-02 stub with the full pipeline; stop after the first failing stage, returning all its diagnostics.
2. Integration test directory `tests/reject/` with at least 2 programs per 023-AC-1 case (a)–(l) (loop and recursion separately for (a)), each with the expected diagnostic code and span.
3. Integration test that every corpus program (023-13) passes `check()` and yields a manifest.

## Acceptance criteria
1. Each of the 12 023-AC-1 categories has ≥ 2 rejection programs; each returns `Err` with ≥ 1 diagnostic of the expected code whose span lies on the offending construct and whose message names the rule.
2. No rejection program produces a panic or an `Ok`.
3. All ≥ 20 corpus programs return `Ok((ir, manifest))`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed

## Out of scope
- Expressiveness programs (023-24); code generation (epic 024).
