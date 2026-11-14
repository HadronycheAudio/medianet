# ADR 22 — Authentication deferred

## Status

Deferred-with-reservation

## Context

The control channel (discovery, election, admission, forced actions) is unauthenticated. In the POC anyone on the LAN can connect outputs, change settings, and request or force elections `[code]`.

A medianet's recommended configuration is a wired, isolated LAN carrying no other traffic. On such a segment the realistic failure is accident, not attack: two installations sharing a LAN by mistake.

Two things weaken the isolation premise. macOS and Windows laptops join as edges; a laptop is dual-homed to the internet and emits discovery chatter on the medianet port. And module ids must be unique, yet devices without a true random source would boot with identical "random" values.

Authentication needs a keyed tag, and a tag alone cannot stop replay; a counter is needed too.

## Decision

**v1 makes no guarantee against hostile control traffic on the segment, and says so. Protection against accident is guaranteed. Wire space for authentication is reserved now, and authentication is scheduled immediately after v1.**

Guaranteed in v1:

- **Domain id:** 16 bits, explicit operator configuration, in every message (C-11). Traffic from a foreign domain is dropped and counted. Domain 0 means unconfigured, and strict mode refuses to start with it.
- **Identity:** module id of 64 bits from the operating system's random source plus a 64-bit instance nonce (C-11). Devices without a true random source mix a hardware unique identifier with a persistent boot counter.
- **Collision:** a module that sees its own id with a foreign nonce detects it within one hello period (1 s, C-30); the later joiner redraws; third parties quarantine the id; it is logged on both modules.
- **Pinned destinations:** a hello with a new address never redirects a live stream.

Reserved (C-12), zeroed in v1, in every control message:

- scheme: 8 bits (0 = none);
- message counter: 64 bits (it doubles as the liveness counter);
- tag: 128 bits.

Diagnostic only: control commands from ids not in the map are logged.

Trust statement for the specification: "trusted segment per the recommended configuration; Windows and macOS edges are best-effort members of a weaker segment". The commissioning baseline is a dedicated medianet NIC with OS discovery chatter suppressed and the firewall scoped to medianet ports.

The data plane is unauthenticated, in v1 and in the reserved scheme.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Shared-key authentication of control messages in v1 | Adds key distribution, replay handling and a cryptographic dependency to a first release whose recommended deployment is an isolated segment; the reservation makes it addable without a wire break |
| Reserve a 128-bit tag only | A tag alone cannot stop replay; a counter and a scheme identifier are needed |
| No reservation | Adding authentication later would be a wire break |
| Random-derived domain id | The domain is operator-owned on a medianet; it must be configured, not drawn |
| 31-bit id from time and process id (POC, P-06) | Too narrow, and a collision is silently fatal: the peer merges into self |
| Id and nonce both from a pseudo-random generator on devices | Without a true random source forty devices boot with the same id and the same nonce |
| Authenticate the data plane | Per-packet cost on every slice; out of scope |

## Consequences

Positive

- Two medianets bridged by accident do not affect each other.
- A collision is never silent and cannot hijack a stream.
- Authentication later costs no wire break and no version bump for the fields.

Negative

- Anyone on the segment can connect, reconfigure and force elections in v1. The documentation must say so.
- 25 bytes of every control message are reserved and unused in v1, inside a bound of 1 252 bytes (C-17).
- Laptop edges make the "isolated" premise weaker than the recommended configuration assumes, which raises the priority of the deferred work.

Neutral

- The message counter is useful in v1 as a liveness counter even with scheme 0.

## Falsifier

- A deployment in which the segment cannot be kept trusted (for example laptops that cannot be given a dedicated, quiet medianet NIC). Authentication then moves into v1.
- AC-22 fails: a forced id collision merges two peers or goes unlogged.
- AC-23 fails: foreign-domain traffic changes state.

## Links

- [01-architecture.md §10.5 Trust statement](../01-architecture.md#105-trust-statement), [§15 Failure semantics](../01-architecture.md#15-failure-semantics), [§3.2 Wire layout](../01-architecture.md#32-wire-layout), [§5 Control-plane invariants](../01-architecture.md#5-control-plane-invariants)
- [02-constants.md](../02-constants.md): C-11, C-12, C-17, C-30, P-06
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-22, AC-23
- Related: [07-Control-plane-invariants.md](07-Control-plane-invariants.md), [23-macOS-and-Windows-edge-roles.md](23-macOS-and-Windows-edge-roles.md), [15-Crate-split-with-state-machines-in-core.md](15-Crate-split-with-state-machines-in-core.md)
