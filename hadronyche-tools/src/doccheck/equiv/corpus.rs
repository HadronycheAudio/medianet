//! The structured mutation corpus: deterministic from a seed.
//!
//! For each target section the generator lists every mutation of seven
//! kinds — delete a table row, delete a unit (paragraph or list item),
//! delete a subsection, strip a tag, negate a keyword, alter a number,
//! rename a heading — then shuffles each with the seed, interleaves the
//! targets of a kind, and takes the kinds round-robin, so a limited corpus
//! still covers every kind and every target.

use super::super::Docs;
use super::super::checks::Target;
use super::super::markdown::Markdown;
use super::super::mutate::{Edit, Mutation};
use super::super::text::{head, rx};
use super::super::units::{UnitKind, units};

/// The default negations: (written, replaced by). The first group turns a
/// denial into an assertion, the second an assertion into a denial.
const NEGATIONS: &[(&str, &str)] = &[
    (" no ", " a "),
    (" not ", " "),
    (" never ", " "),
    (" none", " some"),
    (" is ", " is not "),
    (" are ", " are not "),
    (" has ", " has no "),
];

/// A small deterministic generator (xorshift64*).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = (self.next() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }
}

fn short(file: &str) -> &str {
    file.rsplit('/').next().unwrap_or(file)
}

fn mk(file: &str, edit: Edit, what: String) -> Mutation {
    Mutation {
        file: file.to_string(),
        edit,
        label: what,
    }
}

/// Every mutation of the seven kinds in one target, grouped by kind.
pub fn candidates(docs: &Docs, target: &Target, keywords: &[&str]) -> Vec<Vec<Mutation>> {
    let Some(text) = docs.read(target.file) else {
        return Vec::new();
    };
    let md = Markdown::parse(&text);
    let span = match target.heading {
        Some(p) => match md.find(p).into_iter().next() {
            Some(h) => md.section(h),
            None => return Vec::new(),
        },
        None => md.all(),
    };
    let f = target.file;
    let name = short(f);
    let mut kinds: Vec<Vec<Mutation>> = vec![Vec::new(); 7];
    for t in md.tables(span) {
        for r in &t.rows {
            let what = format!(
                "{name}:{} delete row `{}`",
                r.line + 1,
                head(r.joined().trim(), 50)
            );
            kinds[0].push(mk(
                f,
                Edit::DeleteLines {
                    lo: r.line,
                    hi: r.line,
                },
                what,
            ));
        }
    }
    let body = md.text(span);
    for u in units(&body) {
        if matches!(u.kind, UnitKind::Paragraph | UnitKind::ListItem) {
            let (lo, hi) = (u.lo + span.lo, u.hi + span.lo);
            let what = format!("{name}:{} delete unit `{}`", lo + 1, head(&u.text, 50));
            kinds[1].push(mk(f, Edit::DeleteLines { lo, hi }, what));
        }
    }
    for h in md.heads_in(span) {
        let s = md.section(h);
        let what = format!(
            "{name}:{} delete section `{}`",
            h.line + 1,
            head(&h.text, 50)
        );
        kinds[2].push(mk(
            f,
            Edit::DeleteLines {
                lo: s.lo,
                hi: s.hi - 1,
            },
            what,
        ));
        let to = format!("{} Renamed {}", "#".repeat(h.level), h.text);
        let what = format!(
            "{name}:{} rename heading `{}`",
            h.line + 1,
            head(&h.text, 50)
        );
        let len = md.lines[h.line].len();
        kinds[6].push(mk(
            f,
            Edit::ReplaceSpan {
                line: h.line,
                start: 0,
                end: len,
                to,
            },
            what,
        ));
    }
    line_mutations(&md, span, target, keywords, &mut kinds);
    kinds
}

fn line_mutations(
    md: &Markdown,
    span: super::super::markdown::Span,
    target: &Target,
    keywords: &[&str],
    kinds: &mut [Vec<Mutation>],
) {
    let f = target.file;
    let name = short(f);
    let tag = rx(r"\[(code|measured|kernel-doc|standard|memory|decision)\]");
    let number = rx(r"\b\d+\b");
    let extra: Vec<(String, String)> = keywords
        .iter()
        .map(|k| (k.to_string(), format!("not {k}")))
        .collect();
    let negations = NEGATIONS
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .chain(extra);
    let negations: Vec<(String, String)> = negations.collect();
    for i in span.lo..span.hi {
        if md.in_code[i] || md.lines[i].trim_start().starts_with('#') {
            continue;
        }
        let line = &md.lines[i];
        let at = i + 1;
        if let Some(c) = tag.captures(line) {
            let what = format!("{name}:{at} strip tag [{}]", &c[1]);
            kinds[3].push(mk(
                f,
                Edit::StripTag {
                    line: i,
                    tag: c[1].to_string(),
                },
                what,
            ));
        }
        for (from, to) in &negations {
            if line.contains(from.as_str()) {
                let what = format!("{name}:{at} negate `{}` -> `{}`", from.trim(), to.trim());
                let edit = Edit::Replace {
                    line: i,
                    from: from.clone(),
                    to: to.clone(),
                    nth: 0,
                };
                kinds[4].push(mk(f, edit, what));
            }
        }
        for m in number.find_iter(line) {
            let Ok(n) = crate::doccheck::pycompat::parse_int(m.as_str()) else {
                continue;
            };
            let to = (n + 1).to_string();
            let what = format!("{name}:{at} number {} -> {to}", m.as_str());
            let edit = Edit::ReplaceSpan {
                line: i,
                start: m.start(),
                end: m.end(),
                to,
            };
            kinds[5].push(mk(f, edit, what));
        }
    }
}

fn interleave(groups: Vec<Vec<Mutation>>) -> Vec<Mutation> {
    let mut iters: Vec<_> = groups.into_iter().map(Vec::into_iter).collect();
    let mut out = Vec::new();
    loop {
        let before = out.len();
        for it in &mut iters {
            out.extend(it.next());
        }
        if out.len() == before {
            return out;
        }
    }
}

/// The corpus: every target's candidates, each (kind, target) group
/// shuffled by `seed`, the targets of a kind interleaved, then taken
/// round-robin across kinds up to `limit`.
pub fn corpus(
    docs: &Docs,
    targets: &[Target],
    keywords: &[&str],
    seed: u64,
    limit: usize,
) -> Vec<Mutation> {
    let mut rng = Rng(seed.max(1));
    let mut per_target: Vec<Vec<Vec<Mutation>>> = vec![Vec::new(); 7];
    for t in targets {
        for (k, mut group) in candidates(docs, t, keywords).into_iter().enumerate() {
            rng.shuffle(&mut group);
            per_target[k].push(group);
        }
    }
    // within a kind, the targets take turns, so a large file (wire.toml's
    // thousands of numbers) cannot crowd out a small section
    let mut kinds: Vec<Vec<Mutation>> = per_target.into_iter().map(interleave).collect();
    for group in &mut kinds {
        group.reverse();
    }
    let mut out = Vec::new();
    while out.len() < limit && kinds.iter().any(|g| !g.is_empty()) {
        for group in &mut kinds {
            if out.len() < limit
                && let Some(m) = group.pop()
            {
                out.push(m);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "# Top\n\n## Sec\n\nThe field is 12 bits [decision].\n\n- no item here\n\n| a | b |\n|---|---|\n| x | 3 |\n\n### Sub\n\nText.\n\n## Other\n\nOut 9.\n";

    fn docs() -> Docs {
        crate::doccheck::testing::repo().clone().with("t.md", TEXT)
    }

    const T: Target = Target {
        file: "t.md",
        heading: Some("^Sec$"),
    };

    #[test]
    fn every_kind_is_generated_inside_the_section_only() {
        let kinds = candidates(&docs(), &T, &[]);
        let counts: Vec<usize> = kinds.iter().map(Vec::len).collect();
        assert_eq!(counts, vec![1, 3, 1, 1, 2, 2, 1]);
        let all: Vec<String> = kinds.concat().into_iter().map(|m| m.label).collect();
        assert!(
            all.iter()
                .all(|l| !l.contains("Out 9") && !l.contains("number 9"))
        );
    }

    #[test]
    fn the_corpus_is_deterministic_and_covers_kinds_first() {
        let a = corpus(&docs(), &[T], &[], 7, 7);
        let b = corpus(&docs(), &[T], &[], 7, 7);
        assert_eq!(a, b);
        let kinds: std::collections::BTreeSet<String> = a
            .iter()
            .map(|m| {
                format!("{:?}", m.edit)
                    .split([' ', '{'])
                    .next()
                    .unwrap_or("")
                    .to_string()
            })
            .collect();
        assert!(kinds.len() >= 4);
        assert_eq!(corpus(&docs(), &[T], &[], 7, 100).len(), 11);
        let full = corpus(&docs(), &[T], &[], 7, 100);
        let m = full.iter().find(|m| m.label.contains("number 12")).unwrap();
        assert!(m.apply_text(TEXT).contains("is 13 bits"));
    }
}
