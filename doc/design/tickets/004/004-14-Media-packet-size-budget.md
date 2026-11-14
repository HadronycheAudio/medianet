# 004-14 — Media packet size budget at 1 472 and 1 280 bytes
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-06, 004-08, 004-09, 004-10, 004-11, 004-13
## Context
`wire.md` must show that a default packet fits. Facts `[decision]`:
- C-100: IPv4 only in v1. A 1 500-byte path leaves 1 472 bytes of UDP payload (20 bytes of IPv4 header without options, 8 bytes of UDP header).
- C-04: 6 channels per stream at defaults; 192 bytes per channel per slice (48 × 4, no per-channel id byte; the channel map is in the manifest); 1 152 bytes of audio. This leaves about 320 bytes for all headers, announcements, the measurement extension and the checksum.
- C-05: the default slice fits one packet at the path MTU.
- C-06: path-MTU fallback 1 280 bytes, which over IPv4 leaves 1 252 bytes of UDP payload (C-100, C-17).
- C-137: a default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path. So 6 channels fit at the fallback by rule; the budget proves the cap holds.
- C-205: a bundle source's controllers always ride in their own member stream, so the budget of a bundle source's audio member counts no controller messages.
- C-181 (revised: permissive): a single-stream source may always put its controllers in a second stream, and must when headers plus controllers exceed the 100-byte cap; it may share the audio packet while they fit. So when a single-stream source shares its packet, the controller messages are counted inside the 100-byte cap, and the bytes left for them are the cap minus all headers and the checksum; that allowance is the most a source may share, not an amount it must share.
- C-43/C-44: announcements are re-issued every 16 slices within a 64-slice lead, so packets may carry announcements routinely.

Open sub-question owned by epic 004:

| Question | Owner | Falsifier |
|---|---|---|
| Do all headers and the checksum fit the 100 bytes of C-137, and which announcements and measurement-extension bytes fit beside them on a 1 472-byte path? | This task | Headers and checksum exceed 100 bytes (C-137 falsified; C-04 falls to 5 channels at the fallback) |

## Work
1. Add a "Size budget" section to `wire.md` with a table: packet header, slice header, content-message header(s), 6 × 192 audio bytes, maximum announcements, measurement extension, checksum, total, spare bytes vs 1 472.
2. Show the budget both with and without the optional measurement extension and with the maximum announcement load.
3. Repeat at the 1 280-byte fallback (C-06): 1 252 bytes of UDP payload over IPv4 (C-100); show that 6 channels (1 152 bytes) plus all mandatory headers and the checksum fit, with headers and checksum at most 100 bytes (C-137), and list which announcements and measurement-extension bytes are omitted there.
4. State the bytes the 100-byte cap leaves for a single-stream source's controller messages beside all headers and the checksum (C-181), at both path sizes, as the upper bound for sharing the audio packet (a source may always use a second stream instead).
5. Answer the open sub-question; if headers and checksum exceed 100 bytes, report C-137 as falsified (and C-04, which then falls to 5 channels at the fallback) and update the constants table.
6. Copy the totals into the machine-readable file (or derive them in the 004-02 checker) so they stay consistent with the tables.

## Acceptance criteria
1. The default-packet budget shows 6 channels plus all mandatory headers fit in 1 472 bytes and states the spare bytes as a number (epic AC-2).
2. The worst case (all optional parts at maximum) is also shown with its spare or deficit.
3. The 1 280-byte fallback budget uses 1 252 bytes of UDP payload, shows 6 channels fit with headers and checksum at most 100 bytes (C-137), and lists the optional parts omitted there.
4. The open sub-question is answered with a verdict against its falsifier.
5. Totals in `wire.md` match the sums from the machine-readable tables (checked by the 004-02 checker or by hand with the arithmetic shown).
6. The controller allowance inside the cap for a single-stream source (C-181) is stated as a number.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Ethernet wire size and bit rate recompute (004-15).
- Control-message size bounds (004-16 to 004-18).
