# 005-28 — Changed A propagation and fan-in re-alignment
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-15, 005-16, 005-27

## Context
- Machine 4 (connection states, consumer and producer sides) of `doc/design/spec/control.md`; tables `connection_consumer` and `connection_producer` in `doc/design/spec/control.toml` (005-15, 005-16); the A in force per admitted input from 005-27; checker from 005-03.
- C-142: a fan-in outside any presentation group inherits the A of the downstream presentation group it feeds (the largest if several), learned at admission; with no downstream group it needs an explicit budget in configuration.
- C-198: the receiver report (C-40, one per bundle every 100 ms) carries the consumer's current A, its own or the one it inherited. A change propagates one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186).
- C-186: every fan-in that inherited the old A re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do.
- C-208: each fan-in that learns of the change picks its own re-alignment position, one lead (C-43: 64 slices, 32 ms at defaults) ahead, and announces it in its own output, exactly as for a ratchet-triggered re-alignment; fan-ins at different depths switch at different, announced positions.
- C-211: the commissioning tool checks declared buffer memory before applying a change of A and refuses one that does not fit. If one reaches a fan-in anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running.
- Receiver reports are unicast, never multicast (C-47). L-2: re-alignment travels in-band in slices. L-5: a change of A changes no pinned destination.
- No ticket covered a change of A after admission: 005-27 handles only the A learned at admission.

## Work
1. Add the consumer's current A as a field of the receiver report in the producer input alphabet (`connection_producer`), and to the producer state data the A last reported by each downstream consumer; name the field for epic 004 in the "Regime stamp coupling" list of fields this specification reads or feeds (005-23 Work 6).
2. Define the inherited A of a fan-in as the largest A reported by its downstream consumers (C-142), and a local input "inherited A changed" from the producer side to the fan-in's consumer connections, raised when that largest value changes. A fan-in whose A comes from its own presentation group or a configured budget does not raise it.
3. Add the consumer rows for "inherited A changed" (`connection_consumer`): from Live, re-enter Aligning at a position the fan-in picks itself, one lead (C-43) ahead, announced in its own output (C-208), with rings and ratchet ceiling resized (C-186); under a guard for rings that do not fit the declared buffer memory, the row keeps the deepest ring it can hold as the ratchet ceiling, sets `alarm = true` for cannot-meet-A (C-174) and keeps the streams running (C-211); decide the rows for every other state (for example Aligning, Interrupted, Locking, Demoted, and the pre-admission states, which only update the A in force) and record them in "Decisions on open sub-questions" with a `[decision]` tag and a falsifier.
4. State that the consumer's own receiver reports carry its current A, own or inherited, so a fan-in passes the new value one hop further on its next report (C-198).
5. Add scenario S14 (each hop's announced position one lead after it learns the new A, C-208; plus a fan-in whose declared buffer memory cannot hold the new A, which keeps its deepest ring, raises cannot-meet-A and keeps streaming, C-211) to the "Scenarios" section: the A of a presentation group changes; the fan-in F1 feeding it re-aligns after the first report carrying the new A; the fan-in F2 feeding F1 re-aligns after F1's next report; a fan-in with a configured budget is unaffected.

## Acceptance criteria
1. The receiver report carries the consumer's current A and is the carrier of a changed A (C-198); no input is a multicast receiver report (C-47).
2. A change of the largest downstream A at a fan-in that inherited it moves each Live input to Aligning at a position announced one lead (C-43) ahead by that fan-in (C-208), with rings and ratchet ceiling resized (C-186); when the rings do not fit, the ceiling is the deepest ring it can hold, the cannot-meet-A alarm is raised and no stream stops (C-211); a fan-in with its own group's A or a configured budget has no such transition.
3. In S14 each hop re-aligns after one receiver report, so each hop learns the new A at most one report period (100 ms, C-40) after the hop downstream of it holds it.
4. Every consumer state has a row for "inherited A changed"; the checker of 005-03 passes; no new row changes a pinned destination (L-5) and the re-alignment position travels in-band (L-2).
5. The rows decided beyond the Live case are recorded with their falsifier.

## Definition of done
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- How commissioning chooses or changes a group's A (C-55, other epics). A sink's own switch to a changed A (C-214) is 005-29.
- Ring resizing and the alignment data path (other epics); the ring-fit outcome is the C-211 rule above. The commissioning tool's buffer-memory check before applying a change of A (C-211) belongs to the commissioning epic.
- The receiver report's wire layout (epic 004).
