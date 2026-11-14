# 004-29 — Commissioning window announcement and close messages
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-16, 004-25
## Context
A commissioning tool may drive test traffic only inside a declared commissioning window that every module stamps; outside it the tool is read-only (C-107). Rules `[decision]`:
- C-153: the window is announced by the tool on the control group with an expiry time; modules stamp it while open; it closes at expiry or on explicit close.
- C-178: while a window is open, ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified. The commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in, so the window identifier must be stable and unique enough to be referenced by those records.
- Control layer: UDP over IPv4, multicast to the control group with TTL 1 (C-100); DSCP CS5 (40) (C-13). Common control header and authentication reservation from 004-16 (C-12, C-115: scheme and tag zero, counter live). Each message is at most 1 252 bytes (C-17).
- The stamped form of the open window is laid out in 004-25; it carries the expiry time from the announcement.
- No message for the commissioning window was in the epic at the split; no other ticket lays one out.

## Work
1. Field table for the window announcement: common control header, a window identifier (the reference commissioning results carry, C-178), the announcing tool's identity, and the expiry time (width, unit and clock stated; tag `[decision]`).
2. Field table for the explicit close: common control header and the identifier of the window it closes.
3. Receiver rules: a module stamps the window from the announcement until expiry or close (C-153); a close for an unknown window and an announcement whose expiry is already past have stated outcomes; state how a module treats a second announcement while a window is open.
4. Validity rules with outcomes: generic control violations (too short, bad magic, version mismatch as a named event, foreign domain, oversize, reserved non-zero), expiry out of range, unknown window identifier on close.
5. Add the tables to the machine-readable file and a section to `wire.md` (placeholder created by 004-01).

## Acceptance criteria
1. Announcement and close each have a type code, a field table and a maximum size ≤ 1 252 bytes (epic AC-3).
2. The expiry time has a width, a unit and a clock (epic AC-8, C-153).
3. The open, expiry and explicit-close transitions of C-153 appear as receiver rules.
4. At least 6 invalid forms with outcomes across the two messages.
5. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The commissioning tool and its test traffic (commissioning tooling).
- The stamp section that records the open window (004-25).
