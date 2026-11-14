# 012-07 — Presentation group required E check
**Epic:** 012-Admission-handshake · **Effort:** XS · **Depends on:** 012-01

## Context
Presentation groups `[decision]`: a sink belongs to a presentation group with an operator-configured audition offset A (C-55) and a **required E**. E is the 99.9th percentile of absolute emission error over a one-hour certification run (C-73). A sink whose E exceeds the group's required E is refused at admission with `RefuseReason::RequiredEExceeded` (012-A4, AC-30), and never reaches Live in the group.

Settled by C-103: a sink with no certified E declares E as unknown; it is refused with `RequiredEExceeded` from any presentation group that sets a required E, and admitted to groups that set none. Both values travel in the consumer's manifest as `PresentationTerms { required_e_ns: Option<u32>, sink_e_ns: Option<u32> }` (012-01), where `required_e_ns: None` means the group sets no required E and `sink_e_ns: None` means E unknown. Only a commissioned sink declares a numeric E; uncertified and self-monitored sinks declare E unknown (C-140). The check uses the sink's own declared E only: a sink is refused on its own figure whatever the E of the other sinks in the group (C-194, the AC-30 refusal test). A commissioned sink whose clock is unlocked stays commissioned and keeps its numeric E (C-180); its alignment claim is suspended until the clock locks again, which the connection reflects by waiting in Locking (012-11), and it resumes without re-commissioning, so the lock state is not an input of this check. Units are nanoseconds; the reference targets are 50 µs on Linux, 100 µs on macOS and 1 ms on Windows (C-70 to C-72, pending measurement).

## Work
- `pub fn check_required_e(terms: Option<PresentationTerms>) -> Result<(), RefuseReason>`:
  - no group (`None`) → Ok;
  - group with `required_e_ns: None` → Ok, whatever the sink's E (C-103);
  - group with a required E and `sink_e_ns > required_e_ns` → `RequiredEExceeded`;
  - group with a required E and `sink_e_ns: None` → `RequiredEExceeded` (C-103).
- `pub fn declared_sink_e(state: CertState, e_ns: u32) -> Option<u32>` (names may be refined), used when the sink fills its manifest: `Some(e_ns)` only for the commissioned state, `None` for uncertified and self-monitored (C-140). The certification state comes from the sink's certificate, which is outside this epic.

## Acceptance criteria
1. E = required E → accepted; E = required E + 1 ns → `RequiredEExceeded`.
2. Sink with unknown E in a group that sets a required E → `RequiredEExceeded` (C-103).
3. Sink with unknown E in a group that sets none → accepted (C-103).
4. Sink outside any group → accepted regardless of E.
5. `declared_sink_e` returns a number only for a commissioned sink; an uncertified or self-monitored sink with a low measured E is still refused from a group that sets a required E (C-140).
6. The result depends only on the sink's own E and the group's required E: no other sink's E is an input (C-194), and the clock lock state is not an input (C-180).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows C-103
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- The end-to-end "never reaches Live" test of AC-30 (012-19 scenario vectors).
- Measuring E (certification run) and audition offset A handling (other epics).
