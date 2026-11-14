# 013-05 — StampId derivation
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-02, 013-04

## Context
- `pub struct StampId(pub u32);` with `Stamp::id(&self) -> StampId`. `[decision]`
- Rule 1: a change of regime mid-run starts a new record set; figures are never averaged across regimes. A timeline epoch change is a regime change. `[decision]`
- **Regime fields and counters (epic 013, settled from C-18 and C-47).** The stamp carries two kinds of content. Regime fields: Identity, Capabilities, Network (including per-connection entries), Clock, Alignment (except its measured values), Platform, Injection (including the commissioning window), self-test. Counters: the Drops group, the per-connection summaries of C-47 and C-159, the per-peer observed DSCP entries of C-152 (013-26), and the Alignment group's lateness percentiles (C-160) and reclaimable latency (C-157), written in their own counter section (013-08). Counters are refreshed every 10 s (C-18) and are not a regime change. The `StampId` is a **content-derived function of the regime fields only**; counters do not change it.
- Examples of changes that must change the id (013-AC-4): toggling an injection, losing real-time scheduling, a timeline epoch change, a path-MTU fallback. A change of counters only must not (013-AC-4).
- Falsifier of the configuration sub-question: two identical configurations must **not** yield different stamp ids. So the id must be identical on every node and profile for identical regime content.
- A record involving two endpoints carries both stamp ids, and the collector resolves ids to stamps (013-14, 013-17), so ids must be comparable across nodes.
- The hello carries a stamp version number (C-18). How that number relates to the `StampId` is an open sub-question owned by epic 004 with epic 005; this ticket does not define the hello field.

## Work
1. Choose a deterministic, `no_std`, allocation-free 32-bit function over the canonical bytes of the regime sections (in canonical order) and document it as a `[decision]`, stating which bytes are excluded: counter sections (Drops, per-connection summaries, per-peer observed DSCP, alignment measurements), unknown sections, and the id field itself if stored.
2. Implement `Stamp::id()` and make `StampBuilder` able to report the id of what it built.
3. Define the section-type registry flag "regime" or "counter" used by this function, so new sections declare their kind.
4. State the collision behaviour of a 32-bit id (probability for a given number of stamps per run) and what the collector does on a collision (013-17).

## Acceptance criteria
1. Two stamps built from identical inputs on different builder instances have equal ids (test).
2. Flipping any single byte inside any regime section changes the id (exhaustive test over a sample stamp: every byte, every one of 8 bit positions).
3. Changing any counter (a drop counter, any of the twelve summary fields of C-47 and C-159, a lateness percentile, the reclaimable latency) leaves the id unchanged (test per field).
4. The four named changes (injection toggled, `rt_scheduling` Present → Absent, timeline epoch changed, path MTU lowered on one connection) each produce a different id (test, once the relevant section tickets exist; until then on raw section bytes).
5. Implementation is `no_std` and allocation-free.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" (no new parser; `id()` is called by the 013-03 target); "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (no field added); "Design documents updated where behaviour differs, constants table first" (follows the epic's settled rule); "The stamp format is versioned independently of the media format" (no framing change).

## Out of scope
- Starting a new record set on id change (013-16).
- The hello's stamp version number (epics 004, 005).
