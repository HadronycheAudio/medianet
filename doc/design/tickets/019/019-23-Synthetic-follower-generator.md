# 019-23 — Synthetic follower generator
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-02, epic 015, epic 017

## Context
Epic 019 builds L1, two Linux hosts on one wire through one switch, with harness binary `hadronyche-l1` in crate `hadronyche-tools` (C-133). Its `serving` scenario (019-16) must show AC-25: 64 followers at 50 sync requests/s each with no growth in holder serving lateness, and the 65th refused with a named event.

A medianet has at most 64 active modules (C-34), so a holder has at most 63 real followers. The 64-follower load and the refusal of a 65th are therefore tested with synthetic followers (C-114). Synthetic followers use the test-only hooks beyond the public injection API (C-120): force a module id and nonce, and send a raw packet on a control socket. Each use is stamped like any injection.

Settled facts: followers send 50 sync requests/s by default (C-36); a holder accepts 10 to 200 per second (C-112); sync packets use DSCP CS6 (48) (C-13); IPv4 only (C-100).

## Work
- A generator (reference-module mode from epic 017, or a harness tool in `hadronyche-tools`) that runs K synthetic followers in one process. Each has a distinct forced module id and instance nonce (C-120), sends sync requests at a configurable rate in 10 to 200 per second (C-112, default 50) with DSCP 48, and records each reply's receive stamp and any refusal event.
- Ids are chosen so they cannot collide with the real modules of the run (the harness reads real ids from the stamps first).
- Plan integration: a scenario can ask for K synthetic followers on host `a` or `b`; the generator is started and stopped by the orchestration of 019-02.
- Report fields: per synthetic follower, requests sent, replies received, refusal event if any; check that the holder's stamp lists the injections (C-120).
- Dry-run on loopback with K = 3.

## Acceptance criteria
1. In dry-run, 3 synthetic followers each send 50 requests/s ± 1 % for 5 s with distinct module ids and nonces, and each receives replies from a reference holder.
2. A rate outside 10 to 200 per second is rejected at configuration with a named error (C-112).
3. With the holder's follower limit lowered to 2 in dry-run, the third synthetic follower records the holder's named refusal event.
4. Every synthetic follower appears as an injection in the holder's stamp (C-120).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Running `serving` on the hardware pair (019-16).
- Implementing the C-120 hooks in the runtime (epic 017) or holder admission (epic 015).
