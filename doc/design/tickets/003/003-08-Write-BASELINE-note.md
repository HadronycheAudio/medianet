# 003-08 — Write BASELINE note
**Epic:** 003-POC-smoke-fixes · **Effort:** XS · **Depends on:** 003-02, 003-05, 003-06, 003-07

## Context
- Epic 003 produces a frozen, citable baseline of the proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`) for epic 007 (conformance vectors) and epic 017 (reference modules and L0 harness).
- Two annotated tags: `poc-untouched` (first commit, untouched tree plus `.gitignore`) and `poc-baseline` (second commit, three fixes applied). After `poc-baseline` the POC is not modified again.
- Three fixes: (1) one process-owned writer per metrics path (rule: a file has one writer, owned by the process, never by a module); (2) clock-sync wait bounded by the next request deadline with a 100 ms ceiling instead of a fixed 5 ms receive timeout; (3) headless `--duration` runs exit non-zero and name any in-scope check family that analysed zero windows.
- Sync rates: 50/s is the default (C-36) and inside the accepted range 10 to 200/s (C-112); 1000/s is outside that range and is recorded only for comparison with the pre-fix figures (P-02).
- The baseline is a behavioural reference. Its wire (P-01) is not the runtime's wire (C-01, C-04, C-100); the note says so, so that epics 007 and 017 do not take it as a wire reference.

## Work
Create `BASELINE.md` in the POC root containing:
1. The two tags, `poc-untouched` and `poc-baseline`, and what each contains.
2. The three fixes, one short paragraph each, with the files touched.
3. A table of achieved sync rate (exchanges/s) before (003-02) and after (003-05) at `--sync-hz 50` and `--sync-hz 1000`, each tagged `[measured]`, with machine identity, kernel `CONFIG_HZ`, method, and the statement that the runs were unprivileged and not in strict mode (the POC has none; C-127, C-147).
4. Whether the clock still locks (P-04) within 25 s after the fix (answer from 003-05), and a sentence that 1000/s lies outside the accepted range C-112.
5. The in-scope check families; if epic 002 found that music and/or transport cannot produce samples in loopback without code changes, a statement that the baseline covers the audio family only, with the others listed as **known-inert**.
6. How to run `tools/smoke.sh`, and a statement that it asserts nothing about sync rate, latency or percentiles.
7. A sentence that the POC wire (P-01) differs from the runtime wire and the baseline is not a wire reference.
8. The wording of the zero-sample line printed by 003-06; presentation details of this epic's tooling are recorded here (C-206).

## Acceptance criteria
1. `BASELINE.md` exists at the POC root and names both tags exactly.
2. It contains 4 sync-rate figures (50 and 1000, before and after), each tagged `[measured]`.
3. Every factual claim in it carries an evidence tag (`[code]`, `[measured]` or `[decision]`).
4. Known-inert families, if any, are listed by name.

## Definition of done
- [ ] Both tags exist and `BASELINE.md` records before/after figures
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Committing and tagging (003-09).
- Updating the design set (003-10).
