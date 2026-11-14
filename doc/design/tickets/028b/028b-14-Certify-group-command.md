# 028b-14 — `certify-group` command and membership certificate
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** M · **Depends on:** 028a-03, 028b-08, 028b-10, 028b-12, 028b-13, epic 027

## Context
`hadronyche-commission certify-group <group>` checks every member's E against the group's required E, drives the pairwise skew measurement where a capture device is available, and writes the membership certificate.
- A group may set a required E; the runtime refuses a sink whose E exceeds it (runtime AC-30). Only the commissioned state gives a numeric E; uncertified and self-monitored sinks declare E as unknown (C-140). A sink with unknown E is refused from a group that sets a required E and admitted to one that sets none (C-103).
- E = p99.9 of |actual emission − (T(p) + A)| over one hour, max beside it (C-73).
- Pairwise skew: **L3 procedure** (epic 027), two-channel capture; **pass = the 99.9th percentile of absolute skew ≤ the sum of the two sinks' E values, for 1 hour and across restarts of either; the maximum is reported beside it** (C-79). The E values in that criterion are the platforms' target E (C-70, C-71, C-72), never the E measured in the same run (C-149); a pair containing a sink whose platform has no target E cannot be skew-certified. L3 parameters follow C-166 (comparison windows of 10 s; measurement window after a restart 60 s). Where a no-step or recovery tolerance is applied to a pair of different sinks, it is the larger of the two sinks' own tolerances, and both sinks' figures are reported beside the verdict (C-203).
- A commissioned member whose clock is currently unlocked stays commissioned; its alignment claim is suspended and does not count towards the group's required E until its clock locks again, then resumes without re-commissioning (C-180).
- The pairwise measurement and the restarts it involves are test activity, allowed only inside a commissioning window stamped by every module (C-107), announced with an expiry and closed at expiry or on explicit close (C-153); the E check alone is read-only. Group certification taken inside the window is a valid commissioning result, recorded with the window it was taken in (C-178).
- Certificates are stored beside the collected stamps in the deployment's data directory (C-171).
- Certificate fields: group, A, required E, per sink E and D with method and date, hardware identity, tool version, pair skew records (028b-10).

## Work
- Load configuration and existing `SinkRecord`s (from 028b-12); report members lacking a record.
- If the group sets a required E, refuse members whose E > required E or whose E is unknown, a self-monitored-only record counting as unknown (C-103, C-140). A member whose stamp shows its clock unlocked keeps its record and is not asked to re-certify; it is reported as not currently counting towards the required E (C-180), and no pairwise measurement is started with it until it is locked.
- If `--capture <device>` is given, check the window (028b-08), including that it will not expire before the run ends (C-153), run the L3 pairwise measurement for each adjacent member pair (or all pairs, configurable) for 1 hour including restarts of each member, and check skew p99.9 ≤ the sum of the two platforms' target E (C-79, C-149); record p99.9 and max; for a pair of different sinks apply the larger of their no-step and recovery tolerances and report both (C-203).
- Write the `Certificate` (028b-10) only when all checks pass; otherwise print the failing members/pairs.

## Acceptance criteria
1. On the L3 rig, `certify-group` reproduces the skew result of epic 027 (epic AC-6).
2. A member whose recorded E exceeds the required E, or whose E is unknown, is reported by name and no certificate is written; in a group with no required E a member with unknown E is accepted.
3. A pair with skew p99.9 above the sum of the two platforms' target E fails; at or below passes; the maximum is recorded either way (C-149). Two Linux sinks are judged against 100 µs.
4. The written certificate contains every field listed in Context.
5. With `--capture` outside a commissioning window the command refuses to start (epic AC-10).
6. A fixture with a recorded member whose clock is unlocked keeps its record, reports it as not currently counting towards the required E, and asks for no re-certification (C-180).
7. For a Linux–macOS pair the tolerance applied is the larger of the two sinks' own, and both figures appear beside the verdict (C-203).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Single-sink measurement (028b-12). Capture hardware selection (028b-13).

## Notes
- The L3 reference skew result is a measurement owned by epic 027 (remaining gap until it exists).
