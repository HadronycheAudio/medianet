# Roadmap

Phases are ordered by dependency, not by calendar. A phase starts when its entry criteria hold and ends at its exit gate. Gates reference [03-acceptance-criteria.md](03-acceptance-criteria.md); task numbers reference [implementation/](implementation/).

```mermaid
flowchart LR
  P0["P0 Pre-implementation"]
  P1["P1 Specification"]
  P2["P2 Servo simulation"]
  P3["P3 Core crate"]
  P4["P4 Hosted crate, Linux edge, L1 and L2"]
  P5["P5 DSL compiler"]
  P6["P6 Embedded and L3"]
  P7["P7 Commissioning and documentation"]
  P0 -. "needed to close P1, not to start it" .-> P1
  P1 --> P2
  P1 --> P3
  P2 --> P3
  P3 --> P4
  P3 --> P5
  P3 --> P6
  P4 --> P6
  P5 --> P6
  P4 --> P7
  P6 --> P7
```

P1 drafting may begin while P0 runs; P1 cannot *close* before P0 does. P4 and P5 can run in parallel once P3 closes.

## P0 — Pre-implementation

| | |
|---|---|
| Goal | Make the POC a citable baseline and turn the platform assumptions into facts |
| Entry | None |
| Tasks | 001 Receive-stamp probes · 002 POC archaeology · 003 POC smoke fixes |
| Deliverables | POC under version control with before and after tags; stamp-probe report for macOS and Windows; archaeology report with a named mechanism for each instrument bug; clean loopback smoke run |
| Exit gate | Every `[memory]` claim in the platform capability table is confirmed or corrected; evidence items M-3 and M-4 are explained; open items O-17, O-18, O-19 closed |

## P1 — Specification

| | |
|---|---|
| Goal | A written protocol that two independent implementations could meet |
| Entry | Design set accepted |
| Tasks | 004 Wire specification · 005 Control state machines specification · 006 Timeline-epoch specification · 007 Conformance vectors |
| Deliverables | Field-level wire layouts; state-machine tables for discovery, election, admission and connection; epoch rules; byte-level and scenario vectors including malformed cases |
| Exit gate | Specifications reviewed; vectors exist for every message in both directions; control messages fit the size bound; open item O-23 closed; P0 closed |

## P2 — Servo simulation

| | |
|---|---|
| Goal | Confirm or replace the servo and alignment proposals before anything is built on them |
| Entry | P1 timeline-epoch and sync message definitions stable |
| Tasks | 008 Servo simulation harness |
| Deliverables | The redesigned servo estimator and seqlock in `hadronyche-core`; the simulation suite; the alignment estimator and ratchet simulation; updated constants |
| Exit gate | AC-2 to AC-7 pass in simulation; constants for slew, rate of change, lock bound and acquisition time are confirmed or replaced and their status updated; open items O-1 to O-5 closed for simulation |

This phase addresses risk 1 in [05-risks-and-open.md](05-risks-and-open.md). If the lock bound is unreachable, E targets are revised here, before any edge work.

## P3 — Core crate

| | |
|---|---|
| Goal | The whole protocol as pure `no_std` code, testable with a simulated transport |
| Entry | P1 closed; P2 closed |
| Tasks | 009 Gap and reassembly core · 010 Packetiser and bundles core · 011 Gossip and discovery core · 012 Admission handshake · 013 Regime stamp wire object · 014 Measurement path |
| Deliverables | `hadronyche-core` complete: packetiser, reassembler, alignment, gap classes, control state machines, stamp format, measurement messages |
| Exit gate | AC-1 passes; core builds for a `no_std` target; AC-10, AC-15, AC-16, AC-17, AC-18 and AC-23 pass against the core with a simulated transport |

## P4 — Hosted crate, Linux edge, L1 and L2

| | |
|---|---|
| Goal | A working medianet on Linux, measured on real hardware; then the macOS and Windows edges |
| Entry | P3 closed |
| Tasks | 015 Hosted networking backend · 016 Real-time setup backend · 017 Reference modules and L0 harness · 018 Linux ALSA edge · 019 L1 two-machine harness · 020 L2 certification harness · 021 macOS backend and Core Audio edge · 022 Windows backend and WASAPI edge |
| Deliverables | `hadronyche-std`, `hadronyche-os-linux`, `hadronyche-reference`; the L0, L1 and L2 harnesses; then `hadronyche-os-macos` and `hadronyche-os-windows` |
| Exit gate | AC-8, AC-9, AC-11 to AC-14, AC-19 to AC-22, AC-24, AC-25, AC-26, AC-30, AC-31, AC-34, AC-36, AC-37 pass. macOS and Windows reach AC-27 and AC-28, or their targets are replaced by measured values. Open items O-6 to O-11 closed |

Order inside the phase: 024a (runtime ABI and arithmetic kit, which builds the resamplers every edge uses, C-197) and 015, 016, 017 first (L0); then 018 and 019 (L1); then 020 (L2 Linux); then 021 and 022, which do not block P5.

Two tickets cross into P5: the cost calibration factors of the macOS and Windows edges (021-27 and 022b-09) load values that epic 025b measures (C-106, C-176). They are the last tickets of P4, so the macOS and Windows half of P4 closes after P5; the Linux half, which is what P6 waits for, does not depend on P5.

This phase addresses risks 2 and 3. If two bundles miss deadlines on GbE, the capacity map is revised here.

## P5 — DSL compiler

| | |
|---|---|
| Goal | Modules written in the DSL, with compiler-derived latency, cost and determinism class |
| Entry | P3 closed (the core ABI is stable) |
| Tasks | 023 DSL compiler front end · 024 DSL code generator and runtime ABI · 025 DSL conformance and fingerprint suite |
| Deliverables | `hadronyche-dsl`, `hadronyche-kit`; the reference echo as a DSL program; generated vectors |
| Exit gate | AC-32 and AC-33 pass; the DSL reference echo passes the exact tier in the L0 harness; open items O-14 to O-16 closed |

## P6 — Embedded and L3

| | |
|---|---|
| Goal | A device as a correct member, and the proof of cross-sink simultaneity |
| Entry | P4 Linux edge closed (a hosted reference sink exists); P5 closed (the kit and generated code are available) |
| Tasks | 026 Embedded HAL and reference device · 027 L3 cross-sink rig |
| Deliverables | `hadronyche-embedded`; one reference device; the L3 rig |
| Exit gate | AC-29 and AC-35 pass; open items O-12 and O-13 closed |

## P7 — Commissioning and documentation

| | |
|---|---|
| Goal | An operator can design, validate and certify a medianet |
| Entry | P4 closed; P6 closed |
| Tasks | 028 Commissioning tool · 029 Monitor (optional) |
| Deliverables | `hadronyche-tools`; per-OS checklists; operator documentation |
| Exit gate | The tool proposes A, validates trunks offline, certifies group membership and refuses more than 64 modules; checklists published; every pending constant is decided or carries a measurement plan; open items O-21, O-22 closed |

## Task to phase map

Epics above 40 days were split into lettered parts; where a phase table above names an epic by its number, it means all its parts.

| Phase | Epics |
|---|---|
| P0 | 001, 002, 003 |
| P1 | 004, 005, 006, 007a, 007b |
| P2 | 008a, 008b |
| P3 | 009, 010, 011, 012, 013, 014 |
| P4 | 024a (built first: the kit supplies every edge's resamplers, C-197), 015a, 015b, 016, 017a, 017b, 018, 019, 020, 021, 022a, 022b |
| P5 | 023, 024b, 025a, 025b |
| P6 | 026a, 026b, 027 |
| P7 | 028a, 028b, 029 |
