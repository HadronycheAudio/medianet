# 018-27 — Conversion delay in the manifest, outside L (C-207)
**Epic:** 018-Linux-ALSA-edge · **Effort:** XS · **Depends on:** 018-11, 018-12, 018-16, epic 012

## Context
- **C-207:** an edge's resampler delay is counted only in D and never in L. L is the module's structural delay in the domain-rate timeline; the manifest reports the conversion delay separately, on the device-facing port.
- C-134: a resampler node may appear only inside an edge module, and an edge's conversion delay is part of its plumbing delay D (C-170), never counted in L (C-207). C-105: L is always a whole number of domain-rate samples.
- The resamplers are the kit's (fixed 2:1 / 1:2 node and steerable rate actuator), built by epic 024a; this epic builds none (C-197). Their delays enter D through the resampler part of `DeclaredDelay` (018-11).
- The manifest is carried at admission (epic 012); `AlsaSink` (018-12) and `AlsaCapture` (018-16) each present one.

## Work
1. In the manifests of `AlsaSink` and `AlsaCapture`, report the conversion delay (the resampler part of `DeclaredDelay`, 018-11) as its own figure on the device-facing port (C-207).
2. Keep that delay out of the module's declared structural latency L (C-207, C-134); L stays a whole number of domain-rate samples (C-105).
3. Rustdoc on the manifest fields, citing C-207.

## Acceptance criteria
1. On a fake 48 kHz device and a fake 96 kHz device, the manifest of each module reports on the device-facing port a conversion delay equal to the resampler part of `DeclaredDelay` (018-11).
2. The declared L of each module is the same for the 48 kHz and the 96 kHz device: no resampler delay is counted in L (C-207).
3. D still includes the conversion delay (`DeclaredDelay::total_ns()` unchanged, C-134, C-170).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The manifest format itself (epic 012).
- The resamplers and their delays (epic 024a, C-197).
- Writing D and its components to the regime stamp (018-18).
