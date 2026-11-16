//! The text assertions of AC 1 to AC 4 and AC 6, the self-check of the
//! constants, and the worked-example assertions of AC 5 and AC 3.

use super::rows::{SecTable, ceil_to_offset, floor_to_position, verify};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_15::num::window;
use crate::doccheck::text::{has, has_i, rx};

const FLOOR: &str = r"floor|round(?:ed|ing)?\s+down|toward\s+-?inf|\u{230a}";
const CEIL: &str = r"ceil|round(?:ed|ing)?\s+up|toward\s+\+?inf|\u{2308}";

/// `near(text, a, b, width)`: a match of `a` with `b` within `width`
/// characters of it, both ignoring case.
fn near(text: &str, a: &str, b: &str, width: usize) -> bool {
    let rb = rx(&format!("(?i){b}"));
    rx(&format!("(?i){a}")).find_iter(text).any(|m| rb.is_match(&window(text, m.start(), m.end(), width, width)))
}

/// The assertion of an identity `outer(inner(v)) <rel> v` with the same
/// variable on both sides (the script's backreference).
fn identity(text: &str, head: &str, rel: &str) -> bool {
    let re = rx(&format!(r"{head}([A-Za-z][A-Za-z0-9_]*)\s*\)\s*\)\s*(?:{rel})\s*"));
    re.captures_iter(text).any(|c| {
        let end = c.get(0).map_or(0, |m| m.end());
        text[end..].starts_with(&c[1])
    })
}

/// AC 1: both formulas, rounding, a 128-bit intermediate.
pub(crate) fn ac1(t: &str, r: &mut Report) {
    let pos = has(r"position_at[^\n]{0,140}(?:=|:=|:=|\u{2261})", t);
    let time = has(r"time_of[^\n]{0,140}(?:=|:=|:=|\u{2261})", t);
    if !pos {
        r.fail("AC1", "no 'position_at(...) = ...' formula");
    }
    if !time {
        r.fail("AC1", "no 'time_of(...) = ...' formula");
    }
    if !near(t, "position_at", FLOOR, 260) {
        r.fail("AC1", "position_at does not state floor/round down");
    }
    if !near(t, "time_of", CEIL, 260) {
        r.fail("AC1", "time_of does not state ceil/round up");
    }
    if !has_i(r"\b[ui]?128\b|128[\s-]*bit", t) {
        r.fail("AC1", "does not name a 128-bit (or wider) intermediate for (t - origin) * 96 000");
    }
}

/// The constants the section depends on: exact, over the script's ranges.
/// Returns the problems (empty when all hold).
pub fn self_check() -> Vec<String> {
    let mut problems = Vec::new();
    let (rate, ns) = (96_000i128, 1_000_000_000i128);
    if rate * 125_000 != 12 * ns {
        problems.push("96 000/10^9 != 12/125 000".to_string());
    }
    if ns * 3 != 31_250 * rate {
        problems.push("10^9/96 000 != 31 250/3".to_string());
    }
    if let Some(p) = (0..=300_000i128).find(|&p| floor_to_position(ceil_to_offset(p), 0) != p) {
        problems.push(format!("position_at(time_of({p})) != {p}"));
    }
    for k in 0..=20_000i128 {
        let p = 48 * k;
        if (p * 31_250) % 3 != 0 {
            problems.push(format!("slice boundary p={p} is not whole ns"));
            break;
        }
        if floor_to_position(p * 31_250 / 3, 0) != p {
            problems.push(format!("boundary inverse fails at p={p}"));
            break;
        }
    }
    if let Some(t) = (0..=300_000i128).find(|&t| ceil_to_offset(floor_to_position(t, 0)) > t) {
        problems.push(format!("time_of(position_at({t})) > {t}"));
    }
    problems
}

/// AC 2: the inverse identities, 48 samples = 500 000 ns, the reductions.
pub(crate) fn ac2(t: &str, r: &mut Report) {
    if !identity(t, r"position_at\s*\(\s*time_of\s*\(\s*", "==|=|\u{2261}") {
        r.fail("AC2", "does not state position_at(time_of(p)) == p");
    }
    if !identity(t, r"time_of\s*\(\s*position_at\s*\(\s*", r"\u{2264}|<=|\\leq") {
        r.fail("AC2", "does not state time_of(position_at(t)) <= t");
    }
    if !(has(r"\b48\b", t) && has(r"\b500\s*000\b|\b500000\b", t)) {
        r.fail("AC2", "does not give 48 samples = 500 000 ns");
    }
    let exact = near(t, r"\b500\s*000\b|\b500000\b", r"exact|without\s+rounding|whole\s+nanosecond|no\s+rounding", 260);
    if exact && !t.to_lowercase().contains("boundar") {
        r.fail("AC2", "exactness argument does not mention slice boundaries");
    } else if !exact {
        r.fail("AC2", "no argument that 48-sample slice boundaries are exact (500 000 ns exactly)");
    }
    if !(has(r"\b12\s*/\s*125\s*000\b", t) || has(r"\b125\s*000\s*/\s*12\b", t) || near_cs(t, r"\b125\s*000\b", r"\b12\b", 80)) {
        r.fail("AC2", "does not show the 12 / 125 000 reduction");
    }
    if !(has(r"\b31\s*250\s*/\s*3\b", t) || near_cs(t, r"\b31\s*250\b", r"\b3\b", 80)) {
        r.fail("AC2", "does not show the 31 250 / 3 reduction");
    }
    let problems = self_check();
    if !problems.is_empty() {
        r.fail("AC2", format!("constant self-check failed: {}", problems.join("; ")));
    }
}

fn near_cs(t: &str, a: &str, b: &str, w: usize) -> bool {
    near(t, a, b, w)
}

/// AC 3: the rounding rule off a slice boundary.
pub(crate) fn ac3(t: &str, r: &mut Report) {
    let off = r"off[\s-]*(?:a\s+)?(?:slice\s+)?boundar|inside\s+(?:a\s+)?slice|within\s+(?:a\s+)?slice|not\s+on\s+a\s+slice\s+boundar|off[\s-]boundar";
    let rule = format!(r"{CEIL}|next\s+(?:whole\s+)?nanosecond|round(?:ed|ing)?\s+(?:up|upward)");
    if !(near(t, off, &rule, 300) && near(t, off, r"position|sample", 300)) {
        r.fail("AC3", "does not state the rounding rule for positions off a slice boundary (ceil/round up near the off-boundary case)");
    }
}

/// AC 4: behaviour before the origin.
pub(crate) fn ac4(t: &str, r: &mut Report) {
    let before = rx(r"(?i)\bt\s*<\s*origin|origin[\s_]*ns\s*[<\u{2264}]|origin[\s_]*ns\s*>\s*t|before\s+(?:the\s+)?(?:epoch[\s_]*)?origin|earlier\s+than\s+(?:the\s+)?origin|prior\s+to\s+(?:the\s+)?origin|negative\s+(?:network\s+)?(?:time|position)|would\s+be\s+negative|time\s+before\s+(?:the\s+)?origin");
    let mech = r"Option|None|Err\b|error|clamp|saturat|panic|reject|illegal|out\s+of\s+range";
    let win = before.find_iter(t).map(|m| window(t, m.start(), m.end(), 400, 400)).find(|w| has_i(r"\[decision\]", w) && has_i(mech, w));
    let Some(w) = win else {
        r.fail("AC4", "behaviour of position_at for t before epoch origin is not specified and tagged [decision]");
        return;
    };
    if !has_i(r"\bNone\b|\bErr\b|error|returns?\s+0\b|zero|u64::MAX|MAX|origin|clamp(?:s|ed|ing)?|saturat", &w) {
        r.fail("AC4", "before-origin rule does not say what is returned (None/Err or the clamped value)");
    }
}

/// AC 5 and AC 3 (off-boundary rows): the worked-example table.
pub(crate) fn ac5(tables: &[SecTable], origin: i128, r: &mut Report) {
    let (verified, problems) = verify(tables, origin);
    for p in problems {
        r.fail("AC5", p);
    }
    if verified.len() < 6 {
        r.fail("AC5", format!("worked-example table has {} verified row(s); at least 6 are required", verified.len()));
    }
    let has_dir = |d: &str| verified.iter().any(|v| v.direction == d);
    if !(has_dir("position") && has_dir("time")) {
        r.fail("AC5", "table does not give both directions");
    }
    if !verified.iter().any(|v| v.p % 48 == 0) {
        r.fail("AC5", "no slice-boundary example row (p divisible by 48)");
    }
    if verified.iter().filter(|v| v.p % 48 != 0).count() < 3 {
        r.fail("AC3", "fewer than three off-boundary example rows (at least three are required)");
    }
    for (p, expected) in [(1, 10_417), (2, 20_834), (3, 31_250)] {
        let mut offsets = std::collections::BTreeSet::new();
        for v in verified.iter().filter(|v| v.direction == "time" && v.p == p) {
            offsets.insert(v.output);
            if !v.relative_out {
                offsets.insert(v.output - origin);
            }
        }
        if !offsets.is_empty() && !offsets.contains(&expected) {
            r.fail("AC5", format!("ticket example p={p} should be {expected} ns, table says {offsets:?}"));
        }
    }
}

/// AC 6: the only rate with a position space, and the citations.
pub(crate) fn ac6(t: &str, r: &mut Report) {
    let only = near(t, r"position[\s-]*space", r"\bonly\b|no\s+other|single|sole", 260)
        || near(t, r"\bonly\s+(?:sample\s+)?rate\b", r"position|\b96", 260);
    if !only {
        r.fail("AC6", "does not state that 96 kHz is the only rate with a position space");
    }
    let missing: Vec<&str> = ["C-01", "C-80", "C-134"].into_iter().filter(|c| !t.contains(c)).collect();
    if !missing.is_empty() {
        r.fail("AC6", format!("missing citation(s): {}", missing.join(", ")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_constants_hold() {
        assert!(self_check().is_empty());
    }

    #[test]
    fn identities_need_the_same_variable() {
        let h = r"position_at\s*\(\s*time_of\s*\(\s*";
        assert!(identity("position_at(time_of(p)) == p", h, "==|="));
        assert!(!identity("position_at(time_of(p)) == q", h, "==|="));
    }
}
