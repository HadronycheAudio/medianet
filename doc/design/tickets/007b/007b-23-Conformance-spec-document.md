# 007b-23 — Conformance specification document
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007b-21, 007b-22

## Context
- Deliverable: `doc/design/spec/conformance.md` in the design set's repository (C-130): what conformance means, how to run the vectors, how a vector is added.
- Verification tiers `[decision]`: transport integrity (slice checksum); protocol conformance (this epic: byte-level vectors both directions including malformed inputs, state-machine scenario vectors, fuzzing); reference modules (sender, echo, mixer, bit-exact by construction); exact tier (record inputs and output hashes, replay offline, compare); tolerance tier (analyzer windows, per-voice projections with stated epsilons).
- The fuzz budget is C-95: each parser 60 s on every change and 1 hour nightly, corpus kept. It must be stated in this document.
- The bound on processing one bad input is C-164: linear in packet length and at most 10 µs per packet, a proposal pending L1. The cross-machine pair for bit-exactness is x86-64 and 64-bit ARM (C-151).
- Vector files carry the specification version they were generated from (`spec_version`, 007a-02). The reference hash algorithm and input script format come from 007b-18.
- Epic 007 adds and changes no regime stamp field: it encodes the regime stamp message as vectors with the fields defined by epic 004. The definition-of-done line on regime stamp fields is therefore not applicable here (C-125).

## Work
1. Write `doc/design/spec/conformance.md` with sections:
   - what conformance means (the tiers above, system criteria AC-1 and AC-23 wording);
   - vector kinds and directory layout (`vectors/wire/valid`, `vectors/wire/invalid`, `vectors/control`, `vectors/timeline`, `vectors/transport`, `vectors/reference`) and the `spec_version` stamp;
   - how to implement `UnderTest` (with its observation extension and the C-120 test-only hooks) and run the vectors (the command from 007b-22);
   - how a vector is added: wire vectors only via the generator tables, then regenerate; scenario vectors by hand with transition-row ids; reference vectors via the reference generation;
   - the fuzz budget (C-95) and how to run fuzzing locally;
   - the bad-input time bound (C-164, stated as pending L1) and the cross-machine pair (C-151);
   - the open sub-questions of the epic that are still open when this ticket lands, each with owner and falsifier.
2. Update the design constants table where any value differs from what was implemented.

## Acceptance criteria
1. The document has the 3 parts named by the deliverable: meaning, running, adding.
2. The fuzz budget is stated per target: 60 s per change, 1 hour nightly, corpus kept.
3. A reader following the "add a vector" steps on a test branch produces a vector that passes the schema, coverage and reproducibility checks.
4. Every open sub-question listed has an owner and a falsifier.

## Definition of done
- [ ] The fuzz budget (time or executions per target per CI run) is stated in `conformance.md`
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Regime stamp fields touched are complete and documented": this ticket touches no regime stamp field.

## Out of scope
- Tolerance-tier procedures and the fingerprint suite (other epics).
