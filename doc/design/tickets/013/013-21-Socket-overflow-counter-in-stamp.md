# 013-21 — Socket overflow counter in stamp
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-09

## Context
- Stamp group **Drops**: per-socket overflow, interface drop counters, measurement-buffer overflow. `[decision]` Drops are counters: they are refreshed with the stamp every 10 s (C-18) and do not change the `StampId` (013-05).
- Linux provides a per-socket drop counter via `SO_RXQ_OVFL` `[kernel-doc]`; macOS and Windows have none `[memory]`, so `drop_accounting` is Absent or Unverified there (C-129 states).
- AC-37 (stamp part, 013-AC-9): **an induced socket overflow appears in the stamp's drop counters.**
- Rule 11: measurement-buffer overflow is counted and stamped, because silent loss flatters the tail.
- `[measured]` POC: a garbage "lost" counter of about 21.7 million on a run with zero loss.
- The epic settles the split with epic 014: this ticket fills the **local** Linux socket and measurement-ring counters; epic 014 reads received DSCP bytes and delivers peer-observed values and peer drop counters into the builder.

## Work
In `hadronyche-std` (Linux):
1. Enable `SO_RXQ_OVFL` on medianet receive sockets and read the counter from the ancillary data; feed the per-socket overflow total into `StampBuilder::drops`.
2. Count measurement-buffer overflow in the measurement ring and feed it into the same section.
3. Set `drop_accounting = Present` on Linux when the option was accepted, Absent otherwise.
4. Integration test, unprivileged: a small receive buffer, a sender bursting faster than the receiver reads, then compare.

## Acceptance criteria
1. Induced overflow test: the stamp's per-socket overflow counter is > 0 and equals the kernel-reported drop count.
2. A run with no overflow reports 0 (not a garbage value).
3. A measurement ring forced to overflow by K samples reports exactly K.
4. The stamp before and after the overflow has the same `StampId`.
5. Test runs in CI unprivileged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" (no parser; reads kernel ancillary data through the platform API); "Design documents updated where behaviour differs, constants table first" (no behaviour differs); "No new open item without an owner and a falsifier" (none opened); "The stamp format is versioned independently of the media format" (no format change).

## Out of scope
- Peer-observed drop counters and DSCP reporting path (epic 014), interface drop counters on other OSes, AC-37 parts beyond the stamp.
- Per-connection summary counters (013-23).
