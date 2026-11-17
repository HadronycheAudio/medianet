//! `tomllib` as 004-22 used it ([`crate::doccheck::policy::TomlReader::Tomllib`]):
//! the crate's TOML parser, its values as [`PyValue`]s.
//!
//! The `toml` crate is the library's stand-in for Python's `tomllib` and is
//! used only where a script used `tomllib`. The two differ on a few inputs;
//! the corpus test below lists every difference found on
//! `hadronyche-tools/data/toml_corpus.txt` ([`KNOWN_DIFFERENCES`]), each
//! with what it means for a port. The test fails on any difference not in
//! the list, and on any listed one that has gone.

use super::pyvalue::{Dict, PyValue};

/// The corpus snippets on which the crate and `tomllib` disagree, and why
/// that is acceptable for the port that uses this reader (004-22).
pub const KNOWN_DIFFERENCES: &[(&str, &str)] = &[
    (
        "big integer",
        "tomllib keeps an int beyond 64 bits (Python ints are unbounded); the crate rejects the file. A port then \
         reports the parse failure (004-22: the script's crash, as it would crash on any tomllib error) where the \
         script would carry on with the big int: port stricter on a value no wire.toml field can hold.",
    ),
    ("int64 overflow", "the same, at 2^63."),
];

/// A TOML value as `tomllib` returns it to Python.
pub fn to_py(v: &toml::Value) -> PyValue {
    match v {
        toml::Value::String(s) => PyValue::Str(s.clone()),
        toml::Value::Integer(i) => PyValue::Int(i128::from(*i)),
        toml::Value::Float(f) => PyValue::Float(*f),
        toml::Value::Boolean(b) => PyValue::Bool(*b),
        toml::Value::Datetime(d) => PyValue::Date(d.to_string()),
        toml::Value::Array(a) => PyValue::List(a.iter().map(to_py).collect()),
        toml::Value::Table(t) => {
            PyValue::Dict(Dict(t.iter().map(|(k, v)| (k.clone(), to_py(v))).collect()))
        }
    }
}

/// `tomllib.loads(src)`: the root dict, or the parse error (`tomllib`
/// raises `TOMLDecodeError`).
pub fn parse(src: &str) -> Result<Dict, String> {
    match src.parse::<toml::Table>() {
        Ok(t) => match to_py(&toml::Value::Table(t)) {
            PyValue::Dict(d) => Ok(d),
            _ => unreachable!("a table converts to a dict"),
        },
        Err(e) => Err(format!("TOMLDecodeError: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::super::pyvalue::canonical;
    use super::super::tomlsubset;
    use super::*;

    const CORPUS: &str = include_str!("../../data/toml_corpus.txt");

    /// The corpus records: (name, source, subset result, tomllib result).
    fn records() -> Vec<(String, String, String, String)> {
        let mut out = Vec::new();
        let mut lines = CORPUS.lines().filter(|l| !l.starts_with('#'));
        while let Some(head) = lines.next() {
            let name = head
                .strip_prefix("=== ")
                .expect("a record starts with ===")
                .to_string();
            let src = decode(
                lines
                    .next()
                    .and_then(|l| l.strip_prefix("src "))
                    .expect("src"),
            );
            let sub = lines
                .next()
                .and_then(|l| l.strip_prefix("subset "))
                .expect("subset")
                .to_string();
            let tl = lines
                .next()
                .and_then(|l| l.strip_prefix("tomllib "))
                .expect("tomllib")
                .to_string();
            out.push((name, src, sub, tl));
        }
        out
    }

    /// Decodes `s"..."` with `\u{hex}` escapes.
    fn decode(s: &str) -> String {
        let body = s
            .strip_prefix("s\"")
            .and_then(|b| b.strip_suffix('"'))
            .expect("an encoded string");
        let mut out = String::new();
        let mut rest = body;
        while let Some(k) = rest.find("\\u{") {
            out.push_str(&rest[..k]);
            let end = rest[k..].find('}').expect("closing brace") + k;
            let cp = u32::from_str_radix(&rest[k + 3..end], 16).expect("hex");
            out.push(char::from_u32(cp).expect("a scalar value"));
            rest = &rest[end + 1..];
        }
        out.push_str(rest);
        out
    }

    fn result(r: Result<Dict, String>, sorted: bool) -> String {
        match r {
            Ok(d) => canonical(&PyValue::Dict(d), sorted),
            Err(e) => format!("ERR {}", e.split(':').next().unwrap_or("")),
        }
    }

    #[test]
    fn the_subset_reader_port_equals_the_python_reader_on_the_corpus() {
        let recs = records();
        assert!(recs.len() >= 90, "the corpus has {} records", recs.len());
        let bad: Vec<String> = recs
            .iter()
            .filter_map(|(name, src, want, _)| {
                let got = result(tomlsubset::parse(src), false);
                (&got != want).then(|| format!("{name}: Python {want}, port {got}"))
            })
            .collect();
        assert!(
            bad.is_empty(),
            "the port of the subset reader differs:\n{}",
            bad.join("\n")
        );
    }

    #[test]
    fn the_crate_equals_tomllib_on_the_corpus_but_for_the_listed_differences() {
        let mut found = Vec::new();
        for (name, src, _, want) in records() {
            let got = result(parse(&src), true);
            let same = if want.starts_with("ERR") {
                got.starts_with("ERR")
            } else {
                got == want
            };
            if !same {
                found.push(format!("{name}: tomllib {want}, crate {got}"));
            }
        }
        let listed: Vec<&str> = KNOWN_DIFFERENCES.iter().map(|(n, _)| *n).collect();
        let names: Vec<&str> = found
            .iter()
            .map(|f| f.split(':').next().unwrap_or(""))
            .collect();
        assert_eq!(
            names,
            listed,
            "differences from tomllib:\n{}",
            found.join("\n")
        );
    }
}
