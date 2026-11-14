# 013-06 — Identity and capabilities sections
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-01, 013-02, 013-03, 013-04, epic 001

## Context
- Stamp group **Identity**: protocol version, build identity, full configuration, domain, profile (linux-std, macos-std, windows-std, embedded). `[decision]`
- Domain id is **16 bits, 0 = unconfigured (C-11)**.
- Stamp group **Capabilities**, each in one of four states, present / absent / unverified / not requested (C-129; "not requested" is a deliberate configuration choice, distinct from a denial): receive timestamp source, given as a value (kernel, userspace, mac-hardware, interrupt; C-131) plus its state; real-time scheduling granted (data and clock threads, C-127); memory locked; CPU idle states restricted (best-effort, stamped, C-127); drop accounting; DSCP marking; fault injection; holder-eligible; control isolation. `[decision]` State value 0 is invalid and rejected.
- Both groups are regime fields: any change changes the `StampId` (013-05).
- Platform facts: Linux has kernel wall-clock receive stamps via `SO_TIMESTAMPNS` and a per-socket drop counter via `SO_RXQ_OVFL` `[kernel-doc]`; macOS and Windows kernel receive stamps are claimed but unverified and they have no per-socket drop counter `[memory]`; embedded uses MAC hardware timestamps where available, else interrupt time. Task 001 supplies the measured capability-bit table.
- Control isolation (C-161): housekeeping cannot delay the data path. Source: present on hosted platforms by construction of the thread model; absent or unverified on a device unless the implementer shows a bound.
- Real-time scheduling and memory lock on the embedded profile (C-147): stamped present by construction (strict mode is hosted-only).
- Rule 2: a measurement involving two endpoints carries **both profiles** — the profile must therefore be in every stamp.
- Types: `CapState`, `Profile`, `RxStampSource`, `Capabilities` (013-01). Builder methods: `identity(&mut self, ..)` and `capabilities(&mut self, c: &Capabilities)`.
- Fuzz budget for the parser (C-95): 60 s per change, 1 hour nightly, corpus kept (013-03 harness).

## Work
1. Define the Identity section layout: protocol version, build identity, canonical configuration (from 013-04), domain as `u16`, profile as `u8` (layout per epic 004).
2. Define the Capabilities section layout: the receive-stamp source byte and its state byte, then 8 one-byte capability states, in the struct's field order.
3. Implement `StampBuilder::identity`, `StampBuilder::capabilities` and the matching readers on `Stamp`.
4. Extend the fuzz corpus (013-03) with a stamp containing both sections.
5. Add both sections' field tables to `doc/design/spec/stamp.md` (create the file if absent), with each field's source on each of the four profiles, or "absent" there. State `control_isolation` per C-161 and `rt_scheduling` and `memory_locked` on embedded per C-147; the receive-stamp source on macOS/Windows is "kernel, unverified" pending task 001 (C-131).

## Acceptance criteria
1. Build → parse round-trip returns identical values for all Identity fields and all capability fields (source plus 9 states), for each of the 4 profiles and for each of the 4 states including not requested.
2. Domain values 0, 1 and 65 535 round-trip; domain 0 is readable as "unconfigured".
3. A capability state byte outside {1, 2, 3, 4}, or a source byte outside the 4 C-131 values, makes `parse` return `Reject`.
4. Changing any one Identity or capability field changes `Stamp::id()` (test per field).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Design documents updated where behaviour differs, constants table first" (follows C-147 and C-161 as written); "The stamp format is versioned independently of the media format" (uses 013-02 framing unchanged).

## Out of scope
- Strict-mode refusal on domain 0 (013-20).
- Measuring which capabilities each OS really has (task 001; open sub-question "Which receive-stamp source does each OS really provide?", falsifier: kernel stamps absent, capability bits and E budgets change).
- Showing the bound that would make `control_isolation` present on a device (the device epic's implementer, C-161).
