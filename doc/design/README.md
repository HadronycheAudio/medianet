# Hadronyche Medianets — design set

The design of the Hadronyche Medianets runtime: a protocol and runtime for reliable, aligned audio over wired, isolated LANs, with a module-construction DSL and hosted and embedded implementations. It is derived from the hadropoc proof of concept.

## Documents

| Document | One line |
|---|---|
| [00-context.md](00-context.md) | Product frame, platform roles, the POC's role, the evidence-tag legend, vocabulary |
| [01-architecture.md](01-architecture.md) | The architectural specification, seventeen sections, nine diagrams |
| [02-constants.md](02-constants.md) | Every number and settled rule, once, with status, owner and falsifier. Wins over any other file |
| [03-acceptance-criteria.md](03-acceptance-criteria.md) | AC-1 to AC-37: statement, method, threshold, falsifier |
| [04-definition-of-done.md](04-definition-of-done.md) | Per-task checklist and per-phase exit gates |
| [05-risks-and-open.md](05-risks-and-open.md) | Three riskiest assumptions, open items with owners, the retired list with kill-reasons |
| [06-evidence-appendix.md](06-evidence-appendix.md) | What was measured, POC code pointers, OS facts, the three pre-implementation checks |
| [roadmap.md](roadmap.md) | Phases P0 to P7 with entry criteria, exit gates and dependencies |
| [adr/](adr/) | 24 architecture decision records |
| [implementation/](implementation/) | 36 self-contained epic files |
| [tickets/](tickets/) | One directory per epic: XS, S and M tickets with an index, coverage table and remaining gaps |

Not yet present: `spec/` (wire, control, timeline, conformance, servo and stamp specifications). Those documents are deliverables of the P1 to P3 tasks and will be written beside this set.

## Reading order by audience

**Decision reviewer** (is this the right design?)
1. [00-context.md](00-context.md)
2. [05-risks-and-open.md](05-risks-and-open.md): risks first, then the retired list
3. [adr/](adr/), in number order
4. [01-architecture.md](01-architecture.md), sections 14 and 15 for the boundary and the guarantees
5. [02-constants.md](02-constants.md), for what is decided against what is still a proposal

**Implementing agent** (what do I build?)
1. [00-context.md](00-context.md), for the evidence legend and vocabulary
2. [01-architecture.md](01-architecture.md), in full
3. [02-constants.md](02-constants.md)
4. [roadmap.md](roadmap.md), to find your phase
5. Your task file in [implementation/](implementation/)
6. [04-definition-of-done.md](04-definition-of-done.md)
7. [06-evidence-appendix.md](06-evidence-appendix.md), when you need a POC pointer

**Commissioning and operations** (how do I deploy and trust it?)
1. [00-context.md](00-context.md)
2. [01-architecture.md](01-architecture.md) sections 1, 6, 8, 9, 10 and 11
3. [02-constants.md](02-constants.md), the capacity and edge tables
4. [03-acceptance-criteria.md](03-acceptance-criteria.md), the edge and capacity criteria
5. [implementation/028a-Commissioning-configuration-and-offline-validation.md](implementation/028a-Commissioning-configuration-and-offline-validation.md) and [028b](implementation/028b-Commissioning-survey-certification-and-operations.md)

**Ticket-creation agent** (one file in, tickets out)
- Read exactly one file in [implementation/](implementation/). Each is self-contained: goal, background with its constants repeated, deliverables, dependencies, acceptance criteria, definition of done, open sub-questions and scope.
- Use [roadmap.md](roadmap.md) only for ordering.

## Architecture decision records

| # | Decision |
|---|---|
| [01](adr/01-Single-packet-slice-default.md) | Single-packet slice default, fragmentation forbidden |
| [02](adr/02-Channel-bundling.md) | Channel bundling for wide sources |
| [03](adr/03-Runtime-owned-packetisation.md) | Runtime-owned packetisation |
| [04](adr/04-Detect-and-conceal-redundancy.md) | Detect and conceal; no retransmission, no FEC |
| [05](adr/05-Position-invariant-fan-in-alignment.md) | Position-invariant fan-in alignment |
| [06](adr/06-Ratcheted-offset.md) | Ratcheted offset |
| [07](adr/07-Control-plane-invariants.md) | Control-plane invariants L-1 to L-8 |
| [08](adr/08-Two-sided-admission.md) | Two-sided admission |
| [09](adr/09-Domain-line-with-epochs.md) | Domain-wide timeline with epochs |
| [10](adr/10-Servo-redesign.md) | Servo redesign |
| [11](adr/11-Software-resampler-edge.md) | Software resampler at every edge |
| [12](adr/12-Presentation-groups-with-required-E.md) | Presentation groups with required E |
| [13](adr/13-Interrupted-and-dead-thresholds.md) | Interrupted and dead thresholds |
| [14](adr/14-Capability-bit-holder-eligibility.md) | Capability-bit holder eligibility |
| [15](adr/15-Crate-split-with-state-machines-in-core.md) | Crate split with control state machines in core |
| [16](adr/16-Determinism-classes-with-fingerprint.md) | Determinism classes with fingerprint |
| [17](adr/17-Content-domain-scope.md) | Content-domain scope for v1 |
| [18](adr/18-DSCP-classes.md) | DSCP classes |
| [19](adr/19-Six-channels-per-stream.md) | Six channels per stream |
| [20](adr/20-Ten-GbE-for-wide-nodes.md) | 10 GbE for wide nodes; multicast deferred |
| [21](adr/21-Rewrite-versus-evolve.md) | Rewrite, with the POC frozen as reference |
| [22](adr/22-Authentication-deferred.md) | Authentication deferred with reserved fields |
| [23](adr/23-macOS-and-Windows-edge-roles.md) | macOS and Windows as edges |
| [24](adr/24-Numbers-governance.md) | Numbers governance |

## Implementation tasks

Epics above 40 days were split into lettered parts (007a, 007b, …); a reference to a split epic's number means all its parts.

| # | Epic | Phase | Scope | Tickets |
|---|---|---|---|---|
| [001](implementation/001-Receive-stamp-probes.md) | Receive stamp probes | P0 | L (17 tickets, about 17 days) | [tickets](tickets/001/README.md) |
| [002](implementation/002-POC-archaeology.md) | POC archaeology | P0 | M (14 tickets, about 13 days) | [tickets](tickets/002/README.md) |
| [003](implementation/003-POC-smoke-fixes.md) | POC smoke fixes | P0 | M (10 tickets, about 5.5 days) | [tickets](tickets/003/README.md) |
| [004](implementation/004-Wire-specification.md) | Wire specification | P1 | L (29 tickets, about 30.5 days) | [tickets](tickets/004/README.md) |
| [005](implementation/005-Control-state-machines-specification.md) | Control state machines specification | P1 | L (27 tickets, about 33 days) | [tickets](tickets/005/README.md) |
| [006](implementation/006-Timeline-epoch-specification.md) | Timeline epoch specification | P1 | M (21 tickets, about 11 days) | [tickets](tickets/006/README.md) |
| [007a](implementation/007a-Wire-vectors-and-harness.md) | Wire vectors and harness | P1 | L (15 tickets, about 22.5 days) | [tickets](tickets/007a/README.md) |
| [007b](implementation/007b-Scenario-and-reference-vectors.md) | Scenario and reference vectors | P1 | L (24 tickets, about 37.5 days) | [tickets](tickets/007b/README.md) |
| [008a](implementation/008a-Servo-core-and-simulation-harness.md) | Servo core and simulation harness | P2 | L (16 tickets, about 22 days) | [tickets](tickets/008a/README.md) |
| [008b](implementation/008b-Servo-simulation-cases-and-results.md) | Servo simulation cases and results | P2 | L (20 tickets, about 24.5 days) | [tickets](tickets/008b/README.md) |
| [009](implementation/009-Gap-and-reassembly-core.md) | Gap and reassembly core | P3 | L (29 tickets, about 29 days) | [tickets](tickets/009/README.md) |
| [010](implementation/010-Packetiser-and-bundles-core.md) | Packetiser and bundles core | P3 | L (24 tickets, about 23 days) | [tickets](tickets/010/README.md) |
| [011](implementation/011-Gossip-and-discovery-core.md) | Gossip and discovery core | P3 | L (26 tickets, about 29 days) | [tickets](tickets/011/README.md) |
| [012](implementation/012-Admission-handshake.md) | Admission handshake | P3 | L (30 tickets, about 31.5 days) | [tickets](tickets/012/README.md) |
| [013](implementation/013-Regime-stamp-wire-object.md) | Regime stamp wire object | P3 | L (27 tickets, about 28.5 days) | [tickets](tickets/013/README.md) |
| [014](implementation/014-Measurement-path.md) | Measurement path | P3 | L (25 tickets, about 24 days) | [tickets](tickets/014/README.md) |
| [015a](implementation/015a-Hosted-runtime-core.md) | Hosted runtime core | P4 | L (24 tickets, about 35 days) | [tickets](tickets/015a/README.md) |
| [015b](implementation/015b-Hosted-runtime-verification-and-strict-mode.md) | Hosted runtime verification and strict mode | P4 | M (15 tickets, about 14 days) | [tickets](tickets/015b/README.md) |
| [016](implementation/016-Real-time-setup-backend.md) | Real-time setup backend | P4 | L (16 tickets, about 16 days) | [tickets](tickets/016/README.md) |
| [017a](implementation/017a-Reference-modules.md) | Reference modules | P4 | L (13 tickets, about 21 days) | [tickets](tickets/017a/README.md) |
| [017b](implementation/017b-L0-harness-and-scenarios.md) | L0 harness and scenarios | P4 | L (26 tickets, about 37 days) | [tickets](tickets/017b/README.md) |
| [018](implementation/018-Linux-ALSA-edge.md) | Linux ALSA edge | P4 | L (25 tickets, about 29 days) | [tickets](tickets/018/README.md) |
| [019](implementation/019-L1-two-machine-harness.md) | L1 two-machine harness | P4 | L (27 tickets, about 34.5 days) | [tickets](tickets/019/README.md) |
| [020](implementation/020-L2-certification-harness.md) | L2 certification harness | P4 | L (28 tickets, about 37 days) | [tickets](tickets/020/README.md) |
| [021](implementation/021-macOS-backend-and-Core-Audio-edge.md) | macOS backend and Core Audio edge | P4 | L (31 tickets, about 37 days) | [tickets](tickets/021/README.md) |
| [022a](implementation/022a-Windows-backend-and-WASAPI-edge.md) | Windows backend and WASAPI edge | P4 | L (21 tickets, about 30.75 days) | [tickets](tickets/022a/README.md) |
| [022b](implementation/022b-Windows-edge-L2-certification.md) | Windows edge L2 certification | P4 | M (10 tickets, about 12.25 days) | [tickets](tickets/022b/README.md) |
| [023](implementation/023-DSL-compiler-front-end.md) | DSL compiler front end | P5 | L (32 tickets, about 38.75 days) | [tickets](tickets/023/README.md) |
| [024a](implementation/024a-Runtime-ABI-and-arithmetic-kit.md) | Runtime ABI and arithmetic kit | P4 | L (15 tickets, about 17 days) | [tickets](tickets/024a/README.md) |
| [024b](implementation/024b-DSL-code-generator.md) | DSL code generator | P5 | L (21 tickets, about 32 days) | [tickets](tickets/024b/README.md) |
| [025a](implementation/025a-DSL-conformance-corpus-probe-and-reference-programs.md) | DSL conformance corpus, probe and reference programs | P5 | L (22 tickets, about 26 days) | [tickets](tickets/025a/README.md) |
| [025b](implementation/025b-Replay-cross-target-fingerprint-and-cost-calibration.md) | Replay, cross-target fingerprint and cost calibration | P5 | L (13 tickets, about 17 days) | [tickets](tickets/025b/README.md) |
| [026a](implementation/026a-Embedded-HAL-and-host-integration.md) | Embedded HAL and host integration | P6 | L (13 tickets, about 18 days) | [tickets](tickets/026a/README.md) |
| [026b](implementation/026b-Reference-device-and-porting-guide.md) | Reference device and porting guide | P6 | L (22 tickets, about 23 days) | [tickets](tickets/026b/README.md) |
| [027](implementation/027-L3-cross-sink-rig.md) | L3 cross-sink rig | P6 | L (25 tickets, about 28.75 days) | [tickets](tickets/027/README.md) |
| [028a](implementation/028a-Commissioning-configuration-and-offline-validation.md) | Commissioning configuration and offline validation | P7 | M (11 tickets, about 8.75 days) | [tickets](tickets/028a/README.md) |
| [028b](implementation/028b-Commissioning-survey-certification-and-operations.md) | Commissioning survey, certification and operations | P7 | L (24 tickets, about 31.5 days) | [tickets](tickets/028b/README.md) |
| [029](implementation/029-Monitor.md) | Monitor (optional) | P7 | L (25 tickets, about 27 days) | [tickets](tickets/029/README.md) |

## Conventions

- **Evidence tags** (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`) are defined in [00-context.md](00-context.md). A `[memory]` claim must be checked by a named task before anything depends on it.
- **Constants** are cited by id (C-nn). [02-constants.md](02-constants.md) is the only place a value is defined.
- **Open items** always carry an owner and a falsifier. A decision-shaped open item is a defect in this set.
