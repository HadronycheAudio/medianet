//! 004-22's reference rules: heading references, links, rule numbers.

use std::collections::BTreeSet;

use crate::doccheck::Docs;
use crate::doccheck::docs::dirname;
use crate::doccheck::links::{Dangling, links, resolve};
use crate::doccheck::markdown::{Markdown, norm_title};
use crate::doccheck::text::{rx, strip_links};

/// `RULE_RE`: a first cell "Rule N" or "N" (bold allowed), N of 1 or 2
/// digits.
pub fn rule_number(cell: &str) -> Option<u32> {
    rx(r"(?i)^\**\s*(?:rule\s*)?(\d{1,2})\s*\**$")
        .captures(cell.trim())
        .and_then(|c| c[1].parse().ok())
}

/// `reference_resolves`: a plain heading name, "Section, Subsection" or a
/// backticked heading of the document, looked for in parenthesised groups
/// (split at `;`), in backtick spans and in the `;`-separated parts of the
/// cell, compared after [`norm_title`] with and without section numbers.
pub fn reference_resolves(cell: &str, norms: &BTreeSet<String>) -> bool {
    let text = strip_links(cell);
    let mut segs: Vec<String> = Vec::new();
    for c in rx(r"\(([^()]*)\)").captures_iter(&text) {
        segs.extend(c[1].split(';').map(String::from));
    }
    segs.extend(rx(r"`([^`]+)`").captures_iter(&text).map(|c| c[1].to_string()));
    segs.extend(text.split(';').map(String::from));
    segs.iter().any(|seg| {
        let s = seg.trim().trim_matches(['.', ',']);
        if s.is_empty() {
            return false;
        }
        if norms.contains(&norm_title(s)) {
            return true;
        }
        let parts: Vec<&str> = s.split(',').map(str::trim).collect();
        s.contains(',') && parts.len() >= 2 && parts.iter().all(|p| norms.contains(&norm_title(p)))
    })
}

/// `check_links`: the dangling-link messages of `text`, written in a file
/// of directory `base`, whose own anchors are those of `this` (the script
/// always passes wire.md here, also for a separate record file).
pub fn dangling(docs: &Docs, text: &str, base: &str, this: &Markdown, this_path: &str, place: &str) -> Vec<String> {
    let slugs = this.slugs();
    let mut out = Vec::new();
    for l in links(text) {
        let what = match resolve(docs, base, &slugs, &l) {
            Ok(_) => continue,
            Err(Dangling::NoFile(_)) => "names a file that does not exist".to_string(),
            Err(Dangling::NoAnchor(p)) => {
                let p = if p.is_empty() { this_path.to_string() } else { p };
                format!("names an anchor that is not a heading of {p}")
            }
        };
        out.push(format!("{place}: link [{}]({}) {what}", l.text, l.target));
    }
    out
}

/// The directory of a repository-relative path.
pub fn base_of(path: &str) -> &str {
    dirname(path)
}

/// Python's `repr` of a list of optional integers: `[16, None]`.
pub fn py_list(values: &[Option<i64>]) -> String {
    let items: Vec<String> = values
        .iter()
        .map(|v| v.map_or("None".to_string(), |n| n.to_string()))
        .collect();
    format!("[{}]", items.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_and_references() {
        assert_eq!(rule_number("Rule 7"), Some(7));
        assert_eq!(rule_number("**12**"), Some(12));
        assert_eq!(rule_number("Rule 123"), None);
        let md = Markdown::parse("## Media packet header\n### Field table\n## 3.7 Copies\n");
        let n = md.head_norms();
        assert!(reference_resolves("(Media packet header, Field table)", &n));
        assert!(reference_resolves("see `Copies`", &n));
        assert!(reference_resolves("x; Field table.", &n));
        assert!(!reference_resolves("Media packet, Field table", &n));
        assert_eq!(py_list(&[Some(16), None]), "[16, None]");
    }
}
