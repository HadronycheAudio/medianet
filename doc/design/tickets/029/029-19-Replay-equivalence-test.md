# 029-19 — Replay equivalence test
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-11, 029-12, 029-13, 029-14, 029-15, 029-16, 029-17, 029-18, 029-24, 029-25

## Context
- Epic 029 criterion 029-AC-6: **the monitor renders identically from a recorded stamp file and from the live network.**
- The live source (029-03) reads hellos and stamps (multicast on any regime change and every 10 s, C-18) from the discovery and control groups; the file source (029-04) reads a file recorded by the hosted stamp collector (epic 015). Rendering depends only on model state and observation time (029-10).
- Open question in the epic, owned by epic 015: whether the recording contains hellos. Working assumption: yes.

## Work
1. Integration test: run a short scenario on the test medianet; simultaneously (a) feed `LiveSource` into a model and capture `render_to_text` of all seven panels at fixed observation times, and (b) let the epic 015 collector record the stamp file.
2. Replay the file through `FileSource` and capture the same panels at the same observation times.
3. Compare captures byte for byte; also compare headless output (029-18).

## Acceptance criteria
1. All seven panels (Medianet, Clock, Connections, Bundles, Ports, Sinks, Regime) and headless output are byte-identical between live and replay at every capture point, over a run of at least 60 s (six stamp refreshes, C-18) with at least one regime change.
2. The test runs in CI unprivileged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- File format (epic 015). If epic 015 answers that the recording lacks hellos, hello-derived fields are excluded from the comparison only with an explicit, reviewed exception list.
