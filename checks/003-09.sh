#!/usr/bin/env bash
# checks/003-09.sh — acceptance check for ticket 003-09
# "Verify and tag poc-baseline" (epic 003, deliverable 6 of the POC).
#
# The deliverable lives outside this repository: the proof of concept at
# /home/dorian/workspace/hadropoc (the "POC") must carry, after the three fix
# commits (003-03, 003-04, 003-06), the smoke script (003-07) and BASELINE.md
# (003-08), exactly one RECORD commit anchored on 003-09, and the annotated tag
# `poc-baseline` on that commit.  The RECORD commit is EMPTY (it changes no
# file) and its BODY carries the results of the measuring-machine runs: the
# cargo test result and the three tools/smoke.sh runs.  The three runs are the
# coder's work and cannot be re-run from history, so this check never runs the
# POC: it only READS the POC git history, objects and files.  No cargo, no
# hadropoc, no tools/smoke.sh.  It never writes into the POC.  Override the
# location with HADROPOC=/path (used for the throwaway fixture clones only).
#
# RULINGS applied (lead, 003-09 brief; L1-L6):
#   L1  the results are recorded in ONE final commit made by this ticket: an
#       EMPTY commit (the tree equals the tree after the last 003-08 commit),
#       subject `chore(003-09): ...`, body with the cargo test result (both
#       transport tests named and passing), three `PASS smoke:` lines, three
#       distinct --group ports, the synth order of each run, the machine
#       identity and "unprivileged".  A repeated or retried run is refused.
#   L2  both tags are annotated; poc-baseline points at the 003-09 record commit
#       and its message is non-empty and names 003-09.
#   L3  the diff between the tags touches exactly the six allowed paths, and
#       each of them: src/lib.rs, src/metrics.rs, src/module.rs (003-03 and
#       003-04), src/main.rs (003-06), tools/smoke.sh (003-07), BASELINE.md
#       (003-08).  That set equals the union of the files the fix commits touch.
#   L4  freeze, PERMANENT: HEAD equals poc-baseline^{commit}.  Nothing may be
#       committed to the POC after the tag.  The working tree is clean; while
#       the orchestrator marker swarm/poc-write-in-flight exists in the shared
#       checkout, ONLY the cleanliness assertion is skipped, loudly.
#   L5  the tag's tree contains BASELINE.md and tools/smoke.sh (mode 100755),
#       and BASELINE.md names both tags.
#   L6  no behavioural assertion runs anything: read-only on git and files.
#
# Ticket-id rule (same as 003-01..003-08): a subject belongs to ticket 003-NN
# when it matches `^003-NN([: ]|$)` or `^[a-z]+\(003-NN\)!?: `, and every commit
# after poc-untouched must carry such an id.  No dependence on any `main` ref.
# No sibling loop.  Nothing here hard-codes a commit hash, a host, a kernel or
# a port: the three runs are validated by their form, not by their values.
#
# ASSERTIONS
#   AC0 history: poc-untouched is an ancestor of HEAD; every commit after it has
#       an anchored id; exactly one commit after it is the 003-09 record commit
#       (subject `chore(003-09): ...`).
#   AC1 the record commit changes no file: it has one parent, its tree equals
#       the parent's tree, and its tree equals the tree after the last 003-08
#       commit.
#   AC2 the record commit's body (python, read from git): exactly three
#       `PASS smoke:` lines, each ending `synth1 and synth2 verified` (the
#       line tools/smoke.sh prints at HEAD); exactly three `--group
#       239.72.68.44:<port>` occurrences with three distinct ports; exactly
#       three synth-order lines (`voices on synth1 ... also synth2`, or the
#       swap); both transport tests named with ok/passed; the word
#       "unprivileged"; `CONFIG_HZ=250`; a host and a kernel named; each PASS
#       line anchored at the line start with N/N windows exact and N > 0; the
#       cargo result line "0 failed" (the only failed word allowed); and no
#       word of a failed, replaced, repeated, retried, again, fourth or
#       attempt run (the statement "no run was repeated" is allowed).
#       VALUES: the form is checked here; the values were verified once, by
#       review of PR #28 against the run logs in /tmp/opencode/003-09/.
#   AC3 tags: poc-untouched and poc-baseline are listed; both are annotated
#       (cat-file -t prints tag); poc-baseline^{commit} is the record commit;
#       the poc-baseline message is non-empty and names 003-09.
#   AC4 the diff between the tags: `git diff --name-only poc-untouched
#       poc-baseline` equals the six allowed paths, every one of them is
#       touched, and that set equals the union of the files of the anchored
#       003-03, 003-04, 003-06, 003-07 and 003-08 commits.
#   AC5 consistency: BASELINE.md exists at the tag; tools/smoke.sh at the tag
#       is mode 100755; BASELINE.md at the tag names poc-untouched and
#       poc-baseline.
#   AC6 freeze, PERMANENT: HEAD equals poc-baseline^{commit} (a detached HEAD at
#       the tag passes); no commit reachable from ANY ref (branches, tags,
#       remotes, stash, notes) lies outside the tag's history; every local
#       branch points at the tag's commit; the working tree is clean (loud SKIP
#       of this one assertion while swarm/poc-write-in-flight exists).  Every
#       git command runs with --no-optional-locks so the frozen .git/index is
#       never rewritten.
#
# Exit status: 0 when every assertion passes, 1 otherwise (one FAIL line per
# failed assertion).

set -u
export LC_ALL=C

SHARED_ROOT="${SHARED_ROOT:-/home/dorian/workspace/hadronyche/medianet}"
MARKER_PATH="${MARKER_PATH:-$SHARED_ROOT/swarm/poc-write-in-flight}"
POC="${HADROPOC:-/home/dorian/workspace/hadropoc}"
UNTOUCHED="poc-untouched"
BASELINE_TAG="poc-baseline"
TICKET="003-09"
DOC="BASELINE.md"
SMOKE="tools/smoke.sh"
ALLOWED="BASELINE.md src/lib.rs src/main.rs src/metrics.rs src/module.rs tools/smoke.sh"
TMPROOT="${TMPDIR:-/tmp/opencode}"
mkdir -p "$TMPROOT" 2>/dev/null || TMPROOT=/tmp
TMP="$(mktemp -d "$TMPROOT/003-09.XXXXXX")" || { echo "FAIL  cannot create temp dir"; exit 1; }
trap 'rm -rf "$TMP"' EXIT

fail=0
pass() { printf 'PASS  %s\n' "$*"; }
bad()  { printf 'FAIL  %s\n' "$*"; fail=1; }
skip() { printf 'SKIP  %s\n' "$*"; }
note() { printf '      %s\n' "$*"; }

# --no-optional-locks: no git command may refresh and rewrite the frozen POC's .git/index (T3).
gitpoc() { git --no-optional-locks -C "$POC" "$@"; }
# Anchored ticket-id rule (same as 003-01..003-08).
ID_ANY='^[0-9a-f]+ (003-[0-9][0-9]([: ]|$)|[a-z]+\(003-[0-9][0-9]\)!?: )'

# Commits after poc-untouched whose subject is anchored on ticket $1, oldest first.
ticket_commits() {
  gitpoc log --reverse --format='%H%x09%s' "$UNTOUCHED..HEAD" 2>/dev/null \
    | awk -F'\t' -v id="$1" '$2 ~ ("^" id "([: ]|$)|^[a-z]+[(]" id "[)]!?: ") { print $1 }'
}

echo "== 003-09: verified and tagged baseline (poc-baseline in the POC) =="
echo "   POC:    $POC"
echo "   marker: $MARKER_PATH"

# ---------------------------------------------------------------- precheck
if [ ! -d "$POC" ] || ! gitpoc rev-parse --git-dir >/dev/null 2>&1; then
  bad "precondition: $POC is not a git repository"
  echo "== 003-09 FAILED =="
  exit 1
fi
if ! gitpoc rev-parse --verify -q "$UNTOUCHED^{commit}" >/dev/null 2>&1; then
  bad "precondition: tag $UNTOUCHED does not resolve in the POC"
  echo "== 003-09 FAILED =="
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  bad "precondition: python3 not found in PATH (needed for the body checks)"
  echo "== 003-09 FAILED =="
  exit 1
fi

# -------------------------------------------------------------- AC 0
echo "-- AC0 history: poc-untouched ancestor, anchored ids, one 003-09 record commit"
if gitpoc merge-base --is-ancestor "$UNTOUCHED" HEAD 2>/dev/null; then
  pass "AC0 $UNTOUCHED is an ancestor of HEAD"
else
  bad "AC0 $UNTOUCHED is not an ancestor of HEAD"
fi
bad_ids="$(gitpoc log --format='%h %s' "$UNTOUCHED..HEAD" 2>/dev/null | grep -Ev "$ID_ANY" || true)"
if [ -n "$bad_ids" ]; then
  bad "AC0 commits after $UNTOUCHED without a 003-XX id in the first line:"
  printf '%s\n' "$bad_ids" | head -n 5 | while IFS= read -r l; do note "$l"; done
else
  pass "AC0 every commit after $UNTOUCHED references a 003-XX ticket"
fi

REC_ALL="$(ticket_commits "$TICKET")"
REC_N="$(printf '%s\n' "$REC_ALL" | sed '/^$/d' | wc -l | tr -d ' ')"
REC=""
if [ "$REC_N" -eq 1 ]; then
  REC="$REC_ALL"
  subj="$(gitpoc log -1 --format='%s' "$REC")"
  if printf '%s\n' "$subj" | grep -Eq '^chore\(003-09\)!?: .+'; then
    pass "AC0 the 003-09 record commit is ${REC:0:12}: $subj"
  else
    bad "AC0 the 003-09 record commit subject must be 'chore(003-09): ...', is '$subj'"
  fi
elif [ "$REC_N" -eq 0 ]; then
  bad "AC0 no commit after $UNTOUCHED is anchored on 003-09 (the record commit with the results is not made)"
else
  bad "AC0 $REC_N commits after $UNTOUCHED are anchored on 003-09; exactly one record commit is allowed"
fi

# -------------------------------------------------------------- AC 1
echo "-- AC1 the record commit changes no file (empty commit)"
if [ -z "$REC" ]; then
  bad "AC1 not evaluated: no 003-09 record commit"
else
  nparents="$(gitpoc rev-list --parents -n 1 "$REC" 2>/dev/null | awk '{ print NF - 1 }')"
  if [ "$nparents" = "1" ]; then
    pass "AC1 the record commit has one parent"
  else
    bad "AC1 the record commit has $nparents parent(s); expected one"
  fi
  changed="$(gitpoc show --name-only --format= "$REC" 2>/dev/null | sed '/^$/d')"
  if [ -z "$changed" ]; then
    pass "AC1 the record commit changes no file (git show --name-only is empty)"
  else
    bad "AC1 the record commit changes files; it must be empty:"
    printf '%s\n' "$changed" | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi
  rec_tree="$(gitpoc rev-parse "$REC^{tree}" 2>/dev/null)"
  par_tree="$(gitpoc rev-parse "$REC^1^{tree}" 2>/dev/null)"
  if [ -n "$rec_tree" ] && [ "$rec_tree" = "$par_tree" ]; then
    pass "AC1 the record commit's tree equals its parent's tree"
  else
    bad "AC1 the record commit's tree differs from its parent's tree"
  fi
  last08="$(ticket_commits 003-08 | tail -n 1)"
  if [ -n "$last08" ] && [ "$(gitpoc rev-parse "$last08^{tree}" 2>/dev/null)" = "$rec_tree" ]; then
    pass "AC1 the tree at the record commit equals the tree after the last 003-08 commit (${last08:0:12})"
  else
    bad "AC1 the tree at the record commit differs from the tree after the last 003-08 commit"
  fi
fi

# -------------------------------------------------------------- AC 2
echo "-- AC2 the record commit body: cargo test, three smoke runs, machine, unprivileged"
if [ -z "$REC" ]; then
  bad "AC2 not evaluated: no 003-09 record commit"
else
  gitpoc show -s --format='%B' "$REC" > "$TMP/body.txt" 2>/dev/null
  cat > "$TMP/body-check.py" <<'PY'
import re
import sys

text = open(sys.argv[1], encoding="utf-8").read()
lines = text.split("\n")
results = []


def ok(msg):
    results.append(("PASS", msg))


def bad(msg):
    results.append(("FAIL", msg))


# Three smoke runs: the exact line tools/smoke.sh prints at HEAD (its last line).
SMOKE_RE = re.compile(
    r"^PASS smoke: exit 0, metrics well formed, (\d+)/(\d+) windows exact, 0 lost, "
    r"clock reached LOCKED, synth1 and synth2 verified\s*$")
pass_lines = [l for l in lines if "PASS smoke:" in l]
good = []
for l in pass_lines:
    m = SMOKE_RE.match(l)
    if m and m.group(1) == m.group(2) and int(m.group(1)) > 0:
        good.append(l)
if len(pass_lines) == 3 and len(good) == 3:
    ok("AC2 three 'PASS smoke:' lines, each anchored at the line start, N/N windows exact with N > 0, ending 'synth1 and synth2 verified'")
else:
    bad("AC2 expected exactly three 'PASS smoke:' lines ending 'synth1 and synth2 verified'; found %d 'PASS smoke:' line(s), %d well formed"
        % (len(pass_lines), len(good)))
    for l in pass_lines[:5]:
        print("      " + l.strip())

# Group ports: three runs, three different --group ports on the test network.
ports = re.findall(r"--group\s+239\.72\.68\.44:(\d+)", text)
if len(ports) == 3 and len(set(ports)) == 3:
    ok("AC2 three runs with three distinct --group ports: %s" % ", ".join(ports))
else:
    bad("AC2 expected exactly three '--group 239.72.68.44:<port>' occurrences with three distinct ports; found %s"
        % (ports if ports else "none"))

# Synth order of each run: the main section names one synth, 'also' the other.
ORDER_RE = re.compile(r"voices on (synth[12])\b.*?\balso (synth[12])\b")
order_lines = [l for l in lines if re.search(r"voices on synth[12]", l)]
order_ok = [l for l in order_lines
            if ORDER_RE.search(l) and {ORDER_RE.search(l).group(1), ORDER_RE.search(l).group(2)} == {"synth1", "synth2"}]
if len(order_lines) == 3 and len(order_ok) == 3:
    ok("AC2 synth order stated for each run (voices on synth1 / also synth2, or swapped)")
else:
    bad("AC2 expected exactly three synth-order lines 'voices on synth1 ... also synth2' (or swapped); found %d, %d valid"
        % (len(order_lines), len(order_ok)))

# cargo test: both transport tests named, with ok or passed.
for name in ("time_code", "scenario"):
    hits = [l for l in lines
            if re.search(r"transport::tests::%s\b" % name, l) and re.search(r"\b(ok|passed)\b", l)]
    if hits:
        ok("AC2 cargo test: transport::tests::%s named with ok/passed" % name)
    else:
        bad("AC2 cargo test: no line naming transport::tests::%s with ok or passed" % name)

if re.search(r"\bunprivileged\b", text, re.I):
    ok("AC2 the runs are stated unprivileged")
else:
    bad("AC2 the body does not state 'unprivileged'")
if re.search(r"CONFIG_HZ=250\b", text):
    ok("AC2 machine identity: CONFIG_HZ=250 (P-03)")
else:
    bad("AC2 the body does not state CONFIG_HZ=250 (P-03)")
if re.search(r"\bhost\b", text, re.I) and re.search(r"\bkernel\b", text, re.I):
    ok("AC2 machine identity: host and kernel named")
else:
    bad("AC2 the body does not name the host and the kernel of the measuring machine")

# No repeated, failed, replaced or extra run.  Two exemptions only: the sentence
# "no run was repeated", and the cargo result line with exactly "0 failed"
# (asserted to be 0 and to be present for the two transport tests).
cargo_zero = re.findall(r"test result: ok\. \d+ passed; 0 failed\b", text)
if cargo_zero:
    ok("AC2 cargo test result line reports 0 failed")
else:
    bad("AC2 no cargo test result line 'test result: ok. <n> passed; 0 failed'")
stripped = re.sub(r"test result: ok\. \d+ passed; 0 failed\b", "", text)
stripped = re.sub(r"\bno run (?:was |has been )?(?:repeated|retried|re-?run)\b|\bnone (?:repeated|retried)\b",
                  "", stripped, flags=re.I)
WORDS = r"fail\w*|replac\w*|again\b|fourth|attempt\w*|repeat\w*|retr\w*|re-?run\w*|re-?ran\b"
hits = re.findall(WORDS, stripped, re.I)
if hits:
    bad("AC2 the body mentions a failed, replaced, repeated, retried or extra run (%s); the three runs must be the first three and pass"
        % ", ".join(sorted(set(h.lower() for h in hits))))
else:
    ok("AC2 no failed, replaced, repeated, retried or extra run is mentioned")

for status, msg in results:
    print("%s  %s" % (status, msg))
sys.exit(1 if any(s == "FAIL" for s, _ in results) else 0)
PY
  python3 -I "$TMP/body-check.py" "$TMP/body.txt" > "$TMP/body-out" 2>&1
  rc=$?
  cat "$TMP/body-out"
  if [ "$rc" -ne 0 ]; then
    fail=1
  fi
fi

# -------------------------------------------------------------- AC 3
echo "-- AC3 tags: poc-untouched and poc-baseline, annotated, poc-baseline on the record commit"
if gitpoc tag --list | grep -Fxq "$UNTOUCHED"; then
  pass "AC3 git tag lists $UNTOUCHED"
else
  bad "AC3 git tag does not list $UNTOUCHED"
fi
HAVE_TAG=0
if gitpoc tag --list | grep -Fxq "$BASELINE_TAG"; then
  HAVE_TAG=1
  pass "AC3 git tag lists $BASELINE_TAG"
else
  bad "AC3 git tag does not list $BASELINE_TAG (ticket 003-09 creates it on the record commit)"
fi
for t in "$UNTOUCHED" "$BASELINE_TAG"; do
  if ! gitpoc tag --list | grep -Fxq "$t"; then
    continue
  fi
  objtype="$(gitpoc cat-file -t "$t" 2>/dev/null || true)"
  if [ "$objtype" = "tag" ]; then
    pass "AC3 $t is annotated (cat-file -t = tag)"
  else
    bad "AC3 $t is not annotated (cat-file -t = '${objtype:-<error>}')"
  fi
done
if [ "$HAVE_TAG" -eq 1 ]; then
  tag_commit="$(gitpoc rev-parse -q --verify "$BASELINE_TAG^{commit}" 2>/dev/null || true)"
  if [ -z "$REC" ]; then
    bad "AC3 $BASELINE_TAG cannot be matched: no 003-09 record commit"
  elif [ "$tag_commit" = "$REC" ]; then
    pass "AC3 $BASELINE_TAG points at the 003-09 record commit (${REC:0:12})"
  else
    bad "AC3 $BASELINE_TAG points at ${tag_commit:0:12}, the 003-09 record commit is ${REC:0:12}"
  fi
  tag_msg="$(gitpoc tag -l --format='%(contents)' "$BASELINE_TAG" 2>/dev/null | sed '/^[[:space:]]*$/d')"
  if [ -n "$tag_msg" ] && printf '%s\n' "$tag_msg" | grep -q "$TICKET"; then
    pass "AC3 $BASELINE_TAG message is non-empty and names $TICKET"
  else
    bad "AC3 $BASELINE_TAG message is empty or does not name $TICKET"
  fi
else
  skip "AC3 $BASELINE_TAG commit and message not evaluated: the tag is absent"
fi

# -------------------------------------------------------------- AC 4
echo "-- AC4 diff between the tags: exactly the six allowed paths, equal to the fix commits' files"
if [ "$HAVE_TAG" -eq 1 ]; then
  gitpoc diff --name-only --no-renames "$UNTOUCHED" "$BASELINE_TAG" 2>/dev/null | sort -u > "$TMP/diff-paths"
  : > "$TMP/fix-paths"
  for t in 003-03 003-04 003-06 003-07 003-08; do
    for c in $(ticket_commits "$t"); do
      gitpoc show --name-only --format= "$c" 2>/dev/null >> "$TMP/fix-paths"
    done
  done
  sed '/^$/d' "$TMP/fix-paths" | sort -u > "$TMP/fix-paths-u"
  printf '%s\n' $ALLOWED | sort -u > "$TMP/allowed"
  echo "   diff $UNTOUCHED..$BASELINE_TAG: $(tr '\n' ' ' < "$TMP/diff-paths")"
  extra="$(comm -23 "$TMP/diff-paths" "$TMP/allowed" | sed '/^$/d')"
  missing="$(comm -13 "$TMP/diff-paths" "$TMP/allowed" | sed '/^$/d')"
  if [ -z "$extra" ]; then
    pass "AC4 the diff touches no path outside the six allowed paths"
  else
    bad "AC4 the diff touches paths outside the allowed set:"
    printf '%s\n' "$extra" | head -n 10 | while IFS= read -r l; do note "$l"; done
  fi
  if [ -z "$missing" ]; then
    pass "AC4 every allowed path is touched (metrics writer, clock wait, headless exit, smoke, BASELINE.md)"
  else
    bad "AC4 allowed paths not touched by the diff:"
    printf '%s\n' "$missing" | while IFS= read -r l; do note "$l"; done
  fi
  if [ ! -s "$TMP/fix-paths-u" ]; then
    bad "AC4 the fix commits (003-03, 003-04, 003-06, 003-07, 003-08) touch no files; cannot compare"
  elif cmp -s "$TMP/diff-paths" "$TMP/fix-paths-u"; then
    pass "AC4 the diff equals the union of the files of the anchored fix commits"
  else
    bad "AC4 the diff differs from the union of the fix commits' files:"
    diff "$TMP/fix-paths-u" "$TMP/diff-paths" | head -n 10 | while IFS= read -r l; do note "$l"; done
  fi
else
  skip "AC4 not evaluated: the tag $BASELINE_TAG is absent"
fi

# -------------------------------------------------------------- AC 5
echo "-- AC5 consistency: BASELINE.md and tools/smoke.sh at the tag, the note names both tags"
if [ "$HAVE_TAG" -eq 1 ]; then
  if gitpoc cat-file -e "$BASELINE_TAG:$DOC" 2>/dev/null; then
    pass "AC5 $DOC is in the tree of $BASELINE_TAG"
  else
    bad "AC5 $DOC is not in the tree of $BASELINE_TAG"
  fi
  smode="$(gitpoc ls-tree "$BASELINE_TAG" -- "$SMOKE" 2>/dev/null | awk '{ print $1 }')"
  if [ "$smode" = "100755" ]; then
    pass "AC5 $SMOKE is in the tree of $BASELINE_TAG, mode 755"
  else
    bad "AC5 $SMOKE at $BASELINE_TAG has mode '${smode:-absent}', expected 100755"
  fi
  if gitpoc show "$BASELINE_TAG:$DOC" 2>/dev/null > "$TMP/doc.md"; then
    if grep -Eq '(^|[^A-Za-z0-9_-])poc-untouched([^A-Za-z0-9_-]|$)' "$TMP/doc.md" \
       && grep -Eq '(^|[^A-Za-z0-9_-])poc-baseline([^A-Za-z0-9_-]|$)' "$TMP/doc.md"; then
      pass "AC5 $DOC at $BASELINE_TAG names poc-untouched and poc-baseline"
    else
      bad "AC5 $DOC at $BASELINE_TAG does not name both poc-untouched and poc-baseline"
    fi
  else
    bad "AC5 $DOC cannot be read at $BASELINE_TAG"
  fi
else
  skip "AC5 not evaluated: the tag $BASELINE_TAG is absent"
fi

# -------------------------------------------------------------- AC 6
echo "-- AC6 freeze: HEAD is the tag's commit; the working tree is clean"
if [ "$HAVE_TAG" -eq 1 ]; then
  head_commit="$(gitpoc rev-parse HEAD 2>/dev/null)"
  tag_commit="$(gitpoc rev-parse -q --verify "$BASELINE_TAG^{commit}" 2>/dev/null || true)"
  if [ -n "$tag_commit" ] && [ "$head_commit" = "$tag_commit" ]; then
    pass "AC6 HEAD equals $BASELINE_TAG (a detached HEAD at the tag is allowed)"
  else
    bad "AC6 HEAD is ${head_commit:0:12}, $BASELINE_TAG is ${tag_commit:0:12}: the POC has moved after the tag"
  fi
  # No commit reachable from ANY ref lies outside the tag's history.  --all covers
  # refs/heads, refs/tags, refs/remotes, refs/stash and notes: a commit after the
  # tag on a side branch, a branch moved past the tag, or a stash entry all fail.
  outside="$(gitpoc rev-list --all --not "$BASELINE_TAG" 2>/dev/null | head -n 5)"
  if [ -z "$outside" ]; then
    pass "AC6 no commit reachable from any ref lies outside $BASELINE_TAG's history (refs, stash and notes included)"
  else
    bad "AC6 commits reachable from a ref (branch, tag, remote, stash or note) lie outside $BASELINE_TAG's history:"
    printf '%s\n' "$outside" | while IFS= read -r c; do note "$(gitpoc log -1 --format='%h %D %s' "$c" 2>/dev/null)"; done
  fi
  # Every local branch points at the tag's commit.
  moved="$(gitpoc for-each-ref --format='%(refname) %(objectname)' refs/heads 2>/dev/null \
    | awk -v t="$tag_commit" '$2 != t { print $1 " at " substr($2, 1, 12) }')"
  if [ -z "$moved" ]; then
    pass "AC6 every local branch points at $BASELINE_TAG"
  else
    bad "AC6 local branches not at $BASELINE_TAG (a branch moved past the tag):"
    printf '%s\n' "$moved" | while IFS= read -r l; do note "$l"; done
  fi
else
  skip "AC6 HEAD-equals-tag not evaluated: the tag $BASELINE_TAG is absent"
fi
if [ -e "$MARKER_PATH" ]; then
  echo "SKIP  AC6 working tree clean NOT EVALUATED: orchestrator marker $MARKER_PATH is present (a POC write is in flight)"
else
  porcelain="$(gitpoc status --porcelain 2>&1)"
  if [ -z "$porcelain" ]; then
    pass "AC6 working tree clean (git status --porcelain empty)"
  else
    bad "AC6 working tree not clean:"
    printf '%s\n' "$porcelain" | head -n 5 | while IFS= read -r l; do note "$l"; done
  fi
fi

# ----------------------------------------------------------------- summary
if [ "$fail" -eq 0 ]; then
  echo "== 003-09 PASSED =="
  exit 0
fi
echo "== 003-09 FAILED =="
exit 1
