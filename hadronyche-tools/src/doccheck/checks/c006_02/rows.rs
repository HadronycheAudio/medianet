//! The worked-example table: numbers in cells (`NUM_RE`), the direction of
//! a row, and the arithmetic of the section's own rule, in exact integers.

use super::super::c004_15::num::is_word;
use super::super::c006_01::text::split_row;
use crate::doccheck::text::{has, rx};

/// A number read from a cell: an integer, or a decimal that is not (the
/// script's `Fraction`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Num {
    Int(i128),
    Frac(f64),
}

impl Num {
    fn int(self) -> Option<i128> {
        match self {
            Num::Int(n) => Some(n),
            Num::Frac(_) => None,
        }
    }
}

/// `floor((t - origin) * 12 / 125 000)`.
pub(crate) fn floor_to_position(t: i128, origin: i128) -> i128 {
    ((t - origin) * 12).div_euclid(125_000)
}

/// `ceil(p * 31 250 / 3)`.
pub(crate) fn ceil_to_offset(p: i128) -> i128 {
    -((-(p * 31_250)).div_euclid(3))
}

/// `NUM_RE` with its guards `(?<![\w.\-/])` and `(?![\w])`: the matches as
/// (sign, body without separators, byte end).
fn number_matches(cell: &str) -> Vec<(char, String)> {
    let chars: Vec<char> = cell.chars().collect();
    let prev_ok = |i: usize| i == 0 || !(is_word(chars[i - 1]) || matches!(chars[i - 1], '.' | '-' | '/'));
    let digits = |from: usize| chars[from.min(chars.len())..].iter().take_while(|c| c.is_ascii_digit()).count();
    let tail_ok = |e: usize| !chars.get(e).copied().is_some_and(is_word);
    let (mut out, mut i) = (Vec::new(), 0);
    while i < chars.len() {
        let signed = matches!(chars[i], '+' | '-') && chars.get(i + 1).is_some_and(char::is_ascii_digit);
        if !(signed || chars[i].is_ascii_digit()) || !prev_ok(i) {
            i += 1;
            continue;
        }
        let start = if signed { i + 1 } else { i };
        let run_end = start + digits(start);
        let mut ends = vec![run_end];
        let mut e = run_end;
        while chars.get(e) == Some(&' ') && digits(e + 1) >= 3 {
            e += 4;
            ends.push(e);
        }
        let mut found = None;
        for &e in ends.iter().rev() {
            if chars.get(e) == Some(&'.') && digits(e + 1) > 0 {
                let f = e + 1 + digits(e + 1);
                if tail_ok(f) {
                    found = Some(f);
                    break;
                }
            }
            if tail_ok(e) {
                found = Some(e);
                break;
            }
        }
        match found {
            Some(end) => {
                let body: String = chars[start..end].iter().filter(|c| **c != ' ').collect();
                out.push((if signed { chars[i] } else { '+' }, body));
                i = end;
            }
            None => i += 1,
        }
    }
    out
}

/// `cell_numbers`: every number of a cell with whether the cell says
/// "origin".
pub(crate) fn cell_numbers(cell: &str) -> Vec<(Num, bool)> {
    let rel = cell.to_lowercase().contains("origin");
    let mut out = Vec::new();
    for (sign, body) in number_matches(cell) {
        let neg = sign == '-';
        let val = if body.contains('.') {
            let f: f64 = body.parse().unwrap_or(0.0);
            if f.fract() == 0.0 {
                Num::Int(f as i128)
            } else {
                Num::Frac(if neg { -f } else { f })
            }
        } else {
            Num::Int(body.parse().unwrap_or(i128::MAX))
        };
        out.push((if neg { negate(val) } else { val }, rel));
    }
    if out.is_empty() && rel {
        out.push((Num::Int(0), true));
    }
    out
}

fn negate(n: Num) -> Num {
    match n {
        Num::Int(v) => Num::Int(-v),
        Num::Frac(f) => Num::Frac(-f),
    }
}

fn dir_cell(cells: &[String], pattern: &str) -> usize {
    cells.iter().position(|c| has(pattern, &c.to_lowercase())).unwrap_or(0)
}

/// A row's direction and the cell that names it.
pub(crate) fn row_direction(cells: &[String]) -> Option<(&'static str, usize)> {
    let text = cells.join(" | ").to_lowercase();
    let find = |w: &str| cells.iter().position(|c| c.to_lowercase().contains(w));
    if text.contains("position_at") && !text.contains("time_of") {
        return find("position_at").map(|k| ("position", k));
    }
    if text.contains("time_of") && !text.contains("position_at") {
        return find("time_of").map(|k| ("time", k));
    }
    let to_pos = r"(ns|network time|time)\s*(?:\u{2192}|->|=>)\s*(p\b|pos|position|sample)";
    let to_time = r"(p\b|pos|position|sample)\s*(?:\u{2192}|->|=>)\s*(ns|network time|time)";
    let words_pos = r"ns\s+to\s+(p\b|position)|time\s+to\s+position";
    let words_time = r"position\s+to\s+(ns|time)|p\b\s+to\s+ns";
    for (pat, dir) in [(to_pos, "position"), (to_time, "time"), (words_pos, "position"), (words_time, "time")] {
        if has(pat, &text) {
            return Some((dir, dir_cell(cells, pat)));
        }
    }
    None
}

/// `header_direction`.
pub(crate) fn header_direction(header: &[String]) -> Option<&'static str> {
    let text = header.join(" ").to_lowercase();
    if text.contains("position_at") && !text.contains("time_of") {
        return Some("position");
    }
    if text.contains("time_of") && !text.contains("position_at") {
        return Some("time");
    }
    let (time_re, pos_re) = (rx(r"network\s*time|time\s*\(ns\)|\bns\b"), rx(r"position|\bp\b|samples?"));
    let (t, p) = (time_re.find(&text)?, pos_re.find(&text)?);
    Some(if t.start() < p.start() { "position" } else { "time" })
}

fn matches_position(inv: Num, in_rel: bool, out: Num, origin: i128) -> bool {
    let (Some(i), Some(p)) = (inv.int(), out.int()) else { return false };
    if p < 0 {
        return false;
    }
    let choices: Vec<i128> = if in_rel { vec![origin + i] } else { vec![i, origin + i] };
    choices.iter().any(|&t| t - origin >= 0 && floor_to_position(t, origin) == p)
}

fn matches_time(inv: Num, in_rel: bool, out: Num, origin: i128) -> bool {
    let (Some(p), Some(o)) = (inv.int(), out.int()) else { return false };
    if p < 0 {
        return false;
    }
    let off = ceil_to_offset(p);
    let choices: Vec<i128> = if in_rel { vec![origin + off] } else { vec![off, origin + off] };
    choices.contains(&o)
}

/// A verified worked-example row.
#[derive(Debug, Clone)]
pub(crate) struct Verified {
    pub direction: &'static str,
    pub p: i128,
    pub output: i128,
    pub relative_out: bool,
}

type Item = (Num, bool);

/// A table of the section: header cells and its rows (line, cells).
pub(crate) type SecTable = (Vec<String>, Vec<(usize, Vec<String>)>);

/// `is_separator`.
fn is_separator(cells: &[String]) -> bool {
    !cells.is_empty() && cells.iter().all(|c| has(r"^(?:\:?-{2,}\:?)$", &c.replace(' ', "")))
}

/// `visible_lines` and `parse_tables` over the section's lines.
pub(crate) fn tables(lines: &[String]) -> Vec<SecTable> {
    let (mut vis, mut open) = (vec![true; lines.len()], None::<char>);
    for (i, line) in lines.iter().enumerate() {
        if let Some(m) = rx(r"^\s*(`{3,}|~{3,})").captures(line) {
            let c = m[1].chars().next().unwrap_or('`');
            match open {
                None => open = Some(c),
                Some(o) if o == c => open = None,
                Some(_) => {}
            }
            vis[i] = false;
        } else if open.is_some() {
            vis[i] = false;
        }
    }
    let (mut out, mut i) = (Vec::new(), 0);
    while i < lines.len() {
        if vis[i] && lines[i].trim_start().starts_with('|') {
            let mut block = Vec::new();
            let mut j = i;
            while j < lines.len() && vis[j] && lines[j].trim_start().starts_with('|') {
                if let Some(cells) = split_row(&lines[j]) {
                    block.push((j + 1, cells));
                }
                j += 1;
            }
            if block.len() >= 3 && is_separator(&block[1].1) {
                out.push((block[0].1.clone(), block[2..].to_vec()));
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

fn first_match(nums: &[(Num, bool, String)], dir: &str, origin: i128) -> Option<(Item, Item)> {
    for a in 0..nums.len() {
        for b in a + 1..nums.len() {
            for (x, y) in [(&nums[a], &nums[b]), (&nums[b], &nums[a])] {
                let good = if dir == "position" {
                    matches_position(x.0, x.1, y.0, origin)
                } else {
                    matches_time(x.0, x.1, y.0, origin)
                };
                if good {
                    return Some(((x.0, x.1), (y.0, y.1)));
                }
            }
        }
    }
    None
}

fn show(n: Num) -> String {
    match n {
        Num::Int(v) => v.to_string(),
        Num::Frac(f) => f.to_string(),
    }
}

/// `verify_table_rows`: (verified rows, problems).
pub(crate) fn verify(tables: &[SecTable], origin: i128) -> (Vec<Verified>, Vec<String>) {
    let (mut verified, mut problems) = (Vec::new(), Vec::new());
    for (header, rows) in tables {
        let table_dir = header_direction(header);
        for (line, cells) in rows {
            if cells.iter().all(|c| c.trim().is_empty()) {
                continue;
            }
            let (direction, dir_idx) = match row_direction(cells) {
                Some((d, k)) => (Some(d), Some(k)),
                None => (table_dir, None),
            };
            let mut nums: Vec<(Num, bool, String)> = Vec::new();
            for (k, c) in cells.iter().enumerate().filter(|(k, _)| Some(*k) != dir_idx) {
                let _ = k;
                for (v, rel) in cell_numbers(c) {
                    if let Num::Frac(f) = v {
                        problems.push(format!("line {line}: value {f} is not an integer; the map produces whole nanoseconds and whole positions"));
                    }
                    nums.push((v, rel, c.trim().to_string()));
                }
            }
            let Some(direction) = direction else { continue };
            if nums.len() < 2 {
                problems.push(format!("line {line}: {direction} row has fewer than two numeric values (input and output): | {} |", cells.join(" | ")));
                continue;
            }
            let Some((x, y)) = first_match(&nums, direction, origin) else {
                problems.push(format!("line {line}: {direction} row fails the section's own formula: input {}, output {}  (row: | {} |)", show(nums[0].0), show(nums[1].0), cells.join(" | ")));
                continue;
            };
            let (Some(xi), Some(yi)) = (x.0.int(), y.0.int()) else { continue };
            let p = if direction == "position" { yi } else { xi };
            let output = yi;
            verified.push(Verified { direction, p, output, relative_out: y.1 });
        }
    }
    (verified, problems)
}

