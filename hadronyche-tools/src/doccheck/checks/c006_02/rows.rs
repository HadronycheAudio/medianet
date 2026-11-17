//! The worked-example table: numbers in cells (`NUM_RE`), the direction of
//! a row, and the arithmetic of the section's own rule, in exact integers.

use crate::doccheck::checked::{add, mul, sub};
use crate::doccheck::markdown::Markdown;
use crate::doccheck::policy::{FencePolicy, TableRules};
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::pynum::{Fraction, py_float};
use crate::doccheck::text::{has, rx};

/// A number read from a cell: an integer, or a decimal that is not whole
/// (the script's `Fraction`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Num {
    Int(i128),
    Frac(Fraction),
}

impl Num {
    fn int(self) -> Option<i128> {
        match self {
            Num::Int(n) => Some(n),
            Num::Frac(_) => None,
        }
    }
}

/// `floor((t - origin) * 12 / 125 000)` (`//`).
pub(crate) fn floor_to_position(t: i128, origin: i128) -> i128 {
    mul(sub(t, origin), 12).div_euclid(125_000)
}

/// `ceil(p * 31 250 / 3)`: `-((-a) // b)`.
pub(crate) fn ceil_to_offset(p: i128) -> i128 {
    sub(0, sub(0, mul(p, 31_250)).div_euclid(3))
}

const NUM_RE: &str = r"(?<![\w.\-/])([+\-]?)(\d+(?:[   ]\d{3})*(?:\.\d+)?)(?![\w])";

/// `cell_numbers`: every number of a cell with whether the cell says
/// "origin".
pub(crate) fn cell_numbers(cell: &str) -> Vec<(Num, bool)> {
    let rel = cell.to_lowercase().contains("origin");
    let mut out = Vec::new();
    for m in rx(NUM_RE).captures_iter(cell) {
        let neg = &m[1] == "-";
        let body = m[2].replace('_', " ").replace(',', "");
        let body = rx(r"(?<=\d)[ ](?=\d{3}(?!\d))").replace_all(&body, "");
        let val = if body.contains('.') {
            let Some(f) = py_float(&body) else { continue };
            if f.fract() == 0.0 {
                match Fraction::from_f64(f) {
                    Some(fr) => Num::Int(fr.numerator()),
                    None => continue,
                }
            } else {
                match Fraction::from_f64(f) {
                    Some(fr) => Num::Frac(fr.limit_denominator(1_000_000)),
                    None => continue,
                }
            }
        } else {
            let Ok(n) = parse_int(&body) else { continue };
            Num::Int(n)
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
        Num::Frac(f) => Num::Frac(Fraction::new(-f.numerator(), f.denominator()).unwrap_or(f)),
    }
}

fn dir_cell(cells: &[String], pattern: &str) -> usize {
    cells
        .iter()
        .position(|c| has(pattern, &c.to_lowercase()))
        .unwrap_or_default()
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
    let to_pos = r"(ns|network time|time)\s*(?:→|->|=>)\s*(p\b|pos|position|sample)";
    let to_time = r"(p\b|pos|position|sample)\s*(?:→|->|=>)\s*(ns|network time|time)";
    let words_pos = r"ns\s+to\s+(p\b|position)|time\s+to\s+position";
    let words_time = r"position\s+to\s+(ns|time)|p\b\s+to\s+ns";
    for (pat, dir) in [
        (to_pos, "position"),
        (to_time, "time"),
        (words_pos, "position"),
        (words_time, "time"),
    ] {
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
    let (time_re, pos_re) = (
        rx(r"network\s*time|time\s*\(ns\)|\bns\b"),
        rx(r"position|\bp\b|samples?"),
    );
    let (t, p) = (time_re.find(&text)?, pos_re.find(&text)?);
    Some(if t.start() < p.start() {
        "position"
    } else {
        "time"
    })
}

fn matches_position(inv: Num, in_rel: bool, out: Num, origin: i128) -> bool {
    let (Some(i), Some(p)) = (inv.int(), out.int()) else {
        return false;
    };
    if p < 0 {
        return false;
    }
    let choices: Vec<i128> = if in_rel {
        vec![add(origin, i)]
    } else {
        vec![i, add(origin, i)]
    };
    choices
        .iter()
        .any(|&t| sub(t, origin) >= 0 && floor_to_position(t, origin) == p)
}

fn matches_time(inv: Num, in_rel: bool, out: Num, origin: i128) -> bool {
    let (Some(p), Some(o)) = (inv.int(), out.int()) else {
        return false;
    };
    if p < 0 {
        return false;
    }
    let off = ceil_to_offset(p);
    let choices: Vec<i128> = if in_rel {
        vec![add(origin, off)]
    } else {
        vec![off, add(origin, off)]
    };
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

/// 006-02's `visible_lines` and `parse_tables`: [`FencePolicy::SameChar`]
/// and [`TableRules::T006_02`] over the section's lines.
pub(crate) fn tables(lines: &[String]) -> Vec<SecTable> {
    let md = Markdown::parse_with(&lines.join("\n"), FencePolicy::SameChar);
    md.tables_with(md.all(), TableRules::T006_02)
        .into_iter()
        .filter_map(|t| {
            let rows = t
                .rows
                .iter()
                .map(|r| (r.line + 1, r.cells.clone()))
                .collect();
            Some((t.header?, rows))
        })
        .collect()
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
        Num::Frac(f) => format!("{}/{}", f.numerator(), f.denominator()),
    }
}

/// `verify_table_rows`: (verified rows, problems).
pub(crate) fn verify(tables: &[SecTable], origin: i128) -> (Vec<Verified>, Vec<String>) {
    let (mut verified, mut problems) = (Vec::new(), Vec::new());
    for (header, rows) in tables {
        let table_dir = header_direction(header);
        for (line, cells) in rows {
            if cells.iter().all(|c| c.py_strip().is_empty()) {
                continue;
            }
            let (direction, dir_idx) = match row_direction(cells) {
                Some((d, k)) => (Some(d), Some(k)),
                None => (table_dir, None),
            };
            let mut nums: Vec<(Num, bool, String)> = Vec::new();
            for (_, c) in cells
                .iter()
                .enumerate()
                .filter(|(k, _)| Some(*k) != dir_idx)
            {
                for (v, rel) in cell_numbers(c) {
                    if let Num::Frac(f) = v {
                        problems.push(format!("line {line}: value {}/{} is not an integer; the map produces whole nanoseconds and whole positions", f.numerator(), f.denominator()));
                    }
                    nums.push((v, rel, c.py_strip().to_string()));
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
            let (Some(xi), Some(yi)) = (x.0.int(), y.0.int()) else {
                continue;
            };
            let p = if direction == "position" { yi } else { xi };
            verified.push(Verified {
                direction,
                p,
                output: yi,
                relative_out: y.1,
            });
        }
    }
    (verified, problems)
}
