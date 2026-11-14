# 017b-09 — `hadronyche-l0` runner, record file and inert-family failure
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** M · **Depends on:** 017b-05, 017a-04, epic 013, epic 015

## Context
- Binary `hadronyche-l0` runs a named scenario for a duration, writes **one record file** with its stamp, exits non-zero on any failed assertion. It is a harness binary, so it lives in the `hadronyche-tools` crate (C-133); the reference modules it drives stay in `hadronyche-reference`.
- Scenarios (implemented in later tickets): `baseline`, `diamond`, `loss`, `late-vs-lost`, `gap-classes`, `realign`, `epoch`, `blackhole`, `flood`, `refusal`, `collision`, `malformed`, `partition`, `housekeeping`, `thread-death`.
- AC-9: a scenario whose transport or music family produces zero samples exits non-zero and names the family. Rule `[decision]`: a check family with zero samples is a failed run.
- AC-31: every record file carries a complete stamp; injection is labelled (017b-05); **one writer per file**. POC bug `[measured]`: four `File::create` handles on one metrics CSV path (`module.rs:1354`) corrupted it.
- "Complete stamp" (settled): every field defined by epic 013, transported in sections per C-18, including the per-connection counters of producer and consumer (C-47, completed by C-159: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead), the lateness percentiles (50th, 99th, 99.9th and maximum, C-160) over a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182), the holder's per-follower count of sync requests dropped above 200 per second (C-204), the observed DSCP on peers' hellos and, at the holder, on each follower's sync requests (C-152), the capability states (C-129) and the receive-timestamp source (C-131).
- **Single writer (settled, C-144):** the single-writer file service is the hosted backend's (epic 015). The runner writes its record file through that service rather than building its own.
- **Strict mode (settled, C-127, C-128, C-147):** L0 does not run in strict mode, and its numbers never enter the constants table. Real-time scheduling and memory locking are configured "not requested" and stamped as such (C-129), never left absent or unverified by accident. Non-strict does not relax seqlock safety: if a thread layout's seqlock safety depends on a CPU pin and that pin fails or is unverified, the module refuses to start in every mode (C-177, C-201); the runner reports such a refusal as a failed run, naming it.
- Sync rate (settled, C-112): the accepted sync request rate is 10 to 200 per second. How the rate is configured is owned by epic 015 (open sub-question); this runner passes it through that mechanism.
- Operator policy (strict mode, per-input alignment opt-out, pacing override) is set in the commissioning configuration (C-165); the runner builds that configuration for each scenario, including the mixer's explicit alignment budget (C-142).
- Rule `[decision]`: never assert latency thresholds on loopback; no assertion depends on sync-rate timing.
- **Open sub-question owned by this ticket:** the record-file format, layered on epic 013's stamp encoding. Choose it, document it and propose it through 017b-26. Falsifier: a later rung's tooling (epics 019, 020, 027) cannot read an L0 record file. Because later epics read it, the format goes into the specifications; the runner's other presentation details (output line formats, file names within the stated directories) are this ticket's decision, recorded in the tool's documentation only (C-206).

## Work
1. `hadronyche-tools/src/bin/hadronyche-l0.rs`: CLI `hadronyche-l0 <scenario> --duration <secs> --out <path> [--sync-rate <per-s>]`; a sync rate outside 10..200 is rejected (C-112).
2. Scenario trait, e.g. `trait Scenario { fn name(&self) -> &'static str; fn setup(&mut self, rt, inj: &dyn Injector); fn assertions(&self, report) -> Vec<AssertionResult>; fn required_families(&self) -> &[Family]; }`, and a registry keyed by the 15 names; unknown names exit with code 2 and list valid names.
3. Configure every module non-strict with real-time scheduling and memory locking "not requested" (C-127, C-129).
4. Record file: written through epic 015's single-writer file service (C-144), opened with exclusive create; write the full stamp (all sections, with injection fields from 017b-05) first, then results. A second open on the same path fails. Document the format.
5. After the run: if any required family (at least `transport` and `music` for scenarios using the sender) has zero compared samples, fail with a message `inert family: <name>`; then evaluate assertions; exit 0 only if all pass.
6. A trivial built-in `self-test` scenario for testing the runner, and a record-file reader used by tests and by 017b-24's fuzz target.

## Acceptance criteria
1. `hadronyche-l0 self-test --duration 1` exits 0 and writes exactly one record file whose stamp parses and contains `injected`, the C-47 and C-159 counters, the C-160 lateness percentiles with the maximum since live beside them (C-182), the C-204 per-follower over-rate sync request counts, the C-152 observed DSCP values, the C-129 capability states (real-time scheduling and memory locking "not requested") and the C-131 receive-timestamp source.
2. A test scenario forcing the music family to zero samples exits non-zero and stderr contains `inert family: music` (AC-9); same for `transport`.
3. A failing assertion exits non-zero and names the assertion.
4. Attempting a second writer on the same record path fails; the file is never corrupted (test runs 4 concurrent writers attempting the path, exactly one succeeds).
5. Unknown scenario name exits with code 2; `--sync-rate 5` and `--sync-rate 201` are rejected.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Individual scenarios (017b-10 to 017b-23).
- The single-writer file service itself (epic 015, C-144).
- CI wiring and time budget (017b-25).
