# Status

Last updated: 2026-11-14, by the orchestrator, at swarm start.

## Phase and epic summary

| Phase | Epic | Topic | Done | In flight | Ready for hardware | Blocked | Total |
|---|---|---|---|---|---|---|---|
| P0 | 001 | Receive stamp probes | 0 | 0 | 0 | 0 | 17 |
| P0 | 002 | POC archaeology | 0 | 0 | 0 | 0 | 14 |
| P0 | 003 | POC smoke fixes | 0 | 0 | 0 | 0 | 10 |
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
