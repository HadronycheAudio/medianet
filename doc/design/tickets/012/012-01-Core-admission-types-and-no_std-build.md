# 012-01 — Core admission types and no_std build
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** epic 004, epic 010, epic 011

## Context
Epic 012 implements the two-sided admission handshake and the per-connection state machine in `hadronyche-core`, as pure `no_std` state machines with no allocation. This ticket lays down the shared vocabulary every other 012 ticket uses, and the CI check that the crate builds for a `no_std` target with no allocator (epic criterion 012-A11).

Facts the types must carry `[decision]`:
- A connection is two-sided: the requester names a source output (or bundle members) and a consumer input; both sides accept or refuse.
- Refusal is explicit and terminal, with a named reason. "Target not present" is not a refusal: the request stays Pending, with no timeout (C-45).
- A bundle has at most 32 members, the controller member included (C-08, C-209). An empty member list means the whole output.
- The session number is assigned by the source, one per stream (C-19). The request carries the source's current session of each requested stream; `Accept` carries no session.
- Addresses are IPv4 only (C-100).
- Units declared: bit/s on the wire including headers, packets/s, slices/s, buffer memory in bytes for the listed members only (C-96), and an optional cost bound in abstract operations per slice for DSL modules (C-106).
- A sink declares its certified E or "unknown"; a presentation group may set a required E (C-103).
- Path-MTU fallback is 1 280 bytes (C-06), giving 1 252 bytes of UDP payload over IPv4 (C-100). A control message is at most 1 252 bytes (C-17).
- A Live connection persistently late with its offset at A becomes Demoted (C-109).
- There are 18 refusal reasons (C-185): the 16th, `BadMemberList`, covers an unknown or duplicate bundle member or more than 32 members (C-173); the 17th, `ControlDegraded`, refuses a request to a module whose housekeeping has stopped restarting (C-139); the 18th, `NoSkewBudget`, refuses an aligned input of a fan-in with no downstream group and no configured budget (C-142). `VersionMismatch`, `NameTaken` and `ModuleCap` stay in the enum but are raised at discovery, not at admission (C-185).
- Epic 009 delivers "cannot meet A" as evidence (C-174); it raises an alarm and changes no state.
- An accept carries the A of the presentation group that applies at the consumer, its own or inherited, so a fan-in outside any group can learn it (C-142).
- `ModuleId`, `PortRef`, `StreamId` come from earlier epics (004/010/011); `PeerMap` comes from epic 011.

## Work
In `hadronyche-core`, a new module (suggested `admission`), `#![no_std]`, no `alloc`:
- Define, with the shape given by the epic (names may be refined, but changes must be recorded in the module docs):
  ```rust
  pub type Addr = SocketAddrV4;   // C-100
  pub struct Request<'a> {
      pub source: ModuleId, pub output: PortRef, pub members: &'a [StreamId],   // empty = whole output
      pub sessions: &'a [u32],    // C-19, one per requested stream, same order
      pub consumer: ModuleId, pub input: PortRef,
      pub declared: Declared, pub intended_packet_bytes: u16,
  }
  pub struct Declared {
      pub wire_bits_per_s: u64, pub packets_per_s: u32, pub slices_per_s: u32,
      pub buffer_bytes: u32, pub cost_bound_ops: Option<u32>,
  }
  pub struct PresentationTerms { pub required_e_ns: Option<u32>, pub sink_e_ns: Option<u32> }
  pub struct Accept { pub pinned_source: Addr, pub pinned_dest: Addr, pub reverse_port: u16, pub path_mtu: u16,
                     pub group_a_ns: Option<u32> }   // C-142
  #[repr(u8)]
  pub enum RefuseReason {
      PortCeiling, PacketRate, SliceRate, InputCount, BufferMemory, CostBound,
      ContentMismatch, NotAnInput, VersionMismatch, ForeignDomain,
      PathMtuTooSmall, RequiredEExceeded, FollowerCap, ModuleCap, NameTaken,
      BadMemberList,   // C-173
      ControlDegraded, NoSkewBudget,   // C-185
  }
  pub enum DataPlaneEvidence { ClockUsable(bool), AligningDone { offset_ns: i64 }, MissedSlices(u32),
                               Resumed, SilentFor2s, EndOfStream, Disconnect, RatchetStep, AnnouncedRealign,
                               Demoted, CannotMeetA, LargePacketsVanished, ReportsSilentFor2s }   // CannotMeetA: C-174
  pub enum ConnState { Requested, Pending, Refused(RefuseReason), Admitted, Locking, Aligning,
                       Live, Interrupted, Demoted, Dead, Ended }
  pub enum ConnEvent { State(ConnId, ConnState), ProducerStopped(ConnId), Alarm(ConnId) }
  pub enum RequestError { TableFull, ControlDegraded }   // local, nothing sent (C-139 for the second)
  ```
- Define `ConnId` (small copyable index into the fixed-size connection table).
- Give `RefuseReason` explicit, stable discriminants (0..=17 in the order above) and a `TryFrom<u8>` that rejects unknown values, since the reason travels on the wire.
- Add the constants used by later tickets as `pub const`s with doc comments carrying their ids: `PATH_MTU_FALLBACK: u16 = 1280` (C-06), `MAX_BUNDLE_MEMBERS: usize = 32` (C-08), `CONTROL_MSG_MAX: usize = 1252` (C-17).
- Add a CI job that builds `hadronyche-core` for a `no_std` target without an allocator (for example `thumbv7em-none-eabihf`), unprivileged.

## Acceptance criteria
1. `cargo build -p hadronyche-core --target <no_std target>` succeeds with no `alloc` crate linked (012-A11); the job runs in CI.
2. `RefuseReason` round-trips through `u8` for all 18 variants (C-173, C-185); values 18..=255 are rejected by `TryFrom<u8>`.
3. All types listed above exist, are public, and `Request`, `Declared`, `PresentationTerms`, `Accept`, `ConnState`, `ConnEvent`, `RefuseReason` derive at least `Debug`, `Clone`, `Copy` (where lifetimes allow), `PartialEq`, `Eq`.
4. The three constants exist with the values 1280, 32 and 1252.
5. `Accept` has no session field; `Request` has one session per requested stream.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: `TryFrom<u8>` for one byte is covered by the codec fuzz target (012-21); no message parser here
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field is defined here
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: the types follow the epic shape
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions here
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22
- [ ] No-allocator `no_std` build job added to CI

## Out of scope
- Any behaviour: capacity checks (012-02 to 012-07), message encoding (012-08), the state machine (012-11), the `Admission` engine (012-12 onward), the stamp record (012-23).
- Defining `ModuleId`, `PortRef`, `StreamId`, `PeerMap` (epics 004, 010, 011).
