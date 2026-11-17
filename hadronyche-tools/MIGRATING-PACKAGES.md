# Migrating the four port packages onto the library (M1-lib)

The packages `m/port-wp7` (ad8b544), `m/port-wp8` (8d61625), `m/port-wp9`
(1d524a4) and `m/port-wp5` (8234c4b) were each built with private
compatibility and helper layers. The library on `m/doccheck-lib` now
carries everything they share. Each layer was rewritten from the scripts
(never from a porter's copy) and proven against the scripts' own Python by
a committed corpus (PORTING.md, "The named policies").

This file gives each porter the exact mapping from a private item to a
library item, so adaptation is mechanical. It also lists what was
deliberately not taken, and every place where a private copy disagreed with
its script's Python, or with another package's copy. Each such place is a
verdict the port will change when it moves to the library; rerun
`doccheck-equiv` and expect it.

Merge order: wp7, wp8, wp9, wp5. Each branch merges `m/doccheck-lib`,
deletes its private layer, applies the mapping below, and reruns
`doccheck-equiv <id>` with the committed defaults for each of its checks.

## For every package

1. **Running a check.** Tests and the harness run a check only through
   `SPEC.evaluate(&docs)`. A source test fails on `(SPEC.run)(…)`.
2. **The repertoire guard** is a property of `Docs` (R20). Delete every
   `repertoire::guard` call and every input list. wp7's five ports, which
   had no guard, are covered without change.
3. **Regex.** Use `text::rx`, never `rx_raw` (source test).
   - `rx` takes the script's pattern as written, look-behind and look-ahead
     included. Python's classes, `\b`, `$` and the empty-match rule are
     built in.
   - Delete every hand-written guard scan or "shadow text" layer.
   - `\Z`, `{,n}`, back-references, possessive quantifiers, atomic groups
     and repeated quantifiers are refused.
   - An engine failure is `REGEX_LIMIT` (R19).
4. **Integers** go through `pycompat::parse_int` (or `PyValue::py_int`).
   - Beyond `i128`, a number is red by rule under `INT_LIMIT`, recorded
     automatically. No saturating, clamping, `i64`, `unwrap_or(0)` or
     float fallback.
5. **Crashes** are `report::Stop` / `Report::crash` (PORTING.md, ruling 4,
   "One `Stop` type").
   - Never a failure with criterion `"CRASH"`, and never a FAIL under
     another criterion.
6. **State and threads.** A check module may not hold any of: `static`,
   `OnceLock`, `LazyLock`, `thread_local!`, `Mutex`, threads (source test
   and clippy).
7. **Test files.** Move each check's test file `tests/doc_<id>.rs` to
   `tests/doc/c<id>.rs`, unchanged. It becomes one module of the single
   test binary `tests/doc.rs`; `build.rs` lists the modules, so no shared
   file is edited.
   - Denial tests run in the release step (`cargo test --release -p
     hadronyche-tools --test doc`); the debug `cargo test` ignores them.
   - Write CI and suite filters with `--test doc`, not `--test 'doc_*'`.
   - A `slow:` list in `denial_tests!` is optional: every denial runs in
     the release step.
8. **No module is named after a work package**, and no port imports
   another (with any prefix: `super::c004_14::` included).
9. **`CheckSpec` has a new required field, `fail_reader`.**
   - Set it to `crate::doccheck::equiv::script::DEFAULT_READER` unless the
     script prints its failures another way.
   - A port-specific reader is a `fn(script: &str, lines: &[&str]) ->
     BTreeSet<String>` in its own file, named in that port's spec. It
     plugs in without touching shared lines.
   - Epic 002 (WP1's `equiv::labels002`, WP2's
     `doccheck::fail_form::colon_criterion`): one reader per port, never
     two on one port, never selected by `DOCCHECK_FAIL_FORM` or any other
     environment variable.

## wp7 (004-16, 004-17, 004-18, 004-19, 004-20)

| Private item | Library item |
|---|---|
| `tests/doc_004_16.rs`, `tests/doc_004_17.rs`, `tests/doc_004_18.rs`, `tests/doc_004_19.rs`, `tests/doc_004_20.rs` | `tests/doc/c004_NN.rs` (same content; one module of `tests/doc.rs`) |
| `doccheck::wp7::pytoml::parse`, `PyVal`, `PyDict` | `tomlsubset::parse` → `pyvalue::Dict` / `PyValue` |
| `Fail::Error` / `Fail::Crash` | an `Err` whose type prefix is `TomlError:` / any other type prefix |
| `HUGE` clamping (10^36) | `PyValue::BigInt` → `INT_LIMIT` when read as an int |
| `wp7::py::{universal, splitlines, is_space, strip, lstrip}` | `pycompat::{universal_newlines, splitlines, is_space, PyStr}` |
| `wp7::py::int` | `pycompat::parse_int` |
| `wp7::py::{py_word, shadow, fold_i, compile, finditer, search, has, has_i, matches, fullmatch, sub}` | `text::rx` (`find_iter`, `captures`, `replace_all`), `text::{has, has_i, starts, full}` |
| `wp7::py::{len, slice, char_index, head}` | `chars().count()`, `pycompat::py_slice`, `text::head` |
| `wp7::md::{split_row, is_separator}` | `tables::split_row`, `tables::is_separator_line` |
| `wp7::md::{parse_tables, PyTable, first_table}` | `family004::tables`, `family004::first_field_table` |
| `wp7::md::{heading_index, Head}` | `family004::{heading_index, Head}` |
| `wp7::md::{strip_fences, prose_blocks, clean_start}` | `units::{strip_fences, prose_blocks, clean_start}` |
| `wp7::md::subsections` | `family004::subsections` |
| `wp7::md::section_span` | `family004::section_span(.., SpanRule::AfterHeading004_16)` |
| `wp7::md::{key_of, KEY_RULES}` | `family004::{key_of, KEY_RULES_004_16}` |
| `wp7::md::{is_field_table, width_of, Field, fields_of, find_sub, has_evidence}` | `family004::*` / `evidence::has_tag` |
| `wp7::common::{NUM, to_int}` | `family004::{NUM_004_16, to_int(_, IntRule::Spaces004_16)}` |
| `wp7::common::{const_val, const_int}` | `family004::{const_cell, const_int(.., ConstInt::AnyCase004_16)}` |
| `wp7::common::{md_name, norm_name, tname}` | `family004::{md_name, norm_name(_, NameNorm::Spaced004_16), tname}` |
| `wp7::common::split_sentences` | `sentences::split_raw(_, RAW_PERIOD_SEMI)` |
| `wp7::common::named_reasons` (24-character look-back) | `rejection_members::reasons::named_reasons(o, \|w\| e.contains(w), LOOKBACK_004_16)` (or `negation::negated(.., &Lookback::WINDOW_24_004_16)`) |
| `wp7::common::words` | `evidence::count_words(_, WordCount::AlphaNoCode)` |
| `wp7::common::is_open_item` | `items::OpenItemPolicy::LeadingClean` |
| `wp7::common::form_rows` | `blocks::form_rows(_, FormRows::AnyCell004_17)` |
| `wp7::common::{section_blocks, block_kind}` | `blocks::{section_blocks, block_role(_).name()}` |
| `doccheck::rejection_members::Enumeration::parse(t)` | `rejection_members::Enumeration::parse(t, REJECTION_004_16)`; `has_exactly`, `contains`, `get`, `names` unchanged; `digit_value` → `parse_int` |
| `c004_18` `named_in` | `reasons::named_reasons` over the union |
| 004-19/20 inline 25-character guard (`c004_19/ac5.rs`, `c004_20/ac5.rs` `NEGATED`) | `reasons::{negated_at, negated_tokens}` with `LOOKBACK_004_19` |
| `NON_REJECTION` in `c004_16/17 ac4.rs`, `c004_18/ac6.rs`, `c004_20/ac5.rs` | `reasons::NON_REJECTION_004_16`, `_004_18`, `_004_20` |
| `c004_20::{sentences, norm}` | `SentenceSplit::NEWLINES_004_20`, `NameNorm::Letters004_20` |
| `c004_19` `const_row` | `family004::const_line` |
| DoD rules of 004-16/17/18 | `TagPolicy::ALPHA_004_16` + `TableTagRule::SectionTagged004_16` + `OpenItemPolicy::LeadingClean` |
| DoD rules of 004-19 / 004-20 | `TagPolicy::BLOCKS_004_19` + `TableTagRule::AfterProse004_19` + `OpenItemPolicy::Window3` (004-19) / `WordLeading` (004-20) |
| `report.fail("CRASH", …)` in `c004_16/ac5, ac6`, `c004_17/ac6`, … | `Stop::Crash` / `Report::crash` |
| `wp7::py`'s `text::rx_raw as rx` import | `text::rx` |

Not taken:
- The shadow text and the dotless-ı/İ fold: they are moot under the
  repertoire guard.
- `label(ac)`: one line of report formatting.
- `common`'s value helpers (`Num`, `mul`, `get_or`, `ne`, `str_of`, `read`,
  `set_of`, `sortable`): use `PyValue`.

Delete the whole `doccheck::wp7` module and `doccheck::rejection_members`
from the branch; the library has both.

**Disagreements** (each one changes a verdict when the port moves):
- `wp7::pytoml` clamps ints at 10^36 (5 corpus snippets).
  - `123456789012345678901234567890123456789012`: Python keeps the value,
    wp7 gives 10^36.
  - A 4 301-digit scalar: Python keeps it as a string, wp7 gives 10^36.
- `wp7::py::int("1_000")`: Python gives 1000, wp7 rejects it. Above 10^36,
  wp7 saturates silently.
- `common::const_int` turns Python's `ValueError` into `None`. On
  02-constants.md `C-13` with the pattern `([0-9 ]+)`, Python raises
  (crash) and wp7 gives `None`. This is latent: the real calls capture
  `NUM`, which always holds digits.
- Crashes are failures with criterion `"CRASH"`, which the harness reads as
  an ordinary red port, never as the script's crash.
- Agreement: wp7's `Enumeration::parse` and `named_reasons` agree with
  Python on all 45 rejection cases and 29 outcomes. Its other 22 Markdown
  and common helpers agree on all 223 family-004 inputs, except
  `const_int` above.

## wp8 (004-24, 004-25, 004-26, 004-28, 004-29)

| Private item | Library item |
|---|---|
| `tests/doc_004_24.rs`, `tests/doc_004_25.rs`, `tests/doc_004_26.rs`, `tests/doc_004_28.rs`, `tests/doc_004_29.rs` | `tests/doc/c004_NN.rs` (same content; one module of `tests/doc.rs`) |
| `c004_24::pytoml::parse_toml`, `toml::Value` | `tomlsubset::parse` → `PyValue` (big ints become `BigInt`, not `Float`) |
| its single `Err(String)` | the `Err`'s type prefix decides crash versus caught, as each script's `except` names it |
| `py_iter`, `dicts`, `tables_strict`, `names`, `eq_int`, `as_int` | `PyValue::{py_iter_str, py_dicts, eq_int, py_int}` |
| `c004_24::py::{tables, first_table}` | `family004::{tables, first_field_table}` (= `FAMILY_004`, proven) |
| `py::split_after(_, ".;")` | `sentences::split_raw(_, RAW_PERIOD_SEMI)` |
| `py::sentences_lower(_, ".;:")` | `sentences::split(_, SentenceSplit::COLON_004_24)` |
| `py::nums_before` | `text::rx(r"(?<![\d.])(\d+)\s*bytes")` + `parse_int` |
| `py::has_number` | `text::has(&format!(r"(?<!\d){n}(?!\d)"), …)` |
| `py::truthy` | `PyValue::truthy` |
| `py::py_int` | `PyValue::py_int` |
| `py::{py_text, py_chars}` (shadow) | not needed under the repertoire guard |
| `c004_25::tree::py_eval` | `pynum::eval_arith` (`PyNum`, `EvalError`) |
| `dod::{LINES_004_24, untagged_lines}` | `evidence::{LinePolicy::LINES_004_24, untagged_lines}` |
| `dod::{MENTION_BLOCKS_004_24, block_items}` | `items::OpenItemPolicy::MentionBlocks`, `open_items` / `unowned_items` |
| `c004_25::review::sentences` | `SentenceSplit::BLOCKS_004_25` |
| `review::split_pieces_with(_, false)` | `SentenceSplit::PARAGRAPHS_004_29` |
| `c004_24::landed::rejection_members(ctx, pat)` | `Enumeration::parse(wire, REJECTION_004_24)` (004-24), `REJECTION_004_25` (004-25, 004-28, 004-29) |
| `c004_25::merged::rejections_pinned` | `Enumeration::matches_fixed(&REJ_FIXED)` |
| `Merged::is_control` | `Enumeration::control_members()` |
| 004-29's `needed` list | `has_control_reasons(13, &CONTROL_REASONS_004_29)` |
| `c004_26::landed::enum_names(section_lines(…))` | `REJECTION_004_26` / `REFUSAL_004_26` |
| `NONREJ` in `c004_24/validity.rs`, `c004_25/validity.rs`, `c004_29/forms.rs` | `reasons::NON_REJECTION_004_24`, `_004_25`, `_004_29` |
| `c004_26::{heading_index, section_lines}` | `Markdown.heads` / `find_by(HeadingMatch::RegexAtLevel(n))` + `section` (or `family004::section_span(.., SpanRule::WithHeading004_24)`) |
| `c004_26::to_int` | `family004::to_int(_, IntRule::HexCommas004_26)` |
| `c004_26` open paragraphs | `OpenItemPolicy::Head40` |
| `c004_26` subsection rule | `evidence::subsections_with_table_untagged` |
| `c004_26` prose rule | `TagPolicy::PROSE_004_26` |
| `c004_25/28/29` prose rule | `TagPolicy::PROSE_004_29` |
| `c004_29` open items | `OpenItemPolicy::Leading`; its first-sentence split → `sentences::first_sentence_004_29` |
| `c004_24::Stop(String)`, `py::stopped(report, fallback, why)` | `report::Stop::Crash` + `Report::finish` |

Not taken:
- `c004_25/merged.rs` (`code_block`, `reserved_rows`, `rejections_pinned`'s
  check logic) and `review.rs` (`capnorm`, `c1` to `c4`) are 004-25's own
  checks; they stay in the port, built on the library.

**Disagreements:**
- **Port weaker, inside the repertoire.** `landed::rejection_members`
  reads values with `parse::<i64>().unwrap_or(0)`.
  - With `| 99999999999999999999 | TooShort |`, Python's 004-25/28 DoD
    fails: sorted by value, TooShort is last. wp8 reads TooShort as 0, and
    its DoD passes.
  - The 45-digit case: Python keeps the value, wp8 reads 0.
  - Under the library this is `INT_LIMIT`.
- The members are kept in a `BTreeMap` (name order, not Python's insertion
  order). The order-sensitive outputs differ; the DoD verdicts agreed on
  the corpus's tie and swap cases.
- `py::stopped` models a Python exception as a FAIL under an earlier
  criterion. The script's real outcome is a traceback, so the harness
  reads the pair as unexcused.
- `pytoml` turns ints beyond `i64` into floats on 7 snippets: 2^63 becomes
  a float; a 42-digit int becomes `inf`; a 4 301-digit scalar becomes `inf`
  where Python keeps a string.
- `py::nums_before` blocks on `char::is_numeric`, which also matches `²`, a
  repertoire character.
  - `"²5 bytes"`: Python `['5']`, wp8 `[]`.
  - About 40 random corpus texts disagree the same way.
- `tree::py_eval` returns `None` for any value beyond `i128`, which the
  port reads as "not equal". Where the script's sum equals a stated big
  integer (`2**127`), the script passes and the port fails.
  - `2**-1` gives `None` where Python gives 0.5.
  - `4**0.5` and `1 + 2` are syntax errors in wp8. No script produces such
    text.
- `py::py_int` trims with Rust's `trim` and removes every `_`, so it
  accepts `"_1"` and `"1__0"`, which Python rejects. This was found by
  reading the code.
- Agreement: wp8's 8 Markdown helpers and its `tables()` agree with Python
  on the corpus.

## wp9 (004-01, 004-07, 004-14, 004-15, 006-01, 006-02)

| Private item | Library item |
|---|---|
| `tests/doc_004_01.rs` … `tests/doc_006_02.rs` | `tests/doc/c004_01.rs` … `tests/doc/c006_02.rs` (same content; one module of `tests/doc.rs`) |
| `c004_14::pytoml::parse(src, Flavor::Budget)` | `tomlline::parse(src, Variant::V004_14)` (`TomlReader::Line004_14`) |
| `Flavor::Capacity` (004-15) | `Variant::V004_15` (`TomlReader::Line004_15`) |
| `Parsed{messages, others}` | `tomlline::Doc{messages, others}`; a message is the Python dict itself (`"name"` starts as `Value::None`, `"fields"` is a `Value::List` of `Value::Dict`) |
| `Other::One` / `Many` | `Value::Dict` / `Value::List` in `others` |
| `V::Str(None)`, `V::Raw`, `V::Int(i64)` | `Value::None`, `Value::Str`, `Value::Int(i128)` / `BigInt` |
| `Bad::Value` / `Bad::Crash` | `tomlline::Error::caught_by(variant)` |
| `c004_15::setup`'s import of `c004_14::pytoml` | `tomlline` (the port-to-port import goes) |
| `c004_14::util::{Stop::{Crash, Exit}, R<T>, crash()}`, `Fails` | `report::{Stop::Crash, Stop::Exit{earlier: Earlier::Dropped}, Flow<T>, crash()}`, `Report` |
| `c004_14::util::{label_of, parse_num, first_int, num_match}` | `tables::{cell_label, cell_number_004_14, first_int_004_14, NUM_004_14}` |
| `c004_14::util::to_int` | `parse_int` after removing spaces |
| `c004_14::section::fence_mask` | `markdown::code_mask(.., FencePolicy::Backticks)` |
| `c004_14::section::{section_tables, caption_above}` | `parse_tables(.., FAMILY_004)` + `tables::caption_above` |
| `c004_15::num::{int_matches, guarded, floats, pcts, mbit_list, has_number}` | `text::rx` with the script's `INT_RE` / `FLOAT_RE` / `PCT_RE` / `MBIT_RE` copied as written (`captures_iter`), then `pynum::py_float` / `parse_int` |
| `c004_15::num::window` | `numwin::window` |
| `c004_15::num::{is_word, to_int}` | `pycompat::is_word`, `parse_int` |
| `util::sentences`, `num::sentences` | `text::rx(r"(?<=[.;])\s+").split(..)` or `sentences::split_raw(_, RAW_PERIOD_SEMI)` |
| `c006_01::text::split_row` | `tables::split_row_char_walk` / `RowSplit::CharWalk` |
| 006-01 fence scan | `markdown::code_mask(lines, FencePolicy::Matching)` |
| 006-02 `visible_lines` | `FencePolicy::SameChar` |
| 006-02 `find_section` | `Markdown::parse_with(text, FencePolicy::NoFences).find_by(HeadingMatch::Exact, HEADING)` |
| 006 tables | `tables_with(span, TableRules::T006_01 / T006_02)` (006-01: the header line is `Table::line`, the separator the line after it) |
| `heading_tokens`, `matching_heading` | `markdown::{stem_tokens, stem_match}`, `STOPWORDS_006_01` |
| `c006_01::text::nfkc` | `pycompat::nfkc` |
| `c006_02::rows::Num::{Int, Frac}` | `i128` + `pynum::Fraction` (the script's "is it a Fraction" test is a non-integer float) |
| `tests/wp9_support` | `testing::{report_of, check_denial, denial_tests!}` plus `doccheck-equiv --denials`. It used `repo_root` and `Sandbox`, which clippy bans in test files. |
| the `equiv/script.rs` edit | drop it: the library's label reader reads the grouped form, now with any upper-case group header |

Not taken:
- `Sec`, `row_kind`, `classify_case`, `tol_of`, `close`, `dec_value` and the
  number readers are script logic. They stay in the ports, built on
  `text::rx`.

**Disagreements:**
- `c004_15::num::has_number` implements `(?![\d])` with Rust's
  `is_numeric`, which also covers `²`.
  - On `21.1²`, Python's `(?<![\w.])21\.1(?![\d])` matches and wp9's does
    not. That is a real verdict difference for 004-15's checks at script
    lines 698 and 705.
- `c004_15::num::to_int` parses into `i64`, so a number above 2^63 − 1
  makes the port report a crash where Python carries on.
- `c004_14::util::to_int` uses `.parse::<i64>().unwrap_or(0)`: a 20-digit
  cell reads 0 where Python reads the number.
- `int_matches` accepts ASCII digits only, where Python's `\d` is any
  decimal digit. `٣ 12` gives `['12']` against Python's `['٣','12']`. This
  is outside the repertoire, so the guard keeps it out.
- `c006_01::text::nfkc` leaves `⁻` (U+207B, in the repertoire) unmapped,
  where Python maps it to `−` (U+2212): `"¹⁴⁵⁶⁹⁻₁₂₃"` gives Python
  `"14569−123"`, wp9 `"14569⁻123"`.
- `c004_14::pytoml`, both flavors, on 8 snippets:
  - `fields = 3` then `[[message.field]]`: Python crashes
    (`AttributeError`), wp9 parses.
  - `fields = [1]` then a field: Python gives `[1, {name: "f"}]`, wp9 `[1]`.
  - big ints and the int64 edge: Python gives ints, wp9 a caught failure.
  - 4 300 digits: Python an int, wp9 a string.
  - `a = [040]` before the first header: Python ignores the line, wp9
    fails.
  - `\x1f5\x1f`: Python `5`, wp9 the string (`trim` is not Python's
    `strip`).
  - a header surrounded by `\x1f`: Python reads `[t]`, wp9 drops the
    table.
- Agreement: on 15 075 corpus cases, `c004_15::num`'s values equal
  Python's.

## wp5 (004-06, 004-08, 004-09, 004-10)

| Private item | Library item |
|---|---|
| `tests/doc_004_06.rs`, `tests/doc_004_08.rs`, `tests/doc_004_09.rs`, `tests/doc_004_10.rs` | `tests/doc/c004_NN.rs` (same content; one module of `tests/doc.rs`) |
| `checks/wp5/pytoml::parse`, `Value`, `Dict`, `Value::Int(Big)` | `tomlsubset::parse` → `PyValue` / `Dict`, `PyValue::Int` / `BigInt` |
| `Fail` | as wp7 (the `Err`'s type prefix) |
| `big::Big` | `i128` plus `INT_LIMIT` (red by rule beyond `i128`) |
| `py::{py_int, py_int_base}` | `pycompat::parse_int`; a base other than 10 → `tomlsubset`'s int reader |
| `py::py_float` | `pynum::py_float` |
| `py::float_repr` | `pyvalue::repr_float` / `pynum::str_float` |
| `py::{py_word, py_digit, py_space, py_strip, digit_value}` | `pycompat::{is_word, is_nd, is_space, PyStr}`; a digit value is `parse_int` of the character |
| `py::{universal, read_universal}` | `pycompat::universal_newlines` with `Docs::read` |
| `py::{shadow, fold_i, s, si, find_i, search_i_not_after, letter_or_micro_i}` | `text::{rx, has, has_i}` with the script's look-behind written as is |
| `py::{char_slice, head}` | `pycompat::py_slice`, `text::head` |
| `py::big_to_f64` | `pynum::PyNum::as_f64` |
| `pyuni` tables | `repertoire` and `pycompat` |
| `md::{split_row, parse_tables, MdTable}` | `tables::split_row`, `family004::tables` |
| `md::heading_index` | `family004::heading_index` |
| `md::section_of` | `family004::section_span(.., None, SpanRule::AfterHeading004_16)`, then slice the lines |
| `md::strip_fences` | `units::strip_fences` |
| `md::blocks` | `blocks::blank_blocks` |
| `md::has_evidence` | `evidence::has_tag` |
| `md::norm_name` | `family004::norm_name(_, NameNorm::Alnum004_06)` |
| `md::{parse_int_token, bit_positions, is_field_table}` | the `family004` items of the same name |
| `md::width_to_bits` | `family004::width_to_bits(_, WidthRule::DigitGroups004_06)` |
| `md::is_placeholder` | `blocks::is_placeholder(Some(_))` |
| `md::untagged_blocks` | `evidence::untagged(_, &TagPolicy::EVERY_BLOCK_004_03)`, first line of `.text` |
| `members::{DEFAULT_MEMBERS, members, find_section}` | `rejection_members::{DEFAULT_13, tolerant_members(wire, TOLERANT_004_06)}` |
| `reasons::non_members` | `reasons::unnegated_non_members` per unit |
| `toml.rs` | the reader above plus `PyValue` |

Not taken:
- `py::str_repr`: it only feeds messages, and messages are not compared.
- `reasons_in` (`\bNAME\b` over the members) stays a one-liner in the check.
- The shadow text and the `İ`/`ı` fold: moot under the repertoire guard.

What wp5 used big integers for:
- offsets and widths read with `int()` from field tables (`c004_06/header.rs`,
  `mod.rs`, `toml_side.rs`);
- stated byte sizes;
- interval and cursor arithmetic;
- `off*8` byte checks.

All of these are `i128` now, and `INT_LIMIT` beyond it.

**Disagreements:**
- `wp5/pytoml` has no 4 300-digit limit. A 4 301-digit scalar (alone or in
  an array): Python keeps a string, wp5 an int.
- Agreement: wp5's 11 Markdown helpers, `members()` (all 45 rejection
  cases) and its `py_float`, `float_repr` and `py_int` agree with Python.

## Across packages

- **The same subset reader, three answers on big integers.** wp7 clamps to
  10^36, wp5 keeps the exact value, and wp8 gives a float. They agree on
  every other snippet.
- **The same script family, different crash models.** wp7 uses
  `"CRASH"`, wp8 a FAIL under an earlier criterion, wp9 `Stop`. All three
  are now `report::Stop`.
- **Between scripts (real variants, now named):**
  - fences: 006-01, 006-02 and the 004 family differ on 4 of 26 edge cases;
  - row splits: 006 versus the family differ on 5 of 22 edge cases;
  - `form_rows`: 004-17 versus 004-20;
  - sentence splits: 004-20 versus 004-12 differ on 56 inputs;
  - `norm_name`: 004-06 versus 004-20 differ on 146 inputs;
  - 004-15's line reader versus 004-14's.
- **Library fixes found while proving:**
  - `tomlsubset` now keeps a decimal of more than 4 300 digits as a string,
    as Python 3.10.7+ does;
  - `FencePolicy::Matching` no longer closes on a marker with text after
    it;
  - `EVERY_BLOCK_004_09` and `EVERY_BLOCK_004_13` were wrong (no port used
    them) and are replaced by `EVERY_BLOCK_004_03`;
  - `parse_int` now reads `i128::MIN`.
