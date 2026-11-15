#!/usr/bin/env python3
"""Summarise an lcov.info file: line and branch coverage totals.

Writes to $GITHUB_STEP_SUMMARY when present, and always prints to stdout.
Exit 1 if the file is missing or empty (a coverage run with zero samples
is a failure, never a pass).
"""

import os
import sys

path = sys.argv[1] if len(sys.argv) > 1 else "lcov.info"
if not os.path.isfile(path) or os.path.getsize(path) == 0:
    print(f"no coverage data in {path}")
    sys.exit(1)

lines_hit = lines_total = 0
br_hit = br_total = 0
for line in open(path, encoding="utf-8", errors="replace"):
    if line.startswith("DA:"):
        parts = line.strip().split(",")
        lines_total += 1
        if int(parts[1]) > 0:
            lines_hit += 1
    elif line.startswith("BRDA:"):
        parts = line.strip().split(",")
        br_total += 1
        if int(parts[3]) > 0:
            br_hit += 1

lp = (100 * lines_hit // lines_total) if lines_total else 0
bp = (100 * br_hit // br_total) if br_total else 0
out = [
    f"**Coverage**: lines {lp}% ({lines_hit}/{lines_total})",
    f"**Branches**: {bp}% ({br_hit}/{br_total})",
]
text = "\n".join(out)
print(text)
summary = os.environ.get("GITHUB_STEP_SUMMARY")
if summary:
    with open(summary, "a", encoding="utf-8") as f:
        f.write(text + "\n")
