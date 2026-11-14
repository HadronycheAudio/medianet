# 026a-12 — Hosted stamp collector
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** S · **Depends on:** 026a-11, epic 013

## Context

Devices cannot write files; they publish their regime stamp over the network (026a-11). Deliverable: a hosted **stamp collector** addition in `hadronyche-std` that records device stamps `[decision]`.

Settled transport (C-18): the stamp is a dedicated control message split into sections that each fit C-17 (1 252 bytes); it is multicast to the control group on any change and refreshed every 10 s; a unicast request gives an immediate fetch; the hello carries only a stamp version number. IPv4 only, multicast with TTL 1 (C-100).

Epic criterion 026-AC-1 (AC-35, part): the device's stamp is published over the network and **recorded by the hosted collector**.

Linux hosts are the backbone (clock holder, elections, conformance, commissioning), so the collector runs on a Linux host. The library component lives in `hadronyche-std`; a command-line front end, if one is built, is a tool binary in the `hadronyche-tools` crate (C-133).

The record format for device stamps is a presentation detail of the tool this ticket builds, so it is decided here and recorded in the collector's documentation (C-206); only what another epic or an operator's script depends on goes into the specifications. Falsifier: device and hosted stamps cannot be compared from the records, so the format follows epic 013's hosted-stamp record format wherever that exists and documents every addition. The earlier reading that epic 013 owns the device record format is withdrawn.

## Work

1. In `hadronyche-std`, a collector component that joins the control group, reassembles stamp sections by (module id, instance nonce, stamp version), decodes them with the epic-013 decoder, and records them.
2. Watch hello stamp version numbers; when a peer's version changes and no complete stamp of that version has arrived, send a unicast request.
3. Record format, decided here (C-206): epic 013's hosted-stamp format where it exists, extended for the device fields, one record per (module id, instance nonce), with the receive time and source address. New nonce = new record (device rebooted). Records are kept in the deployment's data directory, where certificates are stored beside the collected stamps (C-171); file names within it are chosen here (C-206).
4. Malformed sections, oversized packets (> 1 252 bytes) and incomplete versions are counted and dropped.

## Acceptance criteria

1. Integration test (host, unprivileged): a simulated device (`hadronyche-embedded` on mock or loopback transport) publishes; the collector writes a record containing every stamp field, including the embedded profile fields, D, D method and E.
2. A device reboot (new nonce) creates a second record; the same nonce with a new version updates the existing one.
3. With the periodic multicast suppressed, a version change seen in a hello leads the collector to request and record the new stamp.
4. Fuzzed section decoder and reassembler at the C-95 budget: no panic on arbitrary input up to 1 252 bytes.
5. Collector documentation states the record format and file names and lists the recorded fields (C-206); a device record and a hosted record of the same fields compare field by field.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope

- The device side of publication (026a-11).
- Hardware end-to-end verification (part of 026b-06 / 026b-08 runs).
