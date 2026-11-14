# 029-21 — End-to-end honesty, regime and gap-class tests
**Epic:** 029-Monitor · **Effort:** M · **Depends on:** 029-13, 029-17, 029-18

## Context
Epic 029 acceptance criteria verified here against a real test medianet with fault injection (the Injection group of the regime stamp always records active injection, including the test-only hooks of C-120 and C-145):
- **029-AC-2 Honest blanks.** One-way figures are blank before lock and show the bound after; a check family with zero samples is displayed as *failed*; an injected run is visibly labelled as injected, and so is a run inside a commissioning window (C-107). (Contributes to the global criteria AC-9 and AC-31.)
- **029-AC-3 Regime change.** Changing a module's regime mid-run (for example a path-MTU fallback to **1 280 bytes**, C-06) starts a new series; no average spans the change. A new session number (C-19) does the same for its connection.
- **029-AC-4 Per-stream separation.** With two return streams reporting different loss, each is shown separately; no last-writer-wins aggregate exists (the POC bug `stats.rs:324` `[code]`).
- **029-AC-5 Gap classes.** Injected delay beyond the deadline shows as *late* (counted missing at the deadline, then reclassified late when it arrives within the ring span, C-108); injected drop as *missing*; an upstream gap as *marked*; after **2 s** without content or a marked gap (C-39, C-111), *dead*. Interrupted after 8 consecutive missed slices (C-38, C-102). Each appears no later than the next stamp refresh (10 s, C-18).
- The per-connection summaries publish received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), so every assertion reads a published counter.

## Work
1. Scenario tests (each its own test case) driving the test medianet and asserting on `render_to_text` captures and headless lines:
   - 029-AC-2: start before clock lock; assert one-way blank; wait for lock; assert value ± bound. Configure a check family with no samples; assert FAILED. Enable an injection; assert INJECTED in header, Regime panel banner and headless lines. Open a commissioning window; assert COMMISSIONING likewise.
   - 029-AC-3: force a path-MTU fallback to 1 280 bytes on one module; assert new-series marker and that post-change figures equal those computed from post-change stamps only. Restart a source; assert the new session opens a new series.
   - 029-AC-4: two return streams, inject drop on one only; assert two rows with different missing counts.
   - 029-AC-5: inject delay beyond the deadline but within the ring span → late (and missing back to its prior value); inject drop → missing; gap upstream of a relay → marked at the downstream consumer; stop the producer for more than 2 s → dead; force a stream onto another timeline epoch with the C-120 hook → other-epoch rises, missing and the interrupted count do not (C-135, C-102). Each assertion waits at most one stamp refresh (10 s) after the event.

## Acceptance criteria
1. Each of 029-AC-2, 029-AC-3, 029-AC-4, 029-AC-5 has a passing automated test case named after it.
2. Tests run in CI unprivileged (fault injection through the runtime's injection facility and the C-120 hooks, not through privileged network tools).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Unit-level checks already in 029-06, 029-08, 029-09; replay (029-19); scale (029-22).
