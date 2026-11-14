#!/usr/bin/env bash
# checks/006-02.sh — domain line and position map.
#
# Ticket: doc/design/tickets/006/006-02-Position-map-arithmetic.md
# Validates the "Domain line and position map" section of
# doc/design/spec/timeline.md against the ticket's acceptance criteria,
# including independent integer verification of every worked-example row.
#
# The check is the contract: the section is expected to use
#   position_at(t) = floor((t - origin_ns) * 96 000 / 10^9)
#                  = floor((t - origin_ns) * 12 / 125 000)
#   time_of(p)     = origin_ns + ceil(p * 10^9 / 96 000)
#                  = origin_ns + ceil(p * 31 250 / 3)
# or a rule that is exact at every slice boundary.  Rows are read as
# relative to origin ("origin + 10 417 ns") or absolute network time; the
# checker tries both readings and accepts a row only when the arithmetic
# of the section's own rule holds.
#
# Unprivileged and read-only (C-146); no file is created or modified.

set -u
cd "$(dirname "$0")/.."

exec python3 - <<'PY'
import re
import sys
import unicodedata
from fractions import Fraction
from pathlib import Path

SPEC = Path("doc/design/spec/timeline.md")
HEADING = "Domain line and position map"
RATE_HZ = 96_000
NS_PER_S = 10**9
SLICE = 48
# 96 000 / 10^9 == 12 / 125 000 and 10^9 / 96 000 == 31 250 / 3.
POS_NUM, POS_DEN = 12, 125_000
TIME_NUM, TIME_DEN = 31_250, 3

results = []  # (ac, ok, message)


def record(ac, ok, message):
    results.append((ac, ok, message))


def ok(ac, message):
    record(ac, True, message)


def bad(ac, message):
    record(ac, False, message)


def norm_text(s):
    s = unicodedata.normalize("NFKC", s)
    for ch in ("\u00a0", "\u2007", "\u202f", "\u2009", "\u200a"):
        s = s.replace(ch, " ")
    s = s.replace("\u200b", "").replace("\ufeff", "")
    s = re.sub(r"[\u2010-\u2015\u2212]", "-", s)
    s = (s.replace("\u2018", "'").replace("\u2019", "'")
         .replace("\u201c", '"').replace("\u201d", '"'))
    return re.sub(r"\s+", " ", s).strip()


def norm_lines(text):
    """Normalize each line but keep line structure (for tables)."""
    return [norm_text(line) for line in text.splitlines()]


def find_section(lines):
    """Return (heading_index, end_index) or None."""
    start = None
    level = None
    for i, line in enumerate(lines):
        m = re.match(r"^(#{1,6})\s+(.*?)\s*$", line)
        if m and m.group(2).strip().rstrip("#").strip() == HEADING:
            start = i
            level = len(m.group(1))
            break
    if start is None:
        return None
    end = len(lines)
    for j in range(start + 1, len(lines)):
        m = re.match(r"^(#{1,6})\s+", lines[j])
        if m and len(m.group(1)) <= level:
            end = j
            break
    return start, end


def split_row(line):
    """Split a Markdown table row on unescaped pipes."""
    s = line.strip()
    if not s.startswith("|"):
        return None
    if s.endswith("|") and not s.endswith("\\|"):
        s = s[:-1]
    s = s[1:]
    cells = []
    cur = []
    i = 0
    while i < len(s):
        c = s[i]
        if c == "\\" and i + 1 < len(s) and s[i + 1] == "|":
            cur.append("|")
            i += 2
            continue
        if c == "|":
            cells.append("".join(cur).strip())
            cur = []
            i += 1
            continue
        cur.append(c)
        i += 1
    cells.append("".join(cur).strip())
    return cells


def is_separator(cells):
    return all(re.fullmatch(r":?-{2,}:?", c.replace(" ", "")) is not None
               for c in cells) if cells else False


def visible_lines(lines):
    vis = [True] * len(lines)
    in_fence = False
    marker = None
    for i, line in enumerate(lines):
        m = re.match(r"^\s*(`{3,}|~{3,})", line)
        if m:
            if not in_fence:
                in_fence = True
                marker = m.group(1)[0]
            elif m.group(1)[0] == marker:
                in_fence = False
            vis[i] = False
            continue
        if in_fence:
            vis[i] = False
    return vis


def parse_tables(lines, vis):
    tables = []
    i = 0
    n = len(lines)
    while i < n:
        if vis[i] and lines[i].lstrip().startswith("|"):
            block = []
            j = i
            while j < n and vis[j] and lines[j].lstrip().startswith("|"):
                cells = split_row(lines[j])
                if cells is not None:
                    block.append((j + 1, cells))
                j += 1
            if len(block) >= 3 and is_separator(block[1][1]):
                tables.append((block[0][1], block[2:]))
            i = j
        else:
            i += 1
    return tables


NUM_RE = re.compile(
    r"(?<![\w.\-/])([+\-]?)(\d+(?:[ \u00a0\u202f]\d{3})*(?:\.\d+)?)(?![\w])"
)


def cell_numbers(cell):
    """All numbers in a cell, in order.  Returns [(value, relative), ...].

    relative is True when the cell says "origin + N" (or just "origin").
    """
    out = []
    low = cell.lower()
    rel = "origin" in low
    for m in NUM_RE.finditer(cell):
        sign = -1 if m.group(1) == "-" else 1
        body = m.group(2).replace("_", " ").replace(",", "")
        body = re.sub(r"(?<=\d)[ ](?=\d{3}(?!\d))", "", body)
        try:
            val = float(body) if "." in body else int(body)
        except ValueError:
            continue
        if isinstance(val, float):
            if val.is_integer():
                val = int(val)
            else:
                val = Fraction(val).limit_denominator()
        out.append((sign * val, rel))
    if not out and rel:
        out.append((0, True))
    return out


def _dir_cell(cells, pattern):
    for k, c in enumerate(cells):
        if re.search(pattern, c.lower()):
            return k
    return 0


def row_direction(cells):
    """Return (direction, direction_cell_index) or (None, None)."""
    text = " | ".join(cells).lower()
    if "position_at" in text and "time_of" not in text:
        for k, c in enumerate(cells):
            if "position_at" in c.lower():
                return "position", k
    if "time_of" in text and "position_at" not in text:
        for k, c in enumerate(cells):
            if "time_of" in c.lower():
                return "time", k
    pat = r"(ns|network time|time)\s*(?:→|->|=>)\s*(p\b|pos|position|sample)"
    if re.search(pat, text):
        return "position", _dir_cell(cells, pat)
    pat = r"(p\b|pos|position|sample)\s*(?:→|->|=>)\s*(ns|network time|time)"
    if re.search(pat, text):
        return "time", _dir_cell(cells, pat)
    pat = r"ns\s+to\s+(p\b|position)|time\s+to\s+position"
    if re.search(pat, text):
        return "position", _dir_cell(cells, pat)
    pat = r"position\s+to\s+(ns|time)|p\b\s+to\s+ns"
    if re.search(pat, text):
        return "time", _dir_cell(cells, pat)
    return None, None


def ceil_div(a, b):
    return -((-a) // b)


def floor_to_position(t_abs, origin):
    return ((t_abs - origin) * POS_NUM) // POS_DEN


def ceil_to_offset(p):
    return ceil_div(p * TIME_NUM, TIME_DEN)


def declared_origin(section_text):
    m = re.search(
        r"origin(?:_ns)?\s*(?:=|is|:)\s*([+\-]?\d[\d _,\u00a0\u202f]*)",
        section_text,
        re.IGNORECASE,
    )
    if not m:
        return 0, False
    raw = re.sub(r"[ _,\u00a0\u202f]", "", m.group(1))
    try:
        return int(raw), True
    except ValueError:
        return 0, False


def matches_position(in_val, in_rel, out_val, origin):
    """Does the row satisfy p = floor(((t) - origin) * 12 / 125000)?"""
    if isinstance(out_val, Fraction) or isinstance(in_val, Fraction):
        return False
    p = out_val
    if p < 0:
        return False
    t_choices = [origin + in_val] if in_rel else [in_val, origin + in_val]
    return any(t - origin >= 0 and floor_to_position(t, origin) == p
               for t in t_choices)


def matches_time(in_val, in_rel, out_val, origin):
    """Does the row satisfy t = origin + ceil(p * 31250 / 3)?"""
    if isinstance(in_val, Fraction) or isinstance(out_val, Fraction):
        return False
    p = in_val
    if p < 0:
        return False
    off = ceil_to_offset(p)
    choices = [origin + off] if in_rel else [off, origin + off]
    return out_val in choices


def header_direction(header):
    """Direction implied by a table header, when rows do not say it."""
    text = " ".join(header).lower()
    if "position_at" in text and "time_of" not in text:
        return "position"
    if "time_of" in text and "position_at" not in text:
        return "time"
    time_re = r"network\s*time|time\s*\(ns\)|\bns\b"
    pos_re = r"position|\bp\b|samples?"
    if not (re.search(time_re, text) and re.search(pos_re, text)):
        return None
    t_first = min(m.start() for m in re.finditer(time_re, text))
    p_first = min(m.start() for m in re.finditer(pos_re, text))
    return "position" if t_first < p_first else "time"


def verify_table_rows(tables, origin):
    """Return (verified rows, problems, unparsed warnings)."""
    verified = []
    problems = []
    warnings = []
    for header, rows in tables:
        table_dir = header_direction(header)
        for line_no, cells in rows:
            if not any(c.strip() for c in cells):
                continue
            direction, dir_idx = row_direction(cells)
            if direction is None:
                direction = table_dir
                dir_idx = None
            nums = []
            for k, c in enumerate(cells):
                if k == dir_idx:
                    continue
                for val, rel in cell_numbers(c):
                    if isinstance(val, Fraction):
                        problems.append(
                            "line %d: value %s is not an integer; the map "
                            "produces whole nanoseconds and whole positions"
                            % (line_no, val)
                        )
                    nums.append((val, rel, c.strip()))
            if direction is None:
                if nums:
                    warnings.append(
                        "line %d: table row has numbers but no direction "
                        "(%s)" % (line_no, " | ".join(cells))
                    )
                continue
            if len(nums) < 2:
                problems.append(
                    "line %d: %s row has fewer than two numeric values "
                    "(input and output): | %s |"
                    % (line_no, direction, " | ".join(cells))
                )
                continue
            found = None
            n = len(nums)
            for a in range(n):
                for b in range(a + 1, n):
                    for (x, y) in ((nums[a], nums[b]), (nums[b], nums[a])):
                        if direction == "position":
                            good = matches_position(x[0], x[1], y[0], origin)
                        else:
                            good = matches_time(x[0], x[1], y[0], origin)
                        if good:
                            found = (x, y)
                            break
                    if found:
                        break
                if found:
                    break
            if not found:
                a, b = nums[0], nums[1]
                problems.append(
                    "line %d: %s row fails the section's own formula: "
                    "input %s, output %s  (row: | %s |)"
                    % (line_no, direction, a[0], b[0], " | ".join(cells))
                )
                continue
            x, y = found
            p = y[0] if direction == "position" else x[0]
            verified.append({
                "line": line_no,
                "direction": direction,
                "p": p,
                "input": x[0],
                "output": y[0],
                "relative_in": x[1],
                "relative_out": y[1],
                "cells": cells,
            })
    return verified, problems, warnings


def near(text, a_re, b_re, width=300):
    for m in re.finditer(a_re, text, re.IGNORECASE):
        w = text[max(0, m.start() - width):m.end() + width]
        if re.search(b_re, w, re.IGNORECASE):
            return True
    return False


def main():
    if not SPEC.is_file():
        bad("AC1", "missing %s" % SPEC)
        report()
        return

    raw = SPEC.read_text(encoding="utf-8")
    lines = norm_lines(raw)
    bounds = find_section(lines)

    if bounds is None:
        bad("AC1", "timeline.md has no '## %s' heading" % HEADING)
        for ac in ("AC2", "AC3", "AC4", "AC5", "AC6"):
            bad(ac, "section missing, cannot check")
        report()
        return

    start, end = bounds
    section_lines = lines[start + 1:end]
    section = "\n".join(section_lines)
    T = norm_text(section)

    if not T or re.search(r"to be written in 006-02", T, re.IGNORECASE):
        bad("AC1", "section is still the 006-02 placeholder "
                   "(no formulas, no table)")
        for ac in ("AC2", "AC3", "AC4", "AC5", "AC6"):
            bad(ac, "section is still the placeholder; cannot check")
        report()
        return

    origin, _origin_declared = declared_origin(T)

    # ---------------- AC 1: both formulas, rounding, 128-bit ----------------
    floor_re = r"floor|round(?:ed|ing)?\s+down|toward\s+-?inf|⌊"
    ceil_re = r"ceil|round(?:ed|ing)?\s+up|toward\s+\+?inf|⌈"

    formula_pos = re.search(r"position_at[^\n]{0,140}(?:=|:=|:=|≡)", T)
    formula_time = re.search(r"time_of[^\n]{0,140}(?:=|:=|:=|≡)", T)
    if formula_pos and formula_time:
        ok("AC1", "gives formulas for both position_at and time_of")
    else:
        if not formula_pos:
            bad("AC1", "no 'position_at(...) = ...' formula")
        if not formula_time:
            bad("AC1", "no 'time_of(...) = ...' formula")

    if near(T, r"position_at", floor_re, 260):
        ok("AC1", "position_at states floor (round down)")
    else:
        bad("AC1", "position_at does not state floor/round down")

    if near(T, r"time_of", ceil_re, 260):
        ok("AC1", "time_of states ceil (round up)")
    else:
        bad("AC1", "time_of does not state ceil/round up")

    if re.search(r"\b[ui]?128\b|128[\s-]*bit", T, re.IGNORECASE):
        ok("AC1", "names a 128-bit (or wider) intermediate")
    else:
        bad("AC1", "does not name a 128-bit (or wider) intermediate for "
                   "(t - origin) * 96 000")

    # ---------------- AC 2: inverse identity and proof ----------------------
    ident_re = (r"position_at\s*\(\s*time_of\s*\(\s*([A-Za-z][A-Za-z0-9_]*)"
                r"\s*\)\s*\)\s*(?:==|=|≡)\s*\1")
    if re.search(ident_re, T):
        ok("AC2", "states position_at(time_of(p)) == p")
    else:
        bad("AC2", "does not state position_at(time_of(p)) == p")

    second_re = (r"time_of\s*\(\s*position_at\s*\(\s*([A-Za-z][A-Za-z0-9_]*)"
                 r"\s*\)\s*\)\s*(?:≤|<=|\\leq)\s*\1")
    if re.search(second_re, T):
        ok("AC2", "states the second identity time_of(position_at(t)) <= t")
    else:
        bad("AC2", "does not state time_of(position_at(t)) <= t")

    has48 = re.search(r"\b48\b", T) is not None
    has500k = re.search(r"\b500\s*000\b|\b500000\b", T) is not None
    if has48 and has500k:
        ok("AC2", "uses 48 samples = 500 000 ns")
    else:
        bad("AC2", "does not give 48 samples = 500 000 ns")

    boundary_exact = near(T, r"\b500\s*000\b|\b500000\b",
                          r"exact|without\s+rounding|whole\s+nanosecond|"
                          r"no\s+rounding", 260)
    if boundary_exact and "boundar" in T.lower():
        ok("AC2", "argues slice boundaries map to whole nanoseconds exactly")
    elif boundary_exact:
        bad("AC2", "exactness argument does not mention slice boundaries")
    else:
        bad("AC2", "no argument that 48-sample slice boundaries are exact "
                   "(500 000 ns exactly)")

    red_ok = (re.search(r"\b12\s*/\s*125\s*000\b", T) is not None
              or re.search(r"\b125\s*000\s*/\s*12\b", T) is not None
              or near(T, r"\b125\s*000\b", r"\b12\b", 80))
    if red_ok:
        ok("AC2", "shows the 96 000/10^9 = 12/125 000 reduction")
    else:
        bad("AC2", "does not show the 12 / 125 000 reduction")

    rev_ok = (re.search(r"\b31\s*250\s*/\s*3\b", T) is not None
              or near(T, r"\b31\s*250\b", r"\b3\b", 80))
    if rev_ok:
        ok("AC2", "shows the 10^9/96 000 = 31 250/3 reduction")
    else:
        bad("AC2", "does not show the 31 250 / 3 reduction")

    # Symbolic and exhaustive self-verification of the stated constants.
    problems = []
    if Fraction(RATE_HZ, NS_PER_S) != Fraction(POS_NUM, POS_DEN):
        problems.append("96 000/10^9 != 12/125 000")
    if Fraction(NS_PER_S, RATE_HZ) != Fraction(TIME_NUM, TIME_DEN):
        problems.append("10^9/96 000 != 31 250/3")
    for p in range(0, 300_001):
        t = ceil_to_offset(p)
        if floor_to_position(t, 0) != p:
            problems.append("position_at(time_of(%d)) != %d" % (p, p))
            break
    for k in range(0, 20_001):
        p = SLICE * k
        if (p * TIME_NUM) % TIME_DEN != 0:
            problems.append("slice boundary p=%d is not whole ns" % p)
            break
        t = p * TIME_NUM // TIME_DEN
        if floor_to_position(t, 0) != p:
            problems.append("boundary inverse fails at p=%d" % p)
            break
    bad_t = None
    for t in range(0, 300_001):
        if ceil_to_offset(floor_to_position(t, 0)) > t:
            bad_t = t
            break
    if bad_t is not None:
        problems.append("time_of(position_at(%d)) > %d" % (bad_t, bad_t))
    if problems:
        bad("AC2", "constant self-check failed: %s" % "; ".join(problems))
    else:
        ok("AC2", "arithmetic self-check: 96 000/10^9 = 12/125 000, "
                  "10^9/96 000 = 31 250/3, inverse holds for p=0..300000, "
                  "boundaries exact for k=0..20000")

    # ---------------- AC 3: off-boundary rounding rule ----------------------
    off_re = (r"off[\s-]*(?:a\s+)?(?:slice\s+)?boundar|"
              r"inside\s+(?:a\s+)?slice|within\s+(?:a\s+)?slice|"
              r"not\s+on\s+a\s+slice\s+boundar|off[\s-]boundar")
    off_rule_re = (ceil_re + r"|next\s+(?:whole\s+)?nanosecond|"
                   r"round(?:ed|ing)?\s+(?:up|upward)")
    if near(T, off_re, off_rule_re, 300) and \
            near(T, off_re, r"position|sample", 300):
        ok("AC3", "states the rounding rule for positions off a slice boundary")
    else:
        bad("AC3", "does not state the rounding rule for positions off a "
                   "slice boundary (ceil/round up near the off-boundary case)")

    # ---------------- AC 4: behaviour before the origin ---------------------
    before_re = (r"\bt\s*<\s*origin|origin[\s_]*ns\s*[<≤]|"
                 r"origin[\s_]*ns\s*>\s*t|"
                 r"before\s+(?:the\s+)?(?:epoch[\s_]*)?origin|"
                 r"earlier\s+than\s+(?:the\s+)?origin|"
                 r"prior\s+to\s+(?:the\s+)?origin|"
                 r"negative\s+(?:network\s+)?(?:time|position)|"
                 r"would\s+be\s+negative|"
                 r"time\s+before\s+(?:the\s+)?origin")
    mech_re = (r"Option|None|Err\b|error|clamp|saturat|panic|reject|"
               r"illegal|out\s+of\s+range")
    decision_win = None
    for m in re.finditer(before_re, T, re.IGNORECASE):
        w = T[max(0, m.start() - 400):m.end() + 400]
        if re.search(r"\[decision\]", w) and re.search(mech_re, w):
            decision_win = w
            break
    if decision_win is None:
        bad("AC4", "behaviour of position_at for t before epoch origin is "
                   "not specified and tagged [decision]")
    else:
        ok("AC4", "specifies before-origin behaviour with [decision]")
        ret_re = (r"\bNone\b|\bErr\b|error|returns?\s+0\b|zero|u64::MAX|MAX|"
                  r"origin|clamp(?:s|ed|ing)?|saturat")
        if re.search(ret_re, decision_win, re.IGNORECASE):
            ok("AC4", "states the concrete result (error/Option or clamp)")
        else:
            bad("AC4", "before-origin rule does not say what is returned "
                       "(None/Err or the clamped value)")

    # ---------------- AC 5 / AC 3: worked-example table + arithmetic --------
    vis = visible_lines(section_lines)
    tables = parse_tables(section_lines, vis)
    verified, problems, warnings = verify_table_rows(tables, origin)

    for prob in problems:
        bad("AC5", prob)

    if len(verified) >= 6:
        ok("AC5", "worked-example table has %d verified rows" % len(verified))
    else:
        bad("AC5", "worked-example table has %d verified row(s); at least 6 "
                   "are required" % len(verified))

    directions = {v["direction"] for v in verified}
    if {"position", "time"}.issubset(directions):
        ok("AC5", "table gives both directions (position_at and time_of)")
    elif directions:
        bad("AC5", "table does not give both directions; found: %s"
            % ", ".join(sorted(directions)))
    else:
        bad("AC5", "table does not give both directions; no examples parsed")

    boundary_rows = [v for v in verified if v["p"] % SLICE == 0]
    off_rows = [v for v in verified if v["p"] % SLICE != 0]
    if boundary_rows:
        ok("AC5", "%d slice-boundary row(s) verified" % len(boundary_rows))
    else:
        bad("AC5", "no slice-boundary example row (p divisible by 48)")

    if len(off_rows) >= 3:
        ok("AC3", "%d off-boundary example rows verified" % len(off_rows))
    else:
        bad("AC3", "%d off-boundary example row(s); at least three are "
                   "required" % len(off_rows))

    # The ticket's own illustrative values, if the table uses p = 1, 2, 3.
    # Only time_of rows say what a position maps to in nanoseconds.
    offsets_by_p = {}
    for v in verified:
        if v["direction"] != "time":
            continue
        offsets = {v["output"]} if v["relative_out"] else {
            v["output"], v["output"] - origin}
        offsets_by_p.setdefault(v["p"], set()).update(offsets)
    for p, expected in ((1, 10_417), (2, 20_834), (3, 31_250)):
        if p in offsets_by_p:
            if expected in offsets_by_p[p]:
                ok("AC5", "ticket example p=%d -> %d ns verified"
                   % (p, expected))
            else:
                bad("AC5", "ticket example p=%d should be %d ns, table says %s"
                    % (p, expected, sorted(offsets_by_p[p])))

    if warnings:
        for w in warnings[:8]:
            print("  note: %s" % w)

    # ---------------- AC 6: only rate with a position space -----------------
    only_rate = near(T, r"position[\s-]*space",
                     r"\bonly\b|no\s+other|single|sole", 260) or \
        near(T, r"\bonly\s+(?:sample\s+)?rate\b",
             r"position|\b96", 260)
    if only_rate:
        ok("AC6", "states 96 kHz is the only rate with a position space")
    else:
        bad("AC6", "does not state that 96 kHz is the only rate with a "
                   "position space")

    cites = {cid: cid in T for cid in ("C-01", "C-80", "C-134")}
    missing = [cid for cid, present in cites.items() if not present]
    if missing:
        bad("AC6", "missing citation(s): %s" % ", ".join(missing))
    else:
        ok("AC6", "cites C-01, C-80 and C-134")

    report()


def report():
    print("006-02: domain line and position map")
    print("  spec: doc/design/spec/timeline.md")
    if not results:
        print("  OK: all checks passed")
        sys.exit(0)

    order = ["AC1", "AC2", "AC3", "AC4", "AC5", "AC6"]
    grouped = {}
    for ac, passed, message in results:
        grouped.setdefault(ac, []).append((passed, message))
    for ac in order:
        if ac not in grouped:
            continue
        print("  %s" % ac)
        for passed, message in grouped[ac]:
            print("    %s %s" % ("OK  " if passed else "FAIL", message))

    failures = sum(1 for _, passed, _ in results if not passed)
    if failures == 0:
        print("RESULT: OK (all %d checks passed)" % len(results))
        sys.exit(0)
    print("RESULT: FAIL (%d check%s failed)"
          % (failures, "" if failures == 1 else "s"))
    sys.exit(1)


if __name__ == "__main__":
    main()
PY
