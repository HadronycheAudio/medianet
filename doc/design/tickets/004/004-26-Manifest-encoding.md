# 004-26 — Manifest encoding in the admission handshake
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-18
## Context
The admission handshake (004-18) carries the module's manifest. The rulings fix what it contains `[decision]`:
- Ports and content types (control rules).
- **Channel map** (C-04): media packets carry no per-channel id byte; audio channels appear in manifest order.
- **Structural latency L** (C-105): always a whole number of domain-rate samples. The domain rate is 96 000 Hz, the only rate on the wire (C-01). A resampler node appears only inside an edge module (C-134), and its conversion delay is part of D, never of L (C-207).
- **Conversion delay of an edge** (C-207, C-189): an edge's resampler delay is counted only in D; the manifest reports the conversion delay separately, on the device-facing port. The "frequency-dependent delay" property stays on outputs, for recursive filters (C-189), which is the join flag below.
- **Inserted join delays** (C-104): when paths of different latency meet at a join, the compiler pads the shorter path with a whole-sample delay line and reports the inserted delay in the manifest; the module's L is the longest path. An explicit construct opts out per join. A path with frequency-dependent delay is padded on its structural part only and the join is flagged in the manifest.
- **Dense controller initial values** (C-158): each dense controller declares an initial value (default 0), used when a gap precedes any received value.
- **Cost bound** (C-106): counted abstract operations per slice, identical on every machine; admission converts it with the calibration factor from the regime stamp (004-25).
- **E** (C-103, C-73): a presentation group may set a required E, or none; a sink declares its E, or unknown when it has no certified E. A sink declaring unknown is refused with `RequiredEExceeded` (004-18) by any group that sets a required E. Only a commissioned sink declares a numeric E; uncertified and self-monitored sinks declare unknown (C-140). E is defined by C-73; the targets are given in microseconds (C-70 to C-72).
- C-17: the manifest-carrying messages are at most 1 252 bytes each. C-09: at most 256 streams per source.
- No constant bounds the number of inserted join delays. This ticket fixes the bound (open item owned by epic 004; falsifier: a compiled module has more padded joins than the bound, or the manifest then exceeds 1 252 bytes).

## Work
1. Field table for the manifest: ports and content types, channel map, L as a whole number of samples (width and valid range), inserted join delays (count, each a whole number of samples, each with the frequency-dependent-delay flag of C-104), dense controller initial values (C-158), the conversion delay on a device-facing port (C-207, width and unit stated, tag `[decision]`), cost bound (operations per slice, width), required E per presentation group with a distinct "none" encoding, declared E with a distinct "unknown" encoding, and the unit of E.
2. Fix maximum counts and compute the maximum manifest size; if it exceeds 1 252 bytes, report the falsified constant and propose a split across messages.
3. Validity rules with outcomes: count above maximum, length inconsistent (`BadLength`), channel map entry out of range or duplicated, a conversion delay on a port that is not device-facing (C-207), reserved non-zero.
4. Add the tables to the machine-readable file.

## Acceptance criteria
1. Every item in Context has a field with width and unit.
2. "None" for required E and "unknown" for declared E have distinct encodings that no valid E value can take.
3. The maximum manifest size is stated in bytes and fits 1 252 bytes, or a falsified constant is reported (epic AC-3).
4. The join-delay bound is recorded as an open item with owner and falsifier.
5. At least 5 invalid forms with outcomes.
6. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The admission messages that carry the manifest (004-18).
- How the compiler computes L, padding or the cost bound (compiler epics).
