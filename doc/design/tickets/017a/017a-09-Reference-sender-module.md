# 017a-09 — Reference sender with replicas and comparison
**Epic:** 017a-Reference-modules · **Effort:** M · **Depends on:** 017a-02, 017a-03, 017a-05, 017a-06, 017a-07, 017a-08, epic 010, epic 015

## Context
- POC sender (`src/sender.rs`) `[code]`: owns the media clock; at each block it prepares (`:209`) and fires (`:290`) two datagrams with the same header — musical-event parts to the synths, a multi-channel audio feed to the mixer — and compares what comes back (`:368`). Most of `sender.rs` was never read by the design; epic 002 reads it.
- Rules `[decision]`:
  - The sender's **replica** runs the same echo and mixer code and compares bits (exact tier); it never applies gaps (the received-positions bitmap is deferred past v1).
  - Transport **authoring** stays in the reference sender only.
  - No libm in the signal path; hand-written Rust first.
- Settled figures: slice = 48 frames = 0.5 ms at 96 kHz, the only rate on the wire (C-01, C-02); 4-byte samples (C-03); 6 channels per stream: 192 bytes per channel and 1 152 bytes of audio per slice, **no per-channel id byte**, the channel map is in the manifest (C-04). The default slice fits one packet at the path MTU; a multi-packet slice is an explicit, stamped regime (C-05). IPv4 only (C-100).
- **Header cap (settled, C-137):** a default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path. The packetiser (epic 010) enforces this; the sender's tests check it on its own streams.
- **Where controllers travel (settled, C-181):** the sender is a single-audio-stream source (one 6-channel feed), so its controllers share the audio feed packet while headers plus controllers stay within the 100-byte cap (C-137); beyond that they go in a second stream. The mixer's volume rides in the audio feed packet; the parts for the echo are the sender's second, controller stream. The compiler's manifest check does not apply to a hand-written module, so the sender's own tests check it.
- **Session number (settled, C-19):** assigned by the source, one per stream; bumped on restart and on timeline epoch change; every consumer of a stream sees the same value.
- `position = (network_time − epoch_origin) × rate`.
- Scenario 017b-14 needs the sender to emit **marked gaps** on command, to prove that a marked gap resets the interrupted count (C-102) and defers the dead threshold (C-111).

## Work
1. Implement `hadronyche-reference/src/sender.rs` against the module API of epic 015, emitting per slice position: a parts stream (from `instrument::phrase`, 017a-02) and a 6-channel audio feed stream, both via the packetiser of epic 010. The audio payload is 1 152 bytes per slice with no per-channel id byte; the channel map goes in the manifest. The mixer's volume (dense controller) rides in the audio feed packet within the 100-byte cap; the parts travel in their own controller stream (C-181).
2. Assign and maintain one session number per stream (C-19).
3. Own transport authoring via the authoring interface of 017a-03.
4. Run in-process replicas of `echo` (017a-07) and `mixer` (017a-08) on the same inputs; feed received outputs and replica outputs into the analyzer (017a-04, 017a-05, 017a-06).
5. A harness control to emit a marked gap instead of content for a given stream and position range.
6. Expose the `analyzer::Report` to the harness.
7. Header comment citing POC `src/sender.rs` and the epic 002 finding.

## Acceptance criteria
1. Sender → echo → sender and sender → mixer → sender on loopback with no injection for 10 s: exact-tier mismatches = 0, and the `audio`, `music` and `transport` families each have > 0 compared windows.
2. Both streams for a given position carry the same position value.
3. Each audio slice payload carries exactly 1 152 bytes of audio for 6 channels and no per-channel id byte; the manifest carries the channel map; every default audio packet, announcements included, has at most 1 252 bytes of UDP payload, that is at most 100 bytes of headers and checksum (C-137); the volume controller is carried in the audio feed packet within that cap, and the manifest shows the parts in a separate controller stream (C-181).
4. Restarting the sender bumps every stream's session number; consumers report the same value as the sender (C-19).
5. A commanded marked gap reaches the consumer as a marked gap, not as missing.
6. Echo and mixer cannot author transport (enforced by 017a-03's API split).
7. The sender's replica output for 10 s is bit-identical across two runs with the same seed.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it

## Out of scope
- The L0 binary and scenarios (017b-09 onwards).
- Cross-machine bit-exactness (017a-10).
- Epoch-change restart behaviour as a scenario (017b-20).
