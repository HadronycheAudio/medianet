# 023-20 — Admission figures
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-07, epic 012

## Context
- Manifest admission figures `[decision]`: bit/s on the wire, packets/s, input count, buffer memory, in the admission units defined by epic 012. They cover **network ports only**; the device-facing side of an edge module (023-29) does not touch the wire.
- 023-AC-7: admission figures for a 6-channel stream are **about 21.1 Mbit/s and 2 000 packets/s (C-21)**; a 96-channel port is 16 streams, **32 000 packets/s and about 338 Mbit/s (C-22)**.
- Derivation: the domain rate, 96 kHz, is the only wire rate (C-01); slice 48 frames = 0.5 ms (C-02) ⇒ 2 000 slices/s, one packet per slice per stream (C-05); 4-byte samples (C-03); 6 channels per stream, 192 bytes per channel per slice, **1 152 bytes of audio per packet with no per-channel id byte** (C-04). One default packet is **about 1 320 bytes on the wire including all Ethernet overhead** (C-20; IPv4, C-100), so 1 320 × 8 × 2 000 ≈ 21.1 Mbit/s (C-21). The difference, about 168 bytes per packet, is header and framing overhead. C-137 caps all headers and the checksum at 100 bytes, so a default slice fits 1 252 bytes of UDP payload (1 152 + 100).
- Remaining gap (owner: epic 012 with epic 004): the field-by-field header breakdown inside the C-137 cap, and the admission units for input count and buffer memory. Alignment rings are sized to the presentation group's A (C-96), which is not known at compile time, so the buffer figure cannot be an absolute byte count from the compiler alone; use epic 012's unit when it exists.
- Controllers of a single-stream source (C-181, decided, revised: permissive): **the source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit. The compiler checks this from the manifest.** The check and the second stream's figures are 023-31; this ticket does not count a controller port of a single-stream source as a stream of its own. Controllers of a bundle source (C-205, decided): **always in their own member stream** (C-181); that member stream's figures are 023-32.
- Wider ports are bundles of up to 32 member streams (C-08), the controller member included (C-209, so at most 31 audio members with controllers; rejection in 023-32); figures scale per member stream; at most 256 streams per source (C-09).

## Work
1. `AdmissionFigures { wire_bits_per_s, packets_per_s, input_count, buffer }` computed per network port and summed per module, using the per-packet wire size from epic 012/004 (C-20); import it, do not duplicate a magic number.
2. Exclude device-facing ports of edge modules. Controller ports of a single-stream source add no stream here; their placement and any second stream come from 023-31 (C-181); the controller member stream of a bundle source comes from 023-32 (C-205).
3. Input count and buffer memory in the units epic 012 defines; until then, a documented placeholder with owner epic 012 and falsifier "admission figures computed from the manifest disagree with epic 012's admission check".

## Acceptance criteria
1. 6-channel stream: packets/s == 2 000 exactly; wire bit/s within ±1 % of 21.1 Mbit/s (C-21).
2. 96-channel port (16 members): packets/s == 32 000 and wire bit/s == 16 × the 6-channel figure, within ±1 % of 338 Mbit/s (C-22).
3. An edge module's device-facing ports contribute 0 to every figure.
4. input_count equals the number of network input ports (bundle counted as specified by epic 012).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Admission decisions themselves (epic 012); the manifest struct (023-19).
