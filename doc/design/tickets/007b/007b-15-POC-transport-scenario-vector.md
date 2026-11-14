# 007b-15 — POC transport scenario vector
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** M · **Depends on:** 007a-07, 007a-04, epic 002, epic 003

## Context
- The frozen proof of concept (`/home/dorian/workspace/hadropoc`, tag `poc-baseline` from task 003) has a transport scenario test at `transport.rs:864-947` `[code]`: a conductor and a follower run a scripted scenario of tempo, time-signature, stop/continue and time-code changes; the follower's clicks must equal the conductor's expected clicks sample for sample, for block sizes 48 and 7 `[code]`.
- This becomes the conformance vector for read-only transport state semantics, re-expressed against the new wire format (read-only transport state content messages, valid encodings from 007a-07).
- The new wire carries one sample rate only, the domain rate of 96 000 Hz (C-01). Whether the POC scenario runs at that rate is an open sub-question of the epic owned by task 002. If it does not, the click positions cannot be copied sample for sample and task 002 restates how epic criterion 007-AC5 is judged; this ticket then waits for that answer.
- The POC may be cited only after task 002 has cleared the relevant instrument; the POC must not be modified (task 003 is the only exception).
- Deliverable: `vectors/transport/*.json`.
- Epic criterion 007-AC5: the transport scenario vector reproduces the POC result: clicks identical for block sizes 48 and 7.

## Work
1. Confirm task 002 has cleared this test and has stated the rate the scenario runs at; record both references in the vector file.
2. Read the scenario script and expected clicks from the POC at tag `poc-baseline` (read only).
3. Re-express it as `vectors/transport/poc-transport-*.json`: the script as a list of transport-state messages in the new wire format, and the expected click sample positions at 96 000 Hz, for block size 48 and block size 7.
4. Add a check that the expected clicks are identical for both block sizes, and a driver path that feeds the scenario to an `UnderTest`.

## Acceptance criteria
1. 2 scenario variants exist: block size 48 and block size 7.
2. The expected click positions in both equal the POC's expected clicks sample for sample (or, if task 002 rules the POC rate differs from 96 000 Hz, satisfy the comparison task 002 states).
3. Every transport-state message in the vector decodes with the valid-vector codec of the generator.
4. The vector file names the POC tag `poc-baseline`, the task 002 clearance and the sample rate.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No vector is extracted from the POC before task 002 has cleared the relevant instrument
- [ ] Every new factual claim carries an evidence tag
- [ ] Vector files carry the specification version they were generated from

## Out of scope
- Reference echo/mixer data (007b-17, 007b-18).
- Transport state across a gap (C-141) and dense controllers with no prior value (C-158) (007b-16).
- Implementing transport-state handling (epics 009–014).
