# 022a-18 — No-ticker check: idle live sink CPU below 0.1 core
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-13

## Context
- Part of epic 022a (Windows edge backend).
- **No spinning ticker on Windows in v1** `[decision]`. Sink edges do not pace: they consume. Capture edges pace off the audio callback. Reason: default timer granularity 15.6 ms, best waitable timer about 0.5 ms `[memory]`, so a 0.5 ms slice clock (C-02) cannot be paced by sleeping.
- "Idle live sink" is settled in the epic: a sink admitted to a group and receiving one 6-channel stream whose content is silence, with no other module on the machine; process CPU averaged over 10 minutes.
- 022-AC-6: no thread spins to pace sends; CPU use of an idle live sink stays below **one tenth of a core**.

## Work
1. Code audit of the crate for busy-wait loops (spin on time, `yield`-loops); record the result.
2. Manual / harness test on reference hardware: a live sink admitted and receiving a stream of silence ("idle live") for 10 minutes; sample process CPU time.
3. Optional CI test with a mock device measuring process CPU over 30 s.

## Acceptance criteria
1. **(022-AC-6.)** Process CPU of an idle live sink averaged over 10 minutes on reference hardware is below 0.10 of one core; result recorded `[measured]`.
2. The audit finds no thread that spins to pace sends.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Sink implementation (022a-13).
