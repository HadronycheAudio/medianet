# 004-25 — Regime stamp section types named by the constants table
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-24, 004-08
## Context
The regime stamp container, version and request are defined in 004-24 (C-18). The constants table names these stamped contents `[decision]`:
- **Multi-packet regime** of a stream (C-05): the values 004-08 hands over (stream number, packets per slice, frames per slice or equivalent).
- **Per-connection health** (C-47, C-159): producer and consumer each publish, per connection, the counters received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events and dead (C-159), plus the observed DSCP (C-47). Late versus missing follows C-108; interrupted counts misses as defined by C-102 (threshold C-38); dead starts 2 s after the last slice with content or a marked gap (C-39, C-111); a slice on another timeline epoch restarts the dead clock, is not a miss and is counted under other-epoch (C-135); demoted rounds make the Demoted state (C-109) visible to third parties.
- **Calibration factor** (C-106): time per counted abstract operation for this target, measured once by the DSL conformance suite of epic 025 running a reference workload (C-176); admission converts the manifest's cost bound with it.
- **Commissioning window** (C-107, C-153): every module stamps the declared window, with its expiry time, while it is open; a commissioning tool may drive test traffic only inside it. The tool's announcement and close messages are laid out in 004-29.
- **Injections** (C-120, C-145): active injections, including the test-only hooks: forced module id and nonce, a stream forced onto a given timeline epoch, raw packets sent on a media or control socket, and a panic injected into a data, clock or housekeeping thread.
- **Capability states** (C-129): each capability is in one of four states: present, absent, unverified, not requested. Strict mode requires real-time scheduling on the data and clock threads and locked memory; restricting CPU idle states is best-effort and is stamped (C-127). Strict mode refuses an unverified capability (C-128); that is behaviour, the stamp only reports the state. Strict mode is hosted-only; a device stamps real-time scheduling and memory lock as present by construction (C-147). The control-isolation capability (housekeeping cannot delay the data path) is present on hosted platforms by construction and absent or unverified on a device unless the implementer shows a bound (C-161).
- **Receive-timestamp source** (C-131): one of kernel, userspace, mac-hardware, interrupt, each reported with a C-129 capability state.

Open items not settled here: further section types (owned by the epics that stamp them, as minor changes). Sections added by C-152, C-157 and C-160 are laid out in 004-28.

## Work
1. One section type per item above, each with a type code and a field table (widths, units, valid ranges, counter wrap or saturation rule).
2. Encode the four capability states of C-129 as one closed enumeration reused by every capability field and by the receive-timestamp source.
3. Encode the four receive-timestamp sources of C-131 as a closed enumeration.
4. Fix maximum counts (connections per health section, injections, capabilities) and show that each section fits 1 252 bytes at the maxima (for example 256 streams per source, C-09, and 64 modules, C-34), splitting a type over several sections where needed.
5. Validity rules with outcomes: unknown enumeration value, count above maximum, length inconsistent (`BadLength`), reserved bits non-zero.
6. Add the tables to the machine-readable file.

## Acceptance criteria
1. Seven section types exist, one per item in Context, each with a type code and a maximum size ≤ 1 252 bytes.
2. The capability-state enumeration has exactly the four values of C-129; the timestamp-source enumeration has exactly the four values of C-131.
3. The health section has the eleven C-159 counters plus the observed DSCP.
4. The capability section includes control isolation (C-161) and the injection section includes the panic hook (C-145).
5. At least 5 invalid forms with outcomes.
6. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The stamp container and request (004-24).
- Measuring the calibration factor or the capabilities (other epics).
- Receiver report layout (004-20).
