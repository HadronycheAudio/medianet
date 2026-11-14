# 00 — Context

## Product frame

**Hadronyche Medianets** are wired, isolated LANs that carry no internet traffic: the only bytes on the wire are the runtime's. On a medianet, modules find each other, agree on one clock, and exchange audio and control content with measured latency and stated guarantees.

The product is three deliverables:

| Deliverable | What it is |
|---|---|
| Packetisation and network handling | The wire protocol, clocking, alignment, admission, measurement and failure semantics described in [01-architecture.md](01-architecture.md) |
| Module-construction DSL | A language and compiler for modules. The compiler derives latency, cost and determinism class, and emits code against the core ABI |
| Two implementations | A hosted (std) version and a `no_std` version for embedded devices, sharing one core crate |

## Platform roles

| Platform | Role |
|---|---|
| Linux | Backbone: clock holders, election candidates, sources, processors, sinks, conformance and commissioning |
| macOS | Edge only: sinks and physical-input capture (the musician's laptop) |
| Windows | Edge only: sinks and physical-input capture, WASAPI exclusive mode |
| Embedded | Follower only: narrow sinks and sources; a device whose hardware runs at 48 kHz converts at its edge |

A medianet needs at least one holder-capable (Linux) node. Wi-Fi is out of scope entirely.

## The POC and its role

**hadropoc** is a Rust proof of concept (Linux, about 8,700 lines, one binary) from which this design is derived. In it, modules discover each other by multicast gossip, elect a clock holder and a transport reference, and stream 96 kHz audio, controllers, musical-event parts and transport in single datagrams, measuring forward and round-trip latency and verifying content bit for bit end to end.

Its role from now on is **frozen reference instrument**:

- It is not evolved into the runtime. The runtime is a rewrite against a written specification ([ADR 21](adr/21-Rewrite-versus-evolve.md)).
- After a small, tagged set of instrument fixes (roadmap phase P0), it is the baseline for the exact verification tier and the first conformance peer wherever the wire formats still overlap.
- Nothing may cite it as the exact-tier baseline until the archaeology task has cleared its known instrument bugs ([06-evidence-appendix.md](06-evidence-appendix.md)).

What the POC proved:

- Decentralised discovery, clock election and transport-reference choice work without a central instance.
- A minimum-round-trip servo locks software clocks across machines well enough to report one-way latency.
- Bit-exact end-to-end verification is achievable when the sender reruns the receiver's code, given add-and-multiply-only arithmetic.
- State-refresh "parts" repair lost control data without retransmission.

What it got wrong silently, each of which became a design requirement:

- Its default datagram is 9.6 KB, so IP fragmentation was the normal operating mode and was never mentioned.
- Its media clock runs on the sender's local crystal, not on network time.
- Its data thread drains inputs without bound and mixes on arrival with no alignment.
- Its sync pacing saturates far below the configured rate; its metrics file corrupts itself in loopback; half of its checks can be inert while the run looks green.

## Evidence-status legend

Every factual claim in this set that is not self-evident carries one of these tags.

| Tag | Meaning | Trust |
|---|---|---|
| `[code]` | Verified by reading the POC source; file and line given where useful | High for "the code does X"; says nothing about runtime behaviour |
| `[measured]` | Observed in an actual run, with the machine and conditions stated | High for that machine |
| `[kernel-doc]` | Documented operating-system behaviour, checked against the documentation | High |
| `[standard]` | A figure cited from a named, published standard, with the clause | High |
| `[memory]` | Recalled, not verified. A named task must check it before anything depends on it | Low until checked |
| `[decision]` | A design choice. Not a fact; it can be changed, at the cost stated in its ADR | — |

Two standing caveats:

- The design was produced by reading the POC, not by running it. The only measurements are the ones listed in the evidence appendix, taken by the product owner.
- Several POC files were only partly read (`audio.rs` `analyse`, `sender.rs`, `transport.rs`, `stats.rs`, `control.rs`, `tui.rs`). Task 002 closes this before conformance vectors are extracted.

## Vocabulary

| Term | Meaning |
|---|---|
| Domain | One medianet's runtime instance, identified by a configured 16-bit id |
| Domain line | The domain-wide timeline: position is network time counted in samples from an epoch |
| Position | A sample index on the domain line |
| Slice | The content of one stream for one stretch of positions (0.5 ms by default) |
| Stream | A narrow sequence of slices from one source (6 channels at defaults) |
| Bundle | A group of aligned streams emitted by one wide source |
| Holder | The module whose clock defines network time |
| Timeline epoch | A counter that changes when the domain line is discontinuous |
| Fan-in | A module with more than one aligned input |
| Offset | The fixed delay after which a fan-in processes a position |
| A | Audition offset: the configured mouth-to-ear latency of a presentation group |
| D | A sink's declared total plumbing delay |
| E | The certified bound-like figure on a sink's emission error (a 99.9th percentile) |
| Regime stamp | The record of the conditions under which a measurement was taken |
| Exact tier / tolerance tier | Bit-for-bit verification / verification within stated epsilons |
