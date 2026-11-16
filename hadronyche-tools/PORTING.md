# Porting a `checks/*.sh` script to a Rust document check

The ticket checks under `checks/` are bash wrappers around embedded Python,
one per ticket. They move into this crate one at a time: the document
assertions become `cargo test` tests on the library `hadronyche_tools::doccheck`,
the assertions that run tools or read repository state go to
`scripts/repo-state-checks.sh`, and the script is deleted in the same commit.
This file is the procedure. It is written so several porters can work in
parallel without editing the same files.

## Rulings that bind every port

1. **Equivalence first.** A port reproduces *its own script's* variant of
   every rule, even where scripts disagree (the same rule is written about
   fifteen ways across the scripts; see
   `swarm/notes/checks-inventory-2026-11-16.md`, section 4). The library
   offers each disputed rule as a function taking an explicit, named policy;
   the port names the policy of its script at the call site. Unifying the
   variants is a later, separate, reviewed step — never a side effect of a
   port.
2. **Not ported** (they stay scripts; the suite keeps running them):
   - 003-01, 003-03, 003-04, 003-06, 003-07, 003-09: they freeze-check,
     build or run the proof of concept;
   - 003-02, 003-05, 003-08: their content parts read files outside the
     repository (untracked notes, the proof of concept's `BASELINE.md`).
   The harness cannot sandbox what lies outside `doc/`, and a test must not
   read outside the repository.
3. **Tool gates run once.** An assertion that runs a tool (`cargo run --bin
   wire-check`, `cargo test -p hadronyche-tools --test wire_tables`,
   `doc/kickoff/check-design.py`, a script running sibling scripts, git
   against a ref, the proof of concept's state) would be circular or
   environment-dependent inside `cargo test`. It goes to
   `scripts/repo-state-checks.sh`, and each tool runs there once: a port
   adds a named assertion on the shared result, keeping any per-ticket pin
   ("at least 12 passed") as its own line.
4. **Header and code disagree → follow the code** (it is what has been
   passing), and record the disagreement in the test's doc comment.

## What a port consists of

For ticket `<id>` (`004-27`), with `<mod>` = `c` + the id with `-` as `_`
(`c004_27`):

| File | What |
|---|---|
| `src/doccheck/checks/<mod>/mod.rs` (or `<mod>.rs`) | the check: `pub fn run(docs: &Docs) -> Report` and `pub const SPEC: CheckSpec`; split into files per criterion as needed |
| `tests/doc_<id>.rs` (`doc_004_27.rs`) | one `#[test]` per acceptance criterion and DoD item, the denial fixtures, the red-phase test |
| `equivalence/<id>.md` | the harness report the port was accepted on |
| `scripts/repo-state-checks.sh`, section `# --- <id>` | the relocated repository-state assertions |
| `checks/<id>.sh` | deleted, in the port's commit |

There is no registry to edit: `build.rs` generates the list of ports from
the files named `src/doccheck/checks/c<digits>…`. A port is registered by
existing.

## Which files a porter may edit

- **Yours alone:** `src/doccheck/checks/<mod>/` (or `<mod>.rs`),
  `tests/doc_<id>.rs`, `equivalence/<id>.md`, and deleting
  `checks/<id>.sh`.
- **Shared, append-only:** `scripts/repo-state-checks.sh`. Add one section
  `# ------------------------------------------------------------ <id>`
  before the `end` section. Reuse the shared tool results at the top
  (`wire_check_rc`, `wire_check_out`, `wire_tables_*`). If your ticket
  needs a tool nobody runs yet, add one line to the shared block that runs
  it once and stores its result; on a merge conflict keep both sides.
- **Not yours:** every other file of `src/doccheck/` (the library and the
  harness), other ports, `scripts/per-change-suite.sh`, anything under
  `doc/`, and every other script under `checks/` (never weaken or edit one;
  the only allowed change to a script is deleting your own with its port).

### Adding a library helper

The library is shared by every port, so a change to it can break another
port's equivalence. The rules:

1. Start private. Write the helper in your own module
   (`src/doccheck/checks/<mod>/helpers.rs`). Your port lands with it.
2. Propose it separately: a commit or PR touching only `src/doccheck/`
   (`feat(tools): doccheck <helper>`), with unit tests in the module, that
   **adds** — a new function, a new enum variant, a new named policy
   constant — and never changes what an existing function returns.
3. A new variant of a disputed rule is a new named policy (for example
   `TagPolicy::PROSE_004_25`), documented in the policy's module table with
   the scripts that use it. Never edit an existing policy: other ports were
   proven equivalent with it.
4. After it merges, switch your port to it in a follow-up and rerun the
   harness.

## The library in one table

| Module | Use it for |
|---|---|
| `docs` | `Docs`: the documents as a value. `Docs::repo()` in tests; `docs.mutate(path, edit)`, `with`, `without` for fixtures (in memory, never a file in `/tmp`, never an environment variable); `Docs::at_commit` for a red-phase tree; `glob`, `is_file`, `commit_exists` |
| `markdown` | headings with GitHub anchors (`slug`), `section`, `section_with(SectionEnd)`, `find`, `find_by(HeadingMatch)`, `find_path`, `head_norms`, `tables_with(RowSplit)`, spans |
| `tables` | `Table` (`head`, `head_norm`, `col`, `col_named`), `Row` (`cell`, `joined`, `rest`, `line`), `split_row_with` |
| `units` | `units` (paragraphs, list items, table rows, headings with lines), `prose_blocks`, `strip_fences`, `clean_start` |
| `links` | `links`, `resolve` (file and anchor across files) |
| `evidence` | the six tags (`TagSet`), `untagged(text, &TagPolicy)`, `subsections_untagged` |
| `items` | `open_items` / `unowned_items(text, OpenItemPolicy)`, `labelled` ("Reported contradiction", "Hand-on", "Open item N"), `status_sentences`, `status_names` |
| `claims` | `affirms` (keyword in a clause with no negation), `exactly_one_of` (verdict words), `designations` (edition, draft, comment record, test suite, working-group document; a bare year never matches) |
| `ticket` | `Ticket::load` (acceptance criteria, DoD, `depends_on`, `sentence_after`), `expand_constant_ids`, `cid`, `Constants` (the constants table by id) |
| `wire_toml` | `WireToml` over the crate's TOML parser: `messages`, `message`, `array`, `field_info`, `fields`, `text`, `int`, `display` |
| `text` | `rx` (the regex cache: always use it, never `Regex::new` in a loop), `has`, `has_i`, `starts`, `full`, Python-compatible `splitlines`, `split_nl`, `join_digit_groups`, `first_int`, `ints`, `head` |
| `mutate` | fixture helpers by content: `delete_row`, `delete_unit`, `delete_section`, `rename_heading`, `strip_tags`, `replace_in_line`, `negate`, `replace_all`, `within_section` |
| `report` | `Report` (`check`, `fail`, `fail_at` with file and line), `assert_criterion`, `assert_clean`, `assert_fails` |

### The named policies

| Policy | Variants now | Where |
|---|---|---|
| `FencePolicy` | `Backticks` (004 family), `BackticksOrTildes` (003-02), `Matching` (006-01) | `policy` |
| `HeadingMatch` | `Regex`, `RegexAtLevel(n)` (`span_of`), `FullMatch` (004-09/10/11), `Substring` (003-02/05), `Exact` (006-02), `NormEq` (004-21), `Slug` | `policy` |
| `SectionEnd` | `SameOrHigher`, `AnyHeading` (003-08), `NextLevel2` (002-10 to 002-14) | `policy` |
| `RowSplit` | `Escaped` (most 004), `Plain` (004-22) | `policy` |
| `TagPolicy` | `UNITS_004_22`, `PROSE_004_27`, `PROSE_004_29` (004-25/28/29), `PROSE_004_26`, `ALPHA_004_16`, `ALNUM_004_14`, `EVERY_BLOCK_004_09` (004-03 to 004-11 family), `EVERY_BLOCK_004_13`; each sets tag set and case, unit, word count, minimum, table inheritance | `evidence` (its module documentation lists the variants still to add) |
| `OpenItemPolicy` | `Mention` (004-22; 004-24 and 004-25 test the same substring on their own blocks), `LeadingWithTicket` (004-27), `Leading` (004-29) | `items` |

The rest of the inventory's variants (the epic 002 pointer-line and
figure-paragraph windows, the separator tests, the reason-token patterns,
the negation look-backs, the `to_int` variants, the 13-name pin) are not in
the library yet: the first port that needs one adds it as above.

## Procedure

1. **Read the script whole**, header and code. Number every assertion it
   makes (every `fail`/`check`/`bad` call), and mark each as *document*
   (reads files under `doc/`) or *repository state* (runs a tool, reads git,
   the proof of concept, the script's own text). Note where the header and
   the code disagree.
2. **Name the variants.** For every shared rule the script uses (fences,
   headings, sections, tables, tags, open items, reasons), find or add the
   policy that reproduces it exactly.
3. **Write the check** in `src/doccheck/checks/<mod>/`. Keep the script's
   criterion labels (`AC1`, `DoD`, ...) as the `Report` criteria: the
   harness compares them. Keep each function under 100 code lines
   (`scripts/function-length.py`); split per criterion.
4. **Fill `SPEC`:** `script_env` (the override variables the script honours
   and that switch off its cargo runs: usually `[("WIRE_MD",
   "doc/design/spec/wire.md")]`), `targets` (the sections the corpus
   mutates: every section the check reads, and whole files such as
   wire.toml or the constants), `keywords` (words whose negation matters),
   and `known_differences` (empty until the harness shows one).
5. **Write the tests** in `tests/doc_<id>.rs`:
   - one `#[test]` per acceptance criterion and DoD item, named after it
     (`ac2_…`, `dod_…`), whose doc comment quotes the ticket sentence and the
     script header's description of what is checked;
   - the denial fixtures: every negative the script's header lists, plus one
     per assertion that has none, each the real documents with one mutation
     in memory (`base().mutate(WIRE_MD, |t| delete_row(t, …))`), asserting
     the right criterion fails with the right message (`assert_fails`);
     scope an edit with `within_section` when the same words occur in an
     earlier section;
   - the red-phase test: the tree at the commit that added the script fails
     the same criteria (skip when `commit_exists` is false).
   Compute the report on the real documents once (`OnceLock`).
6. **Run the harness** until it says EQUIVALENT:

   ```
   cargo run --release -p hadronyche-tools --bin doccheck-equiv -- <id> \
       --out hadronyche-tools/equivalence/<id>.md
   ```

   Options: `--seed N` (default 2026), `--limit N` (default 300 mutations),
   `--no-red` (skip the red-phase tree). Run a second, larger corpus with
   another seed (`--seed 7 --limit 2000`) before you commit.
7. **Relocate** the repository-state assertions to your section of
   `scripts/repo-state-checks.sh` and run it (`bash
   scripts/repo-state-checks.sh`).
8. **Commit** (Conventional Commits, `test(<id>): port checks/<id>.sh to
   Rust; delete the script`) with: the check module, the test file, the
   equivalence report, your repo-state section, and `git rm
   checks/<id>.sh`. The message lists every assertion of the script and
   where it now lives (test name, or repo-state section). Nothing a script
   asserted may disappear; an INFO/NOTE print is not an assertion.

### Gates before a port is pushed

`cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D
warnings`; `cargo fmt --check`; `python3 scripts/function-length.py`;
`python3 doc/kickoff/check-design.py doc/design`; `bash
scripts/repo-state-checks.sh`; every remaining `checks/*.sh` still green; the
harness report EQUIVALENT.

## The harness

`doccheck-equiv` (source `src/doccheck/equiv/`, bin
`src/bin/doccheck-equiv.rs`) compares the port with its script on the same
inputs:

- **(a)** the current tree: both must pass;
- **(b)** the tree at the commit that first added the script
  (`git log --diff-filter=A -- checks/<id>.sh`), where the script was written
  red: both must fail, on the same criteria;
- **(c)** a deterministic corpus of structured mutations of the `targets`:
  every table row deleted, every unit deleted, every subsection deleted,
  every tag stripped, keywords negated (`no`→`a`, `not`→``, `is`→`is not`,
  the spec's `keywords` → `not …`, …), every number incremented, every
  heading renamed; each (kind, target) group shuffled by the seed, targets
  interleaved within a kind, kinds taken round-robin up to the limit. For
  every mutation the script's exit status must equal the port's verdict.

**Acceptance rule.** A verdict difference is acceptable only when the
port's `known_differences` classify it, with a reason a reviewer can check:
`PortStricter` (the port rejects what the script accepted, and why that is
right) or `ScriptWrong` (how the script is wrong). A mutation the script
rejects and the port accepts is **port weaker** and fails the run, whatever
reasons are recorded; an unclassified difference fails the run too. When
both are red, differing criteria are counted and shown but do not fail the
run (the verdict is the contract; look at them anyway).

**How the script runs.** The harness exports `doc/` from the same `Docs` the
port reads (the working tree, or `git archive` of the red-phase commit) into
a temporary directory outside the repository, writes the script to
`checks/<id>.sh` there, and runs `bash checks/<id>.sh` from that directory,
so a script that reads fixed paths reads the export. The spec's
`script_env` variables (e.g. `WIRE_MD`) point at the exported files, which
makes the scripts that honour them skip their cargo runs; `cargo` on the
script's `PATH` is a stub that exits 97, so a cargo run that was not switched
off makes the script red instead of silently testing the real tree. For a
mutation, the harness rewrites the one exported file, runs the script and
restores the file. The directory is removed on exit.

**Limits.** The script is run on the export of `doc/` only: a script that
reads outside `doc/` (the proof of concept, untracked notes, `Cargo.toml`)
cannot be compared, which is one reason for the not-ported list. Python
crashes (a traceback, exit 1, no FAIL line) count as red with no criteria.
The corpus is structural, not semantic: most mutations of a large section
are neutral (both pass), so run a large corpus; the denial fixtures cover
the specific negatives. Mutations are line-based and never touch fenced
code.

**Where it runs.** Never in `cargo test`, the per-change suite or CI: it
needs the old scripts and Python. It reads a deleted script from the parent
of the commit that deleted it, so it stays usable until the last port.

## Naming

- module `c<id with _>` (`c004_27`), constant `SPEC`, function `run`;
- tests file `doc_<id>.rs`; test functions `ac<N>_<what the criterion
  says>`, `dod_<item>`, `ac<N>_denial_<what is broken>`,
  `red_phase_tree_fails_every_criterion`;
- criteria in the `Report` as the script printed them (`AC1`, `AC 1` →
  `AC1`, `DoD`, `WIRE`);
- repo-state lines `PASS <id> <criterion>: …` / `FAIL <id> <criterion>: …`.
