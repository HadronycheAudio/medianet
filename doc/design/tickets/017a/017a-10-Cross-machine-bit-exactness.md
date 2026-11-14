# 017a-10 — Cross-machine bit-exactness of the ported modules
**Epic:** 017a-Reference-modules · **Effort:** M · **Depends on:** 017a-09, epic 002, epic 013

## Context
- Definition-of-done item: **the ported modules interoperate bit-exactly with their own replicas on two different machines (float and fixed)**.
- Exact tier `[decision]`: deterministic modules produce identical bits; the sender's replica runs the same code and compares.
- POC float rendering uses `sine()` built from additions and multiplications only "so that it gives the same bits on every machine" `[code]`; its fixed-point table used libm `f64::sin` per machine (`music.rs:311`), fixed by shipping a constant (017a-01). No libm in any reference module's signal path `[decision]`.
- Do not cite the POC as the exact-tier baseline until epic 002 is done.
- L0 itself is single-process; this check needs two machines, so it runs outside the L0 gate (settled in the epic). It records each machine's regime stamp (epic 013) with the results and does not need the L0 runner of epic 017b.
- **Where results go (settled, C-130):** the result report in `doc/design/reports/`, the evidence in `doc/design/06-evidence-appendix.md`.
- **Machine pair (settled, C-151):** x86-64 and 64-bit ARM.
- **Report presentation (settled, C-206):** the report's file name within `doc/design/reports/` and its layout are decided by this ticket and recorded in the part's documentation.

## Work
1. Run the sender on machine 1 and echo + mixer on machine 2, in float and in fixed mode, over an IPv4 link (C-100); compare echo/mixer output with the sender's replica. The mixer feeds no presentation group, so it is given an explicit alignment budget in configuration (C-142); without one its inputs would be refused with NoSkewBudget (C-185).
2. Alternatively, render a fixed 60 s reference programme on both machines and compare output hashes.
3. Record both machines' stamps and results; write the report to `doc/design/reports/` and the `[measured]` evidence to the evidence appendix.

## Acceptance criteria
1. Float mode: 0 exact-tier mismatches over ≥ 60 s between the two machines.
2. Fixed mode: 0 exact-tier mismatches over ≥ 60 s.
3. One machine is x86-64 and the other 64-bit ARM (C-151), as stated in their stamps.
4. Results are recorded as `[measured]` evidence with the record files, in the locations of C-130.

## Definition of done
- [ ] The ported modules interoperate bit-exactly with their own replicas on two different machines (float and fixed)
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Multi-host network harnesses (epics 019, 020, 027).
