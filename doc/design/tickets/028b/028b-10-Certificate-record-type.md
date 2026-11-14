# 028b-10 — Certificate record type
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028a-01, epic 013

## Context
The tool's output, besides the configuration, is a **membership certificate** per presentation group, stored beside the collected stamps in the deployment's data directory (C-171). Record fields: group, A, required E, per sink E and D with method and date, hardware identity, tool version.
- E = 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, with the maximum beside it (C-73).
- Only the commissioned state gives a numeric E; uncertified and self-monitored sinks declare E as unknown (C-140, C-103). A group may set no required E.
- Methods: L2 marker loop (single sink, epic 020), L3 two-channel capture (pair, epic 027), or self-monitored only; a self-monitored-only record carries E unknown (C-140).
- The tool's own measurements (D, restart spread, group certification) are valid commissioning results even though they are taken inside a commissioning window; each record carries the window it was taken in (C-178).
- D restart spread over 20 restarts (C-74) is recorded by `certify-sink`; an embedded device's limit is its own declared restart spread, measured at L3 (C-191).
- "Both stamps" in a certification record are the regime stamps of the sink and of the holder it followed (C-191); after a takeover during the run, which keeps the epoch (C-213), that is each holder followed.
- A sink record stays valid while the sink's clock is unlocked: the sink stays commissioned, its alignment claim is suspended and does not count towards a group's required E until the clock locks again, then resumes without re-commissioning (C-180). The record carries no clock state.
- Pairwise skew (L3): 99.9th percentile of absolute skew with the maximum beside it, over 1 hour and across restarts (C-79).

## Work
- Type `Certificate { group, a_ms, required_e_us: Option, sinks: Vec<SinkRecord>, pairs: Vec<PairRecord>, tool_version, created }` with `SinkRecord { sink, e_p999_us: Option, e_max_us: Option, d_us, d_spread_us, method, date, hardware_identity, platform_verified: bool, sink_stamp, holder_stamps /* one per holder followed, C-191, C-213 */, window }` and `PairRecord { a, b, skew_p999_us, skew_max_us, date, window }`. A record with method self-monitored has `e_p999_us` and `e_max_us` absent (C-140).
- Serialization (versioned) and parser; fuzz target run per the fuzz budget (C-95).
- `store` / `load` writing to the deployment's data directory, beside the stamps the tool collects (C-171); `--data-dir` selects it.

## Acceptance criteria
1. Round trip `parse(serialize(c)) == c` for fixtures, including a sink with unknown E and a group with no required E.
2. A certificate missing any mandatory field listed in Context is rejected by the parser.
3. Stored certificates can be listed and loaded back by group name from the deployment's data directory, next to the collected stamps (C-171).
4. A self-monitored-only sink record with a numeric E is rejected (C-140).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Producing certificates (028b-12, 028b-14).
