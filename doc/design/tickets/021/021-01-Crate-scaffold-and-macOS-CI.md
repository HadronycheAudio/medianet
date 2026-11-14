# 021-01 — Crate scaffold and macOS CI
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** XS · **Depends on:** epic 015, epic 016, epic 018
## Context
Epic 021 makes a macOS machine a certified **edge** member of a medianet: a Core Audio sink and physical-input capture module plus a macOS implementation of the hosted backend traits (host clock, sockets, real-time threads). Everything lands in one new crate, `hadronyche-os-macos`.

- **Edge only** [decision]: the holder-eligible capability bit is never set. The build uses the hosted crates' edge-only feature (no holder candidacy). The feature's exact name is defined by the hosted crates (epics 015, 016, 018), not by epic 021; take it from them.
- The traits to implement come from epics 015, 016, 018 and 020: `HostClock`, `OsNet`, `OsRealtime`, `DeviceClock`, `RawCapture`.
- v1 is IPv4 only (C-100); the crate needs no IPv6 code path.
- CI must build the crate on macOS; hardware tests are a documented manual job, not CI.
- The few tests that need a privilege (for example stepping the wall clock, 021-28) run in a separate privileged nightly job; the definition-of-done line "unprivileged" applies to per-change CI (C-146).
## Work
- Create crate `hadronyche-os-macos` with empty modules: `clock.rs`, `net.rs`, `rt.rs`, `power.rs`, `permissions.rs`, `coreaudio/mod.rs`, `coreaudio/sink.rs`, `coreaudio/capture.rs`, `coreaudio/guard.rs`.
- Depend on the hosted crates (015, 016, 018) with their edge-only feature enabled; do not enable any holder-candidacy feature.
- Gate the crate to `target_os = "macos"` so non-macOS workspace builds are unaffected.
- Add a macOS CI job that builds the crate and runs its unit tests unprivileged.
- Add a privileged nightly macOS job that runs only the tests marked as needing privilege (C-146).
- Add a short `HARDWARE-TESTS.md` (or section in the crate README) describing the manual hardware job: who runs it, on which machine, which tests are `#[ignore]`-gated hardware tests, and how results are recorded. Later tickets append their hardware tests to it.
## Acceptance criteria
1. `cargo build -p hadronyche-os-macos` succeeds in the macOS CI job.
2. The crate's dependency tree enables the hosted edge-only feature and no holder-candidacy feature (checked by `cargo tree -e features` in CI or a compile-time assertion).
3. The Linux CI jobs still pass with the new crate in the workspace.
4. The manual hardware job is documented in the crate.
5. A test marked privileged is skipped by the per-change job and run by the privileged nightly job (C-146).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
Any trait implementation (021-02 onward). The holder-bit runtime test (021-19).
