# 017b-05 — Injection recorded in the regime stamp
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-01, epic 013, epic 015

## Context
- Rule `[decision]`: injection is public runtime API and is **always written into the regime stamp**. The test-only hooks (C-120: force id and nonce, force epoch, raw send; C-145: inject a panic into a data, clock or housekeeping thread) are stamped like any injection; 017b-06, 017b-07 and 017b-08 use the log built here.
- AC-31: every record file carries a complete stamp; injection is labelled; one writer per file. This ticket covers "injection is labelled"; 017b-09 covers the record file and the single writer.
- The stamp's fields are defined by epic 013. Stamp transport is settled (C-18): a dedicated control message split into sections that each fit the control message bound, at most 1 252 bytes of UDP payload (C-17); multicast to the control group on any regime change and refreshed every 10 s; a unicast request fetches it immediately; the hello carries only a stamp version number.
- Consequence: every injection call is a regime change. It bumps the stamp version carried in the hello and triggers the multicast. Sending the regime stamp message (which epic 013 defines) is the hosted backend's duty (epic 015, C-144): this ticket marks the regime change and supplies the injection sections; epic 015 sends them. An injection record longer than one section is split across sections.

## Work
1. Make the injector keep a log of every injection and test-hook call (method, target module(s) or stream, parameters, time of call in true time).
2. Add stamp field(s) (for example `injection`) that serialise this log, plus a boolean `injected` that is true whenever any injection or hook call has been made in the process.
3. The stamp of a run with no injection calls states `injected = false` explicitly (field present, not absent).
4. Encode the log so it splits across stamp sections, each within 1 252 bytes (C-17, C-18).
5. On each call, bump the stamp version (C-18) and signal the regime change to epic 015's stamp sender (C-144), so peers see the change through the hello and receive the multicast.
6. Document the new stamp fields (name, type, unit, meaning) in the stamp documentation owned by epic 013.

## Acceptance criteria
1. A process that calls `set_clock(m, {offset_ns: 5_000_000, drift_ppb: 20_000})` and `set_link(a, b, {seed: 7, loss: 0.01, ..})` produces a stamp whose injection field contains both calls with all parameter values exactly.
2. A process with no injection calls produces a stamp with `injected = false`.
3. Each of the 7 `Injector` methods and each of the 4 test hooks (C-120, C-145) is represented in a round-trip test (serialise, parse, compare equal).
4. A log of 1 000 calls serialises into sections each at most 1 252 bytes of UDP payload and parses back equal.
5. Each injection call increments the stamp version seen in the next hello.
6. The stamp documentation lists every new field.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Writing the record file and enforcing one writer per file (017b-09).
- Stamp fields not related to injection, and the stamp message itself (defined by epic 013, sent by epic 015, C-144).
- The hooks' behaviour (017b-06, 017b-07, 017b-08).
