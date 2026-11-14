# 029-23 — Monitor documentation and design updates
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-18, 029-19, 029-20, 029-21, 029-22, 029-24, 029-25

## Context
- Closing ticket for epic 029 (`hadronyche-monitor`, crate `hadronyche-tools`).
- Epic definition-of-done items that are documentation-level: a screenshot or text capture of each panel is in the tool's documentation; design documents updated where behaviour differs, constants table first; no new open item without an owner and a falsifier; every new factual claim carries an evidence tag (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`, and `[standard]` for a figure cited from a named published standard with its clause, C-132).
- The epic marks "Regime stamp fields touched are complete and documented" as not applicable (C-125): the monitor adds or changes no stamp field.
- Decided sub-question: third-party observation of stream health is via per-connection summary counters (received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159)) and the observed DSCP in each endpoint's regime stamp (C-47); reports are not multicast and there is no query message. Falsifier: the stamp summaries are too coarse to diagnose a fault.
- Open sub-question owned by Product (not this task): build the monitor in v1 at all, decided at the start of P7; falsifier: operators manage with `survey` and `watch` alone.
- The epic's other open sub-questions (recorded file format, bundle member as connection, where some fields are published, stamp version semantics, C-91 value; D is settled by C-170, the L0 owner by C-192, and the headless line by C-206, which places its format and the panel layouts in this ticket's tool documentation) each have an owner and a falsifier in the epic.

## Work
1. Tool documentation for `hadronyche-monitor`, in the operator guide under `doc/operations/` (C-171): purpose and read-only guarantee (IGMP only, no stamp request; C-18, C-100), command line (`--file`, `--headless`, `--interval`), the seven panels each with a text capture produced by `render_to_text` (Medianet, Clock, Connections, Bundles, Ports, Sinks, Regime) including the id-collision and departure marks (029-24) and the alarm marks (029-25), the headless line format, vocabulary (connection states, Demoted, gap classes, clock states, capability states, certification states) with their C-ids.
2. Update design documents wherever the implementation differs from the epic, constants table first (as used: C-06, C-08, C-17, C-18, C-19, C-21, C-23, C-30, C-31, C-34, C-38, C-39, C-45, C-46, C-47, C-62, C-64, C-65, C-70, C-73, C-90, C-91, C-100, C-101, C-102, C-103, C-106, C-107, C-108, C-109, C-111, C-113, C-117, C-120, C-122, C-124, C-127, C-129, C-131, C-135, C-136, C-139, C-140, C-145, C-152, C-153, C-154, C-157, C-159, C-160, C-161, C-162, C-167, C-168, C-169, C-170, C-173, C-174, C-178, C-180, C-182, C-185, C-187, C-192, C-204, C-205, C-206, C-209). Under C-206 the headless line format, default interval and panel layouts are recorded in the tool documentation (item 1), not in the specifications, except any part another epic or an operator's script depends on.
3. Update the epic's open sub-questions with any answer received while the tickets ran; record any further gap found in 029-02 to 029-25 (for example fields shown as `NotPublished`) as an open item with owner and falsifier.
4. Record, against the C-47 falsifier, whether the 029-21 tests could diagnose each injected fault from stamp summaries alone; write this to `doc/design/reports/` (C-130).

## Acceptance criteria
1. The documentation contains 7 panel captures and the headless format.
2. Every constant used by the monitor appears in the constants table with its C-id, and the monitor's constants module matches the table.
3. Every open item added has an owner and a falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- New features; the Product decision on building the monitor in v1.
