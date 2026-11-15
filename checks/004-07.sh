#!/usr/bin/env bash
# checks/004-07.sh — timeline epoch wrap analysis (C-10, 16 bits).
#
# RED-phase document check. It validates the epoch-wrap analysis that
# ticket doc/design/tickets/004/004-07-Timeline-epoch-wrap-analysis.md
# requires in doc/design/spec/wire.md (recorded beside the epoch field of
# the media packet header), against the ticket's acceptance criteria, and
# encodes the arithmetic itself: 65 536 epochs (2^16, C-10) at the stated
# rates against the 1-year falsifier of C-10's row ("an epoch-bump rate
# that wraps it in under a year of operation").
#
# WIRE_MD / CONST_MD / OPEN_MD may override the document paths. The
# overrides are for self-testing on throwaway fixtures only: when any path
# is overridden the repository-level checks (the design checker and
# checks/004-01.sh) are skipped with a NOTE, because they always target the
# real repository.
#
# What is checked, per acceptance criterion:
#
#   AC 1  wire.md carries the analysis (a heading about the epoch and its
#         wrap/bump/rate, else inside the Media packet header section). It
#         must state BOTH rates — the pathological ceiling and the plausible
#         worst-case operating rate — each tagged [decision] or [memory],
#         name their sources (C-113, C-213, C-136, C-32, C-33, C-37), state
#         the time-to-wrap for 65 536 values at each rate, and give a
#         decidable verdict against the 1-year falsifier. The arithmetic is
#         re-computed here: wrap seconds = 65 536 × the stated per-bump
#         interval (or 65 536 / the stated rate); the stated time-to-wrap
#         must agree with it (30% tolerance for coarse rounding), the
#         stated pathological interval must not be faster than the election
#         timers allow (C-32 1.5 s + C-33 400 ms = 1.9 s minimum per
#         election-driven bump), the plausible rate must not exceed the
#         ceiling, and the verdict must match the arithmetic: plausible
#         wrap >= 1 year (31 557 600 s) -> C-10 not falsified; < 1 year ->
#         falsified. When the pathological ceiling alone wraps in under a
#         year — it always does at the timer bound, 65 536 × 1.9 s is about
#         1.4 days — the verdict paragraph must address the ceiling
#         explicitly, so the falsifier is answered either way.
#   AC 2  A comparison rule for epochs — how a receiver decides "current"
#         vs "stale" — is specified, including its behaviour at wrap. The
#         rule is looked for in the analysis section first, then anywhere
#         in wire.md (the ticket does not pin its location).
#   AC 3  Conditional: if the verdict is "falsified" (or the plausible
#         wrap lands under a year), the falsification, the proposed fix
#         (serial-number arithmetic or a wider field), the flag for the
#         constants table, and an open item with an owner and a falsifier
#         (05-risks-and-open.md row, or stated in the analysis) must all
#         exist. If not falsified, the criterion is satisfied vacuously —
#         but any open item the analysis raises anyway must carry an owner
#         and a falsifier.
#
# DoD also: every rate and the verdict carry an evidence tag, the section
# carries at least three tags overall ([code], [measured], [kernel-doc],
# [standard], [memory], [decision]); the design checker prints OK; and
# checks/004-01.sh still passes (the analysis must not break the skeleton).
set -u
cd "$(dirname "$0")/.."

export WIRE_MD="${WIRE_MD:-doc/design/spec/wire.md}"
export CONST_MD="${CONST_MD:-doc/design/02-constants.md}"
export OPEN_MD="${OPEN_MD:-doc/design/05-risks-and-open.md}"

exec python3 - <<'PY'
import os
import re
import subprocess
import sys
from pathlib import Path

WIRE = Path(os.environ["WIRE_MD"])
CONST = Path(os.environ["CONST_MD"])
OPEN = Path(os.environ["OPEN_MD"])
OVERRIDDEN = (
    os.environ["WIRE_MD"] != "doc/design/spec/wire.md"
    or os.environ["CONST_MD"] != "doc/design/02-constants.md"
    or os.environ["OPEN_MD"] != "doc/design/05-risks-and-open.md"
)

MODULUS = 65_536            # 2^16 epoch values (C-10)
YEAR_S = 31_557_600.0       # 1 year = 365.25 days
MIN_CYCLE_S = 1.9           # C-32 1.5 s + C-33 0.4 s: fastest election cycle
TOL = 0.30                  # tolerance for coarsely rounded stated figures

results = []


def record(ac, passed, message):
    results.append((ac, passed, message))


def ok(ac, message):
    record(ac, True, message)


def bad(ac, message):
    record(ac, False, message)


TAG_RE = re.compile(r"\[(decision|memory|measured|code|standard|kernel-doc)\]")
RATE_TAG_RE = re.compile(r"\[(decision|memory)\]")

UNITS = {
    "ms": 0.001, "millisecond": 0.001, "milliseconds": 0.001,
    "s": 1.0, "sec": 1.0, "secs": 1.0, "second": 1.0, "seconds": 1.0,
    "min": 60.0, "mins": 60.0, "minute": 60.0, "minutes": 60.0,
    "h": 3600.0, "hr": 3600.0, "hrs": 3600.0, "hour": 3600.0, "hours": 3600.0,
    "day": 86400.0, "days": 86400.0,
    "week": 604800.0, "weeks": 604800.0,
    "month": 2629800.0, "months": 2629800.0,
    "year": 31557600.0, "years": 31557600.0,
}

UNIT_ALT = (
    r"(?:ms|milliseconds?|seconds?|secs?|minutes?|mins?|hours?|hrs?|days?|"
    r"weeks?|months?|years?|min|h|hr|s)\b"
)
NUM = r"(\d[\d,\u00a0\u202f ]*(?:\.\d+)?)"

BUMP = r"(?:bumps?|changes?|switches?|epochs?|events?)"

# Interval patterns, most specific tier first. Tier A anchors the phrase on
# a bump/epoch word so "2 000 slices per second" cannot be mistaken for a
# bump interval; tier B is the bare "every/per N <unit>" form.
INTERVAL_TIERS = [
    [  # tier A: "one bump every 3.4 s", "epochs change every 10 minutes"
        rf"{BUMP}[\w\s]{{0,30}}?(?:every|per)\s+(?:(?:one|a|1)\s+)?"
        rf"{NUM}\s*({UNIT_ALT})",
    ],
    [  # tier B
        rf"(?:every|per)\s+(?:(?:one|a|1)\s+)?{NUM}\s*({UNIT_ALT})",
    ],
    [  # tier C: "1.9 s per bump", "65 536 × 3.4 s"
        rf"{NUM}\s*({UNIT_ALT})\s*(?:of\s+(?:network\s+)?time\s+)?"
        rf"per\s+(?:timeline\s+)?epoch\s+(?:bump|change|switch)",
        rf"{NUM}\s*({UNIT_ALT})\s*per\s+(?:epoch\s+)?bump",
        rf"65[\d,\u00a0\u202f ]*536\s*[×x*]\s*{NUM}\s*({UNIT_ALT})",
    ],
]

# Rate patterns ("0.29 bumps per second"), same tiering.
RATE_TIERS = [
    [rf"{NUM}\s*{BUMP}\s*(?:/\s*)?per\s*({UNIT_ALT})"],
    [rf"rate\s+of\s+{NUM}\s*per\s*({UNIT_ALT})"],
]

DUR_RE = re.compile(rf"(?<![\w.]){NUM}\s*({UNIT_ALT})", re.IGNORECASE)

PATHO_MARKER = r"patholog|ceiling"
PLAUS_PRIMARY = (
    r"plausible\s+(?:worst[\s-]*case\s+)?(?:operating\s+)?rate|"
    r"worst[\s-]*case(?:\s+operating)?\s+rate|operating\s+rate|"
    r"plausible\s+(?:bump|epoch|rate)"
)
PLAUS_SECONDARY = r"plausib|worst[\s-]*case|operating"


def norm(text):
    text = text.replace("\u00a0", " ").replace("\u2007", " ")
    text = text.replace("\u202f", " ").replace("\u2212", "-")
    return re.sub(r"[ \t]+", " ", text)


def unit_seconds(u):
    return UNITS.get(u.lower().rstrip("."))


def parse_number(s):
    try:
        return float(re.sub(r"[,\u00a0\u202f ]", "", s))
    except ValueError:
        return None


def first_match(tiers, text):
    """Search the pattern tiers in order; return (seconds_value, matched)."""
    for patterns in tiers:
        for pat in patterns:
            m = re.search(pat, text, re.IGNORECASE)
            if not m:
                continue
            value = parse_number(m.group(1))
            scale = unit_seconds(m.group(2))
            if value is None or scale is None or value <= 0:
                continue
            return value * scale, m.group(0)
    return None, None


def stated_durations(text):
    """All stated durations as (seconds, position)."""
    out = []
    for m in DUR_RE.finditer(text):
        value = parse_number(m.group(1))
        scale = unit_seconds(m.group(2))
        if value is None or scale is None:
            continue
        out.append((value * scale, m.start()))
    return out


def rate_candidates(marker_re, text):
    """For each marker occurrence, the bump rate stated AFTER it.

    Returns a list of dicts with rate (bumps/s), interval (s per bump),
    matched text, and the marker position; only occurrences with a
    parseable rate are returned.
    """
    out = []
    for m in re.finditer(marker_re, text, re.IGNORECASE):
        tail = text[m.end():]
        interval, matched = first_match(INTERVAL_TIERS, tail)
        if interval is None:
            rate, matched = first_match(RATE_TIERS, tail)
            if rate is None:
                continue
            interval = 1.0 / rate
        out.append({
            "rate": 1.0 / interval,
            "interval": interval,
            "matched": matched,
            "pos": m.start(),
        })
    return out


def pairs_with_stated_wrap(candidate, durations, text):
    """Does a stated duration after (or near) the marker agree with
    65 536 x interval?"""
    computed = MODULUS * candidate["interval"]
    pos = candidate["pos"]
    pools = [
        [s for s, p in durations if pos <= p <= pos + 900],
        [s for s, p in durations
         if pos - 700 <= p <= pos + 900],
    ]
    for pool in pools:
        if any(abs(s - computed) <= TOL * computed for s in pool):
            return computed
    return None


def find_heading(lines, pred):
    for i, line in enumerate(lines):
        m = re.match(r"^(#{1,6})\s+(.*?)\s*#*\s*$", line)
        if m and pred(m.group(2).lower()):
            return i, len(m.group(1))
    return None


def section_bounds(lines, start, level):
    end = len(lines)
    for j in range(start + 1, len(lines)):
        m = re.match(r"^(#{1,6})\s+", lines[j])
        if m and len(m.group(1)) <= level:
            end = j
            break
    return start + 1, end


def classify_verdict(text):
    """(falsified_claimed, not_falsified_claimed)."""
    falsified = False
    clear = False
    for m in re.finditer(r"falsified", text, re.IGNORECASE):
        pre = text[max(0, m.start() - 40):m.start()].lower()
        if re.search(r"\b(?:not|never|no)\s*$", pre):
            clear = True
        else:
            falsified = True
    if re.search(r"falsifier\s+(?:does\s+not|is\s+not)\s+"
                 r"(?:bite|met|satisfied|confirmed)", text, re.IGNORECASE):
        clear = True
    if re.search(r"falsifier\s+(?:bites|is\s+met|is\s+satisfied|"
                 r"is\s+confirmed)", text, re.IGNORECASE):
        falsified = True
    if re.search(r"C-10\s+(?:holds|stands|survives)", text, re.IGNORECASE):
        clear = True
    return falsified, clear


def comparison_rule_found(text):
    """A current-vs-stale epoch comparison rule that addresses the wrap."""
    for m in re.finditer(r"stale", text, re.IGNORECASE):
        w = text[max(0, m.start() - 350):m.end() + 350]
        low = w.lower()
        if ("current" not in low or "epoch" not in low or "wrap" not in low):
            continue
        if re.search(r"equal|serial|modular|compar|same\b|differs?", low):
            return True
    return False


def analyse_rate(key, marker_tiers, durations, section, ceiling_interval):
    """Find the rate for one kind ('pathological'/'plausible').

    Returns (chosen, paired_wrap).  chosen is None when no rate is stated;
    paired_wrap is None when no stated time-to-wrap agrees with the
    arithmetic.
    """
    chosen = None
    paired = None
    fallback = None
    for marker_re in marker_tiers:
        for cand in rate_candidates(marker_re, section):
            if ceiling_interval is not None and cand["interval"] < \
                    ceiling_interval - 1e-9:
                # A 'plausible' rate faster than the stated ceiling is
                # internally inconsistent; keep it only as a last resort.
                if fallback is None:
                    fallback = cand
                continue
            wrap = pairs_with_stated_wrap(cand, durations, section)
            if wrap is not None:
                return cand, wrap
            if chosen is None:
                chosen = cand
    if chosen is None:
        chosen = fallback
    if chosen is not None:
        paired = pairs_with_stated_wrap(chosen, durations, section)
    return chosen, paired


def main():
    if not WIRE.is_file():
        bad("AC1", "missing %s" % WIRE)
        bad("AC2", "wire.md missing, cannot check")
        bad("AC3", "wire.md missing, cannot check")
        bad("DoD", "wire.md missing, cannot check")
        report()
        return

    wire_text = norm(WIRE.read_text(encoding="utf-8"))
    wire_lines = wire_text.splitlines()

    # ------------------------------------------------------------------
    # Locate the analysis: a heading about the epoch and its wrap/bump/
    # rate, else the Media packet header section (the analysis lives
    # "beside the epoch field").
    # ------------------------------------------------------------------
    bounds = find_heading(
        wire_lines,
        lambda h: "epoch" in h
        and re.search(r"wrap|bump|rate|width|falsif", h),
    )
    if bounds is None:
        bounds = find_heading(wire_lines, lambda h: "media packet header" in h)
    if bounds is None:
        bad("AC1", "wire.md has neither an epoch wrap/bump section nor the "
                   "Media packet header section")
        bad("AC2", "no analysis section to read the comparison rule from")
        bad("AC3", "no analysis section; the falsifier is unanswered")
        bad("DoD", "no analysis section; no evidence tags to check")
        report()
        return

    start, level = bounds
    s, e = section_bounds(wire_lines, start, level)
    section = norm("\n".join(wire_lines[s:e]))

    if re.search(r"to be written", section, re.IGNORECASE) or \
            not section.strip():
        bad("AC1", "the analysis section is still a placeholder or empty")
        bad("AC2", "the analysis section is still a placeholder or empty")
        bad("AC3", "the analysis section is still a placeholder or empty")
        bad("DoD", "the analysis section is still a placeholder or empty")
        report()
        return

    durations = stated_durations(section)

    # ---------------- AC 1: both rates, sources, wrap, verdict ----------
    if not re.search(r"65[\d,\u00a0\u202f ]*536|2\^16|2\*\*16", section):
        bad("AC1", "the analysis never states the 65 536 (2^16, C-10) "
                   "epoch values the wrap is computed over")
    else:
        ok("AC1", "states the 65 536 epoch values (2^16, C-10)")

    sources = ["C-113", "C-213", "C-136", "C-32", "C-33", "C-37"]
    missing = [c for c in sources if c not in section]
    if missing:
        bad("AC1", "missing source citation(s): %s" % ", ".join(missing))
    else:
        ok("AC1", "cites all six sources (C-113, C-213, C-136, "
                  "C-32, C-33, C-37)")

    patho, patho_wrap = analyse_rate(
        "pathological", [PATHO_MARKER], durations, section, None)
    plaus_interval = patho["interval"] if patho else None
    plaus, plaus_wrap = analyse_rate(
        "plausible", [PLAUS_PRIMARY, PLAUS_SECONDARY], durations, section,
        plaus_interval)

    for key, cand, wrap in (("pathological", patho, patho_wrap),
                            ("plausible", plaus, plaus_wrap)):
        if cand is None:
            bad("AC1", "no %s rate stated (a bump interval or rate in the "
                       "text after the words 'pathological'/'plausible')"
                % key)
            continue
        if wrap is None:
            bad("AC1",
                "the %s rate ('%s' -> one bump per %.4g s) states no "
                "matching time-to-wrap: expected about %.4g s "
                "(65 536 x interval) after it in the section"
                % (key, cand["matched"], cand["interval"],
                   MODULUS * cand["interval"]))
        else:
            ok("AC1",
               "%s rate '%s' -> one bump per %.4g s; stated time-to-wrap "
               "agrees with 65 536 x interval = %.4g s (%.4g days)"
               % (key, cand["matched"], cand["interval"], wrap,
                  wrap / 86400.0))

    # The ceiling must respect the election timers; the plausible rate must
    # not exceed the ceiling.
    if patho is not None:
        if patho["interval"] < MIN_CYCLE_S - 1e-9:
            bad("AC1",
                "the pathological ceiling (one bump per %.4g s) is faster "
                "than the election timers allow: C-32 (1.5 s) + C-33 "
                "(0.4 s) bound a bump at %.4g s"
                % (patho["interval"], MIN_CYCLE_S))
        else:
            ok("AC1",
               "the pathological ceiling (one bump per %.4g s) respects the "
               "election timers (C-32 + C-33 >= %.4g s per bump)"
               % (patho["interval"], MIN_CYCLE_S))
    if patho is not None and plaus is not None:
        if plaus["interval"] < patho["interval"] - 1e-9:
            bad("AC1",
                "internal inconsistency: the plausible rate (one bump per "
                "%.4g s) is faster than the stated pathological ceiling "
                "(one bump per %.4g s)"
                % (plaus["interval"], patho["interval"]))
        else:
            ok("AC1",
               "the plausible rate does not exceed the pathological ceiling")

    # The verdict.
    threshold = re.search(r"1[\s\u00a0]?year|one\s+year|a\s+year|"
                          r"365\s*days", section, re.IGNORECASE)
    if not threshold:
        bad("AC1", "the 1-year falsifier threshold is never named "
                   "('1 year', 'one year', '365 days')")
    else:
        ok("AC1", "names the 1-year falsifier threshold")

    falsified_claimed, clear_claimed = classify_verdict(section)
    if not falsified_claimed and not clear_claimed:
        bad("AC1", "no decidable verdict against the falsifier ('falsified' "
                   "or 'not falsified')")
    else:
        ok("AC1", "a verdict against the falsifier is stated (%s)"
           % ("falsified" if falsified_claimed else "not falsified"))

    verdict_consistent = True
    if falsified_claimed and clear_claimed:
        bad("AC1", "the verdict is contradictory (both 'falsified' and "
                   "'not falsified' claimed)")
        verdict_consistent = False

    plaus_wrap_final = plaus_wrap
    if plaus is not None and plaus_wrap is None:
        plaus_wrap_final = MODULUS * plaus["interval"]

    if plaus_wrap_final is not None and verdict_consistent:
        if plaus_wrap_final >= YEAR_S:
            if falsified_claimed:
                bad("AC1",
                    "the verdict contradicts the arithmetic: at the "
                    "plausible rate 65 536 epochs take %.4g s (%.4g days) "
                    ">= 1 year, yet the analysis says falsified"
                    % (plaus_wrap_final, plaus_wrap_final / 86400.0))
            else:
                ok("AC1",
                   "verdict consistent: the plausible rate wraps 65 536 "
                   "epochs in %.4g s (%.4g days) >= 1 year, so C-10 stands"
                   % (plaus_wrap_final, plaus_wrap_final / 86400.0))
        else:
            if clear_claimed:
                bad("AC1",
                    "the verdict contradicts the arithmetic: at the "
                    "plausible rate 65 536 epochs take only %.4g s (%.4g "
                    "days) < 1 year, yet the analysis says not falsified"
                    % (plaus_wrap_final, plaus_wrap_final / 86400.0))
            else:
                ok("AC1",
                   "verdict consistent: the plausible rate wraps 65 536 "
                   "epochs in %.4g s (%.4g days) < 1 year, so C-10 is "
                   "falsified" % (plaus_wrap_final, plaus_wrap_final / 86400.0))

    # The falsifier must be answered for the ceiling too: at the timer
    # bound the ceiling always wraps in days.
    if patho is not None:
        p_wrap = patho_wrap if patho_wrap is not None \
            else MODULUS * patho["interval"]
        if p_wrap < YEAR_S:
            vw = [section[max(0, m.start() - 500):m.end() + 500]
                  for m in re.finditer(r"falsif|verdict", section,
                                       re.IGNORECASE)]
            if vw and not any(re.search(r"patholog|ceiling", w, re.IGNORECASE)
                              for w in vw):
                bad("AC1",
                    "the pathological ceiling wraps 65 536 epochs in %.4g s "
                    "(%.4g days), under a year, but the verdict never "
                    "addresses the ceiling — the falsifier is not answered "
                    "for it" % (p_wrap, p_wrap / 86400.0))
            else:
                ok("AC1",
                   "the verdict addresses the pathological ceiling, which "
                   "wraps in %.4g days at the timer bound"
                   % (p_wrap / 86400.0))

    # ---------------- AC 2: the current-vs-stale comparison rule --------
    if comparison_rule_found(section):
        ok("AC2", "a current-vs-stale epoch comparison rule is specified "
                  "in the analysis, including behaviour at wrap")
    elif comparison_rule_found(wire_text):
        ok("AC2", "a current-vs-stale epoch comparison rule is specified "
                  "elsewhere in wire.md, including behaviour at wrap")
    else:
        bad("AC2", "no comparison rule for 'current' vs 'stale' epochs "
                   "that addresses behaviour at wrap (an epoch, current, "
                   "stale, a comparison and 'wrap' must appear together)")

    # ---------------- AC 3: conditional falsification record ------------
    really_falsified = falsified_claimed or (
        plaus_wrap_final is not None and plaus_wrap_final < YEAR_S
    )
    open_text = ""
    if OPEN.is_file():
        open_text = norm(OPEN.read_text(encoding="utf-8"))

    if really_falsified:
        if not falsified_claimed:
            bad("AC3", "the plausible rate wraps under a year but the "
                       "analysis does not declare C-10 falsified")
        else:
            ok("AC3", "the falsification is declared")
        if re.search(r"serial[\s-]*number|serial[\s-]*arithmetic|wider|"
                     r"replacement\s+width|proposed\s+width|"
                     r"\d{1,2}[\s\u00a0-]{1,3}bits?[^.]{0,30}\bepoch\b|"
                     r"\bepoch\b[^.]{0,30}\d{1,2}[\s\u00a0]*bits?",
                     section, re.IGNORECASE):
            ok("AC3", "the proposed replacement (serial-number arithmetic "
                      "or a wider field) is stated")
        else:
            bad("AC3", "no proposed replacement width or wrap-safe "
                       "(serial-number) comparison rule")
        if "constants table" in section.lower():
            ok("AC3", "the change is flagged for the constants table")
        else:
            bad("AC3", "the falsification is not flagged for the constants "
                       "table (DoD: constants table first)")
        # An open item with an owner and a falsifier: a row in
        # 05-risks-and-open.md, or stated inside the analysis.
        open_row = False
        for line in open_text.splitlines():
            if not line.lstrip().startswith("|"):
                continue
            if not re.search(r"epoch|wrap", line, re.IGNORECASE):
                continue
            cells = [c.strip() for c in line.split("|")]
            if len(cells) >= 5:
                owner, falsifier = cells[-3], cells[-2]
                if owner not in ("", "—", "-") and falsifier not in (
                        "", "—", "-"):
                    open_row = True
        in_text = ("owner" in section.lower()
                   and re.search(r"falsif", section, re.IGNORECASE))
        if open_row or in_text:
            ok("AC3", "the open item has an owner and a falsifier")
        else:
            bad("AC3", "no open item with an owner and a falsifier for the "
                       "falsified C-10 width")
    else:
        if falsified_claimed:
            bad("AC3", "the analysis claims falsified but the plausible "
                       "wrap is at or above a year")
        else:
            ok("AC3", "not falsified at the plausible rate; no falsification "
                      "record is required")
        for m in re.finditer(r"open\s+item", section, re.IGNORECASE):
            w = section[max(0, m.start() - 300):m.end() + 300]
            if not (re.search(r"owner", w, re.IGNORECASE)
                    and re.search(r"falsif", w, re.IGNORECASE)):
                bad("DoD", "the analysis raises an open item without an "
                           "owner and a falsifier beside it")
                break

    # ---------------- DoD: evidence tags --------------------------------
    tags = TAG_RE.findall(section)
    if len(tags) < 3:
        bad("DoD", "the analysis carries %d evidence tag(s); at least 3 are "
                   "required ([code], [measured], [kernel-doc], [standard], "
                   "[memory], [decision])" % len(tags))
    else:
        ok("DoD", "%d evidence tags in the analysis" % len(tags))

    for key, marker in (("pathological", PATHO_MARKER),
                        ("plausible", PLAUS_PRIMARY)):
        m = re.search(marker, section, re.IGNORECASE)
        if m is None:
            continue
        w = section[max(0, m.start() - 500):m.end() + 500]
        if RATE_TAG_RE.search(w):
            ok("DoD", "the %s rate carries a [decision] or [memory] tag" % key)
        else:
            bad("DoD", "the %s rate carries no [decision]/[memory] tag" % key)

    vm = re.search(r"falsif|verdict", section, re.IGNORECASE)
    if vm is not None:
        vw = section[max(0, vm.start() - 500):vm.end() + 500]
        if TAG_RE.search(vw):
            ok("DoD", "the verdict carries an evidence tag")
        else:
            bad("DoD", "the verdict carries no evidence tag")

    # ---------------- DoD: repository-level checks ----------------------
    if OVERRIDDEN:
        print("NOTE DoD: document paths overridden; repository-level checks "
              "skipped")
    else:
        try:
            design = subprocess.run(
                ["python3", "doc/kickoff/check-design.py", "doc/design"],
                capture_output=True, text=True, timeout=300,
            )
            out = (design.stdout + design.stderr).strip().splitlines()
            last = out[-1].strip() if out else ""
            if design.returncode != 0 or last != "OK":
                tail = "\n".join(out[-8:])
                bad("DoD", "the design checker does not print OK:\n%s" % tail)
            else:
                ok("DoD", "the design checker prints OK")
        except (OSError, subprocess.TimeoutExpired) as error:
            bad("DoD", "could not run the design checker: %s" % error)
        if os.path.exists("checks/004-01.sh"):
            try:
                prev = subprocess.run(
                    ["bash", "checks/004-01.sh"],
                    capture_output=True, text=True, timeout=600,
                )
                if prev.returncode != 0:
                    tail = "\n".join(
                        (prev.stdout + prev.stderr).strip().splitlines()[-8:])
                    bad("DoD", "checks/004-01.sh no longer passes:\n%s" % tail)
                else:
                    ok("DoD", "checks/004-01.sh still passes")
            except (OSError, subprocess.TimeoutExpired) as error:
                bad("DoD", "could not run checks/004-01.sh: %s" % error)

    report()


def report():
    print("004-07: timeline epoch wrap analysis (C-10, 16 bits)")
    print("  spec: doc/design/spec/wire.md")
    if not results:
        print("  OK: all checks passed")
        sys.exit(0)

    order = ["AC1", "AC2", "AC3", "DoD"]
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
