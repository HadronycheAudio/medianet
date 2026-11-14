# 004-10 — Media measurement extension layout
**Epic:** 004-Wire-specification · **Effort:** XS · **Depends on:** 004-06
## Context
Media packets may carry an optional measurement extension (rule 4 `[decision]`): **hop count**, **accumulated residence time**, **accumulated declared delay**. Production streams may omit it. Presence is signalled by a flag in the media packet header (004-06). Forward packets carry no per-receiver state (rule 8).

The declared delay of a module is its structural latency L, always a whole number of domain-rate samples, declared in the manifest (C-105); the domain rate is 96 000 Hz and is the only rate on the wire (C-01). Accumulated declared delay is therefore an exact integer sum of samples. C-137: the extension fits inside the 100-byte header cap or is omitted on a fallback path. No constant fixes the unit of accumulated residence time or the width of the two accumulated fields; this ticket decides them (open item owned by epic 004; falsifier: the declared-delay field saturates over the maximum hop count at the largest L a manifest admits).

## Work
1. Field table in `wire.md`: hop count, accumulated residence time, accumulated declared delay, with widths, units (state the time unit, e.g. ns or samples at the domain rate, `[decision]`), saturation behaviour and valid ranges. Accumulated declared delay is in whole samples at the domain rate (C-105).
2. State how each processor updates the fields when forwarding (increment hop count, add residence time, add declared delay) and saturation instead of wrap.
3. Validity rules with outcomes: flag set but bytes missing (`TooShort`/`BadLength`), hop count beyond the maximum, reserved bits set.
4. State its byte size for the budget (004-14) and whether it fits inside the C-137 cap or is omitted on a fallback path.
5. Add to the machine-readable file.

## Acceptance criteria
1. Three fields with width, unit and saturation rule each; accumulated declared delay is a whole-sample sum (C-105).
2. Byte size of the extension stated.
3. At least 3 invalid forms with outcomes.
4. No field identifies a receiver or destination.
5. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Receiver reports and one-way probes (004-20).
