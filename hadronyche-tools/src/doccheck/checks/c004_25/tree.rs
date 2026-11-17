//! The arithmetic tree of a section type (`build_tree`, `tile_node`,
//! `size_node`, `entry_nodes`, `cdiv`), shared verbatim by 004-25 and 004-28:
//! each list row ("E × max F") of a field table takes the next field table
//! of the subsection, in document order and depth-first, as its entry
//! table; rows tile (a bit row names its range "bit [hi:lo] of byte N"); a
//! list row's entry table sums to E bits; the size is computed at the
//! declared maxima. Also the scripts' check of every product and sum stated
//! in a section, whose sums go through Python's `eval`.

use crate::doccheck::Report;
use crate::doccheck::checks::c004_27::Field;
use crate::doccheck::text::{has, rx};

/// Python's `-(-a // b)` for a positive `b` (every divisor here is a list
/// count or 8).
pub(crate) fn cdiv(a: i64, b: i64) -> i64 {
    -((-a).div_euclid(b))
}

/// The list count of a row, when it is a list row (`f["maxc"]` truthy).
pub(crate) fn maxc(f: &Field) -> Option<i64> {
    f.maxc.filter(|&m| m != 0)
}

/// One row of a node: the field, its start bit (`_start`, set by
/// [`tile_node`]) and its entry table.
#[derive(Debug, Clone)]
pub(crate) struct TRow {
    pub f: Field,
    pub start: i64,
    pub child: Option<Node>,
}

/// A field table with its entry tables.
#[derive(Debug, Clone, Default)]
pub(crate) struct Node {
    pub rows: Vec<TRow>,
    pub size: i64,
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
pub(crate) fn lo_of(f: &Field) -> Option<i64> {
    let c = rx(r"bits?\s*\[(\d+):(\d+)\]").captures(&f.meaning)?;
    let (a, b): (i64, i64) = (c[1].parse().ok()?, c[2].parse().ok()?);
    Some(a.min(b))
}

/// `tile_node`: the rows tile with no gap or overlap; sets each row's start
/// and returns the end in bits.
pub(crate) fn tile_node(node: &mut Node, start_bits: i64, title: &str, label: &str, report: &mut Report) -> i64 {
    let mut pos = start_bits;
    for r in &mut node.rows {
        let f = &r.f;
        let s = match lo_of(f) {
            Some(lo) => f.off * 8 + lo,
            None if f.bits % 8 != 0 => pos,
            None => f.off * 8,
        };
        if s != pos {
            report.fail(label, format!("the {title} table does not tile: a row at bit {s} follows bit {pos}"));
        }
        r.start = s;
        pos = s + f.bits * maxc(f).unwrap_or(1);
        if let Some(child) = &mut r.child {
            tile_node(child, 0, &format!("{title} (entry)"), label, report);
        }
    }
    pos
}

/// `size_node`: the size in bits at the declared maxima; checks that each
/// entry table sums to its list row's width.
pub(crate) fn size_node(node: &mut Node, title: &str, report: &mut Report) -> i64 {
    let mut total = 0;
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
                total += r.f.bits * m;
            }
            None => total += r.f.bits,
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

/// The value of a Python expression over integers with `+ - * **` (what
/// the scripts' `eval` of a stated sum computes). `Err` is a `SyntaxError`
/// (which stops the script); `Ok(None)` a value that is not an integer or
/// does not fit (never equal to a stated integer here).
pub(crate) fn py_eval(expr: &str) -> Result<Option<i128>, ()> {
    let toks = tokens(expr)?;
    let mut p = Parser { toks: &toks, at: 0 };
    let v = p.expr()?;
    if p.at != toks.len() {
        return Err(());
    }
    Ok(v)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Num(String),
    Plus,
    Minus,
    Star,
    Pow,
}

fn tokens(s: &str) -> Result<Vec<Tok>, ()> {
    let c: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            '+' => out.push(Tok::Plus),
            '-' => out.push(Tok::Minus),
            '*' if c.get(i + 1) == Some(&'*') => {
                out.push(Tok::Pow);
                i += 1;
            }
            '*' => out.push(Tok::Star),
            d if d.is_ascii_digit() => {
                let start = i;
                while i + 1 < c.len() && c[i + 1].is_ascii_digit() {
                    i += 1;
                }
                let n: String = c[start..=i].iter().collect();
                if n.len() > 1 && n.starts_with('0') && n.chars().any(|x| x != '0') {
                    return Err(());
                }
                out.push(Tok::Num(n));
            }
            _ => return Err(()),
        }
        i += 1;
    }
    Ok(out)
}

struct Parser<'a> {
    toks: &'a [Tok],
    at: usize,
}

type Val = Result<Option<i128>, ()>;

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn expr(&mut self) -> Val {
        let mut v = self.term()?;
        while let Some(t) = self.peek().cloned() {
            if t != Tok::Plus && t != Tok::Minus {
                break;
            }
            self.at += 1;
            let r = self.term()?;
            v = match (v, r) {
                (Some(a), Some(b)) if t == Tok::Plus => a.checked_add(b),
                (Some(a), Some(b)) => a.checked_sub(b),
                _ => None,
            };
        }
        Ok(v)
    }

    fn term(&mut self) -> Val {
        let mut v = self.factor()?;
        while self.peek() == Some(&Tok::Star) {
            self.at += 1;
            let r = self.factor()?;
            v = v.zip(r).and_then(|(a, b)| a.checked_mul(b));
        }
        Ok(v)
    }

    fn factor(&mut self) -> Val {
        match self.peek() {
            Some(Tok::Plus) => {
                self.at += 1;
                self.factor()
            }
            Some(Tok::Minus) => {
                self.at += 1;
                Ok(self.factor()?.and_then(i128::checked_neg))
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Val {
        let base = match self.peek() {
            Some(Tok::Num(n)) => n.parse::<i128>().ok(),
            _ => return Err(()),
        };
        self.at += 1;
        if self.peek() != Some(&Tok::Pow) {
            return Ok(base);
        }
        self.at += 1;
        let exp = self.factor()?;
        Ok(match (base, exp) {
            (Some(b), Some(e)) if e >= 0 => u32::try_from(e).ok().and_then(|e| b.checked_pow(e)),
            _ => None,
        })
    }
}

/// The products and sums of `text` that are not exact (the scripts'
/// "lesson 4" check); `Err` where the script's `eval` raises.
pub(crate) fn bad_arithmetic(text: &str) -> Result<Vec<String>, ()> {
    let mut bad = Vec::new();
    for c in rx(r"((?:\d+\s*(?:×|\bx\b|\*)\s*)+\d+)\s*=\s*(\d+)").captures_iter(text) {
        let expr = c[1].trim();
        let mut prod: Option<i128> = Some(1);
        for fac in rx(r"\s*(?:×|\bx\b|\*)\s*").split(expr) {
            prod = prod.zip(fac.parse::<i128>().ok()).and_then(|(a, b)| a.checked_mul(b));
        }
        if prod != c[2].parse::<i128>().ok() || prod.is_none() {
            bad.push(format!("{expr} = {}", &c[2]));
        }
    }
    for c in rx(r"([\d\s\x{d7}*+-]*\d[\d\s\x{d7}*+-]*)=\s*(\d+)").captures_iter(text) {
        let expr = &c[1];
        if has(r"\+|-", expr.trim()) && has(r"\d\s*(\+|-)\s*\d", expr) {
            let flat = rx(r"\s").replace_all(expr, "").replace('\u{d7}', "*");
            let val = py_eval(&flat)?;
            if val.is_none() || val != c[2].parse::<i128>().ok() {
                bad.push(format!("{} = {}", expr.trim(), &c[2]));
            }
        }
    }
    Ok(bad)
}

/// 004-28's variant of [`bad_arithmetic`]: a sum Python's `eval` cannot
/// evaluate is reported as not exact (its `eval` is in a `try`), never a
/// stop.
pub(crate) fn bad_arithmetic_lenient(text: &str) -> Vec<String> {
    let mut bad = Vec::new();
    for c in rx(r"((?:\d+\s*(?:×|\bx\b|\*)\s*)+\d+)\s*=\s*(\d+)").captures_iter(text) {
        let expr = c[1].trim();
        let mut prod: Option<i128> = Some(1);
        for fac in rx(r"\s*(?:×|\bx\b|\*)\s*").split(expr) {
            prod = prod.zip(fac.parse::<i128>().ok()).and_then(|(a, b)| a.checked_mul(b));
        }
        if prod.is_none() || prod != c[2].parse::<i128>().ok() {
            bad.push(format!("{expr} = {}", &c[2]));
        }
    }
    for c in rx(r"([\d\s\x{d7}*+-]*\d[\d\s\x{d7}*+-]*)=\s*(\d+)").captures_iter(text) {
        let expr = &c[1];
        if has(r"\d\s*(\+|-)\s*\d", expr) {
            let flat = rx(r"\s").replace_all(expr, "").replace('\u{d7}', "*");
            match py_eval(&flat) {
                Ok(Some(v)) if Some(v) == c[2].parse::<i128>().ok() => {}
                _ => bad.push(format!("{} = {}", expr.trim(), &c[2])),
            }
        }
    }
    bad
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_division_and_eval() {
        assert_eq!(cdiv(7, 2), 4);
        assert_eq!(cdiv(8, 2), 4);
        assert_eq!(cdiv(0, 5), 0);
        assert_eq!(py_eval("58+1194"), Ok(Some(1252)));
        assert_eq!(py_eval("2*3+4"), Ok(Some(10)));
        assert_eq!(py_eval("5--3"), Ok(Some(8)));
        assert_eq!(py_eval("2**3"), Ok(Some(8)));
        assert_eq!(py_eval("-2**2"), Ok(Some(-4)));
        assert_eq!(py_eval("2**-1"), Ok(None));
        assert!(py_eval("08+1").is_err());
        assert_eq!(py_eval("00+1"), Ok(Some(1)));
        assert!(py_eval("5+").is_err());
        assert!(py_eval("*5").is_err());
        assert!(py_eval("2***3").is_err());
    }

    #[test]
    fn arithmetic_is_checked() {
        assert_eq!(bad_arithmetic("64 × 1252 = 80128 and 58 + 1194 = 1252"), Ok(vec![]));
        assert_eq!(bad_arithmetic("2 x 3 = 7").unwrap().len(), 1);
        assert_eq!(bad_arithmetic("1 + 1 = 3").unwrap().len(), 1);
    }
}
