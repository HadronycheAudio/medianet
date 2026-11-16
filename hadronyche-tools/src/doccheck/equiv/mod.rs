//! The equivalence harness: a port against its old script, on the same
//! inputs.
//!
//! Three comparisons, for one check id:
//!
//! * (a) the current tree: both must pass;
//! * (b) the tree at the commit that first added the script (its red-phase
//!   commit): both must fail, on the same criteria;
//! * (c) a deterministic corpus of structured mutations of the documents the
//!   check reads ([`corpus`]): for each, the script's exit status must equal
//!   the port's pass/fail. A difference is acceptable only when the port's
//!   [`CheckSpec::known_differences`] classify it ("port stricter" or
//!   "script wrong", with a reason). A mutation the script rejects and the
//!   port accepts is "port weaker" and fails the run, as does any
//!   unclassified difference.
//!
//! It is a development tool: it needs the old scripts (from the working
//! tree, or from git history once deleted) and Python, so it never runs in
//! the per-change suite. Run it with
//! `cargo run --release -p hadronyche-tools --bin doccheck-equiv -- <id>`.

pub mod corpus;
pub mod script;

use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, Instant};

use super::Docs;
use super::checks::{CheckSpec, DiffClass};
use super::mutate::Mutation;
use super::text::rx;
use script::{Sandbox, ScriptVerdict};

/// Options of one run.
#[derive(Debug, Clone)]
pub struct Options {
    pub seed: u64,
    pub limit: usize,
    /// Run comparison (b).
    pub red_phase: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            seed: 2026,
            limit: 300,
            red_phase: true,
        }
    }
}

/// How a row compares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Same verdict.
    Same,
    /// An understood difference.
    Classified(DiffClass, &'static str),
    /// The script rejects, the port accepts: a failure.
    PortWeaker,
    /// The port rejects, the script accepts, and no reason is recorded: a
    /// failure until classified.
    Unclassified,
}

/// One comparison.
#[derive(Debug, Clone)]
pub struct Row {
    pub case: String,
    pub script: ScriptVerdict,
    pub port_passed: bool,
    pub port_criteria: BTreeSet<String>,
    pub status: Status,
}

impl Row {
    /// True when the row fails the harness.
    pub fn is_failure(&self) -> bool {
        matches!(self.status, Status::PortWeaker | Status::Unclassified)
    }
}

/// Everything one run found.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub id: String,
    pub script_from: String,
    pub current: Row,
    pub red_commit: Option<String>,
    pub red: Option<Row>,
    pub corpus: Vec<Row>,
    pub script_time: Duration,
    pub port_time: Duration,
}

impl Outcome {
    /// The reasons the run fails, empty when it passes.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !(self.current.script.passed && self.current.port_passed) {
            out.push("(a) the current tree is not green for both".to_string());
        }
        if let Some(red) = &self.red {
            if red.script.passed || red.port_passed {
                out.push("(b) the red-phase tree is not red for both".to_string());
            } else if red.script.criteria != red.port_criteria {
                out.push("(b) the red-phase criteria differ".to_string());
            }
        }
        for r in self.corpus.iter().filter(|r| r.is_failure()) {
            out.push(format!("(c) {:?}: {}", r.status, r.case));
        }
        out
    }
}

fn classify(spec: &CheckSpec, label: &str, script: bool, port: bool) -> Status {
    if script == port {
        return Status::Same;
    }
    let known = spec
        .known_differences
        .iter()
        .find(|k| k.port_fails == !port && rx(k.label).is_match(label));
    match (known, port) {
        (Some(k), _) => Status::Classified(k.class, k.reason),
        (None, true) => Status::PortWeaker,
        (None, false) => Status::Unclassified,
    }
}

struct Timer {
    script: Duration,
    port: Duration,
}

fn compare(
    spec: &CheckSpec,
    docs: &Docs,
    sandbox: &Sandbox,
    m: Option<&Mutation>,
    case: String,
    timer: &mut Timer,
) -> Row {
    let t = Instant::now();
    let script = match m {
        Some(m) => sandbox.run_mutated(docs, m),
        None => sandbox.run(),
    };
    timer.script += t.elapsed();
    let t = Instant::now();
    let report = match m {
        Some(m) => (spec.run)(&m.apply(docs)),
        None => (spec.run)(docs),
    };
    timer.port += t.elapsed();
    let status = classify(spec, &case, script.passed, report.passed());
    Row {
        case,
        port_passed: report.passed(),
        port_criteria: report.failed_criteria(),
        script,
        status,
    }
}

/// Runs comparisons (a), (b) and (c) for `spec`; `log` gets progress lines.
pub fn run(
    spec: &CheckSpec,
    repo: &Path,
    opts: &Options,
    log: &mut dyn FnMut(&str),
) -> Result<Outcome, String> {
    let (text, script_from) =
        script::script_source(repo, spec.id).ok_or("no script in the tree or its history")?;
    let mut timer = Timer {
        script: Duration::ZERO,
        port: Duration::ZERO,
    };
    let docs = Docs::dir(repo);
    let sandbox =
        Sandbox::new(&docs, spec.id, &text, spec.script_env).map_err(|e| e.to_string())?;
    log("(a) current tree");
    let current = compare(
        spec,
        &docs,
        &sandbox,
        None,
        "current tree".into(),
        &mut timer,
    );
    let red_commit = if opts.red_phase {
        script::first_added(repo, spec.id)
    } else {
        None
    };
    let red = match &red_commit {
        Some(c) => {
            log(&format!("(b) red-phase tree {c}"));
            let old = Docs::at_commit(repo, c);
            let sb =
                Sandbox::new(&old, spec.id, &text, spec.script_env).map_err(|e| e.to_string())?;
            Some(compare(
                spec,
                &old,
                &sb,
                None,
                format!("tree at {}", &c[..10]),
                &mut timer,
            ))
        }
        None => None,
    };
    let mutations = corpus::corpus(&docs, spec.targets, spec.keywords, opts.seed, opts.limit);
    let mut rows = Vec::new();
    for (i, m) in mutations.iter().enumerate() {
        if i % 25 == 0 {
            log(&format!("(c) mutation {}/{}", i + 1, mutations.len()));
        }
        rows.push(compare(
            spec,
            &docs,
            &sandbox,
            Some(m),
            m.label.clone(),
            &mut timer,
        ));
    }
    Ok(Outcome {
        id: spec.id.to_string(),
        script_from,
        current,
        red_commit,
        red,
        corpus: rows,
        script_time: timer.script,
        port_time: timer.port,
    })
}

fn verdict_word(passed: bool, criteria: &BTreeSet<String>) -> String {
    if passed {
        "pass".to_string()
    } else {
        let c: Vec<&str> = criteria.iter().map(String::as_str).collect();
        format!("FAIL {}", c.join(","))
    }
}

fn status_word(s: &Status) -> String {
    match s {
        Status::Same => "same".into(),
        Status::Classified(DiffClass::PortStricter, why) => format!("port stricter: {why}"),
        Status::Classified(DiffClass::ScriptWrong, why) => format!("script wrong: {why}"),
        Status::PortWeaker => "**PORT WEAKER**".into(),
        Status::Unclassified => "**UNCLASSIFIED** (port rejects, script accepts)".into(),
    }
}

fn row_line(r: &Row) -> String {
    format!(
        "| {} | {} | {} | {} |\n",
        r.case.replace('|', "\\|"),
        verdict_word(r.script.passed, &r.script.criteria),
        verdict_word(r.port_passed, &r.port_criteria),
        status_word(&r.status)
    )
}

/// The counts of the corpus rows: (total, same, stricter, script wrong,
/// weaker, unclassified, both-red with differing criteria).
pub fn counts(o: &Outcome) -> [usize; 7] {
    let mut c = [0; 7];
    for r in &o.corpus {
        c[0] += 1;
        match r.status {
            Status::Same => c[1] += 1,
            Status::Classified(DiffClass::PortStricter, _) => c[2] += 1,
            Status::Classified(DiffClass::ScriptWrong, _) => c[3] += 1,
            Status::PortWeaker => c[4] += 1,
            Status::Unclassified => c[5] += 1,
        }
        if !r.script.passed && !r.port_passed && r.script.criteria != r.port_criteria {
            c[6] += 1;
        }
    }
    c
}

/// The run as Markdown: the summary, (a), (b), then the corpus table.
pub fn render(o: &Outcome, opts: &Options) -> String {
    let c = counts(o);
    let mut s = format!("# Equivalence of the port of {} with its script\n\n", o.id);
    s += &format!(
        "Script: {}. Seed {}, limit {}.\n\n",
        o.script_from, opts.seed, opts.limit
    );
    s += &format!(
        "Summary: {} mutations; {} same verdict; {} port stricter; {} script wrong; {} port weaker; {} unclassified. \
         Both red with different criteria: {}. Time: script {:.1} s, port {:.1} s.\n\n",
        c[0],
        c[1],
        c[2],
        c[3],
        c[4],
        c[5],
        c[6],
        o.script_time.as_secs_f64(),
        o.port_time.as_secs_f64()
    );
    let problems = o.problems();
    s += &if problems.is_empty() {
        "Result: EQUIVALENT (no port-weaker, no unclassified difference).\n\n".to_string()
    } else {
        format!("Result: NOT EQUIVALENT\n\n- {}\n\n", problems.join("\n- "))
    };
    s += "| case | script | port | comparison |\n|---|---|---|---|\n";
    s += &row_line(&o.current);
    if let Some(r) = &o.red {
        s += &row_line(r);
    }
    s += "\n| mutation | script | port | comparison |\n|---|---|---|---|\n";
    for r in &o.corpus {
        s += &row_line(r);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::super::checks::{KnownDifference, Target};
    use super::super::report::Report;
    use super::*;

    fn never(_: &Docs) -> Report {
        Report::default()
    }

    const KNOWN: &[KnownDifference] = &[KnownDifference {
        label: "rename heading",
        port_fails: true,
        class: DiffClass::PortStricter,
        reason: "renamed",
    }];

    const SPEC: CheckSpec = CheckSpec {
        id: "000-00",
        run: never,
        script_env: &[],
        targets: &[Target {
            file: "x",
            heading: None,
        }],
        keywords: &[],
        known_differences: KNOWN,
    };

    #[test]
    fn classification() {
        assert_eq!(classify(&SPEC, "a", true, true), Status::Same);
        assert_eq!(classify(&SPEC, "a", false, true), Status::PortWeaker);
        assert_eq!(classify(&SPEC, "a", true, false), Status::Unclassified);
        assert_eq!(
            classify(&SPEC, "x rename heading y", true, false),
            Status::Classified(DiffClass::PortStricter, "renamed")
        );
        assert_eq!(
            classify(&SPEC, "x rename heading y", false, true),
            Status::PortWeaker,
            "a reason for the port failing never excuses the port passing"
        );
    }

    #[test]
    fn rendering_and_problems() {
        let v = |p| script::verdict(p, if p { "" } else { "FAIL AC1: x" });
        let row = |case: &str, s, p, status| Row {
            case: case.into(),
            script: v(s),
            port_passed: p,
            port_criteria: if p {
                BTreeSet::new()
            } else {
                ["AC1".to_string()].into()
            },
            status,
        };
        let o = Outcome {
            id: "000-00".into(),
            script_from: "here".into(),
            current: row("current tree", true, true, Status::Same),
            red_commit: Some("c".into()),
            red: Some(row("red", false, false, Status::Same)),
            corpus: vec![
                row("m1", false, true, Status::PortWeaker),
                row("m2", true, true, Status::Same),
            ],
            script_time: Duration::ZERO,
            port_time: Duration::ZERO,
        };
        assert_eq!(o.problems().len(), 1);
        assert_eq!(counts(&o), [2, 1, 0, 0, 1, 0, 0]);
        let text = render(&o, &Options::default());
        assert!(text.contains("NOT EQUIVALENT"));
        assert!(text.contains("| m1 | FAIL AC1 | pass | **PORT WEAKER** |"));
    }
}
