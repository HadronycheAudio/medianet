# ADR 21 — Rewrite versus evolve

## Status

Accepted

## Context

The runtime is derived from a working proof of concept: about 8 000 lines of Rust in one binary, with a fabric (discovery, elections, clock, data plane) and a test application (sender, synth, mixer, analysis) that are already cleanly separated `[code]`.

Two paths lead from it to the runtime: extract the fabric into library crates and grow from there, or write the runtime anew against a specification and keep the POC as reference.

Facts about the POC that bear on the choice:

- The fabric's core is one 1 756-line file doing five jobs: builder framework, gossip, elections, metrics and telemetry, command-line helpers `[code]`.
- There are two tests in the whole codebase, both in the transport code `[code]`.
- The directory is not under version control `[measured]`.
- Its first measured run exposed instrument bugs: a corrupt metrics file (four writers on one path), an implausible loss counter, two check families reporting zero samples on a green run, and a sync rate of 37 per second at a configured 50 (P-02) `[measured]`.
- Its tuning encodes hardware experiments that are expensive to repeat.

## Decision

**Rewrite the fabric as new library crates against a written protocol specification. Freeze the POC and keep it as the reference instrument. Port specific pieces; do not extract modules wholesale.**

- The POC is put under version control and tagged before anything else. It is a frozen reference: the baseline for the exact verification tier and the source of the reference sender, echo and mixer.
- Its instrument bugs are diagnosed (archaeology) before it is cited as the exact-tier baseline, and the unread parts of its code are read before conformance vectors are extracted from it.
- **Ported essentially unchanged**, because small, self-contained and right: the servo's estimator idea (minimum round trip per window), the seqlock, the real-time setup with its warning collection, receive with kernel timestamps, the transport state machine with its tests, the add/multiply-only sine and the fixed-point arithmetic of the reference instrument.
- **Not ported:** the servo's actuation (redesigned, ADR 10), the gossip and election code (rewritten as state machines in core, ADR 15), the wire format (ADR 03), the thread model.
- Hard-won constants are carried forward as documented defaults with their reasons: the PM QoS request held open for the process lifetime; lifting the real-time CPU limit for busy-poll threads; locking memory only when the limit does not apply; file output on its own writer thread; skipping missed slices instead of sending a burst; excluding machines under I/O pressure from holding the clock.
- Order of work: specification, then conformance vectors and servo simulation, then the library test-first, then reference modules ported onto it.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Evolve: extract module, net, proto, clock, rt and stats into a library crate and grow | The changes land in the core, not at the edges: packetisation, header, measurement extension, domain id, wider ids, thread model and gossip format all touch the 1 756-line file that would be extracted first; there are almost no tests and no version history, so refactoring has no safety net |
| Rewrite and discard the POC | Loses the reference instrument, the exact-tier baseline and the tuning knowledge |
| Port the servo unchanged | Its actuation violates the continuity and slew requirements (ADR 10) |
| Fix the POC into the runtime in place | Same objection as evolving, plus it destroys the frozen baseline |

## Consequences

Positive

- The specification and conformance vectors, which are needed anyway, are written first; the runtime is then the second implementation that proves the specification complete, the POC being the first.
- Test-first from the start.
- The per-position handler shape, the state machines in core and the new wire format are designed in, not retrofitted.

Negative

- Behaviour nobody wrote down may be lost; the constants list above and the archaeology task are the mitigation.
- The POC and the runtime interoperate only where the wire format still overlaps, which is little.
- The I/O-pressure eligibility rule rests on experiment files whose integrity must be checked before it is inherited.

Neutral

- A small set of fixes to the POC's smoke basics (single metrics writer, deadline-paced sync) is made before the freeze so the baseline is citable; both states are tagged.

## Falsifier

- Archaeology (task 002) finds the POC's comparison of voice audio is not bit-level, or that the I/O-pressure finding does not survive, which changes what the frozen baseline is worth.
- The specification (tasks 004 to 006) turns out to describe so little change to the core that extraction would have been cheaper; not expected.

## Links

- [01-architecture.md §13 Crate split](../01-architecture.md#13-crate-split), [§16 Verification tiers](../01-architecture.md#16-verification-tiers), [§2.4 Servo requirements](../01-architecture.md#24-servo-requirements)
- [02-constants.md](../02-constants.md): P-01, P-02, P-04, P-05, P-06
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-1, AC-8, AC-9
- Related: [15-Crate-split-with-state-machines-in-core.md](15-Crate-split-with-state-machines-in-core.md), [10-Servo-redesign.md](10-Servo-redesign.md), [24-Numbers-governance.md](24-Numbers-governance.md)
