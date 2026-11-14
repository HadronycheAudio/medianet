# 014-08 — Probe request and reply codec
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-01, epic 004
## Context
The v1 reverse-path message set is exactly two kinds `[decision]`: the receiver report and the one-way probe request / reply. Probes can be sent by either end, on demand, with DSCP EF (46), because probes measure the media path (C-13). They are a different message from the path-MTU probe pair of admission (C-116). Request and reply carry the four timestamps of an exchange so either end can compute a round trip on the media class:
- `t1`: request send time (initiator clock);
- `t2`: request receive time (responder clock);
- `t3`: reply send time (responder clock);
- `t4`: reply receive time (initiator clock).

Round trip = `(t4 - t1) - (t3 - t2)`. Tests use injected integer timestamps, so the expected results are exact (settled by the epic).

Signatures from the epic:
```rust
pub struct Probe { pub t1: i64, pub t2: i64, pub t3: i64, pub t4: i64 }
impl Probe { pub fn round_trip_ns(&self) -> i64; }
pub fn encode_probe_request(t1: i64, out: &mut [u8]) -> Result<usize, EncodeError>;
pub fn answer_probe(req: &[u8], t2: i64, t3: i64, out: &mut [u8]) -> Result<usize, Reject>;
pub fn close_probe(reply: &[u8], t4: i64) -> Result<Probe, Reject>;
```

## Work
1. Implement the three functions using epic 004's probe request and reply layouts. `answer_probe` parses the request, copies `t1`, and writes `t2`, `t3` into a reply. `close_probe` parses the reply and returns `Probe` with the caller's `t4`.
2. Implement `Probe::round_trip_ns`.
3. If epic 004's layout includes a probe identifier (to match replies to requests), carry it through; otherwise record that matching is by `t1` echo.
4. `answer_probe` rejects a reply given as a request and vice versa; `close_probe` rejects a request.
5. No allocation, no panics.

## Acceptance criteria
1. With injected path delays `d_fwd = 137 000` ns and `d_ret = 211 000` ns and responder hold `t3 - t2 = 50 000` ns, `round_trip_ns` equals `348 000` exactly (epic AC-9, round-trip part).
2. `answer_probe` and `close_probe` return `Err` (never panic) on empty, truncated and wrong-type input.
3. Undersized `out` gives `Err(EncodeError::BufferTooSmall)` (request) or `Err` (reply), without writing to `out`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (fuzz target delivered in 014-10)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- One-way figures and clock-lock gating (014-09).
- Who initiates probes and at what rate (014-11).
- Sending probes on the socket with EF marking (epic 015).
