# 011-22 — Regime stamp and design doc update
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-04, 011-06, 011-09, 011-12, 011-17, 011-18, 011-23, 011-25, 011-26, epic 013

## Context
Epic 011's definition of done requires:
- the holder-eligible capability bit is in the regime stamp;
- regime stamp fields touched are complete and documented;
- design documents updated where behaviour differs, constants table first.

C-18 (decided): the regime stamp is a dedicated control message split into sections that each fit C-17 (1 252 bytes), multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch; the hello carries only a stamp version number. The regime stamp message is defined by epic 013 and sent by the hosted backend, epic 015 (C-144); the section that holds the holder-eligible bit and its encoding come from epic 013. This ticket adds and documents the field, and documents the per-peer observed hello DSCP that 011-25 exports for the stamp (C-152).

Constants owned or touched by this epic (all to be checked against the implementation):
C-30 hello period 1 s; C-31 hello expiry 3 s, non-live peers only; C-32 listen-before-claim 1.5 s; C-33 election window 400 ms; C-34 at most 64 active modules; C-217 peer map of 63 peers (no entry for self); C-35 at most 64 followers per holder; C-36 50 sync requests per follower per second; C-112 accepted sync request rate 10 to 200 per second; C-37 election on reply silence 1.5 s; C-42 stalled data plane visible within 2 hello periods; C-46 quarantine until one nonce seen for 3 s; C-10 timeline epoch 16 bits; C-11 id / nonce / domain 64 / 64 / 16 bits, domain 0 unconfigured; C-12 / C-115 auth reservation 8 + 64 + 128 bits, scheme 0 and tag zero in v1, 64-bit counter live as the liveness counter; C-17 control message at most 1 252 bytes; C-18 stamp version in the hello; C-100 IPv4, multicast TTL 1; C-213 candidates need usable time (Locked, or holdover with the alignment claim not lapsed), takeover frequency-continuous with the timeline epoch kept; C-113 lowest-id eligible candidate and timeline epoch bump when none has usable time; C-114 synthetic followers for the cap test; C-95 fuzz budget; C-13 DSCP CS5 (40) for control; C-08 / C-209 at most 32 members per bundle, the controller member included; C-152 observed hello DSCP per peer in the stamp; C-173 domain-0 hello ignored and counted; C-185 VersionMismatch, NameTaken and ModuleCap raised at discovery.

Holder-eligible bit `[decision]`: set only on v1 Linux hosts; embedded, macOS and Windows nodes do not set it.

## Work
1. Add the holder-eligible capability bit to the regime stamp section defined by epic 013 (C-144), with its meaning and the v1 rule above.
2. Review every regime stamp field touched by tickets 011-01..011-26 and document each, including the per-peer observed hello DSCP (C-152).
3. Update the design constants table (`doc/design/02-constants.md`) first, then the prose (`doc/design/01-architecture.md`, C-130, C-171), for every behaviour that differs from the design (including the measured hello size from 011-04 tagged `[measured]`, `PEER_MAP_BYTES` from 011-06, and the stall rule from 011-12 as defined by epic 005).
4. Note that C-13 (DSCP CS5) and the IPv4 multicast TTL 1 (C-100) are applied by the platform transport, not by core.

## Acceptance criteria
1. The regime stamp contains the holder-eligible bit with documented semantics.
2. Every C-nn listed above appears in the constants table with a value matching the code's named `const` (checked by a review checklist, one line per constant).
3. Every new factual claim in the changed documents carries an evidence tag.

## Definition of done
- [ ] Regime stamp fields touched are complete and documented
- [ ] The holder-eligible capability bit is in the regime stamp
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Tests green in CI, unprivileged" (documentation and one stamp field definition; the field's encoding is tested where the stamp message is implemented); "Every parser touched is fuzzed" (no parser); "The frozen POC is not modified" (documents only); "No election path reachable from hello expiry" (no transition code).

## Out of scope
- Code changes beyond the regime stamp field.
- The regime stamp message (sections, 10 s refresh, unicast fetch): defined by epic 013, sent by epic 015 (C-144).
