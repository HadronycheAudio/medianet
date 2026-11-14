# 028a-02 — Capacity arithmetic module
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** S · **Depends on:** 028a-01

## Context
The commissioning tool validates link loads offline and alarms on them post hoc. All of that rests on one set of wire-rate constants, which this ticket encodes once, as a pure library module, so `validate` (028a-06, 028a-07), `watch` (028b-17) and `soak` (028b-16) share it.

Constants (ids from the design constants table, `doc/design/02-constants.md`):
- C-01: 96 kHz is the only sample rate on the wire; an edge whose hardware runs at 48 kHz converts at the edge (C-80). The module has one rate.
- C-02: 48 frames per slice = 0.5 ms = 2 000 slices/s.
- C-03: 4-byte samples.
- C-04: default stream = 6 channels; 192 bytes per channel per slice, 1 152 bytes of audio per packet; no per-channel id byte (the channel map is in the manifest).
- C-20: about 1 320 bytes per packet on the wire, including all Ethernet overhead.
- C-21: 2 000 packets/s per stream, about 21.1 Mbit/s per stream, about 3.5 Mbit/s per channel.
- C-22: per source: 24 channels = 4 streams = 84 Mbit/s = 8 000 packets/s; 48 = 8 streams = 169 Mbit/s = 16 000 packets/s; 96 = 16 streams = 338 Mbit/s = 32 000 packets/s.
- C-23: admission ceiling = 70 % of the link rate.
- C-24: GbE port wall about 198 channel-copies (8 × 24, 4 × 48 or 2 × 96 channels).
- C-25: any port carrying more than the GbE wall runs 10 GbE.
- C-26: a 100 Mbit device port holds about 19 channels.
- Multicast fan-out is not available in v1: every consumer of a source is a separate copy on the source's port.
- These rates are audio streams only. A bundle source's controllers ride in their own member stream (C-181, C-205), which counts towards the 32 members of C-08 (C-209); a single-stream source's controllers share the audio packet within the 100-byte header cap and beyond it get a second stream (C-181, C-137). A source with controllers therefore has at most 31 audio members, 186 channels (C-209).
- C-219: the wire cost of a controller stream is computed like an audio stream: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s. Admission (epic 012) and the commissioning tool use this same figure.

## Work
- Module `hadronyche_tools::capacity` with:
  - constants above as named items with their C-nn ids in doc comments;
  - `streams_for(channels) -> u32` (ceil(channels / 6));
  - `packets_per_s(channels) -> u32` (streams × 2 000);
  - `wire_mbit_s(channels) -> f64` (packets/s × 1 320 × 8 / 1e6);
  - `ceiling_mbit_s(link_rate_mbit) -> f64` (0.70 × rate);
  - `channel_copies_ceiling(link_rate_mbit) -> u32` (floor(ceiling / per-channel rate));
  - `controller_stream_mbit_s(header_bytes, controller_bytes) -> f64` ((header_bytes + controller_bytes + 66) × 8 × 2 000 / 1e6, C-219) and its packet rate, 2 000 packets/s;
  - `max_audio_channels_with_controllers() -> u32` (31 members × 6 = 186, C-209);
  - a `PortLoad` accumulator: add a source copy (channels, and its controller stream if any), report total Mbit/s, packets/s and channel-copies.
- Unit tests asserting every number listed in Context.

## Acceptance criteria
1. `wire_mbit_s(6)` is 21.1 ± 0.1; per-channel rate is 3.5 ± 0.1 Mbit/s; audio bytes per default packet are 6 × 192 = 1 152.
2. `wire_mbit_s(24)`, `(48)`, `(96)` round to 84, 169 and 338 Mbit/s (C-22); `packets_per_s` gives 8 000, 16 000 and 32 000.
3. `ceiling_mbit_s(1000)` = 700; `channel_copies_ceiling(1000)` is 198 ± 1; `channel_copies_ceiling(100)` is 19.
4. Three copies of a 96-channel source sum to 1 014 Mbit/s (rounded), which exceeds the GbE ceiling of 700 Mbit/s; two copies sum to 676 Mbit/s (C-92).
5. A controller stream with 100 header bytes and 400 controller bytes per slice costs (100 + 400 + 66) × 8 × 2 000 = 9.056 Mbit/s and 2 000 packets/s (C-219); a 96-channel bundle with that controller member costs the 96-channel figure plus exactly this, and 34 000 packets/s.
6. The module performs no I/O and has no notion of a second sample rate.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Applying the arithmetic to a configuration (028a-06, 028a-07); observed-rate alarms (028b-17).
- Ring memory for the buffer-memory check (028a-10) is the alignment-ring size of C-27 scaled by channels and A; it lives with that check.
