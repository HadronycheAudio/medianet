# 021-19 — Edge only: holder bit never set
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-15, 021-16
## Context
Scope ruling [decision]: **edge only**. The macOS edge never holds the clock and never stands for election. The holder-eligible capability bit is never set; the build uses the hosted crates' edge-only feature (no holder candidacy).

- Only a holder-eligible module may take over as holder: one whose time is usable takes over and keeps the epoch (C-213); when no candidate has usable time, the holder-eligible candidate with the lowest module id wins and bumps the epoch (C-113). With no holder-eligible candidate at all, no holder can be chosen.
- The hello carries the capability bits and only a stamp version number; the regime stamp travels in its own control message (C-18).

Epic criterion 021-AC6: the hello never carries the holder-eligible bit; a medianet of only macOS edges reports "no holder-capable node".
## Work
- Test that every hello emitted by the macOS edge has the holder-eligible bit clear (unit test on the encoded hello, plus wire capture).
- Integration test (manual hardware job): two or more macOS edges on a medianet with no Linux host; confirm the reported state is "no holder-capable node".
## Acceptance criteria
1. Unit test: encoded hello from the macOS build has the holder-eligible bit = 0.
2. Wire capture over 60 s of a running edge: 0 hellos with the bit set.
3. Medianet of only macOS edges reports "no holder-capable node" within the discovery period, and no edge bumps the timeline epoch as an election winner (C-113).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
## Out of scope
Selecting the edge-only feature in the build (021-01). Election logic (core).
