# 020-03 — Position marker generation
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-01, epic 017

## Context
In the Medianet, modules exchange slices addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × rate`; `T(p)` is the network time of position p `[decision]`. A sink emits sample p at `emit(p) = T(p) + A`, where A is the presentation group's audition offset `[decision]`.

The L2 procedure has the sink under test play a stream containing **position markers** — a known impulse or code at known positions — so that the harness can later find each marker in capture and compare its arrival time with `T(p) + A` `[decision]`. The marker stream is produced by the reference source module (epic 017).

The marker stream is at the domain rate, 96 kHz, which is the only rate on the wire (C-01); one sample is 10.4 µs `[decision]`. Detection (020-04) must resolve markers to well below one sample, so the marker must be designed for sub-sample interpolation.

## Work
- In `marker.rs`, choose and document the marker waveform. The epic leaves the choice to this task ("a known impulse or code", recorded with its reason in `marker.rs`); pick one that supports sub-sample timing (for example a band-limited pulse or a short pseudo-random code with a sharp autocorrelation peak) and record the choice and its reason as a `[decision]` in the module doc.
- Implement a marker schedule: marker positions p_k at a fixed period (configurable), with an identifier so a detected marker can be matched unambiguously to its position even after a slip or a restart.
- Implement a generator that, given a position range, returns the samples of the marker stream (silence or a low-level carrier between markers, as decided).
- Hook the generator into the reference source module of epic 017 so the sink under test receives the marker stream as an ordinary slice stream.
- Export the schedule (position, id) so it can be written into a recording (020-02).

## Acceptance criteria
1. For any position range, the generator output is deterministic (same range, same samples) and continuous across range boundaries.
2. Each marker carries an identifier unique within at least 1 hour of stream at 96 kHz.
3. The marker's autocorrelation (or impulse shape) has a single main peak; the ratio of main peak to largest side peak is documented with its measured value `[measured]`.
4. The reference source module emits the marker stream at 96 kHz, verified by a unit test that reads back positions and identifiers.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Marker waveform choice documented with its reason

## Out of scope
- Detection and sub-sample interpolation (020-04).
- A pure-tone stream for the spectral criterion (020-20).
- Changes to the injection API itself (epic 017).
