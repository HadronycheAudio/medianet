# 002-01 — Scratch copy and report skeleton
**Epic:** 002-POC-archaeology · **Effort:** XS · **Depends on:** none
## Context
Epic 002 is pre-implementation check **0b**: diagnose the known instrument anomalies of the proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`, Rust, about 8 700 lines, one binary) down to "instrument bug, mechanism named", read the POC source files never fully read, and confirm by run two behavioural claims only read from code. Until 0b is done the POC must not be cited as the exact-tier baseline and no conformance vector may be extracted from it.

The POC is **frozen**: it is the reference instrument (its sender, synth and mixer become the reference sender, echo and mixer; its analyzer becomes the tolerance tier of the test suite). Only task 003 may modify it. All diagnostic logging for this epic (items (b) and (e)) must live in a **scratch copy** of the POC, never in the frozen tree.

All findings of the epic go into one document, `doc/design/reports/archaeology/REPORT-0b.md`, with one section per item (a) to (e). Each section ends with a verdict line, exactly one of:
- **instrument bug, mechanism: …**
- **not a bug, explanation: …**

tagged `[code]` (read in the POC source) or `[measured]` (observed in a run).

Settled locations (epic 002, Background):
- **Scratch copy:** any path outside `/home/dorian/workspace/hadropoc`; its path and fingerprints are recorded in the report.
- **Report:** the report and its companion files (logs, patches, captured outputs) live in `doc/design/reports/archaeology/` of the design set's repository (C-130), never in the frozen tree. The 002-09 checker's code lives in the `hadronyche-tools` crate and any throwaway probe in `probes/` (C-133).

`CONFIG_HZ=250` on the original measuring machine is POC reference value P-03 `[measured]` in the constants table `02-constants.md`.

This ticket sets up both the scratch copy and the report skeleton so the other tickets of the epic can work in parallel.
## Work
1. Record a content fingerprint of the frozen tree `/home/dorian/workspace/hadropoc` (e.g. a sorted `sha256sum` list of all tracked source files, excluding build output) and store it in the report as "frozen-tree fingerprint, taken <date>".
2. Copy the frozen tree to a scratch location outside `/home/dorian/workspace/hadropoc`. Record the scratch path and the fingerprint of the copy at creation in the report.
3. Build the scratch copy (release build) and confirm `hadropoc loopback --duration 5` runs to completion unprivileged. Record toolchain version, kernel version and `CONFIG_HZ` of the test host `[measured]`, and note whether `CONFIG_HZ` matches P-03.
4. Create `doc/design/reports/archaeology/REPORT-0b.md` (C-130) with:
   - a header stating the purpose (check 0b), the evidence tags in use (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`, as defined in `00-context.md`) and the verdict-line format above;
   - an "Environment" section (host, kernel, `CONFIG_HZ`, toolchain, frozen-tree fingerprint, scratch path);
   - empty sections `(a) Inert check families`, `(b) The 21.7 M counter`, `(c) Experiment metrics integrity`, `(d) Reading notes`, `(e) Run confirmations`, each with a placeholder verdict line `**verdict: pending**`;
   - an "Open sub-questions" section listing the four open sub-questions of the epic with their owner and falsifier (copied from the table in the Notes below).
## Acceptance criteria
1. The scratch copy builds and `hadropoc loopback --duration 5` exits with status 0 as an unprivileged user; the command and its exit status are pasted in the report.
2. The report contains the frozen-tree fingerprint and the scratch path, and the two fingerprints (frozen tree, scratch copy at creation) are identical.
3. The report contains the five section headings (a) to (e), each with exactly one verdict placeholder, and the 4-row open sub-questions table.
4. The report is not inside `/home/dorian/workspace/hadropoc` and sits at `doc/design/reports/archaeology/REPORT-0b.md` (C-130).
5. Re-computing the frozen-tree fingerprint after this ticket gives the same value (frozen tree untouched).
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] No new open item without an owner and a falsifier
- [ ] Scratch path and frozen-tree fingerprint recorded in `doc/design/reports/archaeology/REPORT-0b.md`

Notes — open sub-questions to copy into the report:

| Question | Owner | Falsifier |
|---|---|---|
| Is the voice comparison bit-level? | This epic (ticket 002-02) | `analyse` compares within tolerance only; the "exact tier for voices" description changes |
| Does the I/O-pressure finding survive? | This epic (ticket 002-09) | Experiment files are corrupt or multi-writer; the eligibility rule loses its basis |
| Is the 21.7 M value a decoding error or a display error? | This epic (ticket 002-08) | Neither; the value is a real counter and the loss accounting is wrong |
| Why does a 5 ms timeout behave as about 13 ms? | This epic (ticket 002-12; informational only) | None needed; the runtime paces by deadline |
## Out of scope
- Any diagnostic logging or code change in the scratch copy (002-07, 002-10, 002-11, 002-12).
- Filling any report section (002-02 to 002-12, 002-14).
- Fixing the corrupt metrics file in loopback (task 003).
