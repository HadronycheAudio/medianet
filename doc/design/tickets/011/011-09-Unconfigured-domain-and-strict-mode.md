# 011-09 — Unconfigured domain and strict mode
**Epic:** 011-Gossip-and-discovery-core · **Effort:** XS · **Depends on:** 011-07

## Context
C-11 (decided): domain id is 16 bits; domain 0 means unconfigured. Rule `[decision]`: strict mode refuses to start with domain 0.

The epic keeps `Discovery::new(me: LocalInfo, rng: &mut dyn IdSource) -> Self` infallible and adds a separate start check in core, which the platform calls before `Discovery::new`:
```rust
pub enum StartError { UnconfiguredDomain }
pub fn check_start(domain: Domain, strict: bool) -> Result<(), StartError>;
```
Strict mode is set by the operator policy in the commissioning configuration (C-165). It is hosted-only (C-147): the hosted platform (epic 015) reads it and passes it to `check_start`; a device (epic 026) has no strict mode and passes `strict = false`. The other strict-mode requirements (real-time scheduling on the data and clock threads, memory locked, C-127; an unverified capability refused, C-128) are checked by epic 015, not by core.

## Work
1. Implement `check_start`, returning `Err(StartError::UnconfiguredDomain)` when `strict` and `domain == Domain(0)`.
2. Document that platforms must call it before `Discovery::new`, and that in non-strict mode domain 0 is accepted at start but such a module learns no peers: every hello carrying domain 0 is ignored and counted (C-173), and hellos of a non-zero domain are foreign.

## Acceptance criteria
1. `check_start(Domain(0), true)` is `Err(UnconfiguredDomain)`.
2. `check_start(Domain(0), false)`, `check_start(Domain(7), true)` and `check_start(Domain(7), false)` are `Ok(())`.
3. A non-strict module with domain 0 drops a domain-7 hello as `ForeignDomain`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Every parser touched is fuzzed" (no parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "The frozen POC is not modified" (no code near the POC); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Reading the strict flag from the commissioning configuration (C-165), the other strict-mode checks (C-127, C-128) and reporting a refusal to the operator: hosted platform epic 015.
