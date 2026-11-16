//! AC 1 to AC 3: the budget columns, their arithmetic, and the omission
//! rule.

use std::collections::BTreeMap;

use super::comp::{Comp, Consts};
use super::section::{STable, Sec, cell_num, classify_case, row_kind, row_label};
use super::util::{Fails, norm, sentences};
use crate::doccheck::text::{has, has_i, rx};

/// One budget column (`CASES`).
#[derive(Debug, Clone)]
pub(crate) struct Case {
    pub kind: &'static str,
    pub label: String,
    pub total: i64,
    pub spare: i64,
    pub bound: i64,
    pub ann: i64,
    pub ext: i64,
}

fn sum(vals: &BTreeMap<&'static str, Vec<i64>>, k: &str) -> Option<i64> {
    vals.get(k).map(|v| v.iter().sum())
}

/// Checks the part rows of one column against the toml sizes.
fn parts(vals: &BTreeMap<&'static str, Vec<i64>>, label: &str, k: &Comp, c: &Consts, f: &mut Fails) {
    for need in ["pkt", "slice", "ann", "content", "audio", "cksum"] {
        if !vals.contains_key(need) {
            f.add("AC1", format!("budget column {label} has no {need} row"));
        }
    }
    let pins: [(&str, i64, &str); 5] = [
        ("pkt", k.pkt, "packet header"),
        ("slice", k.slice, "slice header"),
        ("content", k.cw, "content-message header"),
        ("audio", c.audio, "audio"),
        ("cksum", k.cksum, "checksum"),
    ];
    for (key, want, what) in pins {
        if let Some(v) = vals.get(key)
            && *v != [want] {
                f.add("AC1", format!("budget column {label}: {what} {v:?} != {want}"));
            }
    }
}

/// One column `c` of a budget table, or `None` when it is skipped.
fn column(t: &STable, c: usize, k: &Comp, cs: &Consts, f: &mut Fails) -> Option<Case> {
    let totals = t.rows.iter().find(|r| row_label(r).starts_with("total"))?;
    let total = cell_num(totals, c)?;
    let ctext = format!("{} {}", t.caption, t.header[c]);
    let label = format!("{:?} column {:?}", t.caption.chars().take(60).collect::<String>(), t.header[c]);
    let Some(kind) = classify_case(&ctext) else {
        f.add("AC1", format!("budget column {label} is not named as default, worst case or fallback"));
        return None;
    };
    let mut vals: BTreeMap<&'static str, Vec<i64>> = BTreeMap::new();
    let (mut spare, mut bad) = (None, false);
    for r in &t.rows {
        if r.len() <= c || row_label(r).is_empty() {
            continue;
        }
        let kd = row_kind(&row_label(r));
        let Some(v) = cell_num(r, c) else {
            f.add("AC1", format!("budget column {label}: row {:?} has no number", r[0]));
            bad = true;
            continue;
        };
        match kd {
            "total" => {}
            "spare" => spare = Some(v),
            "other" => {
                f.add("AC1", format!("budget column {label}: row {:?} is not a recognised budget part", r[0]));
                bad = true;
            }
            _ => vals.entry(kd).or_default().push(v),
        }
    }
    if bad {
        return None;
    }
    let Some(spare) = spare else {
        f.add("AC1", format!("budget column {label} has no spare row"));
        return None;
    };
    let comp_total: i64 = vals.values().flatten().sum();
    if comp_total != total {
        f.add("AC5", format!("budget column {label}: the parts sum to {comp_total}, its total row says {total}"));
    }
    parts(&vals, &label, k, cs, f);
    let ann = vals.get("ann").map_or(0, |v| v.iter().sum());
    let ext = sum(&vals, "ext").unwrap_or(0);
    judge(&Case { kind, label, total, spare, bound: 0, ann, ext }, k, cs, f)
}

/// The bound, spare and kind rules of one column.
fn judge(c: &Case, k: &Comp, cs: &Consts, f: &mut Fails) -> Option<Case> {
    let label = &c.label;
    if !k.allowed_ann().contains(&c.ann) {
        f.add("AC2", format!("budget column {label}: announcement rows sum to {}, which is neither the count byte alone nor the count plus 12 bytes per announcement (up to {})", c.ann, k.ann_max()));
    }
    if c.ext != 0 && c.ext != k.ext {
        f.add("AC2", format!("budget column {label}: extension {} != 0 or {} (toml)", c.ext, k.ext));
    }
    let bound = if c.kind == "fb" { cs.pay_fb } else { cs.pay_def };
    if c.total > bound {
        f.add("AC1", format!("budget column {label}: total {} exceeds the {bound}-byte bound", c.total));
    }
    if c.spare != bound - c.total {
        f.add("AC1", format!("budget column {label}: spare {} != {bound} - {} = {}", c.spare, c.total, bound - c.total));
    }
    if c.kind == "default" {
        if c.ann != k.ann_count || c.ext != 0 {
            f.add("AC1", format!("default column {label} must hold the mandatory parts only (announcement count byte, no extension)"));
        }
        if c.total != k.mandatory() + cs.audio {
            f.add("AC1", format!("default column {label}: total {} != mandatory {} + audio {}", c.total, k.mandatory(), cs.audio));
        }
    }
    if c.kind == "worst" && c.ann != k.ann_max() {
        f.add("AC2", format!("worst column {label}: announcements are not at maximum ({} != {})", c.ann, k.ann_max()));
    }
    if c.kind == "fb" && c.total - cs.audio > cs.cap {
        f.add("AC3", format!("fallback column {label}: headers plus checksum are {} bytes, above the {}-byte cap of C-137", c.total - cs.audio, cs.cap));
    }
    Some(Case { bound, ..c.clone() })
}

/// The budget tables of the section and their columns (`CASES`).
pub(crate) fn cases(sec: &Sec, k: &Comp, cs: &Consts, f: &mut Fails) -> Vec<Case> {
    let mut out = Vec::new();
    for t in &sec.tables {
        let has_total = t.rows.iter().any(|r| row_label(r).starts_with("total"));
        let present: Vec<&str> = t
            .rows
            .iter()
            .map(|r| row_label(r))
            .filter(|l| !l.is_empty())
            .map(|l| row_kind(&l))
            .collect();
        if !has_total || !present.contains(&"spare") || !present.contains(&"audio") {
            continue;
        }
        for c in 1..t.header.len() {
            if let Some(case) = column(t, c, k, cs, f) {
                out.push(case);
            }
        }
    }
    out
}

/// What the columns must add up to as a set (AC 1 to AC 3).
pub(crate) fn coverage(cs: &[Case], k: &Comp, c: &Consts, sec: &Sec, f: &mut Fails) {
    if cs.is_empty() {
        f.add("AC1", "no budget table: a 'Total' row with numeric columns in the Size budget section");
    }
    let has_kind = |kind: &str, ann: i64, ext: i64| cs.iter().any(|x| x.kind == kind && x.ann == ann && x.ext == ext);
    if !cs.iter().any(|x| x.kind == "default") {
        f.add("AC1", format!("no default 1 472 budget column (mandatory parts only: total {})", k.mandatory() + c.audio));
    }
    if !has_kind("worst", k.ann_max(), k.ext) {
        f.add("AC2", format!("no worst-case 1 472 column with all announcements and the extension (total {})", k.max_non_audio() + c.audio));
    }
    if !has_kind("worst", k.ann_max(), 0) {
        f.add("AC2", "no worst-case 1 472 column with all announcements and no extension");
    }
    if !cs.iter().any(|x| x.kind == "fb") {
        f.add("AC3", "no 1 280-byte fallback budget column");
    }
    for x in cs {
        if x.kind != "fb" && x.bound != c.pay_def {
            f.add("AC1", "a 1 472 column has the wrong bound");
        }
    }
    let t = &sec.sec_text;
    for (word, ac) in [("C-100", "AC1"), ("C-06", "AC3"), ("C-137", "AC3")] {
        if !has(&format!(r"\b{}\b", regex::escape(word)), t) {
            f.add(ac, format!("the Size budget section does not cite {word}"));
        }
    }
    if !has(r"1\s?472", t) || !has(r"1\s?280", t) || !has(r"1\s?252", t) {
        f.add("AC1", "the section does not state the 1 472, 1 280 and 1 252 byte figures");
    }
}

const NOTHING_OMITTED: &str = r"(?i)\b(?:no|nothing|none|never)\b[^.;]{0,80}?\bomitted\b|\bnot omitted\b|\bomitted\b[^.;]{0,20}?\bnever\b";

/// AC 3: the omission rule for the fallback, and the sender-obligation rule.
pub(crate) fn omission(sec: &Sec, cs: &[Case], k: &Comp, f: &mut Fails) {
    let omit_lines: Vec<&str> = sec
        .sec_lines
        .iter()
        .zip(&sec.sec_fence)
        .filter(|(l, fence)| !**fence && has_i(r"\bomit", l))
        .map(|(l, _)| l.as_str())
        .collect();
    let omit_text = norm(&omit_lines.join(" "));
    let nothing = sentences(&sec.sec_text)
        .iter()
        .any(|s| has_i(r"\bfallback\b", s) && rx(NOTHING_OMITTED).is_match(s));
    if !nothing {
        if omit_text.is_empty() {
            f.add("AC3", "the section states neither an omission rule nor that no optional part is omitted for size on a fallback path (no line says 'omit')");
        } else {
            rule_words(&omit_text, f);
            for c in cs.iter().filter(|c| c.kind == "fb") {
                let omits = c.ann != k.ann_max() || c.ext == 0;
                if omits && !has_i(r"\bamend|\bscop(?:e|es|ed|ing)\b|\bsupersed", &sec.sec_text) {
                    f.add("AC3", format!("fallback column {} omits announcements or the extension, but the section does not say that it amends or scopes the landed sender obligations (C-44, 004-10)", c.label));
                }
            }
        }
    }
    if !cs.iter().any(|c| c.kind == "fb") {
        f.add("AC3", "no 1 280-byte fallback budget column");
    }
}

fn rule_words(omit_text: &str, f: &mut Fails) {
    if !has_i("announcement", omit_text) {
        f.add("AC3", "the omission rule does not say which announcements are omitted");
    }
    if !has_i("measurement|extension", omit_text) {
        f.add("AC3", "the omission rule does not say what happens to the measurement extension");
    }
    if !has_i("controller", omit_text) {
        f.add("AC3", "the omission rule does not say what happens to controller messages");
    }
    if !has_i(r"priorit|in this order|\bfirst\b|\bthen\b|\bbefore\b|\bordered\b", omit_text) {
        f.add("AC3", "the omission rule states no priority (which part goes first)");
    }
}
