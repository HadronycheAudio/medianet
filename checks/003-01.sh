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
# This check runs permanently in the per-change suite.  It therefore encodes
# the standing invariant of the frozen baseline, not a snapshot of the tree:
#   1. `poc-untouched` remains an annotated tag on the POC's root commit;
#   2. that commit's tree is still the untouched pre-ticket tree (verified
#      against the immutable tag object, so later epic-003 fixes cannot
#      invalidate it);
#   3. every commit after it is a sanctioned epic-003 change: each commit
#      message's first line references a 003-XX ticket id;
#   4. the POC working tree is clean vs HEAD, except while an orchestrated
#      epic-003 write is in flight.  For that window the orchestrator
#      commits `swarm/poc-write-in-flight` in THIS repository; when present,
#      the tree-cleanliness assertion alone is SKIPPED (loudly) and never
#      fails.  State is restored between commits, so the assertion is
#      meaningful again once the marker is removed.
# Epic 003 is the only task allowed to modify the POC, hence rule 3.
# Override the marker location with MARKER_PATH=/path.
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
# AC3  `poc-untouched` is the POC's root commit and its tree is the untouched
#      pre-ticket tree; commits made after it are sanctioned epic-003 changes.
#      Chosen encoding:
#        (a) `git rev-list --parents -n 1 poc-untouched^{commit}` prints
#            exactly one hash (the commit has no parent);
#        (b) `git show --name-status poc-untouched` (root commit => diff
#            against the empty tree, every path `A`) contains `.gitignore`
#            as added;
#        (c) the set of paths added by that commit equals
#            (pre-ticket inventory ∪ {.gitignore}) — nothing missing,
#            nothing extra.  The pre-ticket inventory is frozen at RED time
#            in `checks/003-01-pre-ticket.sha256`;
#        (d) tag integrity: the SHA-256 of `git cat-file blob
#            poc-untouched:<path>` equals the RED-time record for every
#            manifest path.  This reads the immutable tag object, never the
#            working files, so it remains valid while epic 003 edits the
#            tree;
#        (e) history convention: every commit reachable from HEAD but not
#            from `poc-untouched` has a first line matching `003-[0-9][0-9]`
#            (epic 003 is the only task allowed to change the POC).  A
#            root-only history passes vacuously;
#        (f) tree cleanliness: `git status --porcelain` is empty, i.e. HEAD
#            is committed and nothing but ignored paths is untracked.  When
#            the orchestrator marker `swarm/poc-write-in-flight` exists in
#            this repository (or at $MARKER_PATH), this single assertion is
#            reported as SKIP and does not fail: a serialised epic-003
#            write is in flight and the tree is expected to be dirty.
#      Note: with a root commit `git show --name-status` necessarily lists
#      every path as `A`; "only .gitignore is new" is therefore expressed by
#      (c), the set difference against the frozen pre-ticket inventory.
#
# AC4  `cargo test` runs the two transport tests and both pass.  Since the
#      POC may be mid-fix (epic 003), this is evaluated on the working tree
#      as-is: during an in-flight write it may fail, and that failure is
#      honest — passing it is the fix's own requirement.  We run
#      `cargo test transport` and require:
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
REPO_ROOT="$(cd "$HERE/.." && pwd)"
MANIFEST="$HERE/003-01-pre-ticket.sha256"
MARKER_PATH="${MARKER_PATH:-$REPO_ROOT/swarm/poc-write-in-flight}"
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
echo "   marker:   $MARKER_PATH"

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
echo "-- AC3 pristine root tag, sanctioned post-tag history, clean tree"
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

  # (d) tag integrity: hash the blobs recorded by the immutable tag, not the
  # working files.  Later epic-003 commits cannot change these blobs.
  if [ -f "$MANIFEST" ]; then
    : > "$TMP/hash-bad"
    verified=0
    while IFS= read -r record; do
      [ -n "$record" ] || continue
      want="$(printf '%s\n' "$record" | awk '{ print $1 }')"
      path="$(printf '%s\n' "$record" | awk '{ $1=""; sub(/^[ \t]+/, ""); print }')"
      if [ -z "$path" ]; then
        printf '(unparsable manifest record)\n' >> "$TMP/hash-bad"
        continue
      fi
      if ! gitpoc cat-file -e "poc-untouched:$path" 2>/dev/null; then
        printf '%s (missing from tag)\n' "$path" >> "$TMP/hash-bad"
        continue
      fi
      got="$(gitpoc cat-file blob "poc-untouched:$path" 2>/dev/null | sha256sum | awk '{ print $1 }')"
      if [ "$want" = "$got" ]; then
        verified=$((verified + 1))
      else
        printf '%s (sha256 %s != recorded %s)\n' "$path" "$got" "$want" >> "$TMP/hash-bad"
      fi
    done < "$MANIFEST"
    if [ -s "$TMP/hash-bad" ]; then
      bad "AC3 poc-untouched tag tree does not match the RED-time inventory:"
      head -n 10 "$TMP/hash-bad" | while IFS= read -r l; do note "$l"; done
    else
      pass "AC3 poc-untouched tag blobs match the RED-time SHA-256 ($verified files)"
    fi
  fi

  # (e) every commit after the root is a sanctioned epic-003 change.
  if ! gitpoc rev-parse --verify -q HEAD >/dev/null 2>&1; then
    bad "AC3 POC HEAD does not resolve to a commit"
  else
    : > "$TMP/bad-commits"
    post_count=0
    for c in $(gitpoc rev-list HEAD --not "$tag_commit" 2>/dev/null); do
      post_count=$((post_count + 1))
      subject="$(gitpoc log -1 --format=%s "$c" 2>/dev/null)"
      if ! printf '%s' "$subject" | grep -Eq '003-[0-9]{2}'; then
        printf '%s %s\n' "$(printf '%s' "$c" | cut -c1-12)" "$subject" >> "$TMP/bad-commits"
      fi
    done
    if [ -s "$TMP/bad-commits" ]; then
      bad "AC3 post-poc-untouched commits without a 003-XX ticket id:"
      head -n 10 "$TMP/bad-commits" | while IFS= read -r l; do note "$l"; done
    elif [ "$post_count" -eq 0 ]; then
      pass "AC3 no commits after poc-untouched (root-only history; convention holds vacuously)"
    else
      pass "AC3 all $post_count commit(s) after poc-untouched reference a 003-XX ticket"
    fi
  fi

  # (f) tree cleanliness vs HEAD, skipped for the orchestrated write window.
  if [ -e "$MARKER_PATH" ]; then
    printf 'SKIP: epic-003 write in flight (orchestrator marker present)\n'
    note "marker: $MARKER_PATH"
    note "AC3 tree-cleanliness assertion skipped; this is not a failure"
  else
    porcelain="$(gitpoc status --porcelain 2>&1)"
    if [ -z "$porcelain" ]; then
      pass "AC3 working tree clean vs HEAD (git status --porcelain empty)"
    else
      bad "AC3 working tree not clean:"
      printf '%s\n' "$porcelain" | head -n 5 | while IFS= read -r l; do note "$l"; done
    fi
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
