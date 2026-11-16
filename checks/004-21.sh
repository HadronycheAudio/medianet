#!/usr/bin/env bash
# 004-21 — validity rules completeness audit.
#
# RED-phase document check. It validates the "## Validity rules" section of
# doc/design/spec/wire.md (the consolidated matrix, the uniform outcome, the
# bounded-parsing statements and the hand-offs) against wire.toml and the
# closed enumeration, then re-runs the 004-02 checker once (cargo run --bin
# wire-check and the wire_tables tests). It never runs the other
# checks/004-*.sh (a sibling loop would be quadratic in the per-change suite)
# and never depends on the main git ref.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either path is overridden the
# repository-level checks (cargo and the design checker) are skipped with a
# NOTE, because they always target the real repository.
#
# Nothing is pinned to today's counts: the in-scope message set, the
# enumeration, the exempt set and the open items are all read at run time.
#
# What is checked, per acceptance criterion:
#
#   AC 1  The Validity rules section holds a table whose header names the
#         twelve invalid-form classes (matched loosely by keyword, in any
#         order, extra columns allowed). Its rows are the in-scope message
#         types: every in-scope wire.toml message is named by exactly one row
#         (case-insensitively, by its wire.toml name, with the "regime stamp"
#         prefix optional for section types). Every cell is non-empty and is
#         one of: one or more names of the closed enumeration, a named event
#         ("event"), or "not applicable" / "n/a" followed by a reason of at
#         least three words. A bare dash, a blank or a bare n/a fails.
#   AC 2  Every reason named in a cell and every reason in a rejection_reasons
#         list is a member of the closed enumeration (wire.md "Rejection
#         reasons", cross-checked with the [[reject]] tables). Every
#         enumeration value is used at least once across the matrix and the
#         toml lists. A cell's reason has, in the enumeration's Layers cell,
#         the layer of the row's message.
#   AC 3  For every row whose message carries a version field (a field named
#         version or protocol_version at bit 16, width 16, read from the field
#         table), the wrong-version cell names VersionMismatch and a named
#         event.
#   AC 4  Every in-scope message states a maximum parse length: a figure in
#         the matrix (a column whose header names parse length or bytes
#         examined) or in another Validity rules table whose row names the
#         message. The figure equals wire.toml's max_size; a figure written as
#         arithmetic ("... = N") is read by its final number. Among control
#         messages every figure is at most the C-17 bound (read from
#         02-constants.md) except the messages wire.toml marks with
#         exempt_from_control_bound (only C-116 is accepted).
#         WORK 4: at least one statement citing C-164 covers the bounded-parsing
#         property: the statements together name maximum bytes examined, no
#         loop over an unbounded count, and work linear in packet length, and
#         each in-scope message is named by a C-164 statement.
#   AC 5  For every in-scope message, its wire.toml rejection_reasons list
#         equals the set of reasons named by its matrix row (order free).
#         cargo run --bin wire-check reports "checks clean" and
#         cargo test -p hadronyche-tools --test wire_tables reports at least
#         12 passed and 0 failed (today's count).
#   WORK 2  Each (message, reason) pair of the matrix is stated in the
#         message's own wire.md section (the ## sections mapped below) or in
#         the shared sections Versioning and Reserved fields.
#   WORK 3  One tagged paragraph of the section states the uniform outcome:
#         drop, count per (input, reason), no state change, and a named event
#         for a version mismatch (VersionMismatch).
#   OPEN  Every open item and provisional outcome outside the section that
#         names 004-21 as owner (a bold "Open item", a table row or a prose
#         paragraph that mentions 004-21) is accounted for: either it carries
#         a status sentence ("Status" and "004-21") or the Validity rules
#         section hands it on, naming the wire.md section (and the item
#         number), with a named owner other than 004-21 and a falsifier.
#   DoD   Every prose paragraph of 5+ words in the section carries an evidence
#         tag, and every table is tagged or follows a tagged paragraph.
set -u
cd "$(dirname "$0")/.."

WIRE="${WIRE_MD:-doc/design/spec/wire.md}"
TABLES="${WIRE_TOML:-doc/design/spec/wire.toml}"
python3 - "$WIRE" "$TABLES" <<'PY'
import json
import os
import re
import subprocess
import sys

wire_path, tables_path = sys.argv[1], sys.argv[2]
overridden = bool(os.environ.get("WIRE_MD") or os.environ.get("WIRE_TOML"))
CONSTANTS_PATH = "doc/design/02-constants.md"

fails = []


def fail(tag, msg):
    fails.append((tag, msg))


def read(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


wire_text = read(wire_path)
toml_text = read(tables_path)
if wire_text is None or toml_text is None:
    print(f"FAIL AC1: cannot read {wire_path} or {tables_path}")
    print("1 failure(s)")
    sys.exit(1)
lines = wire_text.splitlines()
const_text = read(CONSTANTS_PATH) or ""

EVIDENCE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)


def has_evidence(text):
    return bool(EVIDENCE.search(text))


def norm(s):
    s = s.replace("`", "").replace("*", "").lower()
    s = re.sub(r"[-_/]", " ", s)
    return re.sub(r"\s+", " ", s).strip()


def phrase_re(phrase):
    return re.compile(r"(?<![a-z0-9])" + re.escape(phrase) + r"(?:s|es)?(?![a-z0-9])")


def words(text):
    return re.findall(r"[A-Za-z]{2,}", re.sub(r"`[^`]*`", "", text))


# ---------------------------------------------------------------------------
# Markdown structure: headings, sections, units (paragraphs, rows), tables
# ---------------------------------------------------------------------------
HEAD = re.compile(r"^(#{1,6})\s+(.*?)\s*#*\s*$")


def headings(ls):
    out = []
    fence = False
    for i, line in enumerate(ls):
        if re.match(r"^\s*```", line):
            fence = not fence
            continue
        if fence:
            continue
        m = HEAD.match(line)
        if m:
            out.append((i, len(m.group(1)), m.group(2).strip()))
    return out


HEADS = headings(lines)


def sec_range(title, level=2):
    for k, (i, lvl, t) in enumerate(HEADS):
        if lvl == level and norm(t) == norm(title):
            end = len(lines)
            for j, lvl2, _ in HEADS[k + 1:]:
                if lvl2 <= level:
                    end = j
                    break
            return (i + 1, end)
    return None


def units(a, b):
    """Units of lines[a:b]: 'prose' paragraphs (blank-line separated, bullets
    start a unit), 'row' (one per table row, with table id 'tid'), and
    'heading'. Separator rows and fenced code are skipped."""
    out = []
    cur = []
    cur_start = None
    fence = False
    tid = 0
    last_kind = None

    def flush():
        nonlocal cur, cur_start
        if cur:
            out.append({"kind": "prose", "text": "\n".join(cur), "line": cur_start})
        cur = []
        cur_start = None

    for i in range(a, b):
        line = lines[i]
        if re.match(r"^\s*```", line):
            fence = not fence
            flush()
            last_kind = None
            continue
        if fence:
            continue
        s = line.strip()
        if not s:
            flush()
            last_kind = None
            continue
        if HEAD.match(line):
            flush()
            out.append({"kind": "heading", "text": s, "line": i})
            last_kind = "heading"
            continue
        if s.startswith("|"):
            flush()
            if re.match(r"^\|?[\s:|-]+\|?$", s) and "-" in s:
                continue
            if last_kind != "row":
                tid += 1
            out.append({"kind": "row", "text": s, "cells": split_row(s), "line": i, "tid": tid})
            last_kind = "row"
            continue
        if s.startswith("- ") and cur:
            flush()
        if not cur:
            cur_start = i
        cur.append(s)
        last_kind = "prose"
    flush()
    return out


def split_row(line):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    cells = re.split(r"(?<!\\)\|", s)
    return [c.strip().replace("\\|", "|") for c in cells]


def tables(us):
    """Group 'row' units by table id: [{'header': [...], 'rows': [...], 'line': n}].
    The first row of a table is its header (the separator row is not a unit)."""
    groups = {}
    order = []
    for u in us:
        if u["kind"] != "row":
            continue
        if u["tid"] not in groups:
            groups[u["tid"]] = []
            order.append(u["tid"])
        groups[u["tid"]].append(u)
    result = []
    for tid in order:
        rows = groups[tid]
        result.append({"header": rows[0]["cells"], "rows": [r["cells"] for r in rows[1:]],
                       "line": rows[0]["line"], "tid": tid})
    return result


# ---------------------------------------------------------------------------
# wire.toml: a line-based reader of the keys this check needs
# ---------------------------------------------------------------------------
def toml_val(v):
    v = v.strip()
    if not v.startswith('"') and " #" in v:
        v = v.split(" #", 1)[0].strip()
    try:
        return json.loads(v)
    except ValueError:
        pass
    if v in ("true", "false"):
        return v == "true"
    try:
        return int(v.replace("_", ""), 0)
    except ValueError:
        return v


def parse_toml(src):
    msgs = []
    rejects = []
    cur = None
    target = None
    for raw in src.splitlines():
        s = raw.strip()
        if not s or s.startswith("#"):
            continue
        if s == "[[message]]":
            cur = {"kv": {}, "fields": []}
            msgs.append(cur)
            target = cur["kv"]
            continue
        if s == "[[message.field]]":
            if cur is None:
                continue
            f = {}
            cur["fields"].append(f)
            target = f
            continue
        if s == "[[reject]]":
            r = {}
            rejects.append(r)
            target = r
            cur = None
            continue
        if s.startswith("["):
            target = None
            cur = None
            continue
        m = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$", s)
        if m and target is not None:
            target[m.group(1)] = toml_val(m.group(2))
    return msgs, rejects


toml_msgs, toml_rejects = parse_toml(toml_text)
LAYERS = {"control", "sync", "media", "measurement"}
SUBPART = re.compile(r" (?:element|entry|entries|member|fixed part|header)$")
in_scope = [m for m in toml_msgs
            if m["kv"].get("layer") in LAYERS
            and not SUBPART.search(str(m["kv"].get("name", "")))
            and m["kv"].get("name") != "example"]
by_name = {m["kv"]["name"]: m for m in in_scope}


def has_version(m):
    for f in m["fields"]:
        k = f.get("kv", {}) if "kv" in f else f
        name = str(k.get("name", ""))
        if "version" in name and k.get("offset") == 16 and k.get("width") == 16:
            return True
    return False


def aliases(name):
    out = {norm(name)}
    if name.startswith("regime stamp "):
        rest = norm(name[len("regime stamp "):])
        if len(rest.split()) >= 2:
            out.add(rest)
    return out


def names_in(label):
    nl = norm(label)
    hits = {}
    for m in in_scope:
        n = m["kv"]["name"]
        for a in aliases(n):
            if phrase_re(a).search(nl):
                hits.setdefault(n, set()).add(a)
    keep = []
    for n in hits:
        covered = False
        for o in hits:
            if o != n and any(a in norm(o) for a in aliases(n)) and norm(o) != norm(n):
                covered = True
        if not covered:
            keep.append(n)
    return sorted(keep)


def names_in_text(text):
    nt = norm(text)
    return [m["kv"]["name"] for m in in_scope
            if any(phrase_re(a).search(nt) for a in aliases(m["kv"]["name"]))]


# ---------------------------------------------------------------------------
# The closed enumeration (wire.md "Rejection reasons", Layers column)
# ---------------------------------------------------------------------------
enum = {}
rr = sec_range("Rejection reasons")
if rr is None:
    fail("AC2", "wire.md has no '## Rejection reasons' section")
else:
    for t in tables(units(*rr)):
        hdr = [h.lower() for h in t["header"]]
        if "name" in hdr and any("layers" in h for h in hdr):
            ni = hdr.index("name")
            li = [k for k, h in enumerate(hdr) if "layers" in h][0]
            for r in t["rows"]:
                if len(r) <= max(ni, li):
                    continue
                name = r[ni].strip("` ")
                layers = {x.strip().lower() for x in re.split(r"[,;]", r[li]) if x.strip()}
                enum[name] = layers
            break
if not enum:
    fail("AC2", "no enumeration table with Name and Layers columns in wire.md 'Rejection reasons'")
ENUM_NAMES = list(enum)
toml_enum = {r.get("name") for r in toml_rejects}
if set(ENUM_NAMES) != toml_enum:
    fail("AC2", f"the [[reject]] names in wire.toml and the wire.md enumeration differ: "
         f"only in wire.md {sorted(set(ENUM_NAMES) - toml_enum)}, only in wire.toml {sorted(toml_enum - set(ENUM_NAMES))}")

REASON_RE = {n: re.compile(r"(?<![A-Za-z])" + re.escape(n) + r"(?![A-Za-z])") for n in ENUM_NAMES}
CAMEL = re.compile(r"\b[A-Z][a-z]+(?:[A-Z][a-z0-9]+)+\b")
NA_RE = re.compile(r"^\s*(?:not applicable|n/a|na)\b[:\s,.-]*(.*)$", re.I)


def reasons_in(text):
    return [n for n in ENUM_NAMES if REASON_RE[n].search(text)]


def check_cell(text):
    t = text.strip()
    if not t:
        return "empty", []
    na = NA_RE.match(t)
    if na:
        return ("na", []) if len(words(na.group(1))) >= 3 else ("bare-na", [])
    rs = reasons_in(t)
    if rs:
        return "reason", rs
    if re.search(r"\bevent\b", t, re.I):
        return "event", []
    return "invalid", []


# ---------------------------------------------------------------------------
# Constants: the C-17 control bound (read, not pinned)
# ---------------------------------------------------------------------------
C17 = None
for line in const_text.splitlines():
    if line.startswith("| C-17 |"):
        m = re.search(r"at most ([\d ]+?) bytes", line)
        if m:
            C17 = int(re.sub(r"\s", "", m.group(1)))
if C17 is None:
    fail("AC4", f"cannot read the C-17 control bound from {CONSTANTS_PATH}")
    C17 = 1252

# ---------------------------------------------------------------------------
# The Validity rules section and its matrix
# ---------------------------------------------------------------------------
VR = sec_range("Validity rules")
vr_units = []
if VR is None:
    fail("AC1", "wire.md has no '## Validity rules' section")
else:
    vr_units = units(*VR)
    vr_text = "\n".join(lines[VR[0]:VR[1]])
    if re.search(r"To be filled", vr_text):
        fail("AC1", "the '## Validity rules' section still holds its 'To be filled' placeholder")

CLASSES = [
    ("truncated", r"truncat|too short"),
    ("bad magic", r"magic"),
    ("wrong version", r"version"),
    ("foreign or zero domain", r"domain"),
    ("length inconsistent", r"length"),
    ("count above maximum", r"count"),
    ("reserved non-zero", r"reserved"),
    ("unknown type code", r"type"),
    ("value out of range", r"value|range"),
    ("oversize", r"oversize|too large"),
    ("wrong epoch", r"epoch"),
    ("unknown sender or stream", r"sender|stream"),
]
FIG_RE = re.compile(r"parse|examined", re.I)


def assign_columns(header):
    cols = {}
    used = {0}
    for cname, rx in CLASSES:
        for j, h in enumerate(header):
            if j in used or FIG_RE.search(h):
                continue
            if re.search(rx, h, re.I):
                cols[cname] = j
                used.add(j)
                break
    return cols


def parse_int(text):
    if text is None:
        return None
    t = re.sub(r"(?<=\d) (?=\d{3}(?!\d))", "", text)
    if "=" in t:
        t = t.rsplit("=", 1)[1]
    m = re.findall(r"\d+", t)
    return int(m[0]) if m else None


all_tables = tables(vr_units)
best = None
for t in all_tables:
    cols = assign_columns(t["header"])
    if best is None or len(cols) > len(best[1]):
        best = (t, cols)

row_reasons_by_row = []      # (label, names, reasons set, {col: reasons})
fig_by_name = {}
mapped = {}                  # name -> [row labels]
cell_pairs = []              # (name, reason, row label, col)
matrix_names_rows = []
if best is None or len(best[1]) < 12:
    have = sorted(best[1]) if best else []
    missing = [c for c, _ in CLASSES if best is None or c not in best[1]]
    fail("AC1", "no table in '## Validity rules' has the twelve invalid-form classes as columns "
         f"(best table has {len(have)}; missing {missing})")
matrix = None
mcols = {}
if best is not None:
    matrix, mcols = best
    header = matrix["header"]
    fig_cols = [j for j, h in enumerate(header) if FIG_RE.search(h)]
    fig_col = fig_cols[0] if fig_cols else None
    if fig_col is None:
        pass  # figures may come from a bounded-parsing table (AC 4)
    ver_col = mcols.get("wrong version")
    for ri, cells in enumerate(matrix["rows"]):
        label = cells[0] if cells else ""
        rlabel = label or f"row {ri + 1}"
        names = names_in(label)
        if not label:
            fail("AC1", f"matrix row {ri + 1} has no message label")
        if label and not names:
            fail("AC1", f"matrix row '{label}' names no in-scope wire.toml message")
        for n in names:
            mapped.setdefault(n, []).append(rlabel)
        row_reasons = set()
        per_col = {}
        for j in range(1, len(header)):
            if j == fig_col:
                continue
            text = cells[j] if j < len(cells) else ""
            colname = header[j]
            kind, rs = check_cell(text)
            if kind == "empty":
                fail("AC1", f"empty cell: row '{rlabel}', column '{colname}'")
                continue
            if kind == "bare-na":
                fail("AC1", f"'n/a' without a reason (three words or more): row '{rlabel}', column '{colname}'")
                continue
            if kind == "invalid":
                fail("AC1", f"cell names neither a reason of the enumeration, a named event nor n/a with a reason: "
                     f"row '{rlabel}', column '{colname}': {text[:70]!r}")
                continue
            if kind in ("reason", "event"):
                bad = [c for c in CAMEL.findall(text) if c not in ENUM_NAMES]
                if bad:
                    fail("AC2", f"reason(s) {bad} in row '{rlabel}', column '{colname}' are not in the closed enumeration")
            if kind == "reason":
                row_reasons |= set(rs)
                per_col[colname] = rs
                for r in rs:
                    for n in names:
                        layer = by_name[n]["kv"].get("layer")
                        if layer not in enum.get(r, set()):
                            fail("AC2", f"reason {r} in row '{rlabel}' is not in the Layers of the enumeration "
                                 f"for the row's layer {layer!r} (its Layers: {sorted(enum.get(r, set()))})")
                    cell_pairs.append((names, r, rlabel, colname))
        if ver_col is not None:
            vtext = cells[ver_col] if ver_col < len(cells) else ""
            for n in names:
                if has_version(by_name[n]):
                    if not (re.search(r"VersionMismatch", vtext) and re.search(r"\bevent\b", vtext, re.I)):
                        fail("AC3", f"row '{rlabel}' ({n}) carries a version field but its wrong-version cell does not "
                             f"name VersionMismatch and a named event: {vtext[:80]!r}")
        if fig_col is not None:
            fig_text = cells[fig_col] if fig_col < len(cells) else ""
            fig = parse_int(fig_text)
            for n in names:
                if fig is None:
                    fail("AC4", f"row '{rlabel}' ({n}) states no maximum parse length")
                else:
                    fig_by_name.setdefault(n, fig)
        for n in names:
            want = set(by_name[n]["kv"].get("rejection_reasons", []) or [])
            if want != row_reasons:
                fail("AC5", f"{n}: wire.toml rejection_reasons differ from the matrix row '{rlabel}': "
                     f"only in wire.toml {sorted(want - row_reasons)}, only in the row {sorted(row_reasons - want)}")
        row_reasons_by_row.append((rlabel, names, row_reasons))
    # Bounded-parsing figures may also come from another Validity rules table.
    for t in all_tables:
        if t is matrix:
            continue
        fc = [j for j, h in enumerate(t["header"]) if re.search(r"parse|examined|maximum|bytes", h, re.I)]
        if not fc:
            continue
        for cells in t["rows"]:
            if not cells:
                continue
            for n in names_in(cells[0]):
                if n not in fig_by_name:
                    v = parse_int(cells[fc[0]] if fc[0] < len(cells) else None)
                    if v is not None:
                        fig_by_name[n] = v

for m in in_scope:
    n = m["kv"]["name"]
    rows_for = mapped.get(n, [])
    if not rows_for:
        fail("AC1", f"no matrix row names the in-scope message type '{n}'")
    elif len(rows_for) > 1:
        fail("AC1", f"the message '{n}' is named by {len(rows_for)} matrix rows")
    if n not in fig_by_name:
        fail("AC4", f"no maximum parse length is stated for '{n}' (matrix column or bounded-parsing table)")
    elif fig_by_name[n] != m["kv"].get("max_size"):
        fail("AC4", f"'{n}': stated maximum parse length {fig_by_name[n]} differs from wire.toml max_size "
             f"{m['kv'].get('max_size')}")

# AC 4: the control bound and the exempt set
exempt_set = []
for m in in_scope:
    kv = m["kv"]
    n = kv["name"]
    ex = kv.get("exempt_from_control_bound")
    if ex is not None:
        exempt_set.append(n)
        if ex != "C-116":
            fail("AC4", f"'{n}' is exempt from the control bound by '{ex}', and only C-116 is an exemption")
        if kv.get("layer") != "control":
            fail("AC4", f"'{n}' carries an exemption but is not a control message")
        continue
    if kv.get("layer") == "control":
        size = max(kv.get("max_size", 0) or 0, fig_by_name.get(n, 0) or 0)
        if size > C17:
            fail("AC4", f"control message '{n}' states {size} bytes, above the C-17 bound of {C17} bytes, "
                 "and carries no C-116 exemption")

# WORK 4: bounded parsing, citing C-164
c164 = [u for u in vr_units if u["kind"] in ("prose", "row") and "C-164" in u["text"]]
if not c164:
    fail("WORK4", "no bounded-parsing statement in 'Validity rules' cites C-164")
else:
    blob = norm(" ".join(u["text"] for u in c164))
    for prop, rx in (("maximum bytes examined", r"examined"),
                     ("no loop over an unbounded count", r"unbounded"),
                     ("work linear in packet length", r"linear")):
        if not re.search(rx, blob):
            fail("WORK4", f"the C-164 bounded-parsing statements do not state {prop}")
    for m in in_scope:
        n = m["kv"]["name"]
        if not any(any(phrase_re(a).search(norm(u["text"])) for a in aliases(n)) for u in c164):
            fail("WORK4", f"no C-164 bounded-parsing statement names '{n}'")

# ---------------------------------------------------------------------------
# WORK 2: each matrix (message, reason) pair is stated in the message's section
# ---------------------------------------------------------------------------
SECTION_MAP = [
    (r"^media packet$", ["Media packet header", "Slice header", "Announcements", "Measurement extension", "Checksum"]),
    (r"^announcement$", ["Announcements"]),
    (r"^slice checksum$", ["Checksum"]),
    (r"^(audio message|dense controller message|sparse keyed controller message|transport state message)$",
     ["Content messages"]),
    (r"^sync (request|reply)$", ["Sync request and reply"]),
    (r"^receiver report", ["Receiver report"]),
    (r"^one-way probe", ["One-way probe"]),
    (r"^hello$", ["Hello"]),
    (r"^(candidacy|holder announcement)$", ["Election messages"]),
    (r"^admission ", ["Admission messages"]),
    (r"^manifest section$", ["Manifest"]),
    (r"^path-MTU probe ", ["Path-MTU probe pair"]),
    (r"^regime stamp", ["Regime stamp"]),
    (r"^commissioning window ", ["Commissioning window messages"]),
]
SHARED = ["Versioning", "Reserved fields"]
w2_unmapped = {}
for names, reason, rlabel, colname in cell_pairs:
    for n in names:
        target = None
        for rx, secs in SECTION_MAP:
            if re.search(rx, n):
                target = secs
                break
        if target is None:
            fail("WORK2", f"no wire.md section is mapped for message '{n}'")
            continue
        text = ""
        for title in target + SHARED:
            rg = sec_range(title)
            if rg is None:
                fail("WORK2", f"wire.md has no '## {title}' section (mapped for '{n}')")
                continue
            text += "\n".join(lines[rg[0]:rg[1]]) + "\n"
        if not REASON_RE.get(reason, re.compile("$^")).search(text):
            w2_unmapped.setdefault(n, []).append(reason)
for n, rs in sorted(w2_unmapped.items()):
    fail("WORK2", f"'{n}': the reasons {sorted(set(rs))} of the matrix are not stated in its wire.md section "
         f"({', '.join(sorted({t for rx, s in SECTION_MAP if re.search(rx, n) for t in s}))}) "
         "or in Versioning / Reserved fields")

# ---------------------------------------------------------------------------
# AC 2: every enumeration value is used across the matrix and the toml lists
# ---------------------------------------------------------------------------
used = set()
for _, _, rs in row_reasons_by_row:
    used |= rs
for m in toml_msgs:
    for r in m["kv"].get("rejection_reasons", []) or []:
        used.add(r)
        if r not in enum:
            fail("AC2", f"{m['kv'].get('name')}: rejection reason '{r}' is not in the closed enumeration")
for n in ENUM_NAMES:
    if n not in used:
        fail("AC2", f"enumeration value '{n}' is used by no matrix cell and no rejection_reasons list")

# ---------------------------------------------------------------------------
# WORK 3: one tagged paragraph states the uniform outcome
# ---------------------------------------------------------------------------
W3 = [("drop", r"\bdrop"), ("count", r"\bcount"), ("input", r"\binput\b"), ("reason", r"\breason\b"),
      ("no state change", r"no state change"), ("VersionMismatch", r"VersionMismatch"),
      ("event", r"\bevent\b")]
w3_best = None
for u in vr_units:
    if u["kind"] != "prose" or not has_evidence(u["text"]):
        continue
    miss = [label for label, rx in W3 if not re.search(rx, u["text"], re.I if label != "VersionMismatch" else 0)]
    if not miss:
        w3_best = []
        break
    if w3_best is None or len(miss) < len(w3_best):
        w3_best = miss
if w3_best is None or w3_best:
    fail("WORK3", "no tagged paragraph of 'Validity rules' states the uniform outcome "
         f"(drop; counted per (input, reason); no state change; a VersionMismatch event); "
         f"closest paragraph misses {w3_best if w3_best else 'everything'}")

# ---------------------------------------------------------------------------
# DoD: evidence tags on the section's prose and tables
# ---------------------------------------------------------------------------
last_prose_tagged = True
prev_tid = None
for u in vr_units:
    if u["kind"] == "heading":
        last_prose_tagged = True
        prev_tid = None
        continue
    if u["kind"] == "prose":
        tagged = has_evidence(u["text"])
        if len(words(u["text"])) >= 5 and not tagged:
            fail("DoD", "a paragraph of 'Validity rules' carries no evidence tag: " + u["text"].splitlines()[0][:90])
        last_prose_tagged = tagged or len(words(u["text"])) < 5
        prev_tid = None
        continue
    if u["kind"] == "row":
        if u["tid"] != prev_tid:
            tbl_text = "\n".join(r["text"] for r in vr_units if r["kind"] == "row" and r["tid"] == u["tid"])
            if not (has_evidence(tbl_text) or last_prose_tagged):
                fail("DoD", f"a table of 'Validity rules' (line {u['line'] + 1}) carries no evidence tag "
                     "and follows an untagged paragraph")
            prev_tid = u["tid"]

# ---------------------------------------------------------------------------
# OPEN: every open item that names 004-21 as owner is accounted for
# ---------------------------------------------------------------------------
sec_of_line = {}
cur_title = ""
for i in range(len(lines)):
    for hi, lvl, t in HEADS:
        if hi == i and lvl == 2:
            cur_title = t
    sec_of_line[i] = cur_title
outside = [u for u in units(0, len(lines))
           if u["kind"] in ("prose", "row")
           and not (VR and VR[0] <= u["line"] < VR[1])
           and "004-21" in u["text"]]
hand_on = []
for u in vr_units:
    if u["kind"] not in ("prose", "row"):
        continue
    t = u["text"]
    if not re.search(r"\bowner", t, re.I) or not re.search(r"falsifier", t, re.I):
        continue
    if not re.search(r"hand(?:ed|s)?[ -]?(?:on|to)|hand-on", t, re.I):
        continue
    owners = re.findall(r"\b\d{3}-\d{2}[a-z]?\b|\bepic \d{3}\b|\bdesign owner\b|\blead\b", t, re.I)
    if not [o for o in owners if o.lower() != "004-21"]:
        continue
    hand_on.append(t)

counts = {"open": 0, "row": 0, "prose": 0}
for u in outside:
    t = u["text"]
    sec = sec_of_line.get(u["line"], "")
    if u["kind"] == "row":
        kind = "row"
        label = f"'{sec}': table row {u['cells'][0][:40]!r}"
        num = None
    else:
        m = re.match(r"^\*\*Open item (\d+)", t)
        kind = "open" if m else "prose"
        num = m.group(1) if m else None
        label = f"'{sec}': " + (f"Open item {num}" if m else f"paragraph {t.splitlines()[0][:50]!r}")
    counts[kind] += 1
    if re.search(r"\bStatus\b", t) and "004-21" in t:
        continue
    ok = False
    for h in hand_on:
        if norm(sec) and not phrase_re(norm(sec)).search(norm(h)):
            continue
        if num is not None and not re.search(r"open item " + num + r"(?!\d)", norm(h)):
            continue
        ok = True
        break
    if not ok:
        fail("OPEN", f"{label} names 004-21 but neither carries a 004-21 status sentence nor is handed on in "
             "'Validity rules' with a named owner other than 004-21 and a falsifier")

# ---------------------------------------------------------------------------
# Repository-level checks: the 004-02 checker and the design checker
# ---------------------------------------------------------------------------
if not overridden:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        if proc.returncode != 0 or "checks clean" not in out:
            tail = "\n".join(out.strip().splitlines()[-8:])
            fail("AC5", f"cargo run --bin wire-check did not pass:\n{tail}")
    except (OSError, subprocess.TimeoutExpired) as error:
        fail("AC5", f"could not run cargo run --bin wire-check: {error}")

    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
    except (OSError, subprocess.TimeoutExpired) as error:
        proc = None
        fail("AC5", f"could not run the wire_tables tests: {error}")
    if proc is not None:
        out = proc.stdout + proc.stderr
        if proc.returncode != 0:
            tail = "\n".join(out.strip().splitlines()[-12:])
            fail("AC5", f"cargo test -p hadronyche-tools --test wire_tables failed:\n{tail}")
        result = None
        for line in out.splitlines():
            if "test result:" in line:
                result = line.strip()
        if result is None:
            fail("AC5", "the wire_tables test run printed no 'test result:' line")
        else:
            mm = re.search(r"(\d+)\s+passed;\s*(\d+)\s+failed;", result)
            if not mm:
                fail("AC5", f"could not read the test counts from: {result}")
            elif int(mm.group(2)) != 0 or int(mm.group(1)) < 12:
                fail("AC5", "wire_tables must stay at least 12 passed, 0 failed; got: " + result)

    if os.path.exists("doc/kickoff/check-design.py"):
        try:
            design = subprocess.run(["python3", "doc/kickoff/check-design.py", "doc/design"],
                                    capture_output=True, text=True, timeout=300)
            out = (design.stdout + design.stderr).strip().splitlines()
            last = out[-1].strip() if out else ""
            if design.returncode != 0 or last != "OK":
                fail("DoD", "the design checker does not print OK: " + "\n".join(out[-6:]))
        except (OSError, subprocess.TimeoutExpired) as error:
            fail("DoD", f"could not run the design checker: {error}")
else:
    print("NOTE AC5: document paths overridden; repository-level checks skipped")

# ---------------------------------------------------------------------------
print(f"INFO in-scope messages: {len(in_scope)}; matrix rows: {len(row_reasons_by_row)}; "
      f"exempt from the control bound: {exempt_set}; C-17 bound: {C17}; "
      f"open-item candidates outside the section: {sum(counts.values())} "
      f"(open {counts['open']}, rows {counts['row']}, prose {counts['prose']})")
if fails:
    for tag, msg in fails:
        print(f"FAIL {tag}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)
print(f"OK: 5/5 acceptance criteria for 004-21, the uniform outcome, bounded parsing, open items and DoD tags checked")
PY
