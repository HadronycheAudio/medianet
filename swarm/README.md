# Swarm state

Orchestrator-maintained state for the implementation swarm. Everything here is
committed so the work can resume after an interruption (kickoff section 4).

| File | What it is |
|---|---|
| `README.md` | This file: the operating manual |
| `build_graph.py` | Rebuilds `graph.json` from the design set; run after design changes |
| `init_ledger.py` | Initialises `ledger.json` with ticket classifications |
| `brief.py` | Prints a ticket brief (ticket file + cited constants rows) |
| `graph.json` | Ticket dependency graph, frontier, epic summary (generated) |
| `ledger.json` | Per-ticket status: `todo`, `in-flight`, `review`, `done`, `hardware-ready`, `blocked`; class `host` / `privileged` / `hardware`; model assignments |
| `status.md` | The running report (kickoff section 9) |
| `models.md` | The model map, role assignments and first-pass failure rates |
| `hardware-queue.md` | Tickets ready for hardware: command, equipment, pass result |
| `escalations.md` | Failed reviews, escalations, blocked tickets |
| `unmeasured.md` | Work resting on unmeasured figures (kickoff section 7) |

## Operating manual

### Hard rules, restated

1. The design is not ours to change. `02-constants.md` wins over any ticket.
   A decided rule that work shows to be wrong is a blocker, reported to the
   design owner; a pending constant is replaced only by the ticket the design
   names for it.
2. The POC at `/home/dorian/workspace/hadropoc` is frozen. Only epic 003
   tickets write there, exactly as the tickets describe. Everyone may read it.
3. Nothing leaves the machine: no pushes, no uploads, no external services
   beyond the model endpoints. The `origin` remote is never pushed to.
4. No invented results. A test that did not run did not pass. A measurement
   that needs absent hardware is queued, never simulated.
5. Tests are never weakened to pass. Only the tester changes a test, after a
   dispute is decided by the tester and the reviewer.
6. After any change to `doc/design/`, `python3 doc/kickoff/check-design.py
   doc/design` must print `OK`.

### Code quality (standing condition from the design owner)

Every crate must be clippy-clean: `cargo clippy --all-targets -- -D
warnings` passes in every crate, and the per-change suite enforces it. Coders
deliver clippy-clean code or the review fails; reviewers run it as part of
the suite. `cargo fmt --check` is not currently required.

Function length (design owner rule, 2026-11-14): "Every function that's over
100 lines of code must be split." Counting convention: a function body over
100 code lines — lines after the opening brace through the closing brace,
excluding blank lines and comment-only lines — must be split. Enforced by the
per-change suite via `scripts/function-length.py`, which scans every Rust
source in this repository (`target/`, `.git` and `doc/` excluded). The frozen
POC at `/home/dorian/workspace/hadropoc` is out of scope: it is not part of
this tree, and epic 003's fixes there must stay minimal.

### One ticket, in five steps (kickoff section 6)

Every agent works in its own git worktree, never in the shared checkout at
the repository root (concurrent agents race there):

```
git worktree add /tmp/opencode/wt/<ticket-id> t/<ticket-id>   # green/review
git worktree add /tmp/opencode/wt/<ticket-id> -b t/<ticket-id> main  # red
```

The orchestrator merges from a dedicated mainline worktree
(`/tmp/opencode/wt/mainline`); worktrees are removed after the merge. Stage
only the ticket's own files — never `git add -A` in a shared checkout.

1. **Brief.** The lead gives the worker the brief from `python3
   swarm/brief.py <ticket-id>`: the ticket file plus the cited constants rows.
   Nothing else.
2. **Red.** The tester (a deepseek model) turns every acceptance criterion
   into failing tests on branch `t/<ticket-id>`, and commits. For a document
   ticket, the tester writes the checks the document must satisfy. The tester
   never sees the implementation.
3. **Green.** The coder (a glm model; `glm-5.3` for hard tickets, otherwise
   `glm-5.3-flash`) makes the tests pass with the smallest change that meets
   the ticket, and commits. The coder never edits, deletes or skips a test;
   disputes go back to the tester and reviewer.
4. **Review.** The reviewer (`glm-5.3`) checks the branch against the ticket,
   the cited rules and the definition of done, and runs the suite. It never
   fixes the code itself; it approves or lists findings.
5. **Merge.** The integrator merges to `main` serially, re-runs the suite,
   marks the ticket in `ledger.json` and updates `status.md` and the epic
   ticket index if the ticket says to.

After two failed reviews the coder is replaced by `glm-5.3`; after a third the
ticket stops and is escalated in `escalations.md`.

### Classification (kickoff section 7)

- **host**: done fully here, unprivileged.
- **privileged**: needs root, RT scheduling or a clock step on this machine;
  the nightly privileged job (C-146).
- **hardware**: build and test everything possible here, then add to
  `hardware-queue.md` with the exact command, equipment and pass result;
  status becomes `hardware-ready`, never `done`.

### Where checks live

- Code tickets: tests inside the crate the ticket names (unit tests are
  unprivileged and fast; long runs, real-network runs and hardware runs are
  demonstrated to the reviewer, not run per-change).
- Document tickets: the tester writes an executable check under `checks/`
  (`checks/<ticket-id>.sh` or `.py`) that validates the document against the
  ticket's acceptance criteria. The per-change suite runs every file in
  `checks/`. When `hadronyche-tools` exists (ticket 002-09), checks that
  belong in the crate move there in that ticket.
- Ticket 003-01's checks verify the POC repository's state from outside; no
  check ever writes into the POC tree.

### Scheduling

- A ticket runs when its ticket-line dependencies are `done` (or
  `hardware-ready`) and its phase is open per `roadmap.md`. The frontier in
  `graph.json` is the candidate set; the orchestrator applies phase gating.
- At most three tickets in flight per team, raised only while `main` stays
  green.
- Tester and coder on one ticket are always different model families.

### Probe crates (C-133)

Every crate under `probes/` is standalone: its Cargo.toml carries an empty
`[workspace]` table so the repository workspace never claims it (cargo's
root `exclude` globs do not free nested packages on this toolchain; the
per-crate table does, verified empirically). The per-change suite runs
`cargo test` and clippy in every `probes/*/`.

### Frozen-POC access (serialisation rule)

Only one agent may write to /home/dorian/workspace/hadropoc at a time (epic
003 tickets), and while an epic-003 write is in flight, no other agent runs
or verifies against the frozen tree (a run mid-write sees mixed code, and a
concurrent verifier's clean-tree check fails spuriously). Reads are always
fine. The orchestrator sequences POC-writing tickets against POC-running
verifications.

### Remote (amended by the design owner, 2026-11-14)

Kickoff rule 4 ("nothing leaves the machine") is amended: the integrator
pushes `main` to `origin` (github.com:HadronycheAudio/medianet) immediately
after every merge and after swarm-state commits that follow it. In-flight
ticket branches are not pushed. Agents never push; the integrator does.
