#!/usr/bin/env python3
"""Print a ticket brief: the ticket file plus the constants rows it cites.

Usage: python3 swarm/brief.py <ticket-id> [--files]
The brief is what a team lead hands to a tester, coder or reviewer:
the ticket file content is referenced by path, and every C-nn / P-nn id
cited in the ticket is expanded from 02-constants.md (single source of
truth). Nothing else is pasted into a fast model (kickoff section 6).
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CONST = REPO / "doc/design/02-constants.md"
GRAPH = REPO / "swarm/graph.json"

import json


def rows_for(ids):
    text = CONST.read_text()
    out = []
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        m = re.match(r"^\|\s*(C-\d+|P-\d+)\s*\|", line)
        if m and m.group(1) in ids:
            out.append(line)
    return out


def main():
    tid = sys.argv[1]
    graph = json.loads(GRAPH.read_text())
    ticket = graph["tickets"][tid]
    path = REPO / ticket["file"]
    body = path.read_text()
    ids = set(re.findall(r"\b([CP]-\d+)\b", body))
    print(f"# Brief for ticket {tid}")
    print(f"Ticket file: {ticket['file']}")
    print(f"Epic: {ticket['epic']} · Effort: {ticket['effort']} · Depends on: "
          + ", ".join(d["raw"] for d in ticket["depends"]))
    print()
    print("## Cited constants rows (from doc/design/02-constants.md, the single source of truth)")
    rows = rows_for(ids)
    for r in rows:
        print(r)
    missing = ids - {re.match(r"^\|\s*(C-\d+|P-\d+)\s*\|", r).group(1) for r in rows}
    if missing:
        print(f"(ids cited but with no table row: {sorted(missing)})")
    print()
    print("Read the ticket file in full: it repeats every rule it needs.")


if __name__ == "__main__":
    main()
