# ADR 08 — Two-sided admission

## Status

Accepted

## Context

Live audio has no backpressure. A feed is on the order of hundreds of megabits per second for a wide source (338 Mbit/s for 96 channels, C-22), so a complicated graph oversubscribes a NIC or a module's real-time budget trivially.

In the POC, connection is unilateral `[code]`: the module that owns an output resolves the target's address from the gossip map and starts sending; the receiver never consents. A connection whose target is absent is retried indefinitely. Nothing refuses a connection for capacity. The data thread also drains every input socket completely before looking at its own deadline, so one flooding input starves the module `[code]`.

On a medianet all traffic is admitted traffic, which makes per-interface sums enforceable.

## Decision

**Connection is a two-sided handshake with explicit admission.**

- The requester names a source output (or bundle members) and a consumer input. Both sides accept or refuse.
- **Refusal is explicit and terminal, with a named reason.** "Target not present" stays Pending and visible.
- The handshake carries: declared rates, the bundle member list, the path-MTU probe pair, the reverse measurement port, the pinned addresses, and the manifest (ports, content types, required E for presentation groups).
- **Units:** bit/s on the wire including headers; packets/s; slice rate; input count; buffer memory (skew budget × rate); and, for DSL modules, the compiler's cost bound per slice.
- **Ceiling:** each endpoint link is admitted up to 70 % of its rate (C-23).
- **Endpoint admission is online and dynamic.** On a single switch every link is an endpoint link, so endpoint sums are complete.
- **Trunk links are never computed by the runtime.** Their capacities are declared in the commissioning configuration; the commissioning tool validates the requested graph offline and alarms post hoc through the stamp.
- Bounds: at most 64 active modules per medianet (C-34) and 64 followers per holder (C-35).
- The graph is a desired set; the runtime publishes satisfied versus unsatisfied. An operator alarm is raised through the hello's alarm field.

**Enforcement at the receiver** makes the bounded-time promise real:

- a bounded drain per input per cycle, 2 × the admitted packets per slice for that input (C-16);
- a source filter, so only admitted peers are parsed;
- a socket buffer sized to the declared rate times the skew budget, with the kernel's per-socket overflow counter read on every receive `[kernel-doc]`.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Unilateral send (POC) | The receiver cannot protect its interface or its real-time budget |
| Lazy retry forever on any failure (POC) | A capacity refusal that is retried silently leaves the graph half-connected with no signal |
| Token bucket on the receiver's real-time thread | By the time the thread sees the packet it has already cost kernel time; user-space throttling bounds only handler work |
| CPU admission for all modules | Cost per slice is not known in advance for non-DSL modules; it stays a diagnostic (overrun counter and alarm) |
| Runtime computes trunk feasibility | Endpoints cannot see trunk links; without stream reservation in the switches this cannot be done online |
| All-or-nothing connection groups | Not in v1; the runtime reports satisfied versus unsatisfied instead |

## Consequences

Positive

- A graph that does not fit is refused, by name, before it degrades anything.
- A flooding input costs no real-time: excess is dropped in the kernel and counted.
- Destinations are pinned at admission (invariant L-5).
- DSL modules get real CPU admission from the compiler's cost bound.

Negative

- A connection takes a round trip to establish, and an address change requires re-admission.
- For networks with more than one switch, correctness of capacity depends on operator-declared trunk figures.
- Degradation is reported, not managed: there is no automatic rerouting.

Neutral

- A wide fan-out that exceeds a port (a third unicast copy of 96 channels on GbE, C-24) is simply refused; the remedy is a faster port (ADR 20).

## Falsifier

- AC-12 fails: a request above the ceiling is admitted, or leaves a half-connected state, or loops on retry.
- AC-11 fails: an input flooded at 10 × its admitted rate causes a missed deadline, which would mean the three enforcement mechanisms are insufficient.
- L1 (task 019) shows legitimate bursts exceeding the bounded drain (C-16).
- A 64-entry map does not fit device memory (C-34), or holder serving lateness grows below 64 followers (C-35).

## Links

- [01-architecture.md §6 Capacity and admission](../01-architecture.md#6-capacity-and-admission), [§6.1 Handshake](../01-architecture.md#61-handshake), [§6.4 Enforcement at the receiver](../01-architecture.md#64-enforcement-at-the-receiver), [§4.4 Connection states](../01-architecture.md#44-connection-states)
- [02-constants.md](../02-constants.md): C-16, C-22, C-23, C-24, C-34, C-35
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-11, AC-12, AC-13, AC-37
- Related: [07-Control-plane-invariants.md](07-Control-plane-invariants.md), [20-Ten-GbE-for-wide-nodes.md](20-Ten-GbE-for-wide-nodes.md), [12-Presentation-groups-with-required-E.md](12-Presentation-groups-with-required-E.md)
