# 013-15 — Process-owned record-set writer
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-11, 013-14, epic 015 (single-writer file service)

## Context
- Rule 5: **a file has one writer, owned by the process**, never by a module. `[decision]`
- `[measured]` In the proof of concept's first measured run (25 s loopback, unprivileged, kernel 6.12) the metrics file was corrupt — about 6 parseable lines of about 80 — because four modules each opened the same path. `[code]` Retired POC behaviour: per-module metrics writers on one path (`module.rs:1354`).
- Owner of the single-writer file service (C-144): the hosted backend, epic 015. Opening files, refusing a second writer on a path and serialising concurrent submissions are that service's job; this ticket writes record sets through it and never opens a file itself.
- AC-31 (013-AC-2): every record set carries a complete stamp (all eight groups: Identity, Capabilities, Network, Clock, Alignment, Platform, Drops, Injection); **one writer per file**.
- Each record carries `RecordHeader { stamp: StampId, peer_stamp: Option<StampId>, family: CheckFamily }`; a two-endpoint record carries both ids.
- The epic settles that the record-set file format is chosen by this ticket and written into `doc/design/spec/stamp.md`; no other epic constrains it. File names within the stated directories are presentation details decided here and recorded in this epic's documentation (C-206); the spec carries only what another epic or an operator's script depends on.
- Counters in the stamp (Drops, per-connection summaries) change without changing the `StampId` (013-05). The writer stores the stamp at the head of each record set; counter refreshes may be appended as stamp snapshots inside the set without opening a new one.
- Rules 3 and 6 (blank one-way figures before lock; loss at packet and slice level in the multi-packet regime) are carried out by the producers of figures (epics 015, 017), not by this writer.

## Work
In `hadronyche-std`:
1. A record-set writer type owned by the process; modules submit records to it (e.g. through a channel or handle) and never open the file themselves. The writer obtains its file from epic 015's single-writer file service (C-144); until that service exists, tests use a double with the same interface.
2. Each record set starts with the full stamp bytes (and the peer stamp id where applicable); a stamp that is not complete (`is_complete()` false) is refused.
3. Each record is written with its `RecordHeader`.
4. Choose and document the file format (record framing, stamp snapshot entries) in `doc/design/spec/stamp.md`; add its reader to the 013-03 fuzz harness (C-95 budget).

## Acceptance criteria
1. Test: 4 modules (threads) submitting 1 000 records each produce a file with exactly 4 000 parseable records and no interleaved bytes.
2. The writer opens no file itself: all output goes through the file-service handle (checked with the test double).
3. Every record set in the output begins with a stamp that parses and is complete; an incomplete stamp is rejected at write time.
4. Two-endpoint records in the file carry both stamp ids.
5. A counter-only stamp refresh is written as a snapshot inside the current record set.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (no stamp field added; stamps are written as received); "The stamp format is versioned independently of the media format" (the record-set file is not the stamp wire format).

## Out of scope
- Starting a new record set on stamp-id change (013-16), the zero-sample check (013-18), output labels (013-19).
- The single-writer file service itself: opening, exclusivity per path and across processes (epic 015, C-144).
- Blanking and two-level loss reporting (epics 015, 017).
