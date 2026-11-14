# 028a-06 — `validate`: endpoint port sums and 10 GbE wall
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** M · **Depends on:** 028a-02, 028a-03, 028a-04, epic 012

## Context
For every endpoint switch port, `validate` sums the wire rate of every connection crossing it and refuses a port above the admission ceiling. Endpoint port admission also stays online and dynamic in the runtime (epic 012); the offline result must match the runtime's own refusal (runtime AC-12). Node port rates come from the configuration (`[[node]] port_mbit`, 028a-03).

Facts (from 028a-02 `capacity`):
- One sample rate on the wire, 96 kHz (C-01). Per stream (6 channels, 192 bytes per channel per slice, 1 152 bytes of audio, C-04; about 1 320 bytes on the wire, C-20): about 21.1 Mbit/s, 2 000 packets/s (C-21); per channel about 3.5 Mbit/s.
- Per source: 24 ch = 84 Mbit/s, 48 ch = 169 Mbit/s, 96 ch = 338 Mbit/s (C-22).
- Admission ceiling 70 % of link rate (C-23): 700 Mbit/s on GbE, 70 Mbit/s on a 100 Mbit port.
- GbE port wall about 198 channel-copies (C-24); any port carrying more runs 10 GbE (C-25). A 100 Mbit device port holds about 19 channels (C-26).
- No multicast fan-out in v1: a source with N consumers puts N copies on its own port.
- A controller stream costs one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s (C-219), the same figure runtime admission uses. A bundle source's controller member counts towards the 32 members of C-08, so a source with controllers has at most 31 audio members, 186 channels (C-209).

## Work
- Build the endpoint-port load model from the configuration: each connection adds one source copy to the producer's port (egress) and the consumer's port (ingress), its controller stream included at the C-219 cost when the output declares `controller_bytes` (028a-03).
- Check `bundle.controller_cap`: refuse an output with a controller stream that declares more than 186 audio channels (C-209), naming the output. The DSL front end rejects such a source at compile time (C-218); the tool refuses the configuration that declares one.
- Check `port.ceiling`: refuse any port whose load > 70 % of its declared rate, with message for GbE ports above the wall: `port would carry <X> Mbit/s; use 10 GbE` (X rounded to integer, thousands separated by a space as in "1 014").
- Check `port.wall` (flag): ports above about 198 channel-copies are flagged for 10 GbE even when declared as 10 GbE (informational).
- 100 Mbit ports: refuse above 70 Mbit/s (about 19 channels).
- Test every refusal and flag.

## Acceptance criteria
1. A 96-channel source fanned out to three consumers on GbE is refused with exactly `port would carry 1 014 Mbit/s; use 10 GbE` (3 × 337.9 Mbit/s, C-20, C-22; epic AC-2), naming the port.
2. The same configuration with the source port declared at 10 000 Mbit/s passes the ceiling check and is flagged as above 198 channel-copies.
3. Two copies of a 96-channel source on GbE (676 Mbit/s, C-92) pass; a 100 Mbit port carrying 20 channels is refused, 19 passes.
4. A test feeds the same scenarios, including a source with a controller stream, to the runtime admission unit (epic 012) where available, and both refuse/accept identically and quote the same Mbit/s figure (C-219).
5. A 96-channel source with a controller member of 100 header bytes and 400 controller bytes per slice adds 9.056 Mbit/s and 2 000 packets/s per copy to each port it crosses (C-219); an output with controllers declaring 192 audio channels is refused (C-209).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] `validate` runs with no network and has tests for every refusal

## Out of scope
- Trunk links (028a-07). Online endpoint admission (epic 012).

## Notes
- A source with controllers carries one more stream than its audio channels imply: a bundle's controllers ride in their own member stream (C-181, C-205, C-209), a single-stream source's go to a second stream once they exceed the header cap (C-181). Its wire cost is the C-219 figure from 028a-02; in the shared case of a single-stream source the controllers are inside the audio packet and add no stream.
