#!/usr/bin/env bash
# 004-15 — the on-wire size of one default packet, the bit rates computed from
# the final layout, the difference from the design's derived figures, and the
# controller-stream wire cost of C-219.
#
# RED-phase document check. It validates a wire-size subsection of
# doc/design/spec/wire.md against every acceptance criterion of the ticket. The
# subsection is either a heading (any level) whose title names the wire size,
# the capacity or the recompute, placed inside "Size budget" (004-14), or the
# first level-2 heading after "Size budget". Its text runs to the next heading of
# the same or a higher level. It does not run the other checks/004-*.sh: the
# per-change suite runs every file in checks/, so a sibling loop here would be
# quadratic.
#
# WIRE_MD / WIRE_TOML may override the paths (self-test fixtures only). The
# constants are always read from doc/design/02-constants.md, and nothing here is
# pinned: every figure is computed at run time from the 004-14 field tables
# (wire.toml [[message]] / [[message.field]], the [budget] total when present)
# and from the constants rows C-02, C-03, C-04, C-20, C-21 and C-22. The only
# literals are the ticket's own arithmetic: Ethernet framing 14 + 4 + 8 + 12 = 38
# (`[memory]` in the ticket), IPv4 header 20 and UDP header 8 (C-100), and the
# 2 000 slices/s of C-02 read from its row.
#
# The author's form, which the check reads (all by content, not by position):
#
#   Section     a heading naming the wire size, capacity or recompute (for
#               example "Wire size and capacity recompute"), level 3 inside
#               "Size budget" or level 2 directly after it. Its prose paragraphs
#               and table rows are the units below.
#   Equations   arithmetic written as "a + b + c = d" (operators + - x × *). Every
#               equation in the subsection is evaluated: the left side must equal
#               the right side in magnitude, to the precision the right side
#               shows. A right side with "Mbit/s" is scaled by 10^6, "kbit/s" by
#               10^3; a bare or "bytes" right side is taken as is.
#   Wire size   an equation whose left side has the default payload as one term
#               (the 004-14 default total) and whose right
#               side is that payload plus 66; or a table with a row whose label
#               names the payload (equal to the default total) and a row whose
#               label is "wire", "total" or "on-the-wire" (equal to payload + 66).
#   Rates       units (paragraphs or rows) that name a stream, a channel or a
#               source with a value in Mbit/s: per stream = wire × 8 × 2 000
#               (C-02), per channel = per stream ÷ 6 (C-04), per source for 24,
#               48 and 96 channels = 4, 8 and 16 streams with 8 000, 16 000 and
#               32 000 packets/s, each in one unit with C-22's figure (84, 169,
#               338). A value is accepted when |stated − computed| is at most half
#               the last shown decimal (0.05 for one decimal).
#   Difference  one unit with 1 320 (C-20), the byte difference (absolute) and a
#               percentage within 0.1 point; one unit with 21.1 (C-21), the rate
#               difference (absolute, Mbit/s) and a percentage within 0.1 point.
#   5 % rule    a sentence with "5 %" or "5%" and one of exceed / above (claims
#               the difference exceeds 5 %) or within / below / "not exceed" (claims
#               it does not). The claim must match the computed difference. If it
#               exceeds: the section names C-20, C-21, C-22, C-24, C-25, C-26, C-90,
#               C-92 and C-118, and says "constants table updated" or has a
#               paragraph starting "Open item" with owner and falsifier. If it does
#               not: a sentence says no constant is recomputed (the constants stand).
#   C-219       paragraphs or rows naming C-219 (or a controller unit with "per
#               slice" or "controller bytes"): the formula with 28 + 38 on the right
#               of an "=" (the 66 bytes of IP, UDP and Ethernet overhead, the "66"
#               written too), a 2 000 packets/s figure, a bit-rate formula with
#               "× 8 × 2 000", the header term named as the 100-byte cap or a
#               Size budget figure. At least one worked example "H + c + 28 + 38 =
#               N" whose header term H is a figure of the budget (the cap 100, or
#               one of the mandatory and optional sums of the 004-14 tables), and
#               one bit-rate example "N × 8 × 2 000 = R" whose N is a worked byte
#               count or the default wire size; both are evaluated.
#   Ethernet    the four framing parts named beside their words: header 14, FCS
#               (or frame check sequence) 4, preamble 8, inter-frame gap 12, with
#               Ethernet total 38. Every unit that states a framing part carries
#               [memory] (and then names a checking task by ticket id with a word
#               such as checks, pins, verifies or checker in the same paragraph) or
#               [standard] (and then a clause with a dotted number in the same
#               sentence or row, C-132).
#   Evidence    every prose paragraph of 5 or more words carries a tag; a paragraph
#               starting "Open item" names an owner and a falsifier.
#   Toml        optional: integer keys in a table whose name contains wire, rate
#               or overhead (or in [budget]) that name a wire size, rate or
#               overhead must equal the computed value. Keys are not required.
#
# Per acceptance criterion of the ticket:
#   AC 1  wire size of one default packet, with the arithmetic shown; the four
#         Ethernet parts stated and summing to 38; the bit rate per stream; per
#         channel; per source for 24, 48, 96 channels with 8 000, 16 000, 32 000
#         packets/s; a v1 sender's largest packet and the format maximum, if shown,
#         checked by the same arithmetic.
#   AC 2  the difference from 1 320 bytes and 21.1 Mbit/s, in absolute and percent;
#         the per-source rates compared with C-22's figures.
#   AC 3  the 5 % rule, consistent with the computed difference (see above).
#   AC 4  the C-219 formula and worked examples (see above).
#   AC 5  toml keys that name a wire size, rate or overhead agree (optional).
#   DoD   evidence tags, owner and falsifier of open items, the Ethernet tags.
set -u
cd "$(dirname "$0")/.."

WIRE="${WIRE_MD:-doc/design/spec/wire.md}"
TABLES="${WIRE_TOML:-doc/design/spec/wire.toml}"
CONSTANTS="doc/design/02-constants.md"
python3 - "$WIRE" "$TABLES" "$CONSTANTS" <<'PY'
import re
import sys

wire_path, toml_path, const_path = sys.argv[1], sys.argv[2], sys.argv[3]
fails = []


def fail(ac, msg):
    fails.append((ac, msg))


def read(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


wire = read(wire_path)
toml_src = read(toml_path)
const_src = read(const_path)
for label, text, path in (("wire.md", wire, wire_path), ("wire.toml", toml_src, toml_path),
                          ("02-constants.md", const_src, const_path)):
    if text is None:
        print(f"FAIL setup: cannot read {label} at {path}")
        sys.exit(1)


def nrm(s):
    return re.sub(r"\s+", " ", re.sub(r"[   ]", " ", s)).strip()


def to_int(s):
    return int(s.replace(" ", ""))


def split_row(line):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    return [c.strip().replace("\\|", "|") for c in re.split(r"(?<!\\)\|", s)]


INT_RE = re.compile(r"(?<![\w.\-])(\d{1,3}(?: \d{3})+|\d+)(?![\w]|\.\d)")
FLOAT_RE = re.compile(r"(?<![\w.\-])(\d+)\.(\d+)(?!\w)")
PCT_RE = re.compile(r"(?<![\w.])(\d+)(?:\.(\d+))?\s*%")
MBIT_RE = re.compile(r"(?<![\w.])(\d+)(?:\.(\d+))?\s*Mbit/s")
EVID_RE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)
TAG_RE = re.compile(r"\[(?:memory|standard)\]", re.I)


def ints(s):
    return [to_int(m.group(1)) for m in INT_RE.finditer(nrm(s))]


def floats(s):
    return [(float(m.group(1) + "." + m.group(2)), len(m.group(2))) for m in FLOAT_RE.finditer(nrm(s))]


def pcts(s):
    return [(float(m.group(1) + ("." + m.group(2) if m.group(2) else "")), len(m.group(2) or ""))
            for m in PCT_RE.finditer(nrm(s))]


def tol_of(decimals):
    return 0.5 * 10 ** -decimals if decimals else 0.5


# ---------------------------------------------------------------------------
# Constants (read, never edited). Rows C-02, C-03, C-04, C-20, C-21, C-22.
# ---------------------------------------------------------------------------
def const_cell(cid):
    for line in const_src.splitlines():
        if not line.lstrip().startswith("|"):
            continue
        cells = split_row(line)
        if len(cells) >= 3 and cells[0] == cid:
            return nrm(cells[2])
    return None


CELL = {cid: const_cell(cid) for cid in ("C-02", "C-03", "C-04", "C-20", "C-21", "C-22", "C-137")}
if any(v is None for v in CELL.values()):
    print("FAIL setup: a constant row the wire-size check needs (C-02, C-03, C-04, C-20, C-21, C-22) is missing")
    sys.exit(1)
SLICE_RATE = to_int(re.search(r"(\d[\d ]*) slices/s", CELL["C-02"]).group(1))   # 2 000
FRAMES = int(re.search(r"(\d+) frames", CELL["C-02"]).group(1))                  # 48
SAMPLE = ints(CELL["C-03"])[0]                                                   # 4
CH = ints(CELL["C-04"])[0]                                                       # 6
C20 = ints(CELL["C-20"])[0]                                                      # 1 320
C21 = floats(CELL["C-21"])[0][0]                                                 # 21.1
C22 = [tuple(to_int(x) for x in t) for t in re.findall(
    r"(\d+) ch: (\d+) streams?, (\d+) Mbit/s, (\d[\d ]*\d|\d+) packets/s", CELL["C-22"])]
if not C22:
    print("FAIL setup: C-22 does not parse as 'NN ch: N streams, N Mbit/s, N packets/s'")
    sys.exit(1)
CAP = int(re.search(r"at most (\d+) bytes", CELL["C-137"]).group(1))   # header cap (C-137): 100
C21_CH = float(re.search(r"([\d.]+) Mbit/s per channel", CELL["C-21"]).group(1))   # 3.5
IP_UDP = 20 + 8        # IPv4 header (C-100) + UDP header
ETH_PARTS = (("header", 14), ("FCS", 4), ("preamble and start-of-frame delimiter", 8), ("inter-frame gap", 12))
ETH = sum(v for _, v in ETH_PARTS)  # 38
OVH = IP_UDP + ETH                  # 66
AUDIO = CH * FRAMES * SAMPLE        # 1 152


# ---------------------------------------------------------------------------
# A small TOML reader for the subset wire.toml uses (Python 3.10 has no tomllib).
# ---------------------------------------------------------------------------
def strip_toml_comment(line):
    out, in_str, i = [], False, 0
    while i < len(line):
        c = line[i]
        if in_str:
            out.append(c)
            if c == "\\" and i + 1 < len(line):
                out.append(line[i + 1])
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
            out.append(c)
        elif c == "#":
            break
        else:
            out.append(c)
        i += 1
    return "".join(out)


def parse_value(s):
    s = s.strip()
    if s.startswith('"'):
        m = re.match(r'"((?:[^"\\]|\\.)*)"', s)
        return m.group(1) if m else None
    if s.startswith("["):
        items = []
        for m in re.finditer(r'"((?:[^"\\]|\\.)*)"|(-?0x[0-9a-fA-F_]+|-?\d[\d_]*)', s):
            items.append(m.group(1) if m.group(1) is not None else int(m.group(2).replace("_", ""), 0))
        return items
    if s in ("true", "false"):
        return s == "true"
    t = s.replace("_", "")
    try:
        return int(t, 0)
    except ValueError:
        try:
            return int(t, 10)
        except ValueError:
            return s


def parse_toml(src):
    messages, others, target = [], {}, None
    for raw in src.splitlines():
        line = strip_toml_comment(raw).strip()
        if not line:
            continue
        if line.startswith("[["):
            name = line[2:line.index("]]")].strip()
            if name == "message":
                target = {"name": None, "fields": []}
                messages.append(target)
            elif name == "message.field":
                target = {}
                messages[-1]["fields"].append(target)
            else:
                target = {}
                others.setdefault(name, [])
                others[name].append(target)
            continue
        if line.startswith("["):
            name = line[1:line.index("]")].strip()
            target = {}
            others[name] = target
            continue
        if "=" in line and target is not None:
            key, value = line.split("=", 1)
            target[key.strip()] = parse_value(value)
    return messages, others


try:
    MESSAGES, OTHERS = parse_toml(toml_src)
except (ValueError, IndexError, KeyError) as error:
    print(f"FAIL setup: wire.toml does not parse: {error}")
    sys.exit(1)
BY_NAME = {m.get("name"): m for m in MESSAGES if m.get("name")}


def field(m, name):
    for f in (m or {}).get("fields", []):
        if f.get("name") == name:
            return f
    return None


def fbytes(f):
    return f["width"] // 8 if f and isinstance(f.get("width"), int) else None


mp = BY_NAME.get("media packet")
ck = BY_NAME.get("slice checksum")
cw_m = BY_NAME.get("content-message header")
ann_count_f = field(mp, "announcement_count")
ann_entry_f = field(mp, "announcement_entry")
if not (mp and ck and cw_m and ann_count_f and ann_entry_f):
    print("FAIL setup: wire.toml lacks the media packet, checksum or content-message header tables")
    sys.exit(1)
PKT = sum(f["width"] for f in mp["fields"] if f.get("offset", 0) + f.get("width", 0) <= 128) // 8
SLICE = sum(f["width"] for f in mp["fields"] if 128 <= f.get("offset", -1) < ann_count_f["offset"]) // 8
ANN_COUNT = fbytes(ann_count_f)
ENTRY = fbytes(ann_entry_f)
ENTRY_MAX = ann_entry_f.get("max_count", 0)
ext_start = ann_entry_f["offset"] + ann_entry_f["width"] * ENTRY_MAX
EXT = sum(f["width"] for f in mp["fields"] if f.get("offset", -1) >= ext_start) // 8
CKSUM = ck.get("max_size")
CW = cw_m.get("max_size")
DEF_PAY = PKT + SLICE + ANN_COUNT + CW + CKSUM + AUDIO         # default UDP payload (004-14)
ANN_MAX_PAY = PKT + SLICE + ANN_COUNT + ENTRY * ENTRY_MAX + EXT + CW + CKSUM + AUDIO
V1_LARGEST_PAY = PKT + SLICE + ANN_COUNT + ENTRY + EXT + CW + CKSUM + AUDIO   # one announcement + extension
WIRE = DEF_PAY + OVH                                          # on-wire size of one default packet
WIRE_MAX = ANN_MAX_PAY + OVH
BITS_STREAM = WIRE * 8 * SLICE_RATE                           # bit/s per stream
RATE_STREAM = BITS_STREAM / 1e6                               # Mbit/s per stream
RATE_CH = RATE_STREAM / CH
DIFF_B = WIRE - C20
PCT_B = DIFF_B / C20 * 100
DIFF_R = RATE_STREAM - C21
PCT_R = DIFF_R / C21 * 100
EXCEEDS = max(abs(PCT_B), abs(PCT_R)) > 5
HDR_ALLOWED = {PKT + SLICE + ANN_COUNT + CW + CKSUM,           # 46 mandatory headers and checksum
               PKT + SLICE + ANN_COUNT + ENTRY + EXT + CKSUM,  # 64: a controller member's headers (no audio)
               PKT + SLICE + ANN_COUNT + ENTRY + CKSUM,        # 55: the same, a stream that cannot measure
               ANN_MAX_PAY - AUDIO,                            # 103 maximum occupancy
               CAP}                                            # the C-137 header cap
HDR_ALLOWED |= {PKT + SLICE + ANN_COUNT + ENTRY * k + EXT * e + CW + CKSUM
                for k in range(ENTRY_MAX + 1) for e in (0, 1)}
HDR_ALLOWED |= {CW + CKSUM + PKT + SLICE + ANN_COUNT + ENTRY * k for k in range(ENTRY_MAX + 1)}
ALLOWED_BYTES = {DEF_PAY + OVH, WIRE_MAX, V1_LARGEST_PAY + OVH}

budget_default = None
for _n, _v in OTHERS.items():
    if "budget" in _n.lower() and isinstance(_v, dict) and isinstance(_v.get("default_total"), int):
        budget_default = _v["default_total"]
if budget_default is not None and budget_default != DEF_PAY:
    fail("AC 1", f"wire.toml [budget] default_total is {budget_default}, the 004-14 components sum to {DEF_PAY}")


# ---------------------------------------------------------------------------
# Markdown: the wire-size subsection, its units (prose paragraphs and table rows).
# ---------------------------------------------------------------------------
lines = wire.splitlines()


def fence_mask(ls):
    mask, inside = [], False
    for line in ls:
        if re.match(r"^\s*```", line):
            inside = not inside
            mask.append(True)
            continue
        mask.append(inside)
    return mask


FENCE = fence_mask(lines)
heads = []
for i, line in enumerate(lines):
    if FENCE[i]:
        continue
    m = re.match(r"^(#{1,6})\s+(.*?)\s*#*\s*$", line)
    if m:
        heads.append((i, len(m.group(1)), m.group(2).strip()))

sec_start = sec_end = None
for k, (idx, level, title) in enumerate(heads):
    if level == 2 and re.fullmatch(r"size budget", title, re.I):
        sec_start = idx
        sec_end = next((j for (j, lv, _) in heads[k + 1:] if lv <= 2), len(lines))
        break
first_after = next((idx for idx, lv, _ in heads if sec_end is not None and idx >= sec_end and lv <= 2), None)

TITLE_RE = re.compile(r"wire[ -]size|capacity|recomput|bit[ -]rate|on[ -]the[ -]wire|on-wire", re.I)
found = None
for idx, lv, title in heads:
    if not TITLE_RE.search(title):
        continue
    inside = sec_start is not None and sec_start < idx < sec_end
    if inside or idx == first_after:
        found = (idx, lv, title)
        break

if found is None:
    fail("AC 1", "no heading naming the wire size or the capacity recompute, inside 'Size budget' or directly after it")
    for ac, msg in fails:
        print(f"FAIL {ac}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

h_idx, h_lv, h_title = found
h_end = next((j for (j, lv, _) in heads if j > h_idx and lv <= h_lv), len(lines))
sub_lines = lines[h_idx + 1:h_end]
sub_fence = fence_mask(sub_lines)


def is_table_line(line):
    return line.lstrip().startswith("|")


def caption_above(ls, fm, i):
    got, k = [], i - 1
    while k >= 0 and i - k <= 5 and len(got) < 2:
        s = ls[k].strip()
        if s and not is_table_line(ls[k]) and not fm[k]:
            got.append(s.lstrip("#* ").strip())
        k -= 1
    return " ".join(reversed(got))


def build_units(ls, fm):
    units, buf = [], []

    def flush():
        if buf:
            units.append({"kind": "prose", "text": nrm(" ".join(buf)), "cells": None, "header": None})
            buf.clear()

    i = 0
    while i < len(ls):
        s = ls[i].strip()
        if fm[i] or not s or s.startswith("#"):
            flush()
            i += 1
            continue
        if is_table_line(ls[i]):
            flush()
            j = i
            while j < len(ls) and not fm[j] and is_table_line(ls[j]):
                j += 1
            block = ls[i:j]
            cap = caption_above(ls, fm, i)
            if len(block) >= 2 and "-" in block[1] and re.match(r"^\s*\|?[\s:|-]+\|?\s*$", block[1]):
                header = split_row(block[0])
                for r in block[2:]:
                    cells = split_row(r)
                    text = nrm(" ".join([cap] + [" | ".join(header)] + [" | ".join(cells)]))
                    own = nrm(" | ".join(header) + " | " + " | ".join(cells))
                    units.append({"kind": "row", "text": text, "own": own, "cells": cells, "header": header})
            else:
                for r in block:
                    units.append({"kind": "prose", "text": nrm(r), "cells": None, "header": None})
            i = j
            continue
        if re.match(r"^([-*+]\s|\d+\.\s)", s):
            flush()
        buf.append(s)
        i += 1
    flush()
    return units


UNITS = build_units(sub_lines, sub_fence)
SUB_TEXT = nrm(" ".join(u["text"] for u in UNITS))
PROSE = [u for u in UNITS if u["kind"] == "prose"]


def mbits(u):
    out = [(float(m.group(1) + ("." + m.group(2) if m.group(2) else "")), len(m.group(2) or ""))
           for m in MBIT_RE.finditer(u["text"])]
    if u["cells"] is not None:
        for h, c in zip(u["header"], u["cells"]):
            if "mbit" in h.lower():
                m = re.search(r"(\d+)(?:\.(\d+))?", c)
                if m:
                    out.append((float(m.group(1) + ("." + m.group(2) if m.group(2) else "")),
                                len(m.group(2) or "")))
    return out


def equations(text):
    """Every 'chain = value' in text: (terms, lhs_value, rhs_scaled, tol, ok, rhs_unit, raw)."""
    out = []
    pat = re.compile(
        r"(?<![\w.])(\d(?:[\d ]*\d)?(?:\s*[+\-−×x*]\s*\d(?:[\d ]*\d)?)+)\s*=\s*"
        r"(\d(?:[\d ]*\d)?)(?:\.(\d+))?(?:\s*(Mbit/s|kbit/s|bit/s|bytes?|%))?")
    for m in pat.finditer(text):
        before = text[:m.start()].rstrip()
        if before and before[-1] in "+−-×*|":
            continue
        lhs = m.group(1)
        if m.group(4) == "%":
            continue
        parts = re.split(r"\s*([+\-−×x*])\s*", lhs.strip())
        nums = [to_int(p) for p in parts[0::2]]
        ops = parts[1::2]
        expr = str(nums[0])
        for op, n in zip(ops, nums[1:]):
            expr += {"+": "+", "-": "-", "−": "-", "×": "*", "x": "*", "*": "*"}[op] + str(n)
        if not re.fullmatch(r"[\d+\-*]+", expr):
            continue
        lhs_val = abs(eval(expr))
        rhs_raw = to_int(m.group(2)) + (float("0." + m.group(3)) if m.group(3) else 0)
        unit = m.group(4) or ""
        scale = {"Mbit/s": 1e6, "kbit/s": 1e3}.get(unit, 1)
        dec = len(m.group(3) or "")
        tol = (0.5 * 10 ** -dec if dec else (0.5 if scale != 1 else 0)) * scale
        rhs_scaled = rhs_raw * scale
        ok = abs(lhs_val - rhs_scaled) <= tol + 1e-9
        out.append({"terms": nums, "ops": ops, "lhs": lhs_val, "rhs": rhs_scaled, "rhs_raw": rhs_raw,
                    "tol": tol, "ok": ok, "unit": unit, "raw": m.group(0)})
    return out


EQS = []
for u in UNITS:
    src = u.get("own", u["text"])   # a table row's own cells; its caption is read in its own paragraph
    for e in equations(src):
        e["unit_text"] = src
        EQS.append(e)

for e in EQS:
    if not e["ok"]:
        fail("AC 1", f"arithmetic does not hold: {e['raw']!r} (the left side is {e['lhs']:g})")


def close(a, b, tol):
    return abs(a - b) <= tol + 1e-9


# ---------------------------------------------------------------------------
# DoD: evidence tags on prose paragraphs; open items name an owner and a falsifier.
# ---------------------------------------------------------------------------
for u in PROSE:
    words = re.findall(r"[A-Za-z0-9]+", u["text"])
    if len(words) >= 5 and not EVID_RE.search(u["text"]):
        fail("DoD", "a prose paragraph of 5+ words has no evidence tag: " + u["text"][:80])
    if re.match(r"^[\s*_]*open item", u["text"], re.I):
        if not re.search(r"owner", u["text"], re.I) or not re.search(r"falsifier", u["text"], re.I):
            fail("DoD", "an open item names no owner or no falsifier: " + u["text"][:80])

# ---------------------------------------------------------------------------
# AC 1: the on-wire size of one default packet, with its arithmetic.
# ---------------------------------------------------------------------------
wire_eq = [e for e in EQS if DEF_PAY in e["terms"]]
if any(e["rhs"] != WIRE for e in wire_eq):
    bad = next(e for e in wire_eq if e["rhs"] != WIRE)
    fail("AC 1", f"the default packet's on-wire size is stated as {bad['rhs_raw']:g}, but UDP payload "
                 f"{DEF_PAY} + {OVH} = {WIRE} (the 004-14 default total and the 66 bytes of overhead)")
elif not any(e["rhs"] == WIRE for e in wire_eq):
    rows = [u for u in UNITS if u["kind"] == "row"]

    def row_val(u):
        for c in u["cells"][1:]:
            m = re.search(r"\d[\d ]*\d|\d", c)
            if m:
                return to_int(m.group(0))
        return None

    has_payload = any(re.search(r"payload", u["cells"][0], re.I) and row_val(u) == DEF_PAY for u in rows if u["cells"])
    has_total = any(re.search(r"wire|total|on-the-wire", u["cells"][0], re.I) and row_val(u) == WIRE
                    for u in rows if u["cells"])
    if not (has_payload and has_total):
        fail("AC 1", f"no arithmetic or table states the on-wire size of one default packet: "
                     f"{DEF_PAY} (UDP payload, the 004-14 default) + 8 + 20 + 38 = {WIRE}")

# Ethernet framing: each part beside its name, and the total 38.
ETH_KEYS = [
    ("header", 14, re.compile(r"Ethernet[^.;|]{0,30}\bheader\b|\bheader\b[^.;|]{0,30}Ethernet", re.I)),
    ("FCS", 4, re.compile(r"\bFCS\b|frame check|check sequence", re.I)),
    ("preamble and start-of-frame delimiter", 8, re.compile(r"preamble", re.I)),
    ("inter-frame gap", 12, re.compile(r"inter-?frame|\bIFG\b", re.I)),
]
SUB_JOINED = nrm(" ".join(l for l, f in zip(sub_lines, sub_fence) if not f))
for name, value, pat in ETH_KEYS:
    hit = False
    for m in pat.finditer(SUB_JOINED):
        window = SUB_JOINED[max(0, m.start() - 80):m.end() + 80]
        if value in ints(window):
            hit = True
            break
    if not hit:
        fail("AC 1", f"the Ethernet {name} of {value} bytes is not stated beside its name")
eth_total = False
for m in re.finditer(r"Ethernet", SUB_JOINED, re.I):
    if ETH in ints(SUB_JOINED[max(0, m.start() - 80):m.end() + 80]):
        eth_total = True
        break
if not eth_total:
    fail("AC 1", f"the Ethernet framing total of {ETH} bytes is not stated beside 'Ethernet'")

# Evidence of the Ethernet framing units (C-132, DoD).
FRAMING = re.compile(r"\bFCS\b|frame check|check sequence|preamble|inter-?frame|"
                     r"Ethernet framing|Ethernet[^.;|]{0,12}header|\bEthernet\b", re.I)
CHECKER_RE = re.compile(r"\b\d{3}[a-z]?-\d{2}\b")
CHECK_WORD = re.compile(r"\b(checks?|checker|pins?|pinned|verifies|verify|checked by)\b", re.I)
CLAUSE_RE = re.compile(r"\b[Cc]lause\s+\d+(?:\.\d+)+")
for u in UNITS:
    own = u.get("own", u["text"])
    figures = ints(re.sub(r"\b\d{3}[a-z]?-\d{2}\b", " ", own))
    if not FRAMING.search(own) or not any(v in figures for v in (14, 4, 8, 12, 38)):
        continue   # a framing unit states a figure; a mention without one is not a figure
    tags = {t.lower() for t in TAG_RE.findall(own)}
    if not tags:
        fail("DoD", "an Ethernet framing figure has no [memory] or [standard] tag (C-132): " + own[:80])
        continue
    if "[memory]" in tags:
        if not (CHECKER_RE.search(own) and CHECK_WORD.search(own)):
            fail("DoD", "an Ethernet framing figure tagged [memory] names no checking task "
                    "(a ticket id with checks/pins/verifies/checker): " + own[:80])
    if "[standard]" in own.lower():
        if u["kind"] == "row":
            label_is_ethernet = re.match(r"\s*Ethernet", u["cells"][0], re.I) is not None
            if label_is_ethernet and not CLAUSE_RE.search(own):
                fail("DoD", "an Ethernet framing row tagged [standard] has no clause with a dotted number: " + own[:80])
        else:
            for sent in re.split(r"(?<=[.;])\s+", own):
                if "[standard]" in sent.lower() and re.search(r"Ethernet|FCS|preamble|inter-?frame", sent, re.I) \
                        and not CLAUSE_RE.search(sent):
                    fail("DoD", "an Ethernet framing sentence tagged [standard] has no clause with a dotted "
                                "number: " + sent[:80])


# ---------------------------------------------------------------------------
# AC 1 and AC 2: bit rates per stream, per channel, per source (C-22).
# ---------------------------------------------------------------------------
def has_rate(pred, value):
    for u in UNITS:
        if pred(u) and any(close(v, value, tol_of(d)) for v, d in mbits(u)):
            return True
    return False


if not has_rate(lambda u: "stream" in u["text"].lower(), RATE_STREAM):
    fail("AC 1", f"no bit rate per stream of {RATE_STREAM:.1f} Mbit/s (wire {WIRE} bytes x 8 x {SLICE_RATE}) is stated")
if not has_rate(lambda u: "channel" in u["text"].lower(), RATE_CH):
    fail("AC 1", f"no bit rate per channel of {RATE_CH:.2f} Mbit/s (per stream / {CH}, C-04) is stated")
for ch, streams, c22_rate, pps in C22:
    expect_rate = streams * RATE_STREAM
    ok_src = False
    for u in UNITS:
        n = ints(u["text"])
        if ch in n and streams in n and pps in n and c22_rate in n:
            if any(close(v, expect_rate, tol_of(d)) for v, d in mbits(u)):
                ok_src = True
                break
    if not ok_src:
        fail("AC 1", f"no unit gives {ch} channels: {streams} streams, {expect_rate:.1f} Mbit/s, {pps} packets/s "
                     f"beside C-22's {c22_rate} Mbit/s (streams = channels / {CH}, rate = streams x per-stream rate)")

# AC 2: the difference from C-20 (bytes) and C-21 (rate), absolute and percent.
def own_of(u):
    return u.get("own", u["text"])


def sentences_of(u):
    return [own_of(u)] if u["kind"] == "row" else re.split(r"(?<=[.;])\s+", u["text"])


def mbit_list(t):
    return [(float(m.group(1) + ("." + m.group(2) if m.group(2) else "")), len(m.group(2) or ""))
            for m in MBIT_RE.finditer(t)]


DIFF_UNITS = [x for u in UNITS for x in sentences_of(u)]
ok_bytes = False
for x in DIFF_UNITS:
    if C20 in ints(x) and abs(DIFF_B) in ints(x):
        if any(abs(v - abs(PCT_B)) <= max(0.1, tol_of(d)) for v, d in pcts(x)):
            ok_bytes = True
            break
if not ok_bytes:
    fail("AC 2", f"no unit gives the byte difference {DIFF_B:+d} bytes against {C20} (C-20) and "
                 f"{PCT_B:+.1f} % in one sentence or row")
ok_rate = False
for x in DIFF_UNITS:
    if re.search(r"(?<![\w.])%s(?![\d])" % re.escape(f"{C21:g}"), x):
        if any(close(v, abs(DIFF_R), tol_of(d)) for v, d in floats(x)) and \
                any(abs(v - abs(PCT_R)) <= max(0.1, tol_of(d)) for v, d in pcts(x)):
            ok_rate = True
            break
if not ok_rate:
    fail("AC 2", f"no unit gives the rate difference {DIFF_R:+.2f} Mbit/s against {C21:g} (C-21) and "
                 f"{PCT_R:+.1f} % in one sentence or row")

# AC 2 (T2): every stated difference and percent against C-22 is evaluated. A difference is
# stated signed ("−3.10 Mbit/s"); it must equal streams x per-stream rate − C-22. A percent is
# stated signed ("−3.7 %"); over C-22's rounded figure, or over C-21 x streams when the sentence
# or row names that unrounded base, it must equal the difference's share.
SIGNED_MBIT = re.compile(r"(?<![\w\-])([−+-])\s*(\d+)(?:\.(\d+))?\s*Mbit/s")
SIGNED_PCT = re.compile(r"(?<![\w\-])([−+-])\s*(\d+)(?:\.(\d+))?\s*%")


def signed_num(m):
    """(magnitude, decimals, sign) of a signed figure: '+' or '−'/'-' before the digits."""
    return (float(m.group(2) + ("." + m.group(3) if m.group(3) else "")), len(m.group(3) or ""),
            -1 if m.group(1) in "−-" else 1)


for ch, streams, c22_rate, pps in C22:
    exp_diff = streams * RATE_STREAM - c22_rate
    base2 = streams * C21
    for u in UNITS:
        for t in sentences_of(u):
            n = ints(t)
            if ch not in n or c22_rate not in n:
                continue
            for m in SIGNED_MBIT.finditer(t):
                v, d, sg = signed_num(m)
                if not close(sg * v, exp_diff, tol_of(d)):
                    fail("AC 2", f"the {ch}-channel difference from C-22 is stated as {sg * v:+g} Mbit/s, but the computed "
                                 f"difference is {exp_diff:+.3f} Mbit/s ({streams} x {RATE_STREAM:.3f} - {c22_rate}): "
                                 + t[:100])
            unrounded = any(abs(v - base2) <= 0.05 for v, _ in floats(t))
            bases = [base2] if unrounded else [c22_rate, base2]
            for m in SIGNED_PCT.finditer(t):
                p, d, _sg = signed_num(m)
                if not any(abs(p - abs(exp_diff) / b * 100) <= max(0.1, tol_of(d)) for b in bases):
                    fail("AC 2", f"the {ch}-channel share of the C-22 difference is stated as {p:g} %, but "
                                 f"{abs(exp_diff):.3f} over {' or '.join(f'{b:g}' for b in bases)} is "
                                 f"{' or '.join(f'{abs(exp_diff) / b * 100:.2f} %' for b in bases)}: " + t[:100])

# AC 2 (T2): a per-channel figure in a comparison sentence is the computed per-channel rate
# (per stream / 6), C-21's per-channel figure, or their difference; a per-stream figure is
# allowed only in a sentence that names the stream.
for u in PROSE:
    for t in re.split(r"(?<=[.;])\s+", u["text"]):
        if "per channel" not in t.lower():
            continue
        allowed = [(RATE_CH, 2), (C21_CH, 1), (abs(RATE_CH - C21_CH), 2)]
        if "stream" in t.lower():
            allowed.append((RATE_STREAM, 2))
        for v, d in mbit_list(t):
            if not any(close(v, a, max(tol_of(d), tol_of(dd))) for a, dd in allowed):
                fail("AC 2", f"a per-channel figure of {v:g} Mbit/s is neither the computed {RATE_CH:.3f} "
                             f"(per stream / {CH}, C-04) nor C-21's {C21_CH:g} or their difference: " + t[:100])

# AC 1 (T3): every row of the wire-size table (a table whose header names the payload and the wire).
def col_of(header, word):
    return next((i for i, h in enumerate(header) if word in h.lower()), None)


for u in UNITS:
    if u["kind"] != "row" or not any("payload" in h.lower() for h in u["header"]) \
            or not any("wire" in h.lower() for h in u["header"]):
        continue
    cells, header = u["cells"], u["header"]
    name = cells[0]
    ip, iw, ist, ich = (col_of(header, w) for w in ("payload", "wire", "stream", "channel"))
    if None in (ip, iw, ist, ich) or max(ip, iw, ist, ich) >= len(cells):
        fail("AC 1", f"wire-size row {name!r} lacks a payload, wire, stream or channel cell")
        continue
    payload = ints(cells[ip])[0] if ints(cells[ip]) else None
    eqw = equations(cells[iw])
    if payload is None or not eqw:
        fail("AC 1", f"wire-size row {name!r} states no payload or no on-the-wire arithmetic")
        continue
    wire_row = eqw[0]["rhs"]
    low = name.lower()
    if "default" in low:
        expected, label = DEF_PAY, "the 004-14 default total"
    elif "largest unshared" in low:
        expected, label = V1_LARGEST_PAY, "the 004-14 fallback one-announcement-and-extension total"
    elif "format maximum" in low:
        expected, label = ANN_MAX_PAY, "the 004-14 format-maximum total"
    else:
        expected, label = None, None
    if expected is not None and payload != expected:
        fail("AC 1", f"wire-size row {name!r}: payload {payload} is not {label} ({expected})")
    if wire_row != payload + OVH:
        fail("AC 1", f"wire-size row {name!r}: on the wire {wire_row:g} is not payload {payload} + {OVH}")
    rate_mbit = wire_row * 8 * SLICE_RATE / 1e6
    rate_vals = mbit_list(cells[ist])
    if not rate_vals:
        fail("AC 1", f"wire-size row {name!r} states no Mbit/s rate")
    for v, d in rate_vals:
        if not close(v, rate_mbit, tol_of(d)):
            fail("AC 1", f"wire-size row {name!r}: rate {v:g} Mbit/s is not {wire_row:g} x 8 x {SLICE_RATE} = "
                         f"{rate_mbit:.3f} Mbit/s")
    for e in equations(cells[ist]):
        if not close(e["rhs"], wire_row * 8 * SLICE_RATE, max(e["tol"], 1e-9)):
            fail("AC 1", f"wire-size row {name!r}: {e['raw']!r} does not give {wire_row * 8 * SLICE_RATE:,} bit/s")
    fl = floats(cells[ich])
    if not fl:
        fail("AC 1", f"wire-size row {name!r} states no per-channel rate")
    elif not close(fl[0][0], rate_mbit / CH, tol_of(fl[0][1])):
        fail("AC 1", f"wire-size row {name!r}: per-channel {fl[0][0]:g} Mbit/s is not {rate_mbit:.3f} / {CH} "
                     f"= {rate_mbit / CH:.3f} Mbit/s (C-04)")

# ---------------------------------------------------------------------------
# AC 3: the 5 % rule, against the computed difference.
# ---------------------------------------------------------------------------
sentences = [s for u in UNITS for s in re.split(r"(?<=[.;])\s+", u["text"])]
five = [s for s in sentences if re.search(r"(?<![\w.])5 ?%", s)]
if not five:
    fail("AC 3", "no sentence states the 5 % rule (a sentence with '5 %' and exceed / within / below / above)")
for s in five:
    within = re.search(r"\bwithin\b|\bbelow\s+(?:the\s+)?5|\bunder\s+(?:the\s+)?5|\bnot exceed|"
                       r"\bdoes not exceed|\bdo not exceed|\bno more than", s, re.I)
    exceed = re.search(r"exceed|above", s, re.I) and not within
    if not (within or exceed):
        fail("AC 3", "a '5 %' sentence states no direction: " + s[:90])
        continue
    if bool(exceed) != EXCEEDS:
        fail("AC 3", f"the section says the difference {'exceeds' if exceed else 'is within'} 5 %, but the computed "
                     f"difference is {PCT_B:+.2f} % bytes and {PCT_R:+.2f} % rate: " + s[:90])
STAND_RE = re.compile(r"no constant[^.;]{0,40}(?:recomput|changed|updated|re-?derived)|"
                      r"\bconstants?\b[^.;]{0,60}\bstand\b|not recomputed|"
                      r"constants? (?:are|is|remain|remains) unchanged", re.I)
if EXCEEDS:
    for cid in ("C-20", "C-21", "C-22", "C-24", "C-25", "C-26", "C-90", "C-92", "C-118"):
        if cid not in SUB_TEXT:
            fail("AC 3", f"the difference exceeds 5 %, but the section does not name {cid} as marked for recompute")
    if STAND_RE.search(SUB_TEXT):
        fail("AC 3", "the difference exceeds 5 %, but the section still says the constants stand or that no "
                     "constant is recomputed")
    updated = re.search(r"constants table updated", SUB_TEXT, re.I)
    capacity_item = any(
        re.match(r"^[\s*_]*open item", u["text"], re.I) and re.search(r"C-20\b|capacity constants", u["text"])
        and re.search(r"owner", u["text"], re.I) and re.search(r"falsifier", u["text"], re.I)
        for u in PROSE)
    if not updated and not capacity_item:
        fail("AC 3", "the difference exceeds 5 %, but neither 'constants table updated' nor an open item that "
                     "itself names C-20 (or capacity constants) with owner and falsifier")
elif not STAND_RE.search(SUB_TEXT):
    fail("AC 3", "the difference is within 5 %, but the section does not say that no constant is recomputed "
                 "(the constants stand)")

# AC 5 (audio, declared rate): IF the section states an audio stream's declared rate, it is
# cap + audio + 66 = 1 318 bytes and 21 088 000 bit/s (C-219's form). No open item is required.
AUD_BYTES = CAP + AUDIO + OVH
AUD_BITS = AUD_BYTES * 8 * SLICE_RATE
for t in sentences:
    if not (re.search(r"declared", t, re.I) and re.search(r"audio", t, re.I)):
        continue
    bits = [to_int(m.group(1)) for m in re.finditer(r"(\d[\d ]*\d|\d)\s*bit/s", t)]
    mb = mbit_list(t)
    if mb or bits:
        if not (any(close(v, AUD_BITS / 1e6, tol_of(d)) for v, d in mb) or any(abs(b - AUD_BITS) <= 0.5 for b in bits)):
            fail("AC 4", f"the audio declared rate is not {AUD_BYTES} bytes (cap {CAP} + audio {AUDIO} + {OVH}) "
                         f"at {AUD_BITS:,} bit/s: " + t[:100])
    big = [to_int(m.group(1)) for m in re.finditer(r"(\d[\d ]*\d|\d)\s*bytes", t)
           if to_int(m.group(1)) >= 1000]
    if big and AUD_BYTES not in big:
        fail("AC 4", f"the audio declared rate's bytes are not {AUD_BYTES} (cap + audio + 66): " + t[:100])

# ---------------------------------------------------------------------------
# AC 4: the controller-stream wire cost of C-219.
# ---------------------------------------------------------------------------
block = [u for u in UNITS if re.search(r"C-219", u["text"]) or
         (re.search(r"controller", u["text"], re.I) and re.search(r"per slice|controller bytes", u["text"], re.I))]
if not block:
    fail("AC 4", "no paragraph states the C-219 wire cost of a controller stream")
else:
    btxt = " ".join(u["text"] for u in block)
    if not re.search(r"(?<![\d.])28 ?\+ ?38(?![\d])", btxt) or not re.search(r"(?<![\d.])66(?![\d.])", btxt):
        fail("AC 4", "the 66 bytes of Ethernet, IP and UDP overhead are not shown as 28 + 38 with 66")
    if not re.search(r"=\s*(?:H|c|B)\b[^=.;]*(?<![\d.])28 ?\+ ?38(?![\d])", btxt):
        fail("AC 4", "no symbolic formula (H, c or B on the left, 28 + 38 on the right of an '=') gives the bytes")
    if not re.search(r"2 ?000", btxt) or not re.search(r"packets?/s|packets? per second|per second", btxt):
        fail("AC 4", "the controller stream is not stated at 2 000 packets/s")
    if not re.search(r"8 ?[×x*] ?2 ?000", btxt):
        fail("AC 4", "no bit-rate formula of the form bytes x 8 x 2 000 for the controller stream")
    if not (re.search(r"(?<![\d.])%d(?![\d.])" % CAP, btxt) and re.search(r"cap|budget", btxt, re.I)):
        fail("AC 4", "the header term of the formula is not named as the 100-byte cap or a Size budget figure")
    cblock = EQS   # worked examples may sit in any paragraph of the subsection
    examples = []
    for e in cblock:
        if e["ops"] and all(o in "+" for o in e["ops"]) and 28 in e["terms"] and 38 in e["terms"] and e["ok"]:
            rest = list(e["terms"])
            rest.remove(28)
            rest.remove(38)
            if len(rest) == 2 and any(r in HDR_ALLOWED for r in rest):
                h = next(r for r in rest if r in HDR_ALLOWED)
                c = rest[1 - rest.index(h)]
                examples.append((e["rhs"], c))
            else:
                fail("AC 4", f"worked example {e['raw']!r}: the header term is not a figure of the budget "
                             f"(the cap 100 or a 004-14 table figure) and two terms remain: {rest}")
    if not examples:
        fail("AC 4", "no worked example 'H + c + 28 + 38 = N' of the C-219 formula (verified by evaluation)")
    controller_totals = {tot for tot, c in examples if c != AUDIO}
    if not controller_totals:
        fail("AC 4", "the C-219 section has no controller example (every worked example carries the audio "
                     f"{AUDIO} bytes as c); a transport state or sparse message example is required")
    bit_ex = [e for e in cblock if e["ok"] and len(e["terms"]) == 3 and 8 in e["terms"] and 2000 in e["terms"]]
    if not bit_ex:
        fail("AC 4", "no worked bit-rate example 'N x 8 x 2 000 = R' of the controller stream")
    else:
        if not any(next(t for t in e["terms"] if t not in (8, 2000)) in controller_totals for e in bit_ex):
            fail("AC 4", "the bit-rate example does not use a controller example's byte count (c not the audio 1 152)")

# ---------------------------------------------------------------------------
# AC 5 (optional toml keys): any integer key naming a wire size, rate or overhead.
# ---------------------------------------------------------------------------
def key_tokens(name):
    return {t for t in re.split(r"[^a-z0-9]+", name.lower()) if t}


for tname, tval in OTHERS.items():
    blocks = tval if isinstance(tval, list) else [tval]
    for block_ in blocks:
        if not isinstance(block_, dict):
            continue
        for k, v in block_.items():
            if not isinstance(v, int) or isinstance(v, bool):
                continue
            toks = key_tokens(k) | key_tokens(tname)
            if not (toks & {"wire", "rate", "overhead"}):
                continue
            want = None
            if "overhead" in toks and not toks & {"ip", "udp", "ethernet", "ipv4"}:
                want, label = {OVH}, f"the overhead of {OVH} bytes"
            elif "ethernet" in toks and toks & {"overhead", "framing"} and not toks & {"ip", "udp"}:
                want, label = {ETH}, f"the Ethernet framing of {ETH} bytes"
            elif toks & {"ipv4", "ip"} and "header" in toks:
                want, label = {20}, "the IPv4 header of 20 bytes"
            elif "udp" in toks and "header" in toks:
                want, label = {8}, "the UDP header of 8 bytes"
            elif "wire" in toks and "default" in toks and not toks & {"rate", "bit", "stream", "channel", "kbit", "mbit", "bps"}:
                want, label = {WIRE}, f"the default on-wire size of {WIRE} bytes"
            elif "wire" in toks and toks & {"worst", "maximum", "max"} and not toks & {"rate", "bit", "stream"}:
                want, label = {WIRE_MAX}, f"the format maximum on-wire size of {WIRE_MAX} bytes"
            elif "rate" in toks and "default" in toks and "stream" in toks:
                want, label = {int(round(BITS_STREAM)), int(round(BITS_STREAM / 1000))}, \
                    f"the default per-stream rate of {BITS_STREAM:.0f} bit/s"
            if want is not None and v not in want:
                fail("AC 5", f"wire.toml [{tname}] key {k!r} = {v}, but {label} is computed from the layout")

# ---------------------------------------------------------------------------
# Report.
# ---------------------------------------------------------------------------
if fails:
    for ac, msg in fails:
        print(f"FAIL {ac}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print(f"OK: 004-15 wire size {WIRE} bytes ({DEF_PAY} + {OVH}), {RATE_STREAM:.1f} Mbit/s per stream; "
      f"difference {DIFF_B:+d} bytes ({PCT_B:+.1f} %), {DIFF_R:+.2f} Mbit/s ({PCT_R:+.1f} %); "
      f"5 % rule {'exceeded' if EXCEEDS else 'not exceeded'}; C-219 formula verified")
PY
PY_RC=$?

if [ "$PY_RC" -ne 0 ]; then
    exit 1
fi
exit 0
