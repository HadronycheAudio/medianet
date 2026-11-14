# 002-05 — Inert families: baseline run and transport trace
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-01, 002-03
## Context
Part of epic 002 (POC archaeology, check 0b), deliverable **(a) Inert check families**, steps 1 and 2. Findings go to section (a) of `doc/design/reports/archaeology/REPORT-0b.md`.

Anomaly `[measured]`: in a 25-second unprivileged loopback run on kernel 6.12, the music and transport check families reported "0 of 0 windows" for the whole run and the display said "playhead: no transport state yet", while the audio check was green (2345/2345 windows exact, 0 datagram loss). A run where half the verification is inert looked identical to a good run. Cause unknown.

`hadropoc loopback` runs the whole graph (sender, synth1, synth2, mixer) in one process (`main.rs:483-500`) `[code]`.

Points to trace (from the epic):
- who is the transport reference in loopback — the sender defaults to reference per the `main.rs` help text for `--transport`;
- whether `ctx.is_reference()` becomes true (`sender.rs:229`);
- the listen-before-claim delay (1.5 s, inherited by the design as C-32; 002-03 confirms the POC value);
- whether transport and music datagrams are routed: the `events` output connections in loopback.

Task 003 will make a zero-sample family fail the run (system criterion **AC-9**: a check family with zero samples fails the run); it needs this explanation.
## Work
1. In the frozen build (or the scratch copy from 002-01, unmodified), run `hadropoc loopback --duration 25 --csv out.csv` unprivileged and paste the final summary in section (a) as captured run output `[measured]`. Keep `out.csv` alongside the report.
2. Using the reading notes of 002-03 and the source, trace the four points above and write each as a short paragraph with `file:line` pointers `[code]`.
3. If a point can only be settled at runtime, add temporary logging in the **scratch copy only** and record the log line `[measured]`.
4. Write an interim conclusion "why no transport state reaches the synths".
## Acceptance criteria
1. The captured summary of a 25 s run is in the report and shows the transport and music counts (expected "0 of 0" if the anomaly reproduces; if it does not reproduce, the report says so with the output).
2. Each of the 4 trace points has a definite answer with at least one `file:line` pointer or a log line.
3. The interim conclusion names the step in the chain where transport state stops.
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] No new open item without an owner and a falsifier
## Out of scope
- The music family trace, the minimal condition and the final (a) verdict (002-06).
- Making zero-sample families fail the run (task 003).
