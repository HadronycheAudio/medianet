# 009-02 — No-alloc test allocator
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-01

## Context
Epic 009's receive path (source filter, dedup, reassembler, alignment ring, fan-in, degraded delay line, concealment) must perform no allocation and take no lock `[decision]`. The definition of done requires the allocation part to be checked by a no-alloc test allocator; the epic's deliverables add a lint that rejects lock types in the receive path, which this ticket also provides. Every later ticket of the epic runs its receive-path tests under the harness built here.

The receive path is pure `no_std` code in `hadronyche-core`, module `rx`; capacities are const generics, so all memory is owned by the structs themselves.

## Work
1. Add a test-only support crate or module (proposed: `hadronyche-core/tests/support/no_alloc.rs`, or a dev-dependency crate `hadronyche-test-alloc`) providing:
   - a `#[global_allocator]` wrapping the system allocator that counts allocations, reallocations and deallocations per thread while a guard is active;
   - `pub fn assert_no_alloc<R>(f: impl FnOnce() -> R) -> R` which runs `f` and panics with the count if any allocation or reallocation happened on the current thread during `f`.
2. A self-test proving the harness fires: `assert_no_alloc(|| Vec::<u8>::with_capacity(1))` panics; `assert_no_alloc(|| [0u8; 16])` does not.
3. Lock check: add a `clippy.toml` `disallowed-types` / `disallowed-methods` list for the `rx` module (at least `std::sync::Mutex`, `std::sync::RwLock`, `spin::Mutex`, `spin::RwLock`, `critical_section::with`) and run clippy with `-D warnings` on `rx` in CI.
4. Document in the module doc how later tickets wrap their tests (one paragraph, with an example).

## Acceptance criteria
1. The harness self-test detects 1 allocation inside the guard and reports the count 1.
2. Zero allocations are reported for a closure that only uses stack arrays.
3. CI fails if any disallowed lock type is used inside `rx` (demonstrated once with a throw-away commit or a `compile_fail`-style clippy test).
4. The harness runs in an unprivileged CI job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Harness usage documented in the module doc

## Out of scope
- Wrapping each receive-path component's tests in the harness: done in each component ticket (009-03 to 009-24).
- Runtime (non-test) allocation tracking on devices (epic 026).
