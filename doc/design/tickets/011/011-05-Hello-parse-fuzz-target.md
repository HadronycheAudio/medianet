# 011-05 — Hello parse fuzz target
**Epic:** 011-Gossip-and-discovery-core · **Effort:** XS · **Depends on:** 011-03

## Context
Definition of done of epic 011: every parser touched is fuzzed. Deliverable: a fuzz target for `Hello::parse`. Hellos arrive from the network (IPv4 multicast, TTL 1, C-100) and are at most 1 252 bytes (C-17). `Hello::parse(bytes: &[u8], my_domain: Domain) -> Result<Hello, Reject>` must never panic.

C-192 (decided): the fuzzing tool is the implementing team's choice. C-95 (decided) sets the fuzz budget: each parser is fuzzed 60 s on every change and 1 hour nightly, and the corpus is kept.

## Work
1. Add a fuzz target `hello_parse` (for example under `hadronyche-core/fuzz/`) that calls `Hello::parse` on arbitrary input, with `my_domain` taken from the first two input bytes.
2. Inside the target, when parse succeeds, re-encode via a `LocalInfo` built from the view and assert the re-parse yields the same fields (round-trip oracle).
3. Seed the corpus with the golden hellos from 011-02 and the truncations from 011-03.
4. CI: run the target 60 s on every change to the hello code, and 1 hour in the nightly job (C-95).
5. Keep the corpus: persist it between runs (checked in or stored as a CI artefact restored at the start of each run), and add every crashing input to it as a regression case.

## Acceptance criteria
1. The fuzz target builds and runs unprivileged.
2. The per-change CI job runs the target for 60 s; the nightly job runs it for 1 hour (C-95).
3. A 1-hour run finds no panic, no out-of-bounds read and no round-trip mismatch.
4. The corpus produced by one run is the starting corpus of the next.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (no behaviour change); "No new open item without an owner and a falsifier" (opens no item); "The frozen POC is not modified" (fuzz harness only); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Fuzzing election messages: 011-13.
