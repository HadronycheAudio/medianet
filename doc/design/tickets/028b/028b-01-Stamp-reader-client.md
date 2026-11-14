# 028b-01 — Read-only stamp reader client
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028a-01, epic 013, epic 015

## Context
All online subcommands (`survey`, `propose-a`, `certify-*`, `burst-test`, `soak`, `watch`) consume the runtime's **regime stamps**: a wire object every module publishes, with sections identity, capability states, network, clock, alignment, platform, drops, injection. Settled facts:
- **Transport (C-18).** A stamp is a dedicated control message split into sections that each fit one packet at the fallback MTU (at most 1 252 bytes of UDP payload, C-17). It is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number, so a version change in a hello signals a new stamp.
- **IPv4 only (C-100).** Multicast with TTL 1.
- **Counters (C-47, completed by C-159).** Producer and consumer each publish per-connection summary counters in their own stamp: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead, and the observed DSCP. Lateness percentiles are the 50th, 99th, 99.9th and the maximum (C-160). The holder reports the DSCP observed on each follower's sync requests, and every module the DSCP observed on each peer's hellos (C-152). The runtime raises connection alarms on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded (C-173). Receiver reports are unicast to the producer, not visible to this tool, and there is no query message for them. The tool uses stamps only.
- **Identity (C-11, C-46).** A module is identified by a 64-bit module id and a 64-bit instance nonce. Two nonces for one module id is a collision; third parties quarantine the id until only one nonce has been seen for 3 s.

Hard rule (epic AC-10, C-107): outside a commissioning window the tool only reads; it never takes part in a live data path. A unicast stamp fetch is a read and is allowed. The hosted runtime is epic 015.

## Work
- Module `hadronyche_tools::stamps`: join the IPv4 control group on the medianet interface as a passive stamp listener, reassemble stamp sections, and decode with the decoder from epic 013 (no own decoder unless needed; any new decoding is fuzzed).
- API: `StampReader::collect(duration) -> Vec<Stamp>` and a streaming `subscribe()`; index by (module id, instance nonce); keep the latest complete stamp per module plus history for a given window; expose collisions (one module id, several nonces) to callers.
- Optional `fetch(module)`: a unicast stamp request for an immediate read (C-18); `collect` uses it for modules whose hello has been seen but whose stamp has not.
- Interface selection: `--interface <name>` (medianet NIC) and `--domain <id>`.
- The client opens only the control-group receive socket and the unicast fetch socket; no stamp publishing, no media sockets.
- Expose the counters, percentiles, DSCP observations and connection alarms above as typed accessors; document which stamp fields the tool reads.
- Test with recorded stamp captures (fixtures) so CI needs no network.

## Acceptance criteria
1. Replaying a recorded capture of N modules yields exactly N latest stamps, with all eight sections decoded, including stamps split over several sections.
2. A capture with one module id under two nonces is reported as a collision, not merged.
3. A test (or socket audit) shows the client sends nothing except unicast stamp fetch requests and never opens a media data socket (epic AC-10).
4. Malformed stamp bytes never panic; any decoding added here is fuzzed per the fuzz budget (C-95).
5. Against a live hosted runtime (epic 015) on a lab LAN, `collect(11 s)` lists every running module passively (one 10 s refresh, C-18), and `collect(2 s)` with fetch enabled lists them too.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Interpreting stamps (028b-02 onward). Changing stamp content (epic 013), except documenting the fields this tool reads.
