# 018-13 — AlsaSink position ring and submit scheduling
**Epic:** 018-Linux-ALSA-edge · **Effort:** M · **Depends on:** 018-12, epic 009

## Context
- A sink is a consumer with a deadline: it holds its input in a **ring indexed by position** (alignment ring, epic 009), **sized to the group's A** (C-96), and takes position p at a fixed time, with whatever arrived. Absent slices are gaps; default concealment for audio is **zero-fill**.
- Edge contract `[decision]`: `emit(p) = T(p) + A`; `submit(p) = emit(p) − D`. A is the operator-configured audition offset of the presentation group; it has **no default value** (C-55). The epic settles that a sink with no A configured does not start emitting and reports a named reason until the operator sets A.
- On a **timeline-epoch change** the sink restarts alignment at the announced position.
- Slices are 48 frames = 0.5 ms at the 96 kHz domain rate (C-01, C-02).
- Input health (missing, late, interrupted, dead, Demoted) is ticket 018-22.
- Contribution to AC-26 (verified by epic 020): zero slip — offset returns within ±1 sample after injected loss and after an announced step (C-76).

## Work
1. Configuration requires A (ns) for the sink's presentation group; without it the sink stays not started with reason "audition offset not configured" (C-55).
2. Size the input ring to cover A (C-96) at the 96 kHz domain rate, in fixed memory allocated at start.
3. In `sink.rs`, for each period: compute the position p whose `submit(p)` falls at the period's hand-off instant (from `EdgeTimebase`/`TimeView` and D from 018-11); read slices for that range from the ring; feed the resampler.
4. Gaps → zero-fill (classification and counting in 018-22).
5. Epoch change: flush, restart alignment at the announced position.

## Acceptance criteria
1. With a fake clock and backend, slices injected at positions p are emitted (fake backend timestamp + D) at T(p) + A within 1 sample.
2. Dropping 1 slice zero-fills exactly 48 frames and the offset returns within **±1 sample** afterwards; the same holds under the L2 loss pattern of C-166 and C-191 (1 % random plus bursts of 8 slices spaced 1 s apart, on the media path only) on the fake backend.
3. An announced timeline-epoch step of 5 ms (C-191): after the step, the offset is within ±1 sample of the new alignment.
4. Missing A → the sink does not start and reports "audition offset not configured"; setting A then starts it without restart of the process.
5. The ring span equals A rounded up to whole slices (C-96), asserted for three values of A.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Missing/late/interrupted/dead/Demoted classification (018-22).
- Re-alignment on a change of A while the input is live (018-28, C-186).
- Rate control and E self-monitoring (018-14).
- The alignment ring itself (epic 009).
