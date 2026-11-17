# 007a-01 — sourced by scripts/repo-state-checks.sh.
# From checks/007a-01.sh ("Crate skeleton and vector layout"): every
# assertion of the script, unchanged. They run tools (cargo, rustup) and read
# repository state (manifests, sources, git), so none is a document check.
#
#   AC1  hadronyche-vectors/Cargo.toml exists and names the package
#        `hadronyche-vectors`; src/lib.rs exists, carries `#![no_std]` and a
#        `//!` doc comment recording C-192; `cargo build` succeeds on the host
#        and for a no_std target (the first installed of aarch64-unknown-none,
#        thumbv7em-none-eabi, thumbv7em-none-eabihf, riscv32imac-unknown-none-elf,
#        x86_64-unknown-none; none installed: `rustup target add
#        aarch64-unknown-none` once, a failure of that is a FAIL).
#   AC2  hadronyche-tools/Cargo.toml exists and names `hadronyche-tools`;
#        `cargo build --bin gen` and `cargo run --bin gen` succeed.
#   AC3  six data directories exist and are tracked (`git ls-files`).
#   AC4  this file's own command lines hold no privilege-escalation tool; the
#        uid is not 0.
#
# Environment, as the script honoured it: REPO_ROOT (default: the checkout),
# CHECK_TIMEOUT (seconds per build, default 300), CARGO_TARGET_DIR (default
# ${TMPDIR:-/tmp/opencode}/target/007a-01), TMPDIR. Neither the target dir nor
# LC_ALL leaks to the files that follow (function-local).
r007a01_self="${BASH_SOURCE[0]}"

r007a01_run() {
    local tmp="$1"
    local REPO VEC TOOLS LIMIT d full tracked n rc cand installed
    local vectors_ok tools_ok TARGET
    REPO="${REPO_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || true)}"
    [ -n "$REPO" ] || REPO="$(pwd)"
    VEC="$REPO/hadronyche-vectors"
    TOOLS="$REPO/hadronyche-tools"
    local -x LC_ALL=C
    local -x CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${r007a01_tmproot}/target/007a-01}"
    local -x CARGO_TERM_COLOR=never
    mkdir -p "$CARGO_TARGET_DIR" 2>/dev/null || true
    LIMIT="${CHECK_TIMEOUT:-300}"

    # usage: tb <logfile> <cmd...>
    tb() { local log="$1"; shift; if command -v timeout >/dev/null 2>&1; then timeout "$LIMIT" "$@" >"$log" 2>&1; else "$@" >"$log" 2>&1; fi; }
    note() { printf '      %s\n' "$*"; }
    logtail() { tail -n 20 "$1" 2>/dev/null | while IFS= read -r l; do note "$l"; done; }

    echo "== 007a-01: crate skeleton and vector layout (repo $REPO, target $CARGO_TARGET_DIR)"

    # ---------------------------------------------------------------- AC 1
    vectors_ok=1
    if [ -f "$VEC/Cargo.toml" ]; then
        pass "007a-01 AC1" "$VEC/Cargo.toml exists"
        if grep -Eq '^[[:space:]]*name[[:space:]]*=[[:space:]]*"hadronyche-vectors"' "$VEC/Cargo.toml"; then
            pass "007a-01 AC1" "package name is hadronyche-vectors"
        else
            bad "007a-01 AC1" "package name in $VEC/Cargo.toml is not hadronyche-vectors"
            vectors_ok=0
        fi
    else
        bad "007a-01 AC1" "$VEC/Cargo.toml does not exist (the crate must sit at the repository root, C-192)"
        vectors_ok=0
    fi

    if [ -f "$VEC/src/lib.rs" ]; then
        if grep -Eq '^[[:space:]]*#!\[no_std\]' "$VEC/src/lib.rs"; then
            pass "007a-01 AC1" "src/lib.rs carries #![no_std]"
        else
            bad "007a-01 AC1" "src/lib.rs does not carry #![no_std]"
            vectors_ok=0
        fi
        if grep -Eq '^[[:space:]]*//!.*C-192' "$VEC/src/lib.rs"; then
            pass "007a-01 AC1" "top-level doc comment records C-192"
        elif grep -Rqs 'C-192' "$VEC/src/lib.rs" "$VEC/Cargo.toml" 2>/dev/null; then
            bad "007a-01 AC1" "C-192 is present but not in a //! doc comment of src/lib.rs (ticket: record it in the crate's top-level doc comment)"
            vectors_ok=0
        else
            bad "007a-01 AC1" "C-192 (vectors crate at the workspace root) is not recorded"
            vectors_ok=0
        fi
    else
        bad "007a-01 AC1" "$VEC/src/lib.rs does not exist"
        vectors_ok=0
    fi

    # Host build. A skeleton build failure is a FAIL, not a skip.
    if [ "$vectors_ok" -eq 1 ] && command -v cargo >/dev/null 2>&1; then
        echo "   running: (cd $VEC && cargo build)  [host, may compile]"
        if tb "$tmp/vec-host.log" sh -c "cd '$VEC' && cargo build"; then
            pass "007a-01 AC1" "cargo build succeeds on the host"
        else
            rc=$?
            bad "007a-01 AC1" "cargo build failed on the host (exit $rc)"
            logtail "$tmp/vec-host.log"
            vectors_ok=0
        fi
    elif [ "$vectors_ok" -eq 1 ]; then
        bad "007a-01 AC1" "cargo not found in PATH"
        vectors_ok=0
    fi

    # A target without std.
    if [ "$vectors_ok" -eq 1 ]; then
        TARGET=""
        if command -v rustup >/dev/null 2>&1; then
            installed="$(rustup target list --installed 2>/dev/null || true)"
            for cand in aarch64-unknown-none thumbv7em-none-eabi thumbv7em-none-eabihf \
                        riscv32imac-unknown-none-elf x86_64-unknown-none; do
                if printf '%s\n' "$installed" | grep -Fxq "$cand"; then
                    TARGET="$cand"
                    break
                fi
            done
            if [ -z "$TARGET" ]; then
                note "no bare-metal target installed; trying: rustup target add aarch64-unknown-none (one network fetch, permitted)"
                tb "$tmp/rustup-add.log" rustup target add aarch64-unknown-none
                rc=$?
                if [ "$rc" -ne 0 ]; then
                    bad "007a-01 AC1" "cannot install no_std target aarch64-unknown-none (rustup exit $rc)"
                    logtail "$tmp/rustup-add.log"
                elif rustup target list --installed 2>/dev/null | grep -Fxq aarch64-unknown-none; then
                    TARGET="aarch64-unknown-none"
                else
                    bad "007a-01 AC1" "rustup target add reported success but aarch64-unknown-none is not installed"
                fi
            fi
        else
            bad "007a-01 AC1" "rustup not found in PATH; cannot select a target without std"
        fi

        if [ -n "$TARGET" ]; then
            pass "007a-01 AC1" "no_std build target: $TARGET"
            echo "   running: (cd $VEC && cargo build --target $TARGET)  [may compile]"
            if tb "$tmp/vec-nostd.log" sh -c "cd '$VEC' && cargo build --target '$TARGET'"; then
                pass "007a-01 AC1" "cargo build succeeds for $TARGET (no std)"
            else
                rc=$?
                bad "007a-01 AC1" "cargo build failed for $TARGET (exit $rc)"
                logtail "$tmp/vec-nostd.log"
            fi
        fi
    else
        note "AC1 builds not attempted: the crate skeleton is missing or incomplete"
    fi

    # ---------------------------------------------------------------- AC 2
    tools_ok=1
    if [ -f "$TOOLS/Cargo.toml" ]; then
        pass "007a-01 AC2" "$TOOLS/Cargo.toml exists"
        if grep -Eq '^[[:space:]]*name[[:space:]]*=[[:space:]]*"hadronyche-tools"' "$TOOLS/Cargo.toml"; then
            pass "007a-01 AC2" "package name is hadronyche-tools"
        else
            bad "007a-01 AC2" "package name in $TOOLS/Cargo.toml is not hadronyche-tools"
            tools_ok=0
        fi
    else
        bad "007a-01 AC2" "$TOOLS/Cargo.toml does not exist (C-133: gen lives in hadronyche-tools)"
        tools_ok=0
    fi

    if [ "$tools_ok" -eq 1 ] && command -v cargo >/dev/null 2>&1; then
        echo "   running: (cd $TOOLS && cargo build --bin gen)  [may compile]"
        if tb "$tmp/tools-build.log" sh -c "cd '$TOOLS' && cargo build --bin gen"; then
            pass "007a-01 AC2" "cargo build --bin gen succeeds"
        else
            rc=$?
            bad "007a-01 AC2" "cargo build --bin gen failed (exit $rc): no runnable binary target 'gen'"
            logtail "$tmp/tools-build.log"
            tools_ok=0
        fi
    elif [ "$tools_ok" -eq 1 ]; then
        bad "007a-01 AC2" "cargo not found in PATH"
        tools_ok=0
    fi

    if [ "$tools_ok" -eq 1 ]; then
        echo "   running: (cd $TOOLS && cargo run --bin gen)"
        tb "$tmp/tools-run.log" sh -c "cd '$TOOLS' && cargo run --quiet --bin gen"
        rc=$?
        if [ "$rc" -eq 0 ]; then
            pass "007a-01 AC2" "cargo run --bin gen exits with status 0"
        elif [ "$rc" -eq 124 ]; then
            bad "007a-01 AC2" "cargo run --bin gen timed out after ${LIMIT}s"
            logtail "$tmp/tools-run.log"
        else
            bad "007a-01 AC2" "cargo run --bin gen exited $rc (expected 0)"
            logtail "$tmp/tools-run.log"
        fi
    fi

    # ---------------------------------------------------------------- AC 3
    for d in vectors/wire/valid vectors/wire/invalid vectors/control \
             vectors/timeline vectors/transport vectors/reference; do
        full="$VEC/$d"
        if [ ! -d "$full" ]; then
            bad "007a-01 AC3" "missing directory hadronyche-vectors/$d"
            continue
        fi
        tracked="$(git -C "$REPO" ls-files -- "hadronyche-vectors/$d" 2>/dev/null || true)"
        if [ -n "$tracked" ]; then
            n="$(printf '%s\n' "$tracked" | sed '/^$/d' | wc -l)"
            pass "007a-01 AC3" "hadronyche-vectors/$d exists and is tracked ($n file(s))"
        else
            bad "007a-01 AC3" "hadronyche-vectors/$d exists but has no tracked placeholder (.gitkeep is enough)"
        fi
    done

    # ---------------------------------------------------------------- AC 4
    # The forbidden-tool pattern is assembled from fragments so that this
    # self-check does not match its own search expression.
    local priv_pat priv_hits
    priv_pat="$(printf 's%sdo|c%spsh' 'u' 'a')"
    priv_hits="$(grep -vE '^[[:space:]]*#' "$r007a01_self" 2>/dev/null | grep -nwE "$priv_pat" || true)"
    if [ -z "$priv_hits" ]; then
        pass "007a-01 AC4" "this check invokes no root-only escalation tool"
    else
        bad "007a-01 AC4" "this check contains root-only command lines:"
        printf '%s\n' "$priv_hits" | head -n 5 | while IFS= read -r l; do note "$l"; done
    fi

    if [ "$(id -u 2>/dev/null || echo 1)" -ne 0 ]; then
        pass "007a-01 AC4" "running unprivileged (uid $(id -u))"
    else
        bad "007a-01 AC4" "running as root (uid 0); the ticket requires an unprivileged run"
    fi
}

r007a01_tmproot="${TMPDIR:-/tmp/opencode}"
mkdir -p "$r007a01_tmproot" 2>/dev/null || r007a01_tmproot=/tmp
if r007a01_tmp="$(mktemp -d "$r007a01_tmproot/007a-01.XXXXXX")"; then
    r007a01_run "$r007a01_tmp"
    rm -rf "$r007a01_tmp"
else
    bad "007a-01" "cannot create temp dir"
fi
unset -f r007a01_run tb note logtail
