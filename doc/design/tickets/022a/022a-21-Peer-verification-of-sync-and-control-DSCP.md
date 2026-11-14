# 022a-21 — Peer verification of sync and control DSCP
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-05, 022a-06, epic 004

## Context
- Part of epic 022a (Windows edge backend), `net.rs`. New ticket for rule C-152.
- DSCP values (C-13): sync 48, media and one-way probes 46, control 40. Hellos and regime stamps are control messages (C-17, C-18).
- C-152: the holder reports in its stamp the DSCP observed on each follower's sync requests; every module reports the DSCP observed on each peer's hellos. This edge is never the holder (edge only), so it reports no sync-request observations; as a module it must report the DSCP it observes on each peer's hellos.
- Consequences for this edge's own DSCP states (C-129): sync becomes *present* when the holder's stamp shows 48 for this edge's sync requests; control becomes *present* when peers' stamps show 40 for this edge's hellos; a different value or no report leaves *unverified*; a refused marking stays *absent* (022a-05). A sink-only edge therefore has two peer-verifiable classes.
- Peer stamps are fetched and parsed by core (layout epic 004, transport C-18). Windows marking is report-only and plain `IP_TOS` is typically ignored `[memory]`.
- The TOS-byte receive and its control-message parser are 022a-05 (fuzzed to the C-95 budget).

## Work
1. On the discovery socket (022a-06), read the TOS byte of every received hello with the 022a-05 parser and keep, per peer module id, the last observed DSCP and a count of hellos whose DSCP differed from 40.
2. Expose the per-peer hello DSCP for this edge's stamp (022a-17).
3. Verification: from the holder's stamp, read the DSCP observed on this edge's sync requests and set the sync `CapState`; from peers' stamps, read the DSCP observed on this edge's hellos and set the control `CapState`. Control is *present* when the reports show 40; a report of any other value leaves *unverified* and is logged with the reporting peer's id.
4. Unit tests with synthetic holder and peer stamps.

## Acceptance criteria
1. A synthetic holder stamp showing 48 for this edge yields sync *present*; another value or no report yields *unverified*; a refused sync marking stays *absent*.
2. Synthetic peer stamps showing 40 for this edge's hellos yield control *present*; a report of another value yields *unverified* with the reporting peer logged.
3. Hellos received with DSCP 0, 40 and 46 from three peers appear in this edge's per-peer hello DSCP report with those values.
4. The edge publishes no sync-request DSCP observations (it is never the holder).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- DSCP marking and the TOS-byte parser (022a-05).
- Parsing peer stamps (core, epic 004 layout).
- The hardware check against Linux peers (022b-06).
