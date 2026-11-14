#!/usr/bin/env bash
# checks/002-01.sh — checks for ticket 002-01 "Scratch copy and report skeleton".
#
# Scope: the deliverable of 002-01 is the document
#   doc/design/reports/archaeology/REPORT-0b.md
# plus the evidence it records. This script never writes anywhere; it reads the
# frozen POC repository at /home/dorian/workspace/hadropoc only.
#
# Sections (a)-(e) may hold either the skeleton-time placeholder
# `**verdict: pending**` or the final verdict line for that section
# ("instrument bug, mechanism: …", "not a bug, explanation: …", or the prefixed
# variant "verdict: <finding> — mechanism: …"). The skeleton state is pending;
# final verdicts arrive per ticket. Each section must always carry exactly one
# verdict line.
#
# Frozen-tree fingerprint convention (fixed by the team lead so that 002-01 and
# 003-01 stay independent). The original recorded value documents the
# 002-01-time working tree (55 files). 003-01's .gitignore later left 27 of
# those files untracked (the experiment data under experiments/{pilot,run1,
# run2,run3}/), so the state has two permanent guards:
#   - the 28 tracked files are recoverable forever from the poc-untouched tag
#     tree (the git-reproducible tracked baseline; 003-01 guards the working
#     tree and the 003-XX-only commit rule, 002-01 does not duplicate that);
#   - the 27 untracked experiment-data files are frozen instrument data and
#     are guarded by checks/002-01-frozen-data.sha256.
# The tag-tree fingerprint is the sha256 of the following byte stream, in
# LC_ALL=C sorted relative-path order:
#
#   for each regular file under src/, tools/, experiments/ (excluding target/,
#   .git/, build/ and .gitignore) and the files Cargo.toml, Cargo.lock,
#   README.md, as recorded in the poc-untouched tag:
#       "path:" <relative-path> NUL <file-contents> NUL
#
# i.e. sorted concatenation of the relative paths and contents. The helper
# below reads the blobs through git from the tag; it never looks at the working
# tree. The recorded 002-01-time (55-file) value is not recomputed here.

set -u

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
REPORT_REL="doc/design/reports/archaeology/REPORT-0b.md"
REPORT="$REPO_ROOT/$REPORT_REL"
POC_ROOT="/home/dorian/workspace/hadropoc"
FROZEN_DATA_MANIFEST="$REPO_ROOT/checks/002-01-frozen-data.sha256"

pass=0
fail=0

ok()  { printf 'PASS: %s\n' "$*"; pass=$((pass + 1)); }
bad() { printf 'FAIL: %s\n' "$*"; fail=$((fail + 1)); }

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# sha256 over the sorted (path, contents) concatenation described above, read
# from the named tag's tree through git. Independent of the working tree.
compute_tag_fingerprint() {
  local root="$1" tag="$2"
  (
    git -C "$root" ls-tree -r "$tag" |
      awk -F'\t' '
        {
          split($1, meta, " ");
          if (meta[2] != "blob") next;
          path = $2;
          if (path ~ /(^|\/)(target|build|\.git)(\/|$)/) next;
          if (path ~ /(^|\/)\.gitignore$/) next;
          if (path ~ /^src\// || path ~ /^tools\// || path ~ /^experiments\// ||
              path == "Cargo.toml" || path == "Cargo.lock" || path == "README.md") {
            print path "\t" meta[3];
          }
        }
      ' |
      LC_ALL=C sort |
      while IFS=$'\t' read -r path oid; do
        printf 'path:%s\0' "$path"
        git -C "$root" cat-file blob "$oid"
        printf '\0'
      done |
      sha256sum | cut -d' ' -f1
  )
}

# Print the first heading (line beginning with #) that contains the fixed text.
heading_line() {
  local file="$1" text="$2"
  awk -v t="$text" '
    /^#+[ \t]/ && index(tolower($0), tolower(t)) > 0 { print NR; exit }
  ' "$file"
}

# Print the text of the section whose heading contains the fixed text: the
# heading plus everything up to the next heading of the same or higher level.
section_text() {
  local file="$1" text="$2"
  awk -v t="$text" '
    function level(s) { match(s, /^#+/); return RLENGTH }
    BEGIN { insec = 0 }
    !insec && /^#+[ \t]/ && index(tolower($0), tolower(t)) > 0 {
      insec = 1; lvl = level($0); print; next
    }
    insec {
      if (/^#+[ \t]/ && level($0) <= lvl) exit
      print
    }
  ' "$file"
}

# Print everything before the first heading containing the fixed text.
text_before_heading() {
  local file="$1" text="$2"
  awk -v t="$text" '
    /^#+[ \t]/ && index(tolower($0), tolower(t)) > 0 { exit }
    { print }
  ' "$file"
}

# Print the 64-hex value on the first line matching the extended regex, or on
# the immediately following non-empty line.
hash_for_regex() {
  local file="$1" re="$2" line no next
  no=$(grep -Ein "$re" "$file" | cut -d: -f1 | head -n 1)
  [ -n "$no" ] || return 1
  line=$(sed -n "${no}p" "$file")
  if [[ $line =~ ([0-9a-fA-F]{64}) ]]; then
    printf '%s\n' "${BASH_REMATCH[1]}"
    return 0
  fi
  next=$((no + 1))
  while :; do
    line=$(sed -n "${next}p" "$file")
    [ -n "$line" ] || return 1
    if [[ $line =~ ([0-9a-fA-F]{64}) ]]; then
      printf '%s\n' "${BASH_REMATCH[1]}"
      return 0
    fi
    return 1
  done
}

trim() {
  local s="$1"
  s="${s#"${s%%[![:space:]]*}"}"
  s="${s%"${s##*[![:space:]]}"}"
  printf '%s' "$s"
}

# ---------------------------------------------------------------------------
# AC 4 — the report exists at the settled location and is not in the POC tree
# ---------------------------------------------------------------------------

if [ ! -f "$REPORT" ]; then
  bad "report missing: $REPORT_REL"
  printf '\n1 check(s) failed; the document for 002-01 does not exist yet.\n'
  exit 1
fi

report_real="$(cd "$(dirname "$REPORT")" && pwd -P)/$(basename "$REPORT")"
case "$report_real" in
  "$POC_ROOT"|"$POC_ROOT"/*)
    bad "report is inside the frozen POC tree: $report_real"
    ;;
  *)
    ok "report sits outside the POC tree at $REPORT_REL"
    ;;
esac

# ---------------------------------------------------------------------------
# Header — purpose (check 0b), evidence tags, verdict-line format (AC 3)
# ---------------------------------------------------------------------------

header=$(text_before_heading "$REPORT" "Environment")

if printf '%s\n' "$header" | grep -qi 'purpose' && printf '%s\n' "$header" | grep -q '0b'; then
  ok "header states the purpose (check 0b)"
else
  bad "header does not state the purpose of check 0b (need 'purpose' and '0b' before the Environment section)"
fi

missing_tags=""
for tag in '[code]' '[measured]' '[kernel-doc]' '[memory]' '[decision]'; do
  printf '%s\n' "$header" | grep -Fq "$tag" || missing_tags="$missing_tags $tag"
done
if [ -z "$missing_tags" ]; then
  ok "header lists the evidence tags in use ([code], [measured], [kernel-doc], [memory], [decision])"
else
  bad "header is missing evidence tag(s):$missing_tags"
fi

if printf '%s\n' "$header" | grep -Fqi 'instrument bug, mechanism:'; then
  bug_line=1
else
  bug_line=0
fi
if printf '%s\n' "$header" | grep -Fqi 'not a bug, explanation:'; then
  nobug_line=1
else
  nobug_line=0
fi
if [ "$bug_line" -eq 1 ] && [ "$nobug_line" -eq 1 ] &&
   printf '%s\n' "$header" | grep -qi 'verdict'; then
  ok "header states the verdict-line format (two allowed forms)"
else
  bad "header does not state both verdict-line forms ('instrument bug, mechanism: …' and 'not a bug, explanation: …')"
fi

# ---------------------------------------------------------------------------
# Environment section (AC 2)
# ---------------------------------------------------------------------------

env_sec=$(section_text "$REPORT" "Environment")
if [ -n "$env_sec" ] && [ -n "$(heading_line "$REPORT" "Environment")" ]; then
  ok "Environment section exists"

  if printf '%s\n' "$env_sec" | grep -Eqi '(^|[^[:alnum:]])host(name)?[[:space:]]*[:=|][[:space:]]*[[:alnum:]]'; then
    ok "Environment records the host"
  else
    bad "Environment does not record the host as a labelled value"
  fi

  current_kernel=$(uname -r)
  if printf '%s\n' "$env_sec" | grep -qi 'kernel' &&
     printf '%s\n' "$env_sec" | grep -Fq "$current_kernel"; then
    ok "Environment records the kernel ($current_kernel)"
  else
    bad "Environment does not record the running kernel version '$current_kernel'"
  fi

  if printf '%s\n' "$env_sec" | grep -q 'CONFIG_HZ' &&
     printf '%s\n' "$env_sec" | grep -Eq '(^|[^0-9])250([^0-9]|$)'; then
    ok "Environment records CONFIG_HZ=250"
  else
    bad "Environment does not record CONFIG_HZ=250"
  fi

  if printf '%s\n' "$env_sec" | grep -Eqi 'toolchain|rustc|cargo' &&
     printf '%s\n' "$env_sec" | grep -Eq '[0-9]+\.[0-9]+\.[0-9]+'; then
    ok "Environment records a toolchain version"
  else
    bad "Environment does not record a toolchain version (rustc/cargo + x.y.z)"
  fi

  if printf '%s\n' "$env_sec" | grep -Eqi 'frozen[- ]tree fingerprint, taken[^|]*[0-9]'; then
    ok "Environment records 'frozen-tree fingerprint, taken <date>'"
  else
    bad "Environment is missing the line 'frozen-tree fingerprint, taken <date>'"
  fi

  scratch_line=$(printf '%s\n' "$env_sec" | grep -Ei 'scratch' |
    grep -E '/[^[:space:]|,;)]+' | head -n 1)
  scratch_path=$(printf '%s' "$scratch_line" |
    grep -Eo '/[A-Za-z0-9._+/-]+' | head -n 1)
  if [ -n "$scratch_path" ]; then
    case "$scratch_path" in
      "$POC_ROOT"|"$POC_ROOT"/*)
        bad "recorded scratch path is inside the frozen POC tree: $scratch_path"
        ;;
      /*)
        ok "Environment records a scratch path outside the POC tree: $scratch_path"
        ;;
      *)
        bad "recorded scratch path is not absolute: $scratch_path"
        ;;
    esac
  else
    bad "Environment does not record an absolute scratch path"
  fi
else
  bad "Environment section is missing"
  scratch_path=""
fi

# ---------------------------------------------------------------------------
# The five sections, each with exactly one verdict line (pending at skeleton
# time, final as the epic's tickets complete) (AC 3)
# ---------------------------------------------------------------------------

titles=(
  "(a) Inert check families"
  "(b) The 21.7 M counter"
  "(c) Experiment metrics integrity"
  "(d) Reading notes"
  "(e) Run confirmations"
)

previous_line=0
order_ok=1
for title in "${titles[@]}"; do
  count=$(awk -v t="$title" '
    /^#+[ \t]/ && index(tolower($0), tolower(t)) > 0 { n++ }
    END { print n + 0 }
  ' "$REPORT")
  line=$(heading_line "$REPORT" "$title")
  if [ "$count" -ne 1 ] || [ -z "$line" ]; then
    bad "expected exactly one heading containing '$title' (found $count)"
    order_ok=0
    continue
  fi
  if [ "$line" -le "$previous_line" ]; then
    order_ok=0
  fi
  previous_line=$line

  sec=$(section_text "$REPORT" "$title")
  verdict_count=$(printf '%s\n' "$sec" |
    grep -Ec '^[[:space:]]*(\*\*)?verdict:')
  if [ "$verdict_count" -ne 1 ]; then
    bad "section '$title' has $verdict_count verdict line(s), expected exactly 1"
    continue
  fi
  verdict_line=$(printf '%s\n' "$sec" |
    grep -E '^[[:space:]]*(\*\*)?verdict:' | head -n 1)
  verdict_body=$(printf '%s\n' "$verdict_line" |
    sed -E 's/^[[:space:]]*(\*\*)?verdict:[[:space:]]*//; s/[[:space:]]*\*\*[[:space:]]*$//')
  if printf '%s' "$verdict_body" | grep -Eqi '^pending$'; then
    ok "section '$title' has exactly one verdict line (pending)"
  elif printf '%s' "$verdict_body" | grep -Eqi 'mechanism:'; then
    ok "section '$title' has exactly one verdict line (instrument bug, mechanism)"
  elif printf '%s' "$verdict_body" | grep -Eqi 'explanation:'; then
    ok "section '$title' has exactly one verdict line (not a bug, explanation)"
  else
    bad "section '$title' has one verdict line but it is not in an allowed form: $verdict_line"
  fi
done
if [ "$order_ok" -eq 1 ]; then
  ok "the five sections appear in order (a) to (e)"
else
  bad "the five sections are missing or out of order (a) to (e)"
fi

# ---------------------------------------------------------------------------
# Open sub-questions — 4-row table with owner and falsifier (AC 3)
# ---------------------------------------------------------------------------

oq_sec=$(section_text "$REPORT" "Open sub-questions")
if [ -z "$oq_sec" ]; then
  bad "Open sub-questions section is missing"
else
  mapfile -t oq_rows < <(printf '%s\n' "$oq_sec" | grep -E '^[[:space:]]*\|')
  header_idx=-1
  for i in "${!oq_rows[@]}"; do
    row="$(trim "${oq_rows[$i]}")"
    row="${row#|}"
    row="${row%|}"
    core=$(printf '%s' "$row" | tr -d '|[:space:]:-')
    [ -n "$core" ] || continue
    if printf '%s' "$row" | grep -qi 'owner' &&
       printf '%s' "$row" | grep -qi 'falsifier'; then
      header_idx=$i
      break
    fi
  done

  if [ "$header_idx" -lt 0 ]; then
    bad "Open sub-questions table does not have an Owner/Falsifier header"
  else
    data_rows=0
    table_ok=1
    for i in "${!oq_rows[@]}"; do
      [ "$i" -gt "$header_idx" ] || continue
      row="$(trim "${oq_rows[$i]}")"
      row="${row#|}"
      row="${row%|}"
      core=$(printf '%s' "$row" | tr -d '|[:space:]:-')
      [ -n "$core" ] || continue

      IFS='|' read -r -a cells <<< "$row"
      if [ "${#cells[@]}" -ne 3 ]; then
        bad "Open sub-questions row has ${#cells[@]} cells, expected 3 (question | owner | falsifier): $row"
        table_ok=0
        continue
      fi
      question="$(trim "${cells[0]:-}")"
      owner="$(trim "${cells[1]:-}")"
      falsifier="$(trim "${cells[2]:-}")"
      if [ -z "$question" ] || ! printf '%s' "$question" | grep -q '[[:alnum:]]'; then
        bad "Open sub-questions row has no question text: $row"
        table_ok=0
      fi
      if [ -z "$owner" ] || ! printf '%s' "$owner" | grep -q '[[:alnum:]]'; then
        bad "Open sub-questions row has no owner: $row"
        table_ok=0
      fi
      if [ -z "$falsifier" ] || ! printf '%s' "$falsifier" | grep -q '[[:alnum:]]'; then
        bad "Open sub-questions row has no falsifier: $row"
        table_ok=0
      fi
      data_rows=$((data_rows + 1))
    done
    if [ "$data_rows" -eq 4 ] && [ "$table_ok" -eq 1 ]; then
      ok "Open sub-questions table has 4 rows, each with owner and falsifier"
    elif [ "$data_rows" -ne 4 ]; then
      bad "Open sub-questions table has $data_rows data rows, expected 4"
    fi
  fi
fi

# ---------------------------------------------------------------------------
# AC 1 — loopback command and exit status pasted in the report
# ---------------------------------------------------------------------------

cmd_hits=$(grep -nF 'hadropoc loopback --duration 5' "$REPORT" | cut -d: -f1)
cmd_found=0
status_ok=0
for no in $cmd_hits; do
  cmd_found=1
  start=$((no - 2)); [ "$start" -lt 1 ] && start=1
  end=$((no + 12))
  window=$(sed -n "${start},${end}p" "$REPORT")
  if printf '%s\n' "$window" |
       grep -Eqi '(exit[[:space:]]*(status|code)?|status|exited with)[^0-9]{0,30}0([^0-9]|$)' ||
     printf '%s\n' "$window" |
       grep -Eq '^[[:space:]]*(\$[[:space:]]*)?0[[:space:]]*$'; then
    status_ok=1
    break
  fi
done
if [ "$cmd_found" -eq 0 ]; then
  bad "report does not paste the command 'hadropoc loopback --duration 5'"
elif [ "$status_ok" -eq 1 ]; then
  ok "report pastes 'hadropoc loopback --duration 5' with exit status 0"
else
  bad "the pasted loopback command is not followed by a visible exit status 0"
fi

# ---------------------------------------------------------------------------
# AC 2 + AC 5 — the historical (55-file) recorded fingerprints are identical;
# the tracked baseline is guarded by the poc-untouched tag, and the 27 untracked
# experiment-data files by the committed per-file manifest
# ---------------------------------------------------------------------------

frozen_hash=$(hash_for_regex "$REPORT" 'frozen[- ]tree fingerprint, taken' || true)
scratch_hash=$(hash_for_regex "$REPORT" 'scratch.*fingerprint|fingerprint.*scratch' || true)

if [ -n "$frozen_hash" ]; then
  ok "recorded frozen-tree fingerprint: $frozen_hash"
else
  bad "no 64-hex frozen-tree fingerprint recorded next to 'frozen-tree fingerprint, taken'"
fi

if [ -n "$scratch_hash" ]; then
  ok "recorded scratch-copy fingerprint at creation: $scratch_hash"
else
  bad "no 64-hex scratch-copy fingerprint at creation recorded (label must mention 'scratch' and 'fingerprint')"
fi

if [ -n "$frozen_hash" ] && [ -n "$scratch_hash" ]; then
  if [ "${frozen_hash,,}" = "${scratch_hash,,}" ]; then
    ok "the two recorded fingerprints are identical"
  else
    bad "the two recorded fingerprints differ: frozen=$frozen_hash scratch=$scratch_hash"
  fi
fi

# (b) The 27 untracked experiment-data files are frozen instrument data. They
# are not in git (003-01's .gitignore), so guard them by their committed hashes.
if [ ! -f "$FROZEN_DATA_MANIFEST" ]; then
  bad "frozen-data manifest missing: checks/002-01-frozen-data.sha256"
else
  manifest_entries=0
  manifest_unexpected=0
  manifest_mismatch=0
  while read -r expected path; do
    [ -n "${expected:-}" ] || continue
    case "$expected" in \#*) continue ;; esac
    manifest_entries=$((manifest_entries + 1))
    case "${path:-}" in
      experiments/pilot/*|experiments/run1/*|experiments/run2/*|experiments/run3/*) ;;
      *)
        bad "manifest entry outside the frozen experiment-data dirs: ${path:-<missing>}"
        manifest_unexpected=$((manifest_unexpected + 1))
        continue
        ;;
    esac
    if [ ! -f "$POC_ROOT/$path" ]; then
      bad "frozen experiment data missing: $path"
      manifest_mismatch=$((manifest_mismatch + 1))
      continue
    fi
    actual=$(sha256sum -- "$POC_ROOT/$path" | cut -d' ' -f1)
    if [ "$actual" != "$expected" ]; then
      bad "frozen experiment data changed: $path (expected=$expected actual=$actual)"
      manifest_mismatch=$((manifest_mismatch + 1))
    fi
  done < "$FROZEN_DATA_MANIFEST"

  if [ "$manifest_entries" -ne 27 ]; then
    bad "frozen-data manifest has $manifest_entries entries, expected 27"
  fi

  # An added file under the four frozen dirs is a modification too: the
  # manifest must cover exactly the files present there.
  manifest_paths=$(grep -v '^#' "$FROZEN_DATA_MANIFEST" | awk '{ print $2 }' | LC_ALL=C sort)
  present_paths=$(cd "$POC_ROOT" &&
    find experiments/pilot experiments/run1 experiments/run2 experiments/run3 -type f |
    LC_ALL=C sort)
  if [ "$manifest_paths" != "$present_paths" ]; then
    added=$(LC_ALL=C comm -13 <(printf '%s\n' "$manifest_paths") <(printf '%s\n' "$present_paths") || true)
    gone=$(LC_ALL=C comm -23 <(printf '%s\n' "$manifest_paths") <(printf '%s\n' "$present_paths") || true)
    [ -z "$added" ] || bad "files added under the frozen experiment-data dirs: $(printf '%s' "$added" | tr '\n' ' ')"
    [ -z "$gone" ] || bad "frozen experiment-data files missing from the working tree: $(printf '%s' "$gone" | tr '\n' ' ')"
  fi

  if [ "$manifest_entries" -eq 27 ] && [ "$manifest_unexpected" -eq 0 ] &&
     [ "$manifest_mismatch" -eq 0 ] && [ "$manifest_paths" = "$present_paths" ]; then
    ok "27 frozen untracked experiment-data files match checks/002-01-frozen-data.sha256"
  fi
fi

# (c) The third report line: the tracked baseline, recomputed from the tag.
tag_hash=$(hash_for_regex "$REPORT" 'poc-untouched tag-tree fingerprint' || true)
if ! printf '%s\n' "$env_sec" |
     grep -Eqi 'poc-untouched tag-tree fingerprint[^|]*taken[^|]*[0-9]'; then
  bad "Environment is missing the dated 'poc-untouched tag-tree fingerprint ... taken <date>' line"
elif [ -z "$tag_hash" ]; then
  bad "the 'poc-untouched tag-tree fingerprint' line has no 64-hex value"
elif ! git -C "$POC_ROOT" rev-parse --verify --quiet 'refs/tags/poc-untouched^{commit}' >/dev/null 2>&1; then
  bad "tag 'poc-untouched' not found in the POC repository at $POC_ROOT (cannot recompute the tracked baseline)"
else
  computed_tag=$(compute_tag_fingerprint "$POC_ROOT" 'poc-untouched' || true)
  if [ -z "$computed_tag" ]; then
    bad "could not compute the 'poc-untouched' tag tree fingerprint in $POC_ROOT"
  elif [ "${computed_tag,,}" = "${tag_hash,,}" ]; then
    ok "poc-untouched tag-tree fingerprint recomputed from git matches the recorded value: $computed_tag"
  else
    bad "poc-untouched tag-tree fingerprint differs (recorded=$tag_hash computed=$computed_tag)"
  fi
fi

# ---------------------------------------------------------------------------

printf '\n%d check(s) passed, %d failed.\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
