#!/usr/bin/env python3
"""Initialise or update swarm/ledger.json with ticket classifications.

Classification (kickoff section 7):
  host        - runs on this Linux machine, unprivileged: do it fully
  privileged  - needs root, real-time scheduling or a clock step, this machine
                only: nightly privileged job (C-146)
  hardware    - needs a second machine, macOS, Windows, an audio interface,
                a measurement rig or the embedded board: build and test what
                can be done here, then queue (never "done")

Epic defaults follow kickoff section 7; keyword overrides refine them. Team
leads confirm or correct the class when scheduling a ticket; corrections are
made by editing ledger.json directly (then committed).
"""

import json
import re
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LEDGER = REPO / "swarm/ledger.json"
GRAPH = REPO / "swarm/graph.json"

# Epic defaults.
EPIC_DEFAULT = {
    "001": "hardware",   # macOS/Windows probes; linux-peer and citation parts host
    "002": "host",       # POC archaeology runs here, unprivileged
    "003": "host",       # POC smoke fixes, unprivileged runs
    "004": "host",
    "005": "host",
    "006": "host",
    "007a": "host",
    "007b": "host",
    "008a": "host",     # simulation; on-target RT checks are privileged
    "008b": "host",
    "009": "host",
    "010": "host",
    "011": "host",
    "012": "host",
    "013": "host",
    "014": "host",
    "015a": "host",     # strict-mode tickets privileged (keyword)
    "015b": "host",
    "016": "host",       # RT verification tickets privileged (keyword)
    "017a": "host",
    "017b": "host",
    "018": "hardware",  # ALSA edge: needs an audio interface for real runs
    "019": "hardware",  # L1 two-machine
    "020": "hardware",  # L2 certification rig
    "021": "hardware",  # macOS
    "022a": "hardware", # Windows
    "022b": "hardware", # Windows L2
    "023": "host",
    "024a": "host",
    "024b": "host",
    "025a": "host",
    "025b": "hardware", # cross-target (x86-64 and 64-bit ARM) and per-target calibration
    "026a": "host",     # no_std cross-compile is host; on-device parts queued (keyword)
    "026b": "hardware", # reference device needs the board
    "027": "hardware",  # L3 rig
    "028a": "host",
    "028b": "hardware", # survey/certification needs the rigs (parts host)
    "029": "host",
}

PRIVILEGED_KEYWORDS = [
    r"strict mode", r"real-time schedul", r"\bSCHED_", r"\bmlock\b",
    r"memory lock", r"clock step", r"\broot\b", r"privileged", r"\bCAP_\w+",
    r"rt setup", r"real-time setup",
]

HARDWORD_KEYWORDS = [
    r"macos", r"core audio", r"wasapi", r"windows", r"second machine",
    r"two-machine", r"cross-target", r"\barm\b", r"aarch64", r"embedded board",
    r"reference device", r"audio interface", r"measurement rig", r"l1 ",
    r"l2 ", r"l3 ", r"certification run", r"one-hour", r"on-target", r"on the board",
    r"hardware queue", r"alcove", r"switch", r"peer machine",
]

HOST_FORCE_EPIC_001 = re.compile(r"linux|ethernet bit.?error|report|constants|fuzz|capabilit", re.I)


def classify(tid, ticket):
    epic = ticket["epic"]
    title = tid + " " + ticket["title"]
    text = Path(REPO / ticket["file"]).read_text().lower()

    if epic == "001" and HOST_FORCE_EPIC_001.search(title):
        return "host"
    for kw in PRIVILEGED_KEYWORDS:
        if re.search(kw, title, re.I):
            return "privileged"
    for kw in HARDWORD_KEYWORDS:
        if re.search(kw, title, re.I):
            return "hardware"
    return EPIC_DEFAULT.get(epic, "host")


def main():
    graph = json.loads(GRAPH.read_text())
    ledger = {}
    if LEDGER.exists():
        ledger = json.loads(LEDGER.read_text())
    for tid, ticket in graph["tickets"].items():
        if tid in ledger and ledger[tid].get("status") not in ("todo", None):
            entry = ledger[tid]  # preserve live state
            entry.setdefault("class", classify(tid, ticket))
            continue
        ledger[tid] = {
            "status": "todo",
            "class": classify(tid, ticket),
            "epic": ticket["epic"],
            "phase": graph["epics"][ticket["epic"]]["phase"] if ticket["epic"] in graph["epics"] else "?",
            "branch": f"t/{tid}",
        }
    LEDGER.write_text(json.dumps(ledger, indent=1, sort_keys=True))
    counts = {}
    for v in ledger.values():
        counts[v["class"]] = counts.get(v["class"], 0) + 1
    print(f"ledger: {len(ledger)} tickets, classes: {counts}")


if __name__ == "__main__":
    main()
