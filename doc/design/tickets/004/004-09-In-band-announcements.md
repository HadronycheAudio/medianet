# 004-09 — In-band announcement layouts
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-06, 004-08
## Context
Media packets may carry optional in-band announcements (rule 3 `[decision]`): **end-of-stream**, **re-alignment**, **timeline epoch switch**, **transport reference change**. Each names a future position (64-bit position on the domain line, as in the slice header).

Constants `[decision]`:
- C-43: in-band announcement lead is 64 slices, flat (at C-02 defaults, 64 × 0.5 ms = 32 ms).
- C-44: announcements are re-issued every 16 slices until the named position passes, in every bundle member. C-08: at most 32 members per bundle.
- Timeline epoch is 16 bits (C-10). A timeline epoch change also bumps the session of every stream (C-19).
- C-186, C-208: after a changed A, each fan-in that learns of it picks its own re-alignment position, one lead (C-43) ahead, and announces it in its own output with this re-alignment announcement, exactly as for a ratchet-triggered re-alignment; fan-ins at different depths switch at different, announced positions. The re-alignment payload must therefore serve both causes.
- C-137: a default slice fits 1 252 bytes of UDP payload, 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. Announcements fit inside that cap or are omitted on a fallback path; on a 1 500-byte path about 320 bytes are available beside the audio.

## Work
1. Define the announcement container: presence flag (from 004-06 flags), count, per-announcement type and length, so unknown types can be skipped by a receiver of the same major version (004-04).
2. Field table for each of the four types: type code, named future position (64 bits), and type-specific payload (e.g. new epoch value for the epoch switch; state whether it also names the new session, C-19). No constant gives the payload of "re-alignment" or "transport reference change": propose the minimum (for re-alignment, one that a receiver can act on whether the cause is a ratchet step or a changed A, C-208) and record it as an open item owned by epic 004, to be confirmed by epic 005 (falsifier: a receiver cannot act on the announcement from its payload alone).
3. State the maximum number of announcements per packet and hence the maximum announcement bytes per packet (needed by 004-14).
4. Restate C-43 and C-44 as sender obligations beside the tables, and the C-137 rule: on a fallback path an announcement that does not fit inside the 100-byte cap is omitted; state how many announcements, if any, fit inside the cap.
5. Validity rules with outcomes: unknown type within same major, length overrun (`BadLength`), named position not in the future, named position closer than the lead, duplicate type in one packet, announcement in a gap slice (if forbidden), count above maximum.
6. Add tables to the machine-readable file.

## Acceptance criteria
1. Four announcement types each have a type code, a field table and a byte size.
2. The maximum announcement bytes per packet is stated as a number.
3. At least 6 invalid forms are listed with outcomes.
4. C-43 (64 slices) and C-44 (every 16 slices, every member) appear as sender rules.
5. The proposed re-alignment and transport reference change payloads are recorded as open items with epic 005 as confirming owner.
6. The machine-readable tables pass the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Semantics of re-alignment and epoch switching in the state machines (epics 005, 006).
- Size budget arithmetic (004-14).
