# 014 — Measurement path

**Phase:** P3 · **Scope:** L (25 tickets, about 24 days) · **Depends on:** 004, 005, 007, 008a, 009, 010, 012, 013 · **Blocks:** 015, 017, 019, 020, 026, 028, 029

## Goal

Implement in `hadronyche-core` (module `measure`) the reverse unicast measurement path and the lineage extension: receiver reports (one per bundle every 100 ms), one-way probe request and reply, the per-slice measurement extension carrying hop count, accumulated residence time and accumulated declared delay, and the per-connection summary counters each end publishes in its regime stamp. The receiver report is also the producer's only evidence that its consumer is alive.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules (sources, processors, sinks) exchange *slices* addressed by *position* on a domain-wide timeline. Every stream on the wire is at the single domain rate of 96 kHz (C-01); an edge whose hardware runs at another rate converts at the edge (C-80); a resampler node may appear only inside an edge module, and its conversion delay is part of the edge's plumbing delay D (C-134, C-170). IP is IPv4 only in v1 (C-100). Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers.

**Evidence tags.** `[code]` read in the POC · `[kernel-doc]` documented · `[memory]` recalled, unverified · `[measured]` measured · `[standard]` cited from a named, published standard with the clause; it replaces `[memory]` once the citation is checked (C-132) · `[decision]` design choice.

**System criteria and invariant used here.** This epic relies on the following wording only:

- AC-1: every message and extension passes its test vectors in both directions (encoding the vector values gives the vector bytes; parsing the vector bytes gives the vector values).
- AC-15: injected delay beyond the consumer's offset appears in the report's late count only; injected drop appears in the missing count only (refined by C-108, see "Receiver report content").
- AC-21 (producer side): with receiver reports withheld, the producer stops at 2 s; with reports flowing and gossip fully suppressed, it never stops.
- AC-34: the receiver report carries the observed DSCP of received media; a sender marking 46 is reported as 46.
- AC-37: an induced socket overflow appears in the report's drop counters on Linux.
- L-1: an admitted, healthy connection becomes dead only from data-plane evidence.

### Why a separate path `[decision]`

Media packets carry **no per-receiver state**, so the same bytes can serve several consumers and the wire stays ready for later fan-out mechanisms. That removes the POC's in-band round-trip closure, so measurement needs its own path. The admission handshake carries a **reverse unicast port** for it.

`[code]` Retired POC behaviour: the stream header carries timing measurements in every packet (88-byte header with `t1`, `t2k`, `t2`, `t3`, `ack_t4`, `tx_late`, `rx_lost`, `drift_ppb`, `servo_err`, `path_delay`, `proto.rs:147-185`), fields are overloaded by direction (`ack_seq` is the forwarding module id in echoes; `ack_t4` a starved-samples count; `path_delay` the sample rate master-to-follower), and forward packets carry the acknowledgement of that receiver's last echo (`F_ACK`).

### The v1 message set is exactly two `[decision]`

| Message | Sender | Rate | Content | DSCP |
|---|---|---|---|---|
| Receiver report | the consumer's **data thread** | one per bundle every 100 ms (C-40) | per-member stream number, session, received, late, missing, marked-gap, other-epoch, too-early, duplicate, incomplete-reassembly, demoted-round and interrupted counts (C-159) and input health; the consumer's current A (C-198); socket and interface drop counters; observed DSCP of received media | CS5 (40) |
| One-way probe request / reply | either end | on demand | timestamps closing a round trip | EF (46), because probes measure the media path |

Nothing else travels on this path in v1. Receiver reports are unicast to the producer only: they are never multicast and there is no query message for them (C-47). Third parties learn stream health from the per-connection summaries in each end's regime stamp (see below). The regime stamp itself travels on its own control message, not on this path (C-18). The path-MTU probe pair of admission (C-116) is a different message and is not part of this set.

### Liveness in the reverse direction (invariant L-1) `[decision]`

An admitted, healthy connection becomes dead only from data-plane evidence. The consumer judges the producer by slices: Interrupted after 8 consecutive missed slices (C-38), where only slices absent at the deadline (missing, or arriving late) count as misses and a marked gap or a slice on another timeline epoch resets the count (C-102); Dead after 2 s (C-39) counted from the last slice received with content or a marked gap (C-111); a slice on another timeline epoch also restarts the dead clock and is counted in its own per-connection counter (C-135). The producer has no data-plane evidence of its consumer except these reports, so: **the producer stops sending only after 2 s without receiver reports** (C-41). This is also the only liveness judgement for a sink-only device, which sends no streams (C-168). Any valid report counts as evidence, whatever its member records contain. Reports must therefore be sent by the consumer's data thread, not by housekeeping; otherwise a starved housekeeping thread would tear down a healthy stream. When the 2 s window starts before the first report is received is not yet decided (open sub-question 5).

### Receiver report content `[decision]`

- One report per **bundle**, not per member, with per-member counters (a 16-member bundle would otherwise cost 160 reports per second instead of 10).
- Each member record names its stream (16-bit stream number, C-10) and that stream's **session** (32 bits, C-10). The session is assigned by the source, one per stream, bumped on restart and on timeline epoch change, and identical for every consumer of the stream (C-19); there is no bundle-level session. The producer ignores a member record whose session is not the stream's current one.
- Counters count **slices**, not packets. At the default slice length (C-02) a member produces 2 000 slices per second, 200 per report interval; the default slice is one packet (C-05), so this is also 2 000 packets per second per stream (C-22).
- Counters distinguish **late** (arrived after the consumer's deadline; the remedy is budget) from **missing** (never arrived; the remedy is the network) and **marked gap** (upstream alive, loss further up). At the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span (rings are sized to A, C-96) it is reclassified as late (missing −1, late +1) and discarded (C-108). A missing count can therefore decrease between two reports.
- Each member record also carries the other per-connection counts the regime stamp publishes (C-159): other-epoch (C-135), too-early, duplicates, incomplete reassemblies and demoted rounds, so that the producer, which builds its summary from reports only, can publish the same counters as the consumer (C-47). A slice on another timeline epoch is counted in other-epoch, never as a miss (C-135).
- Each report carries, once per report, the consumer's **current A**, its own or the one it inherited (C-142). This is how a changed A travels upstream: one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186, C-198). This epic carries and exposes the value; deciding the inherited A and re-aligning belong to the code that applies C-142 and C-186.
- Input health is carried per member and includes the state Demoted: an input persistently late with its offset already at A is treated as gapped, counted separately (the demoted-rounds count, C-159) and cleared only by re-admission (C-109).
- **Drop accounting:** the kernel's per-socket overflow counter, cumulative, read on every receive (`SO_RXQ_OVFL`, Linux, since 2.6.33) `[kernel-doc]`; interface drop counters (deltas), which on a medianet are attributable to the runtime. macOS and Windows have no per-socket counter `[memory]`; there the field is absent (encoded differently from zero) and loss is inferred from position gaps only. Whether each drop counter and the received type-of-service byte can be read is published in the regime stamp with one of the four capability states present, absent, unverified or not requested (C-129); a field the report leaves absent corresponds to a stamp state other than present.
- **Observed DSCP:** the receiver reads the received type-of-service byte (IPv4, C-100) and reports it, because marking is verified by the peer and never self-reported. Expected: sync 48, media and probes 46, control 40 (C-13).
- Size: a report must fit one packet at the fallback path MTU, that is at most 1 252 bytes of UDP payload over IPv4 (C-06, C-17, C-100); a 32-member bundle (C-08) with every C-159 count is the worst case; a bundle source's controller member counts among the 32 (C-209) and has its own member record like any member stream (C-205). There is no split rule: a layout from epic 004 in which 32 members do not fit is a defect returned to epic 004.
- The encoder returns an error, and writes nothing, when given more than 32 members or a buffer smaller than the encoded size.

### Per-connection summaries in the regime stamp `[decision]`

Producer and consumer each publish per-connection summary counters in their own regime stamp: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events and dead (C-159, completing C-47), plus the observed DSCP (C-47). The consumer builds its summary from the counters it reports; the producer builds its summary from the reports it receives. Both ends use the same summary type so a third party can compare the two views. Epic 013 places the fields in the stamp; this epic provides the type and the code that fills it.

### One-way probes `[decision]`

Request and reply carry the four timestamps of an exchange (`t1` request sent, `t2` request received, `t3` reply sent, `t4` reply received) so either end can compute a round trip on the media class: round trip = (t4 − t1) − (t3 − t2). One-way figures derived from them are valid only while the clock's time is usable, carry the published clock bound, and carry the worst-case asymmetry bound. No in-band method can observe path asymmetry; the forward/return split is accurate to half the difference between the two directions. Since each one-way delay lies between 0 and the round trip, that worst case is half the round trip, and this is the asymmetry bound the figures carry. The receive stamps `t2` and `t4` come from the platform backend, whose receive-timestamp source (kernel, userspace, mac-hardware or interrupt) is reported in the stamp with a capability state (C-129, C-131); this epic does not choose the source. Time is usable while Locked and in holdover until the alignment claim lapses (C-65; the bound grows by C-64 meanwhile); for a device whose E is unknown, the lapse comes when its published bound exceeds twice the lock bound, 50 µs, about 25 s into holdover (C-169). After the lapse the clock is Unlocked with time unusable and one-way figures stop (C-162).

The commissioning tool may request probes only inside a declared commissioning window that every module stamps; outside it the tool is read-only (C-107). The window is announced by the tool on the control group with an expiry time and closes at expiry or on explicit close (C-153). While it is open, ordinary production measurements (latency, loss, alignment figures in stamps, including this epic's per-connection summaries) are unqualified, whereas the commissioning tool's own measurements, among them its probe figures and its soak, are valid commissioning results, each recorded with the window it was taken in (C-178). The default initiator and default rate are open sub-question 4, owned by this epic.

### Lineage extension `[decision]`

An optional extension in the slice (production streams may omit it) carries:

- **hop count**;
- **accumulated residence time**: each module adds (its output send time − its input arrival time), the same idea as a transparent clock's correction field;
- **accumulated declared delay**: the sum of the structural delays L declared by the modules on the path. Each L is always a whole number of domain-rate samples (C-105), so the accumulated declared delay is an integer sum of samples; for a DSL module it already includes any whole-sample delay line the compiler inserted at joins (C-104). An edge's conversion delay is part of its plumbing delay D, not of L (C-134). Because the wire has a single rate (C-01), one sample is exactly 1 000 000 000 / 96 000 ns, so declared delay and residence convert to one unit without loss of meaning.

Rules:

- Stamps denote content position and are never restamped per hop; a module with structural delay L has its output stamped p − L. The extension is how deep-graph latency stays attributable: arrival lateness = network + residence + declared delay.
- At a fan-in, the **critical input's** values propagate (the input that determined the offset). Asking for an input index that does not exist yields no value rather than a default.
- Residence is self-accounting: it already appears in downstream lateness. There is **one** event, "processing overrun", raised when a module completes after its deadline; there is no separate latency-violation event. This epic does not raise it: the hosted backend, epic 015, does (C-144).
- The extension fits inside the header cap of a default slice (at most 100 bytes for all headers and the checksum) or is omitted on a fallback path (C-137).
- Positional delay cannot be observed from timestamps; it is verified by a latency probe tool, not by this path.

### Related constants

| Id | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire |
| C-02 | Slice length | 48 frames = 0.5 ms = 2 000 slices/s |
| C-05 | Default slice size | one packet at the path MTU |
| C-06 | Path-MTU fallback | 1 280 bytes |
| C-08 | Members per bundle | at most 32, the controller member included (C-209) |
| C-10 | Stream number / session widths | 16 / 32 bits |
| C-13 | DSCP classes | sync 48; media and probes 46; control 40 |
| C-17 | Control message size bound | at most 1 252 bytes of UDP payload |
| C-19 | Session ownership | by the source, one per stream, same for every consumer |
| C-22 | Wire rate per source | 2 000 packets/s per stream; 64 000 packets/s for two 96-channel bundles (C-92) |
| C-23 | Admission ceiling | 70 % of the link rate |
| C-38 | Interrupted | 8 consecutive missed slices |
| C-39 | Dead | 2 s without data |
| C-40 | Receiver report | one per bundle every 100 ms |
| C-41 | Producer timeout | 2 s without receiver reports |
| C-47 | Third-party observability | per-connection summaries in each end's regime stamp; reports not multicast; no query |
| C-90 | Expected loss floor | about 1.1e-8 per default packet (about 1 320 bytes on the wire) `[memory]`, pending measurement |
| C-91 | Loss alarm threshold | set from clean-network loss measured at L1 |
| C-95 | Fuzz budget | each parser 60 s on every change and 1 hour nightly; corpus kept |
| C-96 | Skew budget cap | A; alignment rings are sized to A |
| C-100 | IP version | IPv4 only in v1 |
| C-102 | Miss for the interrupted threshold | missing or late; a marked gap or another epoch resets |
| C-104 | DSL join padding | whole-sample delay line, reported in the manifest; L is the longest path |
| C-105 | Structural latency L | always a whole number of domain-rate samples (the rational-L rule is withdrawn) |
| C-107 | Commissioning tool activity | test traffic only inside a declared commissioning window |
| C-108 | Late versus missing | missing at the deadline, reclassified late if it arrives within the ring span |
| C-109 | Demoted | health state, counted separately, cleared by re-admission |
| C-111 | Start of the dead threshold | last slice with content or a marked gap |
| C-118 | Loss soak | 24 h at 64 000 packets/s; aggregate loss over all streams; floor predicts about 58 losses |
| C-129 | Capability states in the regime stamp | present, absent, unverified, not requested |
| C-130 | Where task outputs live | specifications in `doc/design/spec/`, reports and results in `doc/design/reports/`, evidence in `doc/design/06-evidence-appendix.md` |
| C-131 | Receive-timestamp source values | kernel, userspace, mac-hardware, interrupt, each with a C-129 state |
| C-132 | Evidence tag for a published standard | `[standard]`, replacing `[memory]` once the citation is checked |
| C-133 | Where prototype and tool code lives | throwaway probes in `probes/`; harness and tool binaries in the `hadronyche-tools` crate |
| C-134 | Where a resampler node may appear | only inside an edge module; its conversion delay is part of D |
| C-135 | A slice on another timeline epoch | restarts the dead clock, is not a miss, has its own per-connection counter |
| C-137 | Header cap | at most 100 bytes for all headers and the checksum in a default slice; the measurement extension fits inside it or is omitted on a fallback path |
| C-142 | Fan-in outside any presentation group | inherits the A of the downstream presentation group it feeds (the largest if several), learned at admission |
| C-144 | Owner of four orphan duties | the hosted backend (epic 015), among them raising the processing-overrun event and sending the regime stamp message |
| C-147 | Strict mode and measurements | any run whose numbers enter the constants table runs in strict mode |
| C-153 | Commissioning window life cycle | announced with an expiry; stamped while open; closes at expiry or explicit close; what it disqualifies is in C-178 |
| C-159 | Per-connection counters in the stamp | received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead |
| C-160 | Lateness percentiles in the stamp | 50th, 99th, 99.9th and the maximum |
| C-162 | Clock after the holdover claim lapses | Unlocked, time unusable: one-way figures stop |
| C-169 | Holdover lapse when E is unknown | unknown E taken as 2 × C-62 = 50 µs; the claim lapses when the published bound exceeds it, about 25 s into holdover |
| C-164 | Bound on processing one bad input | linear in packet length, at most 10 µs per packet (proposal, pending L1) |
| C-167 | Loss alarm C-91 | 10 × the measured clean-wire loss (proposal, pending first L1 runs) |
| C-168 | Liveness of a sink-only device | judged by the producer timeout (C-41) |
| C-178 | What an open commissioning window disqualifies | ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified; the commissioning tool's own measurements are valid commissioning results, recorded with their window |
| C-179 | When measured loss reopens the redundancy decision | above 10 × the C-90 floor (about 580 losses in the C-118 soak); between 1 × and 10 × recorded and investigated as a cabling or host problem |
| C-186 | A changed while inputs are live | every fan-in that inherited the old A (C-142) re-enters aligning at an announced position, with rings and ratchet ceiling resized |
| C-187 | Publisher of the clock lock state and bound | the servo core (epic 008a), through the published time view; every epic that reports one-way figures depends on it |
| C-192 | Smaller settled points (part) | the vectors crate sits at the workspace root; the fuzzing tool is the implementing team's choice |
| C-193 | Supplier of conformance-vector inputs | epic 004: wire layouts and reject reasons |
| C-198 | Carrier of a changed A upstream | the receiver report carries the consumer's current A, its own or inherited; one hop per report; inheriting fan-ins re-align (C-186) |
| C-205, C-209 | Controller member of a bundle | always its own member stream; counts towards the 32 of C-08 |

**Settled since the ticket split.** IPv4 only and a 1 252-byte UDP payload bound (C-100, C-17), so a 32-member report must fit without a split rule. One session per stream, carried per member record (C-19). Counters count slices, 2 000 per second per member at defaults (C-02, C-05, C-22). Late reclassification (C-108) and the Demoted state (C-109). Per-connection summaries in the regime stamp, reports unicast only (C-47). Fuzz budget (C-95). Declared delay as an integer sum of whole domain-rate samples, convertible to time at the single rate (C-01, C-105 as revised; the rational-L rule and its representation question are withdrawn); resampling only inside edge modules, its delay in D (C-134). The extension fits the 100-byte header cap or is omitted on a fallback path (C-137). Per-connection counters completed by C-159, including other-epoch (C-135) and demoted rounds; lateness percentiles are in the stamp (C-160). One-way figures stop once the holdover claim lapses (C-162). The lock state and clock bound come from the servo core, epic 008a, through the published time view (C-187). Commissioning window life cycle (C-153). The processing-overrun event belongs to epic 015 (C-144). Parsers obey the bad-input bound (C-164, proposal pending L1). The loss alarm C-91 is proposed as 10 × the measured clean-wire loss (C-167). A sink-only device is judged by the producer timeout (C-168). The receiver report carries the consumer's current A, so a changed A propagates upstream one hop per report (C-198, C-142, C-186). The loss statistic is aggregate loss over all streams (C-118) against C-90 per default packet. Commissioning-tool probes only inside a commissioning window (C-107); an open window makes production figures in stamps unqualified while the tool's own probe and soak figures are valid commissioning results recorded with their window (C-178, replacing the earlier reading that every measurement in a window is unqualified). Above 10 × the C-90 floor, measured loss reopens the redundancy decision; between 1 × and 10 × it is a cabling or host investigation (C-179), read by the L1 soak with the comparator's threshold as a parameter. The holdover lapse for a device with unknown E (C-169). The controller member counts among the 32 members of a report (C-209, C-205). Vectors crate at the workspace root, fuzzing tool the team's choice (C-192); epic 004 supplies the layouts and reject reasons the vectors expect (C-193). Availability of drop counters and of the received type-of-service byte stamped with the four capability states (C-129); receive-timestamp source reported by the backend (C-131). Decision records and specifications go to `doc/design/spec/`, measured results to `doc/design/reports/` (C-130); socket tests and harnesses live in the `hadronyche-tools` crate (C-133); `[standard]` replaces `[memory]` once a citation is checked (C-132). Encoder error path, `Lineage::critical` with no value for a bad index, asymmetry bound of half the round trip, exact tolerances with injected timestamps, are settled by this epic.

## Deliverables

In `hadronyche-core`, module `measure`, `no_std`, no allocation:

```rust
// shape, names may be refined; byte layouts and type ids come from epic 004
pub struct MemberCounters { pub stream: StreamId /* 16 bits, C-10 */, pub session: u32 /* C-19 */,
                            pub received: u32, pub late: u32, pub missing: u32, pub marked_gap: u32,
                            pub other_epoch: u32, pub too_early: u32, pub duplicates: u32,
                            pub incomplete_reassemblies: u32, pub demoted_rounds: u32 /* C-159; widths from 014-02 */,
                            pub interrupted: u16, pub health: InputHealth /* includes Demoted, C-109 */ }
pub struct DropCounters { pub socket_overflow: Option<u32>, pub iface_rx_dropped: Option<u32>,
                          pub iface_rx_missed: Option<u32> }
pub enum EncodeError { TooManyMembers, BufferTooSmall }

pub struct ReceiverReport<'a> { /* borrowed view */ }
impl<'a> ReceiverReport<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Reject>;
    pub fn at_position(&self) -> u64;
    pub fn members(&self) -> impl Iterator<Item = MemberCounters> + '_;
    pub fn drops(&self) -> DropCounters;
    pub fn observed_dscp(&self) -> Option<u8>;
    pub fn current_a(&self) -> u32;                          // consumer's current A, own or inherited (C-198)
}
pub fn encode_report(at_position: u64, current_a: u32 /* C-198; unit and width from 004 */, members: &[MemberCounters],
                     drops: &DropCounters, observed_dscp: Option<u8>, out: &mut [u8]) -> Result<usize, EncodeError>;

pub struct ReportTimer { /* every 100 ms, driven from the data thread's cycle */ }
impl ReportTimer { pub fn new(now_ns: i64) -> Self; pub fn due(&mut self, now_ns: i64) -> bool; }

pub struct ProducerWatch { /* per consumer */ }
impl ProducerWatch {
    pub fn new(start_ns: i64) -> Self;                       // start condition: open sub-question 5
    pub fn on_report(&mut self, now_ns: i64);
    pub fn silent_for_2s(&self, now_ns: i64) -> bool;        // feeds the connection machine
}

pub struct ConnectionSummary { /* the C-159 counters (received, late, missing, marked_gap, other_epoch, too_early,
                                  duplicates, incomplete_reassemblies, demoted_rounds, interrupted, dead) and observed_dscp (C-47) */ }

pub struct Probe { pub t1: i64, pub t2: i64, pub t3: i64, pub t4: i64 }
pub struct ClockStatus { pub locked: bool /* time usable: Locked, or holdover before the claim lapses (C-162) */, pub bound_ns: u32 }
pub struct OneWay { pub forward_ns: i64, pub return_ns: i64, pub clock_bound_ns: u32, pub asymmetry_bound_ns: i64 }
impl Probe { pub fn round_trip_ns(&self) -> i64; pub fn one_way(&self, clock: ClockStatus) -> Option<OneWay>; }
pub fn encode_probe_request(t1: i64, out: &mut [u8]) -> Result<usize, EncodeError>;
pub fn answer_probe(req: &[u8], t2: i64, t3: i64, out: &mut [u8]) -> Result<usize, Reject>;
pub fn close_probe(reply: &[u8], t4: i64) -> Result<Probe, Reject>;

pub enum ReverseMessage<'a> { Report(ReceiverReport<'a>), ProbeRequest(/* view */), ProbeReply(/* view */) }
pub fn classify_reverse(bytes: &[u8]) -> Result<ReverseMessage<'_>, Reject>;   // anything else rejected and counted

pub struct Lineage { pub hops: u8, pub residence_ns: u32, pub declared_delay_samples: u32 /* whole samples, C-105; width from 004 */ }
impl Lineage {
    pub fn through(self, arrival_ns: i64, send_ns: i64, structural_delay_samples: u32) -> Lineage;
    pub fn critical(inputs: &[Lineage], critical: usize) -> Option<Lineage>;   // fan-in rule
}
```

Plus vector tests (in the vectors crate at the workspace root, with layouts and reject reasons from epic 004, C-192, C-193), fuzz targets for the report, probe, classifier and extension parsers (fuzzing tool of the implementing team's choice, C-192), and a loss-floor comparator that raises the cable-health flag when aggregate measured loss over all streams exceeds the threshold, and a check that every parser of this epic handles a bad input in time linear in its length and within the C-164 bound. The tests that need real sockets (criteria 6 and 7) are Linux-only, unprivileged `std` tests in the `hadronyche-tools` crate (C-133), not in `hadronyche-core`. Decision records produced by this epic go to `doc/design/spec/` and measured figures (for example the report-path benchmark) to `doc/design/reports/` (C-130).

## Dependencies

- Needs from 004: message type ids, the report and probe layouts (including the unit and width of the current A, C-198) and the extension layout. From 005: the connection machine's condition for the start of the producer timeout. From 007: vectors. From 009: the per-input counters (including the C-159 counts), the input health states (including Demoted) and the late reclassification. From 010: where the extension is written in a slice, within the C-137 header cap. From 012: the reverse port and the `ReportsSilentFor2s` evidence hook. From 013: where drop counters, observed DSCP and the per-connection summaries enter the stamp. From 008a, the servo core (C-187): the lock state and the published clock bound, through the published time view.
- Provides to 015: messages to send on the reverse socket and the counters to fill. Epic 015, not this epic, raises the processing-overrun event and sends the regime stamp message (C-144). To 017: the measurements the reference sender displays. To 019 and 020: one-way figures and loss accounting for the ladder. To 026: the same code on the device. To 028: the data the commissioning tool reads to propose A. To the code that applies C-142 and C-186: the consumer's current A from each report and the notice that it changed (C-198).

## Acceptance criteria

1. Report, probe and extension vectors pass in both directions (system criterion AC-1); the report, probe, classifier and extension parsers survive fuzzing with zero panics at the C-95 budget (60 s on every change, 1 hour nightly, corpus kept).
2. Exactly one report per bundle per 100 ms (±1 slice period), sent from the data thread; a 16-member bundle produces 10 reports per second, not 160.
3. A report for a 32-member bundle with every optional field present fits in 1 252 bytes of UDP payload (C-17, C-100).
4. AC-21 (producer side): with reports withheld, the producer stops at 2 s; with reports flowing and gossip fully suppressed, it never stops.
5. AC-15 refined by C-108: injected delay beyond the offset that arrives within the ring span ends in the late count only (a report made between the deadline and the arrival shows it as missing, and the next report shows it reclassified); injected drop, or a delay beyond the ring span, ends in the missing count only; an upstream-marked gap ends in the marked-gap count only; a slice on another timeline epoch ends in the other-epoch count only and is not a miss (C-135).
6. AC-37: an induced socket overflow appears in the report's drop counters on Linux.
7. AC-34: the report carries the observed DSCP of received media; a sender marking 46 is reported as 46.
8. Lineage: through three hops with injected integer timestamps, accumulated residence equals the sum of the injected residences exactly, the hop count is 3, and accumulated declared delay equals the integer sum of the declared L values in whole samples (C-105); at a fan-in the critical input's values are the ones propagated, and an out-of-range index yields no value without panicking.
9. A probe exchange with injected integer timestamps yields a round trip equal to the injected path delays exactly; one-way figures are withheld while the clock's time is unusable (Unlocked, including after the holdover claim lapses, C-162) and, otherwise, carry the clock bound and an asymmetry bound of half the round trip.
10. No media packet contains any field written from a receiver report (no per-receiver state in the forward direction).
11. Compiles for a `no_std` target without an allocator.
12. C-47 and C-159: from the same traffic, the producer's per-connection summary (every C-159 counter) built from received reports equals the consumer's own summary as of the last delivered report; member records carrying a session other than the stream's current one are ignored (C-19).
13. The reverse port accepts only the receiver report and the probe request and reply; every other message type is rejected and counted.
14. C-164: every parser of this epic (report, probe, classifier, extension) processes a bad input in time linear in its length and within 10 µs per packet on the reference target (the figure is a proposal pending L1).
15. C-198: every report carries the consumer's current A (own or inherited); the producer sees a change in the first report that carries it, and through a chain of inheriting modules the change travels one hop per report, without a control-plane message.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Report generation takes bounded time on the data thread and is shown not to allocate or lock
- [ ] The message set on the reverse port is closed: any other message type is rejected and counted

Not applicable in this epic (C-125): none. Every line has a subject here: code and parsers (report, probe, classifier, extension), regime stamp fields (drop counters, observed DSCP, per-connection summaries), and the reverse port's message set.

## Open sub-questions

| # | Question | Owner | Falsifier |
|---|---|---|---|
| 1 | Counter semantics (cumulative or per-interval), widths and wrap handling over long runs. Counters count slices, 2 000 per second per member at defaults; the socket overflow counter sees up to 64 000 packets/s (C-92); a missing count can decrease after reclassification (C-108) | Task 004 with this task (ticket 014-02) | A counter wraps within one report interval at those rates, or a reclassification cannot be expressed in the chosen encoding |
| 2 | Settled: the commissioning tool proposes A from the lateness maximum and percentiles published in the regime stamp (C-160) with the commissioning rule (C-148), not from report counters, so the 100 ms report interval (C-40) does not limit it | — | — |
| 3 | Can interface drop counters be read without privilege on each platform? | Tasks 015, 021, 022 | Not readable; the field is absent there and stamped absent (C-129) |
| 4 | Who initiates one-way probes, and how often by default? (Tool-driven probes are limited to a commissioning window, C-107) | This task (ticket 014-11), with task 012 for the admission-budget granularity | Probes at the default rate change an admission decision against the 70 % ceiling (C-23) |
| 5 | When does the producer's 2 s window start before the first report: at admission, or at another point of the connection machine? | Task 005 | A producer stops a connection whose consumer is alive but has not yet reported, or keeps sending to a consumer that never reports |
| 6 | Is the loss floor figure right? | Pre-implementation check (task 001; the checked citation turns the `[memory]` tag into `[standard]`, C-132), then L1 (task 019) | Published figure differs; clean-network loss well above the floor |
| 7 | Minimum sample size before the loss-floor comparator may raise the cable-health flag (the threshold itself is proposed as 10 × the measured clean-wire loss, C-167) | L1 (task 019) | On a clean network the flag fires, or over a 24 h soak (C-118) it stays silent with loss well above C-91 |
| 8 | Settled: C-105 as revised makes every L a whole number of samples, so the accumulated declared delay is an integer sum; its width and saturation join question 10 | — | — |
| 9 | Settled: the servo core, epic 008a, publishes the clock lock state and bound through the published time view (C-187); ticket 014-09 depends on it | — | — |
| 10 | Range and saturation of the hop count, accumulated residence and accumulated declared delay (whole samples, C-105), within the C-137 header cap, and the rule when a module's send time precedes its arrival time | Task 004 with this task (tickets 014-13, 014-21) | A real path exceeds the field range, the extension does not fit the header cap, or a negative residence is observed on a locked module |

Further gaps whose owner is another epic or a measurement are listed in the tickets index (`tickets/014/README.md`).
