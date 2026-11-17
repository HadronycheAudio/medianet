//! The wire-size subsection and its units: prose paragraphs and table rows
//! (each row carries its caption and header in its text), and the
//! equations written in them.

use super::num::{Dec, fl, mbit_list, nrm, to_int};
use crate::doccheck::family004::heading_index;
use crate::doccheck::markdown::code_mask;
use crate::doccheck::policy::FencePolicy;
use crate::doccheck::pycompat::{PyStr, splitlines};
use crate::doccheck::pynum::{PyNum, eval_arith, py_float};
use crate::doccheck::report::{Flow, Report, crash};
use crate::doccheck::tables::{caption_above, is_separator_line, split_row};
use crate::doccheck::text::{full, has, has_i, rx};

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

fn flush(buf: &mut Vec<String>, units: &mut Vec<Unit>) {
    if !buf.is_empty() {
        let text = nrm(&buf.join(" "));
        units.push(Unit {
            row: false,
            own: text.clone(),
            text,
            cells: Vec::new(),
            header: Vec::new(),
        });
        buf.clear();
    }
}

fn table_units(block: &[&str], cap: &str, units: &mut Vec<Unit>) {
    if !(block.len() >= 2 && is_separator_line(block[1])) {
        for r in block {
            let text = nrm(r);
            units.push(Unit {
                row: false,
                own: text.clone(),
                text,
                cells: Vec::new(),
                header: Vec::new(),
            });
        }
        return;
    }
    let header = split_row(block[0]);
    for r in &block[2..] {
        let cells = split_row(r);
        let text = nrm(&format!(
            "{cap} {} {}",
            header.join(" | "),
            cells.join(" | ")
        ));
        let own = nrm(&format!("{} | {}", header.join(" | "), cells.join(" | ")));
        units.push(Unit {
            row: true,
            text,
            own,
            cells,
            header: header.clone(),
        });
    }
}

/// `build_units`.
fn build_units(ls: &[&str], fm: &[bool]) -> Vec<Unit> {
    let (mut units, mut buf) = (Vec::new(), Vec::new());
    let mut i = 0;
    while i < ls.len() {
        let s = ls[i].py_strip();
        if fm[i] || s.is_empty() || s.starts_with('#') {
            flush(&mut buf, &mut units);
            i += 1;
            continue;
        }
        if ls[i].py_lstrip().starts_with('|') {
            flush(&mut buf, &mut units);
            let mut j = i;
            while j < ls.len() && !fm[j] && ls[j].py_lstrip().starts_with('|') {
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
    let heads = heading_index(&lines);
    let sec = heads
        .iter()
        .position(|h| h.1 == 2 && full(r"(?i)size budget", &h.2))
        .map(|k| {
            (
                heads[k].0,
                heads[k + 1..]
                    .iter()
                    .find(|h| h.1 <= 2)
                    .map_or(lines.len(), |h| h.0),
            )
        });
    let first_after =
        sec.and_then(|(_, end)| heads.iter().find(|h| h.0 >= end && h.1 <= 2).map(|h| h.0));
    let found = heads.iter().find(|h| {
        has_i(
            r"wire[ -]size|capacity|recomput|bit[ -]rate|on[ -]the[ -]wire|on-wire",
            &h.2,
        ) && (sec.is_some_and(|(s, e)| s < h.0 && h.0 < e) || Some(h.0) == first_after)
    })?;
    let end = heads
        .iter()
        .find(|h| h.0 > found.0 && h.1 <= found.1)
        .map_or(lines.len(), |h| h.0);
    let sub: Vec<&str> = lines[found.0 + 1..end].to_vec();
    let fence = code_mask(&sub, FencePolicy::Backticks);
    let units = build_units(&sub, &fence);
    let sub_text = nrm(&units
        .iter()
        .map(|u| u.text.as_str())
        .collect::<Vec<_>>()
        .join(" "));
    let joined = nrm(&sub
        .iter()
        .zip(&fence)
        .filter(|(_, f)| !**f)
        .map(|(l, _)| *l)
        .collect::<Vec<_>>()
        .join(" "));
    Some(Sub {
        units,
        sub_text,
        joined,
    })
}

/// `mbits(u)`: the Mbit/s figures of a unit, and of the cells under a
/// header that names Mbit.
pub(crate) fn mbits(u: &Unit) -> Vec<Dec> {
    let mut out = mbit_list(&u.text);
    if u.row {
        for (h, c) in u.header.iter().zip(&u.cells) {
            if h.to_lowercase().contains("mbit")
                && let Some(m) = rx(r"(\d+)(?:\.(\d+))?").captures(c)
            {
                let text = m
                    .get(2)
                    .map_or(m[1].to_string(), |f| format!("{}.{}", &m[1], f.as_str()));
                if let Some(v) = py_float(&text) {
                    out.push((v, m.get(2).map_or(0, |f| f.as_str().chars().count())));
                }
            }
        }
    }
    out
}

/// One evaluated equation.
#[derive(Debug, Clone)]
pub(crate) struct Eq {
    pub terms: Vec<i128>,
    pub ops: Vec<String>,
    /// `rhs_scaled`: an int when the right side has no decimals and no
    /// scaling unit, else a float.
    pub rhs: PyNum,
    pub rhs_raw: PyNum,
    pub tol: f64,
    pub ok: bool,
    pub raw: String,
    pub lhs: i128,
}

const EQ_PAT: &str = r"(?<![\w.])(\d(?:[\d ]*\d)?(?:\s*[+\-−×x*]\s*\d(?:[\d ]*\d)?)+)\s*=\s*(\d(?:[\d ]*\d)?)(?:\.(\d+))?(?:\s*(Mbit/s|kbit/s|bit/s|bytes?|%))?";

/// `equations(text)`.
pub(crate) fn equations(text: &str) -> Flow<Vec<Eq>> {
    let mut out = Vec::new();
    for m in rx(EQ_PAT).captures_iter(text) {
        let whole = m.get(0).map_or(0, |w| w.start());
        let before = text[..whole].py_rstrip();
        if before
            .chars()
            .next_back()
            .is_some_and(|c| "+−-×*|".contains(c))
        {
            continue;
        }
        if m.get(4).map(|u| u.as_str()) == Some("%") {
            continue;
        }
        let lhs = m[1].py_strip();
        let (mut nums, mut ops, mut at) = (Vec::new(), Vec::<String>::new(), 0);
        for o in rx(r"\s*([+\-−×x*])\s*").captures_iter(lhs) {
            let w = o.get(0).map_or(0..0, |w| w.start()..w.end());
            nums.push(to_int(&lhs[at..w.start]));
            ops.push(o[1].to_string());
            at = w.end;
        }
        nums.push(to_int(&lhs[at..]));
        let Some(nums) = nums.into_iter().collect::<Option<Vec<i128>>>() else {
            return crash("beyond i128 (INT_LIMIT)");
        };
        let mut expr = nums[0].to_string();
        for (op, n) in ops.iter().zip(&nums[1..]) {
            expr.push_str(match op.as_str() {
                "+" => "+",
                "-" | "−" => "-",
                _ => "*",
            });
            expr.push_str(&n.to_string());
        }
        let lhs_val = match eval_arith(&expr) {
            Ok(PyNum::Int(v)) => v.abs(),
            Ok(PyNum::Float(_)) | Err(_) => return crash("eval of the left side"),
        };
        let Some(int_part) = to_int(&m[2]) else {
            return crash("beyond i128 (INT_LIMIT)");
        };
        let frac = m.get(3).map(|x| x.as_str());
        let rhs_raw = match frac {
            Some(f) => {
                let Some(fv) = py_float(&format!("0.{f}")) else {
                    return crash("float()");
                };
                PyNum::Float(fl(int_part) + fv)
            }
            None => PyNum::Int(int_part),
        };
        let unit = m.get(4).map_or("", |u| u.as_str());
        let scale: Option<f64> = match unit {
            "Mbit/s" => Some(1e6),
            "kbit/s" => Some(1e3),
            _ => None,
        };
        let dec = frac.map_or(0, |f| f.chars().count());
        let base = if dec > 0 {
            0.5 * 10f64.powf(-fl(i128::try_from(dec).unwrap_or_default()))
        } else if scale.is_some() {
            0.5
        } else {
            0.0
        };
        let (tol, rhs) = match (scale, rhs_raw) {
            (Some(s), r) => (base * s, PyNum::Float(r.as_f64() * s)),
            (None, r) => (base, r),
        };
        let ok = match rhs {
            PyNum::Int(r) => {
                lhs_val == r
                    || i128::try_from(lhs_val.abs_diff(r)).is_ok_and(|d| fl(d) <= tol + 1e-9)
            }
            PyNum::Float(r) => (fl(lhs_val) - r).abs() <= tol + 1e-9,
        };
        out.push(Eq {
            terms: nums,
            ops,
            rhs,
            rhs_raw,
            tol,
            ok,
            raw: m[0].to_string(),
            lhs: lhs_val,
        });
    }
    Ok(out)
}

/// Every equation of every unit (a row's own cells, a paragraph's text).
pub(crate) fn all_equations(sub: &Sub, f: &mut Report) -> Flow<Vec<Eq>> {
    let mut eqs = Vec::new();
    for u in &sub.units {
        eqs.extend(equations(&u.own)?);
    }
    for e in eqs.iter().filter(|e| !e.ok) {
        f.fail(
            "AC1",
            format!(
                "arithmetic does not hold: {:?} (the left side is {})",
                e.raw, e.lhs
            ),
        );
    }
    Ok(eqs)
}
