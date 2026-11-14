# 025a-01 — Conformance harness and expect format
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** epic 023, epic 024
## Context
Epic 025a proves that the DSL compiler's promises hold and keeps proving it in CI. `[decision]` The DSL moves trust from each module author to the compiler, so the compiler needs its own conformance suite.

This ticket creates the skeleton every later conformance ticket plugs into: the `conformance/` directory in crate `hadronyche-dsl`, the per-program expectation file format in `expect/`, and a test harness that compiles one program and compares the compiler's outputs with the expectation.

Compiler promises that an expectation file must be able to state:
- **Structural latency L** per output: always a **whole number of domain-rate samples**, as in the manifest and ABI (C-105; the earlier rational-L rule is withdrawn). It sums explicit delays, lookahead, block buffering, linear-phase filter centres; it is the module's structural delay in the domain-rate timeline and never includes an edge's resampler delay (C-207). An output reached through a recursive filter is additionally flagged as having **non-constant (frequency-dependent) delay**, a property beside its structural L (C-104). The runtime stamps output at p − L.
- **Determinism class**: `exact-fixed` (bit-exact everywhere, including host to device) or `exact-float` (bit-exact only where the startup fingerprint matches). The compiler never assigns `bounded` to a DSL program: the DSL has no non-deterministic node in v1, and bounded arises only at run time from a fingerprint mismatch or for hand-written modules (C-123).
- **Dependency sets** per output (which inputs each output depends on, from dataflow).
- **Inserted join delay**: where paths of different latency meet, the compiler pads the shorter path with a whole-sample delay line and reports the inserted delay in the manifest; L is the longest path; an explicit construct opts out per join. A path with frequency-dependent delay is padded on its structural part only and the join is flagged in the manifest (C-104).
- **Conversion delay** of an edge module: the resampler delay, counted only in the edge's plumbing delay D, which the manifest reports separately, on the device-facing port (C-207, C-134).
- **Cost bound**: a finite upper bound per slice in counted abstract operations, identical on every machine (C-106).
- **Initial value** of each dense controller input, declared in the manifest (default 0) and used when a gap precedes any received value (C-158).
- For rejected programs (negative corpus, ticket 025a-07): the expected diagnostic.

Constants: domain rate 96 kHz, the only rate on the wire (C-01); slice = 48 frames = 0.5 ms (C-02); 4-byte samples (C-03); 6 channels per stream (C-04).

The format is a specification and is written to `doc/design/spec/` as well as the directory README (C-130). Fuzz budget: each parser 60 s on every change and 1 hour nightly, corpus kept (C-95).
## Work
1. Create `hadronyche-dsl/conformance/` with sub-directories `corpus/`, `expect/`, `vectors/` (empty except a README stating their purpose).
2. Define the expect file format (one file per corpus program, same base name): per output `L` (a whole number of domain-rate samples, C-105) and an optional `non-constant` flag for frequency-dependent outputs (C-104), `class` (`exact-fixed` | `exact-float`; `bounded` is rejected by the parser, C-123), `deps` (set of input names), `joins` (per join: inserted delay in whole samples, or `opt-out`, plus a `flagged` marker when one path has frequency-dependent delay, C-104), `initial` (per dense controller input, the declared initial value, C-158), `conversion_delay` (edge modules only: per device-facing port, the conversion delay the manifest reports there, in whole samples, C-207), `cost_bound` (finite integer, abstract operations per slice); for negative programs, `reject` with the expected diagnostic id. Choose a plain text format (e.g. TOML) and document it.
3. Write the expect-file parser and a fuzz target for it.
4. Write a test harness (e.g. `hadronyche-dsl/tests/conformance.rs`) that, for every `corpus/*.hdsl`, compiles with the epic 023/024 compiler and compares L (integer equality) and the non-constant flag, class, dependency sets, join delays and join flags, dense-controller initial values, edge conversion delays per device-facing port and the presence of a finite cost bound against `expect/`. A program without an expect file is a failure.
5. Add two seed programs (one float gain, one fixed gain) with expectations, so the harness has something to run.
## Acceptance criteria
1. `cargo test -p hadronyche-dsl --test conformance` compiles both seed programs and passes.
2. Changing any one expected field (L, the non-constant flag, class, one dependency, a join delay or flag, an initial value, or removing the cost bound) in a seed expect file makes the harness fail with a message naming the program, the output and the field.
3. A corpus program with no expect file fails the harness.
4. A cost bound that is absent or not finite fails the harness.
5. An expect file stating a non-integer L (for example `1/2` or `0.5`) is rejected by the parser with a message citing C-105.
6. An expect file stating `class = bounded` is rejected by the parser with a message citing C-123.
7. A `conversion_delay` in the expect file of a program that is not an edge module is rejected by the parser; the harness fails when an edge module's reported conversion delay differs from the expected one, naming the program, the port and the field (C-207).
8. The expect parser fuzz target runs 60 s on every change and 1 hour nightly without a crash, and its corpus is kept (C-95).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Expect format documented in `conformance/README` and in `doc/design/spec/`
## Out of scope
- The real corpus (025a-02 to 025a-06), the negative corpus (025a-07), exact vectors (025a-09), the CI job (025a-10), probe comparison of L (025a-15), executed operation count (025a-11), measured time (025b-10).
- Counting the corpus towards 40 (025a-05); float and fixed variants of a program count as one program (C-192).
