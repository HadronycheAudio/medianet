# 023 — DSL compiler front end

**Phase:** P5 · **Scope:** L (32 tickets, about 38.75 days) · **Depends on:** 004, 012, 013, 024a (the kit's resampler node, C-197) · **Blocks:** 024 (code generation; not the kit part 024a, which comes first, C-197), 025

## Goal

Build the front half of the module-construction DSL compiler (`hadronyche-dsl`): the language surface, parsing, type and rate checking, and the **static analyses** whose results the runtime relies on — structural latency, cost bound, state and stack size, determinism class, and the dataflow that drives degraded marking. The front end ends in a checked intermediate representation (IR) plus a **manifest**. Code generation is task 024.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated IPv4 LANs (C-100) whose only traffic is this runtime's. Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline (`position = (network_time − epoch_origin) × rate`). Every stream on the wire is at the domain rate, 96 kHz (C-01). Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers. The product is three things: packetisation and network, this **module-construction DSL**, and two implementations (hosted `std` and `no_std` for devices).

**Boundary.** The runtime owns *when and whether* content moves; module code owns *what* it is. The DSL is how module authors write the *what* so that the runtime can trust figures it would otherwise have to take on faith.

**What the compiler owes the runtime `[decision]`.**

1. **One process function** per module, `no_std`, pure Rust:
   `process(state, inputs at position p with per-input gap flags, position) -> outputs`.
   It never blocks. **No allocation at process time.** The compiler emits the state size and worst-case stack as constants.
2. **No libm and no transcendental functions.** The arithmetic kit is: strict f32 operations; Q1.31 and Q0.32 fixed point; shipped deterministic tables. Float operations are never reordered; summation order is defined by DSL semantics. (This generalises a POC lesson `[code]`: `src/music.rs:263` builds sine from additions and multiplications only "so that it gives the same bits on every machine", while `src/music.rs:311` builds a fixed-point table with libm and is therefore not strictly deterministic across machines.)
3. **A manifest** (see "Manifest contents" in this section).
4. **A cost bound.** With no unbounded loops, an operation-count upper bound per slice is derivable. Its unit is counted abstract operations per slice, identical on every machine (C-106). Each target has a calibration factor (time per operation), measured once and recorded in the regime stamp; admission converts the bound with it (C-106). Measuring the factor is not this task's work: the DSL conformance suite (task 025) runs a reference workload once per listed target and records time per operation (C-176). Admission uses the bound, so CPU stops being a mere diagnostic for DSL modules.
5. **Structural latency L** as a compile-time constant, **always a whole number of domain-rate samples** in the manifest and ABI (C-105): explicit delays, lookahead, block buffering and linear-phase filter centres. A linear-phase FIR with an even number of taps is rejected in v1, with a diagnostic suggesting an odd-tap design, so that L stays exact (C-183). Recursive filters have frequency-dependent delay; the compiler reports that as a *property*, not a number. The runtime stamps a module's output at position p − L while it processes input p. A resampler's delay is not part of L: an edge's conversion delay is counted only in its plumbing delay D and never in L, and L is the module's structural delay in the domain-rate timeline (C-134, C-170, C-207). The compiler knows it exactly and reports it separately in the manifest, also in whole domain-rate samples, on the device-facing port it serves (C-207).
6. **Paths that meet at a join (C-104).** Where paths of different latency meet, the compiler pads the shorter path with a whole-sample delay line, reports each inserted delay in the manifest, and the module's L for that output is the longest path. An explicit DSL construct opts out per join. A path with frequency-dependent delay is padded on its structural part only and the join is flagged in the manifest. Since every L is a whole number of domain-rate samples (C-105), a padding amount is always whole. Inserted delays count in the state size and the cost bound. On the device side of an edge module, paths that differ by an odd number of domain-rate samples cannot be padded with a whole-sample delay line at the device rate; the compiler rejects the join and suggests the per-join opt-out (C-190).
7. **One wire rate; rate conversion only at edges.** Every network port is at the domain rate, 96 kHz (C-01). A resampler node may appear only inside an edge module; the compiler rejects it anywhere else (C-134). An edge whose hardware runs at 48 kHz converts 2:1 through the kit's resampler node, and the wire stays at the domain rate (C-80). The resampler is offered only in designs whose delay is a whole number of samples, 2:1 and 1:2; other ratios are out of v1 (C-105). It is the only node whose input and output rates differ, and the compiler knows its delay exactly. The kit epic 024a builds this node once, with its design and delay; this task models it in the IR and imports its figures (C-197). It is fixed-ratio; an edge's steerable rate actuator is a separate component whose delay is part of D, not a DSL node (C-189, C-210). At the 0.5 ms slice (C-02) a device-facing side at 48 kHz has 24 frames per slice (48 000 × 0.0005, derived).
8. **A determinism class** per module:

   | Class | Scope | Verification |
   |---|---|---|
   | exact-fixed | Bit-exact everywhere, including between host and device | Exact tier: record and replay |
   | exact-float | Bit-exact only where a startup fingerprint matches | Exact tier: record and replay |
   | bounded | Everything else | Tolerance tier |

   The **fingerprint** is a compile-time vector (including subnormal and NaN paths) evaluated at process start **on the processing thread itself**, because floating-point modes are per thread and audio stacks change them. Match = exact-float eligible; mismatch = the module runs bounded-class and the regime stamp says so. (Emitting and evaluating the fingerprint is tasks 024 and 025; this task classifies.) The bounded class arises only from an exact-float fingerprint mismatch and for hand-written (non-DSL) modules that do not claim exactness; **the DSL has no non-deterministic node in v1** (C-123). The static classifier therefore assigns a DSL program exact-fixed or exact-float, never bounded; anything outside the deterministic kit is rejected (criterion 023-AC-1), not downgraded.
9. **Gap handling.** Compiler-generated defaults per content type — audio zero-fill; dense controllers hold last value; sparse keyed controllers rely on state refresh — overridable by DSL constructs. Read-only transport state on a gap: the last known state is held and the playhead advanced as that state dictates — a playing transport moves on at its tempo by the positions elapsed, a stopped one does not advance (C-200) — and the output is marked degraded (C-141). Each dense controller declares an initial value in the manifest (default 0), used when a gap precedes any received value (C-158). Before any transport state has arrived, the input reads stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199). **Degraded marking is derived per output from dataflow and follows the content (C-122):** an output at position p is degraded if an input it depends on was gapped at position p − L, where L is that output's structural latency. The front end supplies, per output, the input dependency set and L.

**Content domains, v1 `[decision]`.**

- **Audio.**
- **Dense controllers**: one value per tick.
- **Sparse keyed controllers**: one field of one voice moving linearly to a value over the slice, repaired by state refresh. (This is what the POC calls musical events `[code]`: `proto::Part` is voice, field, value; `src/music.rs:17-25`.)
- **Read-only transport state** as an input (playing, bar, beat, tempo, time code at position p).
- **Transport authoring is deferred past v1.** The language must not offer it.

The reference echo (a synthesiser driven by sparse keyed controllers, plus a metronome reading transport state) must be expressible; task 025 writes it.

**Constants a program is checked against.** Domain rate 96 kHz, the only wire rate (C-01); slice 48 frames = 0.5 ms (C-02); 4-byte samples f32 or Q1.31 (C-03); 6 channels per stream at defaults, 192 bytes per channel per slice and 1 152 bytes of audio per slice, with no per-channel id byte — the channel map lives in the manifest (C-04). Wider ports become **bundles** of at most 32 members (C-08), so a port is at most 32 × 6 = 192 channels at defaults; the controller member counts towards the 32, so a source with controllers has at most 31 audio members (186 channels) and a wider one is rejected at compile time (C-209); at most 256 streams per source (C-09). Wire figures for admission: about 1 320 bytes per default packet on the wire including all Ethernet overhead (C-20); about 21.1 Mbit/s and 2 000 packets/s per 6-channel stream (C-21); a 96-channel source is 16 streams, 338 Mbit/s, 32 000 packets/s (C-22). A single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit, and the compiler checks this from the manifest (C-181). A bundle source's controllers always ride in their own member stream (C-181, C-205).

**Manifest contents.** Module name; ABI version; ports (direction, content type, rate, channels, bundle layout and channel map; for an edge module, its device-facing side and that side's rate); slice sizes; admission figures (bit/s on the wire, packets/s, input count, buffer memory) for network ports only; state size; stack bound; cost bound (total and per output, in abstract operations, C-106); structural latency L per output as a whole number of domain-rate samples (C-105); non-constant-delay property; delays inserted at joins, joins opted out and joins flagged for a frequency-dependent path (C-104); for an edge module, the conversion delay of each device-facing port, reported separately, counted only in D and never in L (C-134, C-207); determinism class; gap overrides; initial value of each dense controller input (C-158); (later, from task 025) generated conformance vectors for exact-class programs.

**Diagnostics.** Every rejection is a `Diagnostic` with a source span, a stable code and a message that names the violated rule. This task defines the codes (ticket 023-02) and `LANGUAGE.md` lists them; "clear diagnostic" in the criteria means exactly this.

**Where outputs live (C-130, C-133).** Specifications written by this task (syntax decision and grammar, resampler design, manifest binary layout, hand-written module declaration) go in `doc/design/spec/`; the constants table is `doc/design/02-constants.md`. `LANGUAGE.md` and the fuzz targets ship with the `hadronyche-dsl` crate; any throwaway probe goes in `probes/`. Nothing is written into the frozen POC tree.

**Evidence tags.** Claims carry the tags of `00-context.md`; a figure cited from a published standard uses `[standard]` with the clause (C-132).

**Rulings applied to this epic.** C-01, C-04, C-20, C-21, C-22 and C-80 (single wire rate, no per-channel id byte, wire figures); C-95 (fuzz budget); C-100 (IPv4); C-104 (join padding with whole-sample delay lines; frequency-dependent paths padded on their structural part and flagged); C-105 (L always a whole number of domain-rate samples; resampler only at 2:1 and 1:2; the rational-L rule is withdrawn); C-106 and C-176 (cost unit, calibration, and who measures the factor); C-122 (degraded flag follows content); C-123 (where bounded arises); C-125 (definition-of-done applicability); C-126 and C-143 (scope from the ticket tally; no split below 40 days); C-130, C-133 and C-171 (output locations); C-132 (`[standard]` tag); C-134 and C-207 (resampler only inside an edge module; conversion delay counted only in D, defined by C-170, never in L, and reported separately on the device-facing port); C-137 (header cap behind the admission figures); C-141 and C-200 (transport-state gap default; a stopped playhead does not advance through a gap); C-146 (per-change CI unprivileged; this epic has no privileged test); C-158 (dense-controller initial value); C-181 and C-205 (a single-stream source may always put its controllers in a second stream and must beyond the header cap, else it may share its packet; a bundle source's controllers always ride in their own member stream); C-183 (even-tap linear-phase FIR rejected); C-184 and C-199 (transport state before any has arrived; default tempo 120 beats per minute); C-190 (device-side join with an odd domain-rate difference rejected, opt-out suggested); C-189 and C-197 (the DSL resampler node is fixed-ratio and built by the kit epic 024a; the rate actuator is not a DSL node); C-192 (float and fixed corpus variants count as one program; fuzzing tool left to the team); C-206 (the readable dump's format is this epic's presentation detail); C-209 (the controller member counts towards the 32 of C-08); C-218 (this epic implements every compile-time rejection; the code generator 024b only asserts it never receives such a program; the conformance corpus 025a tests it). The other rulings in the constants table (C-18, C-19, C-35, C-45 to C-47, C-94, C-96 except as noted under Open sub-questions, C-101 to C-103, C-107 to C-121, C-124, C-127 to C-129, C-131, C-135, C-136, C-138 to C-140, C-142, C-144, C-145, C-147 to C-157, C-159 to C-169, C-172 to C-175, C-177 to C-180, C-182, C-185 to C-188, C-191, C-193 to C-196, C-198, C-201 to C-204, C-208, C-210 except as noted, C-211, C-212) have no subject in this epic; C-151 (machine pair for bit-exactness) concerns task 025's verification of the class this epic assigns.

## Deliverables

Crate `hadronyche-dsl`:

- `syntax/` — lexer, parser, AST, source spans, diagnostics with stable codes.
- `types/` — content types (audio f32 / Q1.31, dense controller, sparse keyed controller, transport state), rates, channel counts, bundle layouts; checking, including the single-wire-rate and edge-module rules (C-01, C-105, C-134).
- `ir/` — a dataflow graph IR: nodes from a closed set (arithmetic, delay, table lookup, linear ramp, mix, the 2:1 / 1:2 resampler node, state cells), no unbounded loops, no recursion; the join-padding pass (C-104).
- `analysis/latency.rs` — structural L per output, as a whole number of domain-rate samples (C-105); flags non-constant delay; path latencies at each join; edge conversion delays, kept out of L (C-134).
- `analysis/cost.rs` — operation-count bound per slice per output and total, in abstract operations (C-106).
- `analysis/memory.rs` — state size and worst-case stack.
- `analysis/determinism.rs` — class assignment: all fixed-point ⇒ exact-fixed; any float node ⇒ exact-float; a DSL program is never bounded (C-123).
- `analysis/dataflow.rs` — per-output input dependency sets (for degraded marking, C-122).
- `manifest.rs` — manifest type and its serialisation (binary form defined with tasks 004 and 013, written down in `doc/design/spec/`; a readable dump for humans); a declaration path for hand-written modules.
- `LANGUAGE.md` — the language reference, including the diagnostic codes.

Shapes (names and integer widths may be refined):

```rust
pub struct Manifest {
    pub name: String,
    pub abi_version: u16,
    pub ports: Vec<Port>,                     // incl. bundle layout and channel map (C-04); a device-facing port carries its rate and conversion delay (C-134)
    pub slice_frames: u32,                    // frames per slice at the domain rate, 48 at defaults (C-02)
    pub state_bytes: u32,
    pub stack_bytes: u32,
    pub cost_ops_per_slice: u64,              // total, abstract operations (C-106)
    pub cost_per_output: Vec<OutputCost>,
    pub latency: Vec<OutputLatency>,          // per output, domain-rate samples (C-105)
    pub joins: Vec<JoinPadding>,              // inserted delays, opt-outs, frequency-dependent flag (C-104)
    pub determinism: DeterminismClass,
    pub gap_overrides: Vec<PortId>,
    pub dense_initial: Vec<DenseInitial>,     // initial value per dense controller input, default 0 (C-158)
    pub admission: AdmissionFigures,
}

pub enum DeterminismClass { ExactFixed, ExactFloat, Bounded } // Bounded: hand-written or fingerprint mismatch only (C-123)

pub struct Samples(pub u64);  // whole number of domain-rate samples (C-105)

pub struct OutputLatency { pub output: PortId, pub samples: Samples, pub non_constant_delay: bool }

pub fn check(source: &str) -> Result<(Ir, Manifest), Vec<Diagnostic>>;
```

## Dependencies

- **Needs:** content message definitions and the manifest binary layout (004); the admission units and manifest exchange (012); the stamp fields for determinism class (013).
- **Provides:** checked IR, per-output dependency sets and manifest to the code generator (024); the analyses whose outputs the conformance suite verifies (025). Every compile-time rejection is implemented here; the code generator (024b) only asserts it never receives a rejected program, and the conformance corpus (025a) tests the rejections (C-218).

## Acceptance criteria

1. **023-AC-1 — Rejections with a clear diagnostic** (all implemented in this epic, C-218): an unbounded loop or recursion; a call outside the kit (any transcendental); allocation; a cross-rate connection without the resampler node; a network port at a rate other than the domain rate (C-01); a resampler node outside an edge module (C-134); a resampler ratio other than 2:1 or 1:2 (C-105); a transport-authoring construct; a linear-phase FIR with an even number of taps, with a diagnostic suggesting an odd-tap design (C-183); a join on the device side of an edge module whose paths differ by an odd number of domain-rate samples, without the opt-out, with a diagnostic suggesting the per-join opt-out (C-190); a port wider than 32 × 6 = 192 channels, or an explicit bundle layout with more than 32 members or a member wider than 6 channels (C-04, C-08); a source with controllers whose audio needs more than 31 members, more than 186 channels (C-209).
2. **023-AC-2 — Latency (contributes to AC-33).** For a corpus of at least 20 programs (pure gain, explicit delay of N, lookahead limiter, linear-phase FIR of length 2k+1, block buffer of B, resampler 96→48 in an edge module, compositions of these including at least one join of unequal paths), the reported L, a whole number of domain-rate samples (C-105), equals the hand-derived value exactly; for the edge-module program the reported conversion delay equals the hand-derived value and is not included in L (C-134). A program with a recursive filter reports its structural L and sets `non_constant_delay`.
3. **023-AC-3 — Join padding (C-104).** For every corpus program with a join of unequal paths, the compiler inserts exactly the hand-derived delay on each shorter path, lists it in the manifest, and reports the longest path as L; with the opt-out construct on that join no delay is inserted and the manifest records the opt-out; a join fed by a recursive-filter path is padded on its structural part only and flagged in the manifest.
4. **023-AC-4 — Cost (contributes to AC-33).** For the same corpus the cost bound, in abstract operations per slice (C-106), is finite, and monotone under adding nodes. (That measured cost never exceeds it after calibration is checked in task 025.)
5. **023-AC-5 — Determinism class.** An all-fixed-point program is exact-fixed; adding one f32 node makes it exact-float; no corpus program is classified bounded (C-123); the classification is stable under reformatting.
6. **023-AC-6 — Dataflow.** For a two-input, two-output program where output A depends on input 1 only and output B on both, the dependency sets are exactly {1} and {1, 2}.
7. **023-AC-7 — Manifest.** Round-trips through its binary form; a 96-channel port yields a 16-member bundle with a 96-entry channel map (C-04); admission figures for a 6-channel stream are about 21.1 Mbit/s and 2 000 packets/s (C-21), and for the 96-channel port 32 000 packets/s and about 338 Mbit/s (C-22); a single-stream source whose headers plus controllers exceed the 100-byte cap counts a second stream for its controllers, one within the cap counts none unless it opts in to a second stream, which is always accepted (C-181); a bundle source with controllers counts one more member stream for them (C-205), within the 32 members of C-08 (C-209).
8. **023-AC-8 — Expressiveness.** The reference echo's instrument (voices driven by sparse keyed controllers, linear ramps over the slice, table sine, phase reset at gate zero) and a metronome reading transport state both type-check.
9. **023-AC-9 — Fuzzing.** The parser and the manifest decoder are fuzzed within the C-95 budget (each 60 s on every change and 1 hour nightly, corpus kept): zero panics, zero hangs.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] `LANGUAGE.md` covers every construct with one example
- [ ] The closed node set is listed with, per node, its latency, cost and determinism contribution

**Not applicable in this epic (C-125):** none. Every line has a subject here: the parsers are the DSL parser and the manifest decoder; the stamp field touched is the determinism class (task 013); the frozen POC line is checked once at close-out.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Concrete syntax (embedded in Rust as macros, or a standalone language) | This task (ticket 023-01) | Either form cannot express the reference echo without escape hatches |
| ~~Unit of the cost bound and its calibration to time per target~~ — **decided:** abstract operations per slice; per-target calibration factor in the regime stamp (C-106) | — | — |
| ~~Structural latency across the resampler node when rates are not integer multiples~~ — **decided:** L is always a whole number of domain-rate samples; the resampler is offered only at 2:1 and 1:2, other ratios are out of v1 (C-105); its conversion delay is counted only in the edge's D, never in L (C-134, C-207). The delay of the 2:1 / 1:2 design is fixed by the kit epic 024a (C-197), imported in ticket 023-11 and verified by task 025's probe suite | — | — |
| ~~Join rule for paths of different latency~~ — **decided:** pad, report, L = longest path, per-join opt-out (C-104) | — | — |
| ~~Default determinism class of hand-written modules~~ — **decided:** bounded unless the module claims exactness (C-123) | — | — |
| How non-DSL (hand-written) modules declare the same manifest | This task with task 012 (ticket 023-25) | A hand-written module cannot be admitted without a manifest |
| How an edge module declares its device-facing side and that side's rate, in the DSL and in the manifest | This task (tickets 023-01, 023-29) | An edge module for 48 kHz hardware cannot be written, or its manifest cannot be admitted, without an escape hatch |
| ~~How a join is padded when its path latencies differ by a fraction of a domain-rate sample~~ — **decided:** cannot arise; L is always whole (C-105) and padding uses a whole-sample delay line (C-104) | — | — |
| ~~Gap default for read-only transport state~~ — **decided:** hold the last known state, advance the playhead as that state dictates (a stopped playhead does not advance, C-200), mark the output degraded (C-141); implemented in ticket 023-30 | — | — |
| ~~State of a transport-state input before any has arrived~~ — **decided:** stopped, playhead at zero, default tempo (C-184); implemented in ticket 023-30 | — | — |
| ~~Value of the default tempo named by C-184~~ — **decided:** 120 beats per minute (C-199); implemented in ticket 023-30 | — | — |
| ~~Where the controllers of a multi-stream source travel~~ — **decided for a bundle source:** always in their own member stream (C-181, C-205); implemented in ticket 023-32 | — | — |
| ~~Whether the controllers' own member stream of a bundle source (C-205) counts toward the 32-member cap of C-08~~ — **decided:** it counts; a source with controllers has at most 31 audio members (186 channels) and a wider one is rejected at compile time (C-209); implemented in ticket 023-32 | — | — |
| Manifest binary layout (field order, widths, endianness, width of L) | Task 004 with task 013 | The layout cannot carry a 32-member bundle's channel map or the largest L without loss |
| Per-packet header breakdown behind C-20 (C-137 caps all headers and the checksum at 100 bytes; the field layout remains), and the admission units for input count and buffer memory (rings are sized to A, C-96, which is not known at compile time) | Task 012 with task 004 | Admission figures computed from the manifest disagree with task 012's admission check |
| Whether a bundle member may be wider than 6 channels (non-default slice length or the multi-packet regime, C-05) | Task 004 | A port the product needs cannot be expressed within 32 members of 6 channels |
| Stack model linking the IR's live values to the generated code's real stack | Task 024 | Measured stack of generated code exceeds `stack_bytes` |
| Value of `abi_version` and its versioning policy | Task 012 | A runtime admits a manifest whose ABI it does not implement |
| Verification of declared cost, L, state and stack for hand-written modules | Task 025 with task 012 | A hand-written module exceeds a declared figure without being detected |
