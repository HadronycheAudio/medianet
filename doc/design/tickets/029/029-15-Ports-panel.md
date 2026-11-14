# 029-15 — Ports panel
**Epic:** 029-Monitor · **Effort:** XS · **Depends on:** 029-08, 029-10

## Context
- `hadronyche-monitor` (epic 029) panel **Ports**: per port and per direction (declared capacity is per direction, C-117), admitted rate against the **70 % ceiling** (C-23). A default stream is about **21.1 Mbit/s** (C-21).
- The monitor only displays what the runtime publishes in its stamp; it does not measure or compute admitted rate itself. `[decision]`
- Open question in the epic, owned by epic 013: where admitted rate and link rate per port and direction are published. Working assumption: in the stamp; `NotPublished` otherwise.

## Work
1. `views/ports.rs` implementing `Panel`: one row per (module, port, direction) with interface, link rate if published, admitted rate, ceiling (70 % of link rate, C-23), percentage of ceiling used, and an approximate "default streams" equivalent (admitted ÷ 21.1 Mbit/s, C-21).
2. A bar per row; at or above 100 % of the ceiling, error style.
3. Text capture for documentation.

## Acceptance criteria
1. A 1 Gbit/s port with 424 Mbit/s admitted in one direction shows ceiling 700 Mbit/s, 60.6 % of ceiling, about 20 default streams for that direction.
2. The two directions of one port are separate rows with their own figures.
3. An admitted rate equal to the ceiling renders in error style.
4. A port whose admitted rate is not published shows a `NotPublished` marker, not 0.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Admission control itself (other epics).
