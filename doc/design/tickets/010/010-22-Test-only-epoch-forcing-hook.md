# 010-22 — Test-only epoch forcing hook
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** XS · **Depends on:** 010-06, 010-14
## Context
- C-120: beyond the public injection API, tests need these hooks: force a module id and nonce; force a stream onto a given timeline epoch; send a raw packet on a media or control socket. Every one is stamped like any injection.
- Of these, only the timeline-epoch hook touches state this epic owns: the `Packetiser` holds the stream's epoch (C-10: 16 bits). Forcing the module id and sending raw packets belong to other epics.
- Normal epoch changes go through `switch_epoch` (010-16) with an announcement; the hook deliberately bypasses that, to let tests put a stream on a different epoch than its peers.

## Work
- In `hadronyche-core`, behind a test-only cargo feature that is off by default, add `Packetiser::force_epoch(epoch: u16)` and the same on the bundle packetiser (one member or all), taking effect from the next slice, without an announcement and without a session bump.
- Record that a forced epoch is in effect (a flag readable through a getter) so it can be stamped like any injection in the regime stamp (defined by epic 013, sent by epic 015, C-144).
- Make sure a build without the feature has no such method (compile test).

## Acceptance criteria
1. With the feature on, after `force_epoch(7)` the next slice and every later one carry epoch 7, and the session is unchanged.
2. The injection flag reads true after the call and false on a packetiser that never used it.
3. A crate build without the feature does not expose `force_epoch` (a compile-fail test or a cfg check in CI).
4. The `no_std` CI build of 010-01 passes with and without the feature.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The public injection API and stamping of injections (their owners).
- Forcing a module id and nonce, and sending raw packets (C-120; other epics).
