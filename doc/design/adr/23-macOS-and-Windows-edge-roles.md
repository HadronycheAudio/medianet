# ADR 23 — macOS and Windows edge roles

## Status

Accepted

## Context

Windows and macOS machines will be members of medianets: the musician's laptop as a sink or as a capture source. The backbone (clock holders, election candidacy, conformance, commissioning) is Linux; embedded devices are followers.

These platforms differ from Linux in receive timestamps, real-time thread mechanisms, audio stacks, timer resolution and what a process may do without privilege. The design must give each a role, a timebase design and a quality claim it can actually certify, instead of a blanket "weaker forever".

Platform facts below are `[memory]` until the stamp probes (task 001) and the per-platform L2 runs confirm them.

## Decision

**Role.** macOS and Windows join as **edges only**: sinks and physical-input capture. They are never clock holders or election candidates (ADR 14).

**Timebase.** The same two-loop design as every edge (ADR 11), with a different enumeration of clocks: mach time and the Core Audio device callback's sample and host times on macOS; QPC and the device clock's position with its QPC stamp on Windows. Both loops are shared core code; each backend supplies a host-clock trait and a device-clock trait. The media deadline clock exists in core only as an abstract trait.

**Stack per platform.**

| | macos-std | windows-std |
|---|---|---|
| Receive timestamp | Kernel, monotonic basis (to verify) | Kernel on recent builds (to verify), else userspace |
| Real-time threads | Time-constraint policy plus workgroup, unprivileged | MMCSS Pro Audio, not hard real-time |
| Audio stack | Core Audio device callback | WASAPI exclusive |
| D obtained from | Device latency properties plus measurement | Measurement only |
| E target | 100 µs (C-71) | 1 ms (C-72) |
| Restart spread of D, 20 restarts | at most 50 µs (C-74) | at most 0.5 ms (C-74) |
| Typical D | a few ms (C-81) | 6 to 20 ms (C-81) |
| Report-only | Overflow accounting | Overflow accounting, DSCP marking, bounded real-time, sub-millisecond pacing |

**Scope.**

- Capture is **physical device inputs only**. Application-output capture (a virtual audio device) is out of v1.
- **Windows:** WASAPI exclusive mode is mandatory for certification. Sink edges do not pace. Capture edges pace emission off the audio callback, batching several slices per engine period, and this is stamped. There is no spinning ticker.
- **macOS:** direct devices only; aggregate devices are excluded.
- If the musician's own software holds the interface, the edge reports it and certifies nothing.
- E is a percentile claim (C-73), not a bound, on both platforms. Each platform profile carries its own E budget and capability bits in the regime stamp.

**Uncertifiable in v1:** any bound on Windows scheduling latency; any device whose driver misreports latency, without a commissioning measurement; macOS aggregate devices, Bluetooth and network audio outputs; any OS, driver and device combination outside a short reference list (stamped "unverified"); isolation of a laptop.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Linux-only medianets | The musician's laptop is a product requirement |
| Full roles (holder, candidate) on macOS and Windows | Laptops sleep and are dual-homed; serving load, availability and oscillator quality argue against it (ADR 14) |
| Assume no kernel receive timestamps off Linux | Probably false on both platforms; made a capability bit and a pre-implementation check instead of an assumption |
| WASAPI shared mode | Passes through the audio engine with opaque latency and possible resampling; not certifiable |
| Spinning ticker on Windows for a 0.5 ms slice clock | Burns a core; the audio callback already provides the pacing source |
| Application-output capture in v1 | Needs a virtual audio device: feasible in user space on macOS, a signed kernel driver on Windows |
| One E target for all hosted platforms | Windows hand-off is quantised to the engine period, so its E is dominated by the uncertainty of D |
| Per-platform servo code | Replaced by shared loops with two traits per platform |

## Consequences

Positive

- Each platform ships with a named stack, a certification entry and an honest quality number.
- A Windows or macOS sink can never silently join a group tighter than its E (ADR 12).
- No platform-specific servo logic to keep consistent.

Negative

- A Windows capture edge delivers several slices per callback; its lateness includes the engine period and the group's A must cover it.
- Exclusive mode collides with the musician's own audio software.
- A laptop is not an isolated node, which weakens the trusted-segment premise (ADR 22).
- Without a per-socket drop counter on either platform, a host drop cannot be told from network loss there.

Neutral

- Laptop devices often run at 48 kHz; the edge resampler absorbs the 2:1 ratio and it adds to D.
- Commissioning checklists per OS cover interface pinning, discovery chatter, firewall scope, sleep prevention and interrupt coalescing.

## Falsifier

- Task 001: kernel receive stamps are absent on macOS or Windows; the clock bound and the E targets rise.
- L2 macOS / L2 Windows (tasks 020, 021, 022): E or restart spread exceeds target (AC-27, AC-28), or batched capture emission causes gaps at the commissioned A.
- A product decision to capture application output, which reopens the scope.

## Links

- [01-architecture.md §10 Platform compatibility](../01-architecture.md#10-platform-compatibility), [§10.1 Capability table](../01-architecture.md#101-capability-table), [§10.2 Scope on macOS and Windows](../01-architecture.md#102-scope-on-macos-and-windows), [§10.3 Porting traps](../01-architecture.md#103-porting-traps), [§10.4 Uncertifiable in v1](../01-architecture.md#104-uncertifiable-in-v1), [§9.4 Two-loop timebase](../01-architecture.md#94-two-loop-timebase)
- [02-constants.md](../02-constants.md): C-71, C-72, C-73, C-74, C-81
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-27, AC-28, AC-30, AC-34
- Related: [11-Software-resampler-edge.md](11-Software-resampler-edge.md), [12-Presentation-groups-with-required-E.md](12-Presentation-groups-with-required-E.md), [14-Capability-bit-holder-eligibility.md](14-Capability-bit-holder-eligibility.md), [22-Authentication-deferred.md](22-Authentication-deferred.md)
