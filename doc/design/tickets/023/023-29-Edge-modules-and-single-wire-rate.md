# 023-29 — Edge modules, device-facing ports and the single wire rate
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-08

## Context
- Rules (decided):
  - **The domain rate, 96 kHz, is the only rate on the wire: every stream on a medianet is at the domain rate** (C-01). Every network port of every module is therefore at 96 kHz.
  - **A resampler node may appear only inside an edge module; the compiler rejects it anywhere else** (C-134). An edge whose hardware runs at 48 kHz converts 2:1 through the kit's resampler node; the wire stays at the domain rate (C-80).
  - **The resampler is offered only at 2:1 and 1:2; other ratios are out of v1** (C-105). A device-facing port is therefore at 48 kHz (through a resampler) or at 96 kHz (no resampler).
  - An edge's conversion delay is counted only in its plumbing delay D (C-134, C-170) and never in L (C-207); the manifest reports it separately, on the device-facing port, so the edge can include it in D (C-207).
- Open sub-question owned by this epic (with 023-01): how an edge module declares its device-facing side and that side's rate, in the DSL and in the manifest. Falsifier: an edge module for 48 kHz hardware cannot be written, or its manifest cannot be admitted, without an escape hatch. The grammar part is decided in 023-01; this ticket decides the manifest representation and implements the checks.
- Device-facing ports do not touch the wire: they carry no admission figures (023-20) and no bundle layout constraint from C-08 beyond what the hardware side needs.
- 023-AC-1 includes: a network port at a rate other than the domain rate; a resampler node outside an edge module.

## Work
1. Checker pass over the typed AST (from 023-08): network port with any rate other than 96 kHz → diagnostic citing C-01; resampler in a module without the edge-module marker → diagnostic citing C-134; a device-facing rate whose ratio to 96 kHz is not 2:1 or 1:2 → diagnostic citing C-105; device-facing port in a non-edge module → diagnostic.
2. Inside an edge module, require that every path between a device-facing port and a network port at a different rate passes through a resampler (reusing 023-08's cross-rate rule).
3. Decide and write down in `doc/design/spec/` (C-130) the manifest representation of an edge module and its device-facing ports (direction, content type, channels, rate, conversion delay in whole domain-rate samples from 023-15, C-134); hand it to 023-19.

## Acceptance criteria
1. A non-edge module declaring a 48 kHz network port → 1 diagnostic citing C-01.
2. A non-edge module containing a resampler → 1 diagnostic citing C-134.
3. An edge module with a 48 kHz device input, a 2:1 resampler and a 96 kHz network output checks clean; without the resampler → 1 cross-rate diagnostic.
4. A 44.1 kHz device input → 1 diagnostic citing C-105 (ratio out of v1).
5. The spec note records the manifest representation, with owner (this epic) and falsifier.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The edge-module syntax (023-01, 023-04); resampler delay (023-11); admission figures (023-20); the edge runtime and its hardware backend (other epics).
