# 026b-08 — Conformance vectors over Ethernet
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** M · **Depends on:** 026b-03, epic 007

## Context

Epic criterion 026-AC-5 (Conformance): the device passes the **protocol conformance vectors of task 007** over its **real Ethernet interface** (hello, admission, sync, media, receiver reports). Epic 007 is in the epic's dependency list.

The one implementation rule `[decision]`: control logic lives in `hadronyche-core`; the device implements only a transport trait and drives core. This is what makes one conformance suite cover both profiles (hosted and embedded).

Wire facts: IPv4 only, multicast TTL 1 (C-100); control messages ≤ 1 252 bytes UDP payload (C-17), with scheme 0, zero tag and a live 64-bit counter (C-115); stamp in sections, hello carries a stamp version (C-18); media packet ≈ 1 320 bytes on the wire (C-20) with 1 152 bytes of audio and no per-channel id (C-04), 2 000 packets/s (C-21); 63-entry peer map (C-217, C-34); 50 sync requests/s (C-36), accepted range 10 to 200 (C-112); receiver report per bundle every 100 ms (C-40). The device is follower only; holder-related vectors are expected to be "not applicable" with the holder-eligible bit clear.

The harness is a tool binary in the `hadronyche-tools` crate (C-133), provided by epic 007.

## Work

1. Run the task-007 conformance harness from a Linux host against the board over a wired link (no simulation, no loopback).
2. Cover the five groups: hello, admission, sync, media, receiver reports.
3. Mark vectors that require holder eligibility or capabilities stamped absent or unverified (fault injection, drop counters, DSCP) as N/A with the stamp field justifying it.
4. Add the run to CI if a hardware-in-the-loop runner exists; otherwise document the manual procedure and store the result log in `doc/design/reports/` (C-130).

## Acceptance criteria

1. All applicable vectors in hello, admission, sync, media and receiver-report groups pass; 0 failures.
2. Every N/A vector is justified by a stamp field (holder_eligible = false, or a capability stamped absent or unverified, C-129).
3. Result log stored with board, firmware version and harness version.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope

- Fixing core state machines (owned by their epics); failures caused by core are filed there.
