# 023-30 — Transport-state gap default (C-141)
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-12

## Context
- Read-only transport state is a v1 input content type: playing, bar, beat, tempo, time code at position p. Transport authoring is deferred past v1.
- Rule (C-141, decided): **on a gap, the last known transport state is held and the playhead advanced as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo; the output is marked degraded.** Rule (C-200, decided): **a stopped playhead does not advance through a gap.** This replaces the former requirement of an explicit gap override on every transport-state input; the former remaining gap (design owner) is closed.
- The degraded mark needs no extra mechanism: an output depending on a gapped transport input is degraded by the C-122 rule, using the dependency sets of 023-14 and L of 023-15.
- 023-12 gives every input a `GapPolicy`; this ticket fills the transport-state default and keeps it overridable by the DSL override construct like the other defaults.
- Rule (C-184, decided): **before any transport state has arrived, the input reads stopped, playhead at zero, default tempo.** This is the state held when a gap precedes any received state. Rule (C-199, decided): **the default tempo is 120 beats per minute.**
- Positions are on the domain-wide timeline at the domain rate, 96 kHz (C-01), 48 frames per slice (C-02).

## Work
1. Define `GapPolicy::TransportHold` and make it the default for every transport-state input.
2. Specify, in the IR semantics and the module docs, which fields are held (playing, tempo) and which advance with the playhead (time code, and bar and beat derived from it at the held tempo), for the positions elapsed since the last received state, when the held state is playing; when it is stopped, nothing advances (C-200).
3. Build the policy from kit nodes only (state cells and fixed-point or strict f32 arithmetic), so it adds no non-deterministic node (C-123) and its state and cost are counted by 023-17 and 023-16.
4. Initialise the `TransportHold` state to stopped, playhead (time code) at zero and the default tempo, with bar and beat derived from them (C-184); the default tempo is one named constant defined in one place, 120 beats per minute (C-199).
5. Supply the behaviour and one example to `LANGUAGE.md` (023-26).

## Acceptance criteria
1. A transport-state input with no override gets `GapPolicy::TransportHold`; no diagnostic is raised.
2. IR-level test: after a gap of 96 positions (2 slices) with playing = true, the held state keeps playing and tempo, and the time code advances by exactly 96 positions; bar and beat are those derived from the advanced time code at the held tempo.
3. With the override construct on the transport input, the override replaces `TransportHold` and the input appears in the manifest gap-override list.
4. The dependency set of an output fed by the transport input contains that input (so C-122 marks it degraded during the gap).
5. IR-level test: from start-up, before any transport state has arrived, the input reads playing = false, time code 0 and tempo 120 beats per minute (C-199) at every position, including through a gap of 96 positions; the first received state replaces it (C-184).
6. IR-level test: after a gap of 96 positions with playing = false, the held state keeps playing = false and its tempo, and the time code, bar and beat are unchanged (C-200).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] `LANGUAGE.md` covers every construct with one example (transport-gap example supplied to 023-26)

## Out of scope
- Generating the gap code (epic 024); the transport source and its authoring (deferred past v1); degraded marking itself (023-14, epic 024).
