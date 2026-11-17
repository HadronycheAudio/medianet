//! The arithmetic tree of a section type (`build_tree`, `tile_node`,
//! `size_node`, `entry_nodes`, `cdiv`) and 004-28's check of the products
//! and sums of a subsection. Private to 004-28: the library has no
//! arithmetic tree (004-25 carries the same functions; reported as missing
//! from the library). The sums go through the library's port of Python's
//! `eval` ([`eval_arith`]); 004-28's `eval` is in a `try`, so a sum it
//! cannot evaluate is "not exact", never a stop.

use crate::doccheck::checked;
use crate::doccheck::Report;
use crate::doccheck::family004::Field;
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::pynum::{EvalError, eval_arith};
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
            Some(lo) => checked::add(checked::mul(f.off, 8), lo),
            None if f.bits.rem_euclid(8) != 0 => pos,
            None => checked::mul(f.off, 8),
        };
        if s != pos {
            report.fail(label, format!("the {title} table does not tile: a row at bit {s} follows bit {pos}"));
        }
        r.start = s;
        pos = checked::add(s, checked::mul(f.bits, maxc(f).unwrap_or(1)));
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
                total = checked::add(total, checked::mul(r.f.bits, m));
            }
            None => total = checked::add(total, r.f.bits),
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

/// 004-28's products and sums of `text` that are not exact.
pub(crate) fn bad_arithmetic_lenient(text: &str) -> Vec<String> {
    let mut bad = Vec::new();
    for c in rx(r"((?:\d+\s*(?:×|\bx\b|\*)\s*)+\d+)\s*=\s*(\d+)").captures_iter(text) {
        let expr = c[1].py_strip();
        let mut prod: Option<i128> = Some(1);
        for fac in rx(r"\s*(?:×|\bx\b|\*)\s*").split(expr) {
            prod = prod.zip(parse_int(fac).ok()).map(|(a, b)| checked::mul(a, b));
        }
        if parse_int(&c[2]).is_ok_and(|n| prod != Some(n)) {
            bad.push(format!("{expr} = {}", &c[2]));
        }
    }
    for c in rx(r"([\d\s×*+-]*\d[\d\s×*+-]*)=\s*(\d+)").captures_iter(text) {
        let expr = &c[1];
        if !has(r"\d\s*(\+|-)\s*\d", expr) {
            continue;
        }
        let flat = rx(r"\s").replace_all(expr, "").replace('×', "*");
        let exact = match eval_arith(&flat) {
            Ok(v) => parse_int(&c[2]).map_or(true, |n| v.eq_int(n)),
            // beyond i128: INT_LIMIT is recorded, the verdict is red by rule
            Err(EvalError::IntLimit) => true,
            Err(_) => false,
        };
        if !exact {
            bad.push(format!("{} = {}", expr.py_strip(), &c[2]));
        }
    }
    bad
}
