#!/usr/bin/env python3
r"""Generate the committed proof tables of doccheck::family002.

The epic-002 check scripts (checks/002-*.sh) are bash: GNU grep (ERE and
BRE), sed, awk (mawk on this machine), tr, cut, sort, wc and bash's own
pattern matching, run in the locale of the environment (no script sets
LC_ALL or LANG). The Rust module `doccheck::family002` reproduces those
semantics as named helpers. This generator runs the REAL tools on a corpus
and writes what they answered; the Rust unit tests
(`family002::proof`) compare the helpers with these tables without running
any tool. The tool versions and the locale are recorded in each header.

Files written (under --out, default hadronyche-tools/data/family002):

  ranges.tsv    per digit range lo-hi (and a-z, A-Z): its members under
                gnulib's DFA and under glibc's regex matcher.
  chars.tsv     per character of the repertoire (data/repertoire.tsv), minus
                newline (no tool sees a newline inside a line): its class
                memberships for grep -E (with and without -i), bash patterns,
                tr, mawk and cut. Columns are documented in the header.
  patterns.tsv  every (tool, flags, pattern) the corpus proves (awk: the
                regex VALUE mawk compiles, rows AT test, AM match, AG gsub), one per line:
                the list a port's patterns must belong to
                (`family002::unproven_patterns`).
  corpus.tsv    the inputs and, per probe, what the tool printed:
                  I <id> <line>              an input line
                  Q <key> <ids>              grep -q: the inputs that match
                                             (ranges "0-3,7")
                  OD <key> <digests>         grep -o: per input its matches
                                             joined by \x1f, digested
                  SD <key> <digests>         sed: the outputs of every input,
                                             one FNV-1a 64 digest per chunk
                                             of 64 (each output + \x1e)
                  HD <snippet> <ids> <dig,..> a transform's outputs on those
                                             inputs, digested as for SD

                  F <id> <text>              a whole-file fixture
                  R <probe> <fixture> <out>  a file-level probe's output
                Strings are escaped: \\ \t \n \r and \xHH for other control
                bytes and invalid UTF-8; US (\x1f) is written \x1f.

Usage (from the repository root):
  python3 hadronyche-tools/tools/gen_family002.py [--locale L] [--commit C] [--out DIR]
  python3 hadronyche-tools/tools/gen_family002.py --check   # regenerate in memory, compare

--locale defaults to the environment's (LC_ALL, else LC_CTYPE, else LANG);
the tables say which. --commit (default HEAD) is the commit whose
REPORT-0b.md lines seed the corpus. --check regenerates and compares the
rows (not the version comments) with the committed files: exit 1 on a
difference.

Adding a pattern: append it to PATTERNS (with the script it comes from),
regenerate, run `cargo test -p hadronyche-tools family002`. A pattern the
Rust translator cannot reproduce fails that test with the inputs it
disagrees on.
"""

import argparse
import shlex
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = os.path.dirname(HERE)
REPO = os.path.dirname(CRATE)
REPORT = "doc/design/reports/archaeology/REPORT-0b.md"

# ----------------------------------------------------------------- patterns
# 002-03's two dynamic regexes, as written in the script; mawk gets them
# through -v, which keeps the unknown escape \. as it is.
QUANTITY_RE = '(^|[^[:alnum:]_])(constant|const|timeout|period|delay|interval|rate|size|length|budget|window|lead|expiry|expir|election|listen|repeat|room|capacity|duration|deadline|payload|every|per second|bytes|samples?|ticks?|frames?)([^[:alnum:]_]|$)|_(LEN|NS|MS|SECS|BYTES|SLOTS)([^[:alnum:]_]|$)|(^|[^[:alnum:]_])(STATE_LEN|PULSE_LEN|REPORT_LEN|ROOM|CYCLE_SECS|DIVIDER|SLOTS|LISTEN_NS|HELLO_NS|EXPIRE_NS|ELECTION_NS)([^[:alnum:]_]|$)'
UNIT_RE = '(^|[^[:alnum:]_])(ms|milliseconds?|µs|us|microseconds?|ns|nanoseconds?|s|sec|secs|seconds?|minutes?|hours?|Hz|kHz|MHz|bytes?|KiB|KB|bits?|samples?|ticks?|frames?|fps|ppm|%|Q32\\.32|Q1\\.31|Q0\\.32|slots?|entries|channels?|controllers?|slices?|blocks?)([^[:alnum:]_]|$)|per second'
UNIT_OFFENDERS_BODY = '  local line="$1"\n  printf \'%s\\n\' "$line" | awk -v cue="$quantity_re" -v unit="$unit_re" \'\n    {\n      low = tolower($0)\n      if (low !~ cue) next\n      s = $0\n      gsub(/[A-Za-z0-9_.\\/-]+\\.rs:[0-9]+(-[0-9]+)?/, "", s)\n      gsub(/C-[0-9]+/, "", s)\n      gsub(/[0-9][0-9][0-9]-[0-9][0-9]/, "", s)\n      gsub(/Q[0-9]+\\.[0-9]+/, "", s)\n      gsub(/[0-9][0-9]?:[0-9][0-9]:[0-9][0-9][:;][0-9][0-9]/, "", s)\n      gsub(/\\^[ ]*[0-9]+/, "", s)\n      gsub(/<<[ ]*[0-9]+/, "", s)\n      while (match(s, /[0-9]+([.,][0-9]+)?/)) {\n        nstart = RSTART; nlen = RLENGTH; adv = nstart + nlen\n        num = substr(s, nstart, nlen)\n        before = substr(s, 1, nstart - 1)\n        after = substr(s, adv, 24)\n        tail = after\n        sub(/^[ \\t]*/, "", tail)\n        op = substr(tail, 1, 1)\n        if (op ~ /[+*\\/xX<×-]/) { s = substr(s, adv); continue }\n        if (tolower(after) ~ unit) { s = substr(s, adv); continue }\n        prev = before\n        sub(/[ \\t]+$/, "", prev)\n        prevch = substr(prev, length(prev), 1)\n        if (prevch ~ /[\\^+*\\/<]/) { s = substr(s, adv); continue }\n        # Dimensionless identifier assignment: `VOLUME` = 0, T_STATE = 0.\n        if (match(before, /([A-Z][A-Z0-9_]{2,})[`*_]*[ \\t]*=[ \\t]*$/)) {\n          ident = substr(before, RSTART, RLENGTH)\n          sub(/[`*_]*[ \\t]*=[ \\t]*$/, "", ident)\n          if (ident !~ /(_LEN|_NS|_MS|_SECS|_BYTES|_SLOTS)$/ \\\n              && ident != "ROOM" && ident != "CYCLE_SECS" \\\n              && ident != "DIVIDER" && ident != "SLOTS" && ident != "ONE") {\n            s = substr(s, adv); continue\n          }\n        }\n        printf "number %s has no unit near: %s\\n", num, after\n        s = substr(s, adv)\n      }\n    }\''

# (tool+flags, pattern, origin). Tool+flags is one of:
#   grep-E grep-Ei grep-G grep-Gi grep-F grep-Fi   (match per line: -q/-c/-n)
#   grep-Eo grep-Eio grep-Go                      (grep -o: matches per line)
#   sed-E                                          (sed -E script, per line)
PATTERNS = [
    # 002-02
    ("grep-Ei", r"^#+[[:space:]]*\(d\)", "002-02 heading_no (d)"),
    ("grep-Ei", r"^#+[[:space:]]*\(e\)", "002-02 heading_no (e)"),
    ("grep-Ei", r"^#+[[:space:]]*Open sub-questions", "002-02 DoD heading"),
    ("grep-E", r"^#+[[:space:]]", "002-02 heading line"),
    ("grep-G", r"[^[:space:]]", "002-02 AC1 non-empty"),
    ("grep-E", r"audio\.rs:[0-9]+", "002-02 pointer"),
    ("grep-Eo", r"audio\.rs:[0-9]+", "002-02 AC1 pointers"),
    ("grep-G", r".", "002-02 grep -c ."),
    ("grep-Ei", r"toleran|threshold|epsilon|eps\b", "002-02 AC2 tolerance"),
    ("grep-Ei", r"1024|1[ ,]024", "002-02 AC3 fact 1"),
    ("grep-Ei", r"window", "002-02 AC3 fact 1"),
    ("grep-Ei", r"complete", "002-02 AC3 fact 2"),
    ("grep-Ei", r"both", "002-02 AC3 fact 2"),
    ("grep-Ei", r"256|MAX_LAG_WINDOWS", "002-02 AC3 fact 3"),
    ("grep-Ei", r"2[.,]7", "002-02 AC3 fact 3"),
    ("grep-Ei", r"settle|settling", "002-02 AC3 fact 4"),
    ("grep-Ei", r"(1[ -]?s|1 second|1-second)", "002-02 AC3 fact 4"),
    ("grep-Ei", r"(3[ -]?s|3 second|3-second|SETTLE_SECS)", "002-02 AC3 fact 4"),
    ("grep-Ei", r"replica", "002-02 AC3 fact 5"),
    ("grep-Ei", r"gap", "002-02 AC3 fact 5"),
    ("grep-Ei", r"(^|[^[:alnum:]])(confirmed|corrected)([^[:alnum:]]|$)", "002-02 marker"),
    ("grep-F", r"[code]", "002-02 AC3 tag"),
    ("grep-E", r"^[[:space:]]*(-|\*|\+|>|[0-9]+[.)])[[:space:]]|^[[:space:]]*\|", "002-02 AC4 list entry"),
    ("grep-Ei", r"incomplete", "002-02 AC4 category"),
    ("grep-Ei", r"lag|MAX_LAG_WINDOWS", "002-02 AC4 category"),
    ("grep-Ei", r"sender[ -]?gap", "002-02 AC4 category"),
    ("grep-Ei", r"settle|SETTLE_SECS", "002-02 AC4 category"),
    ("grep-Ei", r"voices[ -]?flag|voices.*off|flag off|voices=false|voices = false", "002-02 AC4 category"),
    ("grep-Ei", r"because|since|reason|so that|to avoid|otherwise", "002-02 AC4 reason"),
    ("sed-E", r"s/audio\.rs:[0-9]+//g; s/[0-9]+\.[0-9]+//g; s/\be\.g\.//g; s/\bi\.e\.//g", "002-02 AC5 clean"),
    ("sed-E", r"s/\[(code|measured|kernel-doc|memory|decision)\]//g", "002-02 AC5 tags"),
    ("grep-Eo", r"[.!?]", "002-02 AC5 terminals"),
    ("grep-G", r"[[:alnum:]]", "002-02 AC5 words"),
    ("grep-Gi", r"voice comparison bit-level", "002-02 DoD row"),
    ("grep-E", r"^[[:space:]]*\|", "002-02 DoD table row"),
    ("grep-Gi", r"answered", "002-02 DoD answered"),
    ("grep-Ei", r"(^|[^[:alnum:]])(yes|no)([^[:alnum:]]|$)", "002-02 DoD yes/no"),
    # 002-03
    ("grep-E", r"\[(code|measured|kernel-doc|memory|decision)\]", "002-03 has_tag"),
    ("grep-E", r"[A-Za-z0-9_./-]+\.rs:[0-9]+", "002-03 has_ptr"),
    ("grep-Eo", r"[A-Za-z0-9_./-]+\.rs:[0-9]+", "002-03 AC1 pointers"),
    ("grep-Eo", r"transport\.rs:[0-9]+", "002-03 AC1 own pointers"),
    ("grep-Eo", r"control\.rs:[0-9]+", "002-03 AC1 own pointers"),
    ("grep-E", r"not in (these|the two|either) files?|not found in (these|either)|absent from (these|both)", "002-03 has_marker"),
    ("grep-E", r"transport\.rs", "002-03 find_headings"),
    ("grep-E", r"control\.rs", "002-03 find_headings"),
    ("sed-E", r"s/^(#+).*/\1/", "002-03 heading level"),
    ("grep-Ei", r"who acts[^.]*reference|transport[ -]reference|reference (role|holder)|(sender|synth)[^.]*(is|acts as|serves as|defaults? to)[^.]*reference|reference[^.]{0,30}(is|equals)[^.]{0,20}(sender|synth)([^[:alnum:]_.]|$)", "002-03 fact 1"),
    ("grep-Ei", r"(^|[^[:alnum:]_])sender([^[:alnum:]_.]|$)", "002-03 fact 1"),
    ("grep-Ei", r"is_reference", "002-03 fact 2"),
    ("grep-Ei", r"module\.rs|elected|election|routes", "002-03 fact 2"),
    ("grep-Ei", r"listen|claim", "002-03 fact 3"),
    ("grep-Ei", r"(1[.,]5[^0-9]{0,3}(s|sec|secs|second|seconds))|(1500[[:space:]]*ms)|(1[ _]?500[ _]?000[ _]?000[[:space:]]*ns)", "002-03 fact 3"),
    ("grep-Ei", r"before|hear|guard|already|join|wait", "002-03 fact 3"),
    ("grep-Ei", r"transport|datagram|events", "002-03 fact 4"),
    ("grep-Ei", r"events", "002-03 fact 4"),
    ("grep-Ei", r"output|connection|port|\bsends?\b|\bsent\b|\bsending\b", "002-03 fact 4"),
    ("grep-E", r"[0-9]", "002-03 AC3 digit"),
    ("grep-Ei", QUANTITY_RE, "002-03 AC3 quantity_re"),
    ("grep-E", r"(^|[^A-Za-z0-9])C-30([^0-9]|$)", "002-03 AC4 id"),
    ("grep-E", r"(^|[^A-Za-z0-9])C-31([^0-9]|$)", "002-03 AC4 id"),
    ("grep-E", r"(^|[^A-Za-z0-9])C-32([^0-9]|$)", "002-03 AC4 id"),
    ("grep-E", r"(^|[^A-Za-z0-9])C-33([^0-9]|$)", "002-03 AC4 id"),
    ("grep-Eo", r"C-[0-9]+[[:space:]]*(to|through|\.\.|-)[[:space:]]*C-[0-9]+", "002-03 AC4 ranges"),
    ("sed-E", r"s/^C-([0-9]+).*/\1/", "002-03 AC4 range start"),
    ("sed-E", r"s/.*C-([0-9]+)$/\1/", "002-03 AC4 range end"),
    ("grep-Ei", r"(^|[^[:alnum:]_])differ(s|ed|ing)?([^[:alnum:]_]|$)", "002-03 AC4 differ"),
    ("grep-G", r"002-13", "002-03 AC4 flagged"),
    ("grep-Ei", r"(^|[^[:alnum:]_])(match|matches|matched|matching|differ|differs|differed|differing)([^[:alnum:]_]|$)", "002-03 AC4 verdict"),
    ("grep-E", r"open (item|question)|\btodo\b|\btbd\b|\bunresolved\b", "002-03 DoD open item"),
    ("grep-E", r"(^|[^a-z])(no|none|not)[^.]{0,30}open", "002-03 DoD negation"),
    ("grep-Gi", r"owner", "002-03 DoD owner"),
    ("grep-Gi", r"falsifier", "002-03 DoD falsifier"),
    ("grep-E", r"^[[:space:]]*(\*\*)?verdict:", "002-03 section marker"),
    ("sed-E", r"s/^[[:space:]]*(\*\*)?verdict:[[:space:]]*//; s/[[:space:]]*\*\*[[:space:]]*$//", "002-03 verdict body"),
    ("grep-Ei", r"^pending$", "002-03 verdict pending"),
    ("grep-Ei", r"mechanism:", "002-03 verdict mechanism"),
    ("grep-Ei", r"explanation:", "002-03 verdict explanation"),
    # 002-03 awk (unit_offenders): the regex values mawk compiles
    ("awk-test", QUANTITY_RE, "002-03 awk cue (-v)"),
    ("awk-test", UNIT_RE, "002-03 awk unit (-v)"),
    ("awk-gsub", r"[A-Za-z0-9_.\/-]+\.rs:[0-9]+(-[0-9]+)?", "002-03 awk"),
    ("awk-gsub", r"C-[0-9]+", "002-03 awk"),
    ("awk-gsub", r"[0-9][0-9][0-9]-[0-9][0-9]", "002-03 awk"),
    ("awk-gsub", r"Q[0-9]+\.[0-9]+", "002-03 awk"),
    ("awk-gsub", r"[0-9][0-9]?:[0-9][0-9]:[0-9][0-9][:;][0-9][0-9]", "002-03 awk"),
    ("awk-gsub", r"\^[ ]*[0-9]+", "002-03 awk"),
    ("awk-gsub", r"<<[ ]*[0-9]+", "002-03 awk"),
    ("awk-match", r"[0-9]+([.,][0-9]+)?", "002-03 awk"),
    ("awk-gsub", r"^[ \t]*", "002-03 awk"),
    ("awk-test", r"[+*\/xX<×-]", "002-03 awk operator"),
    ("awk-gsub", r"[ \t]+$", "002-03 awk"),
    ("awk-test", r"[\^+*\/<]", "002-03 awk previous"),
    ("awk-match", r"([A-Z][A-Z0-9_]{2,})[`*_]*[ \t]*=[ \t]*$", "002-03 awk"),
    ("awk-gsub", r"[`*_]*[ \t]*=[ \t]*$", "002-03 awk"),
    ("awk-test", r"(_LEN|_NS|_MS|_SECS|_BYTES|_SLOTS)$", "002-03 awk"),
    # 002-04
    ("grep-E", r"\[(code|measured|kernel-doc|memory|decision|standard)\]", "002-04 has_tag (Tag6)"),
    ("grep-Eo", r"sender\.rs:[0-9]+", "002-04 AC1 own pointers"),
    ("grep-Eo", r"stats\.rs:[0-9]+", "002-04 AC1 own pointers"),
    ("grep-Eo", r"tui\.rs:[0-9]+", "002-04 AC1 own pointers"),
    ("grep-E", r"sender\.rs:435([^0-9]|$)", "002-04 AC2 hop pointer"),
    ("grep-E", r"stats\.rs:324([^0-9]|$)", "002-04 AC2 hop pointer"),
    ("grep-E", r"tui\.rs:(10[7-9]|11[0-4])([^0-9]|$)", "002-04 AC2 hop pointer"),
    ("grep-Ei", r"rx_lost", "002-04 AC2"),
    ("grep-Ei", r"(^|[^[:alnum:]_])(u32|32[ -]?bit|4[ -]?byte)([^[:alnum:]_]|$)", "002-04 AC2 type32"),
    ("grep-Ei", r"(^|[^[:alnum:]_])(packets?|counts?|dimensionless|unitless)([^[:alnum:]_]|$)|no (unit|conversion|scaling)|unchanged", "002-04 AC2 unit"),
    ("grep-Ei", r"header|echo|datagram|reply|returned", "002-04 AC2 source"),
    ("grep-Ei", r"\[code\]", "002-04 AC2 tag"),
    ("grep-Ei", r"lost_fwd", "002-04 AC2"),
    ("grep-Ei", r"(^|[^[:alnum:]_])(u64|64[ -]?bit|8[ -]?byte)([^[:alnum:]_]|$)", "002-04 AC2 type64"),
    ("grep-Ei", r"one[- ]line|display|print|status|format", "002-04 AC2 display"),
    ("grep-Ei", r"sent|lost_total|received", "002-04 AC2 branch"),
    ("grep-Ei", r"lost_fwd|rx_lost", "002-04 context"),
    ("grep-Ei", r"streams?", "002-04 context"),
    ("grep-Ei", r"(^|[^[:alnum:]_])(latest|last|newest|overwrit(e|es|ten|ing)|replac(e|es|ed|ing)|wins?|supersed(e|es|ed))([^[:alnum:]_]|$)", "002-04 context latest"),
    ("grep-Ei", r"no per[ -]?stream|not per[ -]?stream|across all|without (per[ -]?stream|separat)|no (per[ -]?stream )?separat|pooled|merged|single (counter|value|number|lost)|one (counter|value|number)|regardless of (the )?stream|indifferent to (the )?stream", "002-04 context pooled"),
    ("grep-Ei", r"tx pacing lateness|tx_late", "002-04 AC3"),
    ("grep-E", r"sender\.rs:[0-9]+", "002-04 AC3 pointer"),
    ("grep-Ei", r"(^|[^[:alnum:]_])(ns|nanoseconds?|µs|us|microseconds?|ms|milliseconds?)([^[:alnum:]_]|$)", "002-04 AC3 unit"),
]

# Bash snippets proven per input line: the line is in "$line"; the output
# is what the snippet prints. Each is the script's own code.
SNIPPETS = {
    # 002-02 normalise(), verbatim
    "normalise": r'''s="$line"
  s="${s#"${s%%[![:space:]]*}"}"
  s="${s#- }"; s="${s#\* }"; s="${s#+ }"; s="${s#> }"
  s="${s#"${s%%[![:space:]]*}"}"
  s="${s#\*\*}"; s="${s%\*\*}"
  s="${s#"${s%%[![:space:]]*}"}"
  printf '%s' "$s"''',
    "tr_lower": r'''printf '%s' "$line" | tr '[:upper:]' '[:lower:]' ''',
    "rstrip_space": r'''nl="$line"; nl="${nl%"${nl##*[![:space:]]}"}"; printf '%s' "$nl"''',
    "lstrip_space": r'''n="$line"; n="${n#"${n%%[![:space:]]*}"}"; printf '%s' "$n"''',
    "hash_run_len": r'''hashes=${line%%[! #]*}; printf '%s' "${#hashes}"''',
    "tr_delete_space": r'''printf '%s' "$line" | tr -d '[:space:]' ''',
    "word_count": r'''printf '%s' "$line" | tr -s '[:space:]' '\n' | grep -c '[[:alnum:]]' ''',
    "terminal_count": r'''printf '%s' "$line" | grep -oE '[.!?]' | wc -l | tr -d ' ' ''',
    "cut_110": r'''printf '%s' "$line" | cut -c1-110''',
    # mawk's -v assignment of the line (escape processing)
    "awk_v_assign": r'''awk -v x="$line" 'BEGIN { printf "%s", x }' ''',
    # bash test -le on a string (how 002-03 compares range ends)
    "bash_test_le_30": r'''if [ "$line" -le 30 ] 2>/dev/null; then printf yes; else printf no; fi''',
}
# 002-03 unit_offenders(), verbatim, with the script's two regexes
SNIPPETS["002-03 unit_offenders"] = (
    "quantity_re=" + shlex.quote(QUANTITY_RE) + "\nunit_re=" + shlex.quote(UNIT_RE) + "\n" + UNIT_OFFENDERS_BODY
)

# File-level probes: "$f" is the fixture file. Output records are
# separated by \x1e.
FILE_PROBES = {
    "grep_lines": r'''grep -n '' "$f" | while IFS= read -r l || [ -n "$l" ]; do printf '%s\x1e' "$l"; done''',
    "line_count": r'''wc -l < "$f" | tr -d ' ' ''',
    "read_loop_from_2": r'''while IFS= read -r ln; do printf '%s\x1e' "$ln"; done < <(sed -n "2,\$p" "$f")''',
    "read_loop_from_1": r'''while IFS= read -r ln; do printf '%s\x1e' "$ln"; done < <(sed -n "1,\$p" "$f")''',
    "subst_range_1_3": r'''SUB=$(sed -n "1,3p" "$f"); printf '%s' "$SUB"''',
    "subst_range_2_9": r'''SUB=$(sed -n "2,9p" "$f"); printf '%s' "$SUB"''',
    "subst_line_2": r'''l=$(sed -n "2p" "$f"); printf '%s' "$l"''',
    "heredoc_lines_all": r'''SUB=$(sed -n "1,99p" "$f")
while IFS= read -r ln; do printf '%s\x1e' "$ln"; done <<EOF
$SUB
EOF''',
    "heading_no_d": r'''grep -Ein '^#+[[:space:]]*\(d\)' "$f" | awk -F: '/^[0-9]+:#/ { print $1; exit }' ''',
    "heading_no_open": r'''grep -Ein '^#+[[:space:]]*Open sub-questions' "$f" | awk -F: '/^[0-9]+:#/ { print $1; exit }' ''',
    "count_nonempty_sub": r'''SUB=$(sed -n "1,99p" "$f"); printf '%s\n' "$SUB" | grep -c '[^[:space:]]' ''',
    "ptr_distinct": r'''SUB=$(sed -n "1,99p" "$f"); ptrs=$(printf '%s\n' "$SUB" | grep -oE 'audio\.rs:[0-9]+' | sort -u); printf '%s\n' "$ptrs" | grep -c . || true''',
    "any_ptr_distinct": r'''SUB=$(sed -n "1,99p" "$f"); ptrs=$(printf '%s\n' "$SUB" | grep -oE '[A-Za-z0-9_./-]+\.rs:[0-9]+' | sort -u); printf '%s\n' "$ptrs" | grep -c . || true''',
}

FIXTURES = [
    "",
    "\n",
    "one\n",
    "one",
    "one\ntwo",
    "one\ntwo\n",
    "one\ntwo\n\n\n",
    "a\r\nb\r\nc\r\n",
    "a\r\nb\r\nc",
    "\n\nthird\n",
    "## (d) Reading notes\n### audio.rs\nx\n",
    "```\n## (d) in a fence\n```\n## (D) real\n",
    "text (d)\n# (d)\n",
    "##\t(d)\r\n",
    "  ## (d) indented\n## (d)\n",
    "## Open sub-questions\n| q | voice comparison bit-level | answered: yes |",
    "## Open sub-questions\n| q | voice comparison bit-level | answered: yes |\n",
    "## OPEN SUB-QUESTIONS\n",
    "audio.rs:1 audio.rs:01 audio.rs:1\naudio.rs:22\n\naudio.rs:333 x audio.rs:22\n",
    "AUDIO.RS:5\naudio.rs:6\r\n",
    "line with trailing spaces   \n\t\n \n",
    "x\ny\nz\nw\n",
    "a.rs:1 a_rs.rs:1 a-rs.rs:1 a/rs.rs:1 ./a.rs:1 a.rs:01 A.rs:1 a.rs:1² ab.rs:1 a.b.rs:1\nab.rs:2 a-b.rs:2 a_b.rs:2 A_B.rs:2 a.rs:1\n",
    "x.rs:12 x.rs:1²\nx.rs:1₂ x.rs:1⅔ x.rs:12\n",
]


def esc(b):
    """Escape bytes for a table field."""
    out = []
    try:
        s = b.decode("utf-8")
    except UnicodeDecodeError:
        return "".join(f"\\x{x:02x}" if (x < 0x20 or x >= 0x7F or x == 0x5C) and x not in (0x20,) else chr(x) for x in b)
    for c in s:
        o = ord(c)
        if c == "\\":
            out.append("\\\\")
        elif c == "\t":
            out.append("\\t")
        elif c == "\n":
            out.append("\\n")
        elif c == "\r":
            out.append("\\r")
        elif o < 0x20 or o == 0x7F:
            out.append(f"\\x{o:02x}")
        else:
            out.append(c)
    return "".join(out)


def ranges(ids):
    ids = sorted(ids)
    out, i = [], 0
    while i < len(ids):
        j = i
        while j + 1 < len(ids) and ids[j + 1] == ids[j] + 1:
            j += 1
        out.append(str(ids[i]) if i == j else f"{ids[i]}-{ids[j]}")
        i = j + 1
    return ",".join(out) or "-"


class Tools:
    def __init__(self, locale):
        self.env = dict(os.environ)
        self.env["LC_ALL"] = locale
        self.locale = locale
        self.bash = shutil.which("bash")

    def run(self, argv, data=b"", check=False):
        r = subprocess.run(argv, input=data, capture_output=True, env=self.env)
        if check and r.returncode not in (0, 1):
            sys.exit(f"{argv}: exit {r.returncode}: {r.stderr.decode(errors='replace')}")
        return r

    def versions(self):
        def first(argv):
            r = self.run(argv)
            return (r.stdout or r.stderr).decode(errors="replace").splitlines()[0].strip()

        awk = os.path.realpath(shutil.which("awk"))
        return [
            first(["grep", "--version"]),
            first(["sed", "--version"]),
            first([awk, "-W", "version"]) + f" ({awk})",
            first(["tr", "--version"]),
            first(["cut", "--version"]),
            first(["sort", "--version"]),
            first(["wc", "--version"]),
            first([self.bash, "--version"]),
        ]


def repertoire():
    rows = []
    with open(os.path.join(CRATE, "data", "repertoire.tsv"), encoding="utf-8") as f:
        for line in f:
            if line.startswith("#"):
                continue
            rows.append(chr(int(line.split("\t")[0], 16)))
    return [c for c in rows if c != "\n"]


CLASSES = ["alnum", "alpha", "digit", "space", "upper", "lower", "punct", "blank", "print", "graph", "cntrl", "xdigit"]
EXTRA = [("w", r"\w"), ("s", r"\s"), ("dot", "."), ("09", "[0-9]"), ("az", "[a-z]"), ("AZ", "[A-Z]")]


def char_facts(t, chars, tmp):
    path = os.path.join(tmp, "chars.txt")
    with open(path, "wb") as f:
        f.write(b"".join(c.encode() + b"\n" for c in chars))

    def members(flags, pat):
        r = t.run(["grep", "-n", flags, "-e", pat, path], check=True)
        return {int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l}

    cols = {}
    for k in CLASSES:
        cols["g_" + k] = members("-Ex", f"[[:{k}:]]")
        cols["gi_" + k] = members("-Exi", f"[[:{k}:]]")
    for name, pat in EXTRA:
        cols["g_" + name] = members("-Ex", pat)
        cols["gi_" + name] = members("-Exi", pat)
    fold = []
    for i, c in enumerate(chars):
        r = t.run(["grep", "-nixF", "-e", c, path], check=True)
        fold.append(sorted(int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l if int(l.split(b":")[0]) - 1 != i))
    script = r'''while IFS= read -r c || [ -n "$c" ]; do
  o=""
  for k in alnum alpha digit space upper lower punct blank; do
    case "$c" in [[:$k:]]) o="$o$k,";; esac
  done
  printf '%s\037%s\n' "$o" "${#c}"
done'''
    bash_out = t.run([t.bash, "-c", script], data=open(path, "rb").read()).stdout.split(b"\n")
    rows = []
    for i, c in enumerate(chars):
        b = c.encode()
        lower = t.run(["tr", "[:upper:]", "[:lower:]"], data=b).stdout
        deleted = t.run(["tr", "-d", "[:space:]"], data=b).stdout
        cut1 = t.run(["cut", "-c1-1"], data=b + b"\n").stdout[:-1]
        awk = t.run(["awk", '{ printf "%s\037%s\037%d\037%d\037%d\037%d", tolower($0), toupper($0), length($0), ($0 ~ /^[[:alnum:]]$/), ($0 ~ /^[[:space:]]$/), ($0 ~ /^.$/) }'], data=b + b"\n").stdout
        al, au, alen, aaln, asp, adot = awk.split(b"\x1f")
        bclasses, blen = bash_out[i].split(b"\x1f")
        rows.append(
            "\t".join(
                [
                    f"{ord(c):04X}",
                    "".join("1" if i in cols["g_" + k] else "0" for k in CLASSES + [n for n, _ in EXTRA]),
                    "".join("1" if i in cols["gi_" + k] else "0" for k in CLASSES + [n for n, _ in EXTRA]),
                    " ".join(f"{ord(chars[j]):04X}" for j in fold[i]) or "-",
                    bclasses.decode().rstrip(",") or "-",
                    blen.decode(),
                    esc(lower),
                    "1" if deleted == b"" else "0",
                    esc(cut1),
                    esc(al),
                    esc(au),
                    alen.decode(),
                    aaln.decode() + asp.decode() + adot.decode(),
                ]
            )
        )
    return rows


def range_facts(t, chars, tmp):
    """Per digit range lo-hi: the repertoire characters gnulib's DFA puts in
    it (the range alone, which the DFA supports) and those glibc's regex
    matcher puts in it: an alternative with a negated class makes the DFA
    unsupported and the superset DFA accept every line (the class becomes
    "any text"), so the regex matcher decides; the negated class itself
    matches no repertoire character."""
    path = os.path.join(tmp, "chars.txt")
    rows = []
    for lo in "0123456789":
        for hi in "0123456789":
            if hi < lo:
                continue
            out = []
            for pat in (f"[{lo}-{hi}]", f"[{lo}-{hi}]|[^[:print:][:cntrl:]]"):
                r = t.run(["grep", "-nEx", "-e", pat, path], check=True)
                ids = [int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l]
                out.append(" ".join(f"{ord(chars[i]):04X}" for i in ids) or "-")
            rows.append(f"{lo}-{hi}\t{out[0]}\t{out[1]}")
    for lo, hi in (("a", "z"), ("A", "Z")):
        out = []
        for pat in (f"[{lo}-{hi}]x|[{lo}-{hi}]", f"[{lo}-{hi}]|[^[:print:][:cntrl:]]"):
            r = t.run(["grep", "-nEx", "-e", pat, path], check=True)
            ids = [int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l]
            out.append(" ".join(f"{ord(chars[i]):04X}" for i in ids) or "-")
        rows.append(f"{lo}-{hi}\t{out[0]}\t{out[1]}")
    return rows


def inputs(commit):
    text = subprocess.run(["git", "show", f"{commit}:{REPORT}"], cwd=REPO, capture_output=True, check=True).stdout.decode()
    seen, out = set(), []
    allowed = set(repertoire())

    def add(s):
        # only repertoire characters: outside it no port runs (the guard),
        # and str.upper() makes some (µ gives the Greek capital mu)
        if s not in seen and "\n" not in s and all(c in allowed for c in s):
            seen.add(s)
            out.append(s)

    real = text.split("\n")
    for line in real:
        add(line[:140])
        if len(line) > 140:
            add(line[-90:])
    # variants of the lines the patterns aim at
    for line in real:
        if not line.strip():
            continue
        short = line[:160]
        if line.startswith("#") or line.lstrip().startswith(("-", "*", "|", ">", "1", "2", "3")):
            add(short.upper())
            add(short + "\r")
            add(short.replace(" ", "\t", 2))
            add("  " + short + "  ")
    # digits next to their superscript, subscript and fraction variants,
    # where the DFA and the regex matcher disagree on [0-9]
    variant = {"1": "¹", "2": "²", "3": "³", "4": "⁴", "5": "⁵", "6": "⁶", "9": "⁹"}
    sub = {"1": "₁", "2": "₂", "3": "₃"}
    digit_lines = [l[:120] for l in real if any(ch.isdigit() for ch in l)][:120]
    for line in digit_lines:
        add("".join(variant.get(ch, ch) for ch in line))
        add("".join(sub.get(ch, ch) for ch in line))
        add("".join(ch + variant.get(ch, "") for ch in line))
    for a in ["1", "6", "²", "⁹", "₃", "⅔", "⅓"]:
        for b in ["", "²", "⁴", "⁹", "₁", "⅓", "7"]:
            add(f"audio.rs:{a}{b}")
            add(f"see audio.rs:604 and audio.rs:{a}{b}")
            add(f"{a}.{b}5 1.5")
            add(f"{a}{b}.5")
            add(f"x {a}.5{b} and 2.7 s")
    keywords = set()
    for _, pat, _ in PATTERNS:
        for word in pat.replace("\\", " ").replace("|", " ").replace("(", " ").replace(")", " ").replace("^", " ").replace("$", " ").split():
            w = "".join(ch for ch in word if ch.isalnum() or ch in "._:-[]? ")
            if len(w) >= 3:
                keywords.add(w)
    keywords |= {"confirmed", "corrected", "yes", "no", "eps", "e.g.", "i.e.", "audio.rs:604", "audio.rs:", "[code]", "[measured]",
                 "Verdict statement:", "Voice comparison bit-level: yes", "voice comparison bit-level", "answered", "1.5", "2.7 s",
                 "1 s", "3 s", "1-second", "3-second", "SETTLE_SECS", "MAX_LAG_WINDOWS", "1 024", "1,024", "voices=false"}
    contexts = ["", " ", "_", "x", "0", ".", "-", "\t", "\r", "µ", "²", "—"]
    for k in sorted(keywords):
        for c in contexts:
            add(c + k + " end")
            add("start " + k + c)
        add(k.upper())
        add(k.capitalize())
    hand = [
        "", " ", "\t", "\r", " \r", "#", "##", "# (d)", "## (d)", "##(d)", "## (D) x", "#\t(d)", " ## (d)", "## (d)\r", "x ## (d)",
        "## Open sub-questions", "## open SUB-QUESTIONS", "- item", "-item", "* item", "+ item", "> quote", "1. one", "12) two",
        "1.two", " | row |", "|", "\t- tab item", "-\titem", "**bold**", "** bold **", "- **Verdict statement:** x", "> - x",
        "*  x", "- - x", "-  - x", "Verdict statement: one two three four five six.", "Verdict statement:", "Verdict statement:   ",
        "a.b.c", "e.g. i.e. eg ie", "x e.g.", "xe.g.", "_e.g.", "µe.g.", "1.2.3", "3.14 and 2.71", "audio.rs:12.5",
        "[code][measured] [kernel-doc] [memory] [decision] [standard] [CODE]", "Done! Really? Yes.", "...", "?!",
        "yes", "no", "No.", "yes/no", "noyes", "eyes", "know", "no_", "_no", "µno", "²no", "Δno", "no²", "noµ",
        "confirmedly", "unconfirmed", "(confirmed)", "Confirmed.", "CORRECTED", "µconfirmed", "²confirmed", "confirmed²",
        "eps", "epsilon", "eps_", "epsµ", "eps²", "eps-", "Eps.", "EPS", "steps", "eps1",
        "1024", "1 024", "1,024", "1.024", "10240", "2.7", "2,7", "2;7", "256", "1s", "1 s", "1-s", "1  s", "3s", "13 s",
        "voices flag", "voices-flag", "voicesflag", "voices are off", "flag off", "voices = false", "voices=false", "VOICES OFF",
        "sender gap", "sender-gap", "sendergap", "sender  gap", "lag", "flag", "MAX_LAG_WINDOWS", "settle", "Settling",
        "because", "since", "reason", "so that", "to avoid", "otherwise", "so  that",
        "voice comparison bit-level", "Voice Comparison Bit-Level", "voice comparison bit level", "answered", "ANSWERED",
        "x\ty", "a  b", "trailing   ", "\ttab lead", "§ ± × ÷ – — ‖ … ‰ ⅓ ⅔ → ↔ ⇒ ∈ − ∞ ≈ ≠ ≡ ≤ ≥ ⌈ ⌉ ⌊ ⌋ · ⁻ ₁₂₃ ⁴⁵⁶⁹ ²³¹",
        "µs Δt Σx α π τ", "ΔΣαπτµ", "a²b", "word² word", "² ³ ¹", "x — y", "→", "Ünïcödé-free", "back\\slash", "a\\b",
        "\\.", "[.]", "a{b", "{", "}", "$", "^", "a$b", "a^b", "( ) [ ] { } * + ? | \\ / . ^ $",
    ]
    hand += ["30", "030", " 30", "30 ", "+30", "-5", "1e3", "09", "0x1F", "99999999999999999999", "31", "7\t", "\t7", "3²",
             "STATE_LEN = 7", "VOLUME = 0", "T_STATE = 0", "`ROOM` = 64", "x ÷ 2 period", "x × 2 delay", "a ^ 2 size", "1 << 3 rate",
             "period 10 ms", "rate 48 kHz", "delay 1.5 s", "delay 1,5 s", "size 12 bytes", "window 1024", "every 5", "budget 3 µs",
             "length 12 (audio.rs:12-14)", "C-30 to C-33", "C-30..C-33", "C-30-C-33", "C-30 through C-33", "C-3 to C-033",
             "00:01:02;03 time", "Q32.32 and Q1.31", "rate 2-3 Hz", "period 0x10", "constant ONE = 1", "LISTEN_NS = 1500000000",
             "election 1 500 000 000 ns", "listen 1.5 s before", "1.5s", "1.5 sec", "1500 ms", "1_500_000_000 ns", "1.5²s",
             "not in these files", "Not In The Two Files", "absent from both", "not found in either", "verdict: pending",
             "**verdict:** pending  **", "  verdict: instrument bug, mechanism: x", "Verdict: pending", "verdict:pending\r",
             "open item: owner x falsifier y", "no open item", "TODO later", "the tbd value", "unresolved", "nothing is open",
             "x.rs:1 y.rs:2 a/b.rs:3 ./c.rs:4 x.rs:01", "transport.rs:12 xtransport.rs:13 control.rs:9", "Transport.rs:1",
             "it differs (002-13)", "differed", "matching", "mismatch", "the sender is the reference", "who acts as reference",
             "sender.rs:1 only", "is_reference via module.rs", "events output", "it sends events", "sent", "resend",
             "a\\.b", "x\\ty", "q\\\"r", "s\\/t", "u\\\\v", "period 5\\n"]
    hand += ["sender.rs:435 rx_lost u32 packets header [code]", "sender.rs:4350", "sender.rs:435²", "tui.rs:107", "tui.rs:115",
             "tui.rs:11²", "tui.rs:10⁹", "tui.rs:114x", "stats.rs:324 lost_fwd u64 count [code]", "32-bit", "32 bit", "4-byte",
             "u32x", "64bit", "8 byte", "no unit", "no conversion", "unchanged", "unitless", "one-line display", "lost_total",
             "the latest record wins", "overwritten", "superseded", "pooled across all streams", "no per-stream separation",
             "regardless of the stream", "single counter", "TX pacing lateness 3 µs sender.rs:12 [code]", "tx_late in ns",
             "[standard]", "[Standard]", "µs", "us", "msx", "_ms"]
    for h in hand:
        add(h)
    return out


def grep_flags(tool):
    kind = tool.split("-")[1]
    flags = ["-" + kind[0]]
    if "i" in kind[1:]:
        flags.append("-i")
    if "o" in kind[1:]:
        flags.append("-o")
    return flags


def corpus(t, lines, tmp):
    path = os.path.join(tmp, "inputs.txt")
    with open(path, "wb") as f:
        f.write(b"".join(l.encode() + b"\n" for l in lines))
    out = [f"I\t{i}\t{esc(l.encode())}" for i, l in enumerate(lines)]
    for tool, pat, _ in PATTERNS:
        key = f"{tool}\t{esc(pat.encode())}"
        if tool.startswith("grep") and "o" not in tool.split("-")[1][1:]:
            r = t.run(["grep", "-n"] + grep_flags(tool) + ["-e", pat, path], check=True)
            ids = {int(l.split(b":")[0]) - 1 for l in r.stdout.split(b"\n") if l}
            out.append(f"Q\t{key}\t{ranges(ids)}")
        elif tool.startswith("grep"):
            r = t.run(["grep", "-n"] + grep_flags(tool) + ["-e", pat, path], check=True)
            per = {}
            for l in r.stdout.split(b"\n"):
                if not l:
                    continue
                n, _, m = l.partition(b":")
                per.setdefault(int(n) - 1, []).append(m)
            outputs = [b"\x1f".join(per.get(i, [])) for i in range(len(lines))]
            out.append(f"OD\t{key}\t{digests(outputs)}")
        elif tool.startswith("awk-"):
            out.extend(awk_rows(t, tool, pat, key, path))
        elif tool == "sed-E":
            r = t.run(["sed", "-E", pat, path], check=True)
            got = r.stdout.split(b"\n")
            if got and got[-1] == b"":
                got.pop()
            assert len(got) == len(lines), (pat, len(got), len(lines))
            out.append(f"SD\t{key}\t{digests(got)}")
        else:
            sys.exit(f"unknown tool {tool}")
    return out


AWK_PROGRAMS = {
    "awk-test": '{ print ($0 ~ ENVIRON["RE"]) ? 1 : 0 }',
    "awk-match": '{ if (match($0, ENVIRON["RE"])) print RSTART ":" RLENGTH; else print "-" }',
    "awk-gsub": '{ s = $0; gsub(ENVIRON["RE"], "", s); print s }',
}


def awk_rows(t, tool, pat, key, path):
    """mawk on every input, the regex VALUE passed through ENVIRON (no -v
    escape processing): AT ids that match, AM per-input RSTART:RLENGTH,
    AG digests of the gsub outputs."""
    env = dict(t.env)
    env["RE"] = pat
    with open(path, "rb") as f:
        r = subprocess.run(["awk", AWK_PROGRAMS[tool]], stdin=f, capture_output=True, env=env)
    if r.returncode != 0:
        sys.exit(f"awk {pat!r}: {r.stderr.decode(errors='replace')}")
    got = r.stdout.split(b"\n")
    if got and got[-1] == b"":
        got.pop()
    if tool == "awk-test":
        return [f"AT\t{key}\t{ranges([i for i, g in enumerate(got) if g == b'1'])}"]
    if tool == "awk-match":
        return [f"AMD\t{key}\t{digests(got)}"]
    return [f"AG\t{key}\t{digests(got)}"]


def snippets(t, lines, tmp):
    """Transforms: an H row where the output differs from the input, and one
    HS row naming the inputs run. Counters: one HC row with every output."""
    path = os.path.join(tmp, "snip.txt")
    ids = [i for i, l in enumerate(lines) if len(l) <= 200]
    with open(path, "wb") as f:
        f.write(b"".join(lines[i].encode() + b"\n" for i in ids))
    out = []
    for name, body in SNIPPETS.items():
        script = f'while IFS= read -r line; do\n{body}\nprintf "\\x1e\\n"\ndone < "$1"\n'
        r = t.run([t.bash, "-c", script, "snip", path], check=False)
        got = r.stdout.split(b"\x1e\n")
        if got and got[-1] == b"":
            got.pop()
        assert len(got) == len(ids), (name, len(got), len(ids), r.stderr[:300])
        out.append(f"HD\t{name}\t{ranges(ids)}\t" + digests(got))
    return out


def fnv(data):
    """FNV-1a, 64 bits."""
    h = 0xCBF29CE484222325
    for b in data:
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def digests(outputs):
    """One FNV-1a digest per chunk of CHUNK outputs, each output followed by
    \x1e: a difference is located to its chunk without committing every
    output."""
    out = []
    for k in range(0, len(outputs), CHUNK):
        chunk = b"".join(o + RS for o in outputs[k : k + CHUNK])
        out.append(f"{fnv(chunk):016x}")
    return ",".join(out)


CHUNK = 64
RS = b"\x1e"


def files(t, tmp):
    out = [f"F\t{i}\t{esc(x.encode())}" for i, x in enumerate(FIXTURES)]
    for i, text in enumerate(FIXTURES):
        f = os.path.join(tmp, f"fixture{i}.md")
        with open(f, "wb") as h:
            h.write(text.encode())
        for name, body in FILE_PROBES.items():
            r = t.run([t.bash, "-c", body.replace('"$f"', '"$1"'), "probe", f])
            out.append(f"R\t{name}\t{i}\t{esc(r.stdout)}")
    return out


def generate(locale, commit):
    t = Tools(locale)
    with tempfile.TemporaryDirectory(prefix="gen-family002-") as tmp:
        chars = repertoire()
        crows = char_facts(t, chars, tmp)
        rrows = range_facts(t, chars, tmp)
        lines = inputs(commit)
        rows = corpus(t, lines, tmp) + snippets(t, lines, tmp) + files(t, tmp)
    head = [f"# locale {locale}; inputs seeded from {REPORT} at {commit}"] + [f"# tool: {v}" for v in t.versions()]
    chars_tsv = head + [
        "# cp\tgrep -E classes\tgrep -Ei classes\tgrep -iF partners\tbash classes\tbash ${#c}\ttr upper->lower\ttr -d [:space:] deletes\tcut -c1-1\tawk tolower\tawk toupper\tawk length\tawk alnum,space,dot",
        "# class columns, one digit each: " + " ".join(CLASSES + [n for n, _ in EXTRA]),
    ] + crows
    patterns_tsv = head + ["# tool\tpattern\torigin"] + [f"{tool}\t{esc(p.encode())}\t{o}" for tool, p, o in PATTERNS]
    corpus_tsv = head + rows
    ranges_tsv = head + ["# range\tgnulib DFA members\tglibc regex members (code points)"] + rrows
    return {"chars.tsv": chars_tsv, "patterns.tsv": patterns_tsv, "corpus.tsv": corpus_tsv, "ranges.tsv": ranges_tsv}


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    env_locale = os.environ.get("LC_ALL") or os.environ.get("LC_CTYPE") or os.environ.get("LANG") or "C"
    ap.add_argument("--locale", default=env_locale)
    ap.add_argument("--commit", default="HEAD")
    ap.add_argument("--out", default=os.path.join(CRATE, "data", "family002"))
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    if a.check and a.commit == "HEAD":
        with open(os.path.join(a.out, "corpus.tsv"), encoding="utf-8") as f:
            a.commit = f.readline().split(" at ")[1].strip()
    commit = subprocess.run(["git", "rev-parse", "--short=12", a.commit], cwd=REPO, capture_output=True, check=True, text=True).stdout.strip()
    tables = generate(a.locale, commit)
    if a.check:
        bad = 0
        for name, rows in tables.items():
            with open(os.path.join(a.out, name), encoding="utf-8") as f:
                have = [l.rstrip("\n") for l in f if not l.startswith("#")]
            want = [r for r in rows if not r.startswith("#")]
            if have != want:
                bad = 1
                print(f"{name}: differs ({len(have)} rows committed, {len(want)} regenerated)")
        sys.exit(bad)
    os.makedirs(a.out, exist_ok=True)
    for name, rows in tables.items():
        with open(os.path.join(a.out, name), "w", encoding="utf-8", newline="\n") as f:
            f.write("\n".join(rows) + "\n")
        print(f"{name}: {len(rows)} lines", file=sys.stderr)


if __name__ == "__main__":
    main()
