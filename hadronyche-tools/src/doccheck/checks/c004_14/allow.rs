//! AC 4 (the verdict) and AC 6 (the controller allowance and the sparse
//! arithmetic).

use std::collections::BTreeSet;

use super::comp::{Comp, Consts};
use super::section::{STable, Sec, norm, row_label, sentences};
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::report::{Flow, Report, crash};
use crate::doccheck::tables::{NUM_004_14, cell_number_004_14, first_int_004_14};
use crate::doccheck::text::{has, has_i, rx};

/// AC 4: the verdict against the falsifier.
pub(crate) fn verdict(sec: &Sec, k: &Comp, c: &Consts, f: &mut Report) {
    let n = sec.sec_lines.len();
    let mut chunks: Vec<String> = Vec::new();
    for i in 0..n {
        if sec.sec_fence[i] || !sec.sec_lines[i].to_lowercase().contains("verdict") {
            continue;
        }
        let open = |j: usize| !sec.sec_lines[j].py_strip().is_empty() && !sec.sec_fence[j];
        let mut lo = i;
        while lo > 0 && open(lo - 1) {
            lo -= 1;
        }
        let mut hi = i;
        while hi + 1 < n && open(hi + 1) {
            hi += 1;
        }
        let chunk = norm(&sec.sec_lines[lo..=hi].join(" "));
        if !chunks.contains(&chunk) {
            chunks.push(chunk);
        }
    }
    let vtext = chunks.join(" ");
    if vtext.is_empty() {
        f.fail("AC4", "no verdict: the section answers the open sub-question with no paragraph containing 'verdict'");
        return;
    }
    let holds = has_i(r"C-137 holds|\bholds\b|not falsified|\bconfirmed\b", &vtext);
    let falsified = has_i(r"(?<!not )\bfalsified\b", &vtext);
    let (mandatory, cap) = (k.mandatory(), c.cap);
    if mandatory <= cap {
        if falsified {
            f.fail("AC4", format!("the verdict says falsified, but the mandatory headers and checksum are {mandatory} bytes, within the {cap}-byte cap"));
        }
        if !holds {
            f.fail(
                "AC4",
                "the verdict does not say C-137 holds (or 'not falsified' / 'confirmed')",
            );
        }
        return;
    }
    if holds && !falsified {
        f.fail("AC4", format!("the verdict holds, but the mandatory headers and checksum are {mandatory} bytes, above the {cap}-byte cap"));
    }
    if !falsified {
        f.fail("AC4", "the verdict must report C-137 falsified");
    }
    if !has_i(r"5 channels|five channels", &sec.sec_text) {
        f.fail(
            "AC4",
            "a falsified verdict must say C-04 falls to 5 channels at the fallback",
        );
    }
    if c.c04
        .as_deref()
        .and_then(|t| first_int_004_14(t).ok().flatten())
        != Some(5)
    {
        f.fail(
            "AC4",
            "a falsified verdict needs the constants table to show C-04 at 5 channels first",
        );
    }
}

fn column_of(hdr: &[String], pattern: &str, exclude: Option<&str>) -> Option<usize> {
    hdr.iter()
        .position(|h| has_i(pattern, h) && !exclude.is_some_and(|e| has_i(e, h)))
}

/// A computed allowance with its row label.
pub(crate) type AllowRow = (String, i128);

fn cell(r: &[String], i: usize) -> Option<i128> {
    r.get(i)
        .and_then(|c| cell_number_004_14(c).ok().flatten())
        .map(super::comp::limited)
}

/// One allowance row; returns its allowance.
fn allowance_row(
    r: &[String],
    cols: [usize; 5],
    seen: &mut BTreeSet<&'static str>,
    k: &Comp,
    c: &Consts,
    f: &mut Report,
) -> Option<i128> {
    let lab = &r[0];
    let path = if has(r"1\s?472", lab) {
        "default"
    } else if has_i(r"1\s?280|1\s?252|fallback", lab) {
        "fb"
    } else {
        f.fail(
            "AC6",
            format!("allowance row {lab:?} names neither the 1 472 path nor the fallback"),
        );
        return None;
    };
    let vals: Vec<Option<i128>> = cols.iter().map(|&i| cell(r, i)).collect();
    let [
        Some(bound),
        Some(audio),
        Some(counted),
        Some(cksum),
        Some(allow),
    ] = vals[..]
    else {
        f.fail("AC6", format!("allowance row {lab:?} has a missing number"));
        return None;
    };
    seen.insert(path);
    let expect = if bound == c.cap {
        c.cap - counted - cksum
    } else if bound == c.pay_def || bound == c.fb_pay {
        bound - audio - counted - cksum
    } else {
        f.fail("AC6", format!("allowance row {lab:?}: bound {bound} is neither the cap {} nor a path payload ({} or {})", c.cap, c.pay_def, c.fb_pay));
        return None;
    };
    if path == "default" && bound != c.cap && bound != c.pay_def {
        f.fail(
            "AC6",
            format!(
                "allowance row {lab:?}: the 1 472 path takes the cap or {}",
                c.pay_def
            ),
        );
    }
    if path == "fb" && bound != c.cap && bound != c.fb_pay {
        f.fail(
            "AC6",
            format!(
                "allowance row {lab:?}: the fallback takes the cap or {}",
                c.fb_pay
            ),
        );
    }
    if audio != c.audio {
        f.fail(
            "AC6",
            format!("allowance row {lab:?}: audio {audio} != {}", c.audio),
        );
    }
    if cksum != k.cksum {
        f.fail(
            "AC6",
            format!(
                "allowance row {lab:?}: checksum {cksum} != {} (toml)",
                k.cksum
            ),
        );
    }
    if !k.allowed_counted().contains(&counted) {
        f.fail("AC6", format!("allowance row {lab:?}: counted headers {counted} are not the mandatory parts plus an announcement count and the extension (0 or present)"));
    }
    if allow != expect {
        f.fail(
            "AC6",
            format!("allowance row {lab:?}: {allow} != the allowance {expect}"),
        );
    }
    if allow < 0 {
        f.fail("AC6", format!("allowance row {lab:?} is negative"));
    }
    Some(allow)
}

fn allowance_table(t: &STable, k: &Comp, c: &Consts, f: &mut Report) -> Vec<AllowRow> {
    let hdr = &t.header;
    let cols = (
        column_of(hdr, r"bound|cap|payload|limit", None),
        column_of(hdr, r"audio", None),
        column_of(hdr, r"count|header", Some(r"bound|cap|payload")),
        column_of(hdr, r"checksum|crc", None),
        column_of(hdr, r"allow", None),
    );
    let (Some(b), Some(a), Some(cn), Some(ck), Some(l)) = cols else {
        f.fail("AC6", "the allowance table needs columns for the bound (or cap or payload), audio, counted headers, checksum and allowance");
        return Vec::new();
    };
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for r in &t.rows {
        if row_label(r).is_empty() {
            continue;
        }
        if let Some(allow) = allowance_row(r, [b, a, cn, ck, l], &mut seen, k, c, f) {
            rows.push((r[0].to_lowercase(), allow));
        }
    }
    if !seen.contains("default") || !seen.contains("fb") {
        f.fail(
            "AC6",
            "the allowance is not stated at both path sizes (1 472 and fallback)",
        );
    }
    rows
}

const WORDS: [(&str, i128); 11] = [
    ("zero", 0),
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
];

/// `to_int(m.group(0))` of a `NUM_RE` match.
fn num_value(m: &str) -> Option<i128> {
    parse_int(&rx(r"[ \xa0\u202f,]").replace_all(m, "")).ok()
}

fn sparse_sentence(t: &str, k: &Comp, allowances: &[i128]) -> Flow<bool> {
    let mut cands: BTreeSet<i128> = BTreeSet::new();
    for &a in allowances {
        let Some(tr) = k.transport else {
            return crash("TypeError: TRANSPORT + SP_HDR");
        };
        if a >= tr + k.sp_hdr {
            let Some(se) = k.sp_entry else {
                return crash("TypeError: // SP_ENTRY");
            };
            if se == 0 {
                return crash("ZeroDivisionError");
            }
            cands.insert((a - tr - k.sp_hdr).div_euclid(se));
        }
    }
    for s in sentences(t) {
        if !(has_i("transport", &s) && has_i("sparse", &s)) {
            continue;
        }
        let mut nums: BTreeSet<i128> = rx(NUM_004_14)
            .find_iter(&s)
            .filter_map(|m| num_value(m.as_str()))
            .collect();
        for w in rx(r"[a-z]+").find_iter(&s.to_lowercase()) {
            if let Some((_, n)) = WORDS.iter().find(|(name, _)| *name == w.as_str()) {
                nums.insert(*n);
            }
        }
        let has_n = |v: Option<i128>| v.is_some_and(|v| nums.contains(&v));
        if has_n(k.transport) && has_n(k.sp_entry) && nums.iter().any(|n| cands.contains(n)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// AC 6; returns the computed allowances with their row labels.
pub(crate) fn allowance(sec: &Sec, k: &Comp, c: &Consts, f: &mut Report) -> Flow<Vec<AllowRow>> {
    let tables: Vec<&STable> = sec
        .tables
        .iter()
        .filter(|t| t.header.iter().any(|h| has_i("allow", h)))
        .collect();
    let rows = match tables.first() {
        None => {
            f.fail(
                "AC6",
                "no controller allowance table (a header cell containing 'allow')",
            );
            Vec::new()
        }
        Some(t) => allowance_table(t, k, c, f),
    };
    let t = &sec.sec_text;
    if !has_i(r"upper bound|most a source may share", t) {
        f.fail("AC6", "the section does not say the allowance is an upper bound (the most a source may share)");
    }
    if !has_i("second stream", t) || !t.contains("C-181") {
        f.fail(
            "AC6",
            "the section does not say a source may always use a second stream (C-181)",
        );
    }
    let audio_member = sentences(t).iter().any(|s| {
        has_i("audio member", s)
            && has_i(
                r"\bno controller|counts? no controller|carries? no controller",
                s,
            )
    });
    if !t.contains("C-205") || !audio_member {
        f.fail("AC6", "the section does not say a bundle source's audio member counts no controller messages (C-205)");
    }
    let allowances: Vec<i128> = rows.iter().map(|(_, a)| *a).collect();
    if !sparse_sentence(t, k, &allowances)? {
        f.fail("AC6", format!("no sentence gives the sparse entries that fit beside a transport state ({} bytes) inside an allowance, with the sparse entry size {} bytes and header {} bytes", k.transport.map_or(String::from("?"), |v| v.to_string()), k.sp_entry.map_or(String::from("?"), |v| v.to_string()), k.sp_hdr));
    }
    Ok(rows)
}
