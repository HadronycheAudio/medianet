# 004-18 — Admission handshake messages and refusal reasons
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-16
## Context
The admission handshake is on the discovery and control layer (unicast over IPv4, C-100; DSCP CS5 (40), C-13). `wire.md` needs a field table for each admission message. Two parts of the handshake have their own tickets: the manifest (004-26) and the path-MTU probe pair (004-27); this ticket defines the messages that carry them.

Control rules `[decision]`: the admission handshake carries
- declared rates: bit/s on the wire, packets/s, slice rate;
- bundle member list (C-08: at most 32 members, the controller member included, C-209);
- the path-MTU probe pair (004-27);
- the reverse measurement port (receiver reports and probes are sent there);
- pinned addresses (the media receiver demultiplexes on the source address pinned at admission plus stream number, rule 7); IPv4 only in v1 (C-100), so each address is 4 bytes;
- the manifest (004-26);
- for a fan-in outside any presentation group, the A of the downstream presentation group it feeds, the largest if several, learned at admission (C-142);
- a refusal reason when refused.

Settled rulings `[decision]`:
- C-103: a sink with no certified E declares E as unknown and is refused with reason `RequiredEExceeded` from any presentation group that sets a required E; it is admitted to groups that set none. `RequiredEExceeded` is therefore a member of the refusal enumeration.
- C-45: Pending (target not present) has no timeout; no admission message carries one.
- C-219: the declared rates of a controller stream (a bundle source's controller member, or a single-stream source's controller second stream, C-181) are computed like an audio stream's: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of overhead), at 2 000 packets/s; the formula is stated in 004-15, and admission and the commissioning tool use the same figure.
- C-17: every admission message is at most 1 252 bytes of UDP payload; only the path-MTU probe pair is exempt (C-116).
- C-12 with C-115: authentication reservation in every control message, scheme and tag zero, counter live.
- C-09: at most 256 streams per source. C-05: default slice fits one packet at the path MTU.
- C-185 (completing C-173): the refusal reasons are a closed set of 18, all named: `PortCeiling`, `PacketRate`, `SliceRate`, `InputCount`, `BufferMemory`, `CostBound`, `ContentMismatch`, `NotAnInput`, `VersionMismatch`, `ForeignDomain`, `PathMtuTooSmall`, `RequiredEExceeded`, `FollowerCap`, `ModuleCap`, `NameTaken`, `BadMemberList`, `ControlDegraded` and `NoSkewBudget`. Of these: `RequiredEExceeded` (C-103); `PathMtuTooSmall`, which applies when the probe at the 1 280-byte fallback also fails (004-27); `ForeignDomain`, for a different non-zero domain (a hello with domain 0 is ignored and counted, not refused); `BadMemberList`, for an unknown or duplicate bundle member or more than 32 members, the controller member included (C-209); `ControlDegraded`, the target's housekeeping has stopped restarting (C-139), answered by the housekeeping supervisor, which keeps a minimal refusal path; `NoSkewBudget`, a fan-in with no downstream group and no configured budget (C-142). `VersionMismatch`, `NameTaken` and `ModuleCap` are raised at discovery, where they stop a module joining, not at admission (C-185), so no admission refusal message carries them. A connection alarm is raised on refusal.
- No constant names the full admission message set. This ticket proposes it (open item owned by epic 004, confirmed by epic 005; falsifier: epic 005's admission state machine needs a message or field the set lacks). No reason name is left open: the meaning of the reasons C-185 names without a gloss is restated from epic 005's admission rules where they define it, never renamed.

## Work
1. Enumerate the admission messages (e.g. request, accept, refuse, plus the messages carrying the manifest and the probe pair) and give each a type code.
2. Field table for each, with the common control header from 004-16 and the C-12 reservation, carrying every item listed above. Define units and widths for declared rates (wide enough for a controller stream's C-219 figure as well as an audio stream's), IPv4 address encoding (C-100), port numbers, and the reference to the manifest laid out in 004-26.
3. Refusal reasons: a closed enumeration of exactly the 18 names of C-185, each with a numeric value, with `VersionMismatch`, `NameTaken` and `ModuleCap` marked as raised at discovery. Keep the refusal message small enough for the housekeeping supervisor's minimal path to send `ControlDegraded` (C-185).
4. Maximum size of each message with a 32-member bundle; ≤ 1 252 bytes or report a falsified constant.
5. Validity rules with outcomes: generic control violations plus rate fields zero or out of range, member count > 32 or unknown or duplicate member (refused with `BadMemberList`, C-173), address not a valid unicast IPv4 address, unknown refusal reason, a refusal carrying a discovery-raised reason (`VersionMismatch`, `NameTaken`, `ModuleCap`, C-185), downstream A field present where it does not apply.
6. Add tables to the machine-readable file.

## Acceptance criteria
1. Every item in the Context list is a field in at least one admission message (the manifest and probe pair by reference to 004-26 and 004-27).
2. Refusal reasons form a closed numbered enumeration of exactly the 18 names of C-185 (`PortCeiling`, `PacketRate`, `SliceRate`, `InputCount`, `BufferMemory`, `CostBound`, `ContentMismatch`, `NotAnInput`, `VersionMismatch`, `ForeignDomain`, `PathMtuTooSmall`, `RequiredEExceeded`, `FollowerCap`, `ModuleCap`, `NameTaken`, `BadMemberList`, `ControlDegraded` and `NoSkewBudget`), with `VersionMismatch`, `NameTaken` and `ModuleCap` marked as discovery-raised and never carried by an admission refusal.
3. The downstream A of C-142 has a field with width and unit.
4. Every admission message states a maximum size ≤ 1 252 bytes (epic AC-3).
5. No admission message has a timeout field for Pending (C-45).
6. At least 8 invalid forms with outcomes across the set.
7. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- The manifest layout (004-26) and the path-MTU probe pair (004-27).
- Admission policy and state machine (epics 005/006).
- Receiver reports and probes on the reverse port (004-20).
