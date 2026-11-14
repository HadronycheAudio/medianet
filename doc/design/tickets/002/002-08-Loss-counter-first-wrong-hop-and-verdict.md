# 002-08 — Loss counter: first wrong hop and verdict
**Epic:** 002-POC-archaeology · **Effort:** XS · **Depends on:** 002-04, 002-07
## Context
Part of epic 002 (POC archaeology, check 0b), deliverable **(b) The 21.7 M counter**, steps 2 and 3, and the (b) verdict.

The display showed "lost" ≈ 21.7 million in a clean run of ≈ 50 000 packets `[measured]`; magnitude suggests nanoseconds (21.7 ms). Chain `[code]`: counters `mixer.rs:78` / `synth.rs:88` → header field `rx_lost` (`synth.rs:168,195`, `mixer.rs:167,194`) → sender record (`sender.rs:435`) → `lost_fwd` in stats, latest record wins, no per-stream separation (`stats.rs:324`) → display (`tui.rs:107-114`).

002-07 produced a per-hop log; 002-04 describes the field, type and unit at each sender-side hop.

The runtime does not inherit this counter: it publishes per-connection summary counters in each module's regime stamp (C-47): received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159). The verdict here is about the POC instrument only; the hand-off to task 003 concerns the POC fix.

Open sub-question owned by this epic: **"Is the 21.7 M value a decoding error or a display error?"** Falsifier: neither; the value is a real counter and the loss accounting is wrong.
## Work
1. From the 002-07 log, identify the **first hop** where the value is no longer a small packet count (≤ number of packets sent on that input).
2. Name the field that actually holds the printed value (e.g. a timestamp or lateness field decoded at the `rx_lost` offset, a direction-overloaded field), with `file:line` of the decode or write site and its unit.
3. Answer the sub-question: decoding error, display error, or real counter.
4. Note, separately, the "latest record wins across all return streams" behaviour of `stats.rs:324` as a second finding if it is a bug in its own right.
5. Write the (b) verdict line in section (b) of `doc/design/reports/archaeology/REPORT-0b.md`, tagged `[measured]` or `[code]`, and a one-paragraph hand-off note for task 003 (which fixes the bug).
## Acceptance criteria
1. Section (b) names the first wrong hop (one of hops 1-5) and quotes the log lines showing the last correct and first wrong value (epic criterion 002-AC-3).
2. Section (b) names the source field of the printed value with at least one `file:line` pointer (epic criterion 002-AC-3).
3. The sub-question is answered with one of the three outcomes.
4. Section (b) ends with exactly one verdict line with a named mechanism (epic criterion 002-AC-1 for (b)).
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
- Fixing the counter or the per-stream separation (task 003).
- Evidence appendix update (002-13).
