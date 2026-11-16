# 02 — Settled constants

This is the single source of truth for every number in the design set. Other documents repeat these values; if a value differs anywhere, this file wins and the other file is a bug.

**Status values**

| Status | Meaning |
|---|---|
| decided | Chosen during design; changing it is a design change |
| inherited | Taken from the POC without re-examination |
| derived | Computed from other constants; recompute if they change |
| pending-simulation | A proposal that the servo/alignment simulation must confirm |
| pending-measurement | A proposal that a ladder rung (L1, L2, L3) must confirm |
| reserved | Wire space kept for a deferred feature |

Evidence tags (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`) are defined in [00-context.md](00-context.md).

## Media and packetisation

| ID | Constant | Value | Status | Owner when pending | Falsifier |
|---|---|---|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, and it is the only rate on the wire: every stream on a medianet is at the domain rate. An edge whose hardware runs at another rate (typically 48 kHz) converts at the edge | decided | — | — |
| C-02 | Slice length (default) | 48 frames = 0.5 ms = 2 000 slices/s | inherited | — | — |
| C-03 | Wire sample format | 4 bytes per sample, f32 or Q1.31; no 3-byte variant | decided | — | — |
| C-04 | Channels per stream at defaults | 6 (192 bytes per channel per slice, 1 152 bytes of audio; no per-channel id byte, the channel map is in the manifest) | decided | — | Headers plus in-band content exceed the ~320 bytes left in a 1 472-byte payload |
| C-05 | Default slice size | Fits one packet at the path MTU; multi-packet slices are an explicit, stamped regime | decided | — | — |
| C-06 | Path-MTU fallback | 1 280 bytes | decided | — | — |
| C-07 | Copy index | 2 bits in the packet header | reserved | — | — |
| C-08 | Members per bundle | at most 32, the controller member included (C-209) | decided | — | — |
| C-09 | Streams per source | at most 256 | decided | — | — |
| C-10 | Stream number / session / timeline epoch widths | 16 / 32 / 16 bits | decided | — | Spec drafting shows 16-bit epoch wraps within a plausible uptime |
| C-11 | Module id / instance nonce / domain id | 64 / 64 / 16 bits; domain 0 = unconfigured | decided | — | — |
| C-12 | Authentication reservation in control messages | scheme 8 bits (0 = none) + message counter 64 bits + tag 128 bits | reserved | — | — |
| C-13 | DSCP classes | sync CS6 (48); media and one-way probes EF (46); control CS5 (40) | decided | — | — |
| C-14 | Intra-slice egress spread | at most 25 % of the slice period (125 µs at defaults), plus a per-source phase offset derived from the module id | decided | — | L1 shows queue drops with the spread on, or consumer margins broken by it |
| C-15 | Reassembly deadline, arrival-driven multi-packet input | 2 slice periods | decided | — | — |
| C-16 | Bounded drain per input per cycle | 2 × the admitted packets per slice for that input | decided | — | L1 shows legitimate bursts exceeding it |
| C-17 | Control message size bound | must fit one packet at the fallback MTU: at most 1 252 bytes of UDP payload | decided | — | Hello for a 32-member bundle source does not fit |
| C-18 | Regime stamp transport | a dedicated control message split into sections that each fit C-17. Multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number | decided | — | — |
| C-19 | Session number ownership | assigned by the source, one per stream; bumped on restart and on timeline epoch change; every consumer of a stream sees the same value | decided | — | — |

## Capacity (all derived from C-01 to C-04)

| ID | Constant | Value | Status | Owner when pending | Falsifier |
|---|---|---|---|---|---|
| C-20 | Wire size of one default packet | about 1 320 bytes including all Ethernet overhead | derived | — | Final header layout changes the size by more than 5 % |
| C-21 | Wire rate per stream | about 21.1 Mbit/s (about 3.5 Mbit/s per channel) | derived | — | same |
| C-22 | Wire rate per source | 24 ch: 4 streams, 84 Mbit/s, 8 000 packets/s. 48 ch: 8 streams, 169 Mbit/s, 16 000 packets/s. 96 ch: 16 streams, 338 Mbit/s, 32 000 packets/s | derived | — | same |
| C-23 | Admission ceiling | 70 % of the link rate | decided | — | — |
| C-24 | GbE port wall | about 198 channel-copies per port (8 × 24 ch, 4 × 48 ch or 2 × 96 ch) | derived | — | — |
| C-25 | 10 GbE trigger | any port carrying more than about 198 channel-copies | decided | — | — |
| C-26 | 100 Mbit device port | about 19 channels | derived | — | — |
| C-27 | Alignment buffer per 16-member bundle | about 370 KB at a 10 ms skew budget | derived | — | — |

## Control plane

| ID | Constant | Value | Status | Owner when pending | Falsifier |
|---|---|---|---|---|---|
| C-30 | Hello period | 1 s | inherited | — | — |
| C-31 | Hello expiry | 3 s, for non-live peers only | inherited | — | — |
| C-32 | Listen-before-claim | 1.5 s | inherited | — | — |
| C-33 | Election window | 400 ms | inherited | — | — |
| C-34 | Active modules per medianet | at most 64; commissioning refuses more | decided | — | Device memory cannot hold the 63-peer map (C-217) |
| C-35 | Followers per holder | at most 64; excess refused with a named event | decided | L1 (task 019) | Holder serving lateness grows with followers below 64 |
| C-36 | Sync requests per follower | 50 per second | inherited | — | — |
| C-37 | Election on reply silence | 1.5 s | decided | — | — |
| C-38 | Interrupted threshold | 8 consecutive missed slices (4 ms at defaults) | decided | — | — |
| C-39 | Dead threshold | 2 s without data | decided | — | — |
| C-40 | Receiver report | one per bundle every 100 ms | decided | — | — |
| C-41 | Producer timeout | 2 s without receiver reports | decided | — | — |
| C-42 | Stalled data plane visible to peers | within 2 hello periods | decided | — | — |
| C-43 | In-band announcement lead | 64 slices (32 ms at defaults), flat | decided | — | A descendant misses an announcement despite four re-issues |
| C-44 | Announcement re-issue | every 16 slices until the named position passes, in every bundle member | decided | — | — |
| C-45 | Pending timeout (target not present) | none: Pending persists until the target appears or the operator cancels; it is shown as unsatisfied | decided | — | — |
| C-46 | Quarantine of a colliding id by third parties | until only one instance nonce has been seen for one hello-expiry period (3 s, C-31) | decided | — | — |
| C-47 | Third-party observability of stream health | producer and consumer each publish per-connection summary counters in their own regime stamp, one counter per gap class (received, late, missing, marked gap), plus interrupted events, dead, and the observed DSCP; receiver reports are not multicast and there is no query message | decided | — | The summaries are too coarse to diagnose a fault |

## Alignment

| ID | Constant | Value | Status | Owner when pending | Falsifier |
|---|---|---|---|---|---|
| C-50 | Aligning duration | 4 s | decided | simulation, then L1 | Late slices after alignment on a clean network |
| C-51 | Offset estimator | maximum observed lateness plus headroom (percentiles are reported, not used) | decided | simulation, then L1 | same |
| C-52 | Headroom | the largest of: 25 % of the observed maximum, one slice period, twice the published clock bound | decided | simulation, then L1 | same |
| C-53 | Ratchet step | one slice period; ceiling is A | decided | simulation | Oscillation, or A exhausted in normal use |
| C-54 | Dwell before automatic re-alignment | 10 s | decided | simulation | same |
| C-55 | Audition offset A | no default; one per presentation group; proposed by commissioning | decided | commissioning tooling | — |

## Clock

| ID | Constant | Value | Status | Owner when pending | Falsifier |
|---|---|---|---|---|---|
| C-60 | Slew while locked | at most 10 ppm from the holder's rate | pending-simulation | simulation | Lock bound unreachable under this limit |
| C-61 | Rate of change of rate | at most 1 ppm per second | pending-simulation | simulation | same |
| C-62 | Lock bound | at most E/2, that is 25 µs against C-70 | pending-simulation | simulation | Unreachable in acceptable acquisition time |
| C-63 | Acquisition time to Locked | at most 10 s from the first exchange (proposal; the POC minimum is 1.6 s) | pending-simulation | simulation | Not reachable under C-60 and C-61 |
| C-64 | Holdover bound growth | 1 µs per second (1 ppm floor) | decided | — | — |
| C-65 | Alignment claim lapse in holdover | about 25 s: (C-70 − C-62) / C-64 | derived | — | — |
| C-66 | Servo measurement window | 200 ms; a window uses its minimum-round-trip exchange | inherited | simulation | — |
| C-67 | Step threshold | 2 ms | inherited | simulation | — |
| C-68 | Edge rate-loop bandwidth | well below 1 Hz | pending-measurement | L2 | Edge passes servo jitter into the resampler ratio |

## Edge and certification

| ID | Constant | Value | Status | Owner when pending | Falsifier |
|---|---|---|---|---|---|
| C-70 | E, Linux reference sink | 50 µs (20 clock + 20 tracking + 10 uncertainty of D) | pending-measurement | L2 Linux | Measured value exceeds it |
| C-71 | E, macOS, built-in or bus-attached device | 100 µs | pending-measurement | L2 macOS | same |
| C-72 | E, Windows, WASAPI exclusive | 1 ms | pending-measurement | L2 Windows | same |
| C-73 | Definition of E | 99.9th percentile of absolute emission error over a one-hour certification run; the maximum is reported beside it | decided | — | — |
| C-74 | Restart spread of D over 20 restarts | Linux at most 25 µs; macOS at most 50 µs; Windows at most 0.5 ms | pending-measurement | L2 per platform | Measured spread exceeds it |
| C-75 | L2 round-trip marker offset | Linux: constant within ±2 samples over the run. macOS and Windows: constant within that platform's restart spread (C-74) | pending-measurement | L2 | — |
| C-76 | L2 slip after disturbance | zero accumulated slip. Linux: offset returns to its prior value within ±1 sample. macOS and Windows: within that platform's restart spread (C-74) | pending-measurement | L2 | — |
| C-77 | L2 wander | MTIE at most 10 µs for intervals up to 1 s; at most E for any interval | pending-measurement | L2 | — |
| C-78 | L2 handover test | crystal difference of 50 ppm between old and new holder | decided | — | — |
| C-79 | L3 cross-sink skew | the 99.9th percentile of absolute skew is at most the sum of the two sinks' E values, for 1 hour and across restarts of either; the maximum is reported beside it | pending-measurement | L3 | — |
| C-82 | L2 cross-sink skew against the Linux reference sink | macOS at most 150 µs; Windows at most 1.05 ms (the sum of the two E targets) | derived | — | — |
| C-80 | Edge hardware at 48 kHz | the edge converts 2:1 through the kit's resampler node; the wire stays at the domain rate (C-01) | decided | — | — |
| C-81 | Typical D | Linux about 1 ms; macOS a few ms; Windows 6 to 20 ms | pending-measurement | L2 per platform | — |

## Loss and capacity verification

| ID | Constant | Value | Status | Owner when pending | Falsifier |
|---|---|---|---|---|---|
| C-90 | Expected loss floor | at a bit error rate of 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire): one loss per stream per link about every 13 hours at 2 000 packets/s `[memory]` | pending-measurement | pre-implementation check, then L1 | Published figure differs; clean-network loss well above it |
| C-91 | Loss alarm threshold | set from the clean-network loss measured at L1; alarm when exceeded | pending-measurement | L1 | — |
| C-92 | Two-bundle capacity test | one Linux consumer on one GbE port aligns 2 × 96 channels (32 streams, 64 000 packets/s, 676 Mbit/s) for 1 hour with zero late slices at the commissioned A; a third bundle is refused | pending-measurement | L1 | Any late slice |
| C-93 | Guessed receive cost | 2 to 4 µs per packet (13 to 26 % of the slice period for two bundles) `[memory]` | pending-measurement | L1 | Measured cost makes C-92 fail |
| C-94 | Wired jitter profile for simulation (provisional, synthetic) | base one-way delay 100 µs; exponential jitter with 20 µs mean; 1 % of packets in bursts delayed by 1 ms | pending-measurement | L1 (task 019) replaces it with the measured profile | Measured profile differs enough to change a simulation verdict |
| C-95 | Fuzz budget | each parser 60 s on every change and 1 hour nightly; the corpus is kept | decided | — | — |
| C-96 | Skew budget cap | the presentation group's A (C-55): alignment rings are sized to A | decided | — | — |

## Rules settled after the ticket split

Splitting the epics into tickets exposed holes in the design. These rules close them. They are decisions unless marked otherwise.

| ID | Rule | Value |
|---|---|---|
| C-100 | IP version | IPv4 only in v1; multicast with TTL 1. The 1 280-byte fallback (C-06) is a conservative path MTU giving 1 252 bytes of UDP payload |
| C-101 | "Persistently late" (triggers a ratchet step) | at least one late slice in each of 10 consecutive 1 s windows; this is the dwell C-54 expressed as a test |
| C-102 | What counts as a miss for the interrupted threshold (C-38) | only slices absent at the deadline: missing, or arriving late. A marked gap or a slice on another timeline epoch proves the upstream is alive and resets the count |
| C-103 | Sink with no certified E | declares E as unknown; refused with reason RequiredEExceeded from any presentation group that sets a required E; admitted to groups that set none |
| C-104 | DSL paths of different latency meeting at a join | the compiler pads the shorter path with a whole-sample delay line, reports the inserted delay in the manifest, and the module's L is the longest path; an explicit construct opts out per join. A path with frequency-dependent delay is padded on its structural part only and the join is flagged in the manifest |
| C-105 | Representation of structural latency L | always a whole number of domain-rate samples. The kit's resampler node is offered only in designs whose delay is a whole number of samples (2:1 and 1:2); other ratios are out of v1. The earlier rational-L rule is withdrawn |
| C-106 | Unit of the compiler's cost bound | counted abstract operations per slice, identical on every machine. Each target has a calibration factor (time per operation), measured once and recorded in the regime stamp; admission converts with it |
| C-107 | Commissioning tool activity | may drive test traffic only inside a declared commissioning window that every module stamps; read-only outside it |
| C-108 | Late versus missing | at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1) and discarded |
| C-109 | Demotion at the ratchet ceiling | an input that is persistently late with the offset already at A enters the health state Demoted: treated as gapped, counted separately, cleared only by re-admission |
| C-110 | Reassembly deadline for an aligned multi-packet input | the consumer's own deadline for that position (C-15 covers arrival-driven inputs only) |
| C-111 | Start of the dead threshold (C-39) | 2 s counted from the last slice received with content or a marked gap |
| C-112 | Accepted sync request rate | 10 to 200 per second; AC-7 is checked over this range |
| C-113 | Election when no candidate has usable time | the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch. "Usable time" is defined in C-213 |
| C-114 | Follower-cap test | with at most 64 modules a holder has at most 63 real followers; the 64-follower load and the refusal of a 65th (AC-25) are tested with synthetic followers |
| C-115 | Authentication reservation in v1 | scheme 0 and tag zero; the 64-bit counter is live because it is the liveness counter |
| C-116 | Path-MTU probe size | the probe pair is exempt from the control message bound C-17: it is sent at the intended media packet size |
| C-117 | Declared trunk capacity | per direction |
| C-118 | Loss soak | 24 hours at the two-bundle load (64 000 packets/s, about 5.5e9 packets); the statistic is aggregate loss over all streams, where the floor predicts about 58 losses |
| C-119 | L2 drift-ramp scenario | the injected crystal drift sweeps ±100 ppm at no more than 0.5 ppm/s; pass is E held throughout. The slew limits C-60 and C-61 bound the servo's correction term, not the drift estimate it tracks |
| C-120 | Test-only injection hooks, beyond the public injection API | force a module id and nonce; force a stream onto a given timeline epoch; send a raw packet on a media or control socket. Stamped like any injection |
| C-121 | Home of wall-clock step detection | the platform backend that builds each exchange from receive stamps; the servo only ever receives exchanges that passed it |
| C-122 | Degraded flag on an output with latency L | follows the content. Stamps denote content position: while a module processes input position p it emits the output stamped p − L, whose content came from input position p − L. That output is degraded if the input at p − L was gapped, so the flag is delayed by L together with the content |
| C-123 | Where the bounded class arises | from an exact-float fingerprint mismatch, and for hand-written (non-DSL) modules that do not claim exactness. The DSL has no non-deterministic node in v1 |
| C-124 | Laptop edge sleep | sleep is a departure: both timebase loops are reset and the edge returns through normal admission (Locking, Aligning) |
| C-125 | Definition-of-done applicability | a checklist line that has no subject in a task (no code, no parser, no stamp field) is marked not applicable with the reason; it is never silently skipped |
| C-126 | Epic scope labels | recomputed from the ticket tally, not estimated |
| C-127 | What strict mode requires | real-time scheduling granted on the data and clock threads, and memory locked. Restricting CPU idle states stays best-effort and is stamped |
| C-128 | Strict mode and an unverified capability | refused: unverified is treated as absent |
| C-129 | Capability states in the regime stamp | four: present, absent, unverified, not requested. "Not requested" marks a deliberate configuration choice, as distinct from a denial |
| C-130 | Where task outputs live | in the design set's repository, beside the design documents: specifications in `doc/design/spec/`, reports and results (archaeology, stamp probes, L1/L2/L3 results) in `doc/design/reports/`. The evidence appendix is `doc/design/06-evidence-appendix.md`; the constants table is `doc/design/02-constants.md`. Nothing is written into the frozen POC tree except by task 003 |
| C-131 | Receive-timestamp source values | kernel (hosted OS kernel stamp), userspace (clock read after receive), mac-hardware (Ethernet MAC timestamp register, embedded), interrupt (stamped in the receive interrupt, embedded). Each is reported with one of the four capability states of C-129 |
| C-132 | Evidence tag for a published standard | `[standard]`: a figure cited from a named, published standard, with the clause. It replaces `[memory]` once the citation is checked |
| C-133 | Where code lives | in the design set's repository: throwaway probes in `probes/`, harness and tool binaries in the `hadronyche-tools` crate, and all protocol types (timeline, slices, stamps, control messages) in `hadronyche-core` |
| C-134 | Where a resampler node may appear | only inside an edge module; the compiler rejects it anywhere else. An edge's conversion delay is part of its plumbing delay D and is never counted in L (C-207) |
| C-135 | A slice on another timeline epoch | restarts the dead clock (C-111), does not count as a miss (C-102), and has its own per-connection counter |
| C-136 | Holder stall | the holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch |
| C-137 | Header cap | a default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path |
| C-138 | Supported OS versions | set after the receive-stamp probe task (001) reports; the probes test the current release of each OS and the oldest obtainable one |
| C-139 | Housekeeping restart policy | exponential backoff from 100 ms to 10 s; after 5 panics within 60 s restarts stop: the module stays control-degraded with an alarm, its live streams continue, and it accepts no new connection until an operator restarts it |
| C-140 | Which certification state gives a numeric E | commissioned only. Uncertified and self-monitored sinks declare E as unknown (C-103) |
| C-141 | Transport state on a gap | the last known state is held and the playhead advanced as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo. The output is marked degraded |
| C-142 | Fan-in outside any presentation group | inherits the A of the downstream presentation group it feeds (the largest if several), learned at admission. With no downstream group it needs an explicit budget in configuration, or its aligned inputs are refused |
| C-143 | Oversized epics | an epic above 40 days by ticket tally is split into lettered parts (017a, 017b, …). A reference to a split epic's number means all its parts |
| C-144 | Owner of four orphan duties | the hosted backend (epic 015): emitting the gap slice when a module misses its own deadline; raising the processing-overrun event; the housekeeping-restart criterion AC-20 (c); the single-writer file service; sending the regime stamp message that epic 013 defines |
| C-145 | Additional test-only hook | inject a panic into a data, clock or housekeeping thread (extends C-120) |
| C-146 | Privileged tests | the few tests that need a privilege (for example stepping the wall clock) run in a separate privileged nightly job. The definition-of-done line "unprivileged" applies to per-change CI |
| C-147 | Strict mode and measurements | any run whose numbers enter this table as design constants runs in strict mode; L0 does not, and neither do POC runs, whose figures go only in the POC reference rows (P-nn). Strict mode is hosted-only: a device stamps real-time scheduling and memory lock as present by construction |
| C-148 | A for ladder runs (L1, L2, L3) and for the commissioning proposal | computed with the commissioning rule (largest over sinks of maximum lateness + D + headroom), using the maximum lateness since the connection went live, never the rolling 60 s figure of C-182; recorded with the results |
| C-149 | Which E the pass criteria mean | the platform's target E (C-70, C-71, C-72), never the E measured in the same run |
| C-150 | E budget split, macOS and Windows (proposals, pending L2) | macOS: 40 µs clock + 40 µs tracking + 20 µs uncertainty of D. Windows: 100 µs clock + 100 µs tracking + 800 µs uncertainty of D |
| C-151 | Machine pair for cross-machine bit-exactness | x86-64 and 64-bit ARM |
| C-152 | Peer verification of sync and control DSCP | the holder reports in its stamp the DSCP observed on each follower's sync requests; every module reports the DSCP observed on each peer's hellos |
| C-153 | Commissioning window life cycle | announced by the tool on the control group with an expiry time; modules stamp it while open; it closes at expiry or on explicit close. What it disqualifies is in C-178 |
| C-154 | Satisfied connections in the graph view | only Live. Pending, Locking, Aligning, Interrupted and Demoted are shown by name as unsatisfied |
| C-155 | Pacing fit | phase offset plus spread stay inside the slice period: the phase offset is in [0, 75 %) of the period |
| C-156 | The "cycle" of the drain bound (C-16) | one slice period |
| C-157 | Reclaimable latency | offset − (current maximum lateness + headroom), when positive; published in the stamp's alignment group |
| C-158 | Dense controller with no prior value | each dense controller declares an initial value in the manifest (default 0), used when a gap precedes any received value |
| C-159 | Per-connection counters in the stamp (completes C-47) | received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead |
| C-160 | Lateness percentiles in the stamp | 50th, 99th, 99.9th and the maximum |
| C-161 | Meaning of the control-isolation capability | housekeeping cannot delay the data path. Present on hosted platforms by construction of the thread model; absent or unverified on a device unless the implementer shows a bound |
| C-162 | Clock state after the holdover claim lapses (C-65) | Unlocked, with time unusable: one-way figures and alignment claims stop |
| C-163 | Last ratchet step | when the gap between offset and A is less than one slice period, the offset steps to A exactly |
| C-164 | Bound on processing one bad input (proposal, pending L1) | linear in packet length and at most 10 µs per packet |
| C-165 | Operator policy in the commissioning configuration | it sets strict mode, per-input alignment opt-out and pacing override |
| C-166 | L2 and L3 test parameters (proposals, pending first runs) | no-step tolerance at handover ±1 sample on Linux, the platform restart spread elsewhere; comparison windows of 10 s; injected loss 1 % random plus bursts of 8 slices; measurement window after a restart 60 s; restart spread of D is max − min; sample tolerances are measured against the median of the first 60 s; a one-sample slip passes on Linux and is reported as a slip event |
| C-167 | L1 test parameters (proposals, pending first runs) | short scenarios run 10 minutes; serialisation-time tolerance ±5 µs; the loss alarm C-91 is 10 × the measured clean-wire loss; holder serving lateness is reply send time minus request receive time at the 99.9th percentile, and "no growth" means within 20 % from 1 to 64 followers |
| C-168 | Liveness of a sink-only device | judged by the producer timeout (C-41). Interrupted and dead (C-38, C-39) describe streams a peer receives |
| C-169 | Alignment claim in holdover when E is unknown | the lapse rule C-65 needs a fixed reference, so an unknown E is taken as twice the lock bound (2 × C-62, that is 50 µs): the device stops claiming alignment when the bound it currently publishes exceeds that value, about 25 s into holdover at the growth of C-64 |
| C-170 | Definition of D | a sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter. For capture, the mirror: from acoustic instant to the sample being available |
| C-171 | More locations (extends C-130) | the architecture specification is `doc/design/01-architecture.md`; operator checklists and the guide are in `doc/operations/`; certificates are stored beside the collected stamps in the deployment's data directory |
| C-172 | Reference hardware and supported switch families | proposed by the implementing team (after P0 for audio hardware, in P7 for switches), approved by the product owner, recorded in this table |
| C-173 | Admission details | PathMtuTooSmall applies when the probe at the 1 280-byte fallback also fails. A hello with domain 0 is ignored and counted, not refused; ForeignDomain is for a different non-zero domain. A 16th refusal reason, BadMemberList, covers an unknown or duplicate bundle member or more than 32 members. A connection alarm is raised on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded |
| C-174 | "Cannot meet A" alarm | maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows |
| C-175 | "The interface is held" | the audio device cannot be opened in the required mode because another process has it open |
| C-176 | Calibration factor procedure (C-106) | the DSL conformance suite (epic 025) runs a reference workload once per listed target and records time per operation; the value is carried in the regime stamp |
| C-177 | Seqlock safety and a failed pin | if a thread layout's seqlock safety depends on a CPU pin and that pin fails, the module refuses to start in every mode, strict or not. Default priorities (clock above data) never depend on pinning |
| C-178 | What an open commissioning window disqualifies | ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified while a window is open. The commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in |
| C-179 | When measured loss reopens the redundancy decision | clean-medianet loss above 10 × the published floor (C-90), about 580 losses in the 24-hour soak (C-118), reopens it. Between 1 × and 10 × it is recorded and investigated as a cabling or host problem and does not |
| C-180 | Certification of a sink whose clock is unlocked | it stays commissioned; its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning |
| C-181 | Where controllers travel | a bundle source's controllers always ride in their own member stream (C-205). A single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit. The compiler checks this from the manifest |
| C-182 | Window of the stamp's lateness figures | percentiles (C-160), reclaimable latency (C-157) and the cannot-meet-A alarm (C-174) use a rolling 60 s window; the maximum since the connection went live is reported beside them |
| C-183 | Even-tap linear-phase FIR | rejected by the compiler in v1, with a diagnostic suggesting an odd-tap design, so that L stays exact |
| C-184 | Transport state before any has arrived | stopped, playhead at zero, tempo 120 beats per minute (C-199) |
| C-185 | Refusal reasons, the closed set of 18 | PortCeiling, PacketRate, SliceRate, InputCount, BufferMemory, CostBound, ContentMismatch, NotAnInput, VersionMismatch, ForeignDomain, PathMtuTooSmall, RequiredEExceeded, FollowerCap, ModuleCap, NameTaken, BadMemberList (C-173), ControlDegraded (the target's housekeeping has stopped restarting, C-139; answered by the housekeeping supervisor, which keeps a minimal refusal path) and NoSkewBudget (a fan-in with no downstream group and no configured budget, C-142). VersionMismatch, NameTaken and ModuleCap are raised at discovery, where they stop a module joining, not at admission |
| C-186 | A changed while inputs are live | every fan-in that inherited the old A (C-142) re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do |
| C-187 | Publisher of the clock lock state and bound | the servo core (epic 008a), through the published time view; every epic that reports one-way figures depends on it |
| C-188 | Where the holder answers sync requests | on the clock thread, never on housekeeping; this is also where it records the DSCP observed on each follower's requests (C-152) |
| C-189 | Two kinds of resampler | the DSL resampler node is fixed-ratio, 2:1 or 1:2, with whole-sample delay (C-105). An edge's rate actuator is a separate, steerable resampler whose ratio varies by parts per million around its nominal value to track the hardware crystal; its delay is part of D. The ABI keeps the "frequency-dependent delay" property on outputs, for recursive filters |
| C-190 | A join on the device side of an edge module | paths that differ by an odd number of domain-rate samples cannot be padded with a whole-sample delay line at the device rate; the compiler rejects the join and suggests the per-join opt-out |
| C-191 | L2 and L3 test parameters, completed (proposals, pending first runs) | loss bursts are spaced 1 s apart and hit the media path only (sync loss is its own scenario); the announced step is 5 ms; a 10 s comparison window starts when the injected event ends; the handover is triggered 60 s into the run; calibration repeatability is judged against the mean of the 10 calibrations; "both stamps" in a certification record are the regime stamps of the sink and of the holder it followed; an embedded device declares its own restart spread, measured at L3 |
| C-192 | Smaller settled points | float and fixed variants of a corpus program count as one program; the reference dense controller is the mixer's volume; the L0 baseline scenario is defined by epic 017b; the vectors crate sits at the workspace root and the fuzzing tool is the implementing team's choice |
| C-193 | Which specification supplies what to the conformance vectors | epic 004: wire layouts and reject reasons; epic 005: state tables and scenario scripts for discovery, election, admission and connection; epic 006: timeline and epoch scenarios |
| C-194 | The required-E refusal test at L3 (AC-30) | the group's required E is set below the device's own declared E, and the device is refused on its own figure whatever the other sink's E is |
| C-195 | Holdover entry delay | at most the election silence threshold (1.5 s, C-37): a hard limit, so a follower is always in holdover with a growing bound when it calls an election. The value within that limit is pending simulation (epic 008b proposes it; the POC used 2 s). If no value at or below 1.5 s works, that is reported as a finding |
| C-196 | Dependency hygiene | a specification epic never blocks on an epic of a later phase; such a link is a cross-reference, not a dependency |
| C-197 | Who builds the resamplers | the arithmetic kit (epic 024a) builds both the fixed 2:1 / 1:2 node and the steerable rate actuator, once. Epic 024a moves to phase P4, ahead of the Linux edge; every edge (Linux, macOS, Windows, embedded) uses the kit's and builds none of its own |
| C-198 | Carrier of a changed A upstream | the receiver report (C-40) carries the consumer's current A, its own or the one it inherited. A change propagates one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186) |
| C-199 | Default tempo | 120 beats per minute |
| C-200 | Playhead through a gap while stopped | does not advance (see C-141) |
| C-201 | An unverified pin and seqlock safety | treated as failed for C-177: the module refuses to start in every mode |
| C-202 | Sync loss at L2 | no separate scenario. The handover run gains one assertion: emission error stays within the target E while the sink's clock is in holdover between holders. The loss scenario stays on the media path |
| C-203 | Tolerance for a pair of different sinks | the larger of the two sinks' own no-step and recovery tolerances; both sinks' figures are reported beside each verdict |
| C-204 | Sync requests above the accepted rate | requests beyond 200 per second from one follower (C-112) are dropped before any work is done and counted per follower in the holder's stamp |
| C-205 | Controllers of a bundle source | always in their own member stream |
| C-206 | Presentation details of tools | output line formats, panel layouts and file names within the stated directories are decided by the epic that builds the tool and recorded in its documentation. Only what another epic or an operator's script depends on goes into the specifications |
| C-207 | Resampler delay and L | an edge's resampler delay is counted only in D and never in L. L is the module's structural delay in the domain-rate timeline; the manifest reports the conversion delay separately, on the device-facing port |
| C-208 | Who picks the re-alignment position after a changed A | each fan-in that learns of the change picks its own position, one lead (C-43) ahead, and announces it in its own output, exactly as for a ratchet-triggered re-alignment. Fan-ins at different depths switch at different, announced positions |
| C-209 | Controller member and the bundle cap | the controller member counts towards the 32 of C-08: a source with controllers has at most 31 audio members (186 channels). A wider source with controllers is rejected at compile time |
| C-210 | Home of the rate-actuator trait | defined in the kit epic (024a) together with its implementation, with a ratio range of ±1 000 ppm around the nominal ratio; every edge consumes it |
| C-211 | A larger than a fan-in's buffer memory | the commissioning tool checks declared buffer memory before applying a change of A and refuses one that does not fit. If one reaches a fan-in anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running |
| C-212 | How the L2 and L3 handover is triggered | as a holder loss: the old holder stops answering, the sinks pass through about 2 s of holdover (C-37 plus C-33) and the new holder takes over. At L2 the run asserts each sink's emission error within the target E during holdover (C-202) and frequency continuity at takeover (rate change within C-61, no phase step). At L3 the rig sees only the difference between the sinks, so it asserts what C-220 states. A run in which holdover was not observed is failed, not passed. The rig therefore needs a second holder-eligible Linux node |
| C-213 | Who may take over as holder | a holder-eligible module whose time is usable: Locked, or in holdover with its alignment claim not lapsed (C-65, C-169). Such a candidate takes over with frequency continuity and the timeline epoch is kept. The epoch is bumped (C-113) only when no candidate has usable time |
| C-214 | Where a sink switches to a changed A | at its own position, one lead (C-43) ahead, as a fan-in does (C-208); it has nobody to announce it to |
| C-215 | A ratchet step | is a re-alignment like any other: it takes effect at a position the fan-in announces one lead (C-43) ahead in its own output, never silently on the next round |
| C-216 | A clamped A (C-211), in detail | the cannot-meet-A alarm stays raised until a later change of A that fits; the fan-in's receiver report (C-198) carries the A it was given, not the clamped value, so upstream modules size themselves to what the sinks need |
| C-217 | Peer map capacity | 63 peers: a medianet has at most 64 modules (C-34) and a module does not hold an entry for itself |
| C-218 | Who implements a compile-time rejection | the DSL front end (epic 023) implements every compile-time rejection; the code generator (epic 024b) only asserts it never receives such a program; the conformance corpus (epic 025a) tests it |
| C-219 | Wire cost of a controller stream | computed like an audio stream: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s. Admission and the commissioning tool use this same figure |
| C-220 | What L3 asserts through a handover | the skew between the two sinks stays within the sum of their E values (C-79, C-202) through holdover and takeover, and each sink's frequency is continuous at takeover. Per-sink emission error against the timeline is an L2 assertion only: the L3 rig cannot observe it |
| C-221 | Several candidates with usable time (C-213) | a Locked candidate is preferred over one in holdover; among equals the lowest module id wins |
| C-222 | Who the clamp of C-211 applies to | any consumer that buffers to meet A, sinks included; C-216 applies to a sink in the same way |
| C-223 | The 50 ppm frequency difference of the handover test | at L2 (C-78): the measured natural difference between the two holders' crystals plus an injected drift, totalling 50 ppm; both parts are recorded in the run report. In simulation the whole 50 ppm is injected. The L3 handover injects none and records the natural difference |
| C-224 | How a commissioned change of a sink's own A reaches the sink | through the commissioning tool, inside a declared window (C-153): the tool validates buffer memory first (C-211) and then announces the new A to the group's members |
| C-225 | A ratchet step in a sink | a sink has no output to announce in, so it takes the step at its own position one lead (C-43) ahead, as it does for a changed A (C-214) |
| C-226 | Ratchet ceiling of a sink | A − D: a sink hands a position to its interface at T(p) + A − D (C-170), so its offset can grow to A − D and no further; a fan-in's ceiling stays A (C-53, C-55). At the ceiling a persistently late input is Demoted (C-109) |
| C-227 | Which node wins the rig handover | after the holder loss every candidate is in holdover, so C-221 picks the lowest module id: the L2 and L3 harnesses give the second holder node the lowest id among the rig's holder-eligible modules through the test hook (C-120), and stamp it |
| C-228 | Who encodes the commissioning-window messages, the announcement of a changed A included (C-153, C-224) | epic 004 owns the field layout and epic 005 the state machine, as for every control message; epic 013 owns only what the regime stamp records of the window |
| C-229 | The since-live maximum lateness (C-148, C-182) across a re-alignment | it is kept: lateness is measured against the timeline, not against the offset, so a re-alignment does not reset it. Only a new admission of the connection does |

## POC reference values (not design constants)

| ID | Fact | Value | Tag |
|---|---|---|---|
| P-01 | POC default feed | 48 channels, about 9.6 KB per datagram, 7 IP fragments at MTU 1500, about 150 Mbit/s | `[code]` |
| P-02 | POC sync rate achieved at `--sync-hz 50` / `1000` (and at `10` / `200`, the ends of the C-112 range) | 36.1 / 71.4 completed exchanges per second per follower (at `10` / `200`: 9.34 / 71.5), duration-corrected; POC figures from clean single-writer metrics files of unprivileged `poc-untouched` runs without strict mode (C-147), 3 runs of 35 s per rate, CONFIG_HZ=250 (P-03); they replace the figures read from the corrupt metrics file (evidence appendix M-1; REPORT-0b section (e)) | `[measured]` |
| P-07 | POC sync rate achieved after the task 003 fixes, at `--sync-hz 50` / `1000` | 49.86 / 944.77 completed exchanges per second per follower, duration-corrected; measured by ticket 003-05 at POC commit 4a4885e (one 25 s loopback run per rate, two runs in all, unprivileged, without strict mode, C-147), holding at the tag `poc-baseline` because `git diff --stat 4a4885e poc-baseline` touches only the end of `main()` in `src/main.rs`, `tools/smoke.sh` and `BASELINE.md` `[code]`; machine as BASELINE.md states it: host hackrobustus, CPU i7-1365U, kernel 6.12.115+deb13.1-amd64, `CONFIG_HZ=250` (P-03); before the fixes: P-02 | `[measured]` |
| P-03 | Kernel tick on the measuring machine | `CONFIG_HZ=250` | `[measured]` |
| P-04 | POC lock threshold | smoothed error below max(20 µs, path delay / 4), after at least 8 windows; a servo already Locked stays Locked while the smoothed error is below 4 × that threshold (exit hysteresis, `clock.rs:375-376` at `poc-untouched`) | `[code]` |
| P-05 | POC holder rotation | after 2 mandates (retired) | `[code]` |
| P-06 | POC id | 31 effective bits from wall time, pid and monotonic time (retired) | `[code]` |
