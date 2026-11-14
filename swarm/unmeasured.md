# Work resting on unmeasured figures

Later work that depends on a figure a hardware measurement has not yet
produced (kickoff section 7). Revisit each entry when the measurement
arrives.

| Unmeasured figure | Owning ticket / rung | Work that rests on it |
|---|---|---|
| The wire byte-order decision (004-01) cites AArch64 little-endian-by-default as `[memory]` (no freely fetchable standard pins the default; the Arm ARM needs an account) | 004-22 (named checker) | The byte-order justification in doc/design/spec/wire.md; nothing else — x86-64 LE is clause-verified (SysV ABI AMD64 PS §3.1.2) |
