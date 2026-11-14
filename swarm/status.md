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

| Ticket | Team | Phase | Tester | Coder | Reviewer |
|---|---|---|---|---|---|
| 001-01 | Baseline | red | deepseek-v4.1-flash-1017 | — | — |
| 002-01 | Baseline | red | deepseek-v4.1-flash-1017 | — | — |
| 003-01 | Baseline | red | deepseek-v4.1-flash-1017 | — | — |
| 004-01 | Specification | red | deepseek-v4.1-flash-1017 | — | — |
| 006-01 | Specification | red | deepseek-v4.1-flash-1017 | — | — |

(none yet — first wave being scheduled as the swarm starts)

## Standing notes

- The repository was initialised by the design owner with one commit (`init`):
  the design set, the kickoff directory, and a placeholder root crate
  (`medianet`, a hello `src/main.rs`). The placeholder is not one of the
  design's crates (C-133, roadmap deliverables); the first ticket that
  creates a real crate turns the root into a workspace and removes the
  placeholder (007a-01 creates `hadronyche-vectors` at the root, C-192).
- `main` currently carries no product code; the per-change suite is
  `scripts/per-change-suite.sh` = `python3 doc/kickoff/check-design.py
  doc/design` + `cargo test` in every crate + every executable check under
  `checks/`. This is the repository's "CI" (nothing runs off-machine).
- Hard rule 4: the `origin` remote exists but is never pushed to.
- **Network interpretation (for the design owner to veto):** rule 4's "no
  external services beyond the model endpoints" is read as banning anything
  that sends our work or data out (pushes, uploads). Read-only fetches of
  published, unmodified artifacts the toolchain needs (rustup target
  components, published crates) are treated like model endpoints — input,
  not exfiltration. The POC's dependencies are already in the local cargo
  cache. If this reading is wrong, say so: offline mode is possible with the
  local cache for the POC and dependency-free probes, but no_std targets and
  any new crate dependency would be blocked and re-planned.
- Fingerprint convention for epic 002 (lead decision, keeps 002-01
  independent of 003-01): the frozen-tree fingerprint covers src/, tools/,
  experiments/, Cargo.toml, Cargo.lock, README.md; excludes target/, .git/,
  .gitignore.

## Blockers and findings for the design owner

None so far.
