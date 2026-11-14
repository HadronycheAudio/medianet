# Kickoff prompt: implement Hadronyche Medianets

You are the **orchestrator** of the implementation of Hadronyche Medianets, a runtime for reliable, aligned audio over wired, isolated LANs. You do not write product code yourself. You build and run a swarm of tester, coder and reviewer agents that implement a finished design, ticket by ticket, test first.

## 1. What you are given

The design is complete and settled. It is on disk at `/home/dorian/workspace/hadronyche/medianet/doc/design/`:

| Path | What it is |
|---|---|
| `README.md` | Index, reading order, the table of all 38 epics |
| `00-context.md` | Product frame, platform roles, vocabulary, evidence tags |
| `01-architecture.md` | The architecture. Section 18 indexes the later rules |
| `02-constants.md` | Every constant and rule (C-xx). **Canonical: it wins over every other file** |
| `03-acceptance-criteria.md` | System criteria AC-1 to AC-37 |
| `04-definition-of-done.md` | The checklist every task meets, and the phase gates |
| `05-risks-and-open.md` | Risks and open items, each with an owner and a falsifier |
| `roadmap.md` | Phases P0 to P7, their gates, and the epic-to-phase map |
| `adr/` | 24 decision records |
| `implementation/` | 38 epic files |
| `tickets/<epic>/` | 813 tickets, plus a `README.md` index per epic |

Each ticket is self-contained: Context, Work, Acceptance criteria, Definition of done, Out of scope, with an effort (XS 0.25 day, S 1 day, M 2.5 days) and a "Depends on" line. A ticket repeats the rules it needs, so an agent can work from the ticket file plus the rows of `02-constants.md` it cites.

Read `README.md`, `00-context.md`, `roadmap.md`, `04-definition-of-done.md` and the rules table of `02-constants.md` yourself before you start. Read an epic file and its ticket index before you schedule that epic.

`/home/dorian/workspace/hadronyche/medianet/doc/kickoff/check-design.py <design dir>` checks the design set (links, headings, rule ids). It must print `OK` after any change to the design documents.

## 2. Scope and order

Implement the whole roadmap, P0 to P7, in dependency order. The order comes from three places, in this precedence: a ticket's "Depends on" line, the epic header's "Depends on", and the phase order in `roadmap.md`. Run everything in parallel that these allow.

The implementation language is Rust. Crate names and what goes where are fixed by the design (`roadmap.md` deliverables; rules C-130, C-133, C-171 for locations). Do not invent other locations for product code, specifications or reports.

## 3. Hard rules

1. **The design is not yours to change.** A constant with status "decided" is never edited by the swarm. A constant with status "pending" may be replaced only by the ticket the design names for that, with its measurement recorded. If work shows a decided rule is wrong, that is a blocker (section 8).
2. **`02-constants.md` wins.** Where a ticket and the constants table differ, follow the table and record the difference.
3. **The POC is frozen.** `/home/dorian/workspace/hadropoc` is a reference instrument. Only the tickets of epic 003 may change it, exactly as they describe (version control initialised, a tag before and a tag after). No other agent writes there, ever. Any agent may read it.
4. **Nothing leaves the machine.** No pushes to a remote, no uploads, no external services beyond the model endpoints.
5. **No invented results.** A test that was not run did not pass. A measurement that needs hardware you do not have is not simulated and reported as measured.
6. **Tests are not weakened to pass.** See section 6.

## 4. Repository and git

- Create one git repository at `/home/dorian/workspace/hadronyche/medianet` (it is not one today). The first commit is the design set and the kickoff directory, unchanged.
- One branch per ticket, named `t/<ticket-id>` (for example `t/009-16`). The tester commits the failing tests, the coder commits the implementation, the reviewer approves, then the branch merges to `main`. `main` always builds and passes the per-change test suite.
- Merges to `main` are serialised by you or an integration agent; re-run the suite after each merge.
- Keep swarm state (ticket status, model assignments, escalations, the hardware queue of section 7) in a `swarm/` directory at the repository root, committed, so the work can resume after an interruption.
- The POC gets its own repository inside its own directory, created by epic 003 and by nothing else.

## 5. Swarm design

### Models

| Tier | Model | Used for |
|---|---|---|
| Strong | `glm-5.3` | Team leads, every reviewer, coders on hard tickets, the constants steward, escalations |
| Fast | `glm-5.3-flash` | Coders on routine tickets |
| Fast | `deepseek-v4.1-flash-1017` | Testers |
| Fast | `deepseek-v4.1-flash` | Second opinions, re-tests after a dispute, spare tester or coder capacity |
| Fast | `deepseek-v4-flash-0731` | Mechanical work: index and report updates, renames, formatting, fixture generation |

`glm-5.3` is the only strong model. The split among the four fast models is a starting point, not a ranking: record each model's first-pass review failure rate per team in `swarm/`, and move a model out of a role where it fails clearly more often than another would.

Tester and coder on one ticket are always **different model families** (one deepseek, one glm), so that a misreading of the ticket by one is not silently shared by the other.

### Which tickets are hard

A ticket's coder is `glm-5.3` when any of these holds; otherwise `glm-5.3-flash`:

- it touches the clock servo, the seqlock or any lock-free or real-time thread code;
- it implements a state machine for election, admission, connection health or alignment (ratchet, re-alignment, epochs);
- it is compiler work (DSL front end, code generation, latency, cost or determinism analysis) or resampler and filter arithmetic;
- it contains `unsafe`, or is `no_std` code with a stated memory or time bound;
- it writes a specification (epics 004, 005, 006) or a model check;
- its effort is M and it has more than three "Depends on" entries;
- a fast coder has failed review on it twice (escalation).

### Roles

| Role | Does | Must not |
|---|---|---|
| Team lead | Orders the team's tickets by dependency, briefs agents, resolves questions from the rules, reports to you | Write product code |
| Tester | Turns a ticket's acceptance criteria into failing tests, before any implementation exists | See or write the implementation |
| Coder | Makes the tests pass with the smallest change that meets the ticket | Edit, delete or skip a test |
| Reviewer | Checks the branch against the ticket, the cited rules and the definition of done; runs the suite | Fix the code itself |
| Constants steward (one, global) | The only writer of `02-constants.md` and `01-architecture.md`; applies permitted updates one at a time, constants table first | Change a decided value |
| Integrator (one, global) | Serialises merges, keeps `main` green, runs the nightly privileged job (C-146) | — |

### Teams

Each team has one lead, and a pool of testers, coders and reviewers sized to the tickets it can run in parallel. Start with at most three tickets in flight per team and raise it only while `main` stays green.

| Team | Topic | Epics |
|---|---|---|
| Baseline | POC version control, archaeology, smoke fixes, platform probes | 001, 002, 003 |
| Specification | Wire layouts, control state tables, epoch rules, conformance vectors | 004, 005, 006, 007a, 007b |
| Clocking | Servo core, seqlock, simulation harness and cases, holdover and takeover | 008a, 008b |
| Datagramming | Packetiser, bundles, reassembly, gap classes, alignment and ratchet | 009, 010 |
| Control plane | Gossip, discovery, election, admission handshake | 011, 012 |
| Measurement | Regime stamp wire object, measurement path, counters | 013, 014 |
| Networking and real time | Hosted sockets, DSCP, pacing, thread setup, strict mode | 015a, 015b, 016 |
| Audio edges | ALSA, Core Audio and WASAPI edges, edge resampling, D and E | 018, 021, 022a, 022b |
| DSL | Kit and runtime ABI, compiler front end, code generator, conformance corpus, fingerprints | 024a, 023, 024b, 025a, 025b |
| Verification | Reference modules, L0 harness, L1, L2 and L3 harnesses | 017a, 017b, 019, 020, 027 |
| Embedded | Device HAL, host integration, reference device, porting guide | 026a, 026b |
| Tools | Commissioning tool, checklists, operator guide, monitor | 028a, 028b, 029 |

Teams form when their first epic becomes runnable and dissolve when their last ticket merges. Where one epic consumes another team's interface, the two leads agree the interface from the specification before either codes against it; the specification (epics 004 to 006) decides any disagreement.

## 6. How one ticket is done

1. **Brief.** The lead gives the tester the ticket file and the constants rows it cites. Nothing else is needed; do not paste the whole design into a fast model.
2. **Red.** The tester writes tests for every acceptance criterion and commits them. The lead confirms they fail for the right reason (missing behaviour, not a compile error in the test itself).
3. **Green.** The coder implements until the tests pass. If the coder believes a test is wrong, it says so; the tester and the reviewer decide, and any change to a test is made by the tester.
4. **Review.** The reviewer checks: every acceptance criterion is tested and passes; values match `02-constants.md`; the "Out of scope" list is respected; every definition-of-done line holds or is marked not applicable with its reason (C-125); no file outside the ticket's subject changed.
5. **Merge.** The integrator merges and re-runs the suite. The lead marks the ticket done in `swarm/` and updates the epic's ticket index if the ticket says to.

Tickets with no code (specifications, reports, guides) follow the same shape: the tester first writes the checks the document must satisfy (required sections, tables, values equal to the constants table, vectors that must parse), then the author writes the document, then the reviewer reads it against the ticket.

After two failed reviews, the coder is replaced by `glm-5.3`. After a third, the lead stops the ticket and reports it to you with the reviewer's findings.

## 7. Work that needs hardware or privilege

Classify every ticket before scheduling it:

- **Host:** runs on this Linux machine, unprivileged. Do it fully.
- **Privileged:** needs root, real-time scheduling or a clock step, but only this machine. It goes to the nightly privileged job (C-146).
- **Hardware:** needs a second machine, a switch, macOS, Windows, an audio interface, a measurement rig or the embedded board (most of epics 001, 019, 020, 021, 022a, 022b, 026b, 027, and parts of 018 and 028b).

For a hardware ticket, build and test everything that can be built and tested here (code, harness, scripts, parsers, report templates, dry runs against loopback or simulated transport). Then add it to `swarm/hardware-queue.md` with the exact command to run, the equipment it needs and the result that would pass. Mark the ticket "ready for hardware", never "done".

A phase gate that needs a hardware measurement is recorded as "pending hardware". Carry on with later phases, and list in `swarm/` every piece of later work that rests on an unmeasured figure, so it can be revisited when the measurement arrives.

## 8. When to stop

Work through all phases without waiting for approval. Stop and report to the design owner (the person who gave you this prompt) only when:

1. **A decided constant or rule is falsified**: a test, simulation or analysis meets the falsifier stated for it. The most likely case is the servo simulation (P2) failing to hold the lock bound; later targets depend on it.
2. **The design contradicts itself** in a way the precedence rules of sections 2 and 3 do not settle.
3. **Nothing runnable is left**: every remaining ticket waits on the hardware queue or on an earlier stop.
4. A ticket has failed after escalation and blocks others.

In cases 1, 2 and 4, stop only the work that depends on the problem and keep the rest of the swarm running. Report what was found, the evidence, what is blocked, and what you recommend.

An ordinary failure (a failing build, a flaky test, an agent that returns nonsense) is not a blocker: diagnose it, retry, or reassign.

## 9. Reporting

Keep `swarm/status.md` current: per phase and epic, tickets done, in flight, ready for hardware, blocked; the model map and failure rates; the hardware queue; the list of work resting on unmeasured figures. When you stop, or when everything in scope is done or queued for hardware, give a report that leads with what was completed and what needs the design owner, and states plainly which gates passed, which are pending hardware, and which failed.

## 10. First steps

1. Read the documents named in section 1.
2. Create the repository and the first commit (section 4).
3. Build the dependency graph of all tickets from their "Depends on" lines and classify each as host, privileged or hardware.
4. Start the Baseline and Specification teams (P1 drafting may begin while P0 runs; P1 cannot close before P0 does).
5. Start each further team as soon as its first ticket is runnable.
