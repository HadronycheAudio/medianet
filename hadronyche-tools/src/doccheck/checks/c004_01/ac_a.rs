//! AC 1 to AC 4.

use std::collections::BTreeSet;

use super::doc::{Doc, nearby, norm_cell, parse_tables};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_15::num::window;
use crate::doccheck::text::{has, has_i, rx};

fn orders(bo: &str) -> BTreeSet<String> {
    let pats = [
        r"(?i)\b(?:all|every)\s+(?:multi[-\s]?byte|multibyte)\b[^.\n]{0,160}?\b(?P<ord>little|big)[-\s]endian\b",
        r"(?i)\b(?P<ord>little|big)[-\s]endian\b[^.\n]{0,160}?\b(?:for|on|across|of)\s+(?:all|every)\s+(?:multi[-\s]?byte|multibyte)\b",
        r"(?i)\bbyte[\s-]*order\b[^.\n]{0,80}?(?::|is|are|=|\u{2014}|\u{2013}|-)\s*(?P<ord>little|big)[-\s]endian\b",
        r"(?i)\|\s*(?:byte[\s-]*order|endianness)\s*\|\s*(?P<ord>little|big)[-\s]endian\b",
        r"(?i)\b(?:multi[-\s]?byte|multibyte)\b[^.\n]{0,100}?\b(?:integers?|values?|fields?|quantities)\b[^.\n]{0,100}?\b(?P<ord>little|big)[-\s]endian\b",
    ];
    let mut out = BTreeSet::new();
    for p in pats {
        for m in rx(p).captures_iter(bo) {
            let w = m.get(0).map_or(0..0, |w| w.range());
            let prefix = window(bo, w.start, w.start, 24, 0);
            if has_i(r"\b(?:not|never|rather\s+than|instead\s+of|unlike|rejected)\s*$", &prefix) {
                continue;
            }
            out.insert(m["ord"].to_lowercase());
        }
    }
    let scoped = r"(?i)\b(?:all|every)\s+(?:multi[-\s]?byte|multibyte)|\b(?:multi[-\s]?byte|multibyte)\s+(?:integers?|fields?|values?|quantities?)";
    if has(scoped, bo) {
        for (pat, o) in [
            (r"(?i)\bnetwork\s+byte\s+order\b|\bmost[-\s]+significant[-\s]+byte[-\s]+first\b|\bMSB[-\s]+first\b", "big"),
            (r"(?i)\bleast[-\s]+significant[-\s]+byte[-\s]+first\b|\bLSB[-\s]+first\b", "little"),
        ] {
            if has(pat, bo) {
                out.insert(o.to_string());
            }
        }
    }
    if out.is_empty() {
        let mentioned: BTreeSet<String> =
            rx(r"(?i)\b(little|big)[-\s]endian\b").captures_iter(bo).map(|c| c[1].to_lowercase()).collect();
        if mentioned.len() == 1 && has(scoped, bo) {
            out = mentioned;
        }
    }
    out
}

/// AC 1: one byte order, tagged, justified.
pub(crate) fn ac1(d: &Doc, r: &mut Report) {
    let Some(i) = d.find(|t| has_i(r"byte[\s-]*order", t)) else {
        r.fail("AC1", "no byte-order section heading (expected e.g. 'Byte order and alignment')");
        return;
    };
    let bo = d.full(i);
    let o = orders(&bo);
    if o.is_empty() {
        r.fail("AC1", "byte-order section does not state which single order applies to all multi-byte wire integers");
    } else if o.len() > 1 {
        r.fail("AC1", format!("byte-order section declares more than one order: {o:?}"));
    }
    if !bo.contains("[decision]") {
        r.fail("AC1", "byte-order section does not carry the [decision] tag");
    }
    let just = r"(?i)\b(justif\w*|rationale|because|since|reason\w*|we choose|chosen|decided|therefore|thus|so\s+that|ensures?|avoids?|simplif\w*|minimi[sz]\w*|makes?|allows?|matches?|consistent|portable|network\s+byte\s+order|most[-\s]+significant|byte\s+first)\b";
    if !has(just, &bo) {
        r.fail("AC1", "byte-order section has no justification");
    }
}

/// Standalone numbers of 1, 2, 4, 8 or 16 in `text` (`(?<![\d.])(?:1|2|4|8|16)(?![\d.])`).
fn width_numbers(text: &str) -> Vec<&'static str> {
    let chars: Vec<char> = text.chars().collect();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < chars.len() && chars[j].is_ascii_digit() {
            j += 1;
        }
        let run: String = chars[i..j].iter().collect();
        let prev_ok = i == 0 || chars[i - 1] != '.';
        let next_ok = j >= chars.len() || chars[j] != '.';
        if prev_ok && next_ok
            && let Some(w) = ["1", "2", "4", "8", "16"].iter().find(|w| **w == run) {
                out.push(*w);
            }
        i = j;
    }
    out
}

fn mark_widths(text: &str, satisfied: &mut BTreeSet<&'static str>) {
    satisfied.extend(width_numbers(text));
    if has_i(r"sub[-\s]?byte|bit[-\s]?field|bitfield|\bbits?\b", text) {
        satisfied.insert("bit");
    }
}

/// AC 2: alignment stated for widths 1, 2, 4, 8, 16 and bit fields.
pub(crate) fn ac2(d: &Doc, r: &mut Report) {
    let Some(i) = d.find(|t| has_i("align", t)) else {
        r.fail("AC2", "no alignment section heading");
        return;
    };
    let al = d.full(i);
    let mut sat: BTreeSet<&'static str> = BTreeSet::new();
    let answer = |c: &str| {
        let c = c.trim().trim_matches(['*', '_', '`']);
        !c.is_empty() && !["\u{2014}", "\u{2013}", "-", "n/a", "na", "none?", "?"].contains(&c.to_lowercase().as_str())
    };
    for (_, header, rows) in parse_tables(&al) {
        let col = |p: &str| header.iter().position(|h| has(p, &h.to_lowercase()));
        let (Some(w), Some(a)) = (col(r"width|bytes?|size"), col(r"align|guarantee|padding|padded|packed|natural")) else { continue };
        for row in rows.iter().filter(|r| r.len() > w.max(a)) {
            if answer(&row[a]) {
                mark_widths(&row[w], &mut sat);
                if has_i(r"sub[-\s]?byte|bit[-\s]?field|bitfield", &row[w]) {
                    sat.insert("bit");
                }
            }
        }
    }
    for line in crate::doccheck::text::splitlines(&al) {
        if !line.trim_start().starts_with('|') && has_i(r"align|guarantee|padding|padded|packed", line) {
            mark_widths(line, &mut sat);
        }
    }
    for (w, label) in [("1", "1-byte fields"), ("2", "2-byte fields"), ("4", "4-byte fields"), ("8", "8-byte fields"), ("16", "16-byte fields"), ("bit", "sub-byte bit fields")] {
        if !sat.contains(w) {
            r.fail("AC2", format!("alignment section does not state whether natural alignment is guaranteed for {label}"));
        }
    }
}

const BL: &str = r"bit[-\s]?level\s+encoding";

fn inside_bit_level(d: &Doc, i: usize) -> bool {
    for j in (0..i).rev() {
        if d.heads[j].1 < d.heads[i].1 {
            return has_i(BL, &d.heads[j].2);
        }
    }
    false
}

/// AC 3: f32 and Q1.31 bit-exact under bit-level-encoding headings.
pub(crate) fn ac3(d: &Doc, r: &mut Report) {
    let levels: Vec<usize> = (0..d.heads.len()).filter(|&i| has_i(BL, &d.heads[i].2)).collect();
    if levels.is_empty() {
        r.fail("AC3", "no 'bit-level encoding' heading for f32 / Q1.31");
        return;
    }
    for (label, pat) in [("f32", r"\bf32\b"), ("Q1.31", r"\bq1[.\s_]?31\b")] {
        let named = (0..d.heads.len()).any(|i| has_i(pat, &d.heads[i].2) && (has_i(BL, &d.heads[i].2) || inside_bit_level(d, i)));
        if !named {
            r.fail("AC3", format!("no bit-level-encoding heading (or subsection) names {label}"));
        }
    }
    let bl = levels.iter().map(|&i| d.full(i)).collect::<Vec<_>>().join("\n\n");
    let (f32, q) = (r"\bf32\b", r"\bq1[.\s_]?31\b");
    if !has_i(f32, &bl) {
        r.fail("AC3", "bit-level encoding section does not define f32");
    }
    if !has_i(q, &bl) {
        r.fail("AC3", "bit-level encoding section does not define Q1.31");
    }
    let four = r"\b4\s*[- ]?bytes?\b|\b32\s*[- ]?bits?\b";
    let checks: [(&str, &str, usize, &str); 9] = [
        (f32, four, 400, "f32 is not stated as 4 bytes (32 bits)"),
        (q, four, 400, "Q1.31 is not stated as 4 bytes (32 bits)"),
        (f32, r"ieee\s*[- ]?754|binary32", 500, "f32 encoding does not cite IEEE 754 binary32"),
        (f32, r"\bexponent\b", 500, "f32 bit layout does not name the exponent field"),
        (f32, r"\bmantissa\b|\bfraction\b", 500, "f32 bit layout does not name the mantissa/fraction field"),
        (f32, r"\bsign\b", 500, "f32 bit layout does not name the sign bit"),
        (q, r"two'?s\s+complement|2'?s\s+complement", 500, "Q1.31 encoding does not state two's complement"),
        (q, r"\b31\b", 400, "Q1.31 encoding does not state its 31 fractional bits"),
        (q, r"fraction\w*|\bscale\b|2\s*\^?\s*[-\u{2212}]?\s*31", 500, "Q1.31 encoding does not state its fractional scale"),
    ];
    for (a, b, w, msg) in checks {
        if !nearby(&bl, a, b, w) {
            r.fail("AC3", msg);
        }
    }
}

/// AC 4: the five-column field table and the bounded variable-length rule.
pub(crate) fn ac4(d: &Doc, r: &mut Report) {
    let secs: Vec<usize> = (0..d.heads.len()).filter(|&i| has_i(r"field[\s-]*table", &d.heads[i].2)).collect();
    if secs.is_empty() {
        r.fail("AC4", "no field-table format section heading");
        return;
    }
    let want = ["offset", "width", "type", "meaning", "validrange"];
    let found = secs.iter().any(|&i| parse_tables(&d.section(i)).iter().any(|(_, h, _)| h.iter().map(|c| norm_cell(c)).eq(want.iter().map(|s| s.to_string()))));
    if !found {
        r.fail("AC4", "no table with exactly the five columns offset | width | type | meaning | valid range");
    }
    let combined = secs.iter().map(|&i| d.section(i)).collect::<Vec<_>>().join("\n\n");
    let rule = rx(r"(?i)variable[\s-]*length").find_iter(&combined).any(|m| {
        let w = window(&combined, m.start(), m.end(), 400, 400);
        has_i(r"max(?:imum)?", &w) && has_i(r"count|fixed|bounded|limit", &w)
    });
    if !rule {
        r.fail("AC4", "no rule that every variable-length part has a stated maximum count");
    }
}
