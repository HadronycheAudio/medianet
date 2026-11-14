# 029-16 — Sinks panel
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-08, 029-10

## Context
- `hadronyche-monitor` (epic 029) panel **Sinks**: A, D, E, required E, certification state.
- **Presentation group:** sinks sharing an audition offset **A** (C-55) and a **required E**. Each sink shows its **E** (99.9th percentile of absolute emission error over one hour, with the maximum reported beside it, C-73), its **D**, and its certification state: *uncertified*, *self-monitored*, *commissioned*.
- Only a commissioned sink has a numeric E; uncertified and self-monitored sinks declare E as **unknown** (C-140). A sink with E unknown is refused with reason RequiredEExceeded from any group that sets a required E and admitted to groups that set none (C-103).
- **D** (C-170): the sink's total plumbing delay from hand-off to emission — driver buffer, resampler and converter. Typical values Linux about 1 ms, macOS a few ms, Windows 6 to 20 ms (C-81); restart spread C-74. The panel shows D as published, with no derivation, and labels it "plumbing delay".
- A and required E come from the Alignment group of the regime stamp (offset, lateness percentiles, A, required E). The POC's playable-latency panel `[code]` may be reused as a layout idea.
- A commissioned sink whose clock is unlocked (029-07; a lapsed holdover counts as Unlocked, C-162) stays commissioned; its alignment claim is suspended and does not count towards the group's required E until the clock is locked again, then resumes without re-commissioning (C-180).
- D, restart spread and certification are commissioning results: taken inside a commissioning window they are valid, recorded with the window they were taken in, and not labelled unqualified (C-178, 029-08).
- E is a one-hour statistic; it must belong to a single regime series (no average across a regime change, 029-06), and is blank before the clock is locked (029-08).
- Open question in the epic, owned by epic 013: where D, E and certification state are published. Working assumption: in the stamp; `NotPublished` otherwise.

## Work
1. `views/sinks.rs` implementing `Panel`: grouped by presentation group (A, required E in the group header), one row per sink with D, E and its maximum, required E, E ≤ required E check, certification state. A commissioned sink with an unlocked clock shows "commissioned, claim suspended (clock unlocked)" and is left out of the group's E ≤ required E check until it locks (C-180).
2. E shown with the window covered so far (e.g. "E (12 min of 60)") when the series is younger than one hour; an unknown E shown as "unknown" (`Shown::Unknown`), never blank or 0.
3. Text capture for documentation.

## Acceptance criteria
1. Two sinks in one group with A = 5 ms, required E = 0.5 ms render under one header; E 0.3 ms passes, E 0.7 ms is flagged; each shows its maximum beside E.
2. Each of the three certification states renders with its exact name.
3. A sink whose series is 12 minutes old shows E with "12 min of 60".
4. A regime change resets the E window to a new series.
5. An uncertified sink and a self-monitored sink each show E "unknown" even if the stamp carries a self-measured figure (C-140); a commissioned sink shows its numeric E. A sink with E unknown in a group with a required E it appears with refusal reason RequiredEExceeded, in a group without one it appears admitted.
6. A commissioned sink whose clock goes Unlocked still shows certification state "commissioned", marked claim suspended, and does not pass or fail the group's required-E check; when its clock locks again the mark clears with no change of certification state (C-180).
7. A sink whose D and certification were stamped during a commissioning window shows them without the `unqualified` label (C-178).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Computing E (runtime); certification (commissioning epics).
