# 029-10 — Terminal UI shell
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-03, 029-04, 029-06

## Context
- `hadronyche-monitor` (epic 029) is a terminal monitor built with `ratatui` (the only extra dependency the epic allows, as in the POC).
- Seven panels follow in later tickets: Medianet, Clock, Connections, Bundles, Ports, Sinks, Regime.
- The POC's `src/tui.rs` (668 lines, ratatui) `[code]` may be reused for layout ideas (lock colour, playable-latency panel, packet counters); its aggregation (`stats.rs`) must not be reused. The frozen POC is not modified (C-130).
- Epic criterion 029-AC-6 requires identical rendering from a file and from the live network, so rendering must depend only on model state, never on wall-clock time of rendering or on source type.
- Stamps arrive on any regime change and every 10 s (C-18), so stamp-derived figures can be up to 10 s old; their age is part of the display.

## Work
1. `views/mod.rs`: `trait Panel { fn title(&self) -> &str; fn render(&self, model: &Model, area: Rect, buf: &mut Buffer); }` and an `App` holding the panel list and selected panel/module.
2. Main loop: pull observations from the chosen `ObservationSource` (live 029-03 or `--file` 029-04) on a separate thread into `Model` (029-06); redraw at a fixed rate (e.g. 10 Hz) without blocking ingestion.
3. Keys: Tab / number keys switch panels, arrows select a module, `q` quits.
4. "Now" for countdowns (029-07) and stamp ages is the time of the latest observation, not the terminal clock, so replay renders the same as live.
5. A global header line shows **INJECTED** when `run_is_injected()`, **COMMISSIONING** when `run_in_commissioning()` (029-08, C-107), and the source (live / file name).
6. `render_to_text(model, panel, width, height) -> String` using ratatui `TestBackend`, for tests and documentation captures.

## Acceptance criteria
1. With a `VecSource` (029-01) of fixed observations, `render_to_text` of an empty placeholder panel is byte-identical across 10 runs.
2. While the UI redraws at 10 Hz, ingestion of 10 000 queued observations completes without any dropped observation.
3. An injected stamp makes the header contain the word `INJECTED`; a stamp with an open commissioning window makes it contain `COMMISSIONING`.
4. `q` exits with status 0 and restores the terminal.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Panel contents (029-11 to 029-17, 029-24); headless mode (029-18).
