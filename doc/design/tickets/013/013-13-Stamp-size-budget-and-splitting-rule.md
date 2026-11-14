# 013-13 — Stamp size budget and splitting rule
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-06, 013-07, 013-08, 013-09, 013-10, 013-23, 013-26, 013-27, epic 004

## Context
- Epic acceptance criterion 013-AC-10: a **device-profile stamp builds without an allocator, and every message carrying it is at most 1 252 bytes of UDP payload (C-17, C-18); the rule for splitting a section larger than one message is specified and tested.**
- Transport is decided (C-18): a dedicated control message **split into sections that each fit C-17**; multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The former question "does a full stamp fit one control message?" is closed: it need not.
- Control message bound C-17: **at most 1 252 bytes of UDP payload**, the 1 280-byte IPv4 fallback MTU (C-06, C-100) less headers. The message header and the authentication reservation (C-12; in v1 scheme 0 and tag zero, live 64-bit counter, C-115) are part of that budget; their layout is epic 004's.
- Falsifier recorded by the epic for C-18: a single section exceeds 1 252 bytes (for example the per-connection summaries of a 32-member bundle source, C-08). Repeated sections (per-connection entries 013-07, per-connection summaries 013-23 with the twelve fields of C-47 and C-159, per-peer observed DSCP entries 013-26 for up to 63 peers under C-34, with the holder-only dropped sync request count of 013-27, C-204) and possibly the canonical configuration (013-04) can exceed one message; no constant bounds the number of connections in one stamp (C-09, C-34).
- A node that cannot write files (embedded device) publishes its stamp over the network and a hosted collector records it.

## Work
1. Compute the maximum encoded size of each fixed section, and the per-entry size of each repeated section, for each profile (maximum-length strings, all ten injection kinds (C-120, C-145), canonical configuration at its maximum from 013-04). Record the table, with the number of entries of each repeated section that fit one message.
2. Specify the splitting rule for a section larger than one message: a repeated section is cut at entry boundaries into parts; each part carries the `StampId`, the section type, a part index and a part count, so a receiver knows when it has them all; a non-repeated section larger than one message is a build error.
3. Implement the splitting (writer) and reassembly (reader) in `hadronyche-core` without allocation, the reader working over caller-provided storage.
4. Add the reassembly reader to the 013-03 fuzz harness (C-95 budget).
5. Add a test that builds a worst-case `Embedded` stamp into a fixed stack buffer with no allocator.

## Acceptance criteria
1. A worst-case `Embedded`-profile stamp builds in a `no_std`, no-`alloc` test into a fixed buffer, and every message it is sent in is ≤ 1 252 bytes.
2. The size table lists every section for all 4 profiles, with entries-per-message for repeated sections.
3. Splitting then reassembling stamps with 32 and 256 connection summaries yields the original bytes and the same `StampId`; reassembly of parts in any order succeeds; a missing part is detected and named; a duplicate part is ignored.
4. A non-repeated section that cannot fit one message is refused by the builder with an error.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The stamp format is versioned independently of the media format, with the rule for unknown sections (skip and count)
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (no field added; parts only carry existing sections).

## Out of scope
- Sending the messages (hosted backend epic 015 on the schedule of 013-24, C-144; device side epic 026) and receiving them (013-17).
- The control message header layout (epic 004).
