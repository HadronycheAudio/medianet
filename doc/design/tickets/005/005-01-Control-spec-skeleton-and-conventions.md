# 005-01 — Control spec skeleton and conventions
**Epic:** 005-Control-state-machines-specification · **Effort:** XS · **Depends on:** epic 004

## Context
- Epic 005 specifies the four control state machines of the runtime (discovery map, clock election, admission handshake, connection states) as pure tables: *(state, input message or timer) → (new state, output messages, events)*, with no I/O, so that one implementation in the `no_std` core crate `hadronyche-core` serves both the hosted and the embedded profile, and scenario vectors (task 007) can be generated from the tables.
- This ticket creates the normative document `doc/design/spec/control.md` and fixes its conventions so the later per-machine tickets only fill sections.
- Evidence tags used in this document: `[code]` read in the POC · `[measured]` observed · `[decision]` design choice · `[standard]` a figure cited from a named, published standard with its clause (C-132).
- Output locations (C-130): specifications in `doc/design/spec/`; nothing is written into the frozen POC tree.
- The canonical constants table is `doc/design/02-constants.md`; it wins over any copy. The epic's Background table repeats the 92 ids this specification uses: C-01, C-02, C-06, C-08, C-11, C-12, C-17, C-18, C-19, C-22, C-23, C-30 to C-42, C-43, C-45, C-46, C-47, C-50, C-53, C-54, C-55, C-67, C-92, C-95, C-96, C-100, C-101, C-102, C-103, C-106, C-108, C-109, C-111, C-112, C-113, C-114, C-115, C-116, C-117, C-120, C-124, C-130, C-132, C-133, C-135, C-136, C-139, C-140, C-142, C-144, C-152, C-154, C-159, C-161, C-162, C-163, C-165, C-168, C-173, C-174, C-182, C-185, C-186, C-187, C-188, C-193, C-195, C-198, C-204, C-206, C-208, C-209, C-211, C-213, C-214, C-215, C-217, C-221, C-222, C-224, C-225. Key values: domain rate 96 kHz, the only rate on the wire (C-01); slice 48 frames = 0.5 ms (C-02); IPv4 only, multicast TTL 1 (C-100); control messages at most 1 252 bytes of UDP payload (C-17); hello 1 s (C-30), expiry 3 s for non-live peers (C-31), listen-before-claim 1.5 s (C-32), election window 400 ms (C-33), at most 64 modules (C-34), 63 peer entries per map (C-217) and 64 followers (C-35, pending measurement at L1, task 019).
- Control-plane invariants:

| Id | Invariant |
|---|---|
| L-1 | An admitted, healthy connection becomes dead only from data-plane evidence: Interrupted after 8 consecutive missed slices, Dead after 2 s. Gossip silence alone never tears down a live stream. Reverse direction: the producer stops only after 2 s without receiver reports |
| L-2 | All position-deadline control (end-of-stream, re-alignment, timeline epoch switch, transport reference change) travels in-band in slices |
| L-3 | Gossip is discovery, admission bootstrap and election input only. A follower that receives no sync replies for 1.5 s opens an election regardless of hellos |
| L-4 | Housekeeping admits unknown sources only and never touches a live data path |
| L-5 | Destinations are pinned at admission and change only by re-admission. A hello with a new address never redirects a live stream |
| L-6 | The holder is judged by sync replies only. Hello expiry never opens an election while replies flow |
| L-7 | Fail-stop applies to data and clock threads. Housekeeping is supervised and restarted; the module is marked control-degraded |
| L-8 | No lock is shared between housekeeping and the data or clock threads |

## Work
1. Create `doc/design/spec/control.md` with these top-level sections, in order: Purpose and scope; Conventions; Constants; Invariants L-1 to L-8; Enumerations; Machine 1 Discovery map; Machine 2 Election; Machine 3 Admission handshake; Machine 4 Connection states (consumer and producer); Regime stamp coupling; Invariant preservation; Worst-case memory and work; Scenarios; Decisions on open sub-questions; Open items.
2. Copy the epic's Background constants table (92 ids listed above) into the Constants section, first, with the same ids and values, and state that `doc/design/02-constants.md` wins on any difference.
3. Copy the invariants table above.
4. Write the Conventions section:
   - every machine is pure: no I/O, no clock reads, no allocation; time arrives only as timer inputs;
   - the input alphabet has three kinds: wire messages (names and fields from the epic 004 wire specification), timers, and local events from the data and clock threads;
   - the transition-row format: `state | input | guard | new state | output messages | events | invariants preserved (L-ids)`;
   - every (state, input) pair has a row; a pair with no effect is written explicitly as `ignore` (same state, no output, no event);
   - epic acceptance criteria are cited as 005-AC-1 to 005-AC-8; system criteria keep their own ids (AC-12, AC-20, AC-22, AC-36);
   - every new factual claim carries an evidence tag.
5. Leave each machine section with fixed sub-headings: States and data; Memory bound; Input alphabet; Transition table; Outputs and events; Worst-case work per input.

## Acceptance criteria
1. `doc/design/spec/control.md` exists and contains the 15 sections listed in Work, in that order.
2. The constants table lists exactly the 92 ids above with values identical to `doc/design/02-constants.md`, and is the first table in the document.
3. The invariants table lists L-1 to L-8 verbatim.
4. The row format has exactly the 7 columns listed in Work, and the rule "every (state, input) pair has a row, including ignore" is stated.
5. Each of the four machine sections has the 6 fixed sub-headings.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The machine-readable form (005-02) and its checker (005-03).
- The Rust enumerations (005-04).
- Filling any machine's states or tables (005-06 onwards).
