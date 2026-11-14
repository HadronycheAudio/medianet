# ADR 13 — Interrupted and dead thresholds

## Status

Accepted

## Context

Invariant L-1 (ADR 07) says a live connection can be declared dead only from data-plane evidence. That needs a threshold, and one threshold cannot serve two purposes.

A consumer must react fast to missing slices: it has its own deadline, it must conceal, and its module must be told. But declaring a connection dead releases its resources and forces re-admission, which costs seconds of re-alignment (aligning alone takes 4 s, C-50). A link flap, a switch reconvergence or a scheduling hiccup on the sender lasts milliseconds to seconds.

There is also a reverse direction. UDP is one-way: a producer has no data-plane evidence of its consumer. In the POC the producer sends "for as long as the other module is present", which is gossip expiry `[code]`.

## Decision

**Two thresholds on the consumer side.**

- **Interrupted** after 8 consecutive missed slices (C-38), which is 4 ms at defaults. The module is told and conceals. Nothing is released.
- **Dead** after 2 s without data (C-39). An end-of-stream event is delivered and resources are released.
- Between the two, returning slices on the same session **resume immediately at the frozen offset**. No re-alignment is needed.

**One threshold on the producer side.**

- The consumer's data thread sends a receiver report, one per bundle every 100 ms (C-40), on the reverse unicast port given in the admission handshake.
- The producer stops sending after 2 s without receiver reports (C-41).

**Granularity.** Interrupted and dead are per bundle member; the bundle is dead when all members are.

**Visibility.** A stalled data plane is also visible to peers through the hello's liveness counter within 2 hello periods (C-42); the module's own housekeeping sees it first and self-reports.

**Classification.** Interrupted and dead sit at the end of the gap classification: marked gap, late, missing, then interrupted, then dead.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| One threshold of 8 slices for dead | 4 ms of silence would force re-admission and seconds of re-alignment; a scheduling hiccup becomes an outage |
| One threshold of seconds for everything | The module would not be told for seconds that it is concealing; downstream could not distinguish a hiccup from normal operation |
| Gossip expiry as the dead signal | That is the false-expiry class of failure invariant L-1 removes |
| Producer stops on the consumer's hello expiry | Gossip-driven liveness in the reverse direction; a starved housekeeper on the consumer would stop a healthy stream |
| Producer never stops | Holds admitted bandwidth for a consumer that no longer exists |
| Receiver reports sent by housekeeping | Reintroduces housekeeping as a gate on the data plane; they must come from the data thread |
| Receiver reports per stream | Sixteen times the volume per bundle for information that fits one message |

## Consequences

Positive

- Fast reaction and slow teardown are separated; short interruptions cost only their own duration.
- Both directions of a connection have data-plane liveness.
- Downstream tooling can tell "upstream lost slices" from "upstream is dead".

Negative

- The consumer's data thread carries a periodic send; the producer's data thread tracks reports.
- For up to 2 s a producer keeps sending to a consumer that has died, and a consumer keeps resources for a producer that has died.

Neutral

- The receiver report also carries per-member counters, drop counters and the observed DSCP, so the liveness message pays for itself as measurement.

## Falsifier

- AC-16 fails: a short interruption forces re-alignment, or resume is not immediate at the frozen offset.
- AC-21 fails: after a killed data or clock thread, peers do not report interrupted within 4 ms and dead at 2 s, or the producer does not stop after 2 s without reports.
- L1 (task 019) shows ordinary operation producing runs of 8 consecutive misses on a clean network, which would make "interrupted" noise rather than signal.

## Links

- [01-architecture.md §4.4 Connection states](../01-architecture.md#44-connection-states), [§4.6 Gap contract](../01-architecture.md#46-gap-contract), [§5 Control-plane invariants](../01-architecture.md#5-control-plane-invariants), [§3.6 Measurement path](../01-architecture.md#36-measurement-path)
- [02-constants.md](../02-constants.md): C-38, C-39, C-40, C-41, C-42, C-50
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-16, AC-20, AC-21
- Related: [07-Control-plane-invariants.md](07-Control-plane-invariants.md), [04-Detect-and-conceal-redundancy.md](04-Detect-and-conceal-redundancy.md), [02-Channel-bundling.md](02-Channel-bundling.md)
