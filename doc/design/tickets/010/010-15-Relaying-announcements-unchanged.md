# 010-15 — Relaying announcements unchanged
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-02, 010-13
## Context
- Epic AC-8 = system criterion AC-17 (send part): descendants receive the announced position **unchanged** through an intermediate arrival-driven module.
- Announcements name a future position (issue position + 64 slices, C-43) and are re-issued every 16 slices until it passes (C-44).
- Position stamps are never restamped per hop; a module with structural delay L stamps its output p − L while processing input p (L is a whole number of samples, C-105).
- Settled in the epic: the relay entry point is part of the deliverables (`relay_announcement`), and the relayed target is not recomputed and not shifted by the module's L. Repeated receipts of the same re-issued announcement give one pending entry.
- C-208 (with C-186): a re-alignment after a changed A is not relayed. Each fan-in that learns of the change picks its own position one lead ahead (C-43) and announces it in its own output with `announce` (010-13), exactly as for a ratchet-triggered re-alignment; fan-ins at different depths switch at different, announced positions. `relay_announcement` is for announcements a module passes on, not for a re-alignment the module itself performs; the doc comment says so and cites C-208.
- C-215: a ratchet step is a re-alignment like any other; the fan-in announces it one lead (C-43) ahead in its own output through `announce` (010-13), never silently on the next round. It is not relayed either; the doc comment cites C-215 next to C-208.

## Work
- In `hadronyche-core`, add `Packetiser::relay_announcement(kind: Announcement, target_position: u64, now_position: u64)` (name may be refined): stores the received target as is and re-issues every 16 slices from `now_position` until the target passes, with the same rules as 010-13.
- Make it available on the bundle packetiser of 010-14 too (every member).
- Ignore a relayed announcement whose target has already passed; ignore a duplicate (same kind and target) already pending.

## Acceptance criteria
1. Test chain source → arrival-driven intermediate (L = 0 and L = 48) → sink, all in-process: the sink decodes the same target position the source announced (epic AC-8).
2. The intermediate receives the announcement 4 times (re-issues) but holds one pending entry.
3. A relayed announcement whose target is behind `now_position` is not emitted.
4. Relayed announcements re-issue every 16 slices and are absent after the target passes.
5. On a bundle packetiser, a relayed announcement appears in every member.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Receive-side decoding (epic 009); the test may use the epic 009 decoder or a test-only decoder.
- Receive part of system AC-17.
- Deciding when a fan-in re-aligns after a changed A or a ratchet step and which position it picks (C-186, C-208, C-215): the alignment logic of the consumer; it issues its own re-alignment through `announce`.
