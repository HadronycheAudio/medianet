# 028b-05 — Platform reference list and unverified combinations
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-02, epic 020, epic 021, epic 022

## Context
Group-E targets are certified only for known platform combinations. Combinations of **OS build, driver and audio device** outside the reference list are recorded as *unverified*. Stamps carry a platform section per module. Reference E targets: Linux reference sink 50 µs (C-70); macOS 100 µs (C-71); Windows WASAPI exclusive 1 ms (C-72). Edge hardware at 48 kHz converts 2:1 at the edge through the kit's resampler node (C-80). A resampler node exists only inside an edge module and only for 2:1 and 1:2 (C-105, C-134); its conversion delay is part of the edge's plumbing delay D (C-134, C-170), so such a device is certified with its conversion in place.

Where the list comes from: reference audio hardware is proposed by the implementing team after P0, approved by the product owner and recorded in the constants table (C-172); supported OS versions are set after the receive-stamp probe task (001) reports, covering the current release and the oldest obtainable one of each OS (C-138).

## Work
- Data file `reference-platforms.toml` in `hadronyche-tools` listing verified (OS build, driver, audio device) tuples with source (the constants-table entries of C-172 and C-138, epic 020 Linux record, epics 021/022 findings, reports in `doc/design/reports/`, C-130) and evidence tag. The file is a copy of what the constants table records, never a source of its own.
- Parser for that file, fuzzed per the fuzz budget (C-95).
- Finding `platform.unverified` in `survey`: a module whose stamp platform tuple is not in the list; name node and tuple.
- Expose `is_verified(tuple)` for certificates (028b-10, 028b-12).

## Acceptance criteria
1. A fixture stamp with an unlisted audio device yields `platform.unverified` naming the node and the tuple (epic AC-7, part).
2. A listed tuple yields no finding.
3. Every entry of the reference file carries a source and an evidence tag.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Measuring a new combination (028b-12).

## Notes
- The list's contents come from the measurements of epics 020, 021 and 022 and from the C-172 and C-138 entries once recorded in the constants table (remaining gap, owners those epics and task 001). Which stamp field carries the platform tuple is not yet specified (remaining gap, owner epic 013).
