# 005-06 — Discovery map states and inputs
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-02, epic 004

## Context
- Machine 1 of the control specification `doc/design/spec/control.md`: the discovery map, a pure table *(state, input) → (new state, outputs, events)* with no I/O, in the `no_std` core `hadronyche-core`.
- Rules of the discovery map:
  - fixed capacity of 63 peer entries (C-217): a medianet has at most 64 modules (C-34) and a module holds no entry for itself; a 65th module (a 64th peer) is not admitted to the map and a named event is raised (`RefuseReason::ModuleCap`);
  - a hello inserts or refreshes an entry; hellos are sent every C-30 = 1 s, multicast on IPv4 with TTL 1 (C-100), and fit 1 252 bytes of UDP payload (C-17);
  - a non-live peer expires C-31 = 3 s after its last hello; a peer with a live connection is **not** removed by expiry (L-1);
  - foreign-domain hellos (a different non-zero domain) are dropped and counted (`ControlEvent::ForeignDomainSeen`); domain is 16 bits, domain 0 = unconfigured (C-11); a hello with domain 0 is ignored and counted, not refused (C-173);
  - a version mismatch raises a named event (`ControlEvent::VersionMismatchSeen`); `VersionMismatch`, `NameTaken` and `ModuleCap` are raised here, at discovery, where they stop a module joining, not at admission (C-185), so a version-mismatched module never becomes a peer entry;
  - id collision: an id seen with a different instance nonce (id and nonce are 64 bits each, C-11); third parties quarantine the id (ignore both) until only one instance nonce has been seen for the id for one hello-expiry period, 3 s (C-46); the redrawn module has a new id and is a new entry;
  - a newcomer whose name is already present refuses to join (listen-before-claim C-32 = 1.5 s, `RefuseReason::NameTaken`);
  - liveness: the hello carries the 64-bit counter of the authentication reservation (C-12), sent with scheme 0 and a zero tag in v1; that counter is the data-plane liveness counter (C-115); a stalled data plane is visible within 2 hello periods (C-42);
  - regime stamp: the hello carries only the peer's stamp version number; the stamp is a separate control message, multicast on change and every 10 s, and a unicast request fetches it at once (C-18);
  - DSCP verification: every module reports in its stamp the DSCP observed on each peer's hellos (C-152).
- Open sub-question owned by this ticket (epic 005): when a live-pinned peer is removed after its last connection ends (whether a version-mismatched module is inserted is settled by C-185: it is stopped from joining). Falsifier: a scenario vector leaves a peer in the map forever or drops a live peer.
- Message names and fields come from the epic 004 wire specification.

## Work
1. In `control.md`, Machine 1 "States and data": define the module-level states (at least: Listening — the 1.5 s listen-before-claim — Joined, Refused-to-join, Redrawing after collision) and the per-entry states (at least: Fresh, Live-pinned (has a live connection), Quarantined, Expired/free). For each, list the data held (id 8 bytes, nonce 8 bytes, name, domain 2 bytes, version, IPv4 address and port (C-100), capability bits, last-hello time, liveness counter (64-bit, C-115) and its last two values, stamp version (C-18), observed hello DSCP (C-152), quarantine start time and nonce-seen set, foreign-domain counter, domain-0 counter (C-173)).
2. "Memory bound": give the per-entry size in bytes and the worst case for 63 entries (C-217) plus counters; no heap.
3. "Input alphabet": list every input with kind and source: hello message (from 004, with the DSCP observed on receipt, C-152), hello-expiry timer per entry, listen-before-claim timer, hello-send timer (C-30), quarantine timer (3 s, C-46), local events "connection to peer became live" / "connection to peer ended" from the connection machine, local "data-plane liveness tick" from the data thread.
4. Write the domain 0 rule as settled (C-173: ignored and counted, not refused) and the version-mismatch rule as settled (C-185: the module is stopped from joining and is not inserted). Decide the open sub-question (final removal of a live-pinned peer); record it in "Decisions on open sub-questions" with a `[decision]` tag and the falsifier.
5. Mirror states and inputs in `doc/design/spec/control.toml` under machine `discovery` (no transitions yet). Stamp-version inputs and outputs are added by 005-23.

## Acceptance criteria
1. Every state has its data fields listed with a type and a size; addresses are IPv4.
2. The worst-case memory with 63 peers is stated as one number of bytes, and the map holds at most 63 entries (C-217).
3. The input alphabet contains at least the 7 inputs of Work step 3, each with kind `message`, `timer` or `local`.
4. The liveness rule is written so that a C-115 counter unchanged across 2 consecutive hellos (≤ 2 s at C-30 = 1 s) marks the peer's data plane as stalled (C-42).
5. The quarantine rule is C-46 exactly: it ends after 3 s with a single nonce seen for the id.
6. The open sub-question of Context has a recorded decision with a falsifier.
7. `control.toml` declares the same states and inputs as the prose.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The transition table and work-per-input bound (005-07).
- Regime stamp fetch rows (005-23).
- Implementation (epic 011).
