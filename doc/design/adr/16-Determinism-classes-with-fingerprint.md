# ADR 16 — Determinism classes with fingerprint

## Status

Accepted

## Context

The POC verifies content bit-exactly end to end: the sender reruns the echo's code and compares. That works because both ends run the same code, and because the receiving side's oscillator is written with additions and multiplications only, "so that it gives the same bits on every machine" `[code]`. It is a property of the test instrument, not of a runtime, and it cannot hold for arbitrary user modules.

One hole exists even in the POC: the fixed-point sine table is built with the platform's math library on each machine, so it is not strictly cross-machine deterministic `[code]`.

Modules are now written in a DSL and compiled. A compiler can restrict arithmetic and can therefore say, per module, whether bit-exactness is achievable, and across which machines. Floating point makes this subtle: subnormal handling and NaN payloads differ between targets, and floating-point modes are per thread; audio stacks change them.

## Decision

**Three determinism classes, emitted by the compiler in the module manifest.**

| Class | Scope | Verification tier |
|---|---|---|
| exact-fixed | Everywhere, including between host and device | Exact: record and replay |
| exact-float | Only where the startup fingerprint matches | Exact: record and replay |
| bounded | Everything else | Tolerance |

**Arithmetic rules for generated code.**

- No math library and no transcendental functions. The kit is strict f32 operations, Q1.31 and Q0.32 fixed point, and shipped deterministic tables.
- Float operations are never reordered; summation order is defined by DSL semantics.

**Fingerprint.**

- A compile-time vector, including subnormal and NaN paths, is evaluated at start **on the processing thread itself**.
- Match: the module is exact-float eligible on this target.
- Mismatch: the module runs bounded-class and the regime stamp says so.
- The listed-target set is whatever continuous integration verified; nothing else claims exactness.

**Exact tier for user modules.** The runtime records the slices delivered to an exact-class module and a hash of what it emitted; a harness reruns the same generated code offline and compares. Windows touched by loss or by a degraded slice are excluded and counted.

**Edges are never exact-tier**, because they resample.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Bit-exactness as a runtime-wide guarantee | Impossible for modules whose arithmetic the runtime does not control |
| Exact-float everywhere | Subnormal and NaN behaviour differ between targets; exactness between a host and a small device is safe only in fixed point |
| A hand-maintained list of exact-float targets | A list goes stale; the fingerprint tests the actual machine and thread at start |
| Fingerprint evaluated once per process | Floating-point modes are per thread and audio stacks change them |
| Fixed point only | Excludes float DSP for no reason where the fingerprint can prove exactness |
| Tables built at start with the platform math library (POC) | One differing last bit in the library flips a rounded entry |
| Two classes (exact / bounded) | Hides the difference between "exact everywhere" and "exact on verified targets", which decides whether host-to-device replay is valid |

## Consequences

Positive

- The exact tier, which the POC had only for its own instrument, is available to user modules.
- Exactness is claimed per target and per run, and the claim is checked at start.
- The reference echo can be a DSL program and remain bit-exact by construction.

Negative

- The arithmetic kit is restrictive: no transcendental functions outside shipped tables.
- A fingerprint cannot prove exactness for all inputs; its coverage of subnormal and NaN paths must itself be validated.
- A module can change class between two machines; tooling must show that clearly.

Neutral

- Non-DSL modules may claim a class, but that is module-author policy; only DSL modules get a compiler-derived class.

## Falsifier

- CI across listed targets (task 025): a target passes the fingerprint and differs on a conformance vector (AC-32). Then the fingerprint's coverage is insufficient.
- exact-fixed programs differ between any two targets.
- POC archaeology (task 002) finds that the POC's voice comparison is not bit-level, which would weaken the baseline the exact tier inherits.

## Links

- [01-architecture.md §12.5 Determinism classes](../01-architecture.md#125-determinism-classes), [§12.1 Process function](../01-architecture.md#121-process-function), [§16 Verification tiers](../01-architecture.md#16-verification-tiers)
- [02-constants.md](../02-constants.md): C-03
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-8, AC-10, AC-32
- Related: [17-Content-domain-scope.md](17-Content-domain-scope.md), [11-Software-resampler-edge.md](11-Software-resampler-edge.md), [15-Crate-split-with-state-machines-in-core.md](15-Crate-split-with-state-machines-in-core.md)
