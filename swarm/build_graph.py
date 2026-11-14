#!/usr/bin/env python3
"""Build the ticket dependency graph from the design set.

Parses every ticket file in doc/design/tickets/<epic>/ and every epic file in
doc/design/implementation/, extracts the "Depends on" lines, and writes
swarm/graph.json. Run after any change to the design set.

Ticket status lives in swarm/ledger.json (maintained by the orchestrator and
integrator); this script only reads it to report the runnable frontier.
"""

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TICKETS = REPO / "doc/design/tickets"
IMPL = REPO / "doc/design/implementation"
OUT = REPO / "swarm/graph.json"
LEDGER = REPO / "swarm/ledger.json"

TICKET_RE = re.compile(r"^(\d{3}[a-z]?-\d{2})\b")
HEADER_RE = re.compile(
    r"^\*\*Epic:\*\*\s+(\S+)\s+·\s+\*\*Effort:\*\*\s+(\S+)\s+·\s+\*\*Depends on:\*\*\s+(.*)$"
)
EPIC_HEADER_RE = re.compile(
    r"^\*\*Phase:\*\*\s+(.+?)\s+·\s+\*\*Scope:\*\*\s+(.+?)\s+·\s+\*\*Depends on:\*\*\s+(.*?)(?:\s+·\s+\*\*Blocks:\*\*\s+(.*))?$"
)

def parse_dep_tokens(text):
    """Split a Depends-on field into tokens: tickets, epics, qualifiers.

    Handles: 'none', ticket ids, bare epic numbers ('002'), 'epic(s) NNN',
    ranges ('009-01 to 009-16', '024b-03 … 024b-11'), and trailing
    cross-reference clauses after '·' which are NOT dependencies (C-196).
    """
    # cross-reference clauses are not dependencies (C-196)
    text = text.split("·")[0].strip()
    tokens = []

    def add_ticket(tid):
        tokens.append(("ticket", tid, tid))

    def add_epic(eid):
        tokens.append(("epic", eid, eid))

    parts = [p.strip() for p in text.split(",") if p.strip()]
    i = 0
    while i < len(parts):
        raw = parts[i]
        # range: "X to Y" may be split across comma parts or within one
        m = re.match(r"^(\d{3}[a-z]?-\d+)\s*(?:to|…|\.\.\.)\s*(\d{3}[a-z]?-\d+)$", raw)
        if m:
            a, b = m.group(1), m.group(2)
            pa, ea = a.split("-")
            pb, eb = b.split("-")
            if pa == pb:
                for n in range(int(a.split("-")[1]), int(b.split("-")[1]) + 1):
                    add_ticket(f"{pa}-{n:02d}")
                i += 1
                continue
        # range inside one comma part, e.g. "009-01 to 009-16"
        m = re.match(r"^(\d{3}[a-z]?-\d+)\s+(?:to|…|\.\.\.)\s+(\d{3}[a-z]?-\d+)$", raw)
        if m:
            pa = m.group(1).split("-")[0]
            lo = int(m.group(1).split("-")[1])
            hi = int(m.group(2).split("-")[1])
            for n in range(lo, hi + 1):
                add_ticket(f"{pa}-{n:02d}")
            i += 1
            continue
        m = re.match(r"^epics?\s+(\d{3}[a-z]?)(\s*\(.*\))?$", raw, re.IGNORECASE)
        if m:
            add_epic(m.group(1))
            i += 1
            continue
        m = re.match(r"^(\d{3}[a-z]?-\d+)$", raw)
        if m:
            add_ticket(m.group(1))
            i += 1
            continue
        m = re.match(r"^(\d{3}[a-z]?)$", raw)
        if m:
            add_epic(m.group(1))
            i += 1
            continue
        if raw.lower() == "none":
            i += 1
            continue
        tokens.append(("other", raw, raw))
        i += 1
    return tokens


def main():
    epics = {}       # epic id -> {phase, depends: [(kind, id, raw)]}
    tickets = {}     # ticket id -> {epic, title, effort, depends: [(kind, id, raw)]}
    ticket_files = {}

    # epic files
    for p in sorted(IMPL.glob("*.md")):
        epic_id = p.name.split("-")[0]
        for line in p.read_text().splitlines():
            m = EPIC_HEADER_RE.match(line)
            if m:
                dep_text = m.group(3)
                # strip qualifiers like "(report, non-blocking)" while keeping them
                tokens = []
                for part in dep_text.split(","):
                    part = part.strip()
                    if not part:
                        continue
                    em = re.match(r"^(epic\s+)?(\d{3}[a-z]?)(\s*\(.*\))?$", part, re.IGNORECASE)
                    if em:
                        qual = (em.group(3) or "").strip("() \t")
                        blocking = "non-blocking" not in qual
                        tokens.append({"id": em.group(2), "qualifier": qual, "blocking": blocking})
                    else:
                        tokens.append({"id": part, "qualifier": "", "blocking": True, "unparsed": True})
                epics[epic_id] = {"phase": m.group(1), "depends": tokens, "file": str(p.relative_to(REPO))}
                break

    # ticket files
    for p in sorted(TICKETS.glob("*/*.md")):
        if p.name == "README.md":
            continue
        m = TICKET_RE.match(p.stem)
        if not m:
            print(f"WARN: unparsable ticket filename {p}")
            continue
        tid = m.group(1)
        epic_id = tid.split("-")[0]
        header = None
        for line in p.read_text().splitlines():
            hm = HEADER_RE.match(line)
            if hm:
                header = hm
                break
        if header is None:
            print(f"WARN: no header line in {p}")
            continue
        tokens = parse_dep_tokens(header.group(3))
        tickets[tid] = {
            "epic": epic_id,
            "title": p.stem.split("-", 2)[2].replace("-", " ") if len(p.stem.split("-", 2)) == 3 else p.stem,
            "effort": header.group(2),
            "depends": [{"kind": k, "id": i, "raw": r} for (k, i, r) in tokens],
            "file": str(p.relative_to(REPO)),
        }
        ticket_files[tid] = p

    # validate
    # bare epic numbers reference all parts of a split epic (C-143)
    def epic_parts(eid):
        if eid in epics:
            return [eid]
        return sorted(k for k in epics if k.split("-")[0].rstrip("ab") == eid or k == eid)

    errors = []
    for tid, t in tickets.items():
        for d in t["depends"]:
            if d["kind"] == "ticket" and d["id"] not in tickets:
                errors.append(f"{tid} depends on missing ticket {d['id']}")
            if d["kind"] == "epic" and not epic_parts(d["id"]):
                errors.append(f"{tid} depends on missing epic {d['id']}")
            if d["kind"] == "other":
                errors.append(f"{tid} has unparsable dep {d['raw']!r}")
    # cycles (ticket level)
    adj = {tid: [d["id"] for d in t["depends"] if d["kind"] == "ticket"] for tid, t in tickets.items()}
    WHITE, GREY, BLACK = 0, 1, 2
    colour = {tid: WHITE for tid in tickets}
    cycles = []

    def visit(node, stack):
        colour[node] = GREY
        for nxt in adj.get(node, []):
            if nxt not in colour:
                continue
            if colour[nxt] == GREY:
                cycles.append(stack + [node, nxt])
            elif colour[nxt] == WHITE:
                visit(nxt, stack + [node])
        colour[node] = BLACK

    for tid in tickets:
        if colour[tid] == WHITE:
            visit(tid, [])

    if errors:
        for e in errors:
            print("ERROR:", e)
    if cycles:
        for c in cycles:
            print("CYCLE:", " -> ".join(c))

    # ledger-driven frontier
    ledger = {}
    if LEDGER.exists():
        ledger = json.loads(LEDGER.read_text())
    done = {tid for tid, st in ledger.items() if st.get("status") == "done"}

    def epic_closed(eid):
        parts = epic_parts(eid)
        if not parts:
            return False
        for part in parts:
            ts = [tid for tid, t in tickets.items() if t["epic"] == part]
            if not ts:
                return False
            if not all(ledger.get(tid, {}).get("status") in ("done", "hardware-ready") for tid in ts):
                return False
        return True

    def runnable(tid):
        t = tickets[tid]
        if ledger.get(tid, {}).get("status") in ("done", "hardware-ready", "in-flight"):
            return False
        for d in t["depends"]:
            if d["kind"] == "ticket" and d["id"] not in done:
                return False
            if d["kind"] == "epic" and not epic_closed(d["id"]):
                return False
        return True

    frontier = sorted(tid for tid in tickets if runnable(tid))
    # frontier grouped by phase; phase gating beyond this is the
    # orchestrator's call (roadmap entry criteria are finer than a DAG)
    frontier_by_phase = {}
    for tid in frontier:
        ph = epics.get(tickets[tid]["epic"], {}).get("phase", "?")
        frontier_by_phase.setdefault(ph, []).append(tid)
    # epic-level summary
    epic_summary = {}
    for eid, e in sorted(epics.items()):
        ts = [tid for tid, t in tickets.items() if t["epic"] == eid]
        st = {s: sum(1 for tid in ts if ledger.get(tid, {}).get("status") == s)
              for s in ("todo", "in-flight", "done", "hardware-ready", "blocked")}
        st["total"] = len(ts)
        st.setdefault("todo", len(ts) - sum(v for k, v in st.items() if k != "total"))
        epic_summary[eid] = {"phase": e["phase"], **st}

    graph = {
        "tickets": tickets,
        "epics": epics,
        "frontier": frontier,
        "frontier_by_phase": frontier_by_phase,
        "epic_summary": epic_summary,
        "errors": errors,
        "cycles": cycles,
    }
    OUT.write_text(json.dumps(graph, indent=1))
    print(f"{len(tickets)} tickets, {len(epics)} epics, frontier: {len(frontier)}")
    if errors or cycles:
        sys.exit(1)


if __name__ == "__main__":
    main()
