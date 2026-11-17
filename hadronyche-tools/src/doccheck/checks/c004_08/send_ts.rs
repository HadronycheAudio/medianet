//! AC1 of 004-08: the send timestamp's width, unit, clock source and wrap
//! period, each `[decision]`-tagged, and the falsifier arithmetic (the unit
//! resolves one sample period at 96 000 Hz; the wrap outlasts a reassembly
//! span of 2 slice periods). Durations are Python floats.

use super::Ctx;
use crate::doccheck::Report;
use super::wp5::big::Big;
use super::wp5::py::{big_to_f64, fold_i, head, letter_or_micro_i, py_float, py_int, py_strip, s, search_i_not_after, si};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::rx;

/// An exception the script did not catch.
pub(crate) type Crash = String;

const UNIT_ALT: &str = r"nanoseconds?|microseconds?|milliseconds?|seconds?|minutes?|hours?|days?|weeks?|years?|ns|µs|us|ms|sec(?:onds?)?|min";

/// One sample period at 96 000 Hz, in ns (`1e7 / 960.0`).
pub(crate) const SAMPLE_PERIOD_NS: f64 = 1e7 / 960.0;

/// `UNIT_NS[unit]` (exact keys, after `lower()`).
fn unit_ns(unit: &str) -> Option<f64> {
    Some(match unit.to_lowercase().as_str() {
        "ns" | "nanosecond" | "nanoseconds" => 1.0,
        "µs" | "us" | "microsecond" | "microseconds" => 1e3,
        "ms" | "millisecond" | "milliseconds" => 1e6,
        "s" | "sec" | "second" | "seconds" => 1e9,
        "min" | "minute" | "minutes" => 6e10,
        "hour" | "hours" => 3.6e12,
        "day" | "days" => 8.64e13,
        "week" | "weeks" => 6.048e14,
        "year" | "years" => 3.1536e16,
        _ => return None,
    })
}

/// `durations_in(text)`: (value in ns, matched text), in text order. The
/// look-behind before the unit always holds here (a digit or a space
/// precedes it). `2^N` with N of 1024 or more raised OverflowError.
pub(crate) fn durations_in(text: &str) -> Result<Vec<(f64, String)>, Crash> {
    let folded = fold_i(text);
    let mut hits: Vec<(usize, f64, String)> = Vec::new();
    for c in rx(&format!(r"(?i)2\^\s*(\d+)\s*({UNIT_ALT})\b")).captures_iter(&folded) {
        if let Some(u) = unit_ns(&c[2]) {
            let n = py_int(&c[1]).expect("digits");
            let Some(e) = n.to_i64().filter(|&e| e < 1024) else {
                return Err("OverflowError: int too large to convert to float".into());
            };
            hits.push((c.get(0).expect("match").start(), 2f64.powi(e as i32) * u, c[0].to_string()));
        }
    }
    for c in rx(&format!(r"(?i)(\d[\d _]*(?:\.\d+)?)\s*({UNIT_ALT})\b")).captures_iter(&folded) {
        let start = c.get(0).expect("match").start();
        let before = folded[..start].chars().next_back();
        let at = folded[start..].chars().next();
        if before == Some('^') || at == Some('^') {
            continue;
        }
        if let Some(u) = unit_ns(&c[2]) {
            let v = py_float(&c[1].replace([' ', '_', ','], "")).expect("a decimal");
            hits.push((start, v * u, c[0].to_string()));
        }
    }
    hits.sort_by_key(|h| h.0);
    Ok(hits.into_iter().map(|(_, v, t)| (v, t)).collect())
}

/// `stated_wrap_ns(region)`.
fn stated_wrap_ns(region: &str) -> Result<Option<(f64, String)>, Crash> {
    let folded = fold_i(region);
    for pattern in [r"(?i)wrap(?:s)?\s+period[^.;\n]{0,140}", r"(?i)\bwraps?\s+(?:after|at|in)\b[^.;\n]{0,140}"] {
        for m in rx(pattern).find_iter(&folded) {
            if let Some(d) = durations_in(m.as_str())?.into_iter().next() {
                return Ok(Some(d));
            }
        }
    }
    Ok(None)
}

/// `stated_slice_period_ns(prose)`.
fn stated_slice_period_ns(prose: &str) -> Result<(f64, String), Crash> {
    if let Some(c) = rx(r"(\d[\d _]*)\s*frames?\b").captures(prose)
        && s(r"96[\s_]?000", prose)
    {
        let frames = py_int(&c[1].replace([' ', '_'], "")).expect("digits");
        if frames > Big::default() {
            let Some(f) = big_to_f64(&frames) else {
                return Err("OverflowError: int too large to convert to float".into());
            };
            return Ok((f / 96000.0 * 1e9, format!("{frames} frames at 96 000 Hz (C-02, C-01)")));
        }
    }
    let folded = fold_i(prose);
    let pattern = format!(r"(?i)slice\s+periods?\s+(?:of\s+)?(\d[\d _]*(?:\.\d+)?)\s*({UNIT_ALT})\b");
    for c in rx(&pattern).captures_iter(&folded) {
        if let Some(u) = unit_ns(&c[2]) {
            let v = py_float(&c[1].replace([' ', '_', ','], "")).expect("a decimal");
            return Ok((v * u, c[0].to_string()));
        }
    }
    Ok((48.0 / 96000.0 * 1e9, "the C-02 default 48 frames at 96 000 Hz".into()))
}

fn ns_word(t: &str) -> bool {
    search_i_not_after(r"nanoseconds?\b|ns\b", t, letter_or_micro_i)
}

fn us_word(t: &str) -> bool {
    search_i_not_after(r"microseconds?\b|us\b", t, letter_or_micro_i) || si("µs", t)
}

fn ms_word(t: &str) -> bool {
    search_i_not_after(r"milliseconds?\b|ms\b", t, letter_or_micro_i)
}

/// `stated_unit_tick_ns(region)`: the tick in ns, and its name.
fn stated_unit_tick_ns(region: &str) -> Option<(f64, &'static str)> {
    let folded = fold_i(region);
    let window = r"(?i)\bunits?\b\s*(?:is|are|=|:)?\s*(?:the\s+|in\s+|of\s+|one\s+)?[^.;\n]{0,60}";
    for m in rx(window).find_iter(&folded) {
        let w = m.as_str();
        if si(r"samples?\s+at\s+(?:the\s+)?(?:domain\s+rate|96)|sample\s+per\s+(?:domain\s+)?tick", w) {
            return Some((SAMPLE_PERIOD_NS, "samples at 96 000 Hz"));
        }
        if ns_word(w) {
            return Some((1.0, "nanoseconds"));
        }
        if us_word(w) {
            return Some((1e3, "microseconds"));
        }
        if ms_word(w) {
            return Some((1e6, "milliseconds"));
        }
    }
    if ns_word(region) {
        return Some((1.0, "nanoseconds"));
    }
    if us_word(region) {
        return Some((1e3, "microseconds"));
    }
    if ms_word(region) {
        return Some((1e6, "milliseconds"));
    }
    if si(r"samples?\s+at\s+(?:the\s+)?(?:domain\s+rate|96)", region) || si(r"one\s+sample\b|sample\s+period", region) {
        return Some((SAMPLE_PERIOD_NS, "samples at 96 000 Hz"));
    }
    None
}

/// `decision_tagged(pattern)`: (tagged, the first block matching).
fn decision_tagged(blks: &[String], pattern: &str) -> (bool, Option<String>) {
    let Some(k) = blks.iter().position(|b| si(pattern, b)) else {
        return (false, None);
    };
    let tagged = [k.wrapping_sub(1), k, k + 1].iter().any(|&c| c < blks.len() && si(r"\[decision\]", &blks[c]));
    (tagged, Some(blks[k].clone()))
}

/// The send-timestamp blocks and their neighbours, in order.
fn region(blks: &[String]) -> String {
    let mut idx = std::collections::BTreeSet::new();
    for (k, b) in blks.iter().enumerate() {
        if si(r"send[\s_-]*timestamp", b) {
            idx.extend([k.wrapping_sub(1), k, k + 1]);
        }
    }
    idx.into_iter().filter(|&k| k < blks.len()).map(|k| blks[k].as_str()).collect::<Vec<_>>().join("\n")
}

/// The wrap period against one reassembly span, named with the spans.
fn wrap_rules(ctx: &Ctx, region: &str, report: &mut Report) -> Result<(), Crash> {
    let Some((wrap_ns, wrap_text)) = stated_wrap_ns(region)? else {
        report.fail("AC1", "the send timestamp's WRAP PERIOD is not stated");
        return Ok(());
    };
    let (slice_ns, src) = stated_slice_period_ns(&ctx.prose)?;
    let span = 2.0 * slice_ns;
    if wrap_ns <= span {
        report.fail(
            "AC1",
            format!("the stated wrap period ({wrap_text} = {wrap_ns} ns) wraps within one reassembly span (2 slice periods = {span} ns, from {src}; C-15) — the falsifier bites on the width"),
        );
    }
    if !si("reassembly", region) {
        report.fail("AC1", "the wrap-period justification does not name the reassembly span (C-15: 2 slice periods) it is checked against");
    }
    if !si("align", region) {
        report.fail("AC1", "the wrap-period justification does not name the alignment span (C-110: the consumer's own deadline) it is checked against");
    }
    Ok(())
}

/// AC1.
pub(crate) fn ac1(ctx: &Ctx, report: &mut Report) -> Result<(), Crash> {
    if !ctx.open {
        return Ok(());
    }
    let region = region(&ctx.blks);
    if py_strip(&region).is_empty() {
        report.fail("AC1", "no send-timestamp decision in the Slice header section (the open sub-question's answer)");
        return Ok(());
    }
    if !si(r"(\d[\d _]*)\s*[\s_-]*bits?\b", &region) {
        report.fail("AC1", "the send timestamp's WIDTH is not stated in bits");
    }
    match stated_unit_tick_ns(&region) {
        None => report.fail("AC1", "the send timestamp's UNIT is not stated (state it in ns, µs or samples with its rate, so its resolution can be checked)"),
        Some((tick, name)) if tick > SAMPLE_PERIOD_NS + 0.01 => report.fail(
            "AC1",
            format!("the send timestamp's unit ({name}, one tick = {tick} ns) cannot resolve one sample period at 96 000 Hz (10 416 2/3 ns); the falsifier bites on the unit"),
        ),
        Some(_) => {}
    }
    if !si(r"\bclock\b", &region) {
        report.fail("AC1", "the send timestamp's CLOCK SOURCE is not stated (the clock it is read from)");
    }
    wrap_rules(ctx, &region, report)?;
    if !s(r"96[\s_]?000|10[\s_]?416", &region) {
        report.fail("AC1", "the send-timestamp decision does not show the resolution arithmetic (one sample period at 96 000 Hz = 10 416 2/3 ns)");
    }
    let items = [
        ("width", r"\bbits?\b"),
        ("unit", r"\bunits?\b|samples?\s+at"),
        ("clock source", r"\bclock\b"),
        ("wrap period", r"\bwrap"),
    ];
    for (label, pattern) in items {
        let (ok, b) = if label == "unit" { unit_tagged(&ctx.blks) } else { decision_tagged(&ctx.blks, pattern) };
        if !ok {
            let at = b.map_or("?".to_string(), |b| head(splitlines(py_strip(&b)).first().copied().unwrap_or(""), 80));
            report.fail("AC1", format!("the send timestamp's {label} statement carries no [decision] tag: {at}…"));
        }
    }
    Ok(())
}

/// `decision_tagged(r"\bunits?\b|(?<![a-zA-Zµ])ns\b|samples?\s+at")`.
fn unit_tagged(blks: &[String]) -> (bool, Option<String>) {
    let hit = |b: &String| si(r"\bunits?\b|samples?\s+at", b) || search_i_not_after(r"ns\b", b, letter_or_micro_i);
    let Some(k) = blks.iter().position(hit) else {
        return (false, None);
    };
    let tagged = [k.wrapping_sub(1), k, k + 1].iter().any(|&c| c < blks.len() && si(r"\[decision\]", &blks[c]));
    (tagged, Some(blks[k].clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_as_python() {
        let d = durations_in("2^64 ns, 500 µs and 1 000 ms; 2^3 seconds").unwrap();
        let v: Vec<f64> = d.iter().map(|x| x.0).collect();
        assert_eq!(v, vec![2f64.powi(64), 500e3, 1e9, 8e9]);
        assert!(durations_in("2^1024 ns").is_err());
        assert!(ns_word("in ns") && !ns_word("Kns") && !ns_word("µns") && ns_word("10ns"));
        assert_eq!(stated_unit_tick_ns("the unit is 1 ns").map(|x| x.0), Some(1.0));
    }
}
