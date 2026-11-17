//! AC 1 to AC 3 and the DoD tags, on the analysis section.

use super::rates::{Cand, MODULUS, around, candidates, durations, pairs};
use crate::doccheck::Report;
use crate::doccheck::numwin::window;
use crate::doccheck::text::{has, has_i, rx, splitlines};

const YEAR_S: f64 = 31_557_600.0;
const MIN_CYCLE_S: f64 = 1.9;
const PATHO: &str = r"patholog|ceiling";
const PLAUS_PRIMARY: &str = r"plausible\s+(?:worst[\s-]*case\s+)?(?:operating\s+)?rate|worst[\s-]*case(?:\s+operating)?\s+rate|operating\s+rate|plausible\s+(?:bump|epoch|rate)";
const PLAUS_SECONDARY: &str = r"plausib|worst[\s-]*case|operating";
const TAG: &str = r"\[(?:decision|memory|measured|code|standard|kernel-doc)\]";

/// `analyse_rate`: the rate of one kind and the stated wrap that agrees
/// with it.
fn analyse_rate(
    markers: &[&str],
    durs: &[(f64, usize)],
    section: &str,
    ceiling: Option<f64>,
) -> (Option<Cand>, Option<f64>) {
    let (mut chosen, mut fallback): (Option<Cand>, Option<Cand>) = (None, None);
    for marker in markers {
        for c in candidates(marker, section) {
            if ceiling.is_some_and(|ci| c.interval < ci - 1e-9) {
                fallback.get_or_insert(c);
                continue;
            }
            if let Some(w) = pairs(&c, durs) {
                return (Some(c), Some(w));
            }
            chosen.get_or_insert(c);
        }
    }
    let chosen = chosen.or(fallback);
    let paired = chosen.as_ref().and_then(|c| pairs(c, durs));
    (chosen, paired)
}

/// `classify_verdict`: (falsified claimed, not falsified claimed).
fn classify_verdict(text: &str) -> (bool, bool) {
    let (mut falsified, mut clear) = (false, false);
    for m in rx(r"(?i)falsified").find_iter(text) {
        let pre = window(text, m.start(), m.start(), 40, 0).to_lowercase();
        if has(r"\b(?:not|never|no)\s*$", &pre) {
            clear = true;
        } else {
            falsified = true;
        }
    }
    if has_i(
        r"falsifier\s+(?:does\s+not|is\s+not)\s+(?:bite|met|satisfied|confirmed)",
        text,
    ) {
        clear = true;
    }
    if has_i(
        r"falsifier\s+(?:bites|is\s+met|is\s+satisfied|is\s+confirmed)",
        text,
    ) {
        falsified = true;
    }
    if has_i(r"C-10\s+(?:holds|stands|survives)", text) {
        clear = true;
    }
    (falsified, clear)
}

/// `comparison_rule_found`.
fn comparison_rule_found(text: &str) -> bool {
    rx(r"(?i)stale").find_iter(text).any(|m| {
        let low = window(text, m.start(), m.end(), 350, 350).to_lowercase();
        low.contains("current")
            && low.contains("epoch")
            && low.contains("wrap")
            && has(r"equal|serial|modular|compar|same\b|differs?", &low)
    })
}

/// The two rates and their sources (AC 1).
fn rates(section: &str, r: &mut Report) -> (Option<Cand>, Option<f64>, Option<Cand>, Option<f64>) {
    let durs = durations(section);
    let (patho, patho_wrap) = analyse_rate(&[PATHO], &durs, section, None);
    let (plaus, plaus_wrap) = analyse_rate(
        &[PLAUS_PRIMARY, PLAUS_SECONDARY],
        &durs,
        section,
        patho.as_ref().map(|c| c.interval),
    );
    for (key, cand, wrap) in [
        ("pathological", &patho, patho_wrap),
        ("plausible", &plaus, plaus_wrap),
    ] {
        match (cand, wrap) {
            (None, _) => r.fail("AC1", format!("no {key} rate stated (a bump interval or rate in the text after the words 'pathological'/'plausible')")),
            (Some(c), None) => r.fail("AC1", format!("the {key} rate ('{}' -> one bump per {} s) states no matching time-to-wrap: expected about {} s (65 536 x interval) after it in the section", c.matched, c.interval, MODULUS * c.interval)),
            (Some(_), Some(_)) => {}
        }
    }
    (patho, patho_wrap, plaus, plaus_wrap)
}

fn ac1(section: &str, r: &mut Report) -> (bool, bool, Option<f64>) {
    if !has(r"65[\d,\u{a0}\u{202f} ]*536|2\^16|2\*\*16", section) {
        r.fail("AC1", "the analysis never states the 65 536 (2^16, C-10) epoch values the wrap is computed over");
    }
    let missing: Vec<&str> = ["C-113", "C-213", "C-136", "C-32", "C-33", "C-37"]
        .into_iter()
        .filter(|c| !section.contains(c))
        .collect();
    if !missing.is_empty() {
        r.fail(
            "AC1",
            format!("missing source citation(s): {}", missing.join(", ")),
        );
    }
    let (patho, patho_wrap, plaus, plaus_wrap) = rates(section, r);
    if let Some(p) = &patho
        && p.interval < MIN_CYCLE_S - 1e-9
    {
        r.fail("AC1", format!("the pathological ceiling (one bump per {} s) is faster than the election timers allow: C-32 (1.5 s) + C-33 (0.4 s) bound a bump at {MIN_CYCLE_S} s", p.interval));
    }
    if let (Some(p), Some(q)) = (&patho, &plaus)
        && q.interval < p.interval - 1e-9
    {
        r.fail("AC1", format!("internal inconsistency: the plausible rate (one bump per {} s) is faster than the stated pathological ceiling (one bump per {} s)", q.interval, p.interval));
    }
    if !has_i(r"1[\s\u{a0}]?year|one\s+year|a\s+year|365\s*days", section) {
        r.fail(
            "AC1",
            "the 1-year falsifier threshold is never named ('1 year', 'one year', '365 days')",
        );
    }
    let (falsified, clear) = classify_verdict(section);
    if !falsified && !clear {
        r.fail(
            "AC1",
            "no decidable verdict against the falsifier ('falsified' or 'not falsified')",
        );
    }
    let consistent = !(falsified && clear);
    if !consistent {
        r.fail(
            "AC1",
            "the verdict is contradictory (both 'falsified' and 'not falsified' claimed)",
        );
    }
    let wrap = plaus_wrap.or_else(|| plaus.as_ref().map(|c| MODULUS * c.interval));
    if let (Some(w), true) = (wrap, consistent) {
        if w >= YEAR_S && falsified {
            r.fail("AC1", format!("the verdict contradicts the arithmetic: at the plausible rate 65 536 epochs take {w} s >= 1 year, yet the analysis says falsified"));
        } else if w < YEAR_S && clear {
            r.fail("AC1", format!("the verdict contradicts the arithmetic: at the plausible rate 65 536 epochs take only {w} s < 1 year, yet the analysis says not falsified"));
        }
    }
    if let Some(p) = &patho {
        let w = patho_wrap.unwrap_or(MODULUS * p.interval);
        let windows: Vec<String> = rx(r"(?i)falsif|verdict")
            .find_iter(section)
            .map(|m| around(section, m, 500))
            .collect();
        if w < YEAR_S && !windows.is_empty() && !windows.iter().any(|x| has_i(PATHO, x)) {
            r.fail("AC1", format!("the pathological ceiling wraps 65 536 epochs in {w} s, under a year, but the verdict never addresses the ceiling — the falsifier is not answered for it"));
        }
    }
    (falsified, clear, wrap)
}

fn ac3(section: &str, open_text: &str, falsified: bool, wrap: Option<f64>, r: &mut Report) {
    let really = falsified || wrap.is_some_and(|w| w < YEAR_S);
    if !really {
        if falsified {
            r.fail(
                "AC3",
                "the analysis claims falsified but the plausible wrap is at or above a year",
            );
        }
        for m in rx(r"(?i)open\s+item").find_iter(section) {
            let w = around(section, m, 300);
            if !(has_i("owner", &w) && has_i("falsif", &w)) {
                r.fail(
                    "DoD",
                    "the analysis raises an open item without an owner and a falsifier beside it",
                );
                break;
            }
        }
        return;
    }
    if !falsified {
        r.fail("AC3", "the plausible rate wraps under a year but the analysis does not declare C-10 falsified");
    }
    let fix = r"(?i)serial[\s-]*number|serial[\s-]*arithmetic|wider|replacement\s+width|proposed\s+width|\d{1,2}[\s\u{a0}-]{1,3}bits?[^.]{0,30}\bepoch\b|\bepoch\b[^.]{0,30}\d{1,2}[\s\u{a0}]*bits?";
    if !has(fix, section) {
        r.fail(
            "AC3",
            "no proposed replacement width or wrap-safe (serial-number) comparison rule",
        );
    }
    if !section.to_lowercase().contains("constants table") {
        r.fail(
            "AC3",
            "the falsification is not flagged for the constants table (DoD: constants table first)",
        );
    }
    let open_row = splitlines(open_text)
        .into_iter()
        .filter(|l| l.trim_start().starts_with('|') && has_i("epoch|wrap", l))
        .any(|l| {
            let cells: Vec<&str> = l.split('|').map(str::trim).collect();
            cells.len() >= 5 && {
                let (owner, fals) = (cells[cells.len() - 3], cells[cells.len() - 2]);
                !["", "\u{2014}", "-"].contains(&owner) && !["", "\u{2014}", "-"].contains(&fals)
            }
        });
    if !(open_row || (section.to_lowercase().contains("owner") && has_i("falsif", section))) {
        r.fail(
            "AC3",
            "no open item with an owner and a falsifier for the falsified C-10 width",
        );
    }
}

fn dod(section: &str, r: &mut Report) {
    let n = rx(TAG).find_iter(section).count();
    if n < 3 {
        r.fail("DoD", format!("the analysis carries {n} evidence tag(s); at least 3 are required ([code], [measured], [kernel-doc], [standard], [memory], [decision])"));
    }
    for (key, marker) in [("pathological", PATHO), ("plausible", PLAUS_PRIMARY)] {
        if let Some(m) = rx(&format!("(?i){marker}")).find(section)
            && !has(r"\[(decision|memory)\]", &around(section, m, 500))
        {
            r.fail(
                "DoD",
                format!("the {key} rate carries no [decision]/[memory] tag"),
            );
        }
    }
    if let Some(m) = rx(r"(?i)falsif|verdict").find(section)
        && !has(TAG, &around(section, m, 500))
    {
        r.fail("DoD", "the verdict carries no evidence tag");
    }
}

/// AC 1 to AC 3 and the DoD tags.
pub(crate) fn run(section: &str, wire_text: &str, open_text: &str, r: &mut Report) {
    let (falsified, _clear, wrap) = ac1(section, r);
    if !(comparison_rule_found(section) || comparison_rule_found(wire_text)) {
        r.fail("AC2", "no comparison rule for 'current' vs 'stale' epochs that addresses behaviour at wrap (an epoch, current, stale, a comparison and 'wrap' must appear together)");
    }
    ac3(section, open_text, falsified, wrap, r);
    dod(section, r);
}
