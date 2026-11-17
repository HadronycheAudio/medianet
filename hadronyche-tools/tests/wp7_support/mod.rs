//! Support for the WP7 document-check tests (004-16 to 004-20): the denial
//! and acceptance tables, and their opt-in run against the old script.
//!
//! A *denial* is the real documents with one mutation applied in memory
//! that the check must reject, by the named criterion with a message
//! containing the needle. An *acceptance probe* is a mutation the check
//! must accept: each one aims at a place where Python and Rust read text
//! differently (a `²` that Python's `\w` takes, a `40.0` that Python
//! compares equal to 40, ...), so the port is shown to accept what the
//! script accepts there.
//!
//! [`denials!`] and [`accepts!`] declare each row as a `#[test]` against
//! the port and as one row of a table; the ignored test
//! `old_script_agrees` of each test file runs every row against the old
//! script through the equivalence harness's sandbox (`cargo test -p
//! hadronyche-tools --test doc_<id> -- --ignored`; it needs python3 and the
//! script, read from the working tree or, once deleted, from the history).
//! The criterion `CRASH` stands for a Python exception: the script must
//! fail with no `FAIL` line.

#![allow(dead_code)]

use hadronyche_tools::doccheck::Docs;
use hadronyche_tools::doccheck::checks;
use hadronyche_tools::doccheck::docs::repo_root;
use hadronyche_tools::doccheck::equiv::script::{Sandbox, script_source};

/// One table row: the documents and what the check must say about them.
pub struct Row {
    pub name: &'static str,
    /// The criterion that must fail; `None` for an acceptance probe.
    pub criterion: Option<&'static str>,
    pub docs: Docs,
}

/// Declares denial tests: `name => (criterion, needle, |base| docs);`.
/// Needs `base()` and `run` in scope.
#[macro_export]
macro_rules! denials {
    ($( $(#[$m:meta])* $name:ident => ($crit:expr, $needle:expr, $edit:expr); )*) => {
        $(
            $(#[$m])*
            #[test]
            fn $name() {
                let docs = ($edit)(base());
                hadronyche_tools::doccheck::assert_fails(&run(&docs), $crit, $needle);
            }
        )*
        fn all_denials() -> Vec<wp7_support::Row> {
            vec![ $( wp7_support::Row {
                name: stringify!($name),
                criterion: Some($crit),
                docs: ($edit)(base()),
            } ),* ]
        }
    };
}

/// Declares acceptance probes: `name => |base| docs;`. Needs `base()` and
/// `run` in scope.
#[macro_export]
macro_rules! accepts {
    ($( $(#[$m:meta])* $name:ident => $edit:expr; )*) => {
        $(
            $(#[$m])*
            #[test]
            fn $name() {
                let docs = ($edit)(base());
                hadronyche_tools::doccheck::assert_clean(&run(&docs));
            }
        )*
        fn all_accepts() -> Vec<wp7_support::Row> {
            vec![ $( wp7_support::Row {
                name: stringify!($name),
                criterion: None,
                docs: ($edit)(base()),
            } ),* ]
        }
    };
}

/// Runs the old script of `id` on every row and returns the rows where it
/// disagrees with the table: a denial it accepts or rejects by another
/// criterion (or, for `CRASH`, without crashing), a probe it rejects.
pub fn script_disagrees(id: &str, rows: &[Row]) -> Vec<String> {
    let repo = repo_root();
    let (text, from) =
        script_source(&repo, id).expect("the old script, in the tree or its history");
    let spec = checks::find(id).expect("the port is registered");
    eprintln!("old script of {id}: {from}");
    let mut out = Vec::new();
    for r in rows {
        let sb = Sandbox::new(&r.docs, id, &text, spec.script_env).expect("sandbox");
        let v = sb.run();
        drop(sb);
        let crit: Vec<&str> = v.criteria.iter().map(String::as_str).collect();
        let ok = match r.criterion {
            None => v.passed,
            Some("CRASH") => !v.passed && v.criteria.is_empty(),
            // The harness reads no criterion from a label with a hyphen
            // (`FAIL Hand-off:`): such a denial is compared by verdict.
            Some(c) if c.contains('-') => !v.passed,
            Some(c) => !v.passed && v.criteria.contains(c),
        };
        let verdict = if v.passed {
            "accepts".to_string()
        } else {
            format!("rejects {}", crit.join(","))
        };
        eprintln!(
            "{:<64} want {:<10} script {verdict}{}",
            r.name,
            r.criterion.unwrap_or("pass"),
            if ok { "" } else { "  <-- DIFFERS" }
        );
        if !ok {
            out.push(format!(
                "{}: want {}, script {verdict} ({})",
                r.name,
                r.criterion.unwrap_or("pass"),
                v.tail
            ));
        }
    }
    out
}
