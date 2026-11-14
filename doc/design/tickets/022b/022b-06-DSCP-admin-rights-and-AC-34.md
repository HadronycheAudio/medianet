# 022b-06 — DSCP without administrator rights, and AC-34 peer check
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** S · **Depends on:** 022a-05, 022a-14, 022a-17, 022a-21, 022b-01

## Context
- Part of epic 022b (Windows edge L2 certification).
- DSCP (C-13): sync 48, media and one-way probes 46, control 40.
- Windows `[memory]`: plain `IP_TOS` is typically ignored; marking goes through the QoS flow API; arbitrary values may need administrator rights or policy. If the receiving peer cannot confirm, the stamp says *unverified*.
- Peer observation is settled (C-47): the Linux consumer publishes, per connection, the DSCP it observed in its own regime stamp; receiver reports are not multicast and there is no query message. So the check needs the Windows edge as a producer (capture, 022a-14) feeding a Linux consumer.
- All three classes are peer-verifiable: media from the consumer's per-connection DSCP (C-47); sync from the DSCP the holder (the Linux backbone) reports for this follower's sync requests, and control from the DSCP every module reports for this edge's hellos (C-152, implemented in 022a-21). The wire capture records all three classes as the independent check.
- 022-AC-9 (AC-34, Windows part): the Linux consumer's regime stamp shows, for its connection from the Windows capture edge, the DSCP value it actually received; the Windows stamp's media class agrees.
- Open sub-question owned by this task, feeding task 028: can DSCP be set without administrator rights on a standalone laptop? Falsifier: no → marking stays *unverified* and the checklist documents the policy route.

## Work
1. On a standalone (non-domain) reference laptop, as a standard user, run a capture edge to a Linux consumer and read the observed DSCP from the consumer's stamp and from a wire capture (all three classes).
2. Repeat with administrator rights and with a local QoS policy, recording which route yields 48/46/40 on the wire.
3. Compare the edge stamp's DSCP fields with the consumer's stamp (media) and with the holder's and peers' stamps (sync and control, C-152) in each case.
4. Write the finding and the policy route as input for the epic-028 checklist, in `doc/design/reports/` (C-130).

## Acceptance criteria
1. **(022-AC-9.)** In every case, the Linux consumer's stamp shows the DSCP actually received on the media connection, and the Windows stamp says media *present* exactly when it is 46, else *unverified* (or *absent* when marking was refused); likewise sync is *present* exactly when the holder's stamp shows 48 for this edge, and control exactly when the peers' stamps show 40 for its hellos (C-152).
2. The admin-rights sub-question is answered `[measured]` for the reference OS build(s), from the wire capture for all three classes.
3. A policy-route note for epic 028 exists.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The commissioning checklist itself (epic 028).
