## Ethernet bit-error figure and loss floor (ticket 001-11)

### Bit-error-ratio requirement of IEEE 802.3

| PHY | Citation | Tag |
|---|---|---|
| 10GBASE-T | IEEE Std 802.3-2008 Section Four, Clause 55.5.4.1 "Receiver differential input signals": bit error ratio 1e-12 (restated as objective k of Clause 55.1.1, "Support a BER of less than or equal to 10^-12") | [standard] |
| 1000BASE-T | IEEE Std 802.3ab-1999 (now Clause 40 of IEEE Std 802.3), Clause 40.1.1 objectives: bit error ratio 1e-10, text not read | [memory] |

For 10GBASE-T, IEEE Std 802.3-2008 Clause 55.5.4.1 requires a bit error ratio 1e-12 (verbatim: "are received with a BER less than 10–12 and sent to the PCS after link reset completion. This specification shall be satisfied by a frame error ratio less than 9.6 × 10–9 for 800 octet frames") `[standard]`. The text was read in an IEEE-published copy: "IEEE Std 802.3-2008, Section 4" (Clause 44 through Clause 55, third printing 22 June 2010), https://cdn.shopify.com/s/files/1/0790/5963/files/IEEE_802.3-2008_Section_4.pdf, the page footer reading "Copyright © 2008 IEEE" (page 514). The same edition lists the requirement as PICS item PME26, "BER over a 55.7 compliant link", Clause 55.5.4.1 `[standard]`. Later editions were not read; the figure is quoted for the 2008 edition only.

For 1000BASE-T, IEEE Std 802.3ab-1999, Clause 40.1.1, is recalled as requiring a bit error ratio 1e-10 `[memory]`. This figure is not verified; see the fallback paragraph below.

### Fallback statement (C-132)

The clause text of 1000BASE-T (Clause 40 of IEEE Std 802.3) could not be consulted in this session, so its figure stays tagged `[memory]` and is not `[standard]`. What was tried: web searches for the Clause 40 objectives and receiver text returned only secondary material (a test-suite document from a university interoperability laboratory, IEEE working-group comment files, presentations and a vendor article), none of them the clause; the only complete IEEE-published copy found was Section Four of IEEE Std 802.3-2008 (Clauses 44 to 55), which does not contain Clause 40; the Section Three copy of 802.3-2008 (Clauses 34 to 43) returned HTTP 404 on the same host, a third-party page held only Clauses 34 and 35, and the IEEE 802.3 machine-readable extracts hold tables and code, not the objectives. The secondary material is consistent with 1e-10 for 1000BASE-T but is not a check `[memory]`. The checker is ticket 001-13 (REPORT-0a assembly), which reads Clause 40 from an IEEE copy (IEEE GET program or IEEE Xplore) before it includes this section; L1 task 019 is the second checker, through the measured clean-network loss.

### Which figure governs

For the medianet's GbE default links, 1000BASE-T governs, because gigabit copper default links are 1000BASE-T and the bit error ratio of the PHY in use sets the floor `[decision]`. The 10 Gb/s wide-node links of C-25 run 10GBASE-T, so the 10GBASE-T figure matters there and is carried in the table as the second row `[decision]`.

A standard's bit error ratio is a worst-case compliance bound on a compliant link, not the error rate of a typical clean link, so a measured clean-network loss well below the floor is expected and is not a contradiction `[decision]`; L1 (task 019) measures it `[memory]`.

### Loss-floor recomputation

Inputs: the default packet is 1 320 bytes on the wire (C-20), preamble, inter-frame gap and FCS included, which is 10 560 bits `[decision]`. The per-stream rate is 2 000 packets/s. The soak runs 64 000 packets/s for 86 400 s (24 h), about 5.5e9 packets (C-118). Formulas: p = BER × 10 560; mean time between losses = 1 / (p × 2 000) s; soak count = p × 64 000 × 86 400; reopen count = 10 × soak count (C-179) `[decision]`.

| BER | Per-packet loss probability p | Mean hours between losses per stream per link | C-118 soak losses | C-179 reopen count | Tag |
|---|---|---|---|---|---|
| 1e-10 | 1.1e-6 | 0.13 | 5.8e3 | 5.8e4 | [memory] |
| 1e-12 | 1.1e-8 | 13 | 58 | 580 | [standard] |

The 1e-10 row is the governing row for the GbE default links (1000BASE-T) and carries `[memory]` because its figure was not read in the standard; the 1e-12 row is the 10GBASE-T figure and carries `[standard]`. At 1e-10 the mean time between losses is about 473 s, 7.9 min, per stream per link `[memory]`.

### Verdict on C-90

**verdict:** pending, because the governing 1000BASE-T figure of 1e-10 is still `[memory]` and could not be consulted in the standard (checker 001-13); if the clause confirms 1e-10, C-90 is corrected to p 1.1e-6, a C-118 soak prediction of 5.8e3 and a C-179 reopen count of 5.8e4, and the 1e-12 value of C-90 would then hold only for 10GBASE-T `[memory]`.

### Open sub-question

The open sub-question "Is the loss floor C-90 computed from the right bit-error figure?" is partly answered and handed to L1 (task 019) and to ticket 001-13: answered for 10GBASE-T (1e-12, read in the standard), not answered for 1000BASE-T. Owner: ticket 001-13 for the clause check, then L1 task 019. Falsifier: the standard cites a different figure for 1000BASE-T, or the clean-network loss measured at L1 is well above the floor `[memory]`.
