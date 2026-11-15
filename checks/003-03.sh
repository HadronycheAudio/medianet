#!/usr/bin/env bash
# checks/003-03.sh — acceptance check for ticket 003-03
# "Process-owned metrics writer".
#
# The deliverable lives outside this repository: metrics written by the frozen
# proof of concept at /home/dorian/workspace/hadropoc (the "POC").  The fix
# itself is the CODER's work under the epic-003 exception.  This script only
# READS the POC source and git history; it runs the POC (which writes build
# output below the POC's `target/`, excluded by its .gitignore) and points
# `--metrics` at a temporary directory below /tmp/opencode only.  It never
# writes into the POC tree.  Override the location with HADROPOC=/path.
#
# Acceptance criteria and their mechanical encoding
# -------------------------------------------------
# AC1  No `File::create` on the metrics path inside module code; the only
#      creation of a metrics file is in the process-level registry.
#      Encoding (static, source text at RED time was checked to match):
#        (a) `grep -c 'File::create' src/module.rs` == 0.  This is the
#            literal "grep shows no File::create ... inside module code".
#        (b) exactly ONE metrics-file creation site exists across src/*.rs,
#            outside module.rs.  A "creation site" is a source line matching
#            `File::create` or `OpenOptions` in a file whose text mentions
#            `metrics`, excluding the pre-existing `--csv` writer in main.rs
#            (`File::create` applied to `c.csv`, out of scope for this ticket
#            and not to be touched).  Accepting `OpenOptions` too keeps the
#            check honest if the registry opens the file with
#            `.create(true)` rather than `File::create`.
#            The "outside module.rs" clause encodes the ticket's split
#            between process-level registry code and module housekeeping
#            code: with both in one file the static check could not tell
#            them apart.  If a future implementation has a good reason to
#            put the registry in module.rs, the reviewer should amend this
#            clause explicitly — do not silently weaken it.
#        (c) no metrics creation site remains in module.rs (implied by (a)
#            for `File::create`; checked explicitly for `OpenOptions`).
#
# AC2  A 25 s loopback run with the ticket's exact flags, `--no-tui` (verified
#      headless), `--metrics <tmp>/metrics.csv` produces a file in which:
#        (a) every line has exactly as many comma-separated fields as the
#            file's own first line (the header) — 100 % of lines parse
#            against the header;
#        (b) the header line (the first line) occurs exactly once in the
#            whole file.
#      The POC's row values never contain a comma (names, integers, floats),
#      so comma counting is a faithful field count.
#
# AC3  One row per module per second: 4 x 25 = 100 data rows within +/-2 in
#      total, and, because the header has a module-identifying column, 25 +/-2
#      rows per module.  The header emitted at `poc-untouched` is
#          unix_ms,net_ns,offset_ns,name,holder,...
#      Field 4 is `name`; module.rs writes `net.me.name` there, and loopback
#      names its four modules sender, synth1, synth2 and mixer (main.rs:498-
#      504).  Encoding:
#        (a) data rows (all non-header lines) total in [98, 102];
#        (b) exactly 4 distinct values in the `name` column, each in [23, 27].
#      Counts are derived from the header at run time, not hard-coded, so a
#      column reordering would still be measured on `name`; a header WITHOUT
#      a `name` column would skip (b) — but AC4 forbids changing the header,
#      so in practice (b) always applies.
#
# AC4  The header line is byte-identical to the one the POC emits at
#      `poc-untouched`.  Encoding: extract the `METRICS` string literal from
#      `git show poc-untouched:src/module.rs` (the `pub const METRICS`
#      declaration in the metrics section, ~line 1292 at the tag), append the
#      newline the writer emits, and `cmp` it byte-for-byte against line 1 of
#      the run's file.  `LC_ALL=C` throughout.
#
# AC5  The two existing transport tests still pass (epic 003-AC6):
#      `cargo test transport` in the POC, unprivileged, exit 0, exactly
#      `running 2 tests`, two `test transport::tests::<name> ... ok` lines,
#      and `test result: ok. 2 passed; 0 failed;`.
#
# AC6  "The diff of this ticket touches only the metrics writer code."
#      Encoding (approximate by design; exact judgement is at review):
#        (a) the commits whose first line references 003-03 (token rule)
#            change at least one path — otherwise the criterion would pass
#            vacuously on an unfixed tree;
#        (b) at most 3 paths differ.  The ticket's own edits are module.rs
#            (housekeeping) plus the process-level registry.  If the registry
#            is a new file, `lib.rs` must also gain its `pub mod` line, so
#            the canonical shape is 3 paths: src/module.rs, src/<registry>.rs,
#            src/lib.rs.  Limiting to 2 (as a stricter reading might) would
#            reject that shape; limiting to 3 keeps the check honest.  Placing
#            the registry in an existing non-module file (for example
#            src/stats.rs) yields 2 and also passes.
#        (c) none of the differing paths is one of the wire/servo/analyzer/
#            audio/entry-point files the epic must not touch: transport.rs,
#            clock.rs, audio.rs, proto.rs, net.rs, rt.rs, music.rs, main.rs.
#            (main.rs is included because the ticket's work list does not
#            require changing it: modules still receive the metrics path and
#            ask the registry for a sender.)
#        (d) every differing path is under src/ (the metrics writer code).
#      The scope is THIS TICKET'S commits only: the union of
#      `git show --name-only` over the commits after poc-untouched whose first
#      line references 003-03.  Later sanctioned epic-003 commits are not
#      measured here (a diff from poc-untouched would make this check age).
#
# Exit status: 0 all criteria pass, 1 otherwise (one FAIL line per point).
#
# Cost: `cargo build --release` (cached after the first run), a 25 s loopback
# run and `cargo test transport` — roughly 40 s once compiled.

set -u
export LC_ALL=C

POC="${HADROPOC:-/home/dorian/workspace/hadropoc}"
TMPROOT="${TMPDIR:-/tmp/opencode}"
mkdir -p "$TMPROOT" 2>/dev/null || TMPROOT=/tmp
TMP="$(mktemp -d "$TMPROOT/003-03.XXXXXX")" || { echo "FAIL  cannot create temp dir"; exit 1; }
trap 'rm -rf "$TMP"' EXIT

METRICS="$TMP/metrics/metrics.csv"
mkdir -p "$TMP/metrics"

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
note() { printf '      %s\n' "$*"; }

gitpoc() { git -C "$POC" "$@"; }

echo "== 003-03: process-owned metrics writer =="
echo "   POC:      $POC"
echo "   metrics:  $METRICS"

# ---------------------------------------------------------------- precheck
if [ ! -d "$POC" ]; then
  bad "precondition: $POC does not exist"
  echo "== 003-03 FAILED =="
  exit 1
fi
if ! gitpoc rev-parse --git-dir >/dev/null 2>&1; then
  bad "precondition: $POC is not a git repository"
  echo "== 003-03 FAILED =="
  exit 1
fi
if ! gitpoc rev-parse --verify -q 'poc-untouched^{commit}' >/dev/null 2>&1; then
  bad "precondition: poc-untouched does not resolve to a commit"
  echo "== 003-03 FAILED =="
  exit 1
fi
if [ ! -f "$POC/src/module.rs" ]; then
  bad "precondition: $POC/src/module.rs not found"
  echo "== 003-03 FAILED =="
  exit 1
fi

# -------------------------------------------------------------------- AC 1
echo "-- AC1 a single process-level creator of the metrics file"

module_create="$(grep -c 'File::create' "$POC/src/module.rs" 2>/dev/null || true)"
module_create="${module_create:-0}"
if [ "$module_create" -eq 0 ]; then
  pass "AC1 no File::create in src/module.rs"
else
  bad "AC1 src/module.rs still calls File::create ($module_create occurrence(s)):"
  grep -n 'File::create' "$POC/src/module.rs" | head -n 5 | while IFS= read -r l; do note "$l"; done
fi

# Metrics-file creation sites: File::create / OpenOptions in a file that talks
# about metrics, ignoring the pre-existing --csv writer in main.rs.
: > "$TMP/sites"       # candidate sites: "<basename>\t<path>:<line>:<text>"
while IFS= read -r hit; do
  [ -n "$hit" ] || continue
  file="${hit%%:*}"
  rest="${hit#*:}"
  lineno="${rest%%:*}"
  text="${rest#*:}"
  base="${file##*/}"
  case "$text" in
    *csv*) continue ;;      # --csv writer, out of scope for this ticket
  esac
  grep -q 'metrics' "$file" 2>/dev/null || continue
  printf '%s\t%s:%s:%s\n' "$base" "$file" "$lineno" "$text" >> "$TMP/sites"
done < <(grep -rnE 'File::create|OpenOptions' "$POC/src" --include='*.rs' 2>/dev/null || true)

sites_outside="$(awk -F'\t' '$1 != "module.rs"' "$TMP/sites" | awk 'END { print NR }')"
sites_module="$(awk -F'\t' '$1 == "module.rs"' "$TMP/sites" | awk 'END { print NR }')"
total_sites="$(awk 'END { print NR }' "$TMP/sites")"

if [ "$sites_outside" -eq 1 ] && [ "$sites_module" -eq 0 ]; then
  pass "AC1 exactly one metrics-file creation site, in process-level code outside module.rs"
else
  bad "AC1 expected exactly 1 metrics-file creation site outside module.rs and 0 inside; got outside=$sites_outside inside=$sites_module (total=$total_sites)"
  if [ "$total_sites" -gt 0 ]; then
    note "candidate creation sites:"
    cut -f2 "$TMP/sites" | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi
fi

# ------------------------------------------------------------ AC 2/3/4 run
echo "-- AC2/AC3/AC4 25 s loopback run"
BIN="$POC/target/release/hadropoc"
build_ok=1
if ! command -v cargo >/dev/null 2>&1; then
  bad "AC2/AC3/AC4 cargo not found in PATH"
  build_ok=0
else
  echo "   building: (cd $POC && cargo build --release)"
  ( cd "$POC" && CARGO_TERM_COLOR=never cargo build --release ) > "$TMP/build.log" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ]; then
    bad "AC2/AC3/AC4 cargo build --release failed (exit $rc)"
    tail -n 15 "$TMP/build.log" | while IFS= read -r l; do note "$l"; done
    build_ok=0
  elif [ ! -x "$BIN" ]; then
    bad "AC2/AC3/AC4 built binary not found at $BIN"
    build_ok=0
  fi
fi

run_ok=0
if [ "$build_ok" -eq 1 ]; then
  rm -f "$METRICS"
  echo "   running: $BIN loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --metrics $METRICS --no-tui"
  if command -v timeout >/dev/null 2>&1; then
    timeout 180 "$BIN" loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 \
      --metrics "$METRICS" --no-tui > "$TMP/run.log" 2>&1
  else
    "$BIN" loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 \
      --metrics "$METRICS" --no-tui > "$TMP/run.log" 2>&1
  fi
  rc=$?
  if [ "$rc" -ne 0 ]; then
    bad "AC2/AC3/AC4 loopback run failed (exit $rc)"
    tail -n 15 "$TMP/run.log" | while IFS= read -r l; do note "$l"; done
  elif [ ! -s "$METRICS" ]; then
    bad "AC2/AC3/AC4 loopback run left no metrics file at $METRICS"
  else
    run_ok=1
  fi
fi

header=''
if [ "$run_ok" -eq 1 ]; then
  header="$(head -n 1 "$METRICS")"
fi

# -------------------------------------------------------------------- AC 4
# (checked before AC2/AC3 so the reported header is known to be the right one)
echo "-- AC4 header byte-identical to poc-untouched"
expected=''
gitpoc show poc-untouched:src/module.rs > "$TMP/module-at-tag.rs" 2>/dev/null || true
if [ -s "$TMP/module-at-tag.rs" ]; then
  expected="$(sed -n 's/^pub const METRICS: &str = "\(.*\)";$/\1/p' "$TMP/module-at-tag.rs" | head -n 1)"
fi
if [ -z "$expected" ]; then
  bad "AC4 could not extract the METRICS header literal from poc-untouched:src/module.rs"
elif [ "$run_ok" -ne 1 ]; then
  bad "AC4 not evaluated: no metrics file from the run"
else
  printf '%s\n' "$expected" > "$TMP/expected-header"
  head -n 1 "$METRICS" > "$TMP/actual-header"
  if cmp -s "$TMP/expected-header" "$TMP/actual-header"; then
    pass "AC4 header line is byte-identical to poc-untouched ($(printf '%s' "$expected" | awk -F, '{print NF}') fields)"
  else
    bad "AC4 header line differs from poc-untouched"
    note "expected: $(head -c 200 "$TMP/expected-header")"
    note "actual:   $(head -c 200 "$TMP/actual-header")"
  fi
fi

# -------------------------------------------------------------------- AC 2
echo "-- AC2 every line parses against the header; header exactly once"
if [ "$run_ok" -ne 1 ]; then
  bad "AC2 not evaluated: no metrics file from the run"
else
  hdr_fields="$(awk -F, 'NR==1 { print NF; exit }' "$METRICS")"
  if [ -z "$hdr_fields" ] || [ "$hdr_fields" -lt 2 ] 2>/dev/null; then
    bad "AC2 header line has no usable field count ('${hdr_fields:-<empty>}')"
  else
    total_lines="$(awk 'END { print NR }' "$METRICS")"
    bad_lines="$(awk -F, -v n="$hdr_fields" 'NF != n { print NR ":" NF }' "$METRICS")"
    bad_count="$(printf '%s\n' "$bad_lines" | sed '/^$/d' | wc -l)"
    if [ "$bad_count" -eq 0 ]; then
      pass "AC2 all $total_lines lines have $hdr_fields fields, as the header"
    else
      bad "AC2 $bad_count of $total_lines lines do not parse against the header ($hdr_fields fields):"
      printf '%s\n' "$bad_lines" | head -n 5 | while IFS= read -r l; do
        note "line ${l%%:*}: ${l#*:} field(s)"
      done
    fi
  fi

  header_count="$(grep -Fxc -- "$header" "$METRICS" || true)"
  header_count="${header_count:-0}"
  if [ "$header_count" -eq 1 ]; then
    pass "AC2 header line appears exactly once"
  else
    bad "AC2 header line appears $header_count time(s), expected exactly 1"
  fi
fi

# -------------------------------------------------------------------- AC 3
echo "-- AC3 one row per module per second (expected 4 x 25)"
if [ "$run_ok" -ne 1 ]; then
  bad "AC3 not evaluated: no metrics file from the run"
else
  # Data rows: every line that is not the header line.
  grep -Fxv -- "$header" "$METRICS" > "$TMP/data-rows" 2>/dev/null || true
  rows="$(awk 'END { print NR }' "$TMP/data-rows")"
  if [ "$rows" -ge 98 ] && [ "$rows" -le 102 ]; then
    pass "AC3 total data rows = $rows (100 +/- 2)"
  else
    bad "AC3 total data rows = $rows, expected 100 +/- 2 (98..102)"
  fi

  name_col="$(awk -F, 'NR==1 { for (i = 1; i <= NF; i++) if ($i == "name") { print i; exit } }' "$METRICS")"
  if [ -z "$name_col" ]; then
    note "header has no 'name' column: per-module split not applicable"
  else
    awk -F, -v c="$name_col" '{ v = $c; if (v == "") v = "<empty>"; print v }' "$TMP/data-rows" | sort | uniq -c \
      | awk '{ printf "%s %s\n", $1, $2 }' > "$TMP/per-module"
    module_kinds="$(wc -l < "$TMP/per-module")"
    bad_modules="$(awk '$1 < 23 || $1 > 27' "$TMP/per-module")"
    bad_module_count="$(printf '%s\n' "$bad_modules" | sed '/^$/d' | wc -l)"

    if [ "$module_kinds" -eq 4 ] && [ "$bad_module_count" -eq 0 ]; then
      pass "AC3 4 modules, each with 25 +/- 2 rows:"
      while IFS= read -r l; do note "$l row(s)"; done < "$TMP/per-module"
    else
      bad "AC3 expected 4 modules with 23..27 rows each; got $module_kinds module(s), $bad_module_count outside the range:"
      head -n 8 "$TMP/per-module" | while IFS= read -r l; do note "$l row(s)"; done
    fi
  fi
fi

# -------------------------------------------------------------------- AC 5
echo "-- AC5 two transport tests pass"
if ! command -v cargo >/dev/null 2>&1; then
  bad "AC5 cargo not found in PATH"
else
  echo "   running: (cd $POC && cargo test transport)"
  ( cd "$POC" && CARGO_TERM_COLOR=never cargo test transport ) > "$TMP/cargo-test.log" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ]; then
    bad "AC5 cargo test transport failed (exit $rc)"
    tail -n 20 "$TMP/cargo-test.log" | while IFS= read -r l; do note "$l"; done
  else
    if grep -Fxq 'running 2 tests' "$TMP/cargo-test.log"; then
      pass "AC5 cargo reports exactly 2 tests in the transport filter"
    else
      bad "AC5 expected 'running 2 tests', got: $(grep -E '^running [0-9]+ tests?$' "$TMP/cargo-test.log" | head -n 1)"
    fi

    names="$(grep -E '^test transport::tests::[A-Za-z0-9_]+ \.\.\. ok$' "$TMP/cargo-test.log" || true)"
    n="$(printf '%s\n' "$names" | sed '/^$/d' | wc -l)"
    if [ "$n" -eq 2 ]; then
      pass "AC5 both transport tests pass:"
      printf '%s\n' "$names" | while IFS= read -r l; do note "$l"; done
    else
      bad "AC5 expected 2 passing transport tests, found $n"
      grep -E '^test transport::' "$TMP/cargo-test.log" | head -n 10 | while IFS= read -r l; do note "$l"; done
    fi

    if grep -Eq '^test result: ok\. 2 passed; 0 failed;' "$TMP/cargo-test.log"; then
      pass "AC5 test result: ok. 2 passed; 0 failed"
    else
      bad "AC5 no 'test result: ok. 2 passed; 0 failed' line"
      grep -E '^test result:' "$TMP/cargo-test.log" | head -n 5 | while IFS= read -r l; do note "$l"; done
    fi
  fi
fi

# -------------------------------------------------------------------- AC 6
echo "-- AC6 diff of this ticket's commits limited to the metrics writer"
# The bound applies to THIS TICKET'S commits: those whose first line references
# 003-03 (token rule: `003-03:` prefix or `type(003-03):` scope).  A diff from
# poc-untouched would also pick up later sanctioned epic-003 commits (003-04,
# 003-06 touching src/main.rs), which their own checks judge.  The untracked
# files of a not-yet-committed write are not part of a ticket's commits and
# are covered by the tree-cleanliness assertion instead.
gitpoc log --reverse --format='%H%x09%s' poc-untouched..HEAD 2>/dev/null \
  | awk -F'\t' '$2 ~ /(^|[^0-9A-Za-z])003-03($|[^0-9])/ { print $1 }' > "$TMP/t03-commits"
n_commits="$(sed '/^$/d' "$TMP/t03-commits" | wc -l | tr -d ' ')"
: > "$TMP/changed"
while IFS= read -r c; do
  [ -n "$c" ] || continue
  gitpoc show --name-only --format= "$c" >> "$TMP/changed" 2>/dev/null || true
done < "$TMP/t03-commits"
sed '/^$/d' "$TMP/changed" | sort -u > "$TMP/changed-u"
n_changed="$(wc -l < "$TMP/changed-u" | tr -d ' ')"

if [ "$n_commits" -eq 0 ]; then
  bad "AC6 no commit after poc-untouched references 003-03: the metrics-writer fix is not committed"
elif [ "$n_changed" -eq 0 ]; then
  bad "AC6 the $n_commits commit(s) referencing 003-03 change no path: the metrics-writer fix is absent"
else
  note "ticket commits: $(tr '\n' ' ' < "$TMP/t03-commits" | cut -c1-80)"
  if [ "$n_changed" -le 3 ]; then
    pass "AC6 $n_changed path(s) changed by the 003-03 commit(s) (at most 3 allowed)"
  else
    bad "AC6 $n_changed paths changed by the 003-03 commit(s), expected at most 3:"
    head -n 8 "$TMP/changed-u" | while IFS= read -r p; do note "$p"; done
  fi

  hits="$(while IFS= read -r p; do            b="${p##*/}"
            case "$b" in
              transport.rs|clock.rs|audio.rs|proto.rs|net.rs|rt.rs|music.rs|main.rs) printf '%s\n' "$p" ;;
            esac
          done < "$TMP/changed-u")"
  if [ -z "$hits" ]; then
    pass "AC6 no wire/servo/analyzer/audio/entry-point file was touched by 003-03"
  else
    bad "AC6 files outside the metrics writer were touched by 003-03:"
    printf '%s\n' "$hits" | head -n 8 | while IFS= read -r p; do note "$p"; done
  fi

  outside="$(grep -Ev '^src/' "$TMP/changed-u" || true)"
  if [ -z "$outside" ]; then
    pass "AC6 every path changed by 003-03 is under src/"
  else
    bad "AC6 paths changed by 003-03 outside src/:"
    printf '%s\n' "$outside" | head -n 8 | while IFS= read -r p; do note "$p"; done
  fi
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 003-03 PASSED =="
  exit 0
fi
echo "== 003-03 FAILED =="
exit 1
