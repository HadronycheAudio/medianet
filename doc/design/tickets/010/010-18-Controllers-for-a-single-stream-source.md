# 010-18 — Controllers for a single-stream source
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-04, 010-06, epic 004
## Context
- In a bundle, controllers always ride in their own member stream (C-205, cited by C-181).
- A source with a single 6-channel stream has no bundle. C-04: 6 channels × 192 bytes = 1 152 bytes of audio. C-100: at the 1 280-byte fallback the payload is 1 252 bytes, leaving 100 bytes; C-137 makes those 100 bytes the cap for all headers and the checksum.
- Multi-packet regime: transport state, controllers and announcements go in packet 0.
- Settled by C-181 (revised, permissive): a single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit. Sharing is an option, not a default the packetiser imposes: the placement is declared in the manifest and the compiler checks it. The cap applies at every path MTU, not only at the fallback, so the about 320 spare bytes at MTU 1 500 (C-04) do not widen it. No controller byte rate or "dense" threshold is needed: the test is bytes per slice against the cap. C-158 (initial value of a dense controller) is unaffected.
- The second stream is laid out as a 2-member bundle: the audio member plus a controller member with the controller content type, as in 010-04. Its wire cost follows C-219 (010-24).

## Work
- Add a layout constructor for a single-stream source with controllers that takes the placement declared in the manifest (shared or second stream), the controller bytes per slice and the header budget h (epic 004). Second stream: it always returns a 2-member bundle (audio member plus controller member, 010-04), whatever the controller bytes. Shared: when h + controller bytes ≤ `HEADER_CAP_BYTES` (100, C-137) it returns the single-stream layout with controllers in the audio packet; otherwise it refuses with an error, without panic, and never silently switches placement. Cite C-181 in the doc comment. The manifest check itself belongs to the compiler, not to this epic.
- In the shared case, let `SliceWriter` write controller content into the audio packet; controller bytes count against the cap before the optional in-band items, which keep fitting in what remains or are omitted on a fallback path (C-137, 010-23).
- Write the rule in the design document citing C-181 (second stream always allowed, required beyond the cap, sharing allowed while it fits), with the spare-byte figures for controllers per slice (100 − h) computed from the epic 004 header budget.
- Test both layouts.

## Acceptance criteria
1. The design document states the C-181 rule, the controller room per slice (100 − h, from the epic 004 header budget) and cites C-181 and C-137.
2. With placement shared and controller bytes per slice of 100 − h, the constructor returns the single-stream layout, and a test encodes a 6-channel slice plus that controller content as one packet whose UDP payload stays within 1 252 bytes at the 1 280 fallback and within 1 472 bytes at MTU 1 500; the controllers are never omitted.
3. With placement second stream and controller bytes per slice of 1, of 100 − h and of 100 − h + 1, the constructor returns a layout with exactly 2 members, the controller member has the controller content type and 0 audio channels, and the audio packet carries no controller content (C-181).
4. In the shared case on a fallback path, pending announcements and the measurement extension that do not fit beside the controllers are omitted (010-23), and the controllers stay.
5. With placement shared and controller bytes per slice of 100 − h + 1, the constructor returns an error without panic and builds no layout (C-181).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Controller message formats (epic 004).
- Controller placement in multi-member bundles (always a dedicated member, C-205, 010-04).
- The compiler's manifest check of the declared placement (C-181).
- The wire cost of the controller stream (C-219, 010-24).
