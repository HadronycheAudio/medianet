# 023-32 — Controller member stream of a bundle source (C-205, C-209)
**Epic:** 023-DSL-compiler-front-end · **Effort:** XS · **Depends on:** 023-07, 023-20, 023-31

## Context
- Rule (C-205, decided; restated in C-181): **the controllers of a bundle source always ride in their own member stream.** Unlike a single-stream source (C-181, 023-31), no byte sum against the 100-byte cap (C-137) is involved.
- A bundle source is one whose audio travels in a bundle of several member streams (023-07); a single-stream source is the case of 023-31.
- The controller member stream is on the wire, so it adds one stream's packets/s and bit/s to the admission figures (023-20) and counts against the C-09 limit of 256 streams per source.
- Rule (C-209, decided; C-08 revised): **the controller member counts towards the 32 members of C-08: a source with controllers has at most 31 audio members (186 channels). A wider source with controllers is rejected at compile time.** This closes the former remaining gap on the member cap.

## Work
1. Extend `ControllerPlacement` (023-31) with `OwnMemberStream` and return it for every bundle source with controller outputs (C-205).
2. With `OwnMemberStream`, add one stream to the source's admission figures (023-20) and count it against the C-09 stream limit.
3. Show the placement in the readable dump (023-19).
4. Reject a source with controller outputs whose audio needs more than 31 members (more than 186 channels at defaults, or an explicit layout of 32 members) with a diagnostic (stable code, 023-02) citing C-209 and C-08 and naming the 31-audio-member limit; add the code to the 023-AC-1 suite (023-23).

## Acceptance criteria
1. A 96-channel bundle source (16 audio members) with one controller output → `OwnMemberStream`; packets/s is 17 × 2 000 = 34 000 and bit/s counts the controller member stream (C-21, C-205).
2. The same source with controllers whose bytes would fit the 100-byte cap → still `OwnMemberStream` (C-205: always).
3. A bundle source with no controller output → no placement record and figures unchanged (32 000 packets/s for 96 channels, C-22).
4. A 186-channel source (31 audio members) with one controller output → `OwnMemberStream`, 32 members in all, accepted (C-209).
5. A 187-channel and a 192-channel source with one controller output → 1 diagnostic each citing C-209; the same 192-channel source without controllers → accepted with 32 members (C-08).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Controllers of a single-stream source (023-31); the controller encoding on the wire (epic 004).
