# 021-08 — Local-network permission prompt for a CLI binary
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-07, 021-31
## Context
Open sub-question owned by epic 021, feeding epic 028 (commissioning tool):

| Question | Owner | Falsifier |
|---|---|---|
| Does the permission prompt appear for a command-line binary, how is it pre-granted in commissioning, and does a self-addressed multicast probe detect the block? | This task, feeding task 028 | No supported way to pre-grant; or a blocked edge passes the probe |

Run on the reference Mac(s) proposed in 021-31 and approved by the product owner (C-172), on the supported macOS versions, which are set after task 001 reports and include the current and the oldest obtainable release (C-138); record each OS build.

Background [memory]: recent macOS releases gate local-network access behind a per-app permission prompt; until granted, traffic is silently blocked.
## Work
- On each supported macOS version (C-138) on the reference Mac(s), run the edge binary (a) from Terminal, (b) as a launchd agent/daemon, (c) inside an app bundle, and record whether the prompt appears, whether traffic is blocked, and how it is granted.
- Find a supported pre-grant route usable at commissioning (e.g. configuration profile / MDM payload, app bundle with usage description); record it or record that none exists.
- Any throwaway probe binary written for this lives in `probes/` (C-133).
- Write the result as a short note with `[measured]` tags in `doc/design/reports/` (C-130), delivered as input to epic 028's per-OS checklist (operator checklists live in `doc/operations/`, C-171), and as a row update to the epic's open sub-questions table (closing it or naming a new owner and falsifier).
## Acceptance criteria
1. A result table covers the three launch modes on at least one reference macOS build, each cell `[measured]`.
2. The note states either a supported pre-grant procedure, tested once end-to-end on a clean user account, or "no supported way to pre-grant" (falsifier hit) with the consequence for commissioning.
3. 021-07's detection is confirmed to fire in every launch mode where traffic is blocked.
## Definition of done
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
## Out of scope
Implementing the commissioning tool or its checklist (epic 028).
