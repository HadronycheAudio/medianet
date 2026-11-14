# ADR 05 — Position-invariant fan-in alignment

## Status

Accepted

## Context

A module with several inputs receives slices for the same media position over paths of different depth. In a diamond graph (a source feeds a consumer directly and through a processor) the two copies are skewed by a full hop plus queueing, not merely jittered.

The POC does not handle this: its mixer keeps the latest block of each input and mixes on arrival, with no alignment `[code]`. Its data plane is also arrival-driven throughout, and its media clock is the sender's local crystal, so two sources drift apart by their ppm difference `[code]`.

The runtime needs fan-in semantics that are exact, that do not require anyone to know the critical path, and that do not pay a jitter margin at every hop.

## Decision

**Position invariant.** A module's output for position p is derived from its inputs at position p. The runtime stamps output positions; module code cannot change them. A module with declared structural delay L has its output stamped p − L while it processes input p. Stamps denote content position and are never restamped per hop.

**Alignment by position at a fixed time.**

- A fan-in consumer holds each aligned input in a ring indexed by position.
- It processes position p at the fixed time `T(p) + offset`, with whatever has arrived. Anything absent is a gap.
- **It always fires at the fixed time**, never early.
- Alignment is sample-exact by position, whatever the clock error; the clock only decides when the deadline fires.

**Align only where needed.** Single-input modules stay arrival-driven and forward immediately. Deadlines exist only at fan-in points and at sinks.

**Nobody needs to know the critical path.** During the aligning state the consumer measures origin-to-arrival lateness per input, which already includes every upstream hop (ADR 06).

**Precondition.** Sources pace position from network time on one domain line (ADR 09).

**Opt-outs.** A return edge in a cycle is declared unaligned. A per-input "unaligned" option keeps mix-on-arrival for inputs that must not wait.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Per-edge deadlines negotiated upstream (latency budgets pushed to producers) | Upstream cannot do anything to meet a negotiated deadline, so negotiation reduces to refusal |
| Wait until every input arrived *or* its deadline passed (early fire) | Makes output timing data-dependent and passes jitter downstream |
| Deadline-driven at every hop | Every hop pays its own jitter margin and latency grows with depth |
| Mix on arrival (POC) | No alignment at all; skew between paths becomes comb filtering |
| Per-input hold buffers sized to a declared skew budget, as a separate mechanism | Not an alternative: it is the storage that waiting by position requires |
| Runtime restamps positions per hop | Hides each hop's delay; deep-graph skew becomes invisible |
| Critical path computed from the gossip map | Unnecessary: the consumer's own lateness measurement includes every hop |

## Consequences

Positive

- Sample-exact alignment at any fan-in, independent of clock error.
- Fan-in output is re-clocked: its timing is free of upstream jitter.
- Latency along a chain is the sum of medians plus one margin at the fan-in, not a margin per hop.
- Lineage (hop count, accumulated residence, accumulated declared delay) is carried in the measurement extension, so deep-graph delay is attributable.

Negative

- The direct branch of a diamond is delayed to match the slow one; the margin is always paid at a fan-in.
- The handler changes shape, from the POC's per-datagram callback to a per-position callback over a set of inputs.
- Positional delay inside a module (content stamped p that is really input p − L) cannot be observed live; a wrong declared L is undetectable on live audio and is verified only by the latency probe tool.

Neutral

- Residence time is self-accounting: it shows up in the downstream consumer's measured lateness and needs no declaration. One event, "processing overrun", covers a module that completes after its deadline.

## Falsifier

- AC-10 fails: a diamond graph with injected per-hop delay does not reproduce the offline reference bit for bit (L0, task 009).
- L1 (task 019) shows that fixed-time firing at fan-ins and sinks cannot meet zero late slices at any acceptable A.

## Links

- [01-architecture.md §4.1 Position invariant](../01-architecture.md#41-position-invariant), [§4.2 Fan-in alignment](../01-architecture.md#42-fan-in-alignment), [§4.7 Lineage](../01-architecture.md#47-lineage)
- [02-constants.md](../02-constants.md): C-50, C-51, C-52
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-10, AC-14, AC-15, AC-18
- Related: [06-Ratcheted-offset.md](06-Ratcheted-offset.md), [09-Domain-line-with-epochs.md](09-Domain-line-with-epochs.md), [02-Channel-bundling.md](02-Channel-bundling.md)
