//! The Python corpus (`data/pycompat_corpus.txt`) against the library, and
//! unit tests of `pynum`.

use super::*;
use crate::doccheck::pycompat::{nfkc, py_slice};
use crate::doccheck::pyvalue::PyValue;
use crate::doccheck::text::rx;

const CORPUS: &str = include_str!("../../../data/pycompat_corpus.txt");

fn dec(f: &str) -> Option<String> {
    if f == "~" {
        return None;
    }
    let mut out = String::new();
    let mut it = f.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(o) => out.push(o),
            None => out.push('\\'),
        }
    }
    Some(out)
}

fn s(f: &str) -> String {
    dec(f).unwrap_or_default()
}

fn bound(f: &str) -> Option<i64> {
    dec(f).map(|b| i64::try_from(parse_int(&b).expect("a corpus bound")).expect("small"))
}

fn fits(n: &str) -> bool {
    parse_int(n).is_ok()
        || !n
            .trim_start_matches('-')
            .chars()
            .all(|c| c.is_ascii_digit())
}

/// The library's answer for one case, in the corpus's field form.
fn answer(op: &str, a: &[&str]) -> Vec<Option<String>> {
    let some = |v: String| vec![Some(v)];
    match op {
        "search" => match rx(&s(a[0])).captures(&s(a[1])) {
            None => vec![None],
            Some(c) => (0..c.len())
                .map(|i| c.get(i).map(|m| m.as_str().to_string()))
                .collect(),
        },
        "iter" => rx(&s(a[0]))
            .find_iter(&s(a[1]))
            .map(|m| Some(m.as_str().to_string()))
            .collect(),
        "split" => rx(&s(a[0]))
            .split(&s(a[1]))
            .map(|p| Some(p.to_string()))
            .collect(),
        "sub" => some(rx(&s(a[0])).replace_all_literal(&s(a[2]), &s(a[1]))),
        "float" => some(py_float(&s(a[0])).map_or_else(|| "ValueError".into(), repr_float)),
        "fmt" => {
            some(format_float(py_float(&s(a[1])).expect("a float"), &s(a[0])).unwrap_or_default())
        }
        "pct" => {
            some(percent_float(py_float(&s(a[1])).expect("a float"), &s(a[0])).unwrap_or_default())
        }
        "slice" => some(py_slice(&s(a[0]), bound(a[1]), bound(a[2]))),
        "truthy" => {
            let v = s(a[1]);
            let val = match a[0] {
                "str" => PyValue::Str(v),
                "int" => PyValue::Int(parse_int(&v).expect("int")),
                "float" => PyValue::Float(py_float(&v).expect("float")),
                _ => PyValue::List(vec![
                    PyValue::Int(0);
                    usize::try_from(parse_int(&v).expect("n")).expect("n")
                ]),
            };
            some(if val.truthy() { "True" } else { "False" }.into())
        }
        "eval" => some(match eval_arith(&s(a[0])) {
            Ok(n) => n.repr(),
            Err(e) => e.name().into(),
        }),
        "int" => some(match parse_int(&s(a[0])) {
            Ok(n) => n.to_string(),
            Err(IntError::Overflow(_)) => "INT_LIMIT".into(),
            Err(IntError::NotInt(_)) => "ValueError".into(),
        }),
        "frac" => some(
            Fraction::new(
                parse_int(&s(a[0])).expect("n"),
                parse_int(&s(a[1])).expect("d"),
            )
            .map_or_else(|| "ZeroDivisionError".into(), |f| f.to_string()),
        ),
        "fraclim" => some(
            Fraction::from_f64(py_float(&s(a[0])).expect("float")).map_or_else(
                || "INT_LIMIT".into(),
                |f| f.limit_denominator(1_000_000).to_string(),
            ),
        ),
        "nfkc" => some(nfkc(&s(a[0]))),
        other => panic!("unknown corpus op {other}"),
    }
}

/// How many argument fields each operation has.
fn arity(op: &str) -> usize {
    match op {
        "float" | "eval" | "int" | "fraclim" | "nfkc" => 1,
        "sub" | "slice" => 3,
        _ => 2,
    }
}

/// Python's answer, with an integer beyond `i128` replaced by `INT_LIMIT`
/// (the library's named outcome there).
fn expected(op: &str, res: &[&str]) -> Vec<Option<String>> {
    let mut want: Vec<Option<String>> = res.iter().map(|f| dec(f)).collect();
    if matches!(op, "eval" | "int")
        && let Some(Some(v)) = want.first()
        && !fits(v)
    {
        want = vec![Some("INT_LIMIT".into())];
    }
    want
}

#[test]
fn the_corpus_agrees_with_python() {
    let mut bad = Vec::new();
    let mut n = 0;
    for line in CORPUS.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        let op = f[0];
        let k = arity(op);
        let (args, res) = (&f[1..=k], &f[k + 1..]);
        let got = answer(op, args);
        let want = expected(op, res);
        n += 1;
        if got != want {
            bad.push(format!("{op} {args:?}: Python {want:?}, library {got:?}"));
        }
    }
    take_int_limits();
    assert!(n > 5000, "the corpus has {n} cases");
    assert!(
        bad.is_empty(),
        "{} of {n} cases differ:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

#[test]
fn beyond_i128_is_recorded() {
    take_int_limits();
    assert!(parse_int(&"9".repeat(40)).is_err());
    assert_eq!(eval_arith("2**200"), Err(EvalError::IntLimit));
    assert!(PyValue::BigInt("9".repeat(40)).py_int().is_err());
    assert_eq!(take_int_limits().len(), 3);
    assert!(take_int_limits().is_empty());
    restore_int_limits(vec!["x".into()]);
    assert_eq!(take_int_limits(), vec!["x".to_string()]);
}

#[test]
fn eval_types_and_errors() {
    assert_eq!(eval_arith("2**-1"), Ok(PyNum::Float(0.5)));
    assert_eq!(eval_arith("-2**2"), Ok(PyNum::Int(-4)));
    assert_eq!(eval_arith("01"), Err(EvalError::SyntaxError));
    assert!(PyNum::Float(4.0).eq_int(4) && !PyNum::Float(4.5).eq_int(4));
    assert_eq!(
        Fraction::new(2, -4).map(|f| f.to_string()),
        Some("-1/2".into())
    );
    assert_eq!(Fraction::new(1, 0), None);
}
