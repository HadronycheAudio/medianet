# ADR 10 — Servo redesign

## Status

Accepted. An external clock discipline (PTP) is Deferred-with-reservation.

## Context

Followers estimate their offset to the clock holder with request/reply exchanges. The POC servo is sound as an estimator: within each 200 ms window it uses the exchange with the smallest round trip, and a stale or duplicate reply is harmless because the reply carries its own request timestamp `[code]`.

Its actuation and its surroundings are not fit for the runtime `[code]`:

- It steps the phase estimate at every update, so published time is discontinuous.
- At `poc-untouched` it paces requests with a receive timeout (task 003 replaced this by a deadline-paced wait in `poc-baseline`, P-07). Measured at `poc-untouched`: 36.1 completed exchanges per second at a configured 50, and 71.4 at a configured 1 000 (P-02) `[measured]`, on a kernel ticking at 250 Hz (P-03).
- Its holdover never expires and carries no growing bound.
- "Locked" is a state without a number: smoothed error below max(20 µs, path delay / 4) after at least 8 windows (P-04).
- Receive time derives from a wall-clock kernel stamp `[kernel-doc]`; a wall-clock step between stamp and read corrupts the sample.
- At handover the new holder carries the phase on but switches to its own crystal: a rate step.
- The holder rotates after 2 mandates (P-05).

Because sources pace position from network time (ADR 09), the servo is the pacing clock of the whole medianet and reaches hardware edges as resampler ratio. Inside the network, alignment is by position and servo wobble only consumes deadline margin.

## Decision

**Keep the in-band servo as the built-in, always-available discipline. Keep its estimator. Redesign its actuation. Test it in deterministic simulation.**

| Requirement | Statement |
|---|---|
| Continuity | Published time is continuous while locked: zero phase step across updates. Corrections act on frequency only |
| Slew | At most 10 ppm from the holder's rate (C-60) and 1 ppm/s change of rate (C-61) while locked. Acquisition is exempt |
| Lock is a number | The published view carries an error bound. Locked requires bound ≤ 25 µs (C-62) |
| Acquisition | Locked within 10 s of the first exchange (C-63, proposal) |
| Honest lock | Never Locked while true error exceeds the published bound |
| Pacing | Requests paced by deadline, 50 per second per follower (C-36); achieved rate equals configured rate and is reported |
| Wall clock | A wall-clock step between stamp and read is detected and the sample discarded |
| Holdover | Bound grows by 1 µs per second (C-64); alignment claims lapse after about 25 s (C-65) |
| Steps | A step larger than 2 ms (C-67) is a timeline epoch change, never slewed |
| Handover | Frequency-continuous: the new holder starts at the old rate and relaxes to its own within C-61 |
| Holder changes | Only on loss or pressure; rotation is retired |
| Margin | A consumer's headroom includes twice the published clock bound (C-52) |

The view is published through a seqlock. The seam for an external discipline is that published view; no PTP backend is built in v1.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Port the POC servo unchanged | Phase steps, timeout pacing, unbounded holdover and rate-step handover all violate requirements that follow from the servo being the audio pacing clock |
| Stand on PTP / gPTP with phc2sys | Needs privilege, capable NICs and a configured network; fails the requirements of portable one-way measurement, no root where avoidable, and cold-start election bootstrap. Not built in v1; the seam is defined |
| `CLOCK_MONOTONIC_RAW` | Not a synchronisation mechanism; at most a choice of local timebase |
| Pluggable discipline with both backends in v1 | The second backend has no v1 consumer; only the seam is kept |
| Holder rotation (POC) | Every handover is a frequency event on every stream once the clock is the audio clock |
| Tighter election silence | No accuracy pressure over seconds of holdover; false elections cost handovers |

## Consequences

Positive

- Published time is smooth enough for a cascaded edge rate loop (bandwidth well below 1 Hz, C-68) to follow.
- Lock carries a bound that consumers and sinks can budget against.
- Sync rate is what was configured.
- The estimator is a pure function over exchanges, so every requirement is testable in simulation.

Negative

- This is effectively a new servo: frequency-only actuation changes the loop dynamics the POC's gains assume. The slew limits and lock bound are unsimulated proposals.
- Correcting an error at 10 ppm is slow by construction.
- Handover continuity assumes the new holder has a good rate estimate of the old one; hence only a locked module is eligible, and otherwise the timeline epoch is bumped.

Neutral

- Two processes on one host each run their own servo and can disagree by up to twice the clock bound.

## Falsifier

- Simulation (task 008): the 25 µs bound (C-62) cannot be reached within the acquisition time (C-63) under the slew limits (C-60, C-61). This is the riskiest assumption of the whole design; E, every margin and every alignment claim sit on it.
- Simulation: a rate step at handover exceeds 1 ppm/s (AC-5).
- L2 (task 020): the edge rate loop passes servo jitter into the resampler ratio (C-68), or the wander mask (C-77) is violated.

## Links

- [01-architecture.md §2.3 Holder and election](../01-architecture.md#23-holder-and-election), [§2.4 Servo requirements](../01-architecture.md#24-servo-requirements), [§2.5 Publication](../01-architecture.md#25-publication), [§9.4 Two-loop timebase](../01-architecture.md#94-two-loop-timebase)
- [02-constants.md](../02-constants.md): C-36, C-52, C-60, C-61, C-62, C-63, C-64, C-65, C-66, C-67, C-68, C-77, P-02, P-03, P-04, P-05
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-2, AC-3, AC-4, AC-5, AC-6, AC-7
- Related: [09-Domain-line-with-epochs.md](09-Domain-line-with-epochs.md), [11-Software-resampler-edge.md](11-Software-resampler-edge.md), [14-Capability-bit-holder-eligibility.md](14-Capability-bit-holder-eligibility.md)
