# 022a-13 — WASAPI exclusive-mode sink
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** M · **Depends on:** 022a-06, 022a-07, 022a-10, 022a-11, 022a-12, epic 008a, epic 018, epic 024a

## Context
- Part of epic 022a (Windows edge backend), `wasapi/sink.rs`: exclusive-mode event-driven render; position ring → kit resamplers → device buffer.
- Edge contract (all platforms) `[decision]`: `emit(p) = T(p) + A`; `submit(p) = emit(p) − D`. A is the presentation group's audition offset (C-55); D is the declared plumbing delay. **E** = 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, maximum reported beside it (C-73).
- D is the sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter (C-170). On Windows, D is **measurement only**: reported stream latency is unreliable across drivers `[memory]`. Typical D 6 to 20 ms (C-81). The sink reads D from an edge configuration key written by commissioning; who measures D in the field is an open sub-question owned by epic 028 (inside a commissioning window, C-107). A device without that measurement is uncertifiable.
- Sinks do not pace: they consume, driven by the WASAPI event callback (no ticker).
- The wire carries 96 kHz only (C-01); a 48 kHz device is fed through the kit's fixed resampler node at 2:1 (C-80). The resampler node exists only inside an edge module (C-134) and only at 2:1 or 1:2 (C-105). Tracking the device crystal is a separate, steerable rate actuator whose ratio is the slope quotient of epic 018 (C-189). Both come from the arithmetic kit, epic 024a; this ticket builds no resampler of its own (C-197).
- Their conversion delay is part of D (C-134, C-170) and is never counted in L: L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207).
- The position ring is sized to A: the skew budget is capped at the group's A (C-96). The receiver report carries the sink's current A (C-198). When the group's A changes while the sink is live, it re-enters aligning with its ring and ratchet ceiling resized (C-186), switching at a position it picks itself, one lead (64 slices, C-43) ahead of its own position, as a fan-in does (C-208); a sink has no output to announce the position in, so it announces nothing (C-214).
- The kit's rate actuator is driven through the rate-actuator trait defined in epic 024a, with a ratio range of ±1 000 ppm around the nominal ratio (C-210).
- Counting rules: at the deadline an absent slice is counted missing; if it arrives later while still within the ring span it is reclassified late and discarded (C-108). Interrupted after 8 consecutive misses (C-38), where a miss is a slice absent at the deadline and a marked gap or a slice on another timeline epoch resets the count (C-102). Dead 2 s after the last slice with content, a marked gap (C-111, C-39) or a slice on another timeline epoch; such a slice restarts the dead clock, is not a miss and has its own per-connection counter (C-135). A sink whose source is absent stays Pending with no timeout (C-45).
- Bounded drain: per input, at most 2 × the admitted packets per slice (C-16) per slice period (C-156). The sink is driven per device callback, so a drain that covers several slice periods is bounded by the per-period bound times the number of periods it covers.
- Each consumer publishes per-connection counters in its own regime stamp: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159, completing C-47), plus the observed DSCP (C-47) and lateness percentiles 50th, 99th, 99.9th and maximum (C-160) over a rolling 60 s window, with the maximum since the connection went live beside them (C-182); the counters and percentiles come from the epic-018 edge structure, the observed DSCP from 022a-05.
- Admission: a sink whose E exceeds a group's required E is refused with RequiredEExceeded. Only the certification state *commissioned* gives a numeric E; an uncertified or self-monitored sink (not on the reference list, without a commissioned D, or not yet measured) declares E **unknown** (C-140): refused with RequiredEExceeded by any group that sets a required E, admitted to groups that set none (C-103). A *commissioned* sink whose clock is unlocked stays commissioned: its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning (C-180); the lock state comes from the published time view (epic 008a, C-187).
- Receiver report per bundle every 100 ms (C-40); slices 0.5 ms (C-02); 6 channels per stream, 192 bytes per channel per slice (C-04).

## Work
1. `wasapi/sink.rs`: `WasapiSink` built on the edge structure of epic 018.
2. Receive path: slices from `WinNet` (022a-06) into a position-addressed ring sized to A.
3. Render thread: MMCSS "Pro Audio" (022a-07); on each WASAPI event, compute the positions due at `submit(p) = T(p) + A − D` using the deadline clock (022a-12), pull them from the ring, run the kit's steerable rate actuator at the slope-quotient ratio through the kit's rate-actuator trait (C-189, C-210) and, for a 48 kHz device, the kit's fixed 2:1 node (C-80), write the device buffer. No resampler code lives in this crate (C-197).
4. Missing slices render silence; counting per C-108, C-102, C-111, C-135 through the epic-018 edge structure; drain bounded per C-16 and C-156; receiver reports every 100 ms (C-40), carrying the current A (C-198).
5. D is read from the edge configuration key (commissioned, *measured*), never from the driver's reported latency.
6. Declare at admission the certified E when the certification state is *commissioned*, else E unknown (C-140, C-103); accept refusal with RequiredEExceeded.
7. On a change of the group's A while live: pick the switch position one lead (C-43) ahead of the sink's own position, resize the ring and the ratchet ceiling to the new A and re-enter aligning at that position (C-186, C-214); emit no announcement.
8. Keep the certification state *commissioned* while the clock is unlocked; take the suspension of the alignment claim from the published time view (C-180, C-187).
9. Report the conversion delay of the kit resamplers in the sink's manifest on its device-facing port, and keep it out of the module's L (C-207).
10. Unit / integration test with a mock device (no hardware in CI).

## Acceptance criteria
1. With a mock device and a synthetic slice stream, the sink writes each position at `T(p) + A − D` within one device period.
2. 8 consecutive slices absent at the deadline raise *interrupted* (C-38); a marked gap or a slice on another epoch inside the run resets the count (C-102); 2 s after the last slice with content, a marked gap or a slice on another epoch → *dead* (C-111, C-135); each other-epoch slice increments the other-epoch counter, not the missing counter (C-135).
3. A slice arriving after its deadline but within the ring span moves from missing to late and is not rendered (C-108).
4. Receiver reports are emitted every 100 ms ± 10 ms (C-40) and carry the sink's current A (C-198).
5. D is read only from the configuration key; a test asserts the driver-reported latency is not used.
6. **(022-AC-12.)** With E unknown, a group that sets a required E refuses the sink with RequiredEExceeded and a group that sets none admits it; with a certified E above a group's required E, the refusal reason is RequiredEExceeded; an uncertified or self-monitored sink declares E unknown even when an E was measured (C-140) (test with a mock admission peer).
7. Manual: on reference hardware, a 48 kHz device plays a 96 kHz stream for 10 minutes with no underrun.
8. The fixed 2:1 node and the rate actuator are the epic-024a kit's; the crate contains no resampler implementation (C-197).
9. For a 48 kHz mock device, the manifest reports the conversion delay on the device-facing port, the module's L does not include it, and the same delay is inside D (C-207, C-170).
10. A change of A while live resizes the ring and ratchet ceiling and the sink re-enters aligning at a position exactly one lead (64 slices, C-43) ahead of its own position at the time it learned of the change, and sends no announcement (C-186, C-214) (mock).
11. With the clock forced Unlocked, a *commissioned* sink stays *commissioned* and its alignment claim is suspended; on relock the claim resumes with no re-commissioning (C-180) (mock time view).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Measuring E, D restart spread and skew (022b-02, 022b-03).
- Idle CPU check (022a-18).
- Capture (022a-14).
- The admission and Pending state machines themselves (core, epic 005).
