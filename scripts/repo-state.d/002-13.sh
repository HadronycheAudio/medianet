# 002-13 — sourced by scripts/repo-state-checks.sh (pass/bad there).
# From checks/002-13.sh, AC 7 ("Frozen-tree fingerprint identical to the
# 002-01 value"): the reads of the frozen proof of concept, the script's own
# code (tag_fingerprint, hash_for and the manifest loop, unchanged):
#   - the poc-untouched tag-tree fingerprint recomputed from the proof of
#     concept's git tag (ls-tree, cat-file; rev-parse for the tag) matches
#     the value recorded in REPORT-0b's Environment section; a report that
#     records none, or a tag that cannot be read, fails;
#   - checks/002-01-frozen-data.sha256: present, every entry under
#     experiments/{pilot,run1,run2,run3}/, each file present with its
#     recorded sha256, 27 entries, no file added or missing.
# Added here: the recomputed fingerprint equals the port's constant
# POC_TAG_TREE_FINGERPRINT (hadronyche-tools/src/doccheck/checks/c002_13/
# mod.rs), against which the Rust test compares the report's value.
# The rest of the script (the report's side of AC 7 included) is the Rust
# test hadronyche-tools/tests/doc_002_13.rs. The proof of concept is read
# without optional locks and never written.
r13_out=$(GIT_OPTIONAL_LOCKS=0 REPORT=doc/design/reports/archaeology/REPORT-0b.md \
    POC_ROOT=/home/dorian/workspace/hadropoc MANIFEST=checks/002-01-frozen-data.sha256 \
    PORT_SRC=hadronyche-tools/src/doccheck/checks/c002_13/mod.rs python3 - <<'PY'
import os, re, subprocess, hashlib

E = os.environ
POC, MANIFEST = E['POC_ROOT'], E['MANIFEST']


def bad(msg):
    print('FAIL ' + msg)


def ok(msg):
    print('PASS ' + msg)


def read_lines(path):
    with open(path, encoding='utf-8') as fh:
        return fh.read().splitlines()


rep = read_lines(E['REPORT'])


def hash_for(pattern):
    rx = re.compile(pattern, re.I)
    for i, l in enumerate(rep):
        if rx.search(l):
            m = re.search(r'([0-9a-fA-F]{64})', l)
            if m:
                return m.group(1)
            if i + 1 < len(rep):
                m = re.search(r'([0-9a-fA-F]{64})', rep[i + 1])
                if m:
                    return m.group(1)
            return None
    return None


def tag_fingerprint(root, tag):
    res = subprocess.run(['git', '-C', root, 'ls-tree', '-r', tag],
                         capture_output=True, text=True)
    if res.returncode != 0:
        return None
    items = []
    for line in res.stdout.splitlines():
        meta, path = line.split('\t', 1)
        parts = meta.split()
        if parts[1] != 'blob':
            continue
        if re.search(r'(^|/)(target|build|\.git)(/|$)', path):
            continue
        if re.search(r'(^|/)\.gitignore$', path):
            continue
        if path.startswith(('src/', 'tools/', 'experiments/')) or \
           path in ('Cargo.toml', 'Cargo.lock', 'README.md'):
            items.append('%s\t%s' % (path, parts[2]))
    h = hashlib.sha256()
    for item in sorted(items):
        path, oid = item.split('\t')
        blob = subprocess.run(['git', '-C', root, 'cat-file', 'blob', oid],
                              capture_output=True).stdout
        h.update(b'path:' + path.encode() + b'\0' + blob + b'\0')
    return h.hexdigest()


recorded = hash_for(r'poc-untouched tag-tree fingerprint')
tag_ok = subprocess.run(['git', '-C', POC, 'rev-parse', '--verify', '--quiet',
                         'refs/tags/poc-untouched^{commit}'],
                        capture_output=True).returncode == 0
computed = tag_fingerprint(POC, 'poc-untouched') if tag_ok else None
if not recorded:
    bad('AC7: the Environment section records no 64-hex poc-untouched tag-tree fingerprint')
elif not computed:
    bad('AC7: cannot recompute the poc-untouched tag-tree fingerprint in %s' % POC)
elif computed.lower() != recorded.lower():
    bad('AC7: frozen-tree fingerprint changed (recorded=%s computed=%s)' % (recorded, computed))
else:
    ok('AC7: poc-untouched tag-tree fingerprint recomputed from git matches the recorded value')

m = re.search(r'POC_TAG_TREE_FINGERPRINT: &str =\s*"([0-9a-f]{64})"', open(E['PORT_SRC']).read())
if computed and (not m or m.group(1) != computed):
    bad('AC7: the port constant POC_TAG_TREE_FINGERPRINT (%s) is not the recomputed fingerprint %s'
        % (m.group(1) if m else 'missing', computed))
elif computed:
    ok('AC7: the port constant POC_TAG_TREE_FINGERPRINT is the recomputed fingerprint')

entries = []
try:
    for l in read_lines(MANIFEST):
        if not l.strip() or l.startswith('#'):
            continue
        f = l.split()
        entries.append((f[0], f[1]))
except OSError:
    bad('AC7: frozen-data manifest missing: checks/002-01-frozen-data.sha256')
if entries:
    mism = 0
    allowed = ('experiments/pilot/', 'experiments/run1/', 'experiments/run2/', 'experiments/run3/')
    for exp, path in entries:
        if not path.startswith(allowed):
            bad('AC7: manifest entry outside the frozen experiment-data dirs: %s' % path)
            mism += 1
            continue
        fp = os.path.join(POC, path)
        if not os.path.isfile(fp):
            bad('AC7: frozen experiment data missing: %s' % path)
            mism += 1
            continue
        with open(fp, 'rb') as fh:
            actual = hashlib.sha256(fh.read()).hexdigest()
        if actual != exp:
            bad('AC7: frozen experiment data changed: %s' % path)
            mism += 1
    present = []
    for d in ('experiments/pilot', 'experiments/run1', 'experiments/run2', 'experiments/run3'):
        for root, _, files in os.walk(os.path.join(POC, d)):
            for fn in files:
                present.append(os.path.relpath(os.path.join(root, fn), POC))
    if len(entries) != 27:
        bad('AC7: manifest has %d entries, expected 27' % len(entries))
    if sorted(p for _, p in entries) != sorted(present):
        bad('AC7: frozen experiment-data files added or missing under experiments/')
    elif mism == 0 and len(entries) == 27:
        ok('AC7: 27 frozen untracked experiment-data files match checks/002-01-frozen-data.sha256')
PY
)
r13_rc=$?
if [ "$r13_rc" -ne 0 ]; then
    bad "002-13 AC7" "the proof-of-concept reads did not run (python3 exit $r13_rc): $(tail_of "$r13_out")"
fi
while IFS= read -r r13_line; do
    case "$r13_line" in
        "PASS "*) r13_rest=${r13_line#PASS }; pass "002-13 ${r13_rest%%: *}" "${r13_rest#*: }" ;;
        "FAIL "*) r13_rest=${r13_line#FAIL }; bad "002-13 ${r13_rest%%: *}" "${r13_rest#*: }" ;;
    esac
done <<EOF
$r13_out
EOF
