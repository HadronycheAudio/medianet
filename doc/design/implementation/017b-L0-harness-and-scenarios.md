# 017b — L0 harness and scenarios

**Phase:** P4 · **Scope:** L (26 tickets, about 37 days) · **Depends on:** 017a, 007, 008a, 009, 010, 012, 013, 014, 015, 016 · **Blocks:** 019, 020, 025, 027, 029

## Goal

Build the **L0 loopback harness** around the reference modules of epic 017a: a public ground-truth injection API, the test-only hooks, and the `hadronyche-l0` binary with its 15 scenarios — one process, ground-truth injection, hard assertions. L0 is the regression gate that runs unprivileged in CI on every change. This is the second of the two parts of epic 017 (C-143).

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's, over IPv4 only, with multicast at TTL 1 (C-100). Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline (`position = (network_time − epoch_origin) × rate`, where `rate` is the domain rate of 96 kHz, the only rate on the wire, C-01). Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers. L0 runs on the IPv4 loopback interface.

**Split of epic 017 (C-143).** Epic 017 exceeded 40 days by ticket tally and is split into 017a (the reference modules) and 017b (this part). This part depends on 017a: its scenarios drive the reference sender, echo and mixer, and its runner uses 017a's analyzer and per-family sample counts. A reference elsewhere to "epic 017" means both parts.

**What L0 drives (from 017a).** The reference sender emits a parts stream and a 6-channel audio feed and runs in-process replicas of the echo and mixer; the reference echo is arrival-driven with a single input; the reference mixer is a position-aligned fan-in that fires at `T(p) + offset`, zero-fills a missing input and marks the output degraded. Verification tiers: transport integrity (slice checksum), exact (the sender's replica runs the same code and compares bits), tolerance (analyzer windows and per-voice projections). Windows touched by loss or by a degraded slice are excluded from the exact tier and counted. Each module declares a whole-sample structural latency L (C-105), and the degraded flag follows the content: the output stamped q carries the content of input position q and is degraded if that input was gapped (C-122). On a gap, the last known transport state is held and the playhead advanced as that state dictates: a playing transport moves on at its tempo, a stopped one does not advance (C-141, C-200); before any transport state has arrived it is stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199). The reference dense controller is the mixer's volume (C-192), so the scenarios that drive the mixer exercise the dense-controller domain; how the volume is emitted is epic 017a's. The mixer feeds the reference sender, not a presentation group, so every scenario gives it an explicit alignment budget that serves as its A (C-142).

**POC references `[code]`.** Ground truth in the POC: `--sim-offset-ms`, `--sim-drift-ppm` (`src/main.rs:162`, `:165`) inject a simulated crystal per module; loopback has `--duration` and `--csv`. The new API states drift in parts per billion: a POC value in ppm is multiplied by 1 000 when reused. A 25-second POC loopback that looked green `[measured]` had a corrupt metrics CSV (four `File::create` handles on one path, `module.rs:1354`), a "lost" counter random-walking around 21.7 million with zero actual loss, and music and transport check families reporting "0 of 0 windows".

**Gap classes L0 asserts `[decision]`** (implemented by task 009).

- Late versus missing: at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified late (missing −1, late +1) and discarded (C-108).
- Interrupted: after 8 consecutive missed slices (C-38). Only slices absent at the deadline count as misses — missing, or arriving late; a marked gap or a slice on another timeline epoch proves the upstream alive and resets the count (C-102).
- Dead: 2 s (C-39) counted from the last slice received with content or a marked gap (C-111). A slice on another timeline epoch also restarts the dead clock, does not count as a miss, and has its own per-connection counter (C-135).
- Persistently late: at least one late slice in each of 10 consecutive 1 s windows (C-101); this triggers a ratchet step of one slice period (C-53, C-54); when less than one slice period remains below A, the offset steps to A exactly (C-163). An input persistently late with its offset already at A enters the health state **Demoted**: treated as gapped, counted separately, cleared only by re-admission (C-109). (The earlier wording "a flapping input" denotes this case.)
- A connection whose target is not present stays Pending, shown as unsatisfied, with no timeout (C-45). Only Live connections are shown satisfied (C-154).
- A connection alarm is raised on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded (C-173).

**Rules for L0 `[decision]`.**

- **A check family with zero samples is a failed run.** The run exits non-zero and names the inert family.
- **No assertion depends on sync-rate timing.** The POC achieved 37 exchanges/s at a configured 50 and 77/s at 1 000 on a 250 Hz kernel `[measured]` (P-02, P-03); rates and timeouts are not test oracles. The accepted sync request rate is 10 to 200 per second (C-112), and AC-8 checks invariance over that range.
- Never assert latency thresholds on loopback; assert content, loss accounting, lock, classification and ground-truth error. Programmed delays (such as the housekeeping restart backoff) are asserted only as lower bounds.
- **Ground truth for loss accounting.** The injector is seeded and deterministic and logs every decision, so loss accounting is asserted as exact equality with the injector's log, not as a statistical tolerance. The runtime's figures are read from the per-connection counters that producer and consumer each publish in their regime stamp (C-47): received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), with the lateness percentiles (50th, 99th, 99.9th and maximum, C-160) over a rolling 60 s window, the maximum since the connection went live reported beside them (C-182), and the observed DSCP. The holder's stamp also counts, per follower, the sync requests dropped above 200 per second (C-204); at L0's configured rates (10 to 200 per second) that count is zero.
- **DSCP verification (C-152).** The holder reports in its stamp the DSCP observed on each follower's sync requests; every module reports the DSCP observed on each peer's hellos. L0 checks these against the classes of C-13. The holder answers sync requests, and records their DSCP, on its clock thread, never on housekeeping (C-188).
- **Published offset bound.** "Within the published bound" means within the lock bound, 25 µs (C-62, pending simulation); assertions cite C-62 by id so they follow any change. The lock state and the published bound are read from the servo core's published time view (epic 008a, C-187). The slew limits C-60 and C-61 bound the servo's correction term, not the drift estimate it tracks (C-119), so injected drifts of tens of ppm are legitimate.
- Loopback's MTU is 65 536, so it never exercises small packets by itself: L0 must force a small configured path MTU to exercise the multi-packet regime (stamped, C-05) and the 1 280-byte fallback (C-06), which leaves 1 252 bytes of UDP payload (C-17, C-100); a default slice fits that payload with at most 100 bytes of headers and checksum (C-137); controllers sharing a single-stream source's audio packet count inside that cap, beyond it the source emits a second stream for them, and a bundle source's controllers always ride in their own member stream (C-181, C-205). The path-MTU probe pair is sent at the intended media packet size, exempt from C-17 (C-116). PathMtuTooSmall applies when the probe at the fallback also fails (C-173). `drop_above_bytes` compares the IPv4 packet size, the same quantity as a path MTU.
- Injection is public runtime API and is always written into the regime stamp. Every injection call is a regime change: the stamp version carried in the hello changes, the stamp is multicast to the control group, and each stamp section, injection record included, fits the control message bound of 1 252 bytes (C-17, C-18). Sending the stamp message is the hosted backend's duty (task 015, C-144).
- **Test-only hooks** beyond the public injection API (C-120, C-145): force a module id and instance nonce (64 and 64 bits, C-11); force a stream onto a given timeline epoch (16 bits, C-10); send a raw packet on a media or control socket; inject a panic into a data, clock or housekeeping thread. They are stamped like any injection. They are how L0 places an input on another epoch (AC-18), forces an id collision (AC-22), injects malformed, wrong-version and foreign-domain packets (AC-23) and exercises the housekeeping restart policy.
- A clock step injected with `step_clock` is applied to every time the module reads, receive stamps included, so the platform backend's wall-clock step detection sees it and the servo only receives exchanges that passed it (C-121).
- L0 does not use the simulation's provisional jitter profile (C-94); its delays are the injector's seeded uniform ranges and per-hop delays.
- **L0 never runs in strict mode (C-147).** Strict mode requires real-time scheduling on the data and clock threads and locked memory (C-127), and refuses an unverified capability (C-128); unprivileged CI has neither, and L0's numbers never enter the constants table. L0 configures those capabilities as "not requested", and the stamp records each in one of the four states present, absent, unverified, not requested (C-129). The receive-timestamp source in use (kernel, userspace, mac-hardware or interrupt) is stamped with its capability state (C-131). Operator policy (strict mode, alignment opt-out, pacing override) is set in the commissioning configuration (C-165), which the runner builds per scenario. Non-strict does not relax seqlock safety: if a thread layout's seqlock safety depends on a CPU pin and that pin fails or is unverified, the module refuses to start in every mode, L0's included (C-177, C-201), and such a refusal fails the run; default priorities (clock above data) never depend on pinning.
- **Privileged tests (C-146).** Tests that need a privilege run in a separate privileged nightly job; "unprivileged" in the definition of done applies to per-change CI. L0 has none: clock steps are simulated.
- **Duties owned by task 015 (C-144).** The hosted backend emits the gap slice when a module misses its own deadline, raises the processing-overrun event, implements the housekeeping-restart criterion AC-20 (c), provides the single-writer file service (which the L0 runner uses for its record file), and sends the regime stamp message. L0 observes these; it does not implement them.
- **Where outputs live (C-130, C-133, C-171).** The `hadronyche-l0` binary is a harness binary and lives in the `hadronyche-tools` crate; the injection API in `hadronyche-std`. The resynchronisation-bound measurement goes to `doc/design/reports/`; evidence to `doc/design/06-evidence-appendix.md`; new constants to `doc/design/02-constants.md`. Nothing is written into the frozen POC tree. Evidence tags include `[standard]` for a figure cited from a named, published standard with its clause (C-132). The runner's presentation details (output line formats, file names within the stated directories) are this part's decision, recorded in its documentation; only the record-file format, which epics 019, 020 and 027 read, goes into the specifications (C-206).
- Fuzzing budget for every touched parser: 60 s on every change and 1 hour nightly, corpus kept (C-95). The parsers this part touches are the injection record in the stamp sections and the L0 record-file reader; the reference modules' decoders are fuzzed in 017a.

**Diamond topology.** The "processor" of AC-10 is the reference echo, which reproduces the POC's own topology: the sender's audio feed reaches the mixer directly, and the sender's parts reach the mixer through the echo. Per-hop delay is injected on the echo; the offline reference is the mixer's offline function over the feed and the offline rendering of the same parts.

**Scenarios whose behaviour is now stated.**

- `flood`: an input offered more packets per slice than it was admitted for is drained by at most 2 × its admitted packets per slice per cycle (C-16), the cycle being one slice period (C-156); other inputs are unaffected.
- `refusal`: a connection that would take a declared per-direction capacity (C-117) above the 70 % admission ceiling (C-23) is refused with a named reason; a sink with no certified E (only a commissioned sink has a numeric E, C-140) declares E unknown and is refused with reason RequiredEExceeded from a presentation group that sets a required E, and admitted to one that sets none (C-103); a connection to an absent target stays Pending (C-45); a fan-in with no downstream presentation group and no explicit budget has its aligned inputs refused with NoSkewBudget (C-142, C-185); an unknown or duplicate bundle member, or more than 32 members counting the controller member, is refused with BadMemberList (C-173, C-209); every refusal raises a connection alarm (C-173).
- `thread-death`: a stalled data plane is visible to peers within 2 hello periods (C-42); consumers then classify its streams interrupted (C-38) and dead 2 s after the last slice with content or a marked gap (C-39, C-111).
- `housekeeping`: injected housekeeping panics (C-145) restart with exponential backoff from 100 ms to 10 s; after 5 panics within 60 s restarts stop, the module stays control-degraded with an alarm, its live streams continue, and a new connection to it is refused with ControlDegraded, answered by the housekeeping supervisor's minimal refusal path (C-139, C-185); content is undisturbed throughout, since housekeeping cannot delay the data path on hosted platforms (C-161), and if the module is the clock holder its followers stay locked, since sync requests are answered on the clock thread (C-188).
- `malformed`: a hello with domain 0 is ignored and counted, not refused; ForeignDomain is the reason for a different non-zero domain (C-173). VersionMismatch is raised at discovery, where it stops a module joining, not at admission (C-185). Reject reasons are those of epic 004's wire layouts, supplied through epic 007's vectors (C-193). The bound on processing one bad input (C-164) is pending L1 and not asserted.

**Constants used.** Domain rate 96 kHz, the only rate on the wire (C-01); slice 48 frames = 0.5 ms = 2 000 slices/s (C-02); 4-byte samples (C-03); 6 channels per stream, 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte (C-04); single-packet default slice, multi-packet regime stamped (C-05); path-MTU fallback 1 280 bytes (C-06) with 1 252 bytes of UDP payload, IPv4 only (C-17, C-100); members per bundle at most 32, the controller member included (C-08, C-209); stream number, session and epoch widths (C-10); module id, nonce and domain id widths (C-11); DSCP classes (C-13); reassembly deadlines (C-15, C-110); bounded drain 2 × admitted packets per slice (C-16) per slice period (C-156); stamp transport (C-18); session number ownership (C-19); per-stream wire rate about 21.1 Mbit/s (C-21); admission ceiling 70 % (C-23); declared capacity per direction (C-117); hello 1 s (C-30), hello expiry 3 s (C-31), election window 400 ms (C-33); interrupted after 8 consecutive missed slices (C-38, C-102); dead after 2 s (C-39, C-111); foreign-epoch slices (C-135); stalled data plane visible within 2 hello periods (C-42); announcement lead 64 slices (C-43), re-issue every 16 (C-44); Pending without timeout (C-45); collision quarantine by third parties for 3 s (C-46); per-connection counters in the stamp (C-47, C-159); lateness percentiles (C-160); aligning 4 s (C-50); offset estimator and headroom (C-51, C-52); ratchet one slice period, ceiling A (C-53), after 10 s dwell (C-54) expressed as persistent lateness (C-101), last step to A (C-163); A per presentation group, no default (C-55), fan-in budget (C-142); lock bound 25 µs (C-62, pending simulation); step threshold 2 ms (C-67); fuzz budget (C-95); rings sized to A (C-96); sink without certified E (C-103, C-140); whole-sample L (C-105); late versus missing (C-108); Demoted (C-109); sync request rate 10 to 200/s (C-112); hello counter as liveness counter (C-115); probe pair at media packet size (C-116); test-only hooks (C-120, C-145); step detection in the platform backend (C-121); degraded flag with latency (C-122); strict mode and capability states (C-127, C-128, C-129, C-147); output locations (C-130, C-133, C-171); receive-timestamp source (C-131); `[standard]` tag (C-132); header cap (C-137); housekeeping restart policy (C-139); transport state on a gap (C-141); duties of task 015 (C-144); privileged tests (C-146); DSCP verification (C-152); satisfied connections (C-154); control isolation (C-161); bad-input bound, pending (C-164); operator policy (C-165); admission details and alarms (C-173); seqlock safety and a failed or unverified pin (C-177, C-201); controllers within the header cap or in their own stream (C-181, C-205); rolling 60 s window of the lateness figures (C-182); transport before any has arrived and default tempo (C-184, C-199); refusal reasons ControlDegraded and NoSkewBudget, discovery-time reasons (C-185); publisher of lock state and bound (C-187); sync answered on the clock thread (C-188); reference dense controller and L0 baseline (C-192); source of reject reasons (C-193); playhead stopped through a gap (C-200); over-rate sync requests dropped and counted (C-204); presentation details of tools (C-206).

## Deliverables

Crate `hadronyche-std`, module `inject.rs` (public API; shape, names may be refined):

```rust
pub struct ClockInjection { pub offset_ns: i64, pub drift_ppb: i64 }

pub struct LinkInjection {
    pub seed: u64,
    pub loss: f64,              // probability per packet
    pub delay_ns: (i64, i64),   // uniform range added per packet
    pub reorder: f64,           // probability a packet is held one slot
    pub drop_above_bytes: Option<usize>,  // MTU blackhole, IPv4 packet size
}

pub trait Injector {
    fn set_clock(&self, module: ModuleHandle, inj: ClockInjection);
    fn step_clock(&self, module: ModuleHandle, step_ns: i64);
    fn set_link(&self, from: ModuleHandle, to: ModuleHandle, inj: LinkInjection);
    fn set_hop_delay(&self, module: ModuleHandle, delay_ns: i64);
    fn partition(&self, groups: &[&[ModuleHandle]]);
    fn suppress_hello(&self, module: ModuleHandle, on: bool);
    fn kill_thread(&self, module: ModuleHandle, role: ThreadRole);
}
```

Test-only hooks (C-120, C-145), outside the public API (behind a test-only feature; shape, names may be refined), stamped like any injection:

```rust
pub trait TestHooks {
    fn force_identity(&self, module: ModuleHandle, id: u64, nonce: u64);
    fn force_epoch(&self, stream: StreamHandle, epoch: u16);
    fn send_raw(&self, from: ModuleHandle, socket: SocketKind, to: std::net::SocketAddrV4, bytes: &[u8]);
    fn inject_panic(&self, module: ModuleHandle, role: ThreadRole);
}
```

Binary `hadronyche-l0` (in crate `hadronyche-tools`, C-133): runs a named scenario for a duration, writes one record file with its stamp through task 015's single-writer file service (C-144), exits non-zero on any failed assertion. Scenarios: `baseline`, `diamond`, `loss`, `late-vs-lost`, `gap-classes`, `realign`, `epoch`, `blackhole`, `flood`, `refusal`, `collision`, `malformed`, `partition`, `housekeeping`, `thread-death`.

Fuzz targets for the injection record and the record-file reader; the CI job that runs the suite on every change.

## Dependencies

- **Needs:** the reference modules and analyzer (017a); conformance vectors (007); the clock lock state and published bound, through the servo core's time view (008a, C-187); alignment ring and gap classes (009); packetiser and announcements (010); admission and connection states (012); stamp (013); receiver reports and per-connection counters (014); the hosted runtime, its single-writer file service, stamp sending and restart policy (015, C-144); real-time setup and thread roles (016).
- **Provides:** the injection API and record-file format for L1 (019), L2 (020) and L3 (027); the CI gate for every later task, including task 025.
- **Between parts:** 017b depends on 017a.

## Acceptance criteria

1. **AC-8 (trustworthy smoke).** With injected offset, drift and packet loss: zero content mismatches on the exact tier outside excluded windows; estimated minus true offset within the lock bound (C-62); packet and slice loss, as published in the producer and consumer stamps (C-47, C-159), equal to the injector's logged drops; no assertion changes when the sync rate is changed from 10 to 200 per second (C-112), and the holder's per-follower count of over-rate sync requests stays zero at both ends (C-204); the stamps report the observed DSCP of sync requests, hellos and media as the classes of C-13 (C-152).
2. **AC-9.** A scenario whose transport or music family produces zero samples exits non-zero and names the family.
3. **AC-10 (diamond).** Source feeds the mixer directly (audio feed) and through the reference echo (parts) with injected per-hop delay on the echo; mixer output equals the offline reference bit for bit.
4. **AC-15.** Injected delay beyond the offset but within the ring span is counted *late* only, after reclassification from missing (C-108); injected drop is counted *missing* only.
5. **AC-16.** A marked gap propagates with the degraded flag (C-122); interrupted after 8 consecutive missing or late slices, a marked gap or another-epoch slice resetting the count (C-38, C-102); dead 2 s after the last slice with content or a marked gap (C-39, C-111); resume from interrupted is immediate at the frozen offset; an alarm is raised on interrupted and on dead (C-173).
6. **AC-17.** Descendants switch at the announced position (lead 64 slices, re-issue every 16, C-43, C-44); after persistent lateness (C-101) the ratchet steps one slice period and stops at A, the last step landing on A exactly (C-53, C-55, C-142, C-163); an input persistently late with its offset at A is Demoted: treated as gapped, counted separately, cleared only by re-admission (C-109), with a demoted alarm (C-173).
7. **AC-18.** An input forced onto another timeline epoch (C-120) is treated as a gap until it switches, without counting towards interrupted (C-102), without becoming dead however long it lasts, and counted in its own other-epoch counter (C-135).
8. **AC-19 (L0 part).** With `drop_above_bytes` set, the probe pair, sent at the intended media packet size (C-116), detects the blackhole at admission and the connection goes live at a 1 280-byte path MTU, that is at most 1 252 bytes of UDP payload (C-06, C-100, C-137); with the blackhole below 1 280 bytes the connection is refused with PathMtuTooSmall (C-173).
9. **AC-22.** A collision forced with the identity hook (C-120) is detected within 1 s, the later joiner redraws, both modules log it, and a third party quarantines the id until only one instance nonce has been seen for 3 s (C-46).
10. **AC-23.** Malformed, wrong-version and foreign-domain packets, sent with the raw-send hook (C-120), cause no state change and are counted per input and reason; version mismatch is a named event; a domain-0 hello is ignored and counted, a different non-zero domain is counted as ForeignDomain (C-173).
11. **AC-36.** After an injected partition heals: one holder within 1.4 s (400 ms + 1 s); the losing side receives a timeline epoch change and restarts streams at an announced position with a bumped session number (C-19).
12. **AC-31.** Every record file carries a complete stamp (fields as defined by task 013, transported per C-18, including the per-connection counters of C-47 and C-159, the lateness percentiles of C-160 over the rolling 60 s window with the maximum since live beside them (C-182), the holder's per-follower over-rate sync request counts (C-204), the observed DSCP values of C-152, the capability states of C-129 and the receive-timestamp source of C-131); injection and test-hook use are labelled; one writer per file, through task 015's single-writer file service (C-144).
13. **AC-017.13 (local).** The whole L0 suite runs unprivileged in under 10 minutes of CI time.
14. **AC-017.14 (local, flood).** An input offered more than its admitted packets per slice is drained by at most 2 × its admitted packets per slice per slice period (C-16, C-156); every other input keeps zero exact-tier mismatches.
15. **AC-017.15 (local, refusal).** A connection exceeding 70 % of the declared per-direction capacity is refused with a named reason and never goes live (C-23, C-117); a sink with no certified E is refused with RequiredEExceeded by a group that sets a required E and admitted by one that sets none (C-103, C-140); a connection to an absent target stays Pending and unsatisfied for the whole run (C-45, C-154); a fan-in with no downstream group and no budget has its aligned inputs refused with NoSkewBudget (C-142, C-185); an unknown or duplicate bundle member is refused with BadMemberList (C-173); every refusal raises a connection alarm (C-173); existing streams are unaffected.
16. **AC-017.16 (local, thread-death).** Killing a module's data-plane thread is a named event and is visible to peers within 2 hello periods (C-42); its consumers go interrupted and then dead 2 s after its last slice (C-38, C-39, C-111).
17. **AC-017.17 (local, housekeeping).** Injected housekeeping panics (C-145) restart no earlier than the backoff from 100 ms (capped at 10 s); after 5 panics within 60 s restarts stop, the module stamps control-degraded with an alarm, its live streams keep zero exact-tier mismatches, and a new connection to it is refused with ControlDegraded (C-139, C-161, C-173, C-185); if it is the clock holder, its followers stay locked throughout (C-188).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The ported modules interoperate bit-exactly with their own replicas on two different machines (float and fixed)
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it

**Not applicable in this part (C-125):** "The ported modules interoperate bit-exactly with their own replicas on two different machines (float and fixed)" and "Each ported POC piece cites its source file and the archaeology finding that cleared it" — this part ports no POC code and runs no two-machine check; both belong to epic 017a. Every other line has a subject: the part writes code and tests, touches two parsers, adds stamp fields (injection and test-hook records), changes design documents and must leave the frozen POC untouched.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Resynchronisation bound after loss (inherit 3 s?) | This part (ticket 017b-11), from task 002 findings and the `loss` scenario's measurement | A loss scenario shows divergence lasting longer |
| `ModuleHandle`, `StreamHandle`, `ThreadRole`, `SocketKind` and the module API | Tasks 015 and 016 | The injection API, the hooks or the scenarios cannot be written against them |
| Sync-rate configuration mechanism used to run AC-8 at 10 and 200 per second | Task 015 | AC-8's sync-rate invariance cannot be run without code changes |
| When an arrival-driven module (the echo) emits the degraded output for an input position that never arrives | Task 009 | Consumers downstream of the echo count interrupted while the parts source is alive |
| Meaning of "one slot" in `LinkInjection::reorder` | This part (ticket 017b-03), recorded in the constants table | A reorder test passes under one reading and fails under another |
| Record-file format (on top of task 013's stamp encoding), the one runner output other epics read, so it goes into the specifications (C-206) | This part (ticket 017b-09) | A later rung's tooling cannot read an L0 record file |

The former question of which reference module exercises the dense-controller domain is settled: the mixer's volume (C-192).
