# 026a-07 — Admission with declared memory limit
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** S · **Depends on:** 026a-05, epic 012

## Context

A device is a **narrow sink**: it subscribes only to the bundle members (streams) it needs. Admission (epic 012) checks **buffer memory** and refuses beyond the device's declared limit `[decision]`. On no_std, multi-packet slices and skew buffers are bounded by declared memory; admission refuses beyond it.

Settled facts:

- A stream is one packet per slice: 48 frames = 0.5 ms (C-02) at 96 kHz, the only rate on the wire (C-01); 6 channels (C-04); 4-byte samples (C-03); 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte — the channel map is in the manifest (C-04). About 1 320 bytes on the wire (C-20), 2 000 packets/s, about 21.1 Mbit/s (C-21).
- The **unit of subscription is the stream**: the device receives all 6 channels of a stream and picks the ones it plays by the manifest's channel map. A "7th channel" therefore means a second stream.
- Alignment rings are sized to the presentation group's A (C-96) and hold wire slices at the domain rate: buffer memory per stream = 6 × 4 bytes × 96 000 × A; 23 040 bytes at A = 10 ms.
- A 100 Mbit device port holds about 19 channels at the 70 % admission ceiling (C-26, C-23), so a second stream (12 channels, about 42 Mbit/s) fits the port: only the declared memory limit can refuse it.
- A sink with E unknown is refused with reason RequiredEExceeded from any presentation group that sets a required E, and admitted to groups that set none (C-103).
- The reference device subscribes to audio members only. Should a device subscribe to a controller member, admission (epic 012) costs it like an audio stream: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s (C-219); the device adds no arithmetic of its own.
- A Pending connection whose target is absent persists until the target appears or the operator cancels; it is shown as unsatisfied (C-45).

Epic criterion 026-AC-9: a request for a **7th channel** (a second stream, beyond the device's declared buffer memory) is **refused with a named reason**. The refusal reasons are complete at 18 (C-173 completed by C-185), so no new reason is added for buffer memory; which of the 18 a memory refusal carries is not named in the constants table: open item, owner epic 012 (admission), falsifier "the device's refusal is not distinguishable from another reason". This ticket uses epic 012's reason enum as it stands and does not invent a name.

The declared buffer memory is also what the commissioning tool checks before applying a change of A (C-211); the check covers sinks, the device included (C-222), and the device publishes it in its stamp (026a-04, 026a-11). A change of A while the stream is live, and the clamp when one does not fit (C-211, C-216, C-222), are 026b-13.

## Work

1. Device-side admission handler driving the core admission state machine (epic 012).
2. Declared limits come from `EmbeddedProfile` (026a-04): `buffer_memory_limit_bytes`, `max_streams`, port bandwidth (100 Mbit, 70 % ceiling).
3. Memory check: required = Σ over admitted streams of 6 × 4 × 96 000 × A (C-96) plus per-packet overhead; refuse if it would exceed `buffer_memory_limit_bytes`.
4. Port check: refuse if admitted bandwidth would exceed 70 % of the port rate (C-23).
5. E check: with E unknown, a group with a required E refuses with RequiredEExceeded (C-103).
6. Refusal reasons come from epic 012's reason enum.
7. Declared limit for the reference device: one stream at A = 10 ms (the 026-AC-2 workload, 23 040 bytes of ring).

## Acceptance criteria

1. Host test: admitting one 6-channel stream at A = 10 ms succeeds.
2. Host test: requesting a second stream (a 7th channel) is refused on buffer memory; the refusal carries a named reason from epic 012's enum, not a generic error.
3. Host test: with a declared memory limit large enough, a fourth stream (24 channels, about 84 Mbit/s, C-22) exceeds 70 % of a 100 Mbit port and is refused on port bandwidth.
4. Host test: a group with A = 20 ms requires 46 080 bytes per stream and is refused by a device declaring 23 040 bytes.
5. Host test: with E unknown, a group setting a required E refuses with RequiredEExceeded; a group setting none admits (C-103).
6. Host test: a connection request to an absent target stays Pending with no timeout (C-45).
7. Refusal does not disturb an already admitted stream (no gap in its ring during the test).
8. Admission message parsing is fuzzed at the C-95 budget.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope

- Hardware refusal test (026b-18).
- A change of A while the stream is live (026b-13).
- Receiving media (026a-08).
