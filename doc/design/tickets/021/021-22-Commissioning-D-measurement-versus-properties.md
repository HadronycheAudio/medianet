# 021-22 — Commissioning measurement of D versus device properties
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-14, 021-15, 021-16, 021-17, 021-31, epic 020
## Context
D = device latency + stream latency + safety offset + buffer size properties, plus the kit resampler node's conversion delay, a whole number of domain-rate samples counted only in D and never in L (C-80, C-105, C-134, C-207), **plus a commissioning measurement** [memory]. A device whose driver misreports latency is uncertifiable without a commissioning measurement.

Open sub-question owned by epic 021:

| Question | Owner | Falsifier |
|---|---|---|
| Do device-reported latency properties match the commissioning measurement? | This task (L2 macOS) | Mismatch larger than 20 µs, the D share of the macOS E budget (C-150): D must be measurement-only, as on Windows |

Target E = 100 µs (C-71). The proposed macOS budget split is 40 µs clock + 40 µs tracking + 20 µs uncertainty of D, pending L2 (C-150); the D share is therefore 20 µs.

- Commissioning activity [decision, C-107]: the commissioning tool may drive test traffic only inside a declared commissioning window that every module stamps; read-only outside it. Any part of this measurement that sends traffic on the medianet runs inside such a window. The tool announces the window with an expiry time and it closes at expiry or on explicit close (C-153). The window makes ordinary production measurements unqualified, but this measurement of D is the commissioning tool's own and is a valid commissioning result, recorded with the window it was taken in (C-178).
- The reference devices are those approved under C-172 (021-31).
## Work
- Using the epic 020 harness and `CoreAudioRawCapture` (loopback cable output→input), measure actual D for sink and capture on each reference device, inside a declared commissioning window.
- Store the result as a commissioning override, recorded with the commissioning window it was taken in (C-178); `DeclaredD.method` becomes `Commissioned`, used in preference to the property sum (021-14).
- Compare measured vs property-derived D per device; record the difference with `[measured]` tags in `doc/design/reports/` (C-130). If the difference exceeds 20 µs (C-150), switch macOS to measurement-only D and update the design documents, constants table first.
## Acceptance criteria
1. Measured and property-derived D reported for sink and capture on every reference device, each `[measured]`.
2. With an override present, the stamp shows D method = commissioned.
3. The measurement run appears in every module's stamp as an open commissioning window, and no test traffic is seen outside it (C-107); the stored override names that window (C-178).
4. The sub-question row is closed with the result, or the falsifier (mismatch above 20 µs, C-150) is recorded and D policy changed to measurement-only.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Restart-spread and E certification runs (021-23). Opening and closing commissioning windows (commissioning tool, epic 028).
