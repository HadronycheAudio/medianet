# 013-17 — Stamp collector
**Epic:** 013-Regime-stamp-wire-object · **Effort:** M · **Depends on:** 013-05, 013-11, 013-13, 013-15, epic 004, epic 005, epic 015 (single-writer file service)

## Context
- A node that cannot write files (an embedded device) **publishes its stamp over the network and a hosted collector records it**. `[decision]`
- Deliverable in `hadronyche-std`: a **stamp collector** that receives stamps published by any node (including devices) and records them.
- Transport is decided (C-18): a dedicated control message split into sections that each fit 1 252 bytes of UDP payload (C-17); **multicast to the control group (IPv4, TTL 1, C-100) on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch.** The hello carries only a stamp version number (hello handled by epic 005). Oversize sections are split by the rule of 013-13.
- Third-party observability (C-47): stamps carry per-connection summary counters from producer and consumer; receiver reports are not multicast and there is no query message, so the collector is how a third party sees stream health.
- 013-AC-5: a two-endpoint record carries both stamp ids, and **the collector can resolve both**. 013-AC-11: a third party resolves the producer's and consumer's summaries of a connection.
- The `StampId` covers regime fields only; a refresh with new counters keeps the id (013-05).
- The epic settles that the collector's storage format is chosen by this ticket and written into `doc/design/spec/stamp.md`. The store lives in the deployment's data directory, where certificates are stored beside the collected stamps (C-171); files are written through the single-writer file service of epic 015 (C-144). File names within that directory are decided here and recorded in this epic's documentation (C-206).
- Summaries carry the twelve per-connection fields of C-47 and C-159; peer stamps also carry per-peer observed DSCP entries (C-152, 013-26), which the collector stores as counter snapshots.
- `Stamp::parse` handles untrusted bytes; unknown sections are skipped and counted.

## Work
1. Join the control multicast group and receive stamp messages; reassemble split sections per 013-13.
2. When a record references an unknown id, or a hello shows a stamp version the collector has not seen, send a unicast request for an immediate fetch (message per epic 004).
3. Parse each with `Stamp::parse`, drop invalid ones (count them), store valid ones keyed by `StampId` and source node in the deployment's data directory (C-171), writing through the process-owned writer (013-15) and epic 015's file service (C-144). Store regime content once per id; record counter sections from each refresh as time-stamped snapshots.
4. Provide `resolve(id: StampId) -> Option<stored stamp>` for local and peer ids, and `summaries(conn)` returning the latest producer and consumer summaries for a connection.
5. Define behaviour on a 32-bit id collision (same id, different regime bytes): keep both and flag (see 013-05).
6. Choose and document the storage format and its location (C-171).

## Acceptance criteria
1. Integration test: two simulated nodes (one `Embedded` profile, one `LinuxStd`) publish stamps; a record with `stamp` = A and `peer_stamp` = B resolves both through the collector.
2. A record referencing an id not yet received triggers a unicast request and resolves after the reply.
3. Malformed stamps (fuzz-corpus failures) are counted and never crash the collector.
4. A re-published stamp with the same regime content is stored once; its new counters are recorded as a snapshot.
5. For one connection, the producer's and consumer's summaries are both returned by `summaries(conn)`.
6. A colliding id with different regime bytes is detected and reported.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (no field added); "The stamp format is versioned independently of the media format" (the storage format is not the wire format).

## Out of scope
- Publishing a hosted node's own stamp (schedule 013-24, sending epic 015, C-144); storing certificates (their owning epic, C-171); the device-side publisher (epic 026); the message layouts (epic 004); the hello (epic 005); the commissioning tool and monitor that read stamps (epics 028, 029).
