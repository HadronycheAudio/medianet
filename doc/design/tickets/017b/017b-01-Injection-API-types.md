# 017b-01 — Injection API types
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** XS · **Depends on:** epic 015

## Context
- Epic 017b builds the L0 loopback harness: one process, ground-truth injection, hard assertions, run unprivileged in CI on every change.
- Ground-truth injection is **public runtime API** in crate `hadronyche-std`, module `inject.rs` `[decision]`. The shape below is given by the epic; names may be refined.
- Injection must always be written into the regime stamp (done in 017b-05); this ticket only defines the types.
- `ModuleHandle` and `ThreadRole` come from the hosted runtime (epic 015) and the real-time setup (epic 016); their exact definition is an open sub-question owned by those epics.
- Units: drift is in parts per billion; a POC value in ppm (`--sim-drift-ppm`) is multiplied by 1 000. `drop_above_bytes` compares the **IPv4 packet size** (the same quantity as a path MTU; the medianet is IPv4 only, C-100).
- The test-only hooks of C-120 (force id and nonce, force epoch, raw send) and the panic hook of C-145 are not part of this public API; they are tickets 017b-06, 017b-07 and 017b-08.

## Work
1. Add `hadronyche-std/src/inject.rs`, exported publicly, with:
```rust
pub struct ClockInjection { pub offset_ns: i64, pub drift_ppb: i64 }

pub struct LinkInjection {
    pub seed: u64,
    pub loss: f64,              // probability per packet
    pub delay_ns: (i64, i64),   // uniform range added per packet
    pub reorder: f64,           // probability a packet is held one slot
    pub drop_above_bytes: Option<usize>,  // MTU blackhole, IPv4 packet size
}

pub trait Injector {
    fn set_clock(&self, module: ModuleHandle, inj: ClockInjection);
    fn step_clock(&self, module: ModuleHandle, step_ns: i64);
    fn set_link(&self, from: ModuleHandle, to: ModuleHandle, inj: LinkInjection);
    fn set_hop_delay(&self, module: ModuleHandle, delay_ns: i64);
    fn partition(&self, groups: &[&[ModuleHandle]]);
    fn suppress_hello(&self, module: ModuleHandle, on: bool);
    fn kill_thread(&self, module: ModuleHandle, role: ThreadRole);
}
```
2. Derive `Debug, Clone, Copy, PartialEq` where possible (needed to serialise into the stamp in 017b-05). Provide `Default` meaning "no injection" (offset 0, drift 0; loss 0, delay (0,0), reorder 0, no blackhole).
3. Add a `LinkInjection::validate()` returning an error when `loss` or `reorder` is outside `[0, 1]`, or `delay_ns.0 > delay_ns.1`.
4. Rustdoc on every item stating units (ns, parts per billion, IPv4 packet bytes, probability per packet), including the ppm-to-ppb conversion.

## Acceptance criteria
1. `hadronyche-std` builds and exposes `inject::{ClockInjection, LinkInjection, Injector}` publicly.
2. Unit tests: `validate()` rejects `loss = 1.01`, `loss = -0.1`, `reorder = 2.0`, `delay_ns = (10, 5)`; accepts `Default` and `loss = 1.0`.
3. `Default` values are exactly offset 0 ns, drift 0 ppb, loss 0.0, delay (0, 0), reorder 0.0, `drop_above_bytes = None`.
4. `cargo doc` produces no missing-docs warning for `inject.rs`; the rustdoc of `drop_above_bytes` states that it is an IPv4 packet size.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Rustdoc for every public item in `inject.rs`

## Out of scope
- Implementing the trait: clock (017b-02), links and hop delay (017b-03), partition / hello / thread kill (017b-04).
- Writing injection into the regime stamp (017b-05).
- Test-only hooks (017b-06, 017b-07, 017b-08).
