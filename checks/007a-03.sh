#!/usr/bin/env bash
# checks/007a-03.sh — acceptance check for ticket 007a-03
# "No-std vector reader", with the review hardening of PR #34.
#
# The deliverables are in this repository (the design set's repository):
#   hadronyche-vectors/                    no_std data crate (C-192)
#   hadronyche-vectors/tests/reader.rs     acceptance tests
#   hadronyche-vectors/tests/reason_pin.rs Reason pinned to wire.toml (C2)
#   hadronyche-tools/fuzz-targets.txt      fuzz target list
#   hadronyche-tools/src/bin/vector-reader-fuzz.rs  the reader's fuzz harness
#   hadronyche-tools/fuzz-corpus/vector-reader/     its seed corpus
#
# Acceptance criteria and their mechanical encoding
# -------------------------------------------------
# AC1  The reader compiles with #![no_std] for a target without std. The crate
#      root declares #![no_std] unconditionally; `cargo build -p
#      hadronyche-vectors --no-default-features --target aarch64-unknown-none`
#      succeeds (the target CI installs, .github/workflows/ci.yml); and the
#      build has compiled DATA, not empty tables: the six example ids are in
#      a generated file under that target's build output (T1). A missing
#      target FAILs with the rustup command; it is never skipped.
#
# AC2  Enumerating the examples yields the six 007a-02 example fixtures with
#      id and spec_version unchanged; the conformance lookups never yield an
#      example (BINDING separation). Tests in hadronyche-vectors/tests/reader.rs,
#      run by `cargo test -p hadronyche-vectors`.
#
# AC3  For every .bin example the bytes returned equal the file byte for byte.
#
# Work 1  wire examples decode or reject with their declared reason; Reason is
#         pinned to doc/design/spec/wire.toml (tests/reason_pin.rs).
# Work 2  each kind's enumeration yields only its own kind.
# Work 3  the library embeds data at build time: src/ has no file-system or
#         environment access, and the crate is #![no_std] unconditionally (T2).
#
# AC4  No std in the default feature set: `cargo tree -e normal` lists no
#      crate other than hadronyche-vectors, with and without default features;
#      every [dependencies] entry sets default-features = false; no `std` path
#      outside comments in src/; an allocator only behind a documented
#      non-default feature.
#
# T3   No feature leaks: Cargo.toml declares no `examples` feature and no
#      self dev-dependency; the [features] table is absent or only `default =
#      []`; `cargo tree --workspace -e features,dev` shows no feature of
#      hadronyche-vectors beyond `default` (the leak CI's workspace test
#      command would carry).
#
# DoD  (a) the reader's parsing is listed as a fuzz target for 007a-15: the
#      list hadronyche-tools/fuzz-targets.txt names vector-reader and metrics,
#      each entry's harness exists and each corpus directory is tracked and
#      non-empty; vector-reader's harness builds. (b) every vector JSON under
#      the crate's data root and the example fixtures carries spec_version
#      (negative fixtures are excluded: they are invalid on purpose). (c) the
#      check's cargo target dir is under the checkout's ignored target/.
#
# Builds and tests are quiet (logs captured, tails printed on failure) and
# bounded by `timeout ${CHECK_TIMEOUT:-300}` seconds each. This script needs
# only bash, git, rustup/cargo, python3 (T1 and the example-id lookup run
# `python3 -I`), coreutils, awk, sed and grep.
#
# Exit status: 0 all criteria pass, 1 otherwise (one FAIL line per failure).

set -u
export LC_ALL=C

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${REPO_ROOT:-$(git -C "$HERE/.." rev-parse --show-toplevel 2>/dev/null || true)}"
[ -n "$REPO" ] || REPO="$(cd "$HERE/.." && pwd)"

VEC="$REPO/hadronyche-vectors"
TOOLS="$REPO/hadronyche-tools"
EXAMPLES="$VEC/tests/fixtures/vectors/examples"
NOSTD_TARGET="aarch64-unknown-none"

TMPROOT="${TMPDIR:-/tmp/opencode}"
mkdir -p "$TMPROOT" 2>/dev/null || TMPROOT=/tmp
TMP="$(mktemp -d "$TMPROOT/007a-03.XXXXXX")" || { echo "FAIL  cannot create temp dir"; exit 1; }
trap 'rm -rf "$TMP"' EXIT

# The cargo target dir lives under this checkout's own target/ (ignored by git,
# see .gitignore), so two checkouts never share build output and removing the
# checkout removes it. CARGO_TARGET_DIR is set for this script's cargo calls
# only, never exported for the caller.
CHECK_TARGET="${CHECK_CARGO_TARGET_DIR:-$REPO/target/check-007a-03}"
mkdir -p "$CHECK_TARGET" 2>/dev/null || true

LIMIT="${CHECK_TIMEOUT:-300}"
if command -v timeout >/dev/null 2>&1; then
  # usage: tb <logfile> <cmd...>
  tb() { local log="$1"; shift; timeout "$LIMIT" "$@" >"$log" 2>&1; }
else
  tb() { local log="$1"; shift; "$@" >"$log" 2>&1; }
fi

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
note() { printf '      %s\n' "$*"; }
logtail() { tail -n 20 "$1" 2>/dev/null | while IFS= read -r l; do note "$l"; done; }

echo "== 007a-03: no-std vector reader =="
echo "   repo:   $REPO"
echo "   target: $CHECK_TARGET"

# -------------------------------------------------------------------- AC 1
echo "-- AC1 reader compiles with no_std for a target without std"
if [ -f "$VEC/src/lib.rs" ] && grep -Eq '^#!\[no_std\][[:space:]]*$' "$VEC/src/lib.rs"; then
  pass "AC1 hadronyche-vectors/src/lib.rs declares #![no_std] unconditionally"
else
  bad "AC1 hadronyche-vectors/src/lib.rs does not declare #![no_std] unconditionally"
fi

if ! command -v rustup >/dev/null 2>&1; then
  bad "AC1 rustup not found in PATH; cannot confirm target $NOSTD_TARGET"
elif rustup target list --installed 2>/dev/null | grep -Fxq "$NOSTD_TARGET"; then
  if tb "$TMP/nostd.log" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo build -p hadronyche-vectors --no-default-features --target '$NOSTD_TARGET'"; then
    pass "AC1 cargo build -p hadronyche-vectors --no-default-features --target $NOSTD_TARGET succeeds"
  else
    rc=$?
    bad "AC1 no_std build failed for $NOSTD_TARGET (exit $rc)"
    logtail "$TMP/nostd.log"
  fi
else
  bad "AC1 target $NOSTD_TARGET is not installed; run: rustup target add $NOSTD_TARGET (CI installs it too)"
fi

# T1: the no_std build must have compiled data. The six example ids must be in
# a file cargo generated under the target's build output (the build script's
# OUT_DIR for that target), not only in the sources.
ids="$(python3 -I -c '
import glob, json, os, sys
root = sys.argv[1]
for f in sorted(glob.glob(os.path.join(root, "*", "example.json")) + glob.glob(os.path.join(root, "*", "*", "example.json"))):
    print(json.load(open(f))["id"])
' "$EXAMPLES" 2>/dev/null | sort -u)"
n_ids=0; n_found=0
gen_files="$(find "$CHECK_TARGET/$NOSTD_TARGET" -path '*/build/hadronyche-vectors*/out/*' -type f 2>/dev/null || true)"
while IFS= read -r id; do
  [ -n "$id" ] || continue
  n_ids=$((n_ids + 1))
  if [ -n "$gen_files" ] && grep -qF "\"$id\"" $gen_files 2>/dev/null; then
    n_found=$((n_found + 1))
  fi
done <<EOF
$ids
EOF
if [ "$n_ids" -eq 6 ] && [ "$n_found" -eq 6 ]; then
  pass "AC1 no_std build embeds data: all $n_found example ids found in generated build output for $NOSTD_TARGET"
else
  bad "AC1 no_std build does not embed the example data: $n_found of $n_ids example ids in generated output for $NOSTD_TARGET (expected 6 of 6)"
fi

# -------------------------------------------------------------- AC 2 and 3
echo "-- AC2/AC3 hadronyche-vectors tests: examples, bytes, outcomes, reason pin"
TESTLOG="$TMP/test.log"
test_rc=0
if tb "$TESTLOG" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo test --no-fail-fast -p hadronyche-vectors"; then
  pass "AC2 cargo test -p hadronyche-vectors passes"
else
  test_rc=$?
  bad "AC2 cargo test -p hadronyche-vectors did not pass (exit $test_rc)"
  if grep -Eq '^error(\[E[0-9]+\])?:' "$TESTLOG"; then
    note "the test target does not compile; first errors:"
    grep -m3 -A4 -E '^error' "$TESTLOG" | while IFS= read -r l; do note "$l"; done
  else
    logtail "$TESTLOG"
  fi
fi

# One `test <name> ... ok` line per test. A test that did not run (for example
# a compile error) is reported as not run.
check_test() {
  local label="$1" name="$2" desc="$3"
  if grep -Eq "^test ${name} \.\.\. ok\$" "$TESTLOG" 2>/dev/null; then
    pass "$label $desc (test $name)"
  elif [ "$test_rc" -ne 0 ]; then
    bad "$label $desc (test $name not run or failed)"
  else
    bad "$label $desc (test $name not found in the test run)"
  fi
}
check_test "AC2" ac2_examples_enumeration_yields_each_example_with_id_and_spec_version_unchanged \
  "examples() yields exactly the six examples, id and spec_version unchanged"
check_test "AC3" ac3_bytes_of_every_bin_example_equal_the_file_byte_for_byte \
  "bytes of every .bin example equal the file byte for byte"
check_test "Work 1" work1_wire_examples_decode_or_reject_with_their_declared_reason \
  "wire valid decodes; wire invalid rejects with its declared reason"
check_test "Work 1" reason_is_pinned_to_wire_toml \
  "Reason names and values equal wire.toml [[reject]] in order; from_name round-trips"
check_test "Work 2" work2_each_kind_enumerates_only_its_own_vectors \
  "each kind enumerates only its own vectors (conformance and examples)"
check_test "AC2" examples_are_not_reachable_through_the_conformance_lookups \
  "no example id is reachable through all(), of_kind(), find() or with_bytes()"
check_test "Work 3" work3_every_embedded_vector_carries_a_nonempty_id_and_spec_version \
  "every embedded vector and example carries id and spec_version"

# T2 (Work 3): the library reads nothing at run time; data is embedded at build
# time. Comment lines are ignored.
echo "-- Work3 the library does no run-time file or environment access"
rt_hits="$(grep -rnE '(std::fs|fs::|read_dir|\bFile\b|std::io|std::env|env::|OsStr|read_to_string)' \
  "$VEC/src" 2>/dev/null | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' || true)"
if [ -z "$rt_hits" ]; then
  pass "Work3 hadronyche-vectors/src has no file-system or environment access (data is embedded at build time)"
else
  bad "Work3 run-time file or environment access in hadronyche-vectors/src:"
  printf '%s\n' "$rt_hits" | head -n 5 | while IFS= read -r l; do note "$l"; done
fi

# -------------------------------------------------------------------- AC 4
echo "-- AC4 no std in the default feature set"
for flags in "--no-default-features" ""; do
  label="${flags:-default features}"
  if tb "$TMP/tree.log" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo tree -p hadronyche-vectors -e normal $flags --prefix none"; then
    extra="$(awk 'NF && $1 != "hadronyche-vectors" {print $1}' "$TMP/tree.log" | sort -u | tr '\n' ' ')"
    if [ -z "$extra" ]; then
      pass "AC4 normal dependencies with $label: none beyond hadronyche-vectors"
    else
      bad "AC4 normal dependencies with $label pull in: $extra"
    fi
  else
    bad "AC4 cargo tree -e normal $label failed"
    logtail "$TMP/tree.log"
  fi
done

if awk '
  /^\[dependencies\]/ {indeps=1; next}
  /^\[/ {indeps=0}
  indeps && NF && $0 !~ /^[[:space:]]*#/ && $0 !~ /default-features[[:space:]]*=[[:space:]]*false/ {bad=1}
  END {exit bad}' "$VEC/Cargo.toml"; then
  pass "AC4 every [dependencies] entry sets default-features = false (or there are none)"
else
  bad "AC4 a [dependencies] entry in hadronyche-vectors/Cargo.toml keeps default features"
fi

std_hits="$(grep -rnE '(^|[^A-Za-z0-9_:])(std::|extern crate std|use std\b)' "$VEC/src" 2>/dev/null \
  | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' || true)"
if [ -z "$std_hits" ]; then
  pass "AC4 no std path or extern crate std in hadronyche-vectors/src outside comments"
else
  bad "AC4 std used in hadronyche-vectors/src (move test-only code to tests/):"
  printf '%s\n' "$std_hits" | head -n 5 | while IFS= read -r l; do note "$l"; done
fi

alloc_hits="$(grep -rnE '^[[:space:]]*extern crate alloc' "$VEC/src" 2>/dev/null || true)"
if [ -z "$alloc_hits" ]; then
  pass "AC4 no allocator used in src (none needed)"
else
  alloc_ok=1
  while IFS=: read -r file line _; do
    start=$(( line > 2 ? line - 2 : 1 ))
    if ! sed -n "${start},$((line - 1))p" "$file" | grep -Eq 'cfg\(feature *= *"[a-z_-]+"\)'; then
      alloc_ok=0
    fi
  done < <(printf '%s\n' "$alloc_hits")
  default_line="$(grep -E '^default[[:space:]]*=' "$VEC/Cargo.toml" || true)"
  if [ "$alloc_ok" -eq 1 ] && ! printf '%s' "$default_line" | grep -q 'alloc' \
     && grep -Eiq '^[[:space:]]*#.*allocator' "$VEC/Cargo.toml"; then
    pass "AC4 allocator only behind a named feature, not a default one, documented in Cargo.toml"
  else
    bad "AC4 extern crate alloc must sit behind cfg(feature = \"...\") that is not in default, documented in Cargo.toml"
  fi
fi

# -------------------------------------------------------------------- T3
echo "-- T3 no feature leaks into other workspace members"
features_section="$(awk '/^\[features\]/ {on=1; next} /^\[/ {on=0} on && NF && $0 !~ /^[[:space:]]*#/' "$VEC/Cargo.toml")"
if grep -Eq '^[[:space:]]*examples[[:space:]]*=' "$VEC/Cargo.toml"; then
  bad "T3 hadronyche-vectors/Cargo.toml declares an examples feature (examples are always embedded)"
else
  pass "T3 Cargo.toml declares no examples feature"
fi
if grep -Eq '^[[:space:]]*hadronyche-vectors[[:space:]]*=' "$VEC/Cargo.toml"; then
  bad "T3 hadronyche-vectors/Cargo.toml has a dependency on itself (it leaks features under the workspace test command)"
else
  pass "T3 Cargo.toml has no self dependency"
fi
if [ -z "$features_section" ] || [ "$features_section" = 'default = []' ]; then
  pass "T3 the [features] table is absent or only default = []"
else
  bad "T3 the [features] table declares features that change the build: $(printf '%s' "$features_section" | tr '\n' ' ')"
fi
if tb "$TMP/features.log" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo tree --workspace -e features,dev --prefix none"; then
  leaked="$(grep -E 'hadronyche-vectors feature "' "$TMP/features.log" | grep -v 'feature "default"' | sort -u || true)"
  if [ -z "$leaked" ]; then
    pass "T3 cargo tree --workspace -e features,dev: no feature of hadronyche-vectors beyond default"
  else
    bad "T3 a feature of hadronyche-vectors is enabled in the workspace graph:"
    printf '%s\n' "$leaked" | while IFS= read -r l; do note "$l"; done
  fi
else
  bad "T3 cargo tree --workspace -e features,dev failed"
  logtail "$TMP/features.log"
fi

# --------------------------------------------------------------------- DoD
echo "-- DoD the reader's parsing is a fuzz target for 007a-15"
LIST="$TOOLS/fuzz-targets.txt"
entries="$(grep -vE '^[[:space:]]*(#|$)' "$LIST" 2>/dev/null || true)"
if printf '%s\n' "$entries" | awk '$1 == "vector-reader" {f=1} END {exit !f}'; then
  pass "DoD fuzz-targets.txt lists vector-reader"
else
  bad "DoD fuzz-targets.txt has no vector-reader line (the reader's parsing is a fuzz target for 007a-15)"
fi
if printf '%s\n' "$entries" | awk '$1 == "metrics" {f=1} END {exit !f}'; then
  pass "DoD fuzz-targets.txt lists the existing metrics target"
else
  bad "DoD fuzz-targets.txt has no metrics line (the list must be complete)"
fi
while IFS= read -r line; do
  [ -n "$line" ] || continue
  name="$(printf '%s' "$line" | awk '{print $1}')"
  harness_rel="$(printf '%s' "$line" | awk '{print $2}')"
  corpus_rel="$(printf '%s' "$line" | awk '{print $3}')"
  if [ -f "$TOOLS/$harness_rel" ]; then
    pass "DoD $name: harness $harness_rel exists"
  else
    bad "DoD $name: harness $harness_rel named in fuzz-targets.txt does not exist"
  fi
  seeds="$(git -C "$REPO" ls-files -- "hadronyche-tools/$corpus_rel" 2>/dev/null | sed '/^$/d' | wc -l)"
  if [ "$seeds" -gt 0 ]; then
    pass "DoD $name: seed corpus hadronyche-tools/$corpus_rel is tracked ($seeds file(s))"
  else
    bad "DoD $name: seed corpus hadronyche-tools/$corpus_rel is missing or untracked"
  fi
done < <(printf '%s\n' "$entries")

if tb "$TMP/fuzzbin.log" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo build -p hadronyche-tools --bin vector-reader-fuzz"; then
  pass "DoD vector-reader harness builds (cargo build -p hadronyche-tools --bin vector-reader-fuzz)"
else
  bad "DoD vector-reader harness does not build (cargo build -p hadronyche-tools --bin vector-reader-fuzz)"
  logtail "$TMP/fuzzbin.log"
fi

nospec=0; total=0
while IFS= read -r f; do
  [ -n "$f" ] || continue
  total=$((total + 1))
  if ! grep -q '"spec_version"' "$f"; then
    bad "DoD $f has no spec_version"
    nospec=$((nospec + 1))
  fi
done < <(find "$VEC/vectors" "$EXAMPLES" -name '*.json' -not -path '*/negative/*' 2>/dev/null)
if [ "$nospec" -eq 0 ]; then
  pass "DoD every vector JSON file carries spec_version ($total file(s) checked)"
fi

if git -C "$REPO" check-ignore -q "$REPO/target/check-007a-03/.probe" 2>/dev/null; then
  pass "DoD the check's cargo target dir is under the git-ignored target/"
else
  bad "DoD target/ is not git-ignored; the check's cargo target dir would be committed"
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 007a-03 PASSED =="
  exit 0
fi
echo "== 007a-03 FAILED =="
exit 1
