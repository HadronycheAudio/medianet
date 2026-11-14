# 007a-07 — Valid vectors: content messages and slice checksum
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** M · **Depends on:** 007a-05

## Context
- The generator `gen` (in the `hadronyche-tools` crate, C-133) (from 007a-05) derives wire vectors from the machine-readable tables of the wire specification. Each message type needs at least one minimal, one typical and one maximal valid encoding with its decoded structure.
- Content message types `[decision]`:
  - audio at 4 bytes per sample, f32 or Q1.31, with no 3-byte variant (C-03);
  - dense controllers;
  - sparse keyed controllers;
  - read-only transport state.
- Every stream is at the domain rate of 96 000 Hz, the only rate on the wire (C-01); a slice is 48 frames (0.5 ms) at defaults (C-02).
- Audio carries no per-channel id byte; the channel map is in the manifest. One channel is 192 bytes per slice and the default 6-channel slice carries 1 152 bytes of audio (C-04).
- Every slice carries a slice checksum; it is the transport-integrity tier (bytes delivered equal bytes sent, per slice) `[decision]`.
- Sizes: the default slice fits one packet at the path MTU (C-05): at most 1 472 bytes of UDP payload on a 1 500-byte path, about 1 320 bytes on the wire including all Ethernet overhead (C-20). IPv4 only (C-100). Fallback path MTU 1 280 bytes, giving 1 252 bytes of UDP payload (C-06, C-100). A multi-packet slice is an explicit, stamped regime (C-05). Header cap: a default slice must fit 1 252 bytes of UDP payload, 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum (C-137). At most 32 members per bundle, the controller member included: a source with controllers has at most 31 audio members (C-08, C-209). At most 256 streams per source (C-09).
- Where controllers travel (C-181, C-205): a bundle source's controllers always ride in their own member stream. A single-stream source's controllers share the audio packet while headers plus controllers stay within the 100-byte cap of C-137 (with the checksum); beyond that the source emits a second stream for them.

## Work
1. Extend `gen` to emit minimal, typical and maximal vectors for: audio f32, audio Q1.31, dense controllers, sparse keyed controllers, read-only transport state.
2. The typical audio vector is the default slice (6 channels, 48 frames, 1 152 bytes of audio, one packet). The maximal vectors include one sized for the 1 472-byte payload limit, one whose UDP payload fits 1 252 bytes (fallback path), and a multi-packet slice (packet count > 1).
3. Emit the controller forms in both packagings of C-181: a controller-only member stream of a bundle source (no audio), and a single-stream default slice carrying audio plus controllers whose headers, controllers and checksum together take at most 100 bytes.
4. Every vector carries a correct slice checksum; the `.json` records the checksum value.
5. Commit the files to `vectors/wire/valid/`.

## Acceptance criteria
1. At least 15 vectors exist: 3 each for the 5 content forms listed in step 1.
2. The default-slice audio vector has 6 channels, 48 frames, exactly 1 152 bytes of audio with no per-channel id byte, packet count 1 and a UDP payload of at most 1 252 bytes (C-137).
3. Every audio vector's audio length equals channels × frames × 4 bytes.
4. At least 1 vector has a UDP payload of at most 1 252 bytes, and at least 1 vector has packet count ≥ 2.
5. A test recomputes the checksum of every content vector and finds it equal to the recorded value.
6. All files carry `spec_version` and pass the 007a-02 schema test.
7. At least 1 dense and 1 sparse controller vector is a controller-only member stream carrying no audio, and at least 1 single-stream default slice carries audio plus controllers with headers, controllers and checksum at most 100 bytes and a UDP payload of at most 1 252 bytes (C-137, C-181).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Wrong checksums and other invalid encodings (007a-11).
- Reassembly behaviour of multi-packet slices (007b-10).
- Transport-state semantics over time (007b-15).
- Reference audio processing hashes (007b-18).
