# 007a-08 — Valid vectors: control messages
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** S · **Depends on:** 007a-05, epic 004

## Context
- Epic criterion 007-AC1 requires valid vectors in both encode and decode direction for **every** message type of the wire specification, which includes the control plane, not only media packets.
- Every control message fits one packet at the fallback MTU: at most 1 252 bytes of UDP payload (C-17, C-100). The path-MTU probe pair is exempt: it is sent at the intended media packet size (C-116).
- Control messages carry the authentication reservation (scheme 8 bits + counter 64 bits + tag 128 bits, C-12). In v1 the scheme is 0 and the tag is zero; the 64-bit counter is live because it is the liveness counter (C-115).
- Control message facts settled so far `[decision]`:
  - the hello is sent every 1 s (C-30) and carries only a regime stamp version number, not the stamp (C-18);
  - sync replies (election watches for 1.5 s of silence, C-37);
  - admission requests and refusals, with each refusal reason (one is RequiredEExceeded, C-103). There are 18 refusal reasons in all (C-185): the 16 of C-173 plus ControlDegraded (the target's housekeeping has stopped restarting, C-139) and NoSkewBudget (a fan-in with no downstream group and no configured budget, C-142). VersionMismatch, NameTaken and ModuleCap are raised at discovery, where they stop a module joining, not in the admission refusal;
  - receiver reports, every 100 ms per bundle (C-40), unicast, never multicast (C-47); each carries the consumer's current A, its own or the one it inherited from a downstream group (C-198);
  - the path-MTU probe pair (C-116); PathMtuTooSmall applies only when the probe at the 1 280-byte fallback also fails (C-173);
  - the commissioning window announcement, sent by the commissioning tool on the control group with an expiry time, and its explicit close (C-153);
  - there is no stream-health query message (C-47).
- The full list of control message types and their layouts is an open sub-question of the epic owned by epic 004, which supplies the wire layouts (C-193); this ticket takes it from the wire tables. The regime stamp message and its unicast request are in 007a-09.
- Limits that bound maximal encodings: at most 32 members per bundle, the controller member included, so a source with controllers lists at most 31 audio members (C-08, C-209), 256 streams per source (C-09), 64 modules in the discovery map (C-34).

## Work
1. Extend `gen` to emit minimal, typical and maximal vectors for every control message type present in the wire tables, except the regime stamp message and its request (007a-09).
2. Maximal vectors use the largest list sizes allowed (32 bundle members, 256 streams, 64 map entries where a message carries such lists) without exceeding 1 252 bytes; if a list cannot fit, record it as a specification inconsistency in the design documents (the falsifier of C-17 names the hello of a 32-member bundle source).
3. Emit one valid vector per refusal reason of C-185 (18), each in the message that carries it: the admission refusal for the reasons raised at admission, the discovery-time message given by the wire tables for VersionMismatch, NameTaken and ModuleCap.
4. Emit the path-MTU probe pair at the default media packet size (1 472 bytes of UDP payload) and at the fallback size (1 252 bytes).
5. Commit the files to `vectors/wire/valid/`.

## Acceptance criteria
1. Every control message type in the wire tables, other than those of 007a-09, has at least 3 valid vectors (minimal, typical, maximal).
2. No control vector exceeds 1 252 bytes of UDP payload, except the path-MTU probe pair.
3. A maximal probe vector of 1 472 bytes of UDP payload exists.
4. The hello vectors carry a stamp version number and no stamp content.
5. Every vector has scheme 0 and a zero tag; at least one typical vector has a non-zero liveness counter.
6. All files carry `spec_version` and pass the 007a-02 schema test.
7. Each of the 18 refusal reasons of C-185 has at least one valid vector; no admission refusal vector carries VersionMismatch, NameTaken or ModuleCap.
8. Every receiver report vector carries the consumer's current A, and its decoded structure records that value (C-198).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The regime stamp message and its unicast request (007a-09).
- Control state-machine behaviour (007b-01 to 007b-12).
- Over-size and other invalid control messages (007a-11).
