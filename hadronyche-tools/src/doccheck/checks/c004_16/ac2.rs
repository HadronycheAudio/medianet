//! AC2: the sizes, the stated figures and the verdict (script lines
//! 640-777; the falsified branch's fallback-name check is reported under
//! AC3, as the script does).

use crate::doccheck::wp7::common::split_sentences;
use super::ac1::Rows;
use crate::doccheck::wp7::md::{Field, clean_start};
use super::{Ctx, Layout, NUM, py, to_int};
use crate::doccheck::Report;

/// `re.sub(r"(?:(?<=\s)|^)'[^']*'(?=\s|[.,;:)]|$)|\"[^\"]*\"|“[^”]*”|`[^`]*`", " ", text)`.
pub(crate) fn unquote(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let close = |from: usize, q: char| chars[from..].iter().position(|&c| c == q).map(|k| from + k);
    let mut out = String::new();
    let mut p = 0;
    while p < n {
        let c = chars[p];
        let end = match c {
            '\'' if p == 0 || py::is_space(chars[p - 1]) => close(p + 1, '\'').filter(|&e| {
                let next = chars.get(e + 1);
                next.is_none_or(|&x| py::is_space(x) || ".,;:)".contains(x)) || (e + 2 == n && chars[e + 1] == '\n')
            }),
            '"' => close(p + 1, '"'),
            '\u{201c}' => close(p + 1, '\u{201d}'),
            '`' => close(p + 1, '`'),
            _ => None,
        };
        match end {
            Some(e) => {
                out.push(' ');
                p = e + 1;
            }
            None => {
                out.push(c);
                p += 1;
            }
        }
    }
    out
}

/// Matches of `pattern` (any case) not preceded by `not ` (`(?<!not )`).
fn count_not_after_not(pattern: &str, body: &str) -> usize {
    py::finditer(pattern, body, true)
        .iter()
        .filter(|h| {
            let before: Vec<char> = body[..h.start].chars().collect();
            let tail: String = before[before.len().saturating_sub(4)..].iter().collect();
            !(before.len() >= 4 && tail.eq_ignore_ascii_case("not "))
        })
        .count()
}

/// `classify(body)`: "fits", "falsified", "ambiguous" or None.
pub(crate) fn classify(body: &str) -> Option<&'static str> {
    let fits = count_not_after_not(r"\bfits\b", body)
        + py::finditer(r"\bC-17\s+(?:holds|stands)\b|\bdoes\s+not\s+hold\b|\bnot\s+falsified\b", body, true).len();
    let fals = count_not_after_not(r"\bfalsified\b", body)
        + py::finditer(r"\bdoes\s+not\s+fit\b|\bfalsifier\s+holds\b", body, true).len();
    match (fits > 0, fals > 0) {
        (true, true) => Some("ambiguous"),
        (true, false) => Some("fits"),
        (false, true) => Some("falsified"),
        _ => None,
    }
}

/// The bundles-per-hello maximum B and its rules.
fn bundles(ctx: &Ctx, lay: &[Field], rows: &Rows, report: &mut Report) -> Option<i128> {
    let b_text = py::search(r"at\s+most\s+(\d+)\s+bundles", &ctx.plain, true).and_then(|m| py::int(m.g(1)));
    if !rows.has_bundles {
        return b_text;
    }
    let b_max = match rows.bun_row.map(|i| &lay[i]) {
        Some(f) if f.maxc.is_some() => {
            if f.bits != rows.e_bytes.saturating_mul(8) {
                report.fail("AC2", format!("the bundle row is {} bits per bundle; the bundle element sums to {}", f.bits, rows.e_bytes * 8));
            }
            f.maxc
        }
        _ => {
            report.fail("AC2", "the hello layout's bundle row is not a counted row 'E × max B'");
            None
        }
    };
    if !rows.bun_count.is_some_and(|i| lay[i].bits == 8) {
        report.fail("AC2", "the hello layout has no one-byte bundle count row before the bundles");
    }
    match (b_text, b_max) {
        (None, _) => report.fail("AC2", "bundles are carried but the Hello section states no bundles-per-hello maximum ('at most N bundles')"),
        (Some(t), Some(b)) if t != b => report.fail("AC2", format!("the stated bundles-per-hello maximum is {t}; the bundle row says max {b}")),
        _ => {}
    }
    if b_max.is_some() {
        let more = r"(more|above|beyond|exceed\w*)\s+(than\s+)?(\d+|this|that)\s+bundles|bundles\s+(than|above|beyond)\b|source\s+with\s+more\s+bundles";
        let ok_rule = ctx.paras.iter().filter(|t| py::has_i(more, t)).any(|t| {
            py::has_i(r"\brule\b", t)
                || (clean_start(t).to_lowercase().starts_with("open item") && py::has_i("owner", t) && py::has_i("falsifier", t))
        });
        if !ok_rule {
            report.fail("AC2", "no rule or 'Open item' (owner and falsifier) says what a source with more bundles than the stated maximum does");
        }
    }
    b_max
}

/// The one-bundle figure as stated (`ONE_STATED`).
fn one_stated(ctx: &Ctx) -> Option<i128> {
    let one_rx = r"\b(?:one|single)\s+(?:32-member\s+bundle|bundle\s+of\s+32\s+members?|32-member\s+|bundle)\b";
    let flat = ctx.plain.replace('\n', " ");
    for sent in split_sentences(&flat) {
        if !py::has_i(one_rx, sent) {
            continue;
        }
        let m1 = py::search(&format!(r"=\s*({NUM})\s*bytes"), sent, false).or_else(|| py::search(&format!(r"({NUM})\s*bytes"), sent, false));
        if let Some(m) = m1 {
            return to_int(m.g(1));
        }
    }
    None
}

/// What the verdict part needs.
struct Figures {
    abs_all: Option<i128>,
    one: Option<i128>,
    stated_max: Option<i128>,
}

fn verdict(ctx: &Ctx, lay: &Layout, rows: &Rows, fig: &Figures, report: &mut Report) {
    let bound = ctx.k.bound;
    let p = ctx.paras.iter().find(|t| py::matches(r"\s*\*{0,2}\s*verdict\b", clean_start(t), true).is_some());
    let body = unquote(p.map_or("", |s| s.as_str()));
    let v = p.and_then(|_| classify(&body));
    match (p, v) {
        (None, _) => report.fail("AC2", "no 'Verdict' paragraph says whether the hello fits the control bound or is falsified"),
        (Some(_), None) => report.fail("AC2", "the verdict paragraph states neither 'fits' nor 'falsified' in its own words"),
        (Some(_), Some("ambiguous")) => report.fail("AC2", "the verdict paragraph says both 'fits' and 'falsified': state one verdict"),
        _ => {}
    }
    if p.is_some() && fig.one.is_some_and(|o| o <= bound) && py::has_i(r"C-17[^.;]*?(falsified|falsifier\s+holds|holds\s+for)", &body) {
        report.fail("AC2", format!("the verdict says C-17's falsifier holds, but that falsifier is about one 32-member bundle, which computes {} bytes (within {bound}): C-17 is not falsified", fig.one.map_or("None".to_string(), |o| o.to_string())));
    }
    let h = lay.h_bytes;
    match v {
        Some("fits") => {
            if !rows.has_bundles {
                report.fail("AC2", "the verdict says it fits but the hello layout carries no bundle rows");
            }
            if h > bound {
                report.fail("AC2", format!("the absolute maximum hello, with every stated maximum, is {h} bytes, above {bound} (C-17), but the verdict says it fits"));
            }
            if let Some(s) = fig.stated_max
                && s != h
            {
                report.fail("AC2", format!("the maximum hello size is stated as {s} bytes; the table sums to {h}"));
            }
        }
        Some("falsified") => falsified(ctx, lay, rows, fig, &body, p.map_or("", |s| s.as_str()), report),
        _ => {}
    }
}

fn falsified(ctx: &Ctx, lay: &Layout, rows: &Rows, fig: &Figures, body: &str, para: &str, report: &mut Report) {
    let (bound, h) = (ctx.k.bound, lay.h_bytes);
    if rows.has_bundles {
        report.fail("AC2", "the verdict says falsified but the layout still carries the bundle rows: the table must be the fallback");
    }
    match fig.abs_all {
        None => report.fail("AC2", "the all-bundles maximum cannot be computed: the bundle element or the bundles-per-hello maximum is missing"),
        Some(a) if a <= bound => report.fail("AC2", format!("the verdict says falsified but the all-bundles maximum is {a} bytes, not above {bound}")),
        _ => {}
    }
    if h > bound {
        report.fail("AC2", format!("the fallback hello is {h} bytes, above {bound}: the remaining message must fit"));
    }
    if let Some(s) = fig.stated_max
        && s != h
    {
        report.fail("AC2", format!("the maximum hello size is stated as {s} bytes; the fallback table sums to {h}"));
    }
    if let Some(a) = fig.abs_all {
        let stated: Vec<Option<i128>> = py::finditer(&format!(r"({NUM})\s*bytes"), &ctx.plain, false).iter().map(|m| to_int(m.g(1))).collect();
        if !stated.contains(&Some(a)) {
            report.fail("AC2", format!("the all-bundles size {a} bytes is not stated in bytes in the Hello section"));
        }
    }
    if py::has("C-17", body) && fig.one.is_none_or(|o| o <= bound) {
        report.fail("AC2", format!("the verdict says C-17 is falsified, but C-17's falsifier is about one 32-member bundle, which computes {} bytes (within {bound}): C-17 holds", fig.one.map_or("None".to_string(), |o| o.to_string())));
    }
    if !py::has_i("split|handshake", &format!("{para} {}", ctx.plain)) {
        report.fail("AC3", "the falsified verdict does not name the fallback (split hello or handshake)");
    }
}

/// AC2.
pub(crate) fn sizes(ctx: &Ctx, lay: &Layout, rows: &Rows, report: &mut Report) {
    let b_max = bundles(ctx, &lay.lay, rows, report);
    let non_bundle = lay
        .lay
        .iter()
        .filter(|f| py::matches(r"\s*bundle", &f.meaning, true).is_none())
        .map(Field::size_bits)
        .fold(0, i128::saturating_add)
        .div_euclid(8);
    let (mut abs_all, mut one) = (None, None);
    if let Some(b) = b_max
        && rows.e_bytes != 0
    {
        abs_all = Some(non_bundle.saturating_add(1).saturating_add(b.saturating_mul(rows.e_bytes)));
        one = Some(non_bundle.saturating_add(1).saturating_add(rows.e_bytes));
    }
    let k = &ctx.k;
    if let (Some(b), Some(s), Some(m)) = (b_max, k.streams_max, k.members_max)
        && s != 0
        && m != 0
    {
        let needed = -((-s).div_euclid(m));
        let excused = ctx.para_with(&[&format!(r"\b{s}\b"), "bundle", r"rule|open\s+item"]).is_some();
        if b < needed && !excused {
            report.fail("AC2", format!("at most {b} bundles per hello cannot carry C-09's {s} streams ({needed} bundles of {m}); no rule or open item says what a source with more does"));
        }
    }
    let stated_one = one_stated(ctx);
    if let Some(o) = one {
        match stated_one {
            None => report.fail("AC2", "the one-bundle figure is not stated: a sentence with 'one' or 'single', '32' and 'bundle' and the size in bytes (C-08, the case C-17's falsifier names)"),
            Some(s) if s != o => report.fail("AC2", format!("the one-bundle figure is stated as {s} bytes; the tables compute {o}")),
            _ => {}
        }
    }
    let stated_max = py::search(&format!(r"maximum\s+hello\s+size[^\n]*?({NUM})\s*bytes"), &ctx.plain, true).and_then(|m| to_int(m.g(1)));
    if stated_max.is_none() {
        report.fail("AC2", "the maximum hello size is not stated in bytes ('maximum hello size ... N bytes')");
    }
    let fig = Figures { abs_all, one, stated_max };
    verdict(ctx, lay, rows, &fig, report);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_quotes_and_verdicts() {
        assert_eq!(split_sentences("A b. C;  d.e f"), vec!["A b.", "C;", "d.e f"]);
        assert_eq!(unquote("It 'fits' here. 'not'x and \"fits\" `x`"), "It   here. 'not'x and    ");
        assert_eq!(classify("It fits."), Some("fits"));
        assert_eq!(classify("It does not fit: NOT fits."), Some("falsified"));
        assert_eq!(classify("C-17 holds, falsified"), Some("ambiguous"));
        assert_eq!(classify("nothing"), None);
    }
}
