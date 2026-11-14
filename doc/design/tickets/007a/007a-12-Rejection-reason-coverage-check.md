# 007a-12 — Rejection reason coverage check
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** XS · **Depends on:** 007a-11

## Context
- Epic criterion 007-AC2: every rejection reason has at least one invalid vector; coverage is checked mechanically against the reason enumeration.
- The rejection reason enumeration is part of the machine-readable wire tables (epic 004). Invalid vectors live in `vectors/wire/invalid/`, each `.json` naming one `reason`.

## Work
1. Add a test that loads the reason enumeration and the `reason` of every invalid vector, and fails listing each reason with zero vectors.
2. Run it in CI.

## Acceptance criteria
1. The test passes on the committed vectors: 100 % of reasons covered.
2. Adding a new reason to the enumeration without a vector makes the test fail naming that reason.

## Definition of done
- [ ] Tests green in CI, unprivileged

## Out of scope
- Writing the invalid vectors (007a-11, 007a-13).
