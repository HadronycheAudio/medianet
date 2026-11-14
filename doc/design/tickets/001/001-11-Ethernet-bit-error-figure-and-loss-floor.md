# 001-11 — Ethernet bit-error figure and loss floor
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** none

## Context
The design publishes an expected loss floor (C-90): at a bit error rate of 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire), one loss per stream per link about every 13 hours at 2 000 packets/s `[memory]`. A looser 1e-10 figure for gigabit copper was also recalled `[memory]`. Measured loss above the floor raises a cable-health alarm whose threshold L1 sets from the measured clean-network loss (C-91), so the figure must be sourced.

Settled since the ticket split:
- The packet size is the default media packet, about 1 320 bytes on the wire including all Ethernet overhead (C-20); the earlier 1 500-byte figure is gone from C-90. "Wire packet" means the C-20 size, preamble, inter-frame gap and FCS included.
- The same figure drives the 24-hour loss soak: 64 000 packets/s, about 5.5e9 packets, about 58 losses predicted in aggregate over all streams (C-118, C-92).
- The same figure sets the loss level that reopens the redundancy decision: clean-medianet loss above 10 × the floor, about 580 losses in the 24-hour soak; between 1 × and 10 × it is recorded and investigated as a cabling or host problem and does not reopen it (C-179).

- The evidence tag for a figure cited from a published standard is `[standard]`, given with the clause; it replaces `[memory]` once the citation is checked (C-132). The cited BER figures and the loss floor computed from them are tagged `[standard]`.

Open sub-question owned by this ticket, then L1 (task 019): "Is the loss floor C-90 computed from the right bit-error figure?" Falsifier: the standard cites a different figure; clean-network loss well above the floor.

## Work
- Find the bit-error-ratio requirement in IEEE 802.3 for 1000BASE-T and for 10GBASE-T; cite standard edition, clause and subclause, tagged `[standard]` (C-132).
- Recompute the loss floor for a 1 320-byte wire packet (C-20) at 2 000 packets/s:
  - per-packet loss probability p ≈ BER × 1 320 × 8 = BER × 10 560;
  - losses per second = p × 2 000; mean time between losses = 1 / (p × 2 000).
  - For reference, at BER 1e-12: p ≈ 1.06e-8, ≈ 47 300 s ≈ 13 h; at 1e-10: ≈ 473 s ≈ 7.9 min.
- Recompute the C-118 soak prediction: p × 64 000 × 86 400 (about 58 at 1e-12), and the C-179 reopen count, 10 × that prediction (about 580 at 1e-12).
- Write the result as a section to be included in `doc/design/reports/REPORT-0a.md` (draft file `doc/design/reports/REPORT-0a-ber.md`, C-130), stating C-90 confirmed or a corrected value proposed, and, if corrected, the matching C-118 count and C-179 reopen count.

## Acceptance criteria
1. One cited BER figure each for 1000BASE-T and 10GBASE-T, with standard, edition and clause, tagged `[standard]` (C-132).
2. Recomputed loss floor for 1 320 bytes at 2 000 packets/s given as per-packet probability and as mean hours between losses per stream per link, to 2 significant figures.
3. Recomputed C-118 expected loss count for the 24-hour soak, and the C-179 reopen count (10 × it).
4. A one-line verdict: C-90 confirmed, or corrected value proposed with the reason (epic 001-AC5).

## Definition of done
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Tests green in CI, unprivileged", "Every parser touched is fuzzed" and "Probe sources committed" — no code; "Regime stamp fields touched" — none; "Design documents updated" — the constants table is edited in 001-14 from this output; "The frozen POC is not modified" — not touched; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Measuring clean-network loss and setting the alarm threshold C-91 (L1, task 019).
- Editing the constants table (001-14).
