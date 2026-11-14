# 024b-06 — Codegen: join padding delay lines
**Epic:** 024b-DSL-code-generator · **Effort:** M · **Depends on:** 024b-04, 024b-05, epic 023

## Context
Rule (C-104): where DSL paths of different latency meet at a join, the compiler pads the shorter path with a whole-sample delay line, reports the inserted delay in the manifest, and the module's L is the longest path; an explicit DSL construct opts out per join. A path with frequency-dependent delay is padded on its structural part only and the join is flagged in the manifest. Latency is always a whole number of domain-rate samples (C-105), so every latency difference at a join is a whole number of samples. Which nodes have frequency-dependent delay, and their structural part, comes from the checked IR analyses (epic 023; an open item owned by epic 023). Per-path latencies come from 024b-05. The opt-out construct is parsed by the front end (epic 023; its syntax is an open item owned by epic 023) and arrives in the checked IR.

Rule (C-190): on the device side of an edge module, paths that differ by an odd number of domain-rate samples cannot be padded with a whole-sample delay line at the device rate; the compiler rejects the join and suggests the per-join opt-out. The front end (epic 023) implements that rejection; the code generator only asserts it never receives such a join, and the conformance corpus (epic 025a) tests it (C-218). Edge hardware at 48 kHz converts 2:1 (C-80).

Rules for generated code `[decision]`: no allocation, never blocks, state size is a compile-time constant. Slice 48 frames (C-02); 4-byte samples (C-03); 6 channels (C-04).

Epic acceptance criterion 024b/AC-8 (join padding, including the device-side join of C-190).

## Work
- `hadronyche-dsl/src/codegen/joins.rs`: for each join in the IR without opt-out, compute the difference between the longest incoming path latency and each other path's latency; insert a fixed-size delay line of that many samples per channel on each shorter path, held in `State` and counted in `state_bytes`.
- For a path containing a frequency-dependent-delay node, use only its structural latency in the difference, and record a flag on that join.
- Record each inserted delay (join id, path, samples) and each frequency-dependent-join flag for the manifest (written by 024b-12).
- With opt-out on a join, insert nothing.
- For a join on the device side of an edge module without opt-out: assert that the paths differ by an even number of domain-rate samples (an odd difference is an internal compiler error naming C-190, the front end having rejected it, C-218) and pad with a whole-sample delay line at the device rate.
- Complete the padded-join program of the epic's corpus (024b-04).

## Acceptance criteria
1. A program where a direct path and a path of latency 96 samples meet at a mix: the direct path is delayed by 96 samples; an impulse on the shared input appears once, aligned, in the output.
2. The inserted delay appears in the manifest data handed to 024b-12, and the output's L equals the longest path (96).
3. With the opt-out construct on that join, no delay is inserted, the impulse appears twice, and the manifest reports no inserted delay for that join.
4. A join where one path contains the corpus's decaying feedback structure (frequency-dependent delay) is padded on structural latency only, and the manifest data flags that join.
5. In variants of the corpus's edge module with a 2:1 resampler, a device-side join whose paths differ by an odd number of domain-rate samples compiles with the opt-out on that join; a hand-built IR that bypasses the front end with such a join and no opt-out trips the assertion as an internal compiler error (C-218); a device-side join whose paths differ by an even number is padded at the device rate and the impulse appears once, aligned (C-190).
6. The `const` state-size assertion passes with the delay lines included, and the module runs under the aborting allocator.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Parsing the opt-out construct and rejecting an odd-difference device-side join without it (epic 023, C-218).
- Computing per-path latencies (024b-05).
- Writing the manifest (024b-12).
