# 004-23 — Design document updates and epic close-out
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-22
## Context
Epic 004's definition of done requires design documents to be updated where behaviour differs (constants table first), no new open item without an owner and a falsifier, no change to the frozen POC (task 003 is the only exception), and every new factual claim to carry an evidence tag (`[code]`, `[measured]`, `[kernel-doc]`, `[standard]` (C-132), `[memory]`, `[decision]`).

Constants that earlier tickets may have changed or falsified `[decision]`:
- C-04 and C-137: 6 channels per stream, 192 bytes per channel per slice, no per-channel id byte; all headers and the checksum at most 100 bytes so a default slice fits 1 252 bytes (004-14; if the cap does not hold, C-137 is falsified and C-04 falls to 5 channels at the fallback).
- C-10: 16-bit timeline epoch (004-07 wrap analysis, with the bump events of C-113 and C-136).
- C-17: control message ≤ 1 252 bytes (004-16 with a 32-member bundle, C-08; 004-18; 004-24 and 004-25 per regime stamp section; 004-26 for the manifest). The path-MTU probe pair is exempt (C-116, 004-27).
- C-20 / C-21 / C-22: about 1 320 bytes per default wire packet, about 21.1 Mbit/s per stream, per-source rates; recompute if the layout changes the packet size by more than 5 % (004-15), with the dependent constants C-24, C-25, C-26, C-90, C-92 and C-118.

Open sub-questions: the epic's "Open sub-questions" table lists each with owner and falsifier. Those this epic answers: epoch wrap (004-07); headers and checksum within the 100-byte cap of C-137 (004-14); hello size and maxima (004-16); checksum (004-13); send timestamp (004-08); magic and version (004-04); copy index values (004-05); counting key for "input" (004-03); announcement payloads (004-09); dense controllers and transport state (004-11, 004-12); measurement extension units (004-10); election set (004-17); admission message set (004-18); join-delay bound (004-26); authentication reservation on sync and measurement messages (004-19, 004-20); probe semantics (004-20). Those owned elsewhere: confirmation of the proposed election and admission message sets (epic 005); further regime stamp section types (the stamping epics). The refusal reasons are not open: all 18 are named in C-185. Settled since the split, no longer open: Demoted carriage (demoted-rounds counter, C-159); architecture specification location (`doc/design/01-architecture.md`, C-171).

## Work
1. Update the design constants table first for every changed or falsified constant, then the prose of the affected design documents, citing `wire.md`.
2. Record the answer to each open sub-question this epic answers in the epic's table and in the design documents' open-items list, closing it or re-opening it with owner and falsifier; confirm the externally owned ones still carry an owner and a falsifier.
3. Check every new factual claim in `wire.md` and in the updated design documents carries an evidence tag.
4. Confirm no file of the frozen POC was modified by this epic.
5. Confirm CI is green and the machine-readable tables parse and round-trip.
6. Integrate epic 001's report notes on receive timestamps and don't-fragment per OS if it has arrived; otherwise leave the placeholders from 004-08/004-19 with an open item owned by epic 001.

## Acceptance criteria
1. Each of the four constant groups above is either confirmed unchanged or updated in the constants table, with the reason.
2. Every open sub-question in the epic has a recorded answer or remains an open item with owner and falsifier.
3. A tag check over `wire.md` finds 0 untagged factual claims.
4. Zero POC files changed.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The machine-readable tables parse and round-trip

## Out of scope
- New protocol content; any change found necessary goes back to the owning ticket.
