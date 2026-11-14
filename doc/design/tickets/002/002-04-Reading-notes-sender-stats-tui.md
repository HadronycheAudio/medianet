# 002-04 — Reading notes: sender.rs, stats.rs and tui.rs
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-01
## Context
Part of epic 002 (POC archaeology, check 0b). Findings go to section **(d) Reading notes** of `doc/design/reports/archaeology/REPORT-0b.md`.

Three files of the frozen POC (`/home/dorian/workspace/hadropoc`) were never fully read: most of `sender.rs`, most of `stats.rs`, and `tui.rs`.

They carry the loss-counter chain behind the anomalous "lost" value of about 21.7 million seen in a clean run where only about 50 000 packets were sent `[measured]`:
- per-input counters in the mixer (`mixer.rs:78`) and synth (`synth.rs:88`): `(highest − first + 1) − rx_count`, saturating arithmetic on sequence numbers `[code]`;
- written to the echo header field `rx_lost` (`synth.rs:168,195`, `mixer.rs:167,194`) `[code]`;
- copied into the sender's record (`sender.rs:435`) `[code]`;
- stored as `lost_fwd` in the statistics, latest record wins across all return streams, no per-stream separation (`stats.rs:324`) `[code]`;
- printed by the one-line display (`tui.rs:107-114`) `[code]`.

Also relevant: the sender verifies returned content by rerunning the echo's code on its own copy; `sender.rs:229` tests `ctx.is_reference()`; the sender-side metric "TX pacing lateness" is used in 002-10.
## Work
1. Read the three files in full in the frozen tree, read-only.
2. Write in section (d) one sub-section per file, about half a page each: purpose, main types and functions with `file:line` pointers, what is decoded from which header field, and what is displayed or recorded.
3. In the `sender.rs` / `stats.rs` / `tui.rs` notes, describe the hops `sender.rs:435`, `stats.rs:324`, `tui.rs:107-114` precisely: the field read, its type and width, and any unit conversion. Flag any point where a field could be decoded from the wrong offset or reused by direction (input to 002-08).
4. Note where "TX pacing lateness" is computed and in which unit.
## Acceptance criteria
1. Three sub-sections exist, each about half a page, each with at least 3 `file:line` pointers.
2. Each of the 3 hops listed in step 3 is described with field name, type and unit, tagged `[code]`.
3. The computation site and unit of "TX pacing lateness" are given with a pointer.
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
- Logging values at runtime and the verdict on the 21.7 M counter (002-07, 002-08).
- Reading `audio.rs` (002-02), `transport.rs`, `control.rs` (002-03).
