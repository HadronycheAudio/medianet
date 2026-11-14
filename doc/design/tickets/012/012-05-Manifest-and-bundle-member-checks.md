# 012-05 — Manifest and bundle member checks
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-01, epic 010

## Context
The epic 012 handshake exchange carries the manifest: ports, content types, and for presentation groups the required E and the sink's declared E `[decision]`. It also carries the bundle member list with the source's session of each requested stream (C-19); one handshake covers a bundle and lists the members wanted (at most 32 members per bundle, C-08; empty list = whole output).

Settled by C-209 (with C-181, C-205): a bundle source's controllers always ride in their own member stream, and that controller member counts towards the 32 of C-08, so a source with controllers has at most 31 audio members. This check counts the controller member like any other; rejecting a wider source with controllers is the compiler's, at compile time.

Settled by C-181 (revised: permissive): a single-stream source may always put its controllers in a second stream, must do so when headers plus controllers exceed the 100-byte cap (C-137), and may share the audio packet while they fit. The compiler checks this from the manifest. Admission accepts either layout and does not judge it: when the controllers have their own stream, it is a member of the output like any other (requested by id, with its session, counted in the 32).

Every stream on the wire is at the 96 kHz domain rate (C-01), so a sample-rate mismatch cannot occur between an output and an input; content-type comparison does not include a rate.

Retired POC behaviour `[code]`: the only refusals were content-type mismatch and "not an input" (`module.rs:775-820`). Both remain refusal reasons in the new design: `ContentMismatch` and `NotAnInput`.

Settled by C-173: an unknown or duplicate bundle member, or more than 32 members, is refused with the 16th reason `BadMemberList`. The codec (012-08) therefore decodes a member list longer than 32 when it fits the message, and this check refuses it; it is not dropped as malformed.

## Work
In `hadronyche-core` (admission module):
- `pub fn check_manifest(output: &PortManifest, input: &PortManifest, members: &[StreamId], sessions: &[u32], output_members: &[StreamId]) -> Result<(), RefuseReason>` (names may be refined), with:
  - `NotAnInput` if the consumer's named port is not an input;
  - `ContentMismatch` if the output's content type does not match the input's;
  - `sessions.len()` must equal the number of requested streams (the whole output when `members` is empty); otherwise the request is malformed;
  - `BadMemberList` (C-173) if a member is unknown to the output, appears twice, or the list has more than 32 members, the controller member included (C-08, C-209).
- Port and content-type representation come from epic 010 (member lists) and 004 (manifest layout).

## Acceptance criteria
1. A request naming an output port as the consumer port is refused with `NotAnInput`.
2. A request whose output content type differs from the input's is refused with `ContentMismatch`.
3. A member list of 32 valid members with 32 sessions is accepted.
4. An empty member list with one session per output stream is accepted (whole output).
5. A duplicate member, a member absent from the output, or a list of 33 members is refused with `BadMemberList` (C-173); a session count that differs from the stream count is malformed.
6. A list of 31 audio members plus the controller member (32) is accepted; 32 audio members plus the controller member (33) is refused with `BadMemberList` (C-209).
7. C-181: a single-stream source with its controllers in a second stream is accepted when both streams are requested (two sessions) and when only the audio stream is requested; the same source with controllers sharing the audio packet (one stream) is accepted; no layout of controllers produces a refusal.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: the check works on decoded values; the parser is 012-08
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: no new behaviour beyond the epic
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Required E for presentation groups (012-07). Peer identity and caps (012-06). Capacity checks (012-03).
- Encoding the refusal (012-13).
