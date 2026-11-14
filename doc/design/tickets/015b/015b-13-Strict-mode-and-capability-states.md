# 015b-13 — Strict mode and capability states (C-127 to C-129, C-131, C-177, C-201)
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-06, 015a-17, 015a-18
## Context
- Strict mode (C-127) requires real-time scheduling granted on the data and clock threads, and memory locked. Restricting CPU idle states stays best-effort and is stamped.
- In strict mode an unverified capability is refused: unverified is treated as absent (C-128).
- Capability states in the regime stamp (C-129): present, absent, unverified, not requested. "Not requested" marks a deliberate configuration choice, as distinct from a denial.
- Receive-timestamp sources (C-131): kernel, userspace, mac-hardware, interrupt, each reported with a C-129 state. On Linux only kernel and userspace occur; mac-hardware and interrupt are stamped "not requested".
- Strict mode is operator policy, set in the commissioning configuration (C-165). It is hosted-only: a device stamps real-time scheduling and memory lock as present by construction (C-147). Any run whose numbers enter the constants table runs in strict mode (C-147), so 015b-01, 015b-10 and 015b-15 depend on this ticket.
- Seqlock safety and CPU pinning (C-177, C-201): if a thread layout's seqlock safety depends on a CPU pin and that pin fails, or is unverified, the module refuses to start in every mode, strict or not. Default priorities (clock above data) never depend on pinning, so the default layout starts without any pin.
- Control-isolation capability (C-161): housekeeping cannot delay the data path. On hosted platforms it is present by construction of the thread model, so this backend stamps it present.
- Task 016 performs real-time setup and memory locking and reports what it obtained through the `Runtime` hook (015a-17); until 016 lands, the no-op hook reports "unverified" for both, so strict mode refuses — the correct C-128 outcome.
- Unprivileged CI normally cannot obtain real-time scheduling or lock large memory, so the CI tests run with a fake hook that reports each state.
## Work
1. Map the hook's per-thread results to C-129 states for: real-time scheduling (data threads, clock thread), memory locking, CPU idle restriction, and the CPU pin of any thread whose layout's seqlock safety depends on it.
2. Strict mode: if real-time scheduling on any data or clock thread, or memory locking, is not present (absent or unverified), refuse to start with a named reason per capability. CPU idle restriction never refuses.
3. Pin-dependent seqlock safety, in every mode: if the layout depends on a CPU pin and that pin is not present (absent or unverified), refuse to start with a named reason (C-177, C-201).
4. Non-strict mode: otherwise start, and stamp each capability's state; a capability turned off in configuration is "not requested".
5. Receive timestamps: from the per-socket source counts (015a-06), stamp kernel as present (kernel stamps seen), absent (requested but none delivered after traffic) or unverified (no traffic yet); userspace as present when used.
6. Stamp the control-isolation capability present (C-161).
7. Read the strict-mode setting from the commissioning configuration (C-165).
8. Publish all states to the counter board (015a-18).
## Acceptance criteria
1. Strict mode with a hook reporting real-time absent, or unverified, on any data or clock thread refuses to start with a reason naming the thread and capability.
2. Strict mode with memory locking absent or unverified refuses; with everything present it starts.
3. Strict mode with CPU idle restriction absent starts and stamps it absent.
4. Non-strict mode with everything absent starts with the default layout and stamps every state; a capability disabled in configuration is stamped "not requested", never "absent".
5. On loopback the stamp shows kernel receive timestamps present, mac-hardware and interrupt not requested.
6. The stamp shows control isolation present in every mode (C-161).
7. Strict mode is switched on and off through the commissioning configuration (C-165).
8. With a layout whose seqlock safety depends on a CPU pin, a hook reporting that pin absent, or unverified, refuses to start in strict and in non-strict mode with a reason naming the thread and the pin; with the default layout no pin is required in either mode (C-177, C-201).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Design documents updated" — follows the epic; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a receive-cost test.
## Out of scope
- Obtaining real-time scheduling, memory locking and CPU idle restriction (task 016).
