//! The arithmetic tree of a section type (`build_tree`, `tile_node`,
//! `size_node`, `entry_nodes`, `cdiv`) and the check of every product and
//! sum stated in the Regime stamp section. Private to 004-25: the library
//! has no arithmetic tree (004-28 carries the same functions; reported as
//! missing from the library). The sums go through the library's port of
//! Python's `eval` ([`eval_arith`]).

use crate::doccheck::Report;
use crate::doccheck::family004::Field;
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::pynum::{EvalError, eval_arith};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::{has, rx};

/// Python's `-(-a // b)` for a positive `b` (every divisor here is a list
/// count or 8).
pub(crate) fn cdiv(a: i128, b: i128) -> i128 {
    -((-a).div_euclid(b))
}

/// The list count of a row, when it is a list row (`f["maxc"]` truthy).
pub(crate) fn maxc(f: &Field) -> Option<i128> {
    f.maxc.filter(|&m| m != 0)
}

/// One row of a node: the field, its start bit (`_start`, set by
/// [`tile_node`]) and its entry table.
#[derive(Debug, Clone)]
pub(crate) struct TRow {
    pub f: Field,
    pub start: i128,
    pub child: Option<Node>,
}

/// A field table with its entry tables.
#[derive(Debug, Clone, Default)]
pub(crate) struct Node {
    pub rows: Vec<TRow>,
    pub size: i128,
}

fn node(i: usize, ft: &[Vec<Field>], ptr: &mut usize, used: &mut Vec<usize>) -> Node {
    let mut rows = Vec::new();
    for f in &ft[i] {
        let mut child = None;
        if maxc(f).is_some() && *ptr < ft.len() {
            let c = *ptr;
            *ptr += 1;
            used.push(c);
            child = Some(node(c, ft, ptr, used));
        }
        rows.push(TRow { f: f.clone(), start: 0, child });
    }
    Node { rows, size: 0 }
}

/// `build_tree`: the root (table 0) and the indices of the tables no list
/// row takes.
pub(crate) fn build_tree(ft: &[Vec<Field>]) -> (Node, Vec<usize>) {
    let (mut ptr, mut used) = (1, vec![0]);
    let root = node(0, ft, &mut ptr, &mut used);
    let unused = (0..ft.len()).filter(|k| !used.contains(k)).collect();
    (root, unused)
}

/// `lo_of`: the low bit of a "bits [hi:lo]" meaning.
pub(crate) fn lo_of(f: &Field) -> Option<i128> {
    let c = rx(r"bits?\s*\[(\d+):(\d+)\]").captures(&f.meaning)?;
    let (a, b) = (parse_int(&c[1]).ok()?, parse_int(&c[2]).ok()?);
    Some(a.min(b))
}

/// `tile_node`: the rows tile with no gap or overlap; sets each row's start
/// and returns the end in bits.
pub(crate) fn tile_node(node: &mut Node, start_bits: i128, title: &str, label: &str, report: &mut Report) -> i128 {
    let mut pos = start_bits;
    for r in &mut node.rows {
        let f = &r.f;
        let s = match lo_of(f) {
            Some(lo) => f.off.saturating_mul(8).saturating_add(lo),
            None if f.bits.rem_euclid(8) != 0 => pos,
            None => f.off.saturating_mul(8),
        };
        if s != pos {
            report.fail(label, format!("the {title} table does not tile: a row at bit {s} follows bit {pos}"));
        }
        r.start = s;
        pos = s.saturating_add(f.bits.saturating_mul(maxc(f).unwrap_or(1)));
        if let Some(child) = &mut r.child {
            tile_node(child, 0, &format!("{title} (entry)"), label, report);
        }
    }
    pos
}

/// `size_node`: the size in bits at the declared maxima; checks that each
/// entry table sums to its list row's width.
pub(crate) fn size_node(node: &mut Node, title: &str, report: &mut Report) -> i128 {
    let mut total: i128 = 0;
    for r in &mut node.rows {
        match maxc(&r.f) {
            Some(m) => {
                if let Some(child) = &mut r.child {
                    let e = size_node(child, title, report);
                    child.size = e;
                    if e != r.f.bits {
                        report.fail("AC1", format!("the {title} entry table sums to {e} bits, but its list row declares {} bits per entry", r.f.bits));
                    }
                }
                total = total.saturating_add(r.f.bits.saturating_mul(m));
            }
            None => total = total.saturating_add(r.f.bits),
        }
    }
    node.size = total;
    total
}

/// `entry_nodes`: every entry table, depth-first.
pub(crate) fn entry_nodes(node: &Node) -> Vec<&Node> {
    let mut out = Vec::new();
    for r in &node.rows {
        if let Some(c) = &r.child {
            out.push(c);
            out.extend(entry_nodes(c));
        }
    }
    out
}

/// `prod != int(c)` for the products `a × b × … = c`: a product beyond
/// `i128` equals no `c` that fits it (a `c` beyond it is `INT_LIMIT`).
fn product_bad(expr: &str, c: &str) -> bool {
    let mut prod: Option<i128> = Some(1);
    for fac in rx(r"\s*(?:×|\bx\b|\*)\s*").split(expr) {
        let f = parse_int(fac).ok();
        prod = prod.zip(f).and_then(|(a, b)| a.checked_mul(b));
    }
    match parse_int(c) {
        Ok(c) => prod != Some(c),
        Err(_) => false,
    }
}

/// The products and sums of `text` that are not exact (the script's
/// "lesson 4" check); `Err` where the script's `eval` raises (it is not in
/// a `try`).
pub(crate) fn bad_arithmetic(text: &str) -> Flow<Vec<String>> {
    let mut bad = Vec::new();
    for c in rx(r"((?:\d+\s*(?:×|\bx\b|\*)\s*)+\d+)\s*=\s*(\d+)").captures_iter(text) {
        let expr = c[1].py_strip();
        if product_bad(expr, &c[2]) {
            bad.push(format!("{expr} = {}", &c[2]));
        }
    }
    for c in rx(r"([\d\s\x{d7}*+-]*\d[\d\s\x{d7}*+-]*)=\s*(\d+)").captures_iter(text) {
        let expr = &c[1];
        let stripped = expr.py_strip();
        if has(r"\+|-", stripped) && has(r"\d\s*(\+|-)\s*\d", expr) {
            let flat = rx(r"\s").replace_all(expr, "").replace('\u{d7}', "*");
            match eval_arith(&flat) {
                Ok(v) => {
                    if let Ok(n) = parse_int(&c[2])
                        && !v.eq_int(n) {
                            bad.push(format!("{stripped} = {}", &c[2]));
                        }
                }
                // beyond i128: INT_LIMIT is recorded, the verdict is red by rule
                Err(EvalError::IntLimit) => {}
                Err(e) => return crash(format!("{}: eval of {flat:?}", e.name())),
            }
        }
    }
    Ok(bad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn division_and_arithmetic() {
        assert_eq!(cdiv(7, 2), 4);
        assert_eq!(cdiv(8, 2), 4);
        assert_eq!(cdiv(0, 5), 0);
        assert_eq!(bad_arithmetic("64 × 1252 = 80128 and 58 + 1194 = 1252").ok(), Some(vec![]));
        assert_eq!(bad_arithmetic("2 x 3 = 7").ok().map(|v| v.len()), Some(1));
        assert_eq!(bad_arithmetic("1 + 1 = 3").ok().map(|v| v.len()), Some(1));
        assert!(bad_arithmetic("08 + 1 = 9").is_err(), "a leading zero is Python's SyntaxError");
    }
}
