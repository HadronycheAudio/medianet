# Status

Last updated: 2026-11-14, by the orchestrator, at swarm start.

## Phase and epic summary

| Phase | Epic | Topic | Done | In flight | Ready for hardware | Blocked | Total |
|---|---|---|---|---|---|---|---|
| P0 | 001 | Receive stamp probes | 0 | 0 | 0 | 0 | 17 |
| P0 | 002 | POC archaeology | 1 | 2 | 0 | 0 | 14 |
| P0 | 003 | POC smoke fixes | 1 | 2 | 0 | 0 | 10 |
| P1 | 004–007b | Specification epics | 0 | 0 | 0 | 0 | 116 |

Phases P2 to P7: not started (dependency order; P1 drafting runs alongside P0).

## Now in flight

(none yet — first wave being scheduled as the swarm starts)

## Standing notes

- The repository was initialised by the design owner with one commit (`init`):
  the design set, the kickoff directory, and a placeholder root crate
  (`medianet`, a hello `src/main.rs`). The placeholder is not one of the
  design's crates (C-133, roadmap deliverables); the first ticket that
  creates a real crate turns the root into a workspace and removes the
  placeholder.
- `main` currently carries no product code; the per-change suite is
  `python3 doc/kickoff/check-design.py doc/design` plus, once crates exist,
  `cargo test --workspace` (and `cargo build --target <no_std target>` at the
  P3 gate).
- Hard rule 4: the `origin` remote exists but is never pushed to.

## Blockers and findings for the design owner

None so far.

## Tooling note (for the design owner)

`doc/kickoff/check-design.py` rejected any root entry in `doc/design/` beyond
the original set, but the design itself mandates `doc/design/spec/` and
`doc/design/reports/` as deliverable locations (C-130). The root allowlist was
extended by exactly those two names (no other check weakened); the checker
still prints OK on the unchanged design set.

## Recorded rule-2 differences (ticket vs 02-constants.md)

- 006-01: the ticket's constants preview paraphrases 36 of 47 rows and says
  "the others decided or derived", but 02-constants.md marks C-02, C-30,
  C-33, C-67 as `inherited` (C-65 `derived`). The document's table follows
  02-constants.md (which wins); the ticket preview is the bug.

## Lead decisions (recorded for the design owner)

- 003-02: the ticket says "Do not commit them yet" for the measurement notes.
  Multi-agent hand-off (003-05, 003-08 need them) makes an uncommitted file
  unusable, so the notes live at `swarm/notes/003-02.md` as committed WORKING
  NOTES — not a design or baseline document; no figure enters doc/design/**
  from this ticket. 003-08 writes BASELINE.md in the POC repository.
- Scratch copy home (002-01): /home/dorian/workspace/hadronyche/poc-scratch.

## Test disputes (kickoff section 6, step 3)

- 001-01 (2026-11-14): coder stopped 31/32 — test
  `decoder_detects_gaps_through_decoded_payloads` asserted an unobservable
  end-of-stream hole (skipped seq 99, expecting lost == 3; only 2 holes
  observable). Ruling sent to the tester; fix expected: encode two observable
  holes, assert {17, 42}.
