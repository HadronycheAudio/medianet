# 002-03 — Reading notes: transport.rs and control.rs
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-01
## Context
Part of epic 002 (POC archaeology, check 0b). Findings go to section **(d) Reading notes** of `doc/design/reports/archaeology/REPORT-0b.md`.

Two files of the frozen POC (`/home/dorian/workspace/hadropoc`) were never fully read:
- `transport.rs` — 947 lines; only its two tests were read;
- `control.rs` — not read.

The POC streams 96 kHz audio, controllers, musical-event parts and transport in datagrams between modules (sender, synth, mixer) that find each other by multicast gossip and elect a clock holder. In a 25-second loopback run on kernel 6.12 the transport check reported "0 of 0 windows" for the whole run and the display said "playhead: no transport state yet" `[measured]`. These notes are an input to diagnosing that (002-05).

Facts to look for while reading (they are needed by 002-05):
- who acts as transport reference (the `--transport` help text in `main.rs` says the sender defaults to reference);
- how `ctx.is_reference()` is decided (used at `sender.rs:229`);
- any listen-before-claim delay before a module claims the reference role (the design inherits 1.5 s from the POC, C-32; confirm the value and what it guards in the POC);
- how transport datagrams are sent and on which output connection (`events`).

Control-plane timers the design inherits from the POC (constants table `02-constants.md`), to be confirmed with pointers if they appear in `control.rs`: hello period 1 s (C-30), hello expiry 3 s for non-live peers (C-31), listen-before-claim 1.5 s (C-32), election window 400 ms (C-33).
## Work
1. Read both files in full in the frozen tree, read-only.
2. Write in section (d) one sub-section per file, about half a page each: what it does, main types and functions with `file:line` pointers, inputs and outputs (datagram kinds, connections), timers and constants, and anything that can make it silently inactive.
3. List the four "facts to look for" with what the code says, tagged `[code]`, so 002-05 can use them directly.
## Acceptance criteria
1. Two sub-sections exist (`transport.rs`, `control.rs`), each about half a page, each with at least 3 `file:line` pointers.
2. Each of the 4 facts to look for is answered or explicitly marked "not in these files", with a pointer when answered.
3. Every constant quoted (delays, rates, sizes) has its value and unit.
4. Each of C-30 to C-33 found in these files is marked "matches" or "differs" against the constants table, with a pointer; one that differs is flagged for 002-13.
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
- Running the POC or tracing the inert transport family to a verdict (002-05).
- Reading `audio.rs` (002-02) and `sender.rs`, `stats.rs`, `tui.rs` (002-04).
