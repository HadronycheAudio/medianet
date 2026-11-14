#!/usr/bin/env bash
# checks/007a-01.sh — acceptance check for ticket 007a-01
# "Crate skeleton and vector layout".
#
# The deliverables are checked in this repository (the design set's
# repository), which is the checkout this script lives in:
#   hadronyche-vectors/     no_std data crate, at the repository root (C-192)
#   hadronyche-tools/       hosted-only tool crate with a `gen` binary (C-133)
#   hadronyche-vectors/vectors/{wire/{valid,invalid},control,timeline,transport,reference}/
#
# Acceptance criteria and their mechanical encoding
# -------------------------------------------------
# AC1  hadronyche-vectors/Cargo.toml exists and names the package
#      `hadronyche-vectors`; hadronyche-vectors/src/lib.rs contains
#      `#![no_std]`; the crate's top-level doc comment records C-192; and
#      `cargo build` succeeds both on the host and for a target that has no
#      `std` (a bare-metal target).  A crate that does not exist yet, or that
#      fails either build, is a FAIL — that is the RED state of this ticket.
#
#      no_std target selection: the first already-installed target among
#      aarch64-unknown-none, thumbv7em-none-eabi, thumbv7em-none-eabihf,
#      riscv32imac-unknown-none-elf, x86_64-unknown-none is used.  If none is
#      installed, this check runs `rustup target add aarch64-unknown-none`
#      once (a network fetch of a published toolchain component, which is
#      explicitly permitted for this ticket) and retries.  In this RED
#      environment only x86_64-unknown-linux-gnu is installed, so the check
#      is written to add aarch64-unknown-none when the crate exists.
#
# AC2  hadronyche-tools/Cargo.toml exists and names the package
#      `hadronyche-tools`; `cargo build --bin gen` succeeds in that crate
#      (which proves a binary target named `gen` is defined, either
#      src/bin/gen.rs or an explicit [[bin]]); and `cargo run --bin gen`
#      exits with status 0.
#
# AC3  Each of the six data directories fixed by the ticket's Context exists
#      relative to the vectors crate and is tracked: `git ls-files -- <dir>`
#      is non-empty (at least one placeholder, e.g. .gitkeep).  Git does not
#      track directories, so a tracked placeholder is the mechanical test.
#
# AC4  The check runs as an unprivileged user and invokes nothing requiring
#      root.  Self-check: no command line in this script (comments excluded)
#      contains `sudo` or `capsh`; the effective uid is not 0.
#
# Builds are quiet (logs captured, tails printed only on failure) and bounded
# by `timeout ${CHECK_TIMEOUT:-300}` seconds each, so this check can never
# hang.  A shared CARGO_TARGET_DIR under /tmp/opencode keeps rebuilds cheap.
# This script has no dependency beyond bash, git, rustup/cargo and coreutils.
#
# Exit status: 0 all criteria pass, 1 otherwise (one FAIL line per criterion).

set -u
export LC_ALL=C

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${REPO_ROOT:-$(git -C "$HERE/.." rev-parse --show-toplevel 2>/dev/null || true)}"
[ -n "$REPO" ] || REPO="$(cd "$HERE/.." && pwd)"

VEC="$REPO/hadronyche-vectors"
TOOLS="$REPO/hadronyche-tools"

TMPROOT="${TMPDIR:-/tmp/opencode}"
mkdir -p "$TMPROOT" 2>/dev/null || TMPROOT=/tmp
TMP="$(mktemp -d "$TMPROOT/007a-01.XXXXXX")" || { echo "FAIL  cannot create temp dir"; exit 1; }
trap 'rm -rf "$TMP"' EXIT

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$TMPROOT/target/007a-01}"
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

echo "== 007a-01: crate skeleton and vector layout =="
echo "   repo:   $REPO"
echo "   target: $CARGO_TARGET_DIR"

# -------------------------------------------------------------------- AC 1
echo "-- AC1 hadronyche-vectors: no_std crate at the repository root (C-192)"
vectors_ok=1
if [ -f "$VEC/Cargo.toml" ]; then
  pass "AC1 $VEC/Cargo.toml exists"
  if grep -Eq '^[[:space:]]*name[[:space:]]*=[[:space:]]*"hadronyche-vectors"' "$VEC/Cargo.toml"; then
    pass "AC1 package name is hadronyche-vectors"
  else
    bad "AC1 package name in $VEC/Cargo.toml is not hadronyche-vectors"
    vectors_ok=0
  fi
else
  bad "AC1 $VEC/Cargo.toml does not exist (the crate must sit at the repository root, C-192)"
  vectors_ok=0
fi

if [ -f "$VEC/src/lib.rs" ]; then
  if grep -Eq '^[[:space:]]*#!\[no_std\]' "$VEC/src/lib.rs"; then
    pass "AC1 src/lib.rs carries #![no_std]"
  else
    bad "AC1 src/lib.rs does not carry #![no_std]"
    vectors_ok=0
  fi
  if grep -Eq '^[[:space:]]*//!.*C-192' "$VEC/src/lib.rs"; then
    pass "AC1 top-level doc comment records C-192"
  elif grep -Rqs 'C-192' "$VEC/src/lib.rs" "$VEC/Cargo.toml" 2>/dev/null; then
    bad "AC1 C-192 is present but not in a //! doc comment of src/lib.rs (ticket: record it in the crate's top-level doc comment)"
    vectors_ok=0
  else
    bad "AC1 C-192 (vectors crate at the workspace root) is not recorded"
    vectors_ok=0
  fi
else
  bad "AC1 $VEC/src/lib.rs does not exist"
  vectors_ok=0
fi

# Host build.  A skeleton build failure is a FAIL, not a skip.
if [ "$vectors_ok" -eq 1 ] && command -v cargo >/dev/null 2>&1; then
  echo "   running: (cd $VEC && cargo build)  [host, may compile]"
  if tb "$TMP/vec-host.log" sh -c "cd '$VEC' && cargo build"; then
    pass "AC1 cargo build succeeds on the host"
  else
    rc=$?
    bad "AC1 cargo build failed on the host (exit $rc)"
    logtail "$TMP/vec-host.log"
    vectors_ok=0
  fi
elif [ "$vectors_ok" -eq 1 ]; then
  bad "AC1 cargo not found in PATH"
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
      tb "$TMP/rustup-add.log" rustup target add aarch64-unknown-none
      rc=$?
      if [ "$rc" -ne 0 ]; then
        bad "AC1 cannot install no_std target aarch64-unknown-none (rustup exit $rc)"
        logtail "$TMP/rustup-add.log"
      elif rustup target list --installed 2>/dev/null | grep -Fxq aarch64-unknown-none; then
        TARGET="aarch64-unknown-none"
      else
        bad "AC1 rustup target add reported success but aarch64-unknown-none is not installed"
      fi
    fi
  else
    bad "AC1 rustup not found in PATH; cannot select a target without std"
  fi

  if [ -n "$TARGET" ]; then
    pass "AC1 no_std build target: $TARGET"
    echo "   running: (cd $VEC && cargo build --target $TARGET)  [may compile]"
    if tb "$TMP/vec-nostd.log" sh -c "cd '$VEC' && cargo build --target '$TARGET'"; then
      pass "AC1 cargo build succeeds for $TARGET (no std)"
    else
      rc=$?
      bad "AC1 cargo build failed for $TARGET (exit $rc)"
      logtail "$TMP/vec-nostd.log"
    fi
  fi
else
  note "AC1 builds not attempted: the crate skeleton is missing or incomplete"
fi

# -------------------------------------------------------------------- AC 2
echo "-- AC2 hadronyche-tools: gen binary builds and exits 0"
tools_ok=1
if [ -f "$TOOLS/Cargo.toml" ]; then
  pass "AC2 $TOOLS/Cargo.toml exists"
  if grep -Eq '^[[:space:]]*name[[:space:]]*=[[:space:]]*"hadronyche-tools"' "$TOOLS/Cargo.toml"; then
    pass "AC2 package name is hadronyche-tools"
  else
    bad "AC2 package name in $TOOLS/Cargo.toml is not hadronyche-tools"
    tools_ok=0
  fi
else
  bad "AC2 $TOOLS/Cargo.toml does not exist (C-133: gen lives in hadronyche-tools)"
  tools_ok=0
fi

if [ "$tools_ok" -eq 1 ] && command -v cargo >/dev/null 2>&1; then
  echo "   running: (cd $TOOLS && cargo build --bin gen)  [may compile]"
  if tb "$TMP/tools-build.log" sh -c "cd '$TOOLS' && cargo build --bin gen"; then
    pass "AC2 cargo build --bin gen succeeds"
  else
    rc=$?
    bad "AC2 cargo build --bin gen failed (exit $rc): no runnable binary target 'gen'"
    logtail "$TMP/tools-build.log"
    tools_ok=0
  fi
elif [ "$tools_ok" -eq 1 ]; then
  bad "AC2 cargo not found in PATH"
  tools_ok=0
fi

if [ "$tools_ok" -eq 1 ]; then
  echo "   running: (cd $TOOLS && cargo run --bin gen)"
  tb "$TMP/tools-run.log" sh -c "cd '$TOOLS' && cargo run --quiet --bin gen"
  rc=$?
  if [ "$rc" -eq 0 ]; then
    pass "AC2 cargo run --bin gen exits with status 0"
  elif [ "$rc" -eq 124 ]; then
    bad "AC2 cargo run --bin gen timed out after ${LIMIT}s"
    logtail "$TMP/tools-run.log"
  else
    bad "AC2 cargo run --bin gen exited $rc (expected 0)"
    logtail "$TMP/tools-run.log"
  fi
fi

# -------------------------------------------------------------------- AC 3
echo "-- AC3 six tracked data directories relative to the vectors crate"
for d in vectors/wire/valid vectors/wire/invalid vectors/control \
         vectors/timeline vectors/transport vectors/reference; do
  full="$VEC/$d"
  if [ ! -d "$full" ]; then
    bad "AC3 missing directory hadronyche-vectors/$d"
    continue
  fi
  tracked="$(git -C "$REPO" ls-files -- "hadronyche-vectors/$d" 2>/dev/null || true)"
  if [ -n "$tracked" ]; then
    n="$(printf '%s\n' "$tracked" | sed '/^$/d' | wc -l)"
    pass "AC3 hadronyche-vectors/$d exists and is tracked ($n file(s))"
  else
    bad "AC3 hadronyche-vectors/$d exists but has no tracked placeholder (.gitkeep is enough)"
  fi
done

# -------------------------------------------------------------------- AC 4
echo "-- AC4 unprivileged execution"
SELF="$HERE/$(basename "${BASH_SOURCE[0]}")"
# The forbidden-tool pattern is assembled from fragments so that this
# self-check does not match its own search expression.
priv_pat="$(printf 's%sdo|c%spsh' 'u' 'a')"
priv_hits="$(grep -vE '^[[:space:]]*#' "$SELF" 2>/dev/null | grep -nwE "$priv_pat" || true)"
if [ -z "$priv_hits" ]; then
  pass "AC4 this check invokes no root-only escalation tool"
else
  bad "AC4 this check contains root-only command lines:"
  printf '%s\n' "$priv_hits" | head -n 5 | while IFS= read -r l; do note "$l"; done
fi

if [ "$(id -u 2>/dev/null || echo 1)" -ne 0 ]; then
  pass "AC4 running unprivileged (uid $(id -u))"
else
  bad "AC4 running as root (uid 0); the ticket requires an unprivileged run"
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 007a-01 PASSED =="
  exit 0
fi
echo "== 007a-01 FAILED =="
exit 1
