# 017b-18 — Scenario `collision`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-09, 017b-06, epic 012, epic 015

## Context
- AC-22: a **forced id collision is detected within 1 s**, the **later joiner redraws**, **both modules log it**, and a third party quarantines the id.
- **How to force it (settled, C-120):** the test-only hook `force_identity(module, id, nonce)` (017b-06), stamped like any injection. Module id and instance nonce are 64 bits each (C-11); a collision is the same id with different nonces.
- **Third-party quarantine (settled, C-46):** a third party quarantines a colliding id until only one instance nonce has been seen for one hello-expiry period, 3 s (C-31).
- Hello period 1 s (C-30).

## Work
1. Scenario `collision`: modules A (id X, nonce n1) and C (a third party) run; after 2 s, start module B with id X and nonce n2 through the hook.
2. Assertions: detection time from B's start ≤ 1 s; B (the later joiner) draws a new id, A keeps X; both A and B log a collision event; C quarantines X until 3 s after it last saw nonce n2 under X, then lifts it.

## Acceptance criteria
1. `hadronyche-l0 collision --duration 15` exits 0.
2. Detection ≤ 1 s after B joins; B's id ≠ X afterwards; A's id = X.
3. Collision event present in both A's and B's logs in the record file.
4. C's quarantine of X ends no earlier than 3 s after the last hello carrying (X, n2), and not later than one hello period after that.
5. The hook use appears in the stamp's injection record.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Holder election (017b-20); the hook itself (017b-06).
