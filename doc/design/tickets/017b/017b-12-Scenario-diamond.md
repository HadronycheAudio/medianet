# 017b-12 — Scenario `diamond`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-03, 017a-07, 017a-08, 017a-09, 017b-09

## Context
- AC-10 (diamond): the source feeds the mixer directly and through one processor with injected per-hop delay; **mixer output equals the offline reference bit for bit**.
- **The processor (settled in the epic):** the reference echo. This is the POC's own topology: the sender's audio feed reaches the mixer directly, and the sender's parts reach the mixer through the echo (017a-07). The offline reference is `mix_offline(feed, voices_offline(parts), volume)`, using `mix_offline` from 017a-08 and the offline rendering of `instrument::voices` (017a-01). The mixer's volume is the reference dense controller (C-192).
- The reference mixer is a position-aligned fan-in firing at `T(p) + offset` `[decision]`; the offset is measured during aligning (4 s, C-50) from the observed lateness plus headroom (C-51, C-52), so it absorbs the injected hop delay.
- Per-hop delay injection: `Injector::set_hop_delay(module, delay_ns)` (017b-03).
- Never assert latency thresholds on loopback `[decision]`.

## Work
1. Scenario `diamond`: sender → mixer (feed) and sender → echo → mixer (parts, then audio); `set_hop_delay(echo, d)` with `d = 1 ms`, and a second run with `d` varied across [0.1, 2] ms, set before aligning.
2. Assertion: for every non-degraded mixer output slice, output equals the offline reference bit for bit; degraded slices = 0.

## Acceptance criteria
1. `hadronyche-l0 diamond --duration 30` exits 0; mixer bit mismatches = 0; degraded slices = 0.
2. Same result with a hop delay that changes per run within [0.1, 2] ms.
3. Removing the alignment (test build with on-arrival mixing) makes the scenario fail (proves the test is sensitive).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Delays beyond the offset (017b-13).
