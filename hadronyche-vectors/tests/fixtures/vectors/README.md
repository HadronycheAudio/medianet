# Vector fixtures for ticket 007a-02

The vector file schemas are typed Rust definitions in `tests/support/mod.rs`;
`tests/vector_files.rs` validates these files against them. This directory
holds two disjoint sets.

## `examples/` — the coder's green work

One small, hand-written example per kind (ticket Work 7), at the paths
`tests/vector_files.rs` fixes:

| path | kind |
|---|---|
| `examples/wire/valid/example.json` + `example.bin` | wire valid |
| `examples/wire/invalid/example.json` + `example.bin` | wire invalid |
| `examples/control/example.json` | control scenario |
| `examples/timeline/example.json` | timeline |
| `examples/transport/example.json` | transport |
| `examples/reference/example.json` | reference |

They are samples, not generated vectors (007a-05 onward generates the real
ones under `hadronyche-vectors/vectors/`). The control example must carry at
least one C-159 counter and the health state `Demoted`; the invalid example
must carry `expected.time_bound` with the C-164 values
(`per_packet_us: 10`, `linear_in_length: true`).

## `negative/` — the tester's red work

Each subdirectory is a validation root that must be rejected for exactly the
reason it names:

| path | must fail because |
|---|---|
| `negative/no-spec-version/` | a control file has no `spec_version` |
| `negative/duplicate-id/` | two control files share one `id` |
| `negative/bad-reason/` | a wire-invalid file names `reason: "TooBig"`, outside the closed `Reject` enumeration of ticket 004-03 |

Do not "fix" these: `tests/vector_files.rs` asserts that each is genuinely
invalid and that the validator reports the named reason.
