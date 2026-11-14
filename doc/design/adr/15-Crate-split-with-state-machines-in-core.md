# ADR 15 — Crate split with state machines in core

## Status

Accepted

## Context

The product has two implementations: a hosted (`std`) version and a `no_std` version for embedded devices. Hosted now means three operating systems. A module-construction DSL compiles against the runtime.

The question is where the line between shared and profile-specific code falls. The narrow answer puts only packet definitions and pure algorithms in the shared crate and lets each profile implement discovery, election, admission and connection handling against its own network stack.

In the POC all of this sits in one 1,756-line file that does five jobs `[code]`, with gossip as text.

## Decision

**Control logic lives in the core crate, not only packet definitions.** The discovery map, election, admission handshake and connection states are pure state machines: message and time in, messages and events out. A profile implements only a transport trait and a time trait.

| Crate | Content |
|---|---|
| `hadronyche-core` (`no_std`) | Servo, seqlock, packetiser and reassembler, parsers, gap semantics, copy-index dedup, control state machines, regime stamp format, abstract time and transport traits |
| `hadronyche-kit` (`no_std`) | Fixed-point and strict-float arithmetic, tables, resampler node |
| `hadronyche-std` | Hosted common code: sockets, threads, housekeeping supervision, stamp collector |
| `hadronyche-os-linux` | Kernel timestamps, real-time setup, ALSA edge |
| `hadronyche-os-macos` | Mach time, workgroups, Core Audio edge |
| `hadronyche-os-windows` | QPC, MMCSS, WASAPI edge |
| `hadronyche-embedded` | HAL traits, rate actuator, reference device |
| `hadronyche-dsl` | Compiler; emits against the core ABI and the kit |
| `hadronyche-reference` | Reference sender, echo and mixer |
| `hadronyche-vectors` | Conformance data |
| `hadronyche-tools` | Commissioning, monitor |

Consequences fixed by this decision:

- **Fixed-capacity peer map** of 63 peers (C-34, C-217).
- **Control message size bound:** at most 1 252 bytes of UDP payload, one packet at the fallback MTU (C-17).
- **Binary control format**, not text.
- The **regime stamp is a wire object** defined in core, because a device cannot write files.
- Hosted builds have an **edge-only feature**: no holder candidacy.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Core holds packet definitions only; each profile implements the control logic | Two (or four) implementations of the election and admission logic, which is exactly the divergence conformance exists to prevent |
| One hosted crate with operating-system code behind `cfg` | Three audio stacks, three timestamp sources and three real-time mechanisms in one crate; backends are separate so each can be built and certified alone |
| Text gossip (POC) | Parsing and number formatting on a small device; no hard size bound |
| Unbounded peer map | Cannot be sized on a device |
| Monitor and reference modules inside the hosted crate | They are consumers of the runtime, not part of it |
| Arithmetic kit inside core | Generated code and edges need the kit; keeping it separate keeps the core ABI small |
| Extract crates from the POC | See ADR 21 |

## Consequences

Positive

- One implementation of every control state machine for both profiles; the specification's state machines have exactly one implementation to conform to.
- The state machines are testable in deterministic simulation with no sockets and no threads.
- A device implements a transport trait, not a protocol.
- Conformance vectors and scenario vectors apply unchanged to every profile.

Negative

- Core must be allocation-free or bounded everywhere, which constrains its design (fixed maps, bounded messages).
- The transport and time traits become a long-lived ABI between core and every backend.
- More crates to version together.

Neutral

- The split does not decide threading: hosted profiles run the state machines on the housekeeping and clock threads; a device may run them in one loop.

## Falsifier

- L3 / embedded build (task 026): the core state machines with a 63-peer map and 1 252-byte messages do not fit the reference device's memory (C-34, C-17).
- Spec drafting (task 004, task 005): a hello for a 32-member bundle source does not fit the control message bound (C-17).
- A profile needs control behaviour that cannot be expressed as message-and-time-in, messages-and-events-out.

## Links

- [01-architecture.md §13 Crate split](../01-architecture.md#13-crate-split), [§1.3 Per-module runtime](../01-architecture.md#13-per-module-runtime), [§11.1 Regime stamp](../01-architecture.md#111-regime-stamp), [§16 Verification tiers](../01-architecture.md#16-verification-tiers)
- [02-constants.md](../02-constants.md): C-08, C-17, C-34
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-1, AC-35
- Related: [21-Rewrite-versus-evolve.md](21-Rewrite-versus-evolve.md), [07-Control-plane-invariants.md](07-Control-plane-invariants.md), [16-Determinism-classes-with-fingerprint.md](16-Determinism-classes-with-fingerprint.md), [24-Numbers-governance.md](24-Numbers-governance.md)
