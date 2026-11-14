# 013-20 — Strict mode start refusal
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-01, 013-06, 013-10

## Context
- Rule 10: **strict mode turns a failed self-test item, or domain 0 (unconfigured), into a refusal to start**, naming the item. The epic settles that "failed check" at start-up means a failed self-test item, since no other check has run yet. Strict mode is a `hadronyche-std` deliverable of the epic. `[decision]`
- Domain id is 16 bits, **0 = unconfigured (C-11)**.
- Strict mode is **hosted-only** (C-147): a device stamps real-time scheduling and memory lock as present by construction and has no strict mode. Any run whose numbers enter the constants table runs in strict mode; L0 does not (C-147; the harness that runs them is not this ticket).
- Strict mode is set by the **operator policy in the commissioning configuration** (C-165).
- What strict mode requires (C-127): **real-time scheduling granted on the data and clock threads, and memory locked**. Restricting CPU idle states stays best-effort and is stamped; it never causes a refusal.
- Strict mode and an unverified capability (C-128): **refused; unverified is treated as absent**. Capabilities have four states (C-129); since strict mode requires the two capabilities to be granted, "not requested" is refused as well, with a message saying it was not requested.
- Self-test items (Rule 9): receive timestamps present and consistent with the monotonic clock (`rx_stamps_consistent`); real-time scheduling actually granted (`rt_granted`); measured wake latency (`wake_latency_ns`); packets per slice equal to the stamped regime's value, one at defaults, C-05 (`packets_per_slice`); clock locked within its lock bound, C-62 (`locked_within_bound`).
- 013-AC-7: strict mode refuses to start with domain 0, with any failed self-test item, or with `rt_scheduling` or `memory_locked` in any state other than present, **naming the item**.
- The verdict per item comes from the producer (013-10); the wake-latency limit and receive-stamp tolerance are still open (owner: a measurement). This ticket consumes the verdict and does not set thresholds.

## Work
In `hadronyche-std`:
1. A strict-mode switch in the runtime start-up path, read from the operator policy in the commissioning configuration (C-165); hosted profiles only (C-147).
2. After building the stamp and running the self-test, if strict: refuse with domain 0, with any failed self-test item, or with `rt_scheduling` or `memory_locked` Absent, Unverified or NotRequested; exit non-zero with a message naming domain 0, the failed item(s) or the capability and its state. `cpu_idle_restricted` is never a cause of refusal.
3. Without strict mode, start and rely on unqualified labelling (013-19).

## Acceptance criteria
1. Strict, domain 0: refuses, exit non-zero, message names domain 0 as unconfigured.
2. Strict, each of the 5 self-test items failing in turn: refuses and names that item (5 cases).
3. Strict, `rt_scheduling` and then `memory_locked` in each of Absent, Unverified and NotRequested: refuses and names the capability and state (6 cases).
4. Strict, domain ≠ 0, all items passing, both capabilities Present, `cpu_idle_restricted = Absent`: starts.
5. Strict, domain ≠ 0 and all items passing with both capabilities Present: starts.
6. Not strict, domain 0: starts (output unqualified as per 013-19 rules).
7. The switch is taken from the commissioning configuration's operator policy (C-165); with policy strict the cases above apply, with policy not strict case 6 applies.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Every parser touched is fuzzed" (no parser); "Regime stamp fields touched are complete and documented", "Every stamp field has a stated source on each of the four profiles" and "The stamp format is versioned independently of the media format" (no stamp change); "Design documents updated where behaviour differs, constants table first" (follows Rule 10, C-127, C-128, C-147 and C-165 as settled; C-177 and C-201 are out of scope); "No new open item without an owner and a falsifier" (none opened).

## Out of scope
- Running the self-test measurements themselves; setting thresholds.
- A strict mode on devices (none, C-147); making ladder runs strict (the harness, C-147).
- The refusal to start when a thread layout's seqlock safety depends on a CPU pin that failed or is unverified (C-177, C-201): it applies in every mode, strict or not, so it is not a strict-mode check; it belongs to the owner of the thread layout, not this ticket.
