#!/usr/bin/env bash
# checks/007a-03.sh — acceptance check for ticket 007a-03
# "No-std vector reader".
#
# The deliverables are in this repository (the design set's repository):
#   hadronyche-vectors/                  no_std data crate (C-192), src/lib.rs
#   hadronyche-vectors/tests/reader.rs   acceptance tests (PROPOSED API)
#   hadronyche-tools/fuzz-targets.txt    fuzz target list read by 007a-15
#   hadronyche-tools/src/bin/vector-reader-fuzz.rs  the reader's fuzz harness
#   hadronyche-tools/fuzz-corpus/vector-reader/     its seed corpus
#
# Acceptance criteria and their mechanical encoding
# -------------------------------------------------
# AC1  The reader compiles with #![no_std] for a target without std: the
#      crate root declares #![no_std] (or #![cfg_attr(not(test), no_std)];
#      the ticket's own words say #![no_std], both forms are accepted), and
#      `cargo build -p hadronyche-vectors --no-default-features --target
#      aarch64-unknown-none` succeeds. That is the target CI installs
#      (.github/workflows/ci.yml). If it is not installed locally this check
#      FAILs and names the rustup command; it is never skipped.
#
# AC2  Enumerating the embedded vectors yields the six example fixtures of
#      007a-02 with id and spec_version unchanged. Checked by the tests named
#      below in hadronyche-vectors/tests/reader.rs, run by
#      `cargo test -p hadronyche-vectors`.
#
# AC3  For every .bin fixture the bytes returned equal the file byte for
#      byte (test ac3_*), same target.
#
# AC4  No std in the default feature set: `cargo tree -e normal` lists no
#      crate other than hadronyche-vectors, with and without default
#      features; every [dependencies] entry sets default-features = false;
#      no `std` path or `extern crate std` appears in src/ outside comments;
#      an allocator (`extern crate alloc`) is only behind a cfg(feature)
#      guard whose feature is not a default feature and is documented in
#      Cargo.toml.
#
# DoD  (a) the reader's own parsing is listed as a fuzz target for 007a-15:
#      hadronyche-tools/fuzz-targets.txt has a `vector-reader` line whose
#      harness exists, builds, and whose seed corpus is tracked and non-empty;
#      (b) every vector JSON file under the crate's data root and the example
#      fixtures carries spec_version (negative fixtures are excluded: they are
#      invalid on purpose).
#
# Builds and tests are quiet (logs captured, tails printed on failure) and
# bounded by `timeout ${CHECK_TIMEOUT:-300}` seconds each. A shared
# CARGO_TARGET_DIR under /tmp/opencode keeps rebuilds cheap. This script needs
# only bash, git, rustup/cargo, coreutils, awk and grep.
#
# Exit status: 0 all criteria pass, 1 otherwise (one FAIL line per failure).

set -u
export LC_ALL=C

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${REPO_ROOT:-$(git -C "$HERE/.." rev-parse --show-toplevel 2>/dev/null || true)}"
[ -n "$REPO" ] || REPO="$(cd "$HERE/.." && pwd)"

VEC="$REPO/hadronyche-vectors"
TOOLS="$REPO/hadronyche-tools"
NOSTD_TARGET="aarch64-unknown-none"

TMPROOT="${TMPDIR:-/tmp/opencode}"
mkdir -p "$TMPROOT" 2>/dev/null || TMPROOT=/tmp
TMP="$(mktemp -d "$TMPROOT/007a-03.XXXXXX")" || { echo "FAIL  cannot create temp dir"; exit 1; }
trap 'rm -rf "$TMP"' EXIT

# Keyed by checkout path: cargo's artifact hashes for path packages do not
# cover the absolute path, so two checkouts with the same layout (a worktree
# and a scratch copy) sharing one target dir reuse each other's builds.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$TMPROOT/target/007a-03-$(printf '%s' "$REPO" | cksum | cut -d' ' -f1)}"
export CARGO_TERM_COLOR=never
mkdir -p "$CARGO_TARGET_DIR" 2>/dev/null || true

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
echo "   target: $CARGO_TARGET_DIR"

# -------------------------------------------------------------------- AC 1
echo "-- AC1 reader compiles with no_std for a target without std"
if [ -f "$VEC/src/lib.rs" ] && grep -Eq '^[[:space:]]*#!\[(no_std|cfg_attr\(not\(test\), no_std\))\]' "$VEC/src/lib.rs"; then
  pass "AC1 hadronyche-vectors/src/lib.rs declares no_std"
else
  bad "AC1 hadronyche-vectors/src/lib.rs does not declare #![no_std]"
fi

if ! command -v rustup >/dev/null 2>&1; then
  bad "AC1 rustup not found in PATH; cannot confirm target $NOSTD_TARGET"
elif rustup target list --installed 2>/dev/null | grep -Fxq "$NOSTD_TARGET"; then
  if tb "$TMP/nostd.log" sh -c "cd '$REPO' && cargo build -p hadronyche-vectors --no-default-features --target '$NOSTD_TARGET'"; then
    pass "AC1 cargo build -p hadronyche-vectors --no-default-features --target $NOSTD_TARGET succeeds"
  else
    rc=$?
    bad "AC1 no_std build failed for $NOSTD_TARGET (exit $rc)"
    logtail "$TMP/nostd.log"
  fi
else
  bad "AC1 target $NOSTD_TARGET is not installed; run: rustup target add $NOSTD_TARGET (CI installs it too)"
fi

# -------------------------------------------------------------- AC 2 and 3
echo "-- AC2/AC3 hadronyche-vectors tests: enumeration, bytes, decoded or rejected"
TESTLOG="$TMP/test.log"
test_rc=0
if tb "$TESTLOG" sh -c "cd '$REPO' && cargo test -p hadronyche-vectors"; then
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

# One `test <name> ... ok` line per test in reader.rs. A test that did not run
# (for example a compile error) is reported as not run.
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
check_test "AC2" ac2_enumeration_yields_each_example_with_id_and_spec_version_unchanged \
  "the six examples are enumerated with id and spec_version unchanged"
check_test "AC3" ac3_bytes_of_every_bin_example_equal_the_file_byte_for_byte \
  "bytes of every .bin fixture equal the file byte for byte"
check_test "AC2" work1_wire_examples_decode_or_reject_with_their_declared_reason \
  "wire valid decodes; wire invalid rejects with its declared reason"
check_test "AC2" work2_each_kind_enumerates_only_its_own_vectors \
  "each of the six kinds enumerates only its own vectors"
check_test "DoD" work3_every_embedded_vector_carries_a_nonempty_id_and_spec_version \
  "every embedded vector carries id and spec_version"

# -------------------------------------------------------------------- AC 4
echo "-- AC4 no std in the default feature set"
for flags in "--no-default-features" ""; do
  label="${flags:-default features}"
  if tb "$TMP/tree.log" sh -c "cd '$REPO' && cargo tree -p hadronyche-vectors -e normal $flags --prefix none"; then
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
  pass "AC4 no allocator used in src (none needed in the default set)"
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

# --------------------------------------------------------------------- DoD
echo "-- DoD the reader's parsing is a fuzz target for 007a-15"
LIST="$TOOLS/fuzz-targets.txt"
entry="$(grep -E '^[[:space:]]*vector-reader[[:space:]]' "$LIST" 2>/dev/null || true)"
if [ -n "$entry" ]; then
  pass "DoD fuzz-targets.txt lists vector-reader"
  harness_rel="$(printf '%s' "$entry" | awk '{print $2}')"
  corpus_rel="$(printf '%s' "$entry" | awk '{print $3}')"
  if [ -f "$TOOLS/$harness_rel" ]; then
    pass "DoD harness $harness_rel exists"
    if tb "$TMP/fuzzbin.log" sh -c "cd '$REPO' && cargo build -p hadronyche-tools --bin vector-reader-fuzz"; then
      pass "DoD harness builds (cargo build -p hadronyche-tools --bin vector-reader-fuzz)"
    else
      bad "DoD harness does not build (cargo build -p hadronyche-tools --bin vector-reader-fuzz)"
      logtail "$TMP/fuzzbin.log"
    fi
  else
    bad "DoD harness $harness_rel named in fuzz-targets.txt does not exist"
  fi
  seeds="$(git -C "$REPO" ls-files -- "hadronyche-tools/$corpus_rel" 2>/dev/null | sed '/^$/d' | wc -l)"
  if [ "$seeds" -gt 0 ]; then
    pass "DoD seed corpus hadronyche-tools/$corpus_rel is tracked ($seeds file(s))"
  else
    bad "DoD seed corpus hadronyche-tools/$corpus_rel is missing or untracked"
  fi
else
  bad "DoD $LIST has no vector-reader line (007a-15 finds the reader's fuzz target there)"
fi

nospec=0; total=0
while IFS= read -r f; do
  [ -n "$f" ] || continue
  total=$((total + 1))
  if ! grep -q '"spec_version"' "$f"; then
    bad "DoD $f has no spec_version"
    nospec=$((nospec + 1))
  fi
done < <(find "$VEC/vectors" "$VEC/tests/fixtures/vectors/examples" -name '*.json' \
           -not -path '*/negative/*' 2>/dev/null)
if [ "$nospec" -eq 0 ]; then
  pass "DoD every vector JSON file carries spec_version ($total file(s) checked)"
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 007a-03 PASSED =="
  exit 0
fi
echo "== 007a-03 FAILED =="
exit 1
