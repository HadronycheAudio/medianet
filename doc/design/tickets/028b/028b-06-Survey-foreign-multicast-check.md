# 028b-06 — `survey`: foreign multicast on the medianet interface
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-02

## Context
A medianet is a wired, isolated LAN whose only traffic is this runtime's. The per-OS checklists require OS discovery chatter to be suppressed on the medianet interface. The checklist acceptance (epic AC-9) is: following each OS checklist on a fresh machine yields a clean `survey`, meaning **no foreign multicast observed on the medianet interface for 10 minutes**. Stamps cannot carry this.

Settled in the epic `[decision]`: the survey host observes its own medianet interface passively, receive-only. The runtime uses **IPv4 only**, multicast with TTL 1 (C-100), so every IPv6 frame (router and neighbour discovery, MLD, mDNS over IPv6) is foreign, as is any IPv4 multicast or broadcast that is not the runtime's.

## Work
- `survey --listen-foreign <duration>` (default 10 min when used for checklist verification): passively observe multicast and broadcast frames on the medianet interface of the survey host and classify those that are not this runtime's IPv4 traffic: any IPv6; IPv4 to an unknown group or port; mDNS, SSDP, LLMNR, NetBIOS, WS-Discovery and similar.
- Finding `net.foreign_multicast`: one per source host and protocol, with count and first/last time.
- Receive-only capture; document the privilege needed (packet capture). Per-change CI uses recorded pcap fixtures, unprivileged; the live-capture test runs in the separate privileged nightly job (C-146).
- Frame classifier fuzzed per the fuzz budget (C-95).

## Acceptance criteria
1. Replaying a pcap fixture containing mDNS from one host yields one `net.foreign_multicast` finding naming the host and protocol.
2. A fixture containing an IPv6 neighbour solicitation yields a `net.foreign_multicast` finding (C-100).
3. A fixture containing only runtime traffic for 10 minutes yields no finding.
4. The capture never transmits (epic AC-10).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed

## Out of scope
- How to suppress chatter on each OS (028b-21 to 028b-23).
