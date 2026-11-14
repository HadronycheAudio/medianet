# 013-02 — Stamp envelope and section framing
**Epic:** 013-Regime-stamp-wire-object · **Effort:** M · **Depends on:** 013-01, epic 004

## Context
- The regime stamp is a compact, versioned wire object in `hadronyche-core` (`no_std`, no allocation). Its sections are **length-prefixed and versioned**. `[decision]`
- The stamp format is **versioned independently of the media format**. A parser that meets an **unknown section skips it and counts it**. `[decision]`
- A stamp has eight groups: Identity, Capabilities, Network, Clock, Alignment, Platform, Drops, Injection. The builder also writes per-connection entries, per-connection summary counters (C-47) and a self-test result. `[decision]`
- Transport is settled (C-18): a dedicated control message, split into sections that each fit the control message bound of **1 252 bytes of UDP payload (C-17)**; multicast on any regime change, refreshed every 10 s, unicast request for an immediate fetch. A full stamp therefore never has to fit one message; each section must fit one (or be split by the rule of 013-13).
- The byte layout (envelope fields, widths, byte order) is owned by epic 004. This ticket implements it; where epic 004 has not yet fixed a detail, the ticket proposes it in the module docs and marks it "pending epic 004".
- Reject reasons (C-193): epic 004 supplies the wire layouts **and the reject reasons** to the conformance vectors. The stamp's `Reject` variants follow the reasons epic 004 specifies for the stamp layout.
- `Stamp<'a>` is a borrowed view over the bytes; `StampBuilder<'b>` writes into a caller-provided buffer. A device-profile stamp must build without an allocator.

## Work
In `hadronyche-core` module `stamp`:

1. Implement the envelope: a stamp-format version (independent of any media-format version), a total length, then a sequence of sections, each with a section type, a section version and a length prefix. Follow epic 004's layout; record any proposed detail in the module docs (013-22 copies them to `doc/design/spec/stamp.md`).
2. Implement the framing parts of:
   ```rust
   impl<'a> Stamp<'a> { pub fn parse(bytes: &'a [u8]) -> Result<Self, Reject>; }
   impl<'b> StampBuilder<'b> { pub fn finish(self) -> &'b [u8]; }
   ```
   plus a constructor `StampBuilder::new(buf: &'b mut [u8])` and a private "append section" primitive used by the per-group tickets.
3. Parsing rules: reject truncated input, a section length that runs past the end, an unsupported stamp-format major version, and a duplicate known section (repeatable sections, such as per-connection entries and summaries, are exempt); **skip unknown section types and count them**, exposing the count on `Stamp` (proposed name `skipped_sections()`). A known section with a newer section version is skipped and counted like an unknown one; document this.
4. Builder: writing past the end of the caller buffer returns an error (or marks the builder failed so `finish` can report it); it never panics and never allocates. A single section longer than 1 252 bytes minus the message header is reported so that 013-13 can split it.
5. `Reject` gets one variant per failure above, named after the reject reasons of epic 004's stamp layout (C-193); a failure epic 004 has not yet named is proposed in the module docs and marked "pending epic 004".

## Acceptance criteria
1. `parse` of every prefix of a valid stamp (lengths 0..len-1) returns `Err`, never panics.
2. A stamp with 1, 2 and 3 unknown sections inserted between known ones parses, all known sections are readable, and the skipped count is 1, 2, 3 respectively.
3. A known section with a section version one above the supported one is skipped and counted.
4. A section length field pointing 1 byte past the end yields `Reject`.
5. A builder given a buffer 1 byte too small reports an error; the same content in an exactly-sized buffer succeeds.
6. Builds with no `alloc` (CI `no_std` check).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The stamp format is versioned independently of the media format, with the rule for unknown sections (skip and count)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- Envelope layout written in the module docs, ready for `doc/design/spec/stamp.md`.
- "Every parser touched is fuzzed": satisfied by 013-03, which must land before this parser is considered done.
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (framing only, no stamp field); "Design documents updated where behaviour differs, constants table first" (follows C-17, C-18 and C-193 unchanged).

## Out of scope
- Fuzz target (013-03).
- The content of each section (013-06 to 013-10, 013-23, 013-25), `StampId` (013-05), size budget and splitting (013-13), official vectors (013-12).
- Sending the control message (the hosted backend, epic 015, C-144, on the schedule of 013-24) and receiving it (013-17 collector); message header per epic 004.
