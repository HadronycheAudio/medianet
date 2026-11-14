# 026a-03 — Identity derivation
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** S · **Depends on:** 026a-02, epic 011

## Context

Identity per C-11: **module id 64 bits, instance nonce 64 bits, domain id 16 bits with 0 = unconfigured**.

Rule `[decision]`: devices without a true random source **must not draw the id from a pseudo-random generator** — forty devices with identical firmware would boot with the same id and the same nonce. Instead **mix a hardware unique identifier with a persistent boot counter**.

Settled: the **module id is stable across reboots and the nonce changes on every boot**. The nonce is what distinguishes successive instances of one module id; third parties quarantine an id seen with two nonces until only one nonce has been seen for one hello-expiry period, 3 s (C-11, C-46, C-31). So the id derives from the hardware unique identifier alone, and the nonce from the unique identifier and the boot counter (or a true random source when present). The exact mixing function is an implementation detail, documented in rustdoc.

A hello with domain 0 is ignored and counted by peers, not refused (C-173): a device that has no stored non-zero domain id cannot join. How a domain id is written to a device is not settled: open item, owner commissioning tooling with epic 005, falsifier "a device cannot be brought into a domain without rebuilding its firmware".

Epic criterion 026-AC-4 (verified on hardware in 026b-10): with the true-random source disabled, two devices flashed with identical firmware boot with different ids and nonces; rebooting one keeps its id and changes its nonce.

The `Identity` trait (from 026a-02):

```rust
pub trait Identity {
    fn hardware_unique(&self) -> [u8; 16];
    fn next_boot_counter(&mut self) -> u64;  // persistent
    fn true_random(&mut self) -> Option<[u8; 16]>;
    fn domain_id(&self) -> u16;              // persistent; 0 = unconfigured
}
```

## Work

1. In `hadronyche-embedded`, a function e.g. `derive_identity<I: Identity>(id: &mut I) -> (ModuleId, InstanceNonce)` called once per boot.
2. Module id: derived from `hardware_unique()` only, with a fixed, documented hash folded to 64 bits; no allocation, no libm.
3. Instance nonce: if `true_random()` returns `Some`, use it (folded to 64 bits) mixed with the boot counter; else hash(`hardware_unique` ‖ `next_boot_counter()`) folded to 64 bits. Call `next_boot_counter()` exactly once per boot in both cases so the counter always advances.
4. Never call a PRNG seeded from constants or from the clock alone.
5. Domain id comes from `Identity::domain_id()`; 0 (unconfigured) when none is stored. With 0 the device still runs but expects its hellos to be ignored and counted by peers (C-173).
6. Document the mixing function in rustdoc with the C-11 and C-46 references.

## Acceptance criteria

1. Host test: two mock identities with different `hardware_unique` and `true_random` = `None` produce different module ids and different nonces.
2. Host test: the same mock identity booted twice (boot counter persisted) produces the same module id and a different nonce.
3. Host test: 40 mock identities with distinct unique ids and identical boot counters yield 40 distinct (id, nonce) pairs.
4. With `true_random` = `Some`, the nonce still changes on every boot and `next_boot_counter` is still called once.
5. Host test: with `domain_id()` = 0 the derived identity carries domain 0, and a simulated peer ignores and counts the device's hellos (C-173); with a non-zero domain id it admits them.
6. Device build passes the no-alloc / no-libm check.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] The crate builds for at least one device target and for the host (for tests) in CI

## Out of scope

- Hardware unique id, flash boot counter and stored domain id on the board (026b-02).
- How a domain id is written to the device (commissioning tooling with epic 005).
- Two-device hardware test (026b-10).
- Hello message content and collision quarantine, which core performs (026a-05).
