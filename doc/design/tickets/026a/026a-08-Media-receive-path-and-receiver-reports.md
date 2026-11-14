# 026a-08 — Media receive path and receiver reports
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** M · **Depends on:** 026a-07, epic 009, epic 010, epic 014

## Context

Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × rate`; `T(p)` is the network time of position p.

Media facts: one packet per slice by default; slice = 48 frames = 0.5 ms (C-02) at the 96 kHz domain rate, the only rate on the wire (C-01); 6 channels (C-04); 4-byte samples (C-03); 192 bytes per channel per slice, 1 152 bytes of audio, **no per-channel id byte — the channel map is in the manifest** (C-04); about 1 320 bytes on the wire (C-20); 2 000 packets/s (C-21). The device subscribes to whole streams and extracts the channels it plays through the manifest channel map. The session number of each stream is assigned by its source and is the same for every consumer (C-19).

Liveness and alignment rules (unchanged from hosted, provided by core):
- Interrupted after 8 consecutive missed slices (C-38), 8 × 0.5 ms = 4 ms. Only slices absent at the deadline count as misses (missing, or arriving late); a marked gap or a slice on another timeline epoch resets the count (C-102).
- Dead after 2 s (C-39), counted from the last slice received with content or a marked gap (C-111). A slice on another timeline epoch also restarts the dead clock, is not a miss, and has its own counter (C-135).
- Late versus missing: an absent slice is counted missing at the deadline; if it arrives later while its position is still within the ring span it is reclassified late (missing −1, late +1) and discarded (C-108).
- Receiver report per bundle every 100 ms (C-40); receiver reports are unicast to the producer, never multicast (C-47). Producer stops after 2 s without reports (C-41).
- In-band announcements: lead 64 slices (C-43), re-issued every 16 (C-44).
- A ratchet step is a re-alignment like any other: an upstream fan-in takes it at a position it announces one lead (C-43) ahead in its own output, never silently on the next round (C-215), so the device sees an upstream ratchet step as an in-band announcement on the input it consumes. The device's own ratchet step on one of its inputs has no output to announce in: it takes the step at its own position one lead ahead (C-225), as it does for a changed A (C-214).
- Ring sized to the group's A (C-96). Persistent lateness — a late slice in each of 10 consecutive 1 s windows (C-101) — triggers a ratchet step of one slice period (C-53); at the ceiling A the input enters Demoted: treated as gapped, counted separately, cleared only by re-admission (C-109).
- An aligned multi-packet input uses the consumer's own deadline for reassembly (C-110).
- Drain per input is bounded to 2 × the admitted packets per slice for that input in each slice period (C-16, C-156): 2 packets per 0.5 ms for a one-packet-per-slice stream.

Core provides: alignment ring and gap classes (epic 009), packetiser and bundle demux (epic 010), receiver reports (epic 014).

## Work

1. In `Device::poll`, route media datagrams to core's bundle demux, drain each input at most 2 × its admitted packets per slice per slice period (C-16, C-156), keep only subscribed streams, write into a statically sized alignment ring (size from A and the declared memory limit, 026a-07).
2. Extract the played channels by the manifest channel map (no per-channel id on the wire).
3. Handle in-band announcements (lead 64, re-issue every 16), including an upstream fan-in's announced ratchet step (C-215), through core.
4. Take the device's own ratchet step at its own position one lead (C-43) ahead, unannounced (C-225).
5. Emit one receiver report per bundle every 100 ms via `Transport::send`, unicast.
6. Apply core liveness and alignment rules above; on reconnect within the dead time resume at the frozen offset; after dead, request re-admission.
7. Ring storage is a fixed array; no allocation.

## Acceptance criteria

1. Host test: a 6-channel stream at 2 000 packets/s for 10 s fills the ring with 0 gaps; a subset of channels (e.g. 2 of 6) is extracted correctly through the manifest channel map.
2. Host test: receiver reports observed at 100 ms ± 5 ms intervals per bundle, unicast to the producer.
3. Host test: 8 consecutive slices absent at the deadline mark the stream interrupted (within 4 ms of the first missing slice); 7 do not; 7 absent then a marked gap then 7 absent does not (C-102).
4. Host test: 2 s after the last slice with content or a marked gap the stream is dead (C-111); resumption within 2 s resumes at the frozen offset; after 2 s the device re-requests admission.
5. Host test: a slice arriving after its deadline but within the ring span moves one count from missing to late (C-108).
6. Host test: slices on another timeline epoch neither count as misses nor let the dead timer expire (C-135).
7. Host test: a burst of 10 packets for one one-packet-per-slice input is drained at no more than 2 per slice period (C-16, C-156).
8. Host test: late slices in 10 consecutive 1 s windows cause one ratchet step (C-101), taking effect 64 slices after the position at which it was decided, at the device's own position, with nothing announced (C-225); repeated at the ceiling A, the input enters Demoted (C-109).
9. Host test: an input carrying an upstream fan-in's ratchet step, announced one lead ahead (C-215), is handled through core's announcement path; the stream keeps playing and is not re-admitted.
10. Media and announcement parsers fuzzed at the C-95 budget.
11. Device build has no alloc / libm symbols.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] Every new factual claim carries an evidence tag

## Out of scope

- Per-connection summary counters in the stamp (026b-11).
- Resampling and audio output (026a-09, 026a-10).
- Hardware cable-pull test (026b-16).
- A change of A while the stream is live, and the current A in receiver reports (026b-13).
