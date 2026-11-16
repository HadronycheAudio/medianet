//! Support for the WP9 document-check tests: the denial table, and the
//! opt-in run of every denial fixture against the old script.
//!
//! A denial fixture is the real documents with one mutation applied in
//! memory. [`denials!`] declares each as one `#[test]` (the port must
//! reject it, by the named criterion and message) and as one row of a table;
//! the ignored test `old_script_agrees_on_the_denials` of each test file
//! exports every row's documents through the equivalence harness and runs the
//! old script on them (`cargo test -p hadronyche-tools --test doc_<id> --
//! --ignored`; it needs Python and the script, which is read from the working
//! tree or from the history once deleted). The script must reject every
//! denial, except the ones the file lists as `SCRIPT_ACCEPTS`, each recorded
//! as "script wrong" in the test's doc comment.

#![allow(dead_code)]

use hadronyche_tools::doccheck::Docs;
use hadronyche_tools::doccheck::checks;
use hadronyche_tools::doccheck::docs::repo_root;
use hadronyche_tools::doccheck::equiv::script::{Sandbox, script_source};

/// One denial fixture.
pub struct Denial {
    pub name: &'static str,
    pub criterion: &'static str,
    pub needle: &'static str,
    pub docs: Docs,
}

/// Declares denial tests: `name => (criterion, needle, |base| docs)`, each
/// with its own attributes and doc comments. Needs `base()` and `run` in
/// scope.
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
        fn all_denials() -> Vec<wp9_support::Denial> {
            vec![ $( wp9_support::Denial {
                name: stringify!($name),
                criterion: $crit,
                needle: $needle,
                docs: ($edit)(base()),
            } ),* ]
        }
    };
}

/// Runs the old script of `id` on every denial; returns the names of those
/// it accepts (and prints a line per denial).
pub fn script_accepts(id: &str, all: &[Denial]) -> Vec<&'static str> {
    let repo = repo_root();
    let (text, from) =
        script_source(&repo, id).expect("the old script, in the tree or its history");
    let spec = checks::find(id).expect("the port is registered");
    eprintln!("old script of {id}: {from}");
    let mut accepted = Vec::new();
    for d in all {
        let sb = Sandbox::new(&d.docs, id, &text, spec.script_env).expect("sandbox");
        let v = sb.run();
        let criteria: Vec<&str> = v.criteria.iter().map(String::as_str).collect();
        eprintln!(
            "{:<60} script {} {}{}",
            d.name,
            if v.passed { "ACCEPTS" } else { "rejects" },
            criteria.join(","),
            if !v.passed && !v.criteria.contains(d.criterion) {
                "  (not by the expected criterion)"
            } else {
                ""
            }
        );
        if v.passed {
            accepted.push(d.name);
        }
    }
    accepted
}
