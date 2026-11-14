# 015a-24 — Changed A carried upstream and fan-in re-alignment (C-198)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-11, 015a-13, epic 009, epic 014
## Context
- File: `hadronyche-std/src/data_thread.rs`. The skew budget of an aligned input is capped by the presentation group's audition offset A (C-96); a fan-in outside any presentation group uses the A of the downstream group it feeds, the largest if several, or its configured budget (C-142).
- C-198: the receiver report (C-40) carries the consumer's current A, its own or the one it inherited. A change propagates one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186).
- C-186: every fan-in that inherited the old A re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do. The re-alignment itself is core's (alignment ring, epic 009); this ticket detects the change and hands it to core.
- C-208: each fan-in that learns of the change picks its own re-alignment position, one lead (64 slices, C-43) ahead, and announces it in its own output, exactly as for a ratchet-triggered re-alignment; fan-ins at different depths switch at different, announced positions. Picking and announcing the position are core's (epics 009, 010); the data thread sends the announcement with its output.
- C-211: the commissioning tool checks declared buffer memory before applying a change of A (epic 028). If a larger A reaches a fan-in anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running. The ring ceiling is core's; this ticket keeps the data thread running and publishes the alarm.
- C-214: a sink whose A changes switches at its own position, one lead (C-43) ahead, as a fan-in does (C-208); it has nobody to announce it to. Picking the position is core's (epic 009); the data thread hands the change to core.
- C-215: a ratchet step is a re-alignment like any other: it takes effect at a position the fan-in announces one lead (C-43) ahead in its own output, never silently on the next round. The step and its position are core's (epic 009); the data thread sends the announcement with its output by the same path as for a changed A.
- C-216: when A is clamped (C-211), the cannot-meet-A alarm stays raised until a later change of A that fits, and the fan-in's receiver report carries the A it was given, not the clamped value, so upstream modules size themselves to what the sinks need. This alarm is held by that rule, not released by the rolling 60 s window of C-182.
- The socket buffer of each aligned input is declared rate × A (C-96, 015a-11); a re-alignment to a new A applies the same rule.
- Sending the A in each report is 015a-13. Liveness and receiver reports run on the data threads, never through housekeeping (L-1); the change therefore never crosses the housekeeping queue.
## Work
1. Producer side of the data thread: read the A carried in each received receiver report (core decoder, epic 014) and keep it per output destination.
2. For a fan-in using an inherited A (C-142): recompute the inherited A from the reports (the largest if several); when it differs from the A in use, hand the change to core so the fan-in's aligned inputs re-align at the position core picks one lead ahead and announces in the fan-in's own output (C-186, C-208), and request the inputs' socket buffers at declared rate × new A, counting a kernel clamp.
3. From the next report on, the fan-in's own receiver reports carry the new A (015a-13), so the change moves one hop further upstream per report. When the new A is clamped (C-211), the reports carry the A the fan-in was given, not the clamped value (C-216).
4. A module with its own A (in a presentation group) or a configured budget does not change its A on a received report. When a sink's own A changes (commissioning, epic 028), hand the change to core so the sink switches at its own position one lead ahead; the sink announces nothing (C-214).
5. If the new A exceeds what the fan-in's buffer memory holds (C-211): keep the streams running on the deepest ring core can hold, and publish the cannot-meet-A alarm (C-174) to the counter board (015a-18). Keep the alarm raised until a later change of A that fits, then clear it (C-216).
6. When core takes a ratchet step on a fan-in's aligned inputs, send the re-alignment position core announces with the fan-in's output, by the same path as step 2 (C-215).
## Acceptance criteria
1. In a loopback chain sink → fan-in → fan-in → source, changing the sink's A makes each fan-in re-align once, in order, and each fan-in's reports carry the new A from its next report on. Each fan-in announces its own re-alignment position in its output at least 64 slices ahead, and the two fan-ins switch at different announced positions (C-208).
2. A fan-in feeding two downstream groups inherits the larger A of the two.
3. After re-alignment, the fan-in's input sockets are requested at declared rate × new A.
4. A module with its own A ignores a different A in a received report.
5. No message related to a change of A received in a report crosses the housekeeping queue (code inspection and test).
6. With a new A larger than the fan-in's declared buffer memory, the fan-in keeps streaming without interruption, its ring ceiling is the deepest it can hold, and the cannot-meet-A alarm appears in its stamp (C-211). The alarm is still raised after more than 60 s with no further change; its receiver reports carry the A it was given, not the clamped value; a later change to an A that fits clears the alarm (C-216).
7. Changing a sink's A makes the sink switch at a position at least 64 slices after the change, with no announcement in any output (C-214).
8. A ratchet step on a fan-in takes effect at a position announced in the fan-in's output at least 64 slices ahead, never on the next round (C-215).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — report decoding is core's (epic 014); "Regime stamp fields touched" — the cannot-meet-A alarm is a field defined by core (epic 013), only published here; "Design documents updated" — follows the epic; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — measured in 015b-10.
## Out of scope
- The re-alignment procedure, choice of position, ratchet steps, ring and ratchet ceiling resizing, and the cannot-meet-A computation (core, epics 009, 010).
- How a commissioned change of a sink's own A reaches its module (commissioning, epic 028).
- The buffer-memory check before a change of A is applied (commissioning, epic 028, C-211).
- Commissioning a group's A (epic 028).
