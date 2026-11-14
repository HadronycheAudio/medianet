# 001-05 — macOS drop counter check
**Epic:** 001-Receive-stamp-probes · **Effort:** XS · **Depends on:** 001-01, 001-02

## Context
On Linux, `SO_RXQ_OVFL` (since Linux 2.6.33) provides a cumulative per-socket drop counter in a control message `[kernel-doc]`. Claim to verify: macOS has no per-socket drop counter equivalent to `SO_RXQ_OVFL` `[memory]`. The result sets the expected `drop_accounting` capability state for macOS, one of present, absent, unverified or not requested (C-129), in 001-12.

The forced-drop run uses the Linux peer (001-01) at 2 000 packets/s. It puts no number in the constants table, so it needs no strict mode (C-147). It is made on the current macOS release; the oldest obtainable release is 001-16 (C-138). Outputs go under `doc/design/reports/` (C-130).

## Work
- Add a `drops` mode to `probes/macos-rxstamp/` that, without privilege:
  - tries any per-socket drop-reporting option found in the SDK headers and logs the result;
  - reads per-interface and protocol-wide counters available without root (for example UDP "dropped due to full socket buffers" from the `net.inet.udp.stats` sysctl, interface input-drop counters from `getifaddrs` / `if_data`), and logs which are readable.
- Force drops (small `SO_RCVBUF`, receiver paused while the Linux peer sends at 2 000 packets/s) and show whether each counter moves.

## Acceptance criteria
1. Output lists each candidate counter with scope (per-socket / per-interface / system-wide), readable unprivileged yes/no.
2. For each readable counter, a forced-drop run shows its delta against the number of missing sequence numbers.
3. A one-line verdict: per-socket drop counter present or absent.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated

Not applicable (C-125): "Tests green in CI, unprivileged" — the mode reads OS counters and has no pure logic to unit-test; "Every parser touched is fuzzed" — no parser added; "Regime stamp fields touched" — the capability value is set in 001-12; "Design documents updated" — 001-14; "No new open item" — opens none; "The frozen POC is not modified" — not touched; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Defining the capability bit (001-12).
- Windows (001-10).
- The run on the oldest obtainable macOS release (001-16).
