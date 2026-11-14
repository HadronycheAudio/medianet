# 018-15 — Admission with self-declared E
**Epic:** 018-Linux-ALSA-edge · **Effort:** XS · **Depends on:** 018-14, epic 012

## Context
- A presentation group carries a **required E**. A sink whose E exceeds it is **refused at admission with a named reason** `[decision]`. Admission and the manifest carrying required E are epic 012.
- Epic criterion 018-A4 = **AC-30**: a sink configured with a self-declared E above the group's required E is refused at admission with a named reason.
- **C-103:** a sink with no certified E declares its E as **unknown**; it is refused with reason `RequiredEExceeded` from any presentation group that sets a required E, and admitted to groups that set none.
- The declared E is a configuration value of the sink (the certified value from its certification run); absent means unknown. Only the *commissioned* state gives a numeric E; an *uncertified* or *self-monitored* sink declares E as unknown even if a value is configured (C-140, `declares_numeric_e()` from 018-06).
- A refusal raises a connection alarm (C-173).
- **C-180:** a *commissioned* sink whose clock becomes unlocked stays commissioned, so it keeps declaring its numeric E; its alignment claim is suspended and does not count towards the group's required E until the clock is locked again, then resumes without re-commissioning.
- Linux reference target E = 50 µs (C-70, pending measurement).

## Work
1. Add an optional `declared_e_ns` to the `AlsaSink` configuration; present it at admission (epic 012 manifest) as a number when the certification state is *commissioned*, otherwise as unknown (C-103, C-140).
2. Ensure the refusal reason from admission is surfaced in the sink's status and log, and that the connection alarm is raised (C-173).
3. Integration test with the epic 012 admission path.

## Acceptance criteria
1. **AC-30:** group required E = 50 µs, *commissioned* sink declared E = 60 µs → refused with reason `RequiredEExceeded`; the message names both values; a connection alarm is raised (C-173).
2. *Commissioned*, declared E = 50 µs → admitted.
3. No declared E, group with a required E → refused with `RequiredEExceeded` (C-103).
4. No declared E, group with no required E → admitted (C-103).
5. *Self-monitored* sink with a configured E of 50 µs, group with a required E → declared as unknown and refused with `RequiredEExceeded` (C-140).
6. **C-180:** an admitted *commissioned* sink whose fake clock goes unlocked keeps declaring its numeric E and reports its claim as suspended; on relock the claim resumes with no commissioning step and no new refusal.

## Definition of done
- [ ] Tests green in CI, unprivileged

## Out of scope
- The admission engine itself (epic 012).
- Computing the certification state (018-06).
- Discounting a suspended claim from the group's required E on the group side (C-180; epic 012).
