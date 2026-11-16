//! Durations and bump rates read from the analysis text: the script's tiered
//! patterns, in floating point as the script's Python floats.

use crate::doccheck::checks::c004_15::num::{guarded, is_word, window};
use crate::doccheck::text::rx;

pub(crate) const MODULUS: f64 = 65_536.0;
pub(crate) const TOL: f64 = 0.30;

const UNIT_ALT: &str = r"(?:ms|milliseconds?|seconds?|secs?|minutes?|mins?|hours?|hrs?|days?|weeks?|months?|years?|min|h|hr|s)\b";
const NUM: &str = r"(\d[\d,\u{a0}\u{202f} ]*(?:\.\d+)?)";
const BUMP: &str = r"(?:bumps?|changes?|switches?|epochs?|events?)";

/// `unit_seconds`.
fn unit_seconds(u: &str) -> Option<f64> {
    let k = u.to_lowercase();
    Some(match k.trim_end_matches('.') {
        "ms" | "millisecond" | "milliseconds" => 0.001,
        "s" | "sec" | "secs" | "second" | "seconds" => 1.0,
        "min" | "mins" | "minute" | "minutes" => 60.0,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3600.0,
        "day" | "days" => 86400.0,
        "week" | "weeks" => 604800.0,
        "month" | "months" => 2629800.0,
        "year" | "years" => 31557600.0,
        _ => return None,
    })
}

fn parse_number(s: &str) -> Option<f64> {
    s.chars().filter(|c| !matches!(c, ',' | '\u{a0}' | '\u{202f}' | ' ')).collect::<String>().parse().ok()
}

fn interval_tiers() -> Vec<Vec<String>> {
    vec![
        vec![format!(r"(?i){BUMP}[\w\s]{{0,30}}?(?:every|per)\s+(?:(?:one|a|1)\s+)?{NUM}\s*({UNIT_ALT})")],
        vec![format!(r"(?i)(?:every|per)\s+(?:(?:one|a|1)\s+)?{NUM}\s*({UNIT_ALT})")],
        vec![
            format!(r"(?i){NUM}\s*({UNIT_ALT})\s*(?:of\s+(?:network\s+)?time\s+)?per\s+(?:timeline\s+)?epoch\s+(?:bump|change|switch)"),
            format!(r"(?i){NUM}\s*({UNIT_ALT})\s*per\s+(?:epoch\s+)?bump"),
            format!(r"(?i)65[\d,\u{{a0}}\u{{202f}} ]*536\s*[\u{{d7}}x*]\s*{NUM}\s*({UNIT_ALT})"),
        ],
    ]
}

fn rate_tiers() -> Vec<Vec<String>> {
    vec![
        vec![format!(r"(?i){NUM}\s*{BUMP}\s*(?:/\s*)?per\s*({UNIT_ALT})")],
        vec![format!(r"(?i)rate\s+of\s+{NUM}\s*per\s*({UNIT_ALT})")],
    ]
}

/// `first_match`: the first tier pattern that matches with a usable value:
/// (seconds, matched text).
fn first_match(tiers: &[Vec<String>], text: &str) -> Option<(f64, String)> {
    for pats in tiers {
        for pat in pats {
            let Some(m) = rx(pat).captures(text) else { continue };
            let (Some(v), Some(scale)) = (parse_number(&m[1]), unit_seconds(&m[2])) else { continue };
            if v <= 0.0 {
                continue;
            }
            return Some((v * scale, m[0].to_string()));
        }
    }
    None
}

/// `stated_durations`: (seconds, character position).
pub(crate) fn durations(text: &str) -> Vec<(f64, usize)> {
    let re = rx(&format!(r"(?i){NUM}\s*({UNIT_ALT})"));
    guarded(re, text, |c| is_word(c) || c == '.')
        .iter()
        .filter_map(|m| {
            let (v, s) = (parse_number(&m[1])?, unit_seconds(&m[2])?);
            Some((v * s, char_pos(text, m.get(0).map_or(0, |w| w.start()))))
        })
        .collect()
}

/// The character index of a byte offset.
pub(crate) fn char_pos(text: &str, byte: usize) -> usize {
    text[..byte].chars().count()
}

/// A candidate rate.
#[derive(Debug, Clone)]
pub(crate) struct Cand {
    pub interval: f64,
    pub matched: String,
    pub pos: usize,
}

/// `rate_candidates`: for each marker, the rate stated after it.
pub(crate) fn candidates(marker: &str, text: &str) -> Vec<Cand> {
    let (it, rt) = (interval_tiers(), rate_tiers());
    let mut out = Vec::new();
    for m in rx(&format!("(?i){marker}")).find_iter(text) {
        let tail = &text[m.end()..];
        let (interval, matched) = match first_match(&it, tail) {
            Some(x) => x,
            None => match first_match(&rt, tail) {
                Some((rate, matched)) => (1.0 / rate, matched),
                None => continue,
            },
        };
        out.push(Cand { interval, matched, pos: char_pos(text, m.start()) });
    }
    out
}

/// `pairs_with_stated_wrap`: a stated duration within 700 characters
/// before and 900 after the marker (the script's two pools; the second
/// contains the first) agrees with 65 536 x interval within 30 %.
pub(crate) fn pairs(c: &Cand, durs: &[(f64, usize)]) -> Option<f64> {
    let computed = MODULUS * c.interval;
    let pos = c.pos as i64;
    durs.iter()
        .filter(|(_, p)| pos - 700 <= *p as i64 && *p as i64 <= pos + 900)
        .any(|(s, _)| (s - computed).abs() <= TOL * computed)
        .then_some(computed)
}

/// A window of the text in characters.
pub(crate) fn around(text: &str, m: regex::Match, n: usize) -> String {
    window(text, m.start(), m.end(), n, n)
}
