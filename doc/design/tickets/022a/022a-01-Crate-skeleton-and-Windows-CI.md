# 022a-01 — Crate skeleton and Windows CI
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** epic 015, epic 016

## Context
- Epic 022 (parts 022a and 022b) makes a Windows machine a certified **edge** member of a medianet: a sink and a physical-input capture module on WASAPI in exclusive mode, plus a Windows implementation of the hosted backend traits (sockets, host clock, real-time threads). The Windows edge never holds the clock and never stands for election `[decision]`.
- All the work lands in one new crate, `hadronyche-os-windows`. Its planned modules are: `clock.rs`, `net.rs`, `rt.rs`, `firewall.rs`, `power.rs`, `wasapi/sink.rs`, `wasapi/capture.rs`, `wasapi/device.rs`, `deadline.rs`.
- The build uses the **edge-only feature**; the holder-eligible capability bit is never set `[decision]`.
- The hosted trait surface it implements (`HostClock`, `OsNet`, `DatagramSocket`, `OsRealtime`, `DeviceClock`, `RawCapture`, the core deadline-clock trait) comes from epics 015 and 016.
- Code placement (C-133): this crate holds the backend; harness and tool binaries (the L2 harness Windows build, the hardware-job runner) live in the `hadronyche-tools` crate and throwaway probes in `probes/`.
- The medianet is IPv4 only (C-100); no IPv6 dependency is needed.
- Definition-of-done rule for the whole epic: CI builds the crate on Windows; hardware tests are a documented manual job; per-change CI tests run unprivileged. The few tests that need a privilege (for example administrator rights for a QoS marking route or a firewall block rule) run in a separate privileged nightly job; the definition-of-done line "unprivileged" applies to per-change CI (C-146).

## Work
1. Create crate `hadronyche-os-windows` in the workspace, compiled only for `target_os = "windows"` (empty or a `compile_error!`-free stub on other targets so the workspace still builds on Linux).
2. Add empty modules `clock`, `net`, `rt`, `firewall`, `power`, `deadline`, and `wasapi::{sink, capture, device}` with a module doc comment each naming its responsibility (as listed in Context).
3. Depend on the core crates that define the hosted traits (epics 015, 016) with the edge-only feature enabled; do not enable any holder/election feature.
4. Add the Windows API binding dependency needed by later tickets (Win32 sockets, QPC, MMCSS, WASAPI, power, firewall APIs).
5. Add a CI job on a Windows runner that builds the crate and runs its unit tests, unprivileged (no administrator rights), on every change. Add a separate privileged nightly Windows job that runs only the tests marked privileged (C-146), with a test attribute or feature that marks them.
6. Add a `#[cfg(test)]` smoke test so the CI job runs at least one test.

## Acceptance criteria
1. `cargo build -p hadronyche-os-windows` succeeds on the Windows CI runner.
2. `cargo test -p hadronyche-os-windows` runs at least one test and passes on the Windows CI runner without administrator rights.
3. The full workspace still builds on Linux with the new crate present.
4. The crate's dependency tree enables the edge-only feature and no holder-eligible / election feature (checked by inspecting `cargo tree -e features` in CI or a test).
5. A test marked privileged is skipped by the per-change job and run by the privileged nightly job (C-146).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on Windows; hardware tests are a documented manual job
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Crate and module layout match the list in Context

## Out of scope
- Any trait implementation (022a-02 onward).
- The documented manual hardware job (022b-01).
