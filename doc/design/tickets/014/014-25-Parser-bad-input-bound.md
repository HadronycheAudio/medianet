# 014-25 — Parser bad-input bound (C-164)
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-04, 014-08, 014-12, 014-13
## Context
C-164 bounds the cost of processing one bad input: linear in packet length and at most 10 µs per packet. The value is a proposal pending L1. This epic owns four parsers that see untrusted bytes: the receiver report parser (014-04), the probe request and reply parsers behind `answer_probe` and `close_probe` (014-08), the reverse-port classifier (014-12) and the lineage extension parser (014-13). The reverse port must also count every rejected message (epic AC-13), so a flood of bad input must not cost more than the bound per packet. A valid report is at most 1 252 bytes (C-17, C-100). Fuzzing (C-95) shows the parsers do not panic; it does not show their cost.

Settled locations: harness and benchmark binaries live in the `hadronyche-tools` crate (C-133); measured results go to `doc/design/reports/` (C-130). A run whose numbers enter the constants table runs in strict mode (C-147); this ticket's figure goes to a report and stays a proposal until L1.

## Work
1. For each of the four parsers, add a deterministic test (std, test-only) that counts the bytes examined while parsing bad inputs of length 0 to 1 500 bytes: random bytes, every truncation of each valid vector, a member count of 33 with a long tail, and a valid header followed by garbage. The count must not exceed a fixed multiple of the input length plus a constant.
2. Check by inspection, recorded in the test module, that no parser loop runs over anything other than the input bytes or a member count already bounded by 32 (C-08), and that a rejection exits at the first failed check.
3. Add a micro-benchmark in `hadronyche-tools` (C-133) that measures the worst per-packet time of each parser over the bad-input corpus of the fuzz targets (014-05, 014-10, 014-12, 014-14) on the Linux reference machine, and record the figures with `[measured]` in `doc/design/reports/` (C-130).

## Acceptance criteria
1. For every parser, bytes examined on every bad input are at most k × length + c for constants k and c stated in the test (linear in packet length, C-164).
2. The benchmark's worst per-packet time for each parser is at most 10 µs on the reference machine, and the result is recorded with `[measured]` in `doc/design/reports/`; a figure above 10 µs is reported as a falsifier of C-164 for L1, not hidden.
3. The tests run unprivileged in per-change CI.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (fuzz targets delivered in 014-05, 014-10, 014-12, 014-14)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Confirming the 10 µs figure for the constants table (L1, task 019).
- Parsers of other epics (media, control and stamp messages).
- The socket receive loop and its drain bound (C-16; epic 015).
