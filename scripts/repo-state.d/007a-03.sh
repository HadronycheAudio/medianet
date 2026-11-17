# 007a-03 — sourced by scripts/repo-state-checks.sh.
# From checks/007a-03.sh ("No-std vector reader", with the review hardening of
# PR #34): every assertion of the script, unchanged. They run tools (cargo,
# rustup, python3 for T1), and read repository state and manifests/sources
# outside doc/, so none is a document check.
#
#   AC1  src/lib.rs declares `#![no_std]` unconditionally; `cargo build -p
#        hadronyche-vectors --no-default-features --target aarch64-unknown-none`
#        succeeds; T1: the six example ids are in a generated file under that
#        target's build output. A missing target FAILS with the rustup command,
#        never skips.
#   AC2/AC3/Work 1/Work 2/Work 3  `cargo test --no-fail-fast -p
#        hadronyche-vectors` passes, and seven named tests report `ok`.
#   T2   src/ has no file-system or environment access (comments ignored).
#   AC4  `cargo tree -e normal` lists nothing beyond hadronyche-vectors, with
#        and without default features; every [dependencies] entry sets
#        default-features = false; no std path outside comments; an allocator
#        only behind a documented non-default feature.
#   T3   no `examples` feature, no self dependency, [features] absent or only
#        `default = []`; `cargo tree --workspace -e features,dev` shows no
#        feature of hadronyche-vectors beyond `default`.
#   DoD  fuzz-targets.txt lists vector-reader and metrics, every entry's
#        harness exists and its corpus is tracked; vector-reader-fuzz builds;
#        every vector JSON carries spec_version; the cargo target dir is under
#        the git-ignored target/.
#
# Environment, as the script honoured it: REPO_ROOT (default: the checkout),
# CHECK_TIMEOUT (seconds per tool run, default 300), CHECK_CARGO_TARGET_DIR
# (default $REPO/target/check-007a-03), TMPDIR. CARGO_TARGET_DIR is set for
# this file's cargo calls only. LC_ALL=C is function-local.
r007a03_run() {
    local tmp="$1"
    local REPO VEC TOOLS EXAMPLES NOSTD_TARGET CHECK_TARGET LIMIT TESTLOG test_rc rc
    local ids n_ids n_found gen_files id rt_hits flags label extra std_hits alloc_hits alloc_ok
    local file line start default_line features_section leaked LIST entries name harness_rel
    local corpus_rel seeds nospec total f l
    REPO="${REPO_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || true)}"
    [ -n "$REPO" ] || REPO="$(pwd)"
    VEC="$REPO/hadronyche-vectors"
    TOOLS="$REPO/hadronyche-tools"
    EXAMPLES="$VEC/tests/fixtures/vectors/examples"
    NOSTD_TARGET="aarch64-unknown-none"
    local -x LC_ALL=C
    CHECK_TARGET="${CHECK_CARGO_TARGET_DIR:-$REPO/target/check-007a-03}"
    mkdir -p "$CHECK_TARGET" 2>/dev/null || true
    LIMIT="${CHECK_TIMEOUT:-300}"

    # usage: tb <logfile> <cmd...>
    tb() { local log="$1"; shift; if command -v timeout >/dev/null 2>&1; then timeout "$LIMIT" "$@" >"$log" 2>&1; else "$@" >"$log" 2>&1; fi; }
    note() { printf '      %s\n' "$*"; }
    logtail() { tail -n 20 "$1" 2>/dev/null | while IFS= read -r l; do note "$l"; done; }

    echo "== 007a-03: no-std vector reader (repo $REPO, target $CHECK_TARGET)"

    # ---------------------------------------------------------------- AC 1
    if [ -f "$VEC/src/lib.rs" ] && grep -Eq '^#!\[no_std\][[:space:]]*$' "$VEC/src/lib.rs"; then
        pass "007a-03 AC1" "hadronyche-vectors/src/lib.rs declares #![no_std] unconditionally"
    else
        bad "007a-03 AC1" "hadronyche-vectors/src/lib.rs does not declare #![no_std] unconditionally"
    fi

    if ! command -v rustup >/dev/null 2>&1; then
        bad "007a-03 AC1" "rustup not found in PATH; cannot confirm target $NOSTD_TARGET"
    elif rustup target list --installed 2>/dev/null | grep -Fxq "$NOSTD_TARGET"; then
        if tb "$tmp/nostd.log" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo build -p hadronyche-vectors --no-default-features --target '$NOSTD_TARGET'"; then
            pass "007a-03 AC1" "cargo build -p hadronyche-vectors --no-default-features --target $NOSTD_TARGET succeeds"
        else
            rc=$?
            bad "007a-03 AC1" "no_std build failed for $NOSTD_TARGET (exit $rc)"
            logtail "$tmp/nostd.log"
        fi
    else
        bad "007a-03 AC1" "target $NOSTD_TARGET is not installed; run: rustup target add $NOSTD_TARGET (CI installs it too)"
    fi

    # T1: the no_std build must have compiled data. The six example ids must be
    # in a file cargo generated under the target's build output (the build
    # script's OUT_DIR for that target), not only in the sources.
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
        # shellcheck disable=SC2086
        if [ -n "$gen_files" ] && grep -qF "\"$id\"" $gen_files 2>/dev/null; then
            n_found=$((n_found + 1))
        fi
    done <<EOF
$ids
EOF
    if [ "$n_ids" -eq 6 ] && [ "$n_found" -eq 6 ]; then
        pass "007a-03 AC1" "no_std build embeds data: all $n_found example ids found in generated build output for $NOSTD_TARGET"
    else
        bad "007a-03 AC1" "no_std build does not embed the example data: $n_found of $n_ids example ids in generated output for $NOSTD_TARGET (expected 6 of 6)"
    fi

    # ------------------------------------------------------- AC 2 and 3
    TESTLOG="$tmp/test.log"
    test_rc=0
    if tb "$TESTLOG" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo test --no-fail-fast -p hadronyche-vectors"; then
        pass "007a-03 AC2" "cargo test -p hadronyche-vectors passes"
    else
        test_rc=$?
        bad "007a-03 AC2" "cargo test -p hadronyche-vectors did not pass (exit $test_rc)"
        if grep -Eq '^error(\[E[0-9]+\])?:' "$TESTLOG"; then
            note "the test target does not compile; first errors:"
            grep -m3 -A4 -E '^error' "$TESTLOG" | while IFS= read -r l; do note "$l"; done
        else
            logtail "$TESTLOG"
        fi
    fi

    # One `test <name> ... ok` line per test. A test that did not run (for
    # example a compile error) is reported as not run.
    check_test() {
        local label="$1" name="$2" desc="$3"
        if grep -Eq "^test ${name} \.\.\. ok\$" "$TESTLOG" 2>/dev/null; then
            pass "007a-03 $label" "$desc (test $name)"
        elif [ "$test_rc" -ne 0 ]; then
            bad "007a-03 $label" "$desc (test $name not run or failed)"
        else
            bad "007a-03 $label" "$desc (test $name not found in the test run)"
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

    # T2 (Work 3): the library reads nothing at run time; data is embedded at
    # build time. Comment lines are ignored.
    rt_hits="$(grep -rnE '(std::fs|fs::|read_dir|\bFile\b|std::io|std::env|env::|OsStr|read_to_string)' \
        "$VEC/src" 2>/dev/null | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' || true)"
    if [ -z "$rt_hits" ]; then
        pass "007a-03 Work3" "hadronyche-vectors/src has no file-system or environment access (data is embedded at build time)"
    else
        bad "007a-03 Work3" "run-time file or environment access in hadronyche-vectors/src:"
        printf '%s\n' "$rt_hits" | head -n 5 | while IFS= read -r l; do note "$l"; done
    fi

    # ---------------------------------------------------------------- AC 4
    for flags in "--no-default-features" ""; do
        label="${flags:-default features}"
        if tb "$tmp/tree.log" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo tree -p hadronyche-vectors -e normal $flags --prefix none"; then
            extra="$(awk 'NF && $1 != "hadronyche-vectors" {print $1}' "$tmp/tree.log" | sort -u | tr '\n' ' ')"
            if [ -z "$extra" ]; then
                pass "007a-03 AC4" "normal dependencies with $label: none beyond hadronyche-vectors"
            else
                bad "007a-03 AC4" "normal dependencies with $label pull in: $extra"
            fi
        else
            bad "007a-03 AC4" "cargo tree -e normal $label failed"
            logtail "$tmp/tree.log"
        fi
    done

    if awk '
      /^\[dependencies\]/ {indeps=1; next}
      /^\[/ {indeps=0}
      indeps && NF && $0 !~ /^[[:space:]]*#/ && $0 !~ /default-features[[:space:]]*=[[:space:]]*false/ {bad=1}
      END {exit bad}' "$VEC/Cargo.toml"; then
        pass "007a-03 AC4" "every [dependencies] entry sets default-features = false (or there are none)"
    else
        bad "007a-03 AC4" "a [dependencies] entry in hadronyche-vectors/Cargo.toml keeps default features"
    fi

    std_hits="$(grep -rnE '(^|[^A-Za-z0-9_:])(std::|extern crate std|use std\b)' "$VEC/src" 2>/dev/null \
        | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' || true)"
    if [ -z "$std_hits" ]; then
        pass "007a-03 AC4" "no std path or extern crate std in hadronyche-vectors/src outside comments"
    else
        bad "007a-03 AC4" "std used in hadronyche-vectors/src (move test-only code to tests/):"
        printf '%s\n' "$std_hits" | head -n 5 | while IFS= read -r l; do note "$l"; done
    fi

    alloc_hits="$(grep -rnE '^[[:space:]]*extern crate alloc' "$VEC/src" 2>/dev/null || true)"
    if [ -z "$alloc_hits" ]; then
        pass "007a-03 AC4" "no allocator used in src (none needed)"
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
            pass "007a-03 AC4" "allocator only behind a named feature, not a default one, documented in Cargo.toml"
        else
            bad "007a-03 AC4" "extern crate alloc must sit behind cfg(feature = \"...\") that is not in default, documented in Cargo.toml"
        fi
    fi

    # ------------------------------------------------------------------ T3
    features_section="$(awk '/^\[features\]/ {on=1; next} /^\[/ {on=0} on && NF && $0 !~ /^[[:space:]]*#/' "$VEC/Cargo.toml")"
    if grep -Eq '^[[:space:]]*examples[[:space:]]*=' "$VEC/Cargo.toml"; then
        bad "007a-03 T3" "hadronyche-vectors/Cargo.toml declares an examples feature (examples are always embedded)"
    else
        pass "007a-03 T3" "Cargo.toml declares no examples feature"
    fi
    if grep -Eq '^[[:space:]]*hadronyche-vectors[[:space:]]*=' "$VEC/Cargo.toml"; then
        bad "007a-03 T3" "hadronyche-vectors/Cargo.toml has a dependency on itself (it leaks features under the workspace test command)"
    else
        pass "007a-03 T3" "Cargo.toml has no self dependency"
    fi
    if [ -z "$features_section" ] || [ "$features_section" = 'default = []' ]; then
        pass "007a-03 T3" "the [features] table is absent or only default = []"
    else
        bad "007a-03 T3" "the [features] table declares features that change the build: $(printf '%s' "$features_section" | tr '\n' ' ')"
    fi
    if tb "$tmp/features.log" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo tree --workspace -e features,dev --prefix none"; then
        leaked="$(grep -E 'hadronyche-vectors feature "' "$tmp/features.log" | grep -v 'feature "default"' | sort -u || true)"
        if [ -z "$leaked" ]; then
            pass "007a-03 T3" "cargo tree --workspace -e features,dev: no feature of hadronyche-vectors beyond default"
        else
            bad "007a-03 T3" "a feature of hadronyche-vectors is enabled in the workspace graph:"
            printf '%s\n' "$leaked" | while IFS= read -r l; do note "$l"; done
        fi
    else
        bad "007a-03 T3" "cargo tree --workspace -e features,dev failed"
        logtail "$tmp/features.log"
    fi

    # ----------------------------------------------------------------- DoD
    LIST="$TOOLS/fuzz-targets.txt"
    entries="$(grep -vE '^[[:space:]]*(#|$)' "$LIST" 2>/dev/null || true)"
    if printf '%s\n' "$entries" | awk '$1 == "vector-reader" {f=1} END {exit !f}'; then
        pass "007a-03 DoD" "fuzz-targets.txt lists vector-reader"
    else
        bad "007a-03 DoD" "fuzz-targets.txt has no vector-reader line (the reader's parsing is a fuzz target for 007a-15)"
    fi
    if printf '%s\n' "$entries" | awk '$1 == "metrics" {f=1} END {exit !f}'; then
        pass "007a-03 DoD" "fuzz-targets.txt lists the existing metrics target"
    else
        bad "007a-03 DoD" "fuzz-targets.txt has no metrics line (the list must be complete)"
    fi
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        name="$(printf '%s' "$line" | awk '{print $1}')"
        harness_rel="$(printf '%s' "$line" | awk '{print $2}')"
        corpus_rel="$(printf '%s' "$line" | awk '{print $3}')"
        if [ -f "$TOOLS/$harness_rel" ]; then
            pass "007a-03 DoD" "$name: harness $harness_rel exists"
        else
            bad "007a-03 DoD" "$name: harness $harness_rel named in fuzz-targets.txt does not exist"
        fi
        seeds="$(git -C "$REPO" ls-files -- "hadronyche-tools/$corpus_rel" 2>/dev/null | sed '/^$/d' | wc -l)"
        if [ "$seeds" -gt 0 ]; then
            pass "007a-03 DoD" "$name: seed corpus hadronyche-tools/$corpus_rel is tracked ($seeds file(s))"
        else
            bad "007a-03 DoD" "$name: seed corpus hadronyche-tools/$corpus_rel is missing or untracked"
        fi
    done < <(printf '%s\n' "$entries")

    if tb "$tmp/fuzzbin.log" sh -c "cd '$REPO' && CARGO_TARGET_DIR='$CHECK_TARGET' cargo build -p hadronyche-tools --bin vector-reader-fuzz"; then
        pass "007a-03 DoD" "vector-reader harness builds (cargo build -p hadronyche-tools --bin vector-reader-fuzz)"
    else
        bad "007a-03 DoD" "vector-reader harness does not build (cargo build -p hadronyche-tools --bin vector-reader-fuzz)"
        logtail "$tmp/fuzzbin.log"
    fi

    nospec=0; total=0
    while IFS= read -r f; do
        [ -n "$f" ] || continue
        total=$((total + 1))
        if ! grep -q '"spec_version"' "$f"; then
            bad "007a-03 DoD" "$f has no spec_version"
            nospec=$((nospec + 1))
        fi
    done < <(find "$VEC/vectors" "$EXAMPLES" -name '*.json' -not -path '*/negative/*' 2>/dev/null)
    if [ "$nospec" -eq 0 ]; then
        pass "007a-03 DoD" "every vector JSON file carries spec_version ($total file(s) checked)"
    fi

    if git -C "$REPO" check-ignore -q "$REPO/target/check-007a-03/.probe" 2>/dev/null; then
        pass "007a-03 DoD" "the check's cargo target dir is under the git-ignored target/"
    else
        bad "007a-03 DoD" "target/ is not git-ignored; the check's cargo target dir would be committed"
    fi
}

r007a03_tmproot="${TMPDIR:-/tmp/opencode}"
mkdir -p "$r007a03_tmproot" 2>/dev/null || r007a03_tmproot=/tmp
if r007a03_tmp="$(mktemp -d "$r007a03_tmproot/007a-03.XXXXXX")"; then
    r007a03_run "$r007a03_tmp"
    rm -rf "$r007a03_tmp"
else
    bad "007a-03" "cannot create temp dir"
fi
unset -f r007a03_run tb note logtail check_test
