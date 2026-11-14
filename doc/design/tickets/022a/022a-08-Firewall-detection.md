# 022a-08 — Firewall blocking detection at startup
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-06

## Context
- Part of epic 022a (Windows edge backend), `firewall.rs`: detects a blocking firewall state and reports a named startup failure.
- Porting trap `[memory]`: inbound UDP is blocked until a first-run prompt is accepted; if the prompt is dismissed, traffic is silently dropped. Detect and report.
- 022-AC-5: with inbound UDP blocked, startup fails with a **named reason within 5 s** (not a silent hang).
- Open sub-question owned by this ticket (a measurement): does the firewall query alone detect a dismissed first-run prompt, or is the active probe needed? Falsifier: the query reports "allowed" while the probe datagram is dropped → the probe becomes mandatory.
- The medianet is IPv4 only (C-100); the probe uses the same IPv4 sockets as the edge (022a-03).
- Commissioning baseline (checklist by task 028): dedicated medianet NIC, discovery chatter suppressed, firewall scoped to medianet ports.

## Work
1. `firewall.rs`: `check_inbound_udp(iface_index, ports) -> Result<(), FirewallBlock>`:
   - query the firewall state / rules for the process and the medianet interface's network profile (API-based check, unprivileged);
   - as a fallback or confirmation, an active probe: send a datagram from a second local socket on the medianet interface to the media/discovery port and wait for receipt, with a timeout so the whole check completes within 5 s.
2. Named error `StartupError::FirewallBlocksInboundUdp { profile, ports }` with an operator-readable message naming the fix.
3. Call the check during edge startup before admission; fail startup on block.
4. Tests: unit test of the decision logic with mocked API results; manual test on reference hardware with an explicit block rule and with a dismissed first-run prompt.
5. Answer the sub-question: record, `[measured]`, whether the query alone detected each blocked case; keep the probe if it did not.

## Acceptance criteria
1. **(022-AC-5.)** With inbound UDP blocked (explicit block rule on reference hardware), startup fails with `FirewallBlocksInboundUdp` within 5 s, measured from process start to error, over 5 runs.
2. With the firewall allowing the medianet ports, the check passes in under 1 s.
3. The check needs no administrator rights.
4. If the firewall state cannot be determined, startup reports a named warning (not a hang) and proceeds to the active probe.
5. The query-versus-probe sub-question is answered `[measured]` for 022b-07.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The firewall fact is confirmed `[measured]` or corrected, for 022b-08

## Out of scope
- Scoping the firewall to medianet ports (commissioning checklist, epic 028).
