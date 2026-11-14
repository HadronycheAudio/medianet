# 004-01 — wire.md skeleton and encoding conventions
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** none
## Context
Epic 004 writes the field-level wire specification of the Hadronyche Medianets protocol: media packets, control messages, sync exchanges and measurement messages, with exact offsets, widths, byte order, versioning and the required handling of every malformed case. The design fixes the rules; epic 004 fixes the bytes. This ticket creates the document and the conventions every later ticket of the epic writes against.

Facts this ticket needs:
- Two implementations will speak the protocol from one `no_std` core crate, so the specification must be implementable without allocation and with bounded parsing time `[decision]`.
- There are four layers `[decision]`:

| Layer | Carries | Transport | DSCP (C-13) |
|---|---|---|---|
| Discovery and control | hello, election, admission handshake with the manifest and the path-MTU probe pair, regime stamp sections and the stamp request | UDP over IPv4 (C-100): multicast with TTL 1, and unicast | CS5 (40) |
| Clock sync | request / reply exchanges with the holder | UDP unicast | CS6 (48) |
| Media | slices and in-band announcements | UDP unicast, one packet per slice by default | EF (46) |
| Measurement | receiver reports; one-way probes | UDP unicast, on the reverse port given in the handshake | reports CS5 (40); probes EF (46) |

- C-03: sample format is 4 bytes per sample, f32 or Q1.31; there is no 3-byte variant `[decision]`.
- C-01: the wire carries one sample rate only, 96 000 Hz; every stream on a medianet is at the domain rate, so there is one timeline per domain and no rate field is needed on the wire `[decision]`.
- C-100: IPv4 only in v1; multicast with TTL 1. A 1 500-byte path leaves 1 472 bytes of UDP payload and the 1 280-byte fallback (C-06) leaves 1 252 (C-17) `[decision]`.
- The POC (`proto.rs`) used an 88-byte little-endian header (`HDR = 88`, `MAGIC = 0x4844`, `VERSION = 11`) with direction-overloaded fields; all of that is retired `[code]`. The byte order of the new protocol is not inherited and must be decided here.
- Evidence tags used throughout: `[code]` read in the POC, `[measured]` observed, `[kernel-doc]` documented kernel behaviour, `[standard]` cited from a named, published standard with the clause, replacing `[memory]` once checked (C-132), `[memory]` recalled, unverified, `[decision]` design choice.
- C-130: specifications live in `doc/design/spec/`; nothing is written into the frozen POC tree.

## Work
1. Create `doc/design/spec/wire.md` beside the design documents.
2. Write the **byte order and alignment rules** section: one byte order for every multi-byte integer on the wire (decide and justify, tag `[decision]`); alignment guarantees (or explicit absence of them) for every field; how f32 and Q1.31 samples (C-03) are encoded; how signed integers are encoded; how bit fields inside a byte are numbered.
3. Fix the **field-table format** every later section uses, with exactly these columns: offset, width, type, meaning, valid range. State how variable-length parts (lists, strings) are expressed (count field plus fixed maximum) so that every message has a fixed maximum size.
4. Write the **layer overview** with the four-layer table above, including DSCP values (C-13) and transports.
5. Create empty, headed sections (with a one-line "to be filled by 004-TT" note naming the ticket) for: rejection reasons, versioning, reserved fields, media packet header, slice header, announcements, measurement extension, content messages, checksum, size budget, hello, election messages, admission messages, manifest, path-MTU probe pair, regime stamp, commissioning window messages, sync request and reply, receiver report, one-way probe, validity rules, traceability table.
6. Add a "Constants" subsection listing C-01 to C-13, C-17 to C-22, C-34, C-40, C-43, C-44, C-100 and C-137 (header cap: at most 100 bytes for all headers and the checksum beside 1 152 bytes of audio) with their values as given in the design constants table (`doc/design/02-constants.md`), to be referenced (not redefined) by later sections. The constants table wins over any copy.

## Acceptance criteria
1. `doc/design/spec/wire.md` exists and contains a byte-order section stating exactly one byte order for all multi-byte wire integers, with a `[decision]` tag and justification.
2. The alignment rules state, for each field width used (1, 2, 4, 8, 16 bytes and sub-byte bit fields), whether natural alignment is guaranteed.
3. The encoding of f32 and Q1.31 samples (both 4 bytes, C-03) is defined bit-exactly.
4. The field-table format has the five columns offset, width, type, meaning, valid range, and a rule that every variable-length part has a stated maximum count.
5. All 22 placeholder sections listed in Work step 5 are present.
6. The four-layer table matches the one in Context exactly (DSCP 40, 48, 46, 40/46; IPv4 with TTL 1 for multicast).
7. The Constants subsection matches the constants table for every listed id.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The document renders as valid Markdown with all tables well-formed

## Out of scope
- The machine-readable form (004-02) and the rejection enumeration (004-03).
- Any message field table (004-06 onwards).
- Versioning rules (004-04) and reserved-field rules (004-05).
