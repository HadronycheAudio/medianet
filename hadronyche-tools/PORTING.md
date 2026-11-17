# Porting a `checks/*.sh` script to a Rust document check

The ticket checks under `checks/` are bash wrappers around embedded Python,
one per ticket. They move into this crate one at a time:
- the document assertions become `cargo test` tests on the library
  `hadronyche_tools::doccheck`;
- the assertions that run tools or read repository state go to
  `scripts/repo-state.d/<id>.sh`;
- the script is deleted in the same commit.

This file is the procedure. It is written so that several porters can work in
parallel without editing the same files.

## Rulings that bind every port

1. **Equivalence first.** A port reproduces *its own script's* variant of
   every rule, even where scripts disagree. The same rule is written about
   fifteen ways across the scripts (`swarm/notes/checks-inventory-2026-11-16.md`,
   section 4).
   - The library offers each disputed rule as a function that takes an
     explicit, named policy. The port names its script's policy at the call
     site.
   - Unifying the variants is a later, separate, reviewed step. It is never a
     side effect of a port.
2. **Python compatibility is part of equivalence.** The scripts ran on
   Python. Wherever their Python differs from Rust's defaults, use the
   helper of `doccheck::pycompat`, never the Rust default:
   - `\w`, `\W`, `\s`, `\S`: Python's classes. `²` is a word character, a
     combining mark is not, and `\x1c`–`\x1f` are white space. `text::rx`
     translates every pattern, so you get them by using `rx`.
   - `strip`, `lstrip`, `rstrip`, `split()`: the `PyStr` trait
     (`py_strip`, `py_lstrip`, `py_rstrip`, `py_split`). Never use `trim`
     or `split_whitespace` in a check.
   - Reading a file: `open()` gives universal newlines. Splitting:
     `split("\n")` or `splitlines()`. Pick `LineSplit::Nl` or
     `LineSplit::PySplitlines`, and normalise every document text with it
     before use.
   - Paths: `os.path.normpath` keeps a leading `..` (`docs::join`, policy
     `PathNorm`). A path outside the repository names no document.
   - Integers: `int()` has no size limit. Use `pycompat::parse_int` and
     `text::first_int`/`ints`, which are checked. A number beyond `i128` is
     *reported* as a failure, never wrapped, and never a panic. Arithmetic
     on document numbers saturates.
   - TOML values as the script's reader and Python saw them: `py_int`
     (`int()`), `py_is_int` (`isinstance`), `py_eq_int` and `py_eq`
     (`40.0 == 40`), `py_iter` (a string iterates its characters),
     `py_dicts`.
   - Known limit: `\b` keeps Rust's word definition. It differs from
     Python's only next to `²`-like numerals and combining marks. Write the
     boundary as a class if your script depends on that.
3. **TOML reader semantics.** Most 004 scripts used a hand-written subset
   reader (`TomlReader::SubsetReader`); 004-22 used `tomllib`. Ports parse
   with the crate's TOML parser and read values through `pycompat`.
   - A type error the script caught must still fail. Example:
     `rejection_reasons = "BadLength"`, where a string stands for an array.
   - A script crash (KeyError, TypeError) is reported by the port as a
     failure ("the script's reader crashes here").
   - Where the subset reader rejects valid TOML and no assertion was aimed
     at that, classify the difference `SubsetTomlReader`.
   - Where the typed parser rejects what Python let through, classify it
     `PortStricterTyped`.
4. **Not ported.** These stay scripts, and the suite keeps running them:
   - 003-01, 003-03, 003-04, 003-06, 003-07, 003-09: they freeze-check,
     build or run the proof of concept;
   - 003-02, 003-05, 003-08: their content parts read files outside the
     repository (untracked notes, the proof of concept's `BASELINE.md`).
5. **Tool gates run once.** An assertion that runs a tool would be circular
   or environment-dependent inside `cargo test`. Examples: `cargo run --bin
   wire-check`, `cargo test --test wire_tables`, `check-design.py`, sibling
   scripts, git against a ref, the proof of concept.
   - Such assertions go to `scripts/repo-state.d/<id>.sh`.
   - `scripts/repo-state-checks.sh` runs each tool once, with the scripts'
     900 s timeout, and exposes the results.
   - Per-ticket pins ("at least 12 passed") stay as named lines.
6. **A check reads only its `Docs`.** The library test
   `doccheck::checks::tests::checks_read_only_their_docs` scans every check
   source and every `tests/doc_*.rs` file for forbidden uses, and fails on
   any of them:
   - processes, files and the environment (`std::process`, `Command`,
     `std::fs`, `env::var`, `include_str!`, …);
   - other document sources (`Docs::repo`, `Docs::dir`, `at_commit`,
     `repo_root`, `commit_exists`).

   Test files get their documents from `doccheck::testing`, the allow-list.
   A script's "reads the working tree only / runs no other check" DoD lives
   there.
7. **Header and code disagree → follow the code.** The code is what has been
   passing. Record the disagreement in the test's doc comment.
8. **A check reports, never panics,** on any input.
   `checks::tests::no_check_panics_on_malformed_input` runs every registered
   check on malformed documents (empty, garbage, huge integers, wrong TOML
   types).

## What a port consists of

For ticket `<id>` (`004-27`), with `<mod>` = `c` + the id with `-` as `_`
(`c004_27`):

| File | What |
|---|---|
| `src/doccheck/checks/<mod>/mod.rs` | the check: `pub fn run(docs: &Docs) -> Report`, `pub const SPEC: CheckSpec`, `const KNOWN` (classified differences); split into files per criterion |
| `src/doccheck/checks/<mod>/denials.rs` | `pub const DENIALS: &[Denial]`: the denial list (see below) |
| `tests/doc_<id>.rs` | one `#[test]` per acceptance criterion and DoD item, the red-phase test, and `hadronyche_tools::denial_tests!(SPEC; …)` with every denial name |
| `equivalence/<id>.md` | the committed harness summary |
| `scripts/repo-state.d/<id>.sh` | the relocated repository-state assertions |
| `checks/<id>.sh` | deleted, in the port's commit |

There is no registry to edit: `build.rs` generates it from
`src/doccheck/checks/c<digits>…`. A malformed entry fails the build with a
message, for example a directory without `mod.rs` or a `c…` file that is not
`.rs`.

`SPEC` fields:

| Field | What it holds |
|---|---|
| `id` | the ticket id |
| `run` | the check function |
| `script_env` | the override variables that make the script skip its cargo runs, usually `[("WIRE_MD", "doc/design/spec/wire.md")]` |
| `targets` | the sections the corpus mutates |
| `keywords` | extra words to negate |
| `known_differences` | `KNOWN` |
| `denials` | `denials::DENIALS` |
| `toml` | the script's `TomlReader` |

## Which files a porter may edit

- **Yours alone:**
  - `src/doccheck/checks/<mod>/` (all of it);
  - `tests/doc_<id>.rs`;
  - `equivalence/<id>.md`;
  - `scripts/repo-state.d/<id>.sh`;
  - deleting `checks/<id>.sh`.
- **Shared, by addition only:** a new shared tool goes in
  `scripts/repo-state-checks.sh`, as one block in "shared tool runs" that
  exports its results. On a merge conflict, keep both blocks.
- **Not yours:**
  - every other file of `src/doccheck/` (the library and the harness);
  - other ports;
  - `scripts/per-change-suite.sh`;
  - `.github/`;
  - anything under `doc/`;
  - every other script under `checks/`. Never weaken or edit one; the only
    allowed change to a script is deleting your own, with its port.

### Adding to the library, and fixing a library bug

The library is shared by every port, so a change to it can break another
port's equivalence.

1. **Start private.** Write a helper in your own module
   (`src/doccheck/checks/<mod>/helpers.rs`). Your port lands with it.
2. **A new rule variant** is a new named policy: a new enum variant or a new
   constant (`TagPolicy::PROSE_004_25`, `TableRules::…`), documented in its
   module's table with the scripts that use it. Never change what an
   existing policy does: other ports were proven with it.
3. **A library addition** is its own commit or PR touching only
   `src/doccheck/` (`feat(tools): doccheck <what>`). It carries unit tests
   and adds only.
4. **A library bug fix** changes existing behaviour, for example a Python
   semantic the library got wrong (round 1 of PR #42 fixed `\w`,
   `normpath`, `splitlines`, the separator rule). It goes in a library PR
   that:
   - fixes the primitive and adds a unit test for it;
   - reruns `cargo run --release -p hadronyche-tools --bin doccheck-equiv --
     --all --out hadronyche-tools/equivalence`, which reruns every
     registered port with its denials;
   - commits the regenerated summaries;
   - lists in the PR every port whose summary changed and why.

   A port that becomes "port weaker" blocks the fix until it is resolved.
5. After an addition merges, switch your port to it in a follow-up and rerun
   the harness for your id.

## The denial list

A denial is the real documents with one edit applied in memory, plus what
the port must report on them (`Denial { name, file, edit, expect }`):
- `edit` is a `DenialEdit::Text(fn)` built from `doccheck::mutate` helpers,
  or `DenialEdit::Delete`;
- `expect` lists `(criterion, message substring)` pairs. An empty `expect`
  means the port must pass: the agreement is the point, or the script's red
  is a classified difference.

Every port has ONE list (`denials.rs`), used twice:
- **as tests:** `denial_tests!(SPEC; name, …)` makes one `#[test]` each,
  plus a test that the list and the tests agree;
- **by the harness against the old script:** the targeted denials are the
  strongest evidence, since a random corpus is only 8–13 % red.

Write one denial per negative the script's header lists, plus one per
assertion that has none. Scope an edit with `mutate::within_section` when
the same words occur in an earlier section.

## The library in one table

| Module | Use it for |
|---|---|
| `docs` | `Docs`, the documents as a value: `with`, `without` and `mutate` for edits in memory; `glob`; `is_file`; `join` (`normpath`). `at_commit` returns `Err` on any git failure. |
| `pycompat` | Python semantics: `translate`, `PyStr`, `splitlines`, `universal_newlines`, `normpath`, `parse_int`, `py_int`, `py_is_int`, `py_eq_int`, `py_eq`, `py_iter`, `py_dicts`, `is_word`, `is_space`, `is_nd` |
| `policy` | `FencePolicy`, `HeadingMatch`, `SectionEnd`, `RowSplit`, `SeparatorRule`, `HeaderlessRuns`, `TableRules`, `LineSplit`, `PathNorm`, `TomlReader` |
| `markdown` | headings with GitHub anchors, `section`, `section_with`, `find`, `find_by` (literal variants never compile a regex), `find_path`, `head_norms`, `tables_with(span, TableRules)` |
| `tables` | `Table`, `Row`, `split_row_with`, `is_separator`, `is_separator_line` |
| `units` | `units`, `prose_blocks`, `strip_fences`, `clean_start` |
| `links` | `links`, `resolve`, `file_slugs` |
| `evidence` | `TagSet`, `TagPolicy` constants, `untagged`, `subsections_untagged` |
| `items` | `OpenItemPolicy`, `open_items`, `unowned_items`, `labelled`, `status_sentences` |
| `claims` | `must_affirm` (the document must say it) and `claims` (the document must not say it): two directions of the negation guard; `exactly_one_of`; `designations` (a bare year or year range is never an edition) |
| `ticket` | `Ticket`, `expand_constant_ids`, `cid`, `Constants` |
| `wire_toml` | `WireToml`, `fields`, `text`, `int`, `display` |
| `text` | `rx` (Python classes; a cache bounded at `RX_CACHE_LIMIT`, sharing compiled regexes; escape document text you embed in a pattern), `rx_raw`, `has`, `has_i`, `starts`, `full`, `first_int`, `ints`, `join_digit_groups`, `head` |
| `mutate` | `delete_row`, `delete_unit`, `delete_section`, `rename_heading`, `strip_tags`, `replace_in_line`, `negate`, `replace_all`, `within_section` |
| `report` | `Report` (`check`, `fail`, `fail_at`), `assert_criterion`, `assert_clean`, `assert_fails` |
| `denial` | `Denial`, `DenialEdit` |
| `testing` | `repo`, `report_of`, `red_phase`, `check_denial`, `denial_tests!` |

### The named policies

| Policy | Variants now |
|---|---|
| `LineSplit` | `Nl` (004-22), `PySplitlines` (004 family) |
| `TableRules` (`RowSplit` × `SeparatorRule` × `HeaderlessRuns`) | `T004_22` (plain split, cell-wise separator, headerless runs kept); `FAMILY_004` (escaped split, line-regex separator, headerless runs dropped; 004-01, 03-06, 08-13, 16-21, 24-29) |
| `FencePolicy` | `Backticks` (004 family), `BackticksOrTildes` (003-02), `Matching` (006-01) |
| `HeadingMatch` | `Regex`, `RegexAtLevel(n)` (`span_of`), `FullMatch` (004-09/10/11), `Substring` (003-02/05), `Exact` (006-02), `NormEq` (004-21), `Slug` |
| `SectionEnd` | `SameOrHigher`, `AnyHeading` (003-08), `NextLevel2` (002-10 to 002-14) |
| `PathNorm` | `PyNormpath` |
| `TomlReader` | `Tomllib` (004-22), `SubsetReader` (004 family) |
| `TagPolicy` | `UNITS_004_22`, `PROSE_004_27`, `PROSE_004_29` (004-25/28/29), `PROSE_004_26`, `ALPHA_004_16`, `ALNUM_004_14`, `EVERY_BLOCK_004_09`, `EVERY_BLOCK_004_13`. Each sets the tag set and case, the unit, the word count, the minimum and table inheritance. |
| `OpenItemPolicy` | `Mention` (004-22), `LeadingWithTicket` (004-27), `Leading` (004-29) |

The word class is not a parameter: `rx` is always Python's.

**Not in the library yet.** The first port that needs one of these adds it
as above:
- the epic 002 pointer-line and figure-paragraph tag windows;
- 003-08's three-tag rule and 004-04's list inheritance;
- the reason-token regexes, the negation look-backs and the `NON_REJECTION`
  sets;
- the `to_int` variants;
- the 13-name pin.

## Procedure

1. **Read the script whole**, header and code.
   - Number every assertion it makes (every `fail`, `check` or `bad` call).
   - Mark each one as *document* (reads files under `doc/`) or *repository
     state* (runs a tool, reads git or the proof of concept, greps its own
     text).
   - Note where its Python semantics matter: how it reads and splits
     lines, `\w`, `int()`, TOML values.
   - Note where the header and the code disagree.
2. **Name the variants.** For every shared rule the script uses, find or add
   the policy that reproduces it exactly.
3. **Write the check** in `src/doccheck/checks/<mod>/`.
   - Keep the script's criterion labels (`AC1`, `DoD`, `WIRE`) as the
     `Report` criteria; the harness compares them criterion by criterion.
   - Keep each function under 100 code lines (`scripts/function-length.py`).
4. **Write the denial list and the tests**, then fill `SPEC`.
5. **Run the harness** (below) until it says EQUIVALENT. Classify every
   difference that is genuinely not "port weaker" in `KNOWN`, with a reason
   a reviewer can check.
6. **Relocate** the repository-state assertions to
   `scripts/repo-state.d/<id>.sh`, and run `bash
   scripts/repo-state-checks.sh`.
7. **Commit** with the Conventional Commits title `test(<id>): port
   checks/<id>.sh to Rust; delete the script`. The commit holds the check,
   the denial list, the test file, `equivalence/<id>.md`, the repo-state
   file, and `git rm checks/<id>.sh`.
   - The message lists every assertion of the script and where it now
     lives.
   - Nothing a script asserted may disappear. An INFO or NOTE print is not
     an assertion.

### Gates before a port is pushed

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `python3 scripts/function-length.py`
- `python3 doc/kickoff/check-design.py doc/design`
- `bash scripts/repo-state-checks.sh`
- every remaining `checks/*.sh` still green
- the harness summary EQUIVALENT

## The harness

`doccheck-equiv` lives in `src/doccheck/equiv/`, with the bin
`src/bin/doccheck-equiv.rs`. Usage:

```
cargo run --release -p hadronyche-tools --bin doccheck-equiv -- <id> --out hadronyche-tools/equivalence/<id>.md
cargo run --release -p hadronyche-tools --bin doccheck-equiv -- --all --out hadronyche-tools/equivalence
cargo run --release -p hadronyche-tools --bin doccheck-equiv -- <id> --denials-only     # quick: (a) and the denials
```

**The committed run is the default.** It covers:
- **(a)** the current tree: both must pass;
- **(b)** the tree at the commit that first added the script: both red on
  the same criteria;
- **(c)** two seeds (2026 and 7) of 2 000 structured mutations each: rows,
  units and subsections deleted, tags stripped, keywords negated, numbers
  incremented, headings renamed. Each (kind, target) group is shuffled by
  the seed, then targets and kinds take turns;
- **(d)** the denial list, on both the script and the port.

Other options: `--seeds A,B`, `--limit N`, `--no-red`, `--no-denials`,
`--full` (every row; never commit it).

It writes the **summary**: the tree and script commits, counts per seed and
for the denials, rows (a) and (b), and every row that is not "same". A
script run takes about 0.3 s, so the committed run takes about 20 minutes
per check.

**Comparison.**
- "Same" means both pass, or both fail the same set of criteria.
- Every other pair is a difference of a kind:
  - script red, port pass;
  - script pass, port red;
  - port fails more criteria;
  - port misses a criterion;
  - script crash with port red;
  - script crash with port pass.
- A script **crash** (traceback, non-zero exit without a FAIL line, or the
  300 s timeout) is its own outcome; it never matches a port red.
- A script that prints FAIL and exits 0 counts as red and is reported as an
  anomaly.
- Every edit of the export is read back after writing and after restoring.
  Any error aborts the run, so an unapplied mutation can never read "same".

**Acceptance rule.**
- A difference passes only when a `KNOWN` entry matches its label, with a
  class allowed to excuse that kind:

  | Class | Use |
  |---|---|
  | `PortStricter` | the port rejects what the script accepted, for a stated reason |
  | `PortStricterTyped` | the typed TOML parser rejects what Python let through |
  | `ScriptWrong` | the script's verdict is wrong; the reason says how |
  | `SubsetTomlReader` | the subset reader rejected valid TOML |
  | `ScriptCrashed` | the script crashed; the port reports the cause |

- The weaker kinds (script red/port pass, port misses a criterion, script
  crash/port pass) are "**port weaker**". No class excuses them, except
  `SubsetTomlReader` (lead ruling R3).
- An unexcused difference, or a denial whose expectations the port does
  not meet, fails the run.

**How the script runs.**
- The harness exports `doc/` from the same `Docs` the port reads (the
  working tree, or `git archive` of the red-phase commit) into a temporary
  directory outside the repository, about 14 MB, removed on exit.
- It writes the script to `checks/<id>.sh` there and runs it from that
  directory, so fixed paths resolve inside the export.
- `script_env` points override variables at the exported files.
- A `cargo` stub that exits 97 makes any un-skipped cargo run red.
- A deleted script is read from the parent of the commit that deleted it.

**Limits.**
- Only `doc/` is exported, so a script that reads outside it cannot be
  compared (one reason for the not-ported list).
- The corpus is structural and mostly neutral; that is why there are
  denials.
- Mutations are line-based and never touch fenced code.

**Where it runs.** Never in `cargo test`, the per-change suite or CI. It
needs the old scripts and Python.

## Red-phase tests

```rust
#[test]
fn red_phase_tree_fails_every_criterion() {
    red_phase(&SPEC, "<full commit>", &["AC1", "AC2", "AC3", "AC4", "DoD"]);
}
```

- A missing commit **fails** the test. With `DOCCHECK_ALLOW_SHALLOW=1` it
  prints a visible `SKIPPED` line instead.
- A run prints a visible `EXECUTED` line. CI checks out the full history
  (`fetch-depth: 0`) and runs the red-phase tests in their own step, so the
  log shows them.

## Naming

- Module `c<id with _>` (`c004_27`), constant `SPEC`, function `run`,
  denial list `denials::DENIALS`.
- Test file `doc_<id>.rs`. Tests are named `ac<N>_<what the criterion
  says>` and `dod_<item>`; denials `ac<N>_denial_<what is broken>`;
  review-born denials `review_<finding>_<what>`.
- Criteria in the `Report` as the script printed them (`AC 1` becomes
  `AC1`, plus `DoD`, `WIRE`).
- Repo-state lines read `PASS <id> <criterion>: …` and `FAIL <id>
  <criterion>: …`.
