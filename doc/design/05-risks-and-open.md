# 05 — Risks, open items and retired items

## The three riskiest assumptions

### 1. The redesigned servo works

A frequency-only servo with slow slew (10 ppm, 1 ppm/s) and software timestamps must hold a 25 µs bound. It is unsimulated. E, every alignment margin and every presentation claim sit on it, and it now also carries loop 1 of every hosted edge on three operating systems.

- **If false:** the lock bound rises, E targets rise with it, and tight presentation groups are not achievable in software.
- **Closed by:** roadmap P2 (task 008), then L2.

### 2. Hosts keep up at 32 000 to 64 000 packets per second

Six channels per packet is cheap on the wire and expensive in system calls. The two-bundle target rests on a guessed 2 to 4 µs per packet. At 10 µs a single 96-channel bundle already takes a third of the slice period. If hosts drop at these rates, the premise that loss sits at the bit-error floor falls, and so does the deferral of redundancy.

- **If false:** fewer channels per port than the capacity map states, or redundancy returns to v1.
- **Closed by:** L1 (task 019), criterion AC-13.

### 3. Edges deliver a repeatable D

Three audio stacks must each give a plumbing delay that is stable across restarts. Windows is the weakest: half a millisecond of restart spread against 3 to 10 ms engine periods.

- **If false on Windows:** contained; Windows sinks fall out of tight presentation groups.
- **If false on Linux:** cross-sink simultaneity cannot be demonstrated on the reference platform.
- **Closed by:** L2 per platform (tasks 018, 020, 021, 022), criteria AC-26 to AC-28.

Beneath all three: the design was derived by reading the POC, and the POC's role as exact-tier baseline is provisional until task 002 clears its instrument bugs.

## Open items

Every item has an owner that is a milestone or a verification, and a falsifier. There are no decision-shaped items.

| # | Item | Owner | Falsifier |
|---|---|---|---|
| O-1 | Frequency-only servo: gains, acquisition time, 25 µs bound under the slew limits | Simulation (task 008) | Bound unreachable in acceptable time |
| O-2 | Frequency continuity at handover | Simulation (task 008) | Rate step exceeds 1 ppm/s |
| O-3 | Maximum-plus-headroom estimator over 4 s | Simulation (task 008), then L1 (task 019) | Late slices after alignment on a clean network |
| O-4 | Ratchet and dwell behaviour | Simulation (task 008) | Oscillation, or A exhausted in normal use |
| O-5 | Wander masks | Simulation (task 008), then L2 (task 020) | Mask violated at the edge |
| O-6 | Loss at the bit-error floor | L1 (task 019) | Clean-network loss well above the floor |
| O-7 | Receive cost per packet; two 96-channel bundles on one GbE port | L1 (task 019) | Late slices in the one-hour test |
| O-8 | Holder serving 64 followers | L1 / L3 (tasks 019, 027) | Serving lateness grows with followers |
| O-9 | Interrupt-coalescing bias on receive stamps | L1 (task 019) | Offset shifts when coalescing is toggled |
| O-10 | E and restart spread of D, per platform | L2 (tasks 018, 020, 021, 022) | Targets missed |
| O-11 | Windows batched capture emission against consumer margins | L2 Windows (task 022) | Consumers gap at the commissioned A |
| O-12 | Cross-sink skew | L3 (task 027) | Exceeds the sum of the two E values |
| O-13 | Device fit: resampler cost, 63-peer map (C-217), bounded control handling | Embedded build and L3 (task 026) | Does not fit, or no bound can be shown |
| O-14 | Compiler cost-bound calibration | DSL milestone (task 024) | Measured cost exceeds the bound |
| O-15 | Fingerprint vector coverage (subnormal and NaN paths) | DSL milestone (task 025) | A target passes the fingerprint but differs on a vector |
| O-16 | Structural-latency probe suite; resampler node exactness | DSL milestone (task 025) | Probe disagrees with the compiler's L |
| O-17 | Receive-stamp availability and basis on macOS and Windows (check 0a) | Pre-implementation (task 001) | Absent; clock bound and E targets rise |
| O-18 | POC archaeology (check 0b) | Pre-implementation (task 002) | Voice comparison is not bit-level, or the I/O-pressure finding fails |
| O-19 | POC smoke basics (check 0c) | Pre-implementation (task 003) | — |
| O-20 | MTU probe pair on a real switch; batch receive behaviour; published bit-error figure | Pre-implementation (task 001), L1 (task 019) | Probe gives a false pass on a smaller hop |
| O-21 | Commissioning tools: A proposal, trunk validation, group-E certification | Commissioning tooling (task 028) | — |
| O-22 | Checklists: per-OS chatter and firewall, coalescing, switch QoS and burst test | Commissioning tooling (task 028) | — |
| O-23 | Wire layouts, state-machine tables, conformance vectors | Specification drafting (tasks 004 to 007) | Does not fit device memory or the control message bound |
| O-24 | Loss alarm threshold | L1 (task 019) | — |

## Retired items

Reviewers: this is what was consciously excluded. Reintroducing any of it needs a new ADR.

### POC design, not carried forward

| Retired | Kill-reason | Replaced by |
|---|---|---|
| Wi-Fi profile, its servo windows, send-buffer constant and airtime rationale | Wi-Fi is out of scope | Wired medianets only |
| One UDP datagram per slice with IP fragmentation; the 48-channel default feed | 7 fragments by default: loss multiplied, invisible, host-wide reassembly cliff | Single-packet default, bundles |
| Media clock on the sender's local crystal | Two sources drift apart; fan-in cannot align | Sources paced from network time |
| Mix on arrival with no alignment | Unequal paths are skewed, not just jittered | Position-indexed fan-in at a fixed time |
| Unbounded socket drain on the data thread | One flooding input starves the deadline | Bounded drain, source filter, sized buffer |
| Three threads per module | N servos and N gossip participants per process | Per-process housekeeping and clock, one data thread per module |
| Holder rotation | The clock is the audio clock; every handover is a rate event | Change on loss or pressure only |
| Hello expiry opening elections or tearing down live streams | A starved housekeeper kills healthy streams | L-1, L-6 |
| Unilateral send; retrying forever | Receiver never consents; no capacity control | Two-sided admission with explicit refusal |
| First-match name routing; duplicate names merely warned | Non-deterministic resolution | Bind to id at admission; newcomer with a taken name refuses to join |
| Rebuilding live routes on address change | A wrong hello redirects a stream | L-5 |
| 31-bit time-and-pid id | Collision is silently fatal | 64-bit random id plus nonce |
| Text gossip | Unbounded parsing on devices | Binary control format with a size bound |
| Header fields overloaded by direction; per-receiver acknowledgements in forward packets; measurements in the stream header | Unreadable, blocks multicast later | Measurement extension and measurement path |
| Servo phase-step corrections | Discontinuous published time | Frequency-only correction |
| Timeout-paced sync requests | Measured 36.1/s and 71.4/s for 50 and 1 000 configured (P-02) | Deadline pacing |
| Holdover that never expires; lock without a numeric bound | Stale time reported as usable | Growing bound, numeric lock |
| Mutex shared between clock thread and housekeeping | Priority inversion | L-8 |
| Rate step at handover | Edge resamplers see a frequency jump | Frequency-continuous handover |
| Transport reset driven by gossip | Content altered by the control plane | In-band reference change |
| Fixed-point table built with libm | Not strictly deterministic across machines | Shipped constant tables |
| Per-module writers on one file path | Corrupt metrics file | One writer per file, owned by the process |
| Silent drops (measurement rings, malformed packets, version mismatch) | Flatter the numbers, hide faults | Counted, stamped, named events |
| A check family with zero samples shown as green | Half the verification can be inert | Zero samples fails the run |

### Guarantees withdrawn

| Withdrawn | Reason |
|---|---|
| Bit-exactness as a runtime-wide property | Impossible for user modules; replaced by tiers |
| Exact tier for edge modules | They resample |
| Exact-float between host and device | Only fixed point is safe across profiles |
| Fail-stop for every thread | A housekeeping panic must not kill live streams |
| E as a bound on hosted platforms | It is a percentile claim |

### Proposals killed during design

| Proposal | Kill-reason |
|---|---|
| Retransmission | Per-receiver state and inbound requests on the source's real-time path; traffic outside the admitted rate; resends over the path that just failed |
| Forward error correction; the parity hook | With single-packet slices, per-slice parity is duplication and cross-slice parity costs latency |
| Per-edge deadline negotiation | Upstream cannot act on a deadline; negotiation reduces to refusal |
| Early-fire fan-in | Output timing becomes data-dependent |
| Token bucket on the data thread | The packet has already cost kernel time |
| Gossiped topology version | In-band announcements reach exactly the descendants |
| Percentile-only offset estimator | Over a short window the 99.9th percentile rests on a handful of samples |
| Offsets frozen until an operator acts | Graphs mutate while playing |
| Unbounded automatic re-alignment | One flapping edge glitches the subgraph forever |
| Runtime resampling to slew an offset | Breaks the when/what boundary and bit-exactness |
| A separate latency-violation event; live verification of positional delay | Residence is already covered by overrun; positional delay is invisible to timestamps |
| Eight missed slices as the dead threshold | A 4 ms hiccup would cost seconds of re-alignment |
| Per-stream position lines | A mixer of independent sources has no single timeline |
| A computed live | Changing A is a domain-wide discontinuity |
| A term in the announcement lead | In-band announcements are processed in position order |
| Sync in the media DSCP class | The servo wants least and most symmetric delay |
| A random-derived domain id | The domain is operator-owned |
| Same-card loopback as a servo test | Shared crystal; two resamplers cancel |
| Trunk admission as a runtime mechanism | Endpoints cannot see trunks |
| Excluding devices from holding the clock by profile | The real reasons are serving load, availability, oscillator quality |
| A core crate with packet definitions only | Two implementations of the election would diverge |
| Evolving the POC in place; porting the servo unchanged | Changes land in the core; almost no tests; the servo is redesigned |
| 7 channels per stream | Only 121 bytes left for headers and in-band content |
| A socket per stream; receiver reports per stream | 16 times the cost for no benefit |
| Self-reported DSCP | Platforms cannot be trusted to mark; the peer observes |
| A 3-byte sample format | Closed: density is already priced into the capacity map |
| A spinning ticker on Windows | Burns a core; capture paces off the audio callback instead |

### Deferred past v1, with the reservation kept

| Deferred | Reservation |
|---|---|
| Multicast fan-out | Streams identified by source and stream number; no per-receiver state in forward packets; listen address in the admission reply |
| Dual-path and delayed duplication | 2-bit copy index; dedup by stream, position, packet index |
| Control authentication | Scheme byte, 64-bit counter, 128-bit tag; scheduled immediately after v1 |
| Kernel pacing | None needed |
| External clock discipline (PTP) | The published time view is the seam |
| Hardware clock steering | Rate-actuator trait |
| Transport authoring in the DSL | Transport state is a read-only input in v1 |
| Shared clock between processes | None needed |
| Received-positions bitmap | One optional field in the measurement extension |
| Application-output capture (virtual audio device) | None; product scope |
| Tiered sync relay beyond 64 followers | None; documented as the future answer |
