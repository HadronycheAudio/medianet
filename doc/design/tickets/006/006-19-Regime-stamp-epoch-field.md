# 006-19 — Epoch field of the regime stamp
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-03, 006-09 · **Cross-reference:** epic 013 (C-196)
## Context
A definition-of-done item of epic 006 requires that the epoch field of the regime stamp is specified: which epoch a record set belongs to, and that an epoch change starts a new record set. Epic 008 also needs "the published view's epoch field".

Facts (`[decision]`):
- A timeline epoch is a 16-bit counter (C-10), distinct from the election epoch; carried in every sync reply and every slice header.
- At an epoch change, each source ends its old session and starts a new one with a bumped session number (32-bit, C-10, C-19) at an announced position.
- Regime stamp transport (C-18): a dedicated control message split into sections that each fit one packet at the fallback MTU (at most 1 252 bytes of UDP payload, C-17). It is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch; the hello carries only a stamp version number. The message is defined by epic 013 and sent by the hosted backend, epic 015 (C-144).
- Epic 013 is a cross-reference, not a dependency (C-196): this ticket does not wait for epic 013's layout.
- Producers and consumers each publish per-connection summary counters in their own regime stamp: one counter per gap class, plus interrupted events, dead, and the observed DSCP (C-47). The full list is received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159).
- Because an epoch change starts a new record set, it is a regime change: the module re-multicasts its stamp with a new stamp version (C-18).

## Work
1. Add a "Regime stamp epoch field" section to `doc/design/spec/timeline.md` stating: the field's width (16 bits), its meaning (the timeline epoch of the line in force when the records were taken), and the rule "an epoch change starts a new record set".
2. Specify which epoch a record set belongs to at the boundary: records for positions before the switch position belong to the old epoch, from the switch position on to the new one. State how this applies to the C-47 per-connection counters (each record set carries its own counters).
3. State that an epoch change triggers an immediate multicast of the stamp with a new version (C-18). The section of the stamp that carries the epoch field is epic 013's to place: cite it if epic 013's layout already has it, otherwise state the field as a requirement on the stamp for epic 013 to carry (C-196).
4. Specify the epoch field of the published view (as given to epic 008; the view is published by the servo core, epic 008a, with the lock state and bound, C-187, a cross-reference under C-196): the epoch of the line the view describes.
5. Cross-reference the document that owns the regime stamp message (epic 013, C-144, C-196). If that document exists and lacks the field, record the request there (design documents updated where behaviour differs); if it does not exist yet, the requirement stated in step 3 is the cross-reference epic 013 picks up.

## Acceptance criteria
1. The section states width (16 bits), meaning, the boundary rule at the switch position, and "an epoch change starts a new record set".
2. The section states that an epoch change is a regime change and triggers a stamp multicast with a new version (C-18).
3. The published view's epoch field is specified in one sentence with its width.
4. The section cross-references epic 013 (C-196): either epic 013's layout already carries the field and is cited, or the field is stated as a requirement for epic 013 (and recorded in epic 013's document if that document exists). The ticket does not wait for epic 013.

## Definition of done
- [ ] The epoch field of the regime stamp is specified (which epoch a record set belongs to; an epoch change starts a new record set)
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The other regime stamp fields, the section layout and the record format (epic 013); sending the stamp (epic 015, C-144).
- The rules for the other-epoch counter (006-21, C-135, C-159).
- The simulation that consumes the published view (epic 008).
