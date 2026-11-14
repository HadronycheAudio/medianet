# 024b-02 — Decide subnormal policy for exact-float
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024a-08

## Context
The epic's open sub-question "Subnormal policy for exact-float: forbid by construction (keep values away), or rely on the fingerprint alone?" is owned by this task, with falsifier: "A real program produces subnormals in normal operation".

Facts `[decision]`: results can differ across machines through non-IEEE subnormal handling (flush-to-zero or denormals-are-zero modes, set per thread). The fingerprint is evaluated on the processing thread at start; match ⇒ exact-float eligible; mismatch ⇒ bounded-class. Fingerprint vectors cover subnormal and NaN-producing paths. Exactness between a host and a small device is promised only for exact-fixed.

## Work
- Evaluate both options on representative DSL programs (gain, mixer, ramped controller, resampler float path if any, a decaying filter/feedback structure): count subnormals produced in normal operation (instrumented hosted run, written as a probe in `probes/`, C-133).
- Decide: (a) forbid by construction (e.g. generator inserts a flush-to-zero-by-arithmetic step or offset on feedback paths, defined bit-exactly), or (b) rely on the fingerprint alone.
- Write the decision note in `doc/design/reports/` (C-130) with the measurements `[measured]` and the consequence for 024b-04 (lowering) and 024b-11 (fingerprint synthesis).
- Record an open item with owner and falsifier for the falsifier above.

## Acceptance criteria
1. A measurement table lists, per test program, the number of subnormal values produced over 10⁴ slices of a typical input, tagged `[measured]`.
2. The decision note names the chosen policy and the exact rule (if (a): which constructs get which bit-exact treatment).
3. The decision note states the exact consequence for codegen lowering (024b-04) and fingerprint synthesis (024b-11), in terms an implementer can apply without further decision.
4. The open item is recorded with owner and falsifier.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Fingerprint coverage of every differing path (owned by task 025).
- Implementing the policy in codegen (024b-04, 024b-11).
