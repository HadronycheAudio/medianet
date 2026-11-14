# 018-17 — AlsaCapture bundle above 6 channels
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-16, epic 004

## Context
- `AlsaCapture` **emits a bundle when wider than 6 channels** (deliverable of epic 018). Up to 6 channels it emits one plain stream (018-16).
- A stream carries at most 6 channels: 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte; the channel map is in the manifest (C-04). A bundle has at most 32 members, a controller member included (C-08, C-209). The capture has no controllers, so all 32 members can be audio and one capture module carries at most 192 channels.
- Slices: 48 frames = 0.5 ms at the 96 kHz domain rate (C-01, C-02), 4-byte samples (C-03). Example: 24 channels = 4 streams (C-22).
- The bundle wire format (shared position per slice, member order) is owned by the wire specification, epic 004; it is a remaining gap of this epic until 004 publishes it. Use the runtime's bundle type.

## Work
1. When the configured channel count is > 6, split channels into ⌈N/6⌉ member streams of the runtime's bundle type, with one shared position stamp per 48-frame slice.
2. Refuse a configuration above 192 channels (C-08) with a named reason.
3. Write the full channel map (capture channel → member, slot) into the manifest (C-04).
4. Up to 6 channels, keep one plain stream.

## Acceptance criteria
1. 6-channel capture → one plain stream; 7-channel → bundle of 2 members; 24-channel → bundle of 4 members.
2. All members of a bundle carry the same position, equal to the position of the equivalent plain slices.
3. Channel order is preserved end to end through the manifest channel map (test with per-channel distinct tones).
4. 193 channels → refused with a named reason; 192 channels → bundle of 32 members.

## Definition of done
- [ ] Tests green in CI, unprivileged

## Out of scope
- Defining the bundle wire format (epic 004).
