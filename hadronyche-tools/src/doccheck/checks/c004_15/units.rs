//! The wire-size subsection and its units: prose paragraphs and table rows
//! (each row carries its caption and header in its text), and the
//! equations written in them.

use super::num::{Dec, Fails, R, crash, dec_value, guarded, is_word, mbit_list, nrm, to_int};
use crate::doccheck::checks::c004_14::section::fence_mask;
use crate::doccheck::markdown::Markdown;
use crate::doccheck::tables::split_row;
use crate::doccheck::text::{has, has_i, rx, splitlines};

/// A unit: a prose paragraph or one data row of a table.
#[derive(Debug, Clone)]
pub(crate) struct Unit {
    pub row: bool,
    pub text: String,
    /// A row's own cells with its header (the caption excluded); for prose
    /// the same as `text`.
    pub own: String,
    pub cells: Vec<String>,
    pub header: Vec<String>,
}

/// The subsection and what is read from it.
#[derive(Debug)]
pub(crate) struct Sub {
    pub units: Vec<Unit>,
    pub sub_text: String,
    pub joined: String,
}

fn caption_above(ls: &[&str], fm: &[bool], i: usize) -> String {
    let mut got: Vec<String> = Vec::new();
    let mut k = i as isize - 1;
    while k >= 0 && i as isize - k <= 5 && got.len() < 2 {
        let line = ls[k as usize];
        let s = line.trim();
        if !s.is_empty() && !line.trim_start().starts_with('|') && !fm[k as usize] {
            got.push(s.trim_start_matches(['#', '*', ' ']).trim().to_string());
        }
        k -= 1;
    }
    got.reverse();
    got.join(" ")
}

fn flush(buf: &mut Vec<String>, units: &mut Vec<Unit>) {
    if !buf.is_empty() {
        let text = nrm(&buf.join(" "));
        units.push(Unit { row: false, own: text.clone(), text, cells: Vec::new(), header: Vec::new() });
        buf.clear();
    }
}

fn table_units(block: &[&str], cap: &str, units: &mut Vec<Unit>) {
    let is_table = block.len() >= 2 && block[1].contains('-') && rx(r"^\s*\|?[\s:|-]+\|?\s*$").is_match(block[1]);
    if !is_table {
        for r in block {
            let text = nrm(r);
            units.push(Unit { row: false, own: text.clone(), text, cells: Vec::new(), header: Vec::new() });
        }
        return;
    }
    let header = split_row(block[0]);
    for r in &block[2..] {
        let cells = split_row(r);
        let text = nrm(&format!("{cap} {} {}", header.join(" | "), cells.join(" | ")));
        let own = nrm(&format!("{} | {}", header.join(" | "), cells.join(" | ")));
        units.push(Unit { row: true, text, own, cells, header: header.clone() });
    }
}

/// `build_units`.
fn build_units(ls: &[&str], fm: &[bool]) -> Vec<Unit> {
    let (mut units, mut buf) = (Vec::new(), Vec::new());
    let mut i = 0;
    while i < ls.len() {
        let s = ls[i].trim();
        if fm[i] || s.is_empty() || s.starts_with('#') {
            flush(&mut buf, &mut units);
            i += 1;
            continue;
        }
        if ls[i].trim_start().starts_with('|') {
            flush(&mut buf, &mut units);
            let mut j = i;
            while j < ls.len() && !fm[j] && ls[j].trim_start().starts_with('|') {
                j += 1;
            }
            table_units(&ls[i..j], &caption_above(ls, fm, i), &mut units);
            i = j;
            continue;
        }
        if has(r"^([-*+]\s|\d+\.\s)", s) {
            flush(&mut buf, &mut units);
        }
        buf.push(s.to_string());
        i += 1;
    }
    flush(&mut buf, &mut units);
    units
}

/// Finds the subsection; `None` is the script's "no heading" failure.
pub(crate) fn read(wire: &str) -> Option<Sub> {
    let lines = splitlines(wire);
    let md = Markdown::parse(&lines.join("\n"));
    let heads = &md.heads;
    let size = heads.iter().position(|h| h.level == 2 && has_i(r"^(?:size budget)$", &h.text));
    let sec = size.map(|k| {
        (heads[k].line, heads[k + 1..].iter().find(|h| h.level <= 2).map_or(lines.len(), |h| h.line))
    });
    let first_after = sec.and_then(|(_, end)| heads.iter().find(|h| h.line >= end && h.level <= 2).map(|h| h.line));
    let title = rx(r"(?i)wire[ -]size|capacity|recomput|bit[ -]rate|on[ -]the[ -]wire|on-wire");
    let found = heads.iter().find(|h| {
        title.is_match(&h.text) && (sec.is_some_and(|(s, e)| s < h.line && h.line < e) || Some(h.line) == first_after)
    })?;
    let end = heads.iter().find(|h| h.line > found.line && h.level <= found.level).map_or(lines.len(), |h| h.line);
    let sub: Vec<&str> = lines[found.line + 1..end].to_vec();
    let fence = fence_mask(&sub);
    let units = build_units(&sub, &fence);
    let sub_text = nrm(&units.iter().map(|u| u.text.as_str()).collect::<Vec<_>>().join(" "));
    let joined = nrm(&sub.iter().zip(&fence).filter(|(_, f)| !**f).map(|(l, _)| *l).collect::<Vec<_>>().join(" "));
    Some(Sub { units, sub_text, joined })
}

/// `mbits(u)`: the Mbit/s figures of a unit, and of the cells under a
/// header that names Mbit.
pub(crate) fn mbits(u: &Unit) -> Vec<Dec> {
    let mut out = mbit_list(&u.text);
    if u.row {
        for (h, c) in u.header.iter().zip(&u.cells) {
            if h.to_lowercase().contains("mbit")
                && let Some(m) = rx(r"(\d+)(?:\.(\d+))?").captures(c) {
                    out.push(dec_value(&m[1], m.get(2).map(|x| x.as_str())));
                }
        }
    }
    out
}

/// One evaluated equation.
#[derive(Debug, Clone)]
pub(crate) struct Eq {
    pub terms: Vec<i64>,
    pub ops: Vec<String>,
    pub rhs: f64,
    pub rhs_raw: f64,
    pub tol: f64,
    pub ok: bool,
    pub raw: String,
    pub lhs: f64,
}

/// Evaluates `n0 op n1 op ...` with `*` before `+` and `-`.
fn eval(nums: &[i64], ops: &[&str]) -> i64 {
    let mut total = 0;
    let mut sign = 1;
    let mut term = nums[0];
    for (op, n) in ops.iter().zip(&nums[1..]) {
        match *op {
            "+" | "-" | "\u{2212}" => {
                total += sign * term;
                sign = if *op == "+" { 1 } else { -1 };
                term = *n;
            }
            _ => term *= *n,
        }
    }
    total + sign * term
}

const EQ_PAT: &str = r"(\d(?:[\d ]*\d)?(?:\s*[+\-\u{2212}\u{d7}x*]\s*\d(?:[\d ]*\d)?)+)\s*=\s*(\d(?:[\d ]*\d)?)(?:\.(\d+))?(?:\s*(Mbit/s|kbit/s|bit/s|bytes?|%))?";

/// `equations(text)`.
pub(crate) fn equations(text: &str) -> R<Vec<Eq>> {
    let mut out = Vec::new();
    for m in guarded(rx(EQ_PAT), text, |c| is_word(c) || c == '.') {
        let whole = m.get(0).map_or(0..0, |w| w.range());
        let before = text[..whole.start].trim_end();
        if before.chars().next_back().is_some_and(|c| "+\u{2212}-\u{d7}*|".contains(c)) {
            continue;
        }
        if m.get(4).map(|u| u.as_str()) == Some("%") {
            continue;
        }
        let lhs = m[1].trim();
        let split = rx(r"\s*([+\-\u{2212}\u{d7}x*])\s*");
        let mut nums = Vec::new();
        let mut ops: Vec<String> = Vec::new();
        let mut at = 0;
        for o in split.captures_iter(lhs) {
            let w = o.get(0).map_or(0..0, |w| w.range());
            nums.push(to_int(&lhs[at..w.start])?);
            ops.push(o[1].to_string());
            at = w.end;
        }
        nums.push(to_int(&lhs[at..])?);
        let norm_ops: Vec<&str> = ops.iter().map(|o| match o.as_str() {
            "\u{d7}" | "x" | "*" => "*",
            "\u{2212}" | "-" => "-",
            _ => "+",
        }).collect();
        let lhs_val = eval(&nums, &norm_ops).abs() as f64;
        let frac = m.get(3).map(|x| x.as_str());
        let rhs_raw = to_int(&m[2])? as f64 + frac.map_or(0.0, |f| format!("0.{f}").parse().unwrap_or(0.0));
        let unit = m.get(4).map_or("", |u| u.as_str());
        let scale = match unit {
            "Mbit/s" => 1e6,
            "kbit/s" => 1e3,
            _ => 1.0,
        };
        let dec = frac.map_or(0, str::len);
        let base = if dec > 0 { 0.5 * 10f64.powi(-(dec as i32)) } else if scale != 1.0 { 0.5 } else { 0.0 };
        let tol = base * scale;
        let rhs = rhs_raw * scale;
        out.push(Eq {
            terms: nums,
            ops,
            rhs,
            rhs_raw,
            tol,
            ok: (lhs_val - rhs).abs() <= tol + 1e-9,
            raw: m[0].to_string(),
            lhs: lhs_val,
        });
    }
    Ok(out)
}

/// Every equation of every unit (a row's own cells, a paragraph's text).
pub(crate) fn all_equations(sub: &Sub, f: &mut Fails) -> R<Vec<Eq>> {
    let mut eqs = Vec::new();
    for u in &sub.units {
        eqs.extend(equations(&u.own)?);
    }
    for e in eqs.iter().filter(|e| !e.ok) {
        f.add("AC1", format!("arithmetic does not hold: {:?} (the left side is {})", e.raw, e.lhs));
    }
    Ok(eqs)
}

/// Placeholder to keep `crash` linked for sibling modules.
#[allow(dead_code)]
pub(crate) fn never() -> R<()> {
    crash("unreachable")
}
