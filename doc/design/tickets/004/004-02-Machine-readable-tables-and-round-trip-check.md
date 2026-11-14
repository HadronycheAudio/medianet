# 004-02 — Machine-readable tables and round-trip check
**Epic:** 004-Wire-specification · **Effort:** M · **Depends on:** 004-01
## Context
Epic 004 delivers, beside the human-readable `doc/design/spec/wire.md`, a machine-readable form of every field table (`doc/design/spec/wire.toml` or similar). Task 007 (conformance) generates its test vectors from it, and tasks 009 to 014 implement the layouts it describes. The definition of done requires that "the machine-readable tables parse and round-trip".

Facts this ticket needs:
- Field tables in `wire.md` have the columns offset, width, type, meaning, valid range (fixed by 004-01).
- Every message must have a fixed maximum size, because a `no_std` core crate parses it without allocation and in bounded time `[decision]`.
- Sub-byte fields exist: the copy index is 2 bits (C-07); flags include *gap* and *degraded*.
- Field widths that must be expressible: 2-bit, 8-, 16-, 32-, 64- and 128-bit integers (C-10, C-11, C-12), 4-byte samples (C-03), and counted lists with a maximum (e.g. at most 32 bundle members, C-08; at most 64 peers, C-34). No rational type is needed: structural latency L and inserted join delays are whole numbers of domain-rate samples (C-104, C-105).
- Some messages are sectioned: the regime stamp is split into sections that each fit 1 252 bytes (C-18). One message is exempt from the 1 252-byte control bound: the path-MTU probe pair, sent at the intended media packet size (C-116).
- The epic leaves the schema to this ticket. The checker is tool code, so it lives in the `hadronyche-tools` crate (C-133); the tables live in `doc/design/spec/` (C-130). It runs in CI without privileges.
- C-125: a definition-of-done line with no subject is marked not applicable with the reason, never skipped.

## Work
1. Choose the format (`doc/design/spec/wire.toml` is the default) and write a schema description at the top of the file or in a "Machine-readable form" section of `wire.md`: one table per message; per field: name, offset (bits or bytes, stated), width, type, meaning, valid range, reserved value if any; per message: maximum size, layer, DSCP, and the list of rejection reasons it can produce.
2. Encode counted/variable parts as a count field plus a maximum count, so a consumer can compute the maximum message size from the file alone. Provide a "section of" attribute for sectioned messages.
3. Write a checker in the `hadronyche-tools` crate (C-133), run in CI unprivileged, that:
   - parses the file;
   - re-serialises it and checks the result parses to an identical model (round-trip);
   - checks, per message, that field offsets are contiguous and non-overlapping, and that the declared maximum size equals the sum computed from fields and maximum counts;
   - checks that every rejection reason referenced is a member of the closed `Reject` enumeration (once 004-03 lands; until then the check is skipped with a visible notice);
   - checks the size bound per message: at most 1 252 bytes for control messages and for each regime stamp section, with an explicit exemption attribute for the path-MTU probe pair (C-116).
4. Add one example message (a placeholder with two fields) so the checker has something to verify; later tickets replace it with real tables.
5. Wire the checker into CI.

## Acceptance criteria
1. The schema is documented and covers all column types listed in Work step 1.
2. The checker fails CI on: a parse error, a round-trip mismatch, overlapping fields, a gap between fields not declared as padding, and a declared maximum size different from the computed one (each demonstrated by one negative fixture, 5 fixtures in total).
3. The checker passes on the example message.
4. The checker runs without root privileges and without network access.
5. The checker's run time on the full file is bounded (no unbounded recursion; it finishes in under 10 s in CI).
6. The checker rejects a control message or regime stamp section whose maximum size exceeds 1 252 bytes unless it carries the probe exemption (one negative fixture).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The machine-readable tables parse and round-trip
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Not applicable unless the checker's file parser is hand-written (C-125): the default is a standard TOML library, which is not a parser of this epic. If this ticket writes its own parser, it is fuzzed under C-95 (60 s per change, 1 hour nightly, corpus kept)

## Out of scope
- The real field tables: each message ticket (004-06 to 004-20) adds its table to both `wire.md` and the machine-readable file.
- Vector generation (task 007).
- Wire parsers in the core crate (tasks 009 to 014).
