#!/usr/bin/env bash
# 004-22 — traceability table and review against the architecture specification.
#
# RED-phase document check. It validates the Traceability section of
# doc/design/spec/wire.md, the per-receiver field audit and the review record
# against the ticket's acceptance criteria. Every list it needs (the rule
# numbers, the constants) is read at run time from the ticket, the field set
# from wire.toml, the heading anchors from wire.md and 01-architecture.md.
#
# Overrides (self-testing on fixtures only):
#   WIRE_MD      the specification (default doc/design/spec/wire.md)
#   WIRE_TOML    the machine-readable tables (default doc/design/spec/wire.toml)
#   ARCH_MD      the architecture specification (default doc/design/01-architecture.md)
#   REPORTS_DIR  where a separate review record may live (default doc/design/reports)
#   TICKET_MD    the ticket (default doc/design/tickets/004/004-22-*.md)
# When any of WIRE_MD, WIRE_TOML, ARCH_MD, REPORTS_DIR or TICKET_MD is
# overridden, the repository-level cargo runs are skipped with a NOTE.
#
# What is checked, per acceptance criterion:
#
#   AC1  The Traceability section has a table with one row per rule 1..22
#        (first cell "Rule N" or "N"; each number once). Each row's mapping
#        cell (every cell after the first) is non-empty and holds at least
#        one reference that resolves: a heading of wire.md (plain name,
#        "Section, Subsection", or a markdown link to an anchor). Every
#        markdown anchor in the section resolves. A backticked snake_case
#        field name must be a field of wire.toml.
#   AC2  Every constant of the ticket's "Constants traced:" list (ranges
#        expanded, read from the ticket) is named by a row of the section
#        whose mapping resolves. A constant the ticket marks wire-visible
#        (the ids after "wire-visible rulings") needs a row that maps it to a
#        backticked wire.toml field and states a unit (a unit word in the
#        row, in the field's wire.toml text, or "unit: none (reason)").
#        A row that says a constant is NOT wire-visible gives a reason of at
#        least three words.
#   AC3  A section headed "audit" lists every field of the media-layer
#        messages of wire.toml (the media packet, its announcement and slice
#        checksum sections, the content-message header, the four content
#        messages and the sparse keyed entry), each with a verdict. A row with
#        a destination-specific or per-receiver verdict STOPS the ticket: it
#        fails with the stop message, and its field must also appear in the
#        review record's discrepancies. The audit states that no
#        destination-specific or per-receiver field exists.
#   AC4  A review record (a wire.md section headed "Review record", or a file
#        under REPORTS_DIR linked from the Traceability section) states the
#        reviewer, a date, the protocol and packetisation heading of
#        01-architecture.md (it must exist), a discrepancy list whose every row
#        has a resolution (or an explicit "no discrepancy"), a resolution that
#        changes another design document names 004-23 or an owner, and that the
#        traceability table is attached (a link to the Traceability anchor or
#        the table itself).
#   DoD  Every paragraph of 5+ words in the new sections carries an evidence
#        tag; open items name an owner and a falsifier; no `main` ref; the
#        record's links resolve. When not overridden: `cargo run --bin
#        wire-check` exits 0 and `cargo test -p hadronyche-tools --test
#        wire_tables` reports at least 12 passed and 0 failed (today's count).
set -u
cd "$(dirname "$0")/.."

WIRE="${WIRE_MD:-doc/design/spec/wire.md}"
TABLES="${WIRE_TOML:-doc/design/spec/wire.toml}"
ARCH="${ARCH_MD:-doc/design/01-architecture.md}"
REPORTS="${REPORTS_DIR:-doc/design/reports}"
TICKET="${TICKET_MD:-}"
if [ -z "$TICKET" ]; then
    TICKET=$(ls doc/design/tickets/004/004-22-*.md 2>/dev/null | head -n 1)
fi

PY=""
for candidate in python3 python3.13 python3.12 python3.11; do
    if "$candidate" -c 'import tomllib' >/dev/null 2>&1; then
        PY="$candidate"
        break
    fi
done
if [ -z "$PY" ]; then
    echo "FAIL WIRE: no python with tomllib (3.11 or later) on PATH"
    exit 1
fi

overridden=0
if [ -n "${WIRE_MD:-}" ] || [ -n "${WIRE_TOML:-}" ] || [ -n "${ARCH_MD:-}" ] || [ -n "${REPORTS_DIR:-}" ] || [ -n "${TICKET_MD:-}" ]; then
    overridden=1
fi

"$PY" - "$WIRE" "$TABLES" "$ARCH" "$REPORTS" "$TICKET" "$overridden" <<'PY'
import os
import re
import subprocess
import sys

wire_path, tables_path, arch_path, reports_dir, ticket_path, overridden = sys.argv[1:7]
overridden = overridden == "1"
import tomllib

fails = []  # (label, message)


def fail(label, msg):
    fails.append((label, msg))


def info(msg):
    print(f"INFO: {msg}")


# ---------------------------------------------------------------- inputs
for path, what in ((wire_path, "wire.md"), (tables_path, "wire.toml"),
                   (arch_path, "01-architecture.md"), (ticket_path, "the ticket")):
    if not path or not os.path.isfile(path):
        print(f"FAIL WIRE: cannot read {what} at {path!r}")
        sys.exit(1)

with open(tables_path, "rb") as fh:
    toml = tomllib.load(fh)

# Known field names: every [[message.field]] and [[reserved]] name, with its
# width and its text (for the unit test of a field).
FIELD_INFO = {}
for msg in toml.get("message", []):
    for f in msg.get("field", []):
        FIELD_INFO.setdefault(f["name"], []).append(f)
for r in toml.get("reserved", []):
    FIELD_INFO.setdefault(r["name"], []).append(r)
KNOWN = set(FIELD_INFO)


def field_text(name):
    out = []
    for f in FIELD_INFO.get(name, []):
        for key in ("meaning", "valid", "reserved", "outcome", "value"):
            if key in f:
                out.append(str(f[key]))
    return " ".join(out)


# ---------------------------------------------------------------- markdown
def norm(t):
    t = re.sub(r"[`*_]", "", t)
    t = re.sub(r"\s+", " ", t).strip().lower()
    return t.rstrip(".")


def slugify(t):
    t = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", t)
    t = re.sub(r"[`*~]", "", t).strip().lower()
    t = re.sub(r"[^\w\- ]", "", t)
    return t.replace(" ", "-")


class Doc:
    def __init__(self, path):
        with open(path, encoding="utf-8") as fh:
            self.text = fh.read()
        self.lines = self.text.split("\n")
        self.in_code = []
        fence = False
        for line in self.lines:
            if line.strip().startswith("```"):
                fence = not fence
                self.in_code.append(True)
            else:
                self.in_code.append(fence)
        self.heads = []
        seen = {}
        for i, line in enumerate(self.lines):
            if self.in_code[i]:
                continue
            m = re.match(r"^(#{1,6})\s+(.*?)\s*#*\s*$", line)
            if not m:
                continue
            text = m.group(2).strip()
            slug = slugify(text)
            if slug in seen:
                seen[slug] += 1
                slug = f"{slug}-{seen[slug]}"
            else:
                seen[slug] = 0
            self.heads.append({"idx": i, "level": len(m.group(1)), "text": text, "slug": slug})
        self.slugs = {h["slug"] for h in self.heads}
        self.head_norms = set()
        for h in self.heads:
            self.head_norms.add(norm(h["text"]))
            self.head_norms.add(norm(re.sub(r"^\d+(\.\d+)*\.?\s+", "", h["text"])))

    def section_range(self, h):
        end = len(self.lines)
        for later in self.heads:
            if later["idx"] > h["idx"] and later["level"] <= h["level"]:
                end = later["idx"]
                break
        return h["idx"], end

    def find_heads(self, pattern):
        return [h for h in self.heads if re.search(pattern, h["text"], re.I)]


def split_row(line):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    return [c.strip() for c in s.split("|")]


def tables_in(doc, lo, hi):
    out = []
    i = lo
    while i < hi:
        if doc.in_code[i] or not doc.lines[i].strip().startswith("|"):
            i += 1
            continue
        j = i
        group = []
        while j < hi and not doc.in_code[j] and doc.lines[j].strip().startswith("|"):
            group.append(split_row(doc.lines[j]))
            j += 1
        if len(group) > 1 and group[1] and all(re.fullmatch(r":?-+:?", c) for c in group[1] if c != "") \
                and any(group[1]):
            out.append({"header": [norm(c) for c in group[0]], "rows": group[2:], "line": i + 1})
        else:
            out.append({"header": None, "rows": group, "line": i + 1})
        i = j
    return out


def strip_links(text):
    return re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)


_file_cache = {}


def file_slugs(path):
    if path not in _file_cache:
        _file_cache[path] = Doc(path).slugs if os.path.isfile(path) else None
    return _file_cache[path]


def check_links(text, base_dir, self_doc, self_path, label, where):
    """Return (resolved_count, [dangling messages])."""
    resolved = 0
    dangling = []
    for m in re.finditer(r"\[([^\]]*)\]\(([^)\s]*)\)", text):
        target = m.group(2)
        if target.startswith("http"):
            continue
        if "#" in target:
            path, anchor = target.split("#", 1)
        else:
            path, anchor = target, None
        if path == "":
            slugs = self_doc.slugs
            target_path = self_path
        else:
            target_path = os.path.normpath(os.path.join(base_dir, path))
            if not os.path.isfile(target_path):
                dangling.append(f"{where}: link [{m.group(1)}]({target}) names a file that does not exist")
                continue
            slugs = file_slugs(target_path)
        if anchor is not None and slugs is not None and anchor not in slugs:
            dangling.append(f"{where}: link [{m.group(1)}]({target}) names an anchor that is not a heading of {os.path.relpath(target_path)}")
            continue
        resolved += 1
    return resolved, dangling


def reference_resolves(cell_text, doc):
    """True when a plain heading name, 'Section, Subsection' or a backticked heading resolves."""
    text = strip_links(cell_text)
    segs = []
    for g in re.findall(r"\(([^()]*)\)", text):
        segs += g.split(";")
    segs += re.findall(r"`([^`]+)`", text)
    segs += text.split(";")
    for seg in segs:
        s = seg.strip().strip(".,")
        if not s:
            continue
        if norm(s) in doc.head_norms:
            return True
        if "," in s:
            parts = [p.strip() for p in s.split(",")]
            if len(parts) >= 2 and all(norm(p) in doc.head_norms for p in parts):
                return True
    return False


def expand_ids(text):
    ids = set()
    for m in re.finditer(r"C-(\d{1,3})(?:\s*(?:to|–)\s*C-(\d{1,3}))?", text):
        a = int(m.group(1))
        b = int(m.group(2)) if m.group(2) else a
        if b >= a:
            ids.update(range(a, b + 1))
    return ids


UNIT_RE = re.compile(r"\b(bytes?|bits?|ns|µs|us|ms|samples?|frames?|packets?|slices?|Hz|count|code|id|flag)\b")
UNIT_NONE_RE = re.compile(r"unit:\s*none\s*\(")
TAG_RE = re.compile(r"\[(decision|code|measured|kernel-doc|standard|memory)\]")
VERDICT_OK = re.compile(r"^(none|no|clear|pass|absent|ok|not (destination|per)|none found)\b", re.I)
VERDICT_BAD = re.compile(r"^(yes,?\s*)?(destination[- ]specific|per[- ]receiver)( field)?\.?$", re.I)
RULE_RE = re.compile(r"^\**\s*(?:rule\s*)?(\d{1,2})\s*\**$", re.I)
ISO_DATE = re.compile(r"\b20\d\d-\d\d-\d\d\b")
MONTH_DATE = re.compile(r"\b\d{1,2}\s+(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*\.?,?\s+20\d\d\b", re.I)
TRACE_ANCHOR_TEXT = "traceability"

# ---------------------------------------------------------------- ticket
with open(ticket_path, encoding="utf-8") as fh:
    ticket_text = fh.read()
m = re.search(r"Constants traced:(.*)", ticket_text)
if not m:
    print("FAIL AC2: the ticket has no 'Constants traced:' sentence")
    sys.exit(1)
sentence = m.group(1).split("\n")[0]
wv_at = sentence.find("wire-visible rulings")
general_part = sentence if wv_at < 0 else sentence[:wv_at]
visible_part = "" if wv_at < 0 else sentence[wv_at:]
LIST_IDS = expand_ids(sentence)
VISIBLE_IDS = expand_ids(visible_part)
info(f"constants read from the ticket: {len(LIST_IDS)} ids: " + ", ".join(f"C-{i:02d}" if i < 100 else f"C-{i}" for i in sorted(LIST_IDS)))
info(f"wire-visible (ticket marks them after 'wire-visible rulings'): {len(VISIBLE_IDS)} ids")

# ---------------------------------------------------------------- wire.md
W = Doc(wire_path)
trace_heads = [h for h in W.heads if norm(h["text"]).startswith(TRACE_ANCHOR_TEXT)]
trace_heads.sort(key=lambda h: h["level"])
if not trace_heads:
    fail("AC1", f"{wire_path} has no Traceability section (a heading starting 'Traceability')")
    trace_lo, trace_hi = None, None
    trace = None
else:
    trace = trace_heads[0]
    trace_lo, trace_hi = W.section_range(trace)
    trace_slug = trace["slug"]
    info(f"Traceability section: heading '{trace['text']}' at line {trace['idx'] + 1}")

trace_rows = []  # (table, row) within the section
if trace is not None:
    for t in tables_in(W, trace_lo, trace_hi):
        for row in t["rows"]:
            trace_rows.append(row)

# AC1 rule rows
rule_seen = {}
for row in trace_rows:
    if not row:
        continue
    rm = RULE_RE.match(row[0].strip())
    if rm:
        n = int(rm.group(1))
        rule_seen.setdefault(n, []).append(row)

RULES = list(range(1, 23))  # the ticket's rules 1..22 (Context, numbered rules)
if trace is not None:
    missing = [n for n in RULES if n not in rule_seen]
    dup = [n for n in RULES if len(rule_seen.get(n, [])) > 1]
    if missing:
        fail("AC1", f"the Traceability table has no row for rule(s) {missing} of the ticket's 1..22 (rows are identified by a 'Rule N' or 'N' first cell)")
    if dup:
        fail("AC1", f"rule(s) {dup} have more than one row in the Traceability table")
    for n in RULES:
        rows = rule_seen.get(n, [])
        if not rows:
            continue
        row = rows[0]
        mapping = " | ".join(row[1:]).strip()
        if not mapping or not re.sub(r"[\s|\-]", "", mapping):
            fail("AC1", f"rule {n}: the mapping cell is empty")
            continue
        if not reference_resolves(mapping, W):
            fail("AC1", f"rule {n}: no reference in its mapping cell resolves to a heading of wire.md ('{mapping[:90]}')")

# Dangling links and unknown field citations in the whole trace section
if trace is not None:
    for row in trace_rows:
        text = " | ".join(row)
        _, dang = check_links(text, os.path.dirname(wire_path), W, wire_path, "AC1", "Traceability row")
        for d in dang:
            fail("AC1", d)
        for tok in re.findall(r"`([^`]+)`", text):
            if re.fullmatch(r"[a-z][a-z0-9]*(_[a-z0-9]+)+", tok) and tok not in KNOWN:
                fail("AC1", f"a Traceability row cites `{tok}`, which is not a field of wire.toml")

# AC2 constants
const_rows = {}
if trace is not None:
    for row in trace_rows:
        if not row:
            continue
        ids = expand_ids(" | ".join(row))
        for i in ids:
            const_rows.setdefault(i, []).append(row)

def row_mapping(row):
    return " | ".join(row[1:]).strip()

def row_fields(row):
    names = []
    for tok in re.findall(r"`([^`]+)`", " | ".join(row)):
        if tok in KNOWN:
            names.append(tok)
    return names

def row_unit_ok(row, fields):
    text = " | ".join(row)
    if UNIT_RE.search(text) or UNIT_NONE_RE.search(text):
        return True
    for f in fields:
        if UNIT_RE.search(field_text(f)):
            return True
    return False

def cid(i):
    return f"C-{i:02d}" if i < 100 else f"C-{i}"

for i in sorted(LIST_IDS):
    rows = const_rows.get(i, [])
    if not rows:
        fail("AC2", f"constant {cid(i)} of the ticket's list has no row in the Traceability section")
        continue
    resolved_rows = [r for r in rows if reference_resolves(row_mapping(r), W)]
    if not resolved_rows:
        fail("AC2", f"constant {cid(i)}: no row naming it has a mapping cell that resolves to a heading of wire.md")
        continue
    if i in VISIBLE_IDS:
        ok = False
        for r in resolved_rows:
            fields = row_fields(r)
            if fields and row_unit_ok(r, fields):
                ok = True
                break
        if not ok:
            fail("AC2", f"wire-visible constant {cid(i)} has no row mapping it to a wire.toml field (backticked name) with a width and a unit")

# explicit not-wire-visible reasons
if trace is not None:
    for row in trace_rows:
        text = " | ".join(row)
        mm = re.search(r"not wire[- ]visible|not on the wire", text, re.I)
        if mm and expand_ids(text):
            rest = re.sub(r"^[\s:;,\-–—(]+", "", text[mm.end():])
            if len(rest.split()) < 3:
                fail("AC2", f"a row says {cid(sorted(expand_ids(text))[0])} is not wire-visible without a reason of at least three words")

# AC3 audit
MEDIA_MSGS = [m for m in toml.get("message", [])
              if m["name"] == "media packet" or m.get("section_of") == "media packet"
              or re.search(r"content|audio message|dense controller|sparse keyed|transport state", m["name"])]
EXPECTED = []
for m in MEDIA_MSGS:
    for f in m.get("field", []):
        if f["name"] not in EXPECTED:
            EXPECTED.append(f["name"])
info("media-layer field set (wire.toml messages -> fields): " + "; ".join(
    f"{m['name']} -> " + ",".join(f["name"] for f in m.get("field", [])) for m in MEDIA_MSGS))
info(f"media-layer distinct field names: {len(EXPECTED)}")

audit_heads = [h for h in W.heads if re.search(r"audit", h["text"], re.I)]
audit_ranges = [W.section_range(h) for h in audit_heads]
audit_text = "\n".join("\n".join(W.lines[lo:hi]) for lo, hi in audit_ranges)
audit_rows = []
if not audit_heads:
    fail("AC3", f"{wire_path} has no section headed 'audit' (the per-receiver field audit of Work 2)")
else:
    for lo, hi in audit_ranges:
        for t in tables_in(W, lo, hi):
            audit_rows.extend(t["rows"])

def cell_plain(c):
    return re.sub(r"[`*]", "", c).strip().lower()

bad_fields = []
missing_fields = []
for name in EXPECTED:
    found = False
    for row in audit_rows:
        for c in row:
            if re.search(r"`%s`" % re.escape(name), c) or cell_plain(c) == name:
                found = True
    if not found:
        missing_fields.append(name)
if audit_heads and missing_fields:
    fail("AC3", f"the audit lists no row for media field(s): {', '.join(missing_fields)}")

for row in audit_rows:
    names = [n for n in EXPECTED if any(re.search(r"`%s`" % re.escape(n), c) or cell_plain(c) == n for c in row)]
    if not names:
        continue
    cells_plain = [cell_plain(c) for c in row]
    if any(VERDICT_BAD.match(c) for c in cells_plain):
        bad_fields.extend(names)
        continue
    if not any(VERDICT_OK.match(c) for c in cells_plain):
        fail("AC3", f"the audit row for {', '.join(names)} states no verdict (none / no destination-specific or per-receiver content)")

if bad_fields:
    fail("AC3", "STOP: the audit finds a destination-specific or per-receiver media field "
                f"({', '.join(sorted(set(bad_fields)))}); epic AC-5 is violated: the ticket stops here, "
                "the field is a discrepancy to record and escalate, and no audit may pass with it")

if audit_heads and not re.search(
        r"no\s+destination[- ]specific\s+(?:or|and|nor)\s+(?:no\s+)?(?:a\s+|any\s+)?per[- ]receiver\s+field"
        r"|no\s+per[- ]receiver\s+(?:or|and|nor)\s+(?:no\s+)?(?:a\s+|any\s+)?destination[- ]specific\s+field",
        audit_text, re.I):
    fail("AC3", "the audit's conclusion does not state that no destination-specific or per-receiver field exists")

# ---------------------------------------------------------------- review record
record_text = None
record_path = None
record_lo = None
trace_links_to_file = []
if trace is not None:
    for mm in re.finditer(r"\[[^\]]*\]\(([^)\s#]+)(?:#[^)\s]*)?\)", "\n".join(W.lines[trace_lo:trace_hi])):
        target = mm.group(1)
        if target.startswith("http"):
            continue
        p = os.path.normpath(os.path.join(os.path.dirname(wire_path), target))
        if os.path.isfile(p) and os.path.realpath(p).startswith(os.path.realpath(reports_dir) + os.sep):
            trace_links_to_file.append(p)
if trace_links_to_file:
    record_path = trace_links_to_file[0]
    with open(record_path, encoding="utf-8") as fh:
        record_text = fh.read()
    info(f"review record: separate file {record_path}")
else:
    rec_heads = [h for h in W.heads if re.match(r"^\W*review\b", h["text"], re.I) or re.search(r"review record", h["text"], re.I)]
    if rec_heads:
        lo, hi = W.section_range(rec_heads[0])
        record_text = "\n".join(W.lines[lo:hi])
        record_lo = (lo, hi)
        record_path = wire_path
        info(f"review record: section '{rec_heads[0]['text']}' of wire.md, line {rec_heads[0]['idx'] + 1}")

if record_text is None:
    fail("AC4", "no review record: neither a wire.md section headed 'Review record' nor a file under the reports directory linked from the Traceability section")
    record_text = ""

RT = record_text
if record_text:
    rec_base = os.path.dirname(record_path)

    # reviewer
    reviewer_ok = False
    for line in RT.split("\n"):
        mm = re.search(r"reviewer", line, re.I)
        if mm:
            after = re.sub(r"[*_|:`]", " ", line[mm.end():]).strip()
            if len(after) >= 2 and not re.match(r"(tbd|todo|unknown|\?)", after, re.I):
                reviewer_ok = True
    if not reviewer_ok:
        fail("AC4", "the review record names no reviewer (a line 'Reviewer: <name>' with a name)")

    # date
    date_ok = False
    for line in RT.split("\n"):
        if re.search(r"\bdate\b|reviewed on|reviewed:", line, re.I) and (ISO_DATE.search(line) or MONTH_DATE.search(line)):
            date_ok = True
    if not date_ok:
        fail("AC4", "the review record states no date (a 'Date:' line with an ISO date such as 2026-11-16)")

    # architecture heading
    arch = Doc(arch_path)
    arch_cands = [h for h in arch.heads if "protocol" in norm(h["text"]) and "packetis" in norm(h["text"])]
    if not arch_cands:
        fail("AC4", f"{arch_path} has no heading on protocol and packetisation (cannot review against it)")
    else:
        rec_norm = norm(RT)
        named = []
        for h in arch_cands:
            title = norm(re.sub(r"^\d+(\.\d+)*\.?\s+", "", h["text"]))
            if title in rec_norm:
                named.append(h["text"])
        if not named:
            fail("AC4", "the review record names no architecture heading on protocol and packetisation that exists in "
                        f"01-architecture.md (candidates: {', '.join(h['text'] for h in arch_cands)})")
        else:
            info(f"architecture section reviewed: '{named[0]}' ({os.path.relpath(arch_path)}, line {arch_cands[0]['idx'] + 1})")

    # discrepancies
    rec_lines = RT.split("\n")
    disc_idx = None
    for i, line in enumerate(rec_lines):
        if re.search(r"discrepanc", line, re.I):
            disc_idx = i
            break
    disc_body = ""
    if disc_idx is None:
        fail("AC4", "the review record has no discrepancy list (a 'Discrepancies' heading or table)")
    else:
        j = disc_idx + 1
        while j < len(rec_lines) and not re.match(r"^#{1,6}\s", rec_lines[j]):
            j += 1
        disc_body = "\n".join(rec_lines[disc_idx:j])
        if re.search(r"no discrepanc|none found", disc_body, re.I):
            pass
        else:
            tables = []
            k = disc_idx + 1
            while k < j:
                if rec_lines[k].strip().startswith("|"):
                    grp_lo = k
                    while k < j and rec_lines[k].strip().startswith("|"):
                        k += 1
                    tables.append("\n".join(rec_lines[grp_lo:k]))
                else:
                    k += 1
            list_items = [l for l in rec_lines[disc_idx + 1:j] if re.match(r"^\s*([-*]|\d+\.)\s+", l)]
            if not tables and not list_items:
                fail("AC4", "the discrepancy list has no rows, items or explicit 'no discrepancy' statement")
            for tbl in tables:
                grp = [split_row(l) for l in tbl.split("\n")]
                header = [norm(c) for c in grp[0]]
                if "resolution" not in header:
                    fail("AC4", "a discrepancy table has no 'Resolution' column")
                    continue
                ridx = header.index("resolution")
                data = [r for r in grp[1:] if not all(re.fullmatch(r":?-+:?", c) for c in r if c)]
                if not data:
                    fail("AC4", "a discrepancy table has no discrepancy row and no explicit 'no discrepancy' statement")
                for r in data:
                    res = r[ridx] if ridx < len(r) else ""
                    if not res or res in ("-", "tbd", "TBD", "todo"):
                        fail("AC4", f"a discrepancy has no resolution: '{' | '.join(r)[:100]}'")
                        continue
                    if re.search(r"\b0\d-[a-z-]+\.md\b|doc/design/(?!spec/wire)", res) and not (
                            "004-23" in " ".join(r) or re.search(r"owner", " ".join(r), re.I)):
                        fail("AC4", f"a resolution changes a design document other than wire.md without naming 004-23 or an owner: '{res[:100]}'")
            for li in list_items:
                if not re.search(r"resolution\s*[:→—–-]\s*\S", li, re.I):
                    fail("AC4", f"a discrepancy list item has no resolution: '{li.strip()[:100]}'")
                    continue
                if re.search(r"\b0\d-[a-z-]+\.md\b|doc/design/(?!spec/wire)", li) and not (
                        "004-23" in li or re.search(r"owner", li, re.I)):
                    fail("AC4", f"a resolution changes a design document other than wire.md without naming 004-23 or an owner: '{li.strip()[:100]}'")

    # attached traceability table
    attached = False
    if trace is not None:
        for mm in re.finditer(r"\]\(([^)\s]*)#([^)\s]+)\)", RT):
            target_file, anchor = mm.group(1), mm.group(2)
            if anchor == trace["slug"] or anchor == trace_slug:
                attached = True
    rule_hits = set()
    for line in RT.split("\n"):
        if line.strip().startswith("|"):
            cells = split_row(line)
            if cells and RULE_RE.match(cells[0].strip()):
                rule_hits.add(int(RULE_RE.match(cells[0].strip()).group(1)))
    if set(RULES) <= rule_hits:
        attached = True
    if not attached:
        fail("AC4", "the traceability table is not attached to the review record (no link to the Traceability anchor and no 22-row rule table)")

    # links in the record resolve
    if record_lo is None:
        _, dang = check_links(RT, rec_base, W, wire_path, "AC4", "review record")
    else:
        _, dang = check_links(RT, os.path.dirname(wire_path), W, wire_path, "AC4", "review record")
    for d in dang:
        fail("AC4", d)

    # discrepancy rows naming a bad doc stated above; audit bad fields must be discrepancies
    if bad_fields:
        for name in sorted(set(bad_fields)):
            if name not in disc_body:
                fail("AC3", f"destination-specific or per-receiver field {name} is not recorded as a discrepancy in the review record")

# ---------------------------------------------------------------- DoD scopes
scopes = []  # (label, text)
if trace is not None:
    scopes.append(("Traceability", "\n".join(W.lines[trace_lo:trace_hi])))
if audit_heads:
    scopes.append(("audit", audit_text))
if record_text:
    scopes.append(("review record", RT))

for label, text in scopes:
    if re.search(r"`(origin/)?main`|origin/main|\bmain branch\b", text):
        fail("DoD", f"the {label} text refers to a main ref")
    lines = text.split("\n")
    blocks = []
    cur = []
    in_code = False
    for line in lines + [""]:
        if line.strip().startswith("```"):
            in_code = not in_code
            continue
        if in_code:
            continue
        if line.strip() == "":
            if cur:
                blocks.append(cur)
                cur = []
            continue
        if line.strip().startswith("|") or line.strip().startswith("#"):
            if cur:
                blocks.append(cur)
                cur = []
            continue
        if re.match(r"^\s*([-*]|\d+\.)\s+", line) and cur:
            blocks.append(cur)
            cur = []
        cur.append(line)
    if cur:
        blocks.append(cur)
    for b in blocks:
        joined = " ".join(l.strip() for l in b)
        words = len(re.findall(r"\S+", strip_links(joined)))
        if words >= 5 and not TAG_RE.search(joined):
            fail("DoD", f"an untagged paragraph of 5+ words in the {label}: '{joined[:80]}'")
        if re.search(r"open item", joined, re.I):
            if not (re.search(r"owner", joined, re.I) and re.search(r"falsifier", joined, re.I)):
                fail("DoD", f"an open item in the {label} names no owner or no falsifier: '{joined[:80]}'")

# ---------------------------------------------------------------- repository-level
if overridden:
    print("NOTE: fixture overrides set; cargo wire-check and wire_tables are skipped (they target the real repository)")
else:
    try:
        proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                              capture_output=True, text=True, timeout=900)
        if proc.returncode != 0:
            tail = "\n".join((proc.stdout + proc.stderr).strip().splitlines()[-6:])
            fail("DoD", f"cargo run --bin wire-check did not pass:\n{tail}")
    except (OSError, subprocess.SubprocessError) as error:
        fail("DoD", f"could not run cargo run --bin wire-check: {error}")
    try:
        proc = subprocess.run(["cargo", "test", "-p", "hadronyche-tools", "--test", "wire_tables"],
                              capture_output=True, text=True, timeout=900)
        out = proc.stdout + proc.stderr
        res = re.findall(r"test result: \w+\. (\d+) passed; (\d+) failed", out)
        if not res:
            fail("DoD", "the wire_tables test run printed no 'test result:' line")
        else:
            passed = sum(int(p) for p, _ in res)
            failed = sum(int(f) for _, f in res)
            if failed != 0 or passed < 12:
                fail("DoD", f"wire_tables must report at least 12 passed and 0 failed; got {passed} passed, {failed} failed")
    except (OSError, subprocess.SubprocessError) as error:
        fail("DoD", f"could not run the wire_tables tests: {error}")

# ---------------------------------------------------------------- report
if fails:
    for label, msg in fails:
        print(f"FAIL {label}: {msg}")
    print(f"FAIL: {len(fails)} assertion(s) failed for 004-22")
    sys.exit(1)
print("OK: 4/4 acceptance criteria for 004-22 and the DoD checks pass")
PY
