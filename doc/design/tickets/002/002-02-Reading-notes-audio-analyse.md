# 002-02 — Reading notes: audio.rs analyse and analyse_voices
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-01
## Context
Part of epic 002 (POC archaeology, check 0b). Findings go to section **(d) Reading notes** of `doc/design/reports/archaeology/REPORT-0b.md` (skeleton created in 002-01).

In `audio.rs` of the frozen POC (`/home/dorian/workspace/hadropoc`), the functions `analyse` (line 715) and `analyse_voices` (line 801) were never fully read. Consequence: it is not known whether voice windows are compared bit for bit or only within tolerance. The design inherits the POC loss handling as the **exact-tier gap baseline**, which is why `analyse` must be read. Epics 007 and 017 need "the statement of what `analyse` compares" before citing the POC as exact-tier baseline.

Known loss handling in the POC `[code]` (`audio.rs:20-34`, `audio.rs:620-685`), to be confirmed or corrected by this reading:
- comparison is per **1 024-sample window**;
- a window is compared only when complete on both sides;
- a window still incomplete **256 windows** later (`MAX_LAG_WINDOWS`, 256 × 1 024 samples at 96 kHz, about 2.7 s; 96 kHz is also the design's only wire rate, C-01) is counted lost or as a sender gap and never analysed;
- a synth silent for more than **1 second** gets a **3-second** settling exclusion (`SETTLE_SECS`);
- the sender's replica never applies gaps.

Settling exclusions are also applied at `audio.rs:604` and `audio.rs:644`; the analyzer has a `voices` flag; voice frames are placed by `place_voices`.

The (d) verdict line, written by 002-13, is **not a bug, explanation: …** stating what `analyse` compares, unless a reading finds an instrument bug (epic 002, deliverable (d)); this sub-section must give 002-13 that statement in one sentence.

Open sub-question owned by this epic: **"Is the voice comparison bit-level?"** Falsifier: `analyse` compares within tolerance only; then the "exact tier for voices" description of the design changes.
## Work
1. Read `audio.rs` lines 1-120 (constants), 580-900 (window bookkeeping, `analyse`, `analyse_voices`, `place_voices`) in the frozen tree, read-only.
2. Write in section (d) a sub-section "audio.rs — analyse / analyse_voices" of about half a page:
   - what each function does, with `file:line` pointers;
   - the comparison operator used for audio windows and for voice windows (equality of samples, difference against a threshold, or both), with the threshold value and units if any;
   - the full list of window exclusions (incomplete, lag over `MAX_LAG_WINDOWS`, sender gap, `SETTLE_SECS` settling, `voices` flag off, any other) and the reason for each;
   - whether the five loss-handling facts listed in Context hold; correct any that do not, tagged `[code]`.
3. End the sub-section with a one-sentence statement of what `analyse` compares, usable as the (d) verdict explanation, and flag any instrument bug found.
4. Answer the sub-question with one line: **Voice comparison bit-level: yes** or **no**, with the pointer.
## Acceptance criteria
1. The sub-section exists, is at most about one page, and contains at least 4 `audio.rs:NNN` pointers.
2. The bit-level question is answered with exactly "yes" or "no" (epic criterion 002-AC-5), and if "no" the report says which tolerance is used.
3. Each of the 5 loss-handling facts in Context is marked confirmed or corrected `[code]`.
4. Every exclusion rule listed has a pointer and a stated reason.
5. The one-sentence statement for the (d) verdict is present.
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Open sub-question "Is the voice comparison bit-level?" marked answered in the report's sub-questions table
## Out of scope
- Using this reading to explain the zero music windows of the loopback run (002-06, which consumes these notes).
- Reading notes for `transport.rs`, `control.rs` (002-03) and `sender.rs`, `stats.rs`, `tui.rs` (002-04).
- Updating the design's "exact tier for voices" text and the evidence appendix (002-13).
