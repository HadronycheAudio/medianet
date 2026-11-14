#!/usr/bin/env bash
# checks/003-01.sh — acceptance check for ticket 003-01
# "Init repository and tag poc-untouched".
#
# The deliverable lives outside this repository: the proof of concept at
# /home/dorian/workspace/hadropoc (the "POC") must be a git repository whose
# first commit is tagged `poc-untouched` before any source change.  This check
# only READS the POC; it never writes into it.  `cargo test` writes build
# output below the POC's `target/`, which the ticket's own .gitignore excludes.
# Override the location with HADROPOC=/path for reproduction.
#
# Acceptance criteria and their mechanical encoding
# -------------------------------------------------
# AC1  `git tag` lists poc-untouched and `git cat-file -t poc-untouched`
#      prints `tag` (annotated tag, not a lightweight one).
#
# AC2  No tracked path is under `target/` and none is under
#      `experiments/<dir>/` (i.e. no tracked path matching
#      `experiments/[^/]+/...`).  `find`-style paths directly inside
#      experiments/ such as `experiments/chain.sh` and `experiments/run1.log`
#      stay tracked: the ignore rule is `experiments/*/`, which matches
#      directories only.  We additionally assert the behaviour of that rule
#      with `git check-ignore` on synthetic probe paths.
#
# AC3  The tagged commit is a root commit and, relative to the pre-ticket
#      tree, the only new path is `.gitignore`.  Chosen encoding:
#        (a) `git rev-list --parents -n 1 poc-untouched^{commit}` prints
#            exactly one hash (the commit has no parent);
#        (b) `git show --name-status poc-untouched` (root commit => diff
#            against the empty tree, every path `A`) contains `.gitignore`
#            as added;
#        (c) the set of paths added by that commit equals
#            (pre-ticket inventory ∪ {.gitignore}) — nothing missing,
#            nothing extra.  The pre-ticket inventory is frozen at RED time
#            in `checks/003-01-pre-ticket.sha256`;
#        (d) source-content invariant: every pre-ticket file still has the
#            SHA-256 recorded at RED time (`sha256sum -c`), so no source
#            file was modified before being committed (DoD "no source file
#            of the POC differs from its pre-ticket content");
#        (e) the working tree equals the tagged tree: `git status --porcelain`
#            is empty and `git diff --quiet poc-untouched` succeeds (no
#            untracked, staged, unstaged or deleted paths).  Ignored dirs
#            (target/, experiments/<dir>/) are outside this comparison by
#            construction.
#      Note: with a root commit `git show --name-status` necessarily lists
#      every path as `A`; "only .gitignore is new" is therefore expressed by
#      (c), the set difference against the frozen pre-ticket inventory.
#
# AC4  `cargo test` at poc-untouched runs the two transport tests and both
#      pass.  We run `cargo test transport` in the POC's working tree, which
#      AC3(e) has just proven identical to the tagged tree, and require:
#        - exit status 0;
#        - one `running 2 tests` line;
#        - exactly two lines `test transport::tests::<name> ... ok`;
#        - `test result: ok. 2 passed; 0 failed; ...`.
#      The filter keeps this exactly the two transport tests even if another
#      test module is added later; a compile failure is a failure, never
#      masked.  The first run may compile for a few minutes.
#
# Exit status: 0 all criteria pass, 1 otherwise (one FAIL line per criterion).

set -u
export LC_ALL=C

POC="${HADROPOC:-/home/dorian/workspace/hadropoc}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MANIFEST="$HERE/003-01-pre-ticket.sha256"
TMPROOT="${TMPDIR:-/tmp/opencode}"
mkdir -p "$TMPROOT" 2>/dev/null || TMPROOT=/tmp
TMP="$(mktemp -d "$TMPROOT/003-01.XXXXXX")" || { echo "FAIL  cannot create temp dir"; exit 1; }
trap 'rm -rf "$TMP"' EXIT

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
note() { printf '      %s\n' "$*"; }

gitpoc() { git -C "$POC" "$@"; }

echo "== 003-01: POC repository and poc-untouched tag =="
echo "   POC:      $POC"
echo "   manifest: $MANIFEST"

# ---------------------------------------------------------------- precheck
if [ ! -d "$POC" ]; then
  bad "precondition: $POC does not exist"
  echo "== 003-01 FAILED =="
  exit 1
fi
if ! gitpoc rev-parse --git-dir >/dev/null 2>&1; then
  bad "precondition: $POC is not a git repository (git init is the green-phase work of this ticket)"
  bad "AC1/AC2/AC3/AC4 not evaluated: no repository to inspect"
  echo "== 003-01 FAILED =="
  exit 1
fi

# -------------------------------------------------------------------- AC 1
echo "-- AC1 annotated tag poc-untouched"
if gitpoc tag --list | grep -Fxq 'poc-untouched'; then
  pass "AC1 git tag lists poc-untouched"
else
  bad "AC1 git tag does not list poc-untouched"
fi

objtype="$(gitpoc cat-file -t 'poc-untouched' 2>/dev/null || true)"
if [ "$objtype" = "tag" ]; then
  pass "AC1 poc-untouched is annotated (cat-file -t = tag)"
else
  bad "AC1 poc-untouched is not an annotated tag (cat-file -t = '${objtype:-<error>}')"
fi

# -------------------------------------------------------------------- AC 2
echo "-- AC2 ignored trees are not tracked"
gitpoc ls-files > "$TMP/ls-files" 2>"$TMP/ls-files.err" \
  || bad "AC2 git ls-files failed: $(head -n 1 "$TMP/ls-files.err")"

if [ -s "$TMP/ls-files" ]; then
  pass "AC2 git ls-files is non-empty ($(wc -l < "$TMP/ls-files") paths)"
else
  bad "AC2 git ls-files is empty: the source tree is not committed"
fi

target_hits="$(grep -E '^target/' "$TMP/ls-files" || true)"
if [ -z "$target_hits" ]; then
  pass "AC2 no tracked path under target/"
else
  bad "AC2 tracked paths under target/:"
  printf '%s\n' "$target_hits" | head -n 5 | while IFS= read -r p; do note "$p"; done
fi

exp_hits="$(grep -E '^experiments/[^/]+/' "$TMP/ls-files" || true)"
if [ -z "$exp_hits" ]; then
  pass "AC2 no tracked path under experiments/<dir>/"
else
  bad "AC2 tracked paths under experiments/<dir>/:"
  printf '%s\n' "$exp_hits" | head -n 5 | while IFS= read -r p; do note "$p"; done
fi

# Work step 2, observed behaviourally: both trees are ignored by a rule.
for probe in "target/003-01-probe" "experiments/003-01-probe/file"; do
  if gitpoc check-ignore -q -- "$probe"; then
    pass "AC2 .gitignore ignores $probe"
  else
    bad "AC2 .gitignore does not ignore $probe (target/ and experiments/*/ must be excluded)"
  fi
done

# -------------------------------------------------------------------- AC 3
echo "-- AC3 root commit introducing only .gitignore"
tag_commit='poc-untouched^{commit}'
if ! gitpoc rev-parse --verify -q "$tag_commit" >/dev/null 2>&1; then
  bad "AC3 poc-untouched does not resolve to a commit"
else
  # (a) root commit: exactly one field, the hash itself.
  parents="$(gitpoc rev-list --parents -n 1 "$tag_commit" 2>/dev/null | tr '\n' ' ')"
  # shellcheck disable=SC2086
  set -- $parents
  if [ "$#" -eq 1 ]; then
    pass "AC3 tagged commit is a root commit (no parent)"
  else
    bad "AC3 tagged commit has $(( $# - 1 )) parent(s); expected none"
  fi

  # (b)+(c) added path set vs frozen pre-ticket inventory.
  manifest_paths="$TMP/manifest-paths"
  if [ -f "$MANIFEST" ]; then
    awk '{ $1=""; sub(/^[ \t]+/, ""); print }' "$MANIFEST" | sed '/^$/d' | sort -u > "$manifest_paths"
    pass "AC3 pre-ticket inventory loaded ($(wc -l < "$manifest_paths") paths)"
  else
    : > "$manifest_paths"
    bad "AC3 missing pre-ticket inventory $MANIFEST"
  fi

  gitpoc show --name-status --format= "$tag_commit" > "$TMP/name-status" 2>"$TMP/show.err" \
    || bad "AC3 git show --name-status failed: $(head -n 1 "$TMP/show.err")"
  awk -F'\t' '$1 ~ /^A/ { print $2 }' "$TMP/name-status" | sort -u > "$TMP/added"

  if grep -Fxq '.gitignore' "$TMP/added"; then
    pass "AC3 .gitignore is added by the tagged commit"
  else
    bad "AC3 .gitignore is not added by the tagged commit"
  fi

  extra="$(comm -23 "$TMP/added" "$manifest_paths")"
  missing="$(comm -13 "$TMP/added" "$manifest_paths")"
  if [ "$extra" = ".gitignore" ] && [ -z "$missing" ]; then
    pass "AC3 added paths = pre-ticket inventory + .gitignore (nothing else new, nothing lost)"
  else
    [ -n "$missing" ] && { bad "AC3 pre-ticket paths missing from the commit:"; printf '%s\n' "$missing" | head -n 5 | while IFS= read -r p; do note "$p"; done; }
    if [ -z "$extra" ]; then
      bad "AC3 .gitignore missing from added set"
    elif [ "$extra" != ".gitignore" ]; then
      bad "AC3 untracked-at-RED paths committed besides .gitignore:"
      printf '%s\n' "$extra" | grep -Fxv '.gitignore' | head -n 5 | while IFS= read -r p; do note "$p"; done
    fi
  fi

  # (d) content invariant: no pre-ticket source file was modified.
  if [ -f "$MANIFEST" ]; then
    if ( cd "$POC" && sha256sum -c "$MANIFEST" ) > "$TMP/hashcheck" 2>&1; then
      pass "AC3 every pre-ticket file matches its RED-time SHA-256"
    else
      bad "AC3 pre-ticket file content changed:"
      grep -E 'FAILED|No such file' "$TMP/hashcheck" | head -n 10 | while IFS= read -r l; do note "$l"; done
    fi
  fi

  # (e) working tree equals the tagged tree (ignored dirs excluded).
  porcelain="$(gitpoc status --porcelain 2>&1)"
  if [ -z "$porcelain" ]; then
    pass "AC3 working tree clean vs HEAD (git status --porcelain empty)"
  else
    bad "AC3 working tree not clean:"
    printf '%s\n' "$porcelain" | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi

  gitpoc diff --quiet "$tag_commit" -- >/dev/null 2>&1
  rc=$?
  if [ "$rc" -eq 0 ]; then
    pass "AC3 working tree identical to poc-untouched (git diff --quiet)"
  else
    bad "AC3 working tree differs from poc-untouched (git diff exit $rc)"
    gitpoc diff --stat "$tag_commit" -- 2>/dev/null | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi
fi

# -------------------------------------------------------------------- AC 4
echo "-- AC4 two transport tests pass"
if ! command -v cargo >/dev/null 2>&1; then
  bad "AC4 cargo not found in PATH"
elif [ ! -f "$POC/Cargo.toml" ]; then
  bad "AC4 $POC/Cargo.toml not found"
else
  echo "   running: (cd $POC && cargo test transport)  [may compile]"
  ( cd "$POC" && CARGO_TERM_COLOR=never cargo test transport ) > "$TMP/cargo.log" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ]; then
    bad "AC4 cargo test transport failed (exit $rc)"
    tail -n 20 "$TMP/cargo.log" | while IFS= read -r l; do note "$l"; done
  else
    if grep -Fxq 'running 2 tests' "$TMP/cargo.log"; then
      pass "AC4 cargo reports exactly 2 tests in the transport filter"
    else
      bad "AC4 expected 'running 2 tests', got: $(grep -E '^running [0-9]+ tests?$' "$TMP/cargo.log" | head -n 1)"
    fi

    names="$(grep -E '^test transport::tests::[A-Za-z0-9_]+ \.\.\. ok$' "$TMP/cargo.log" || true)"
    n="$(printf '%s\n' "$names" | sed '/^$/d' | wc -l)"
    if [ "$n" -eq 2 ]; then
      pass "AC4 both transport tests pass:"
      printf '%s\n' "$names" | while IFS= read -r l; do note "$l"; done
    else
      bad "AC4 expected 2 passing transport tests, found $n"
      grep -E '^test transport::' "$TMP/cargo.log" | head -n 10 | while IFS= read -r l; do note "$l"; done
    fi

    if grep -Eq '^test result: ok\. 2 passed; 0 failed;' "$TMP/cargo.log"; then
      pass "AC4 test result: ok. 2 passed; 0 failed"
    else
      bad "AC4 no 'test result: ok. 2 passed; 0 failed' line"
      grep -E '^test result:' "$TMP/cargo.log" | head -n 5 | while IFS= read -r l; do note "$l"; done
    fi
  fi
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 003-01 PASSED =="
  exit 0
fi
echo "== 003-01 FAILED =="
exit 1
