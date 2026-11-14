# 002-07 — Loss counter: per-hop logging run
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-01
## Context
Part of epic 002 (POC archaeology, check 0b), deliverable **(b) The 21.7 M counter**, step 1.

Anomaly `[measured]`: during a clean 25 s loopback run (0 datagram loss on audio) the one-line display showed a "lost" value of about **21.7 million**, random-walking. Only about **50 000** packets were sent, so it is not a packet count; its magnitude suggests nanoseconds (21.7 ms). Suspects: a header field overloaded by direction, or a field decoded from the wrong place.

Traced chain `[code]`:
1. per-input counters in the mixer (`mixer.rs:78`) and synth (`synth.rs:88`): `(highest − first + 1) − rx_count`, saturating arithmetic on sequence numbers;
2. written to the echo header field `rx_lost` (`synth.rs:168,195`, `mixer.rs:167,194`);
3. copied into the sender's record (`sender.rs:435`);
4. stored as `lost_fwd` in the statistics; latest record wins across all return streams, no per-stream separation (`stats.rs:324`);
5. printed by the one-line display (`tui.rs:107-114`).

Diagnostic logging must live in the scratch copy of the POC (created in 002-01), never in the frozen tree.
## Work
1. In the scratch copy, add one log line per hop (1 to 5 above), each tagged with hop number, module name, input / return-stream id, sequence number and the raw value (plus raw header bytes at hops 2 and 3). Rate-limit to at most 10 lines per second per hop.
2. Run `hadropoc loopback --duration 25` from the scratch copy, unprivileged, and save the log next to the report.
3. Keep the patch as a diff file next to the report so the run can be reproduced.
4. Paste a representative excerpt (at least 5 consecutive records per hop) in section (b) of `doc/design/reports/archaeology/REPORT-0b.md`, tagged `[measured]`.
## Acceptance criteria
1. The scratch-copy diff touches only logging (no behaviour change); the frozen-tree fingerprint recorded in 002-01 is unchanged.
2. The saved log has entries for all 5 hops, from both synths and the mixer.
3. The display in the instrumented run still shows a "lost" value in the millions (anomaly reproduced), or the report states that it did not reproduce, with the captured display line.
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
- Analysing the log and writing the (b) verdict (002-08).
- Fixing the counter (task 003).
