# 03 — System acceptance criteria

Each criterion is a testable statement with its method, threshold and falsifier. Constants are quoted from [02-constants.md](02-constants.md) by id. Ladder rungs L0 to L3 are defined in [01-architecture.md §9.5](01-architecture.md#95-certification-ladder).

**Methods:** CI (unprivileged, every change) · SIM (deterministic simulation against the pure core) · L0 (loopback, one process) · L1 (two machines, one wire) · L2 (reference sink per platform) · L3 (hosted plus embedded sink).

Thresholds marked *proposal* are pending-simulation or pending-measurement in the constants table; the criterion stands, the number may move.

## Protocol and conformance

| ID | Statement | Method | Threshold | Falsified by |
|---|---|---|---|---|
| AC-1 | Conformance vectors pass in both directions, malformed cases included; every parser survives fuzzing | CI | 100 % of vectors; zero panics, zero hangs in the fuzz budget (C-95: 60 s per parser on every change, 1 hour nightly) | Any vector mismatch; any parser panic |
| AC-19 | An MTU blackhole on the path is detected and survived | L0 with injected size-dependent drop; L1 on a real switch | Probe pair detects at admission; fallback to 1 280 bytes (C-06); re-probe triggers when large packets vanish while liveness holds; path MTU appears in the stamp | A connection goes live at a size the path drops |
| AC-23 | Malformed, wrong-version and foreign-domain traffic is harmless and visible | CI + L0 with injection | No crash, no state change; each rejected packet counted per input and per reason; version mismatch produces a named event | Any state change; any uncounted rejection |
| AC-34 | DSCP marking is verified by the receiving peer | L1 | Receiver report shows sync 48, media and probes 46, control 40 (C-13), or the stamp says "unverified" | Stamp claims a marking the peer did not observe |

## Clock

| ID | Statement | Method | Threshold | Falsified by |
|---|---|---|---|---|
| AC-2 | The servo locks within its bound | SIM | Bound at most 25 µs (C-62) within 10 s (C-63, *proposal*) under the wired jitter profile (C-94, provisional) and drift up to ±100 ppm | Bound not reached under C-60 and C-61 |
| AC-3 | Published time is continuous and smooth while locked | SIM | Zero phase step across updates; slew at most 10 ppm (C-60); rate change at most 1 ppm/s (C-61), in every scenario | Any step; any limit exceeded |
| AC-4 | Lock is honest | SIM, property test | The servo never reports Locked while true error exceeds its published bound | One counter-example |
| AC-5 | Handover is frequency-continuous | SIM, then L2 | With a 50 ppm crystal difference (C-78), rate change stays within C-61 and published time has no step | A rate step at handover |
| AC-6 | Holdover is bounded and expires | SIM | Bound grows by 1 µs/s (C-64); alignment claim lapses when it exceeds the margin (about 25 s, C-65); the state is never reported usable beyond that | Holdover reported usable with a stale bound |
| AC-7 | Achieved sync rate equals configured rate | SIM + L0 | Within 2 % over the whole accepted range of 10 to 200 requests per second (C-112); the achieved rate is reported | The POC behaviour (37/s at 50, 77/s at 1 000, P-02) |
| AC-25 | A holder serves its follower cap | L1 / L3 | 64 followers (C-35) at 50 requests/s each with no growth in serving lateness; the 65th is refused with a named event. Followers beyond the 63 real ones a 64-module medianet allows are synthetic (C-114) | Serving lateness grows below 64 |
| AC-36 | A healed partition converges | L0 with injected partition | One holder within election window plus hello period (C-33 + C-30 = 1.4 s); the losing side receives a timeline epoch change and its streams restart at an announced position | Two holders persist; a silent step |

## Graph semantics

| ID | Statement | Method | Threshold | Falsified by |
|---|---|---|---|---|
| AC-10 | A diamond graph aligns sample-exactly | L0 | Source feeds a fan-in directly and through one processor with injected per-hop delay; the fan-in output equals the offline reference bit for bit (exact tier) | Any sample misaligned |
| AC-14 | No late slices at the commissioned A | L1 | Zero late slices for 1 hour on a clean medianet | Any late slice |
| AC-15 | Late and lost are distinguished | L0 with injection | Injected delay beyond the offset is counted as late only; injected drop as lost (missing) only | A delayed slice counted as lost, or the reverse |
| AC-16 | The four gap classes are produced correctly | L0 with injection | Marked gap propagates with the degraded flag; interrupted after 8 consecutive misses (C-38); dead after 2 s (C-39); resume from interrupted is immediate at the frozen offset | Wrong class; re-alignment forced by a short interruption |
| AC-17 | Re-alignment is coordinated and terminates | SIM + L0 | Descendants switch at the announced position (lead 64 slices, C-43; re-issue every 16, C-44); the ratchet steps one slice after 10 s dwell (C-53, C-54) and stops at A; a flapping input is demoted to gapped | Oscillation; a descendant switching at a different position; offset above A |
| AC-18 | A fan-in never aligns across timeline epochs | L0 | An input on another epoch is treated as a gap until it switches | Samples from two epochs combined |

## Control plane and failure semantics

| ID | Statement | Method | Threshold | Falsified by |
|---|---|---|---|---|
| AC-20 | Housekeeping cannot damage a live stream | L0 with injection | (a) hellos suppressed for 30 s: live streams continue (L-1). (b) holder hello suppressed while replies flow: no election (L-6). (c) housekeeping thread killed: restarted, streams unaffected, module marked control-degraded (L-7). (d) a hello with a changed address: no redirect (L-5). (e) static check: no lock shared with data or clock threads (L-8) | Any stream interruption in (a) to (d) |
| AC-21 | Thread death is fail-stop and detected | L0 | A killed data or clock thread terminates the process; peers report interrupted within 4 ms and dead at 2 s; the producer stops after 2 s without reports (C-41) | A module alive but silent |
| AC-22 | An id collision is never silent | L0 with a forced id | Detected within one hello period (C-30); the later joiner redraws; third parties quarantine the id; logged on both modules | A merged peer; an unlogged collision |
| AC-11 | A flooding input is isolated | L0 / L1 | One input flooded at 10 × its admitted rate: zero late slices on other inputs, no missed deadline on the module; excess appears in the socket overflow counter | The module's own deadline is missed |
| AC-12 | Admission refuses explicitly | L0 / L1 | A request exceeding 70 % of a port (C-23) is refused with a named reason; the graph shows the connection unsatisfied; no retry loop | A half-connected state; a silent retry |
| AC-37 | Drops are accounted | L0 with injection | An induced socket overflow appears in the stamp's drop counters and in the receiver report | Loss with no matching counter on Linux |

## Capacity

| ID | Statement | Method | Threshold | Falsified by |
|---|---|---|---|---|
| AC-13 | Two 96-channel bundles on one GbE port | L1 | 32 streams, 64 000 packets/s, 676 Mbit/s aligned for 1 hour with zero late slices at the commissioned A; a third bundle refused at admission (C-92) | Any late slice; the third bundle admitted |
| AC-24 | The two-machine baseline holds | L1 | Asymmetry bound holds against an external check; loss at or below the floor (C-90); in the multi-packet regime first-packet and slice-complete latency differ by the serialisation time; coalescing bias measured and recorded | Loss well above the floor on a clean wire |

## Numbers governance

| ID | Statement | Method | Threshold | Falsified by |
|---|---|---|---|---|
| AC-8 | The loopback smoke run is trustworthy | L0 / CI | Injected offset, drift and packet loss; zero content mismatches on the exact tier outside excluded windows; estimated minus true offset within the published bound; packet and slice loss consistent with the injected rate; **no assertion depends on sync-rate timing** | Any mismatch; an assertion that changes with the sync rate |
| AC-9 | A check family with zero samples fails the run | CI | The run exits non-zero and names the inert family | A green run with an inert family (the POC behaviour) |
| AC-31 | Every record set carries a complete stamp | CI + L0 | All stamp groups present; injection and missing privileges labelled; a regime change starts a new record set; one writer per file | A record without a stamp; two writers on one file (the POC behaviour) |

## Edge and platforms

| ID | Statement | Method | Threshold | Falsified by |
|---|---|---|---|---|
| AC-26 | Linux reference sink meets its claims | L2 Linux | E at most 50 µs (C-70, definition C-73); restart spread at most 25 µs over 20 restarts (C-74); marker offset within ±2 samples (C-75); zero slip (C-76); wander within C-77 | Any threshold missed |
| AC-27 | macOS edge meets its claims | L2 macOS | E at most 100 µs (C-71); restart spread at most 50 µs; cross-sink skew against the Linux sink at most 150 µs | Any threshold missed |
| AC-28 | Windows edge meets its claims | L2 Windows | E at most 1 ms (C-72); restart spread at most 0.5 ms; cross-sink skew against the Linux sink at most 1.05 ms; batched capture emission causes no gaps at the commissioned A | Any threshold missed |
| AC-29 | Cross-sink simultaneity | L3 | The 99.9th percentile of absolute skew between a hosted and an embedded sink is at most the sum of their E values, with the maximum reported, for 1 hour and across restarts of either (C-79) | Skew exceeds the sum |
| AC-30 | Required E is enforced | L0 / L2 | A sink whose E exceeds the group's required E is refused at admission with a named reason | Such a sink goes live in the group |
| AC-35 | An embedded device is a correct member | L3 | Joins as follower with a 63-peer map (C-34, C-217); control handling bounded or stamped report-only; stamp published over the network | Device memory cannot hold the map; unbounded control handling unstamped |

## DSL

| ID | Statement | Method | Threshold | Falsified by |
|---|---|---|---|---|
| AC-32 | Determinism classes are honest | CI across listed targets | exact-fixed programs bit-identical on every target; exact-float programs bit-identical where the fingerprint matches; a mismatch runs bounded-class and is stamped | A target that passes the fingerprint and differs on a vector |
| AC-33 | Compiler-derived figures are right | CI | Structural latency L equals the probe-measured value exactly; measured cost never exceeds the cost bound | Probe disagrees; cost exceeds the bound |

Total: 37 criteria (AC-1 to AC-37).
