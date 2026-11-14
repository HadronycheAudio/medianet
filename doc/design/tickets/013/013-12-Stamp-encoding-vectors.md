# 013-12 — Stamp encoding vectors
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-06, 013-07, 013-08, 013-09, 013-10, 013-13, 013-23, 013-25, 013-26, 013-27, epic 007

## Context
- Epic acceptance criterion 013-AC-1 / system criterion **AC-1**: stamp vectors **encode and decode byte for byte**.
- Epic 007 supplies the vectors for the stamp encoding, from the byte layout and the reject reasons of epic 004 (C-193). The vectors crate sits at the workspace root (C-192).
- The stamp is a versioned envelope of length-prefixed, versioned sections (Identity, Capabilities, Network + per-connection entries, per-connection summaries (C-47), Clock, Alignment, Platform including the calibration factor (C-106), Drops, Injection including the commissioning window (C-107, C-153) and test hooks (C-120, C-145), per-peer observed DSCP (C-152), alignment measurements (C-157, C-160), Self-test); unknown sections are skipped and counted. It travels as sections that each fit 1 252 bytes (C-17, C-18), with oversize sections split by 013-13.
- The `StampId` covers regime sections only; counters (Drops, summaries, per-peer observed DSCP, alignment measurements) are excluded (013-05).

## Work
1. Use the epic 007 stamp vectors from the vectors crate at the workspace root (C-192) in `hadronyche-core` tests (each vector: input field values and expected bytes, or input bytes and the expected reject reason of epic 004, C-193).
2. Test both directions: building from the field values yields exactly the expected bytes; parsing the expected bytes yields exactly the field values and the expected `StampId`. For each reject vector, parsing yields exactly the expected reject reason.
3. Include at least: one vector per profile (4), one with an active injection, one with an unknown section, one minimal device-profile stamp, one with per-connection summaries, one split section (013-13), and a pair differing only in counters with equal `StampId`.
4. Add every vector to the fuzz seed corpus (013-03).

## Acceptance criteria
1. 100 % of epic 007 stamp vectors pass in both directions, byte for byte; every reject vector yields its expected reject reason (C-193).
2. Vectors cover all 4 profiles, an active injection, an unknown section (skipped count = 1), per-connection summaries, and a split section.
3. The counters-only pair has equal ids.
4. Tests run in CI unprivileged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The stamp format is versioned independently of the media format, with the rule for unknown sections (skip and count)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (no field added); "Design documents updated where behaviour differs, constants table first" (no behaviour change; follows C-192 and C-193); "No new open item without an owner and a falsifier" (none opened); "Every new factual claim carries an evidence tag" (no factual claim).

## Out of scope
- Producing the vectors (epic 007). If epic 007 vectors are not yet available, this ticket is blocked; it does not invent its own "official" vectors.
