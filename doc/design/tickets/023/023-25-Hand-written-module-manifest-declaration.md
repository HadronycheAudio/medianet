# 023-25 — Manifest declaration for hand-written (non-DSL) modules
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-20, 023-21, epic 012

## Context
- Open sub-question owned by **this task with epic 012**: how non-DSL (hand-written) modules declare the same manifest. Falsifier: **a hand-written module cannot be admitted without a manifest.**
- Manifest contents: module name; ABI version; ports (direction, content type, rate, channels, bundle layout and channel map); slice sizes; admission figures; state size; stack bound; cost bound in abstract operations (C-106); structural latency L per output as a whole number of domain-rate samples (C-105); non-constant-delay property; join padding (C-104, normally empty for hand-written code); for an edge module, the conversion delay of each device-facing port, part of D (C-134); determinism class; gap overrides; dense controller initial values (C-158).
- Determinism class (C-123, decided): **a hand-written module is `Bounded` unless it claims exactness**; a claimed exact class is then verified by the exact tier (record and replay, epic 025).
- For DSL modules the compiler derives the figures; a hand-written module declares them. How declared cost, L, state and stack are verified is a remaining gap owned by **epic 025 with epic 012** (falsifier: a hand-written module exceeds a declared figure without being detected).
- The same port rules apply: network ports at the domain rate (C-01); rate conversion only in edge modules (C-134), at 2:1 or 1:2 (C-105); bundles ≤ 32 members of ≤ 6 channels, a controller member included, so at most 31 audio members with controllers (C-04, C-08, C-209); ≤ 256 streams per source (C-09).
- Output location (C-130): the declaration mechanism is a specification in `doc/design/spec/`.

## Work
1. Design and document, with epic 012's owner, the declaration mechanism in `doc/design/spec/`: a builder API in `hadronyche-dsl::manifest` (`ManifestBuilder`) producing the same `Manifest` and binary form, reusing 023-07 bundle layout and 023-20 admission computation.
2. Default the determinism class to `Bounded`; accept `ExactFixed` or `ExactFloat` only as an explicit claim, recorded as such in the manifest documentation.
3. Implement the builder with validation (same port rules as the checker).

## Acceptance criteria
1. A hand-written 6-channel module's manifest built with the builder round-trips through the binary form of 023-21.
2. The builder rejects a 200-channel port, a network port at 48 kHz and a 192-channel source with controllers (C-209), like the DSL checker.
3. A builder with no class given yields `Bounded`; one claiming `ExactFixed` yields `ExactFixed`.
4. The design note records the mechanism, owner (epic 023 with epic 012) and falsifier, and names the verification gap with its owner (epic 025 with 012).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Admission enforcement for hand-written modules (epic 012); verifying declared figures (epic 025).
