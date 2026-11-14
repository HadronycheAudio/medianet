# 023-31 — Controller placement of a single-stream source (C-181)
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-19, 023-20, epic 004

## Context
- Rule (C-181, decided, revised: permissive): **a single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit. The compiler checks this from the manifest.** The earlier reading "share while within the cap, second stream only beyond it" is replaced: within the cap the placement is the source's choice, not computed.
- C-137: a default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum.
- The check reads the manifest (023-19): the source's network output ports, their content types (dense controllers, sparse keyed controllers, 023-06), its audio stream count (023-07) and the placement the source chose. A single-stream source is one whose audio travels in one stream.
- How the source chooses: an opt-in construct for a second controller stream, part of the grammar outline of 023-01 (implemented by 023-03 and 023-04). Without it, the compiler shares the packet while headers plus controllers fit and emits the second stream beyond the cap, as C-181 requires; with it, the second stream is emitted whatever the byte sum.
- The second stream, when emitted, is on the wire, so it changes the admission figures (023-20): one more stream's packets/s and bit/s.
- Inputs not settled here: the per-packet header bytes inside the cap and the controller encoding sizes per slice come from epic 004 (content message definitions) and remaining gap 2 (header breakdown, owner epic 012 with epic 004); import them, do not duplicate a magic number. A bundle source's controllers always ride in their own member stream (C-181, C-205); that case is 023-32, not this ticket.

## Work
1. `manifest::controller_placement(&Manifest) -> ControllerPlacement { SharedWithAudio, SecondStream }` for a single-stream source with controller outputs: `SecondStream` when the source opted in to a second stream, or when headers plus the controllers' bytes per slice exceed the 100-byte cap (C-137, C-181); otherwise `SharedWithAudio`. Record the placement, and whether it was chosen or forced by the cap, in the manifest (023-19).
2. With `SecondStream`, add one stream to the source's admission figures (023-20) and count it against the C-09 stream limit.
3. Show the placement, the byte sum behind it and whether it was chosen or forced in the readable dump (023-19).
4. Until epic 004 supplies the header and controller sizes, a documented placeholder with owner epic 004 (with 012) and falsifier "the placement computed from the manifest disagrees with the packets the source sends".

## Acceptance criteria
1. A single-stream source without the opt-in whose headers plus controllers are within 100 bytes → `SharedWithAudio`; admission figures equal those of the audio stream alone (2 000 packets/s for one stream, C-21).
2. The same source with controllers added until headers plus controllers exceed 100 bytes → `SecondStream` (forced by the cap); packets/s is 4 000 and bit/s counts the second stream.
3. The boundary case at exactly 100 bytes without the opt-in → `SharedWithAudio` (C-181: "while they fit").
4. The source of criterion 1 with the opt-in → `SecondStream` (chosen), accepted, packets/s 4 000 (C-181: "may always").
5. A source with no controller output → no placement record and figures unchanged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The header field layout (epic 004 with 012); controllers of a bundle source (023-32, C-205); the source's packetisation at runtime (other epics).
