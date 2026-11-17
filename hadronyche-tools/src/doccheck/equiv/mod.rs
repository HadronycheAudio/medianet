//! The equivalence harness: a port against its old script, on the same
//! inputs.
//!
//! For one check id:
//!
//! * (a) the current tree: both must pass;
//! * (b) the tree at the commit that first added the script (its red-phase
//!   commit): both must fail, on the same criteria;
//! * (c) for each seed, a deterministic corpus of structured mutations of
//!   the documents the check reads ([`corpus`]);
//! * (d) the check's denial list (`CheckSpec::denials`), each run on both.
//!
//! In (c) and (d) the verdicts are compared criterion by criterion: "same"
//! means both pass, or both fail the same set of criteria. A script crash (a
//! traceback, or a non-zero exit without a FAIL line) is its own outcome and
//! never matches a port's red; a script that prints FAIL and exits 0 counts
//! red and is reported as an anomaly. Every other pair is a difference of a
//! [`DiffKind`]; it passes only when one of the port's
//! [`CheckSpec::known_differences`] matches its label with a class that may
//! excuse that kind. An unexcused weaker kind is "port weaker"; any
//! unexcused difference fails the run, as does a denial whose expectations
//! the port does not meet. A write or restore error aborts the run.
//!
//! A development tool: it needs the old scripts (from the working tree, or
//! from git history once deleted) and Python, so it never runs in the
//! per-change suite. `cargo run --release -p hadronyche-tools --bin
//! doccheck-equiv -- <id>|--all`.

pub mod corpus;
pub mod script;

use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, Instant};

use super::Docs;
use super::checks::{CheckSpec, DiffClass, DiffKind};
use super::text::rx;
use script::{Sandbox, ScriptOutcome, ScriptVerdict};

/// Options of one run.
#[derive(Debug, Clone)]
pub struct Options {
    /// One corpus per seed.
    pub seeds: Vec<u64>,
    pub limit: usize,
    /// Run comparison (b).
    pub red_phase: bool,
    /// Run the denial list (d).
    pub denials: bool,
}

impl Default for Options {
    /// The committed run: seeds 2026 and 7, 2 000 mutations each, (b) and
    /// the denials.
    fn default() -> Options {
        Options {
            seeds: vec![2026, 7],
            limit: 2000,
            red_phase: true,
            denials: true,
        }
    }
}

/// How a row compares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Same verdict, criterion for criterion.
    Same,
    /// An understood difference.
    Classified(DiffKind, DiffClass, &'static str),
    /// A difference no known difference excuses: a failure.
    Unexcused(DiffKind),
    /// A denial whose expectations the port does not meet: a failure.
    Unmet(String),
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
        matches!(self.status, Status::Unexcused(_) | Status::Unmet(_))
    }
}

/// One seeded corpus.
#[derive(Debug, Clone)]
pub struct Corpus {
    pub seed: u64,
    pub rows: Vec<Row>,
}

/// Everything one run found.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub id: String,
    pub script_from: String,
    pub tree: String,
    pub limit: usize,
    pub current: Row,
    pub red_commit: Option<String>,
    pub red: Option<Row>,
    pub corpora: Vec<Corpus>,
    pub denials: Vec<Row>,
    pub script_time: Duration,
    pub port_time: Duration,
}

impl Outcome {
    /// Every row of (c) and (d).
    pub fn rows(&self) -> impl Iterator<Item = &Row> {
        self.corpora
            .iter()
            .flat_map(|c| c.rows.iter())
            .chain(self.denials.iter())
    }

    /// The reasons the run fails, empty when it passes.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !(self.current.script.passed() && self.current.port_passed) {
            out.push("(a) the current tree is not green for both".to_string());
        }
        if let Some(red) = &self.red
            && (red.script.outcome != ScriptOutcome::Red
                || red.port_passed
                || red.script.criteria != red.port_criteria)
        {
            out.push("(b) the red-phase tree is not red for both on the same criteria".to_string());
        }
        for r in self.rows().filter(|r| r.is_failure()) {
            out.push(format!("{}: {}", status_word(&r.status), r.case));
        }
        out
    }
}

/// The kind of difference between a script verdict and a port verdict, or
/// `None` when they are the same.
pub fn diff_kind(
    script: &ScriptVerdict,
    port_passed: bool,
    port: &BTreeSet<String>,
) -> Option<DiffKind> {
    match (script.outcome, port_passed) {
        (ScriptOutcome::Pass, true) => None,
        (ScriptOutcome::Pass, false) => Some(DiffKind::ScriptPassPortRed),
        (ScriptOutcome::Red, true) => Some(DiffKind::ScriptRedPortPass),
        (ScriptOutcome::Crash, true) => Some(DiffKind::ScriptCrashPortPass),
        (ScriptOutcome::Crash, false) => Some(DiffKind::ScriptCrashPortRed),
        (ScriptOutcome::Red, false) if &script.criteria == port => None,
        (ScriptOutcome::Red, false) if script.criteria.is_subset(port) => {
            Some(DiffKind::PortMoreCriteria)
        }
        (ScriptOutcome::Red, false) => Some(DiffKind::PortMissingCriteria),
    }
}

fn classify(spec: &CheckSpec, label: &str, kind: Option<DiffKind>) -> Status {
    let Some(kind) = kind else {
        return Status::Same;
    };
    spec.known_differences
        .iter()
        .find(|k| kind.excused_by(k.class) && rx(k.label).is_match(label))
        .map_or(Status::Unexcused(kind), |k| {
            Status::Classified(kind, k.class, k.reason)
        })
}

struct Timer {
    script: Duration,
    port: Duration,
}

fn row(
    spec: &CheckSpec,
    case: String,
    script: ScriptVerdict,
    docs: &Docs,
    timer: &mut Timer,
) -> Row {
    let t = Instant::now();
    let report = (spec.run)(docs);
    timer.port += t.elapsed();
    let port_criteria = report.failed_criteria();
    let status = classify(
        spec,
        &case,
        diff_kind(&script, report.passed(), &port_criteria),
    );
    Row {
        case,
        script,
        port_passed: report.passed(),
        port_criteria,
        status,
    }
}

fn timed<T>(timer: &mut Timer, f: impl FnOnce() -> T) -> T {
    let t = Instant::now();
    let out = f();
    timer.script += t.elapsed();
    out
}

fn red_phase(
    spec: &CheckSpec,
    repo: &Path,
    text: &str,
    timer: &mut Timer,
    log: &mut dyn FnMut(&str),
) -> Result<(Option<String>, Option<Row>), String> {
    let Some(c) = script::first_added(repo, spec.id) else {
        return Err(format!("no commit adds checks/{}.sh", spec.id));
    };
    log(&format!("(b) red-phase tree {c}"));
    let old = Docs::at_commit(repo, &c)?;
    let sb = Sandbox::new(&old, spec.id, text, spec.script_env).map_err(|e| e.to_string())?;
    let v = timed(timer, || sb.run());
    let r = row(spec, format!("tree at {}", &c[..10]), v, &old, timer);
    Ok((Some(c), Some(r)))
}

fn corpora(
    spec: &CheckSpec,
    docs: &Docs,
    sb: &Sandbox,
    opts: &Options,
    timer: &mut Timer,
    log: &mut dyn FnMut(&str),
) -> Result<Vec<Corpus>, String> {
    let mut out = Vec::new();
    for &seed in &opts.seeds {
        let mutations = corpus::corpus(docs, spec.targets, spec.keywords, seed, opts.limit);
        let mut rows = Vec::new();
        for (i, m) in mutations.iter().enumerate() {
            if i % 250 == 0 {
                log(&format!(
                    "(c) seed {seed}: mutation {}/{}",
                    i + 1,
                    mutations.len()
                ));
            }
            let v = timed(timer, || sb.run_mutated(docs, m))?;
            rows.push(row(spec, m.label.clone(), v, &m.apply(docs), timer));
        }
        out.push(Corpus { seed, rows });
    }
    Ok(out)
}

fn denials(
    spec: &CheckSpec,
    docs: &Docs,
    sb: &Sandbox,
    timer: &mut Timer,
) -> Result<Vec<Row>, String> {
    let mut out = Vec::new();
    for d in spec.denials {
        let edited = d.edited(docs)?;
        let v = timed(timer, || sb.run_edited(d.file, edited.as_deref()))?;
        let mutated = d.apply(docs)?;
        let mut r = row(spec, format!("denial {}", d.name), v, &mutated, timer);
        let unmet = d.unmet(&(spec.run)(&mutated));
        if !unmet.is_empty() {
            r.status = Status::Unmet(unmet.join("; "));
        }
        out.push(r);
    }
    Ok(out)
}

/// Runs (a), (b), (c) and (d) for `spec`; `log` gets progress lines.
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
    let sb = Sandbox::new(&docs, spec.id, &text, spec.script_env).map_err(|e| e.to_string())?;
    log("(a) current tree");
    let v = timed(&mut timer, || sb.run());
    let current = row(spec, "current tree".into(), v, &docs, &mut timer);
    let (red_commit, red) = if opts.red_phase {
        red_phase(spec, repo, &text, &mut timer, log)?
    } else {
        (None, None)
    };
    let corpora = corpora(spec, &docs, &sb, opts, &mut timer, log)?;
    let denials = if opts.denials {
        log(&format!("(d) {} denials", spec.denials.len()));
        denials(spec, &docs, &sb, &mut timer)?
    } else {
        Vec::new()
    };
    Ok(Outcome {
        id: spec.id.to_string(),
        script_from,
        tree: script::head_commit(repo).unwrap_or_else(|| "unknown".into()),
        limit: opts.limit,
        current,
        red_commit,
        red,
        corpora,
        denials,
        script_time: timer.script,
        port_time: timer.port,
    })
}

fn verdict_word(passed: bool, criteria: &BTreeSet<String>) -> String {
    if passed {
        "pass".to_string()
    } else {
        format!(
            "FAIL {}",
            criteria
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

fn script_word(v: &ScriptVerdict) -> String {
    match v.outcome {
        ScriptOutcome::Crash => format!("CRASH ({})", v.tail.replace('|', "/")),
        _ if v.anomaly => format!(
            "{} (FAIL printed, exit 0)",
            verdict_word(false, &v.criteria)
        ),
        _ => verdict_word(v.passed(), &v.criteria),
    }
}

/// The comparison column of a row.
pub fn status_word(s: &Status) -> String {
    match s {
        Status::Same => "same".into(),
        Status::Classified(kind, class, why) => format!("{kind:?}, classified {class:?}: {why}"),
        Status::Unexcused(kind) if kind.weaker() => {
            format!("**PORT WEAKER** ({kind:?}, unexcused)")
        }
        Status::Unexcused(kind) => format!("**UNCLASSIFIED** ({kind:?})"),
        Status::Unmet(why) => format!("**DENIAL UNMET BY THE PORT**: {why}"),
    }
}

fn row_line(r: &Row) -> String {
    format!(
        "| {} | {} | {} | {} |\n",
        r.case.replace('|', "\\|"),
        script_word(&r.script),
        verdict_word(r.port_passed, &r.port_criteria),
        status_word(&r.status).replace('|', "\\|")
    )
}

/// Counts of a set of rows: (total, same, of which both red, classified,
/// failures, anomalies, script crashes).
pub fn counts<'a>(rows: impl Iterator<Item = &'a Row>) -> [usize; 7] {
    let mut c = [0; 7];
    for r in rows {
        c[0] += 1;
        match r.status {
            Status::Same => {
                c[1] += 1;
                if !r.port_passed {
                    c[2] += 1;
                }
            }
            Status::Classified(..) => c[3] += 1,
            _ => c[4] += 1,
        }
        c[5] += usize::from(r.script.anomaly);
        c[6] += usize::from(r.script.outcome == ScriptOutcome::Crash);
    }
    c
}

fn count_line(name: &str, c: [usize; 7]) -> String {
    format!(
        "| {name} | {} | {} | {} | {} | {} | {} | {} |\n",
        c[0], c[1], c[2], c[3], c[4], c[5], c[6]
    )
}

/// The committed summary: the trees, the counts per seed and for the
/// denials, rows (a) and (b), and every row that is not "same".
pub fn render(o: &Outcome) -> String {
    let mut s = format!("# Equivalence of the port of {} with its script\n\n", o.id);
    s += &format!(
        "Tree: {}. Script: {}. Mutations per seed: {}. Time: script {:.0} s, port {:.0} s.\n\n",
        o.tree,
        o.script_from,
        o.limit,
        o.script_time.as_secs_f64(),
        o.port_time.as_secs_f64()
    );
    let problems = o.problems();
    s += &if problems.is_empty() {
        "Result: EQUIVALENT (no port weaker, no unexcused difference, every denial met).\n\n"
            .to_string()
    } else {
        format!("Result: NOT EQUIVALENT\n\n- {}\n\n", problems.join("\n- "))
    };
    s += "| run | rows | same | of which both red | classified | failures | script FAIL with exit 0 | script crashes |\n|---|---|---|---|---|---|---|---|\n";
    for c in &o.corpora {
        s += &count_line(&format!("corpus, seed {}", c.seed), counts(c.rows.iter()));
    }
    s += &count_line("denials", counts(o.denials.iter()));
    s += "\n| case | script | port | comparison |\n|---|---|---|---|\n";
    s += &row_line(&o.current);
    if let Some(r) = &o.red {
        s += &row_line(r);
    }
    let others: Vec<&Row> = o
        .rows()
        .filter(|r| r.status != Status::Same || r.script.anomaly)
        .collect();
    s += &format!("\nRows that are not \"same\" ({}):\n\n", others.len());
    if !others.is_empty() {
        s += "| case | script | port | comparison |\n|---|---|---|---|\n";
        for r in others {
            s += &row_line(r);
        }
    }
    s
}

/// The full table of every row (`--full`; not committed).
pub fn render_full(o: &Outcome) -> String {
    let mut s = render(o);
    s += "\n| case | script | port | comparison |\n|---|---|---|---|\n";
    for r in o.rows() {
        s += &row_line(r);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::super::checks::{KnownDifference, Target};
    use super::super::policy::TomlReader;
    use super::super::report::Report;
    use super::*;

    fn never(_: &Docs) -> Report {
        Report::default()
    }

    const KNOWN: &[KnownDifference] = &[KnownDifference {
        label: "typed",
        class: DiffClass::PortStricterTyped,
        reason: "typed",
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
        denials: &[],
        toml: TomlReader::Tomllib,
    };

    fn set(xs: &[&str]) -> BTreeSet<String> {
        xs.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn criteria_are_compared_not_only_exit_status() {
        let red = script::verdict(false, "FAIL AC1: x\nFAIL AC2: y");
        assert_eq!(diff_kind(&red, false, &set(&["AC1", "AC2"])), None);
        assert_eq!(
            diff_kind(&red, false, &set(&["AC1", "AC2", "AC4"])),
            Some(DiffKind::PortMoreCriteria)
        );
        assert_eq!(
            diff_kind(&red, false, &set(&["AC1", "AC4"])),
            Some(DiffKind::PortMissingCriteria)
        );
        assert_eq!(
            diff_kind(&red, true, &set(&[])),
            Some(DiffKind::ScriptRedPortPass)
        );
        let crash = script::verdict(false, "Traceback (most recent call last):");
        assert_eq!(
            diff_kind(&crash, false, &set(&["AC4"])),
            Some(DiffKind::ScriptCrashPortRed)
        );
    }

    #[test]
    fn classes_excuse_only_their_direction() {
        let k = Some(DiffKind::ScriptPassPortRed);
        assert!(matches!(
            classify(&SPEC, "x typed", k),
            Status::Classified(..)
        ));
        assert_eq!(
            classify(&SPEC, "x other", k),
            Status::Unexcused(DiffKind::ScriptPassPortRed)
        );
        let weaker = Some(DiffKind::ScriptRedPortPass);
        assert_eq!(
            classify(&SPEC, "x typed", weaker),
            Status::Unexcused(DiffKind::ScriptRedPortPass)
        );
        assert!(DiffKind::ScriptRedPortPass.weaker() && !DiffKind::ScriptPassPortRed.weaker());
        assert!(DiffKind::ScriptRedPortPass.excused_by(DiffClass::SubsetTomlReader));
        assert!(!DiffKind::ScriptRedPortPass.excused_by(DiffClass::ScriptWrong));
    }

    #[test]
    fn rendering_and_problems() {
        let mk = |case: &str, out: &str, port: bool, status| Row {
            case: case.into(),
            script: script::verdict(out.is_empty(), out),
            port_passed: port,
            port_criteria: if port { BTreeSet::new() } else { set(&["AC1"]) },
            status,
        };
        let o = Outcome {
            id: "000-00".into(),
            script_from: "here".into(),
            tree: "abc".into(),
            limit: 2,
            current: mk("current tree", "", true, Status::Same),
            red_commit: Some("c".into()),
            red: Some(mk("red", "FAIL AC1: x", false, Status::Same)),
            corpora: vec![Corpus {
                seed: 1,
                rows: vec![
                    mk(
                        "m1",
                        "FAIL AC1: x",
                        true,
                        Status::Unexcused(DiffKind::ScriptRedPortPass),
                    ),
                    mk("m2", "", true, Status::Same),
                ],
            }],
            denials: vec![mk(
                "denial d",
                "FAIL AC1: x",
                false,
                Status::Unmet("expected".into()),
            )],
            script_time: Duration::ZERO,
            port_time: Duration::ZERO,
        };
        assert_eq!(o.problems().len(), 2);
        let text = render(&o);
        assert!(text.contains("NOT EQUIVALENT"));
        assert!(text.contains("| m1 | FAIL AC1 | pass | **PORT WEAKER**"));
        assert!(!text.contains("| m2 |"), "a same row is not in the summary");
        assert!(render_full(&o).contains("| m2 |"));
    }
}
