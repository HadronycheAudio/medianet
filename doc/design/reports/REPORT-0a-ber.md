## Ethernet bit-error figure and loss floor (ticket 001-11)

### Bit-error-ratio requirement of IEEE 802.3

| PHY | Citation | Tag |
|---|---|---|
| 10GBASE-T | IEEE Std 802.3-2008 Section Four, Clause 55.5.4.1 "Receiver differential input signals": bit error ratio 1e-12 (restated as objective k of Clause 55.1.1, "Support a BER of less than or equal to 10^-12") | [standard] |
| 1000BASE-T | IEEE 802.3 working-group text, not the published clause: bit error ratio 1e-10 at IEEE 802.3 Clause 40.1 (edition IEEE Std 802.3-2005, Clause 40; draft P802.3ab Draft 4.0), given as an objective, with the specification in 40.6.1.3.4; sources (a) and (b) below | [standard] |

For 10GBASE-T, IEEE Std 802.3-2008 Clause 55.5.4.1 requires a bit error ratio 1e-12 (verbatim: "are received with a BER less than 10–12 and sent to the PCS after link reset completion. This specification shall be satisfied by a frame error ratio less than 9.6 × 10–9 for 800 octet frames") `[standard]`. The text was read in an IEEE-published copy: "IEEE Std 802.3-2008, Section 4" (Clause 44 through Clause 55, third printing 22 June 2010), https://cdn.shopify.com/s/files/1/0790/5963/files/IEEE_802.3-2008_Section_4.pdf, the page footer reading "Copyright © 2008 IEEE" (page 514). The same edition lists the requirement as PICS item PME26, "BER over a 55.7 compliant link", Clause 55.5.4.1 `[standard]`. Later editions were not read; the figure is quoted for the 2008 edition only.

For 1000BASE-T the bit error ratio is 1e-10 `[standard]`, taken from IEEE 802.3 working-group text, named here as what it is (a draft, a comment record and a test suite), never as the published edition. The published clause text was not read. The acceptance of working-group text for the `[standard]` tag is the design owner's decision of 2026-11-16 `[decision]`. The sources:

- (a) IEEE P802.3ab Draft 4.0 working-group comment database, https://grouper.ieee.org/groups/802/3/ab/comments/abWGCOMM.pdf `[standard]`. Comment #155 on Cl 40 SC 40.1 says "The Bit Error Rate of 10^-10 is not adequate" and asks for 10^-12; the task force's response (reject) says "The BER of 10^-10 is what is in the project approval documents". The response to comment #73 says "The BER figure in line 42 is an objective, not a specification. Specification is provided in 40.6.1.3.4."
- (b) UNH-IOL Gigabit Ethernet Consortium, Clause 40 PMA test suite v2.5, Test 40.2.1, https://www.iol.unh.edu/sites/default/files/testsuites/ethernet/CL40_PMA/PMA_Test_Suite_v2.5.pdf `[standard]`: "The operation of the 1000BASE-T PMA sublayer is defined in [1], to operate with a bit error rate of 10-10, as specified in [2]", with [1] = IEEE Std 802.3-2005, clause 40, and [2] = clause 40.4.2.3. This is a secondary source (a test suite) citing the standard.

The 10GBASE-T figure above was read in an IEEE-published copy and stays as cited.

### Provenance of the 1000BASE-T figure

The 1000BASE-T figure rests on a draft, a comment record and a test suite. If the published Clause 40 of IEEE Std 802.3 gives a different figure, the figure and the rows below change `[decision]`. Owner of that check: ticket 001-13 when it reads Clause 40 from an IEEE copy, then L1 task 019 through the measured clean-network loss.

### Which figure governs

For the medianet's GbE default links, 1000BASE-T governs, because gigabit copper default links are 1000BASE-T and the bit error ratio of the PHY in use sets the floor `[decision]`. The 10 Gb/s wide-node links of C-25 run 10GBASE-T, so the 10GBASE-T figure matters there and is carried in the table as the second row `[decision]`.

A standard's bit error ratio is a worst-case compliance bound on a compliant link, not the error rate of a typical clean link, so a measured clean-network loss well below the floor is expected and is not a contradiction `[decision]`; L1 (task 019) measures it `[decision]`.

### Loss-floor recomputation

Inputs: the default packet is 1 320 bytes on the wire (C-20), preamble, inter-frame gap and FCS included, which is 10 560 bits `[decision]`. The per-stream rate is 2 000 packets/s. The soak runs 64 000 packets/s for 86 400 s (24 h), about 5.5e9 packets (C-118). Formulas: p = BER × 10 560; mean time between losses = 1 / (p × 2 000) s; soak count = p × 64 000 × 86 400; reopen count = 10 × soak count (C-179) `[decision]`.

| BER | Per-packet loss probability p | Mean hours between losses per stream per link | C-118 soak losses | C-179 reopen count | Tag |
|---|---|---|---|---|---|
| 1e-10 | 1.1e-6 | 0.13 | 5.8e3 | 5.8e4 | [standard] |
| 1e-12 | 1.1e-8 | 13 | 58 | 580 | [standard] |

The 1e-10 row is the governing row for the GbE default links (1000BASE-T); the 1e-12 row is the 10GBASE-T figure. At 1e-10 the mean time between losses is about 473 s, 7.9 min, per stream per link `[standard]`.

Arithmetic, recomputed `[decision]`. Wire packet 1 320 bytes x 8 = 10 560 bits. At BER 1e-10: p = 1e-10 x 10 560 = 1.056e-6, about 1.1e-6; losses per second = 1.056e-6 x 2 000 = 2.112e-3, so one loss every 473.5 s = 0.1315 h, about 0.13 h; soak = 1.056e-6 x 64 000 x 86 400 = 5 839, about 5.8e3 (soak packets 64 000 x 86 400 = 5.53e9, about 5.5e9); reopen = 10 x 5 839 = 58 390, about 5.8e4. At BER 1e-12: p = 1.056e-8, about 1.1e-8; one loss every 47 348 s = 13.15 h, about 13 h; soak = 58.4, about 58; reopen = 584, about 580.

### Verdict on C-90

**verdict:** C-90 corrected for the GbE default links, because the governing 1000BASE-T figure is a bit error ratio of 1e-10, not 1e-12: p 1.1e-6 per default packet, one loss per stream per link about every 0.13 h (473 s), a C-118 soak prediction of 5.8e3 and a C-179 reopen count of 5.8e4; the 1e-12 values of C-90 (p 1.1e-8, 13 h, 58, 580) hold only for 10GBASE-T `[standard]`

### Open sub-question

The open sub-question "Is the loss floor C-90 computed from the right bit-error figure?" is answered: C-90 used 1e-12, the right figure for 10GBASE-T, but the 1000BASE-T figure is 1e-10 (working-group text, see above), so the C-90 value is corrected; the clean-network measurement is handed to L1 (task 019). Owner of the check against the published Clause 40: ticket 001-13. Falsifier: the published Clause 40 cites a figure other than 1e-10 for 1000BASE-T, or the clean-network loss measured at L1 is well above the corrected floor `[decision]`.

### Proposed change to the constants table (draft for the design owner; not applied, 001-14 edits the table)

- C-90, current text: "at a bit error rate of 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire): one loss per stream per link about every 13 hours at 2 000 packets/s" (tagged as recalled in the table) `[decision]`.
- C-90, proposed text: "for 1000BASE-T gigabit copper default links, at a bit error rate of 1e-10, about 1.1e-6 per default packet (about 1 320 bytes on the wire): one loss per stream per link about every 473 s (0.13 h) at 2 000 packets/s `[standard]`; for 10GBASE-T, 1e-12, about 1.1e-8, about every 13 hours `[standard]`".
- C-118, current: "the floor predicts about 58 losses"; proposed: "about 5.8e3 losses at the 1000BASE-T floor (58 at 1e-12)" `[standard]`.
- C-179, current: "about 580 losses in the 24-hour soak"; proposed: "about 5.8e4 losses in the 24-hour soak (580 at 1e-12)" `[standard]`.
- Reason: the 1000BASE-T figure is 1e-10 (sources (a) and (b)); the table's 1e-12 holds for 10GBASE-T. Evidence tag `[standard]` rests on the design owner's decision of 2026-11-16 to accept working-group text `[decision]`.
