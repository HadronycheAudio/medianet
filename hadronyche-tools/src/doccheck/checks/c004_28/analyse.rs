//! `analyse`: the field tables of one subsection as the arithmetic tree of
//! 004-25 (shared, [`crate::doccheck::checks::c004_25::tree`]), its
//! section maximum, its ⌈M / F⌉ statements, its section count and its type
//! code; and 004-28's number helpers (`has_num2`, `exact_product_results`).

use crate::doccheck::Report;
use crate::doccheck::checks::c004_24::py::tables;
use crate::doccheck::checks::c004_25::tree::{Node, build_tree, cdiv, entry_nodes, lo_of, maxc, size_node, tile_node};
use crate::doccheck::checks::c004_25::types::type_code;
use crate::doccheck::checks::c004_27::{Field, fields_of, is_field_table};
use crate::doccheck::text::rx;

/// What `analyse` returns.
#[derive(Debug, Clone, Default)]
pub(crate) struct Analysis {
    pub text: String,
    pub sec_max: Option<i64>,
    pub nt: Option<i64>,
    pub root: Option<Node>,
    pub body_rel: bool,
    pub stmts: Vec<(i64, i64, i64)>,
    pub fl_all: Vec<Field>,
    pub code: Option<i64>,
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
    pub fn lists(&self) -> Vec<i64> {
        self.root.as_ref().map(|r| r.rows.iter().filter_map(|x| maxc(&x.f)).collect()).unwrap_or_default()
    }
}

/// `STMT.findall(text)`.
pub(crate) fn statements(text: &str) -> Vec<(i64, i64, i64)> {
    rx(r"(?i)(?:⌈|ceil\s*\()\s*(\d+)\s*/\s*(\d+)\s*(?:⌉|\))\s*=\s*(\d+)")
        .captures_iter(text)
        .filter_map(|c| Some((c[1].parse().ok()?, c[2].parse().ok()?, c[3].parse().ok()?)))
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
    tile_node(&mut root, first.off * 8 + lo_of(&first).unwrap_or(0), title, label, report);
    let rbits = size_node(&mut root, title, report);
    out.sec_max = Some(58 + cdiv(rbits, 8));
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

/// True when `c` is a word character (Python's, the text having been made
/// to agree by `py_text`).
fn word(c: char) -> bool {
    let mut b = [0u8; 4];
    rx(r"^\w$").is_match(c.encode_utf8(&mut b))
}

/// `has_num2(text, n)`: `n` as a whole token — not after a word character
/// (or, for a non-negative `n`, a `-` or `.`), not before a digit (or a
/// `.digit`).
pub(crate) fn has_num2(text: &str, n: i64) -> bool {
    let s = n.to_string();
    text.match_indices(&s).any(|(at, _)| {
        let prev = text[..at].chars().next_back();
        let rest = &text[at + s.len()..];
        let mut next = rest.chars();
        let (n1, n2) = (next.next(), next.next());
        let after_digit = n1.is_some_and(|c| c.is_ascii_digit() || (c.is_numeric() && rx(r"^\d$").is_match(&c.to_string())));
        if n < 0 {
            !prev.is_some_and(word) && !after_digit
        } else {
            let before = prev.is_some_and(|p| word(p) || p == '-' || p == '.');
            let dot_digit = n1 == Some('.') && n2.is_some_and(|c| rx(r"^\d$").is_match(&c.to_string()));
            !before && !after_digit && !dot_digit
        }
    })
}

/// `exact_product_results(text)`.
pub(crate) fn exact_product_results(text: &str) -> Vec<i128> {
    let mut out = Vec::new();
    for c in rx(r"((?:\d+\s*(?:×|\bx\b|\*)\s*)+\d+)\s*=\s*(\d+)").captures_iter(text) {
        let mut prod: Option<i128> = Some(1);
        for fac in rx(r"\s*(?:×|\bx\b|\*)\s*").split(c[1].trim()) {
            prod = prod.zip(fac.parse::<i128>().ok()).and_then(|(a, b)| a.checked_mul(b));
        }
        let r: Option<i128> = c[2].parse().ok();
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
