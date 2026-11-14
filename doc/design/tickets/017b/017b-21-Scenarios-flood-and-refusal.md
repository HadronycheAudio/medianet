# 017b-21 — Scenarios `flood` and `refusal`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** M · **Depends on:** 017a-08, 017a-09, 017b-09, 017b-07, epic 012

## Context
- `flood` and `refusal` are in the epic's scenario list; their expected behaviour is now stated by the constants table (epic AC-017.14 and AC-017.15).
- **Flood (settled, C-16):** the drain per input per cycle is bounded to 2 × the admitted packets per slice for that input. The cycle is one slice period (C-156). An input offered more is drained at most that much per slice period; other inputs are unaffected.
- **Refusal (settled):**
  - Admission ceiling: 70 % of the link rate (C-23), against a declared trunk capacity per direction (C-117). A default stream is about 21.1 Mbit/s (C-21): for example a declared 100 Mbit/s per direction admits three streams (about 63 Mbit/s) and refuses a fourth (about 84 Mbit/s).
  - Sink with no certified E (C-103): declares E unknown; refused with reason RequiredEExceeded from a presentation group that sets a required E; admitted to one that sets none. A reference module joining as a sink has no certified E: only the commissioned state gives a numeric E (C-140).
  - Pending (C-45): a connection whose target is not present persists as Pending, shown unsatisfied, with no timeout; it is not a refusal. Only Live connections are shown satisfied (C-154).
  - Fan-in with no downstream presentation group and no explicit budget (C-142): its aligned inputs are refused with NoSkewBudget (C-185).
  - BadMemberList (C-173): the refusal reason for an unknown or duplicate bundle member or more than 32 members (C-08), the controller member counted among the 32 (C-209).
  - Refusal reasons number 18 (C-185); VersionMismatch, NameTaken and ModuleCap are raised at discovery, not at admission, so `refusal` does not expect them.
  - A connection alarm is raised on every refusal (C-173).
- Reason names and connection states are epic 012's.
- L0 rules `[decision]`: assert content, loss accounting, lock, classification and ground-truth error; never latency on loopback; no assertion on sync-rate timing; a family with zero samples fails the run.

## Work
1. Scenario `flood`: once an input is admitted, send well-formed packets for that stream above its admitted packets per slice, through `send_raw` (017b-07) or a sender test mode; read the drain count per slice period (C-156).
2. Scenario `refusal`, five cases: (a) declare a per-direction capacity and request one stream more than 70 % allows; (b) a reference module joins as a sink with E unknown, first to a group with a required E, then to a group with none; (c) request a connection to a module that is not present; (d) start a reference mixer with no downstream presentation group and no explicit budget (C-142); (e) request a bundle connection naming a duplicate member, and one naming an unknown member (C-173).
3. Write the expected behaviour into the scenario source as rustdoc citing the C-ids and epic 012.

## Acceptance criteria
1. `hadronyche-l0 refusal --duration 20` and `hadronyche-l0 flood --duration 20` exit 0.
2. `flood`: the drain per slice period (C-156) for the flooded input never exceeds 2 × its admitted packets per slice (C-16); other inputs have 0 exact mismatches outside excluded windows; required families non-zero.
3. `refusal` (a): the extra connection is refused with a named reason and never goes live; earlier connections stay live with 0 mismatches (C-23, C-117); a connection alarm is raised (C-173).
4. `refusal` (b): refused with RequiredEExceeded by the group with a required E; admitted by the group without one (C-103).
5. `refusal` (c): the connection stays Pending and unsatisfied for the whole run, with no refusal and no timeout (C-45, C-154).
6. `refusal` (d): the mixer's aligned inputs are refused with NoSkewBudget and never go live (C-142, C-185).
7. `refusal` (e): both requests are refused with BadMemberList and never go live (C-173).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Defining admission policy (epic 012).
