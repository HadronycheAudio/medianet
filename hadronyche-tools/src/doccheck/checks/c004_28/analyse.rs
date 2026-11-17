//! `analyse`: the field tables of one subsection as the arithmetic tree
//! ([`super::tree`]), its section maximum, its ⌈M / F⌉ statements, its
//! section count and its type code; and 004-28's number helpers
//! (`has_num2`, `exact_product_results`). Every `int()` goes through the
//! library ([`parse_int`]: a value beyond `i128` is `INT_LIMIT`).

use super::tree::{Node, build_tree, cdiv, entry_nodes, lo_of, maxc, size_node, tile_node};
use crate::doccheck::Report;
use crate::doccheck::family004::{Field, fields_of, is_field_table, tables};
use crate::doccheck::pycompat::{PyStr, parse_int};
use crate::doccheck::text::{has, rx};

/// `TYPE_CODE_RE`: the first number after "type code", "code" or "section
/// type".
pub(crate) fn type_code(text: &str) -> Option<i128> {
    let c = rx(r"(?i)type code\D{0,20}?(\d+)|\bcode (\d+)\b|section type\D{0,20}?(\d+)\b").captures(text)?;
    (1..=3).find_map(|i| c.get(i)).and_then(|m| parse_int(m.as_str()).ok())
}

/// `SIZE_MAX_RE`: every "maximum … N bytes" figure.
pub(crate) fn stated_sizes(text: &str) -> Vec<i128> {
    rx(r"(?i)maximum[^.;]{0,120}?(\d+)\s*bytes").captures_iter(text).filter_map(|c| parse_int(&c[1]).ok()).collect()
}

/// What `analyse` returns.
#[derive(Debug, Clone, Default)]
pub(crate) struct Analysis {
    pub text: String,
    pub sec_max: Option<i128>,
    pub nt: Option<i128>,
    pub root: Option<Node>,
    pub body_rel: bool,
    pub stmts: Vec<(i128, i128, i128)>,
    pub fl_all: Vec<Field>,
    pub code: Option<i128>,
    pub ok: bool,
}

impl Analysis {
    /// `entries`: the entry tables, depth-first.
    pub fn entries(&self) -> Vec<&Node> {
        self.root.as_ref().map(entry_nodes).unwrap_or_default()
    }

    /// The rows of every entry table.
    pub fn entry_fields(&self) -> Vec<&Field> {
        self.entries().into_iter().flat_map(|n| n.rows.iter().map(|r| &r.f)).collect()
    }

    /// The list counts of the root's list rows.
    pub fn lists(&self) -> Vec<i128> {
        self.root.as_ref().map(|r| r.rows.iter().filter_map(|x| maxc(&x.f)).collect()).unwrap_or_default()
    }
}

/// `STMT.findall(text)`.
pub(crate) fn statements(text: &str) -> Vec<(i128, i128, i128)> {
    rx(r"(?i)(?:⌈|ceil\s*\()\s*(\d+)\s*/\s*(\d+)\s*(?:⌉|\))\s*=\s*(\d+)")
        .captures_iter(text)
        .filter_map(|c| Some((parse_int(&c[1]).ok()?, parse_int(&c[2]).ok()?, parse_int(&c[3]).ok()?)))
        .collect()
}

/// `analyse(title, text, label)`.
pub(crate) fn analyse(title: &str, text: &str, label: &str, report: &mut Report) -> Analysis {
    let mut out = Analysis { text: text.to_string(), ..Analysis::default() };
    let mut ft = Vec::new();
    for tb in tables(text).iter().filter(|t| is_field_table(t.head())) {
        let fl = fields_of(Some(tb), label, title, report);
        out.fl_all.extend(fl.iter().cloned());
        ft.push(fl);
    }
    if ft.first().is_none_or(Vec::is_empty) {
        report.fail(label, format!("the {title} has no field table of Offset, Width, Type, Meaning, Valid range"));
        return out;
    }
    let (mut root, unused) = build_tree(&ft);
    if !unused.is_empty() {
        report.fail(label, format!("the {title} has {} field table(s) that no list row names as its entry", unused.len()));
    }
    let first = root.rows[0].f.clone();
    out.body_rel = first.off != 58;
    if first.off != 0 && first.off != 58 {
        report.fail(label, format!("the {title} table starts at offset {}: a body starts at 0 and a section at 58", first.off));
    }
    tile_node(&mut root, first.off.saturating_mul(8).saturating_add(lo_of(&first).unwrap_or_default()), title, label, report);
    let rbits = size_node(&mut root, title, report);
    out.sec_max = Some(cdiv(rbits, 8).saturating_add(58));
    out.stmts = statements(text);
    let mut nt = 1;
    for f in root.rows.iter().filter_map(|r| maxc(&r.f)) {
        if let Some(s) = out.stmts.iter().find(|s| s.1 == f) {
            nt = nt.max(cdiv(s.0, f));
        }
    }
    out.nt = Some(nt);
    out.code = type_code(text);
    out.root = Some(root);
    out.ok = true;
    out
}

/// `has_num2(text, n)`: `n` as a whole token — not after a word character
/// (or, for a non-negative `n`, a `-` or `.`), not before a digit (or a
/// `.digit`); Python's classes (`²` is a word character, not a digit).
pub(crate) fn has_num2(text: &str, n: i128) -> bool {
    let s = n.to_string();
    if n < 0 {
        return has(&format!(r"(?<![\w]){}(?!\d)", regex::escape(&s)), text);
    }
    has(&format!(r"(?<![\w\-.]){s}(?!\d|\.\d)"), text)
}

/// `exact_product_results(text)`.
pub(crate) fn exact_product_results(text: &str) -> Vec<i128> {
    let mut out = Vec::new();
    for c in rx(r"((?:\d+\s*(?:×|\bx\b|\*)\s*)+\d+)\s*=\s*(\d+)").captures_iter(text) {
        let mut prod: Option<i128> = Some(1);
        for fac in rx(r"\s*(?:×|\bx\b|\*)\s*").split(c[1].py_strip()) {
            prod = prod.zip(parse_int(fac).ok()).and_then(|(a, b)| a.checked_mul(b));
        }
        let r: Option<i128> = parse_int(&c[2]).ok();
        if prod.is_some() && prod == r {
            out.push(r.unwrap_or_default());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_as_tokens() {
        assert!(has_num2("needs 71 sections", 71));
        assert!(!has_num2("C-17", 17));
        assert!(!has_num2("1.5", 5));
        assert!(!has_num2("5.0", 5));
        assert!(has_num2("is -324 here", -324));
        assert!(!has_num2("x-324", -324));
        assert_eq!(exact_product_results("2 × 3 = 6, 2 × 3 = 7"), vec![6]);
    }
}
