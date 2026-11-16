//! `doccheck-equiv` — compare a ported document check with its old script.
//!
//! ```text
//! cargo run --release -p hadronyche-tools --bin doccheck-equiv -- <id> [--seed N] [--limit N] [--no-red] [--out FILE]
//! ```
//!
//! Prints the Markdown report (see `hadronyche_tools::doccheck::equiv`) and
//! exits 1 when the port is not equivalent: a mutation the script rejects
//! and the port accepts, an unclassified difference, the current tree not
//! green for both, or the red-phase tree not red for both on the same
//! criteria. A development tool: it needs the old script and Python, and is
//! never run by the per-change suite.

use std::process::exit;

use hadronyche_tools::doccheck::checks;
use hadronyche_tools::doccheck::docs::repo_root;
use hadronyche_tools::doccheck::equiv::{Options, render, run};

fn usage() -> ! {
    let ids: Vec<&str> = checks::registry().iter().map(|s| s.id).collect();
    eprintln!(
        "usage: doccheck-equiv <id> [--seed N] [--limit N] [--no-red] [--out FILE]\nported checks: {}",
        ids.join(", ")
    );
    exit(2)
}

fn number(value: Option<String>) -> u64 {
    value
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| usage())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(id) = args.next() else { usage() };
    let mut opts = Options::default();
    let mut out = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seed" => opts.seed = number(args.next()),
            "--limit" => opts.limit = number(args.next()) as usize,
            "--no-red" => opts.red_phase = false,
            "--out" => out = Some(args.next().unwrap_or_else(|| usage())),
            _ => usage(),
        }
    }
    let Some(spec) = checks::find(&id) else {
        eprintln!("no port of {id} is registered");
        usage()
    };
    let outcome = match run(spec, &repo_root(), &opts, &mut |l| eprintln!("{l}")) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("doccheck-equiv {id}: {e}");
            exit(2)
        }
    };
    let text = render(&outcome, &opts);
    if let Some(path) = out {
        std::fs::write(&path, &text).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
    }
    print!("{text}");
    exit(i32::from(!outcome.problems().is_empty()))
}
