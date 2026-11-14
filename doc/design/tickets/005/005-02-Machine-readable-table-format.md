# 005-02 — Machine-readable table format
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-01

## Context
- Epic 005 specifies four pure control state machines (discovery map, election, admission handshake, connection states; the connection machine has a consumer and a producer side) as tables *(state, input) → (new state, output messages, events)*.
- Deliverable 2: a machine-readable form of the tables, `doc/design/spec/control.toml` "or similar", used by task 007 to generate scenario vectors, and used to check mechanically that the tables are total.
- The prose document is `doc/design/spec/control.md` (created by 005-01). Its transition-row format is: `state | input | guard | new state | output messages | events | invariants preserved (L-ids)`. Every (state, input) pair has a row, including `ignore`.
- Inputs are of three kinds: wire message (names from the epic 004 wire specification), timer, local event (from the data and clock threads).
- Invariant ids are L-1 to L-8. Constants are referred to by id; the canonical table is `doc/design/02-constants.md`, repeated in the Constants section of `control.md` (92 ids, 005-01).

## Work
1. Choose TOML at `doc/design/spec/control.toml` (the epic allows "or similar"; record the choice and the reason with a `[decision]` tag in the Conventions section of `control.md`).
2. Define the schema and document it in `control.md` (Conventions section):
   - `[[machine]]` with `name` (one of `discovery`, `election`, `admission`, `connection_consumer`, `connection_producer`);
   - `[[machine.state]]` with `name`, `data` (list of fields with type and bound), `memory_bound_bytes`, `initial` / `terminal` flags;
   - `[[machine.input]]` with `name`, `kind` (`message` | `timer` | `local`), `source` (004 message name, constant id for timers, or thread name);
   - `[[machine.transition]]` with `state`, `input`, `guard` (string, empty if none), `next`, `outputs` (list of message names), `events` (list of `ControlEvent` variants), `counters` (list of regime stamp counters incremented, may be empty: the C-159 per-connection counters, and for machine `election` the per-follower count of dropped sync requests, C-204), `alarm` (bool: the row raises a C-173 connection alarm), `invariants` (list of L-ids), `ignore` (bool);
   - a top-level `[constants]` table keyed by C-id with value and unit, for every constant of `control.md` whose value is a number; rule-only ids (for example C-18, C-19, C-47, C-103) are cited in guards and rows by id and not copied.
3. Add one small worked example (two states, two inputs, four rows) in the file as a fixture for the checker, under a machine name `example` excluded from the normative set.
4. State that the TOML is normative together with the prose tables, and that a mismatch between them is a defect.

## Acceptance criteria
1. `doc/design/spec/control.toml` exists and parses with a standard TOML 1.0 parser.
2. The schema in `control.md` names every key listed in Work step 2 with its type.
3. The `example` machine has exactly 2 states × 2 inputs = 4 transition rows, one of them with `ignore = true`.
4. The `[constants]` table contains every numeric constant of `control.md` with value and unit, and each value equals `doc/design/02-constants.md`; at least: C-01 = 96 000 Hz, C-02 = 48 frames (0.5 ms), C-06 = 1 280 bytes, C-08 = 32 members (controller member included, C-209), C-17 = 1 252 bytes, C-23 = 70 %, C-30 = 1 s, C-31 = 3 s, C-32 = 1.5 s, C-33 = 400 ms, C-34 = 64, C-217 = 63 peers, C-35 = 64 (pending L1), C-67 = 2 ms, C-36 = 50/s, C-37 = 1.5 s, C-38 = 8 slices, C-39 = 2 s, C-40 = 100 ms, C-41 = 2 s, C-42 = 2 hello periods, C-43 = 64 slices, C-46 = 3 s, C-50 = 4 s, C-53 = one slice period, C-54 = 10 s, C-101 = 10 windows of 1 s, C-111 = 2 s, C-112 = 10 to 200/s, C-139 = backoff 100 ms to 10 s and 5 panics within 60 s, C-174 = 10 windows of 1 s, C-182 = 60 s rolling window.
5. The format decision carries a `[decision]` tag.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Format documented so that task 007 can consume it without reading the prose

## Out of scope
- The totality checker (005-03).
- Normative content of any real machine (005-06 onwards).
- Vector generation itself (epic 007).
