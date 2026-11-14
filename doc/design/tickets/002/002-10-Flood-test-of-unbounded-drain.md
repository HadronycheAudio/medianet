# 002-10 — Flood test of the unbounded drain
**Epic:** 002-POC-archaeology · **Effort:** M · **Depends on:** 002-01, 002-04
## Context
Part of epic 002 (POC archaeology, check 0b), deliverable **(e) Run confirmations**, item 1.

Claim read from code, never run `[code]`: the POC data thread empties every input socket (`while let Some(..) = net::recv_now(..)`, `module.rs:1669`) before it looks at its own deadline, so **one flooding input should starve the ticker**.

The POC streams 96 kHz audio in datagrams; the graph in `hadropoc loopback` is sender, synth1, synth2, mixer (`main.rs:483-500`) `[code]`. The sender-side metric "TX pacing lateness" is located by 002-04. The POC default feed is 48 channels, about 9.6 KB per datagram, 7 IP fragments at MTU 1500, about 150 Mbit/s (P-01) `[code]`; the nominal rate of one mixer input is not known and is measured here.

Settled test procedure (epic 002, deliverable (e) 1): measure the nominal rate first; flood with well-formed datagrams from a standalone sender or a scratch-copy option; baseline and flooded runs of at least 25 s each; the mixer metric is ticker lateness, deadline against actual tick time, logged in the scratch copy if the POC does not expose it.

The runtime does not inherit this loop: it bounds the drain per input per cycle to 2 × the admitted packets per slice (C-16), where a cycle is one slice period (C-156). This ticket only confirms or refutes the POC behaviour.

Diagnostic logging lives in the scratch copy of the POC (002-01). A standalone flood sender is a throwaway probe in `probes/` of the design set's repository (C-133). Nothing goes into the frozen tree (C-130); captured outputs go to `doc/design/reports/archaeology/`.
## Work
1. Measure the nominal datagram rate of one mixer input in a normal run (datagrams per second on that input) `[measured]`.
2. Build a flood source: either a small standalone UDP sender in `probes/` (C-133) or a scratch-copy option, that sends well-formed datagrams to that one mixer input at **10×** the measured nominal rate.
3. Run a baseline (no flood) and a flooded run, each at least 25 s, unprivileged, and record for both:
   - sender-side "TX pacing lateness" (distribution: median, p99, max);
   - the mixer's ticker lateness (add a scratch-copy log of deadline vs. actual tick time if the POC does not expose it).
4. Write the result in section (e) of `doc/design/reports/archaeology/REPORT-0b.md`: the two distributions side by side, the achieved flood rate, and "starvation: confirmed" or "refuted", with the verdict line tagged `[measured]`.
## Acceptance criteria
1. The achieved flood rate is within 10 % of 10× the measured nominal rate, both numbers reported.
2. Median, p99 and max of both latenesses are reported for both runs, with units.
3. The report states confirmed or refuted, and the verdict line names the mechanism with `module.rs:1669` or another `file:line` pointer (epic criteria 002-AC-1 and 002-AC-6 for (e) 1).
4. The frozen-tree fingerprint from 002-01 is unchanged.
5. If 10× the nominal rate cannot be reached on the test host (for example because of the P-01 datagram size), the report gives the highest rate achieved and states that the (e) 1 verdict holds only up to it.
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
- Fixing the drain loop (the runtime bounds it, C-16 and C-156; not this epic).
- Servo step (002-11) and sync rate (002-12, 002-14) measurements.
