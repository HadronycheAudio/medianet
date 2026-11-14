# 004-28 — Regime stamp sections: observed DSCP and alignment group
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-24, 004-25
## Context
The regime stamp container, version and request are defined in 004-24 (C-18); the section types named at the split are laid out in 004-25. Rules settled later put more state on the wire in the stamp `[decision]`:
- **Observed DSCP of peers** (C-152): the holder reports in its stamp the DSCP observed on each follower's sync requests; every module reports the DSCP observed on each peer's hellos. DSCP classes are sync CS6 (48) and control CS5 (40) (C-13). At most 64 active modules (C-34), so at most 63 peers and, for a holder, at most 63 real followers (C-114).
- **Sync requests dropped above the accepted rate** (C-204): requests beyond 200 per second from one follower (C-112) are dropped before any work is done and counted per follower in the holder's stamp. The holder's per-follower entry of the observed-DSCP section therefore also carries this counter.
- **Alignment group** (C-157, C-160, C-182): per aligned input, the lateness percentiles 50th, 99th, 99.9th and the maximum (C-160), and the reclaimable latency, offset − (current maximum lateness + headroom) when positive (C-157). Headroom is defined by C-52; the offset is bounded by A (C-53, C-96). The percentiles, the maximum and the reclaimable latency use a rolling 60 s window; the maximum since the connection went live is reported beside them (C-182).
- Every section fits C-17 (at most 1 252 bytes of UDP payload); a type may span several sections (004-24). Adding a section type is a minor change (004-04).
- C-144: the hosted backend (epic 015) sends the stamp; this ticket lays out bytes only.

## Work
1. One section type for observed DSCP: per peer, the peer's module id (64 bits, C-11), the DSCP observed on its hellos, and, in a holder's stamp, the DSCP observed on its sync requests, with a distinct "not observed" encoding, and the count of its sync requests dropped above 200 per second (C-204), with width and wrap or saturate rule. Fix the maximum entry count (63, from C-34) and show the section fits 1 252 bytes.
2. One section type for the alignment group: per input, an input identifier consistent with the health section of 004-25, the four lateness figures of C-160 over the rolling 60 s window, the maximum since the connection went live (C-182) and the reclaimable latency of C-157, with widths, units and valid ranges; state the window in `wire.md`. Fix the maximum entry count and show each section fits 1 252 bytes, splitting the type over several sections where needed.
3. Validity rules with outcomes: DSCP value above 63, count above maximum, length inconsistent (`BadLength`), percentiles not in non-decreasing order (50th ≤ 99th ≤ 99.9th ≤ maximum ≤ maximum since live, C-182), reserved bits non-zero (`ReservedNonZero`).
4. Add the tables to the machine-readable file using the 004-02 "section of" attribute.

## Acceptance criteria
1. Two section types exist, each with a type code and a maximum size ≤ 1 252 bytes (epic AC-3).
2. The DSCP section covers both C-152 observations (hellos, every module; sync requests, holder only) and, in a holder's stamp, the per-follower dropped-request counter of C-204 with a width.
3. The alignment section carries exactly the four lateness figures of C-160 (rolling 60 s window, C-182), the maximum since live (C-182) and the reclaimable latency of C-157, each with a width and a unit (epic AC-8).
4. At least 5 invalid forms with outcomes.
5. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The stamp container and request (004-24) and the section types of 004-25.
- Measuring DSCP or lateness, and computing reclaimable latency (implementation epics).
