//! `doccheck-equiv` — compare ported document checks with their old scripts.
//!
//! ```text
//! cargo run --release -p hadronyche-tools --bin doccheck-equiv -- <id>|--all
//!     [--seeds 2026,7,31337,99] [--limit 2000] [--no-red] [--no-denials] [--denials-only] [--jobs 4] [--script-timeout 60]
//!     [--out FILE] [--allow-dirty] [--full]
//! ```
//!
//! The defaults are the committed run: (a), (b), four seeds of 2 000
//! mutations and the denial list. `--denials-only` runs (a) and the denial
//! list against the old script (no corpus). `--all` runs every registered
//! port (after any library change) and, with `--out DIR`, writes
//! `DIR/<id>.md` for each. The output is the summary (`--full` adds every
//! row). Exits 1 when a port is not equivalent. A development tool: it
//! needs the old scripts and Python, and is never run by the per-change
//! suite.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)] // clippy.toml allow-list: a command-line tool

use std::process::exit;

use hadronyche_tools::doccheck::checks::{self, CheckSpec};
use hadronyche_tools::doccheck::docs::repo_root;
use hadronyche_tools::doccheck::equiv::{Options, render, render_full, run};

fn usage() -> ! {
    let ids: Vec<&str> = checks::registry().iter().map(|s| s.id).collect();
    eprintln!(
        "usage: doccheck-equiv <id>|--all [--seeds A,B] [--limit N] [--no-red] [--no-denials] [--denials-only] [--jobs N] [--script-timeout S] [--out PATH] [--allow-dirty] [--full]\nported checks: {}",
        ids.join(", ")
    );
    exit(2)
}

struct Args {
    ids: Vec<&'static CheckSpec>,
    opts: Options,
    out: Option<String>,
    allow_dirty: bool,
    full: bool,
}

fn parse() -> Args {
    let mut args = std::env::args().skip(1);
    let first = args.next().unwrap_or_else(|| usage());
    let ids = if first == "--all" {
        checks::registry()
    } else {
        vec![checks::find(&first).unwrap_or_else(|| {
            eprintln!("no port of {first} is registered");
            usage()
        })]
    };
    let mut a = Args {
        ids,
        opts: Options::default(),
        out: None,
        allow_dirty: false,
        full: false,
    };
    while let Some(flag) = args.next() {
        let mut value = || args.next().unwrap_or_else(|| usage());
        match flag.as_str() {
            "--seeds" => {
                a.opts.seeds = value()
                    .split(',')
                    .map(|s| s.parse().unwrap_or_else(|_| usage()))
                    .collect();
            }
            "--limit" => a.opts.limit = value().parse().unwrap_or_else(|_| usage()),
            "--no-red" => a.opts.red_phase = false,
            "--no-denials" => a.opts.denials = false,
            "--denials-only" => {
                a.opts.seeds.clear();
                a.opts.red_phase = false;
            }
            "--out" => a.out = Some(value()),
            "--allow-dirty" => a.allow_dirty = true,
            "--full" => a.full = true,
            "--jobs" => a.opts.jobs = value().parse().unwrap_or_else(|_| usage()),
            "--script-timeout" => {
                a.opts.script_timeout_s = value().parse().unwrap_or_else(|_| usage())
            }
            _ => usage(),
        }
    }
    a
}

fn main() {
    let a = parse();
    let all = a.ids.len() > 1;
    if all && let Some(dir) = &a.out {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| {
            eprintln!("cannot create {dir}: {e}");
            exit(2)
        });
    }
    // a committed summary is of a commit: refuse a dirty tree (R4 item 5)
    let dirty = hadronyche_tools::doccheck::equiv::script::dirty_paths(&repo_root());
    if a.out.is_some() && !dirty.is_empty() && !a.allow_dirty {
        eprintln!(
            "doccheck-equiv: refusing to write a summary from a tree with uncommitted changes: {}\n\
             (commit them, or pass --allow-dirty to name them in the summary's header)",
            dirty.join(", ")
        );
        exit(2)
    }
    // the tree the summaries describe: read once, at the start; the run
    // aborts if HEAD (or the tracked files) change underneath it
    let start = hadronyche_tools::doccheck::equiv::script::head_commit(&repo_root())
        .unwrap_or_else(|| "unknown".into());
    let mut failed = false;
    for spec in &a.ids {
        let outcome = match run(spec, &repo_root(), &a.opts, &mut |l| {
            eprintln!("{}: {l}", spec.id)
        }) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("doccheck-equiv {}: ABORTED: {e}", spec.id);
                exit(2)
            }
        };
        let now = hadronyche_tools::doccheck::equiv::script::head_commit(&repo_root())
            .unwrap_or_else(|| "unknown".into());
        if now != start {
            eprintln!(
                "doccheck-equiv {}: ABORTED: the tree changed during the run ({start} at the start, {now} now)",
                spec.id
            );
            exit(2)
        }
        let mut outcome = outcome;
        outcome.tree = start.clone();
        let text = if a.full {
            render_full(&outcome)
        } else {
            render(&outcome)
        };
        if let Some(path) = &a.out {
            let path = if all {
                format!("{path}/{}.md", spec.id)
            } else {
                path.clone()
            };
            std::fs::write(&path, &text).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
        }
        print!("{text}");
        failed |= !outcome.problems().is_empty();
    }
    exit(i32::from(failed))
}
