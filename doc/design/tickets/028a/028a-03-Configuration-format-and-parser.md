# 028a-03 — Configuration format and parser
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** S · **Depends on:** 028a-01

## Context
The commissioning configuration is a declarative TOML file. It is the operator's policy: the runtime owns *when and whether*, module code owns *what*, the operator owns *how much* latency and capacity to spend `[decision]`. The shape given by the epic (names may be refined):

```toml
domain = 17
[[node]]
name = "desk"; os = "linux"; port_mbit = 1000; switch = "sw1"; strict = true   # strict mode, hosted only (C-147, C-165)
[[node]]
name = "stage-left"; os = "macos"; port_mbit = 1000; switch = "sw2"
[[module]]
name = "desk"; node = "desk"; outputs = { out = 24 }   # an output with controllers adds controller_bytes and header_bytes (C-219)
[[module]]
name = "stage-left"; node = "stage-left"; sink = true; e_us = 100   # omit e_us: E unknown (C-103); buffer_kb = ... declares buffer memory (C-211)
[[link]]            # trunks only; endpoint links are discovered
name = "sw1-sw2"; a = "sw1"; b = "sw2"; rate_mbit = 10000   # per direction (C-117)
[[group]]
name = "main"; a_ms = 6.0; required_e_us = 100   # required_e_us optional (C-103)
members = ["stage-left", "stage-right"]
[[group]]
name = "default"; a_ms = 20.0   # no members: every sink not in another group
[[connection]]
from = "desk.out"; to = "stage-left.in"; path = ["sw1-sw2"]   # align = false opts this input out of alignment (C-165)
# a module may carry pacing = { ... } (override, C-155) and, for a fan-in feeding no group, budget_ms (C-142)
```

Rules that bear on the format:
- Domain id is 16 bits; 0 means unconfigured (C-11).
- Audition offset A is set per presentation group and has no default value (C-55). The default group (the whole domain) is a `[[group]]` named `default` with no `members` list; it holds every sink not listed in another group; its `a_ms` is required like any other (epic 028a Background).
- A group's `required_e_us` is optional. A sink with no certified E declares E as unknown; it is refused from any group that sets a required E and admitted to groups that set none (C-103). So `e_us` is optional on a sink and its absence means "unknown", not 0.
- `[[link]]` declares trunk links only (switch-to-switch); endpoint links are not declared. A declared trunk capacity is per direction (C-117); `a` and `b` name the two switches so the direction of each connection along `path` can be derived from the switches its endpoints attach to.
- Operator policy in the configuration (C-165): strict mode (per hosted node; strict mode is hosted-only, C-147), per-input alignment opt-out (per connection) and pacing override (per module). A fan-in that feeds no presentation group needs an explicit skew budget in the configuration (C-142). Absent fields mean: strict mode off, input aligned, no pacing override, no budget.
- The tool checks declared buffer memory before applying a change of A and refuses one that does not fit (C-211). A module that aligns inputs, a sink included (C-222), may therefore declare its buffer memory (`buffer_kb`); absent means "not declared", not 0.
- The wire cost of a controller stream is one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s (C-219). An output with a controller stream (a bundle source's controller member, C-205, C-209, or a single-stream source's second stream, C-181) therefore declares `controller_bytes` (controller bytes per slice) and `header_bytes` (at most 100, C-137), copied from the module's manifest; absent `controller_bytes` means the output has no controller stream.
- Nodes carry no holder or follower role: the holder is elected at run time (C-113), every other module follows it, and with at most 64 modules a holder has at most 63 real followers (C-114).

## Work
- Module `hadronyche_tools::config`: serde types `Config`, `Node`, `Module`, `Link`, `Group`, `Connection` for the shape above. Node: name, OS (`linux` / `macos` / `windows` / `embedded`), endpoint port rate in Mbit/s, switch. Module: name, node, output channel counts, sink flag, optional declared E in µs, optional pacing override, optional fan-in budget in ms, optional declared buffer memory in KB (C-211, C-222); per output, optional `controller_bytes` and `header_bytes` (C-219). Node: optional `strict`. Connection: optional `align` (default true).
- `parse(&str) -> Result<Config, ConfigError>` with errors carrying line/column.
- Writer: `Config::to_toml()` so the tool can write configuration (e.g. a proposed A from 028b-11).
- Fuzz target `fuzz_config_parse` (cargo-fuzz) over arbitrary bytes; runs per the fuzz budget (C-95).

## Acceptance criteria
1. The sample above parses into two nodes (`desk` with strict mode on), two modules, one link (`sw1-sw2`, `sw1`–`sw2`, 10 000 Mbit/s per direction), two groups (`main`: A = 6.0 ms, required E = 100 µs, 2 members; `default`: A = 20.0 ms, no members, no required E) and one connection (path `["sw1-sw2"]`).
2. A missing `a_ms` on a group parses but is represented as "absent" (not 0, not a default) so that 028a-08 can refuse it. A missing `e_us` on a sink is represented as "unknown"; a missing `required_e_us` as "none"; a missing `budget_ms` as "none" (so that 028a-09 can refuse an unbudgeted fan-in); a missing `buffer_kb` as "not declared" (so that 028a-10 can report it unchecked); a missing `controller_bytes` as "no controller stream". A `header_bytes` above 100 is a parse error (C-137, C-219).
3. Round trip: `parse(to_toml(c)) == c` for every test fixture.
4. Malformed input yields an error with a line number, never a panic; the fuzz target runs 60 s on every change and 1 hour nightly without a crash, with its corpus kept (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Semantic checks (028a-04 to 028a-08, 028a-09).
