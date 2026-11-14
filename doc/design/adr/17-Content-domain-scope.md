# ADR 17 — Content-domain scope

## Status

Accepted. Transport authoring in the DSL is Deferred-with-reservation.

## Context

The POC carries four kinds of content: audio, controller values, "musical events" and transport `[code]`. The DSL for module construction must decide which of these a v1 module can consume and produce.

An obvious cut is "audio and controllers only; events and transport are specified on the wire but not supported in the DSL". That cut misreads what the POC's events are. They are not discrete events. A part is one field of one voice and the value it should move to; it takes effect by moving the field linearly from its current value to the new one over the audio of the slice that brought it; and loss is repaired by re-sending current values `[code]`. That is a sparse, keyed controller with state-refresh transport.

The cut also contradicts another requirement: the reference echo is to be a DSL program, and its instrument is driven by exactly these parts.

Transport is different in size: tempo map, time code, lead-time announcements, a conductor that authors state `[code]`.

## Decision

**v1 content domains in the DSL:**

- **Audio.**
- **Dense controllers.**
- **Sparse keyed controllers:** one field of one voice moving linearly to a value over the slice, repaired by state refresh.
- **Read-only transport state** as an input (playing, bar, beat, tempo at a position).

**Deferred past v1:** transport **authoring** in the DSL. Transport semantics are specified at wire level in the runtime specification.

Related rules:

- Default gap behaviour per content type is compiler-generated and overridable by DSL constructs: audio zero-fill; dense controllers hold last value; sparse keyed controllers rely on state refresh.
- Degraded marking is derived per output from dataflow.
- In a bundle, controllers ride in their own member; a lost controller slice means hold-last-value while audio plays.
- Transport reference changes are recognised in-band, not from gossip.
- The reference echo is a DSL program.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Audio and dense controllers only | The reference echo cannot be a DSL program without sparse keyed controllers, because its instrument is driven by them; and the POC's "events" are controllers, so cutting them saves almost nothing |
| Discrete one-shot events as a v1 domain | The POC has none to carry over; every "event" it sends is state |
| Full transport in the DSL, including authoring | Large surface (tempo map, time code, lead-time announcements) with one authoring module in the whole system |
| No transport at all in the DSL | A metronome or any beat-aware module needs to read the playhead; reading is cheap and looks like a controller input |
| Retransmit lost events | State refresh already repairs loss without retransmission and without per-receiver state |

## Consequences

Positive

- Synthesisers and the reference echo are expressible in the DSL.
- Loss repair for sparse controllers needs no extra mechanism.
- The DSL's surface stays small: every domain is "a signal over positions".

Negative

- A module that conducts (authors transport) cannot be written in the DSL in v1; the reference conductor stays hand-written against the runtime.
- State refresh spends some bytes of every slice on re-sending current values.

Neutral

- Wording changes from the POC: content "takes effect over the slice that brought it", not "the datagram that brought it".
- Reservation cost: none on the wire; transport messages are already specified there.

## Falsifier

- The reference echo cannot be expressed with these four domains (tasks 023, 024).
- A v1 product module needs to author transport from DSL code.
- POC archaeology (task 002), on reading the unread transport and sender code, finds a discrete event the description above missed.

## Links

- [01-architecture.md §12.7 Content domains, v1](../01-architecture.md#127-content-domains-v1), [§12.6 Gaps and degraded marking](../01-architecture.md#126-gaps-and-degraded-marking), [§4.6 Gap contract](../01-architecture.md#46-gap-contract), [§7 Channel bundles](../01-architecture.md#7-channel-bundles)
- [02-constants.md](../02-constants.md): C-02, C-03
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-8, AC-32
- Related: [16-Determinism-classes-with-fingerprint.md](16-Determinism-classes-with-fingerprint.md), [02-Channel-bundling.md](02-Channel-bundling.md), [07-Control-plane-invariants.md](07-Control-plane-invariants.md)
