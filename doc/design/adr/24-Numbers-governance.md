# ADR 24 — Numbers governance

## Status

Accepted

## Context

The runtime reports latency, loss, clock quality and content correctness. Those numbers are the product's evidence, and a skeptic attacks their basis before the code.

The proof of concept shows why this needs design, not goodwill. Its first measured run, which looked green, contained `[measured]`:

- a corrupt metrics file: every module in the process opened the same path and wrote from its own offset (four writers on one file);
- a loss counter showing values around 21.7 million while about 50 000 packets had been sent;
- two whole check families reporting "0 of 0" for the entire run;
- a sync rate of 37 per second at a configured 50, and 77 at a configured 1 000, read from that corrupt file (superseded by the clean figures 36.1 and 71.4, P-02, REPORT-0b section (e)).

Further `[code]`: a run where real-time scheduling silently failed looks identical to a good one; measurement buffers drop silently when full, and the samples lost under load are the worst ones; a version mismatch is a silent drop.

An embedded device cannot write files at all.

## Decision

**Provenance, ground truth and self-validation are first-class runtime surface.**

**Regime stamp.** A wire object defined in core. A device publishes its stamp and a hosted collector records it. It carries: identity and full configuration; capability bits as present / absent / unverified (receive timestamp source, real-time scheduling granted, memory locked, drop accounting, DSCP marking, fault injection, holder-eligible); network regime (interface and path MTU, packets per slice, pacing mode, DSCP classes and peer-observed values, coalescing state); clock state and bound at each one-way figure; alignment (offset, percentiles, A, required E); platform (OS build, audio device, driver); drop counters; any active injection.

**Rules.**

- A change of regime mid-run starts a new record set; it is never averaged across.
- A measurement involving two endpoints carries both profiles.
- One-way figures are blank until lock and carry the bound thereafter.
- **A check family with zero samples is a failed run.**
- **A file has one writer, owned by the process**, never by a module.
- Every drop is counted: per-socket overflow `[kernel-doc]`, interface counters, measurement-buffer overflow, rejected packets per input and per reason.
- Loss is reported at both levels in the multi-packet regime: packet and slice.
- DSCP marking is verified by the receiving peer.

**Injection as public API.** Clock offset and drift; packet loss, delay and reordering from a seeded generator; per-hop delay. Any active injection is stamped.

**Self-test** before any number is reported: receive timestamps present and consistent with the monotonic clock; real-time scheduling actually granted; wake latency; packets per slice; clock locked within its bound. Numbers produced earlier are labelled unqualified. **Strict mode** turns a failed check, or an unconfigured domain, into a refusal to start.

**Loss floor.** The expected floor is published (C-90) and loss above the measured clean-network threshold (C-91) raises an alarm.

**Smoke assertions never depend on sync-rate timing.**

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Best-effort degradation with warnings on screen (POC) | A degraded run is indistinguishable from a good one in its records |
| Stamp as a file header written by each module | Devices cannot write files; several modules writing one file is the POC's corruption |
| Hard privilege requirements, no degradation | Removes the ability to run unprivileged in continuous integration and on a laptop; strict mode offers refusal where wanted |
| Treat an empty check family as "nothing to report" | Half the verification can be inert on a green run |
| Self-reported DSCP and capabilities | A platform can believe it marked or scheduled when it did not |
| External tools only for fault injection | Needs privilege and cannot run on loopback for multi-packet or fan-in behaviour |
| Average across a regime change | Mixes measurements taken under different conditions |

## Consequences

Positive

- Every number can be traced to the conditions that produced it.
- An injected or under-privileged run can never pass as a real one.
- Ground truth (injected offset and drift against the servo's estimate) is available in every loopback run.
- Loss rate becomes a cable-health diagnostic.

Negative

- The stamp is a sizeable wire object that every profile must produce and keep current.
- A run can fail for reasons of instrumentation alone (an inert check, an incomplete stamp).
- On macOS, Windows and some devices several capability bits will read "absent" or "unverified"; the numbers from those nodes are weaker and say so.

Neutral

- The stamp format is data in core; producing it is profile code.

## Falsifier

- AC-9 fails: a run with an inert check family exits green.
- AC-31 fails: a record set without a complete stamp, or two writers on one file.
- AC-37 fails: an induced socket overflow does not appear in the drop counters on Linux.
- AC-8 fails: an assertion in the smoke run changes with the sync rate.

## Links

- [01-architecture.md §11 Numbers governance](../01-architecture.md#11-numbers-governance), [§11.1 Regime stamp](../01-architecture.md#111-regime-stamp), [§11.2 Injection as public API](../01-architecture.md#112-injection-as-public-api), [§11.3 Self-test](../01-architecture.md#113-self-test), [§11.4 Loss floor](../01-architecture.md#114-loss-floor)
- [02-constants.md](../02-constants.md): C-90, C-91, P-02
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-8, AC-9, AC-31, AC-34, AC-37
- Related: [04-Detect-and-conceal-redundancy.md](04-Detect-and-conceal-redundancy.md), [15-Crate-split-with-state-machines-in-core.md](15-Crate-split-with-state-machines-in-core.md), [18-DSCP-classes.md](18-DSCP-classes.md), [21-Rewrite-versus-evolve.md](21-Rewrite-versus-evolve.md)
