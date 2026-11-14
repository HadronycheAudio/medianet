# 015a-08 — Single-writer file service (files.rs)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-01
## Context
- File: `hadronyche-std/src/files.rs`.
- Rule `[decision]`: a file has **one writer, owned by the process**, never by a module. Writes go through a dedicated writer thread, because under I/O load the kernel can hold a writer for seconds `[code]` (POC comment at `src/module.rs:1351`).
- POC defect `[code]`: `src/module.rs:1354` — each module calls `File::create` on the same metrics path; in loopback four handles overwrite each other and the file is corrupt `[measured]`.
- Real-time threads (data, clock) must never block on file I/O, and no lock may be shared between housekeeping and data or clock threads (L-8).
- Framing (epic): one record per line. The record content schema is still open; its consumer, the L1 harness (epic 019), owns it. This ticket handles opaque line records.
## Work
1. A `FileService` that owns one writer thread and a registry `path → handle`; opening the same path twice returns the same logical handle.
2. Producers submit complete records (one line each, no embedded newline) through a bounded lock-free queue per producer; the writer thread writes each record whole (no interleaving within a line).
3. A full queue drops the record and counts the drop; producers never block. Publish the drop count lock-free (015a-18).
4. Flush and close on `Runtime` shutdown.
## Acceptance criteria
1. Two handles requested for the same path from different producers write into one file opened once.
2. 4 producers × 100 000 lines into one path produce a file of exactly 400 000 well-formed lines minus the counted drops; no line fusion.
3. A producer's submit never blocks: with the writer thread stalled (test hook), 10 000 submits return in < 10 ms total, and the drop counter is correct.
4. No `std::sync::Mutex` or other lock is taken by a submitting thread.
5. A record containing a newline is refused at submit (counted), so framing cannot break.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "Design documents updated" — no behaviour differs; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — no receive path.
## Out of scope
- Metrics record schema (epic 019).
- The four-modules-in-one-process test through `Runtime` (015b-08).
