# 007b-09 — Connection alarm scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007b-05, 007b-07, 007b-08

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Alarms are read as named events through the observation extension of 007a-04. Every transition row must be exercised (epic criterion 007-AC3).
- A connection alarm is raised on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded (C-173) `[decision]`.
- "Cannot meet A": maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows (C-174). Headroom is the largest of 25 % of the observed maximum, one slice period, and twice the published clock bound (C-52); D is the sink's total plumbing delay from hand-off to emission (C-170). The maximum lateness is taken over a rolling 60 s window; the maximum since the connection went live is reported beside it and does not drive the alarm (C-182).
- A larger than a fan-in's buffer memory (C-211): a fan-in that cannot hold a ring for the A it has learned keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm and keeps its streams running; the alarm then stays raised until a later change of A that fits (C-216); that scenario is in 007b-24.
- Triggers already covered elsewhere: refusal (007b-05, 007b-06), Interrupted and Dead (007b-07), Demoted (007b-08). This ticket adds the alarm expectations to those triggers and the cannot-meet-A rule.
- Control-degraded is entered when housekeeping restarts stop (C-139); producing it needs the panic-injection hook (C-145) and belongs to the hosted backend (epic 015, C-144).
- The alarm names come from the tables of epic 005.

## Work
1. Write scenarios in `vectors/control/alarm-*.json`:
   - one each for refusal, Interrupted, Dead and Demoted, each expecting exactly one alarm of the matching name at the transition;
   - cannot meet A: maximum lateness + D + headroom above A in each of 10 consecutive 1 s windows raises the alarm at the end of the 10th window; the same with one window at or below A (9 of 10) raises none;
   - rolling window (C-182): a single lateness value that alone puts the sum above A keeps the condition true while it stays in the 60 s window, so it raises the alarm at the end of the 10th window; once that value is more than 60 s old and no later value exceeds A, the condition is false; a value older than 60 s never contributes;
   - the scenario states the A, D, lateness series and clock bound it uses.
2. Add one scenario per remaining alarm row in the tables, except control-degraded.

## Acceptance criteria
1. Each of the 5 alarms refusal, interrupted, dead, demoted and cannot-meet-A has at least 1 scenario expecting it exactly once.
2. The cannot-meet-A alarm is raised at 10 consecutive windows and not at 9.
3. No scenario expects an alarm without one of the triggers of C-173.
4. All files carry `spec_version`.
5. The cannot-meet-A scenarios compute maximum lateness over the rolling 60 s window only; a lateness value older than 60 s never contributes, even though the maximum since live still shows it (C-182).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Producing the control-degraded state (epic 015, C-139, C-144, C-145).
- How alarms are shown to the operator.
