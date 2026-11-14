# 004-11 — Content messages: audio and dense controllers
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-08
## Context
After the slice header, a slice carries content messages (rule 5 `[decision]`): audio, dense controllers, sparse keyed controllers, read-only transport state. This ticket defines the common content-message header plus audio and dense controllers; 004-12 defines the other two.

Facts `[decision]` unless tagged:
- C-01: 96 000 Hz is the only rate on the wire; an edge whose hardware runs at 48 kHz converts 2:1 at the edge (C-80), through a resampler node that may appear only inside an edge module (C-134). No sample-rate field is needed.
- C-02: slice length 48 frames (0.5 ms at 96 000 Hz).
- C-03: 4 bytes per sample, f32 or Q1.31; no 3-byte variant.
- C-04: 6 channels per stream at defaults; 192 bytes per channel per slice = 48 × 4; 1 152 bytes of audio for 6 channels. There is no per-channel id byte: the channel map is in the manifest (laid out in 004-26), and channels appear in the message in manifest order.
- C-158: each dense controller declares an initial value in the manifest (default 0, laid out in 004-26), used when a gap precedes any received value; the dense controller message carries no initial-value field.
- C-205: the controllers of a bundle source always ride in their own member stream; an audio member stream of a bundle source carries no controller messages, and the controller member stream carries no audio. The controller member counts towards the 32 members of C-08, so a source with controllers has at most 31 audio members (C-209).
- C-181 (revised: permissive): a single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap of C-137; it may share the audio packet while they fit. The second stream carries controllers and no audio. The compiler checks this from the manifest. A controller second stream whose controllers would have fitted in the audio packet is valid.
- C-219: a controller stream's wire cost is computed like an audio stream's, one packet per slice at 2 000 packets/s (stated in 004-15).
- C-137: the content-message header(s) count against the 100-byte cap for all headers and the checksum; when a single-stream source shares its audio packet with its controllers, the controller messages count against it too (C-181).
- The POC's 8-byte message header `Msg` (kind in the high 4 bits, channels, count, length, signed offset) is a reasonable starting point `[code]`.

## Work
1. Define the common content-message header (kind, length, and whatever else is needed for skipping unknown kinds within a major version), starting from the POC `Msg` shape; state its size.
2. Audio message: per channel frames × 4 bytes, no id byte (C-04); sample format (f32 or Q1.31) signalling; channel ordering as given by the manifest's channel map; maximum channel count per message.
3. Dense controllers: one value per frame (or per stated sub-rate) per controller, width and format, maximum count.
4. Validity rules with outcomes: unknown kind, length not matching frames × channels × 4 (`BadLength`), channel count different from the manifest's channel map for the stream, sample-format value invalid, message overrunning the slice, audio in a gap slice, a dense controller message in an audio member stream of a bundle source or audio in its controller member stream (C-205), a packet carrying audio whose headers plus controller messages exceed the 100-byte cap (C-181), audio in a single-stream source's controller second stream. A controller second stream is never rejected because its controllers would have fitted in the audio packet (C-181, permissive).
5. No constant gives the field set of dense controllers; decide it and record it as an open item owned by epic 004 (falsifier: a producer cannot express a value it must send without a major change).
6. Confirm that 6 channels × 192 = 1 152 bytes holds with the chosen layout, or state the new figure.
7. Add tables to the machine-readable file.

## Acceptance criteria
1. Common content-message header table with size in bytes.
2. Audio layout reproduces 192 bytes per channel per default slice (48 × 4, no id byte) and 1 152 bytes for 6 channels, or states and justifies the difference.
3. Dense controller layout has width, format and maximum count.
4. At least 6 invalid forms with outcomes; no validity rule rejects a single-stream source's controller second stream because its controllers would have fitted the audio packet (C-181).
5. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Sparse keyed controllers and transport state (004-12).
- Checksum (004-13) and the size budget (004-14).
