//! Running an old script on an exported copy of the documents.
//!
//! The scripts read their inputs in two ways: some honour override
//! variables (`WIRE_MD`, `WIRE_TOML`, `ARCH_MD`, `TICKET_MD`,
//! `REPORTS_DIR`, ...) and then skip their cargo runs; others read fixed
//! paths relative to the repository root, which they find as `$0/..`. Both
//! work against a sandbox: a temporary directory outside the repository
//! holding `doc/` exported from the same [`Docs`] the port reads, the script
//! at `checks/<id>.sh`, and nothing else. The script is run from there, so
//! its fixed paths land in the export; the spec's override variables point
//! at the exported files; and `cargo` on the script's `PATH` is a stub that
//! fails loudly (exit 97), so a cargo run the overrides did not switch off
//! shows up as a red verdict instead of silently testing the real tree.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::Docs;
use super::super::mutate::Mutation;
use super::super::text::rx;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// How a script run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptOutcome {
    /// Exit 0, no FAIL line.
    Pass,
    /// FAIL lines (exit 1, or exit 0: then [`ScriptVerdict::anomaly`]).
    Red,
    /// A Python traceback, or a non-zero exit with no FAIL line: never
    /// matched to a port's red (lead ruling R6).
    Crash,
    /// Still running after the harness's per-script timeout
    /// ([`SCRIPT_TIMEOUT_S`], or [`Sandbox::with_timeout`]): Python has no
    /// backtracking limit, so a pathological pattern takes minutes. Never
    /// "same"; matched only to a port red under `REGEX_LIMIT` (R19).
    TimedOut,
}

/// A script's verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptVerdict {
    pub outcome: ScriptOutcome,
    /// The labels of its `FAIL <label>:` lines.
    pub criteria: BTreeSet<String>,
    /// FAIL lines but exit status 0: counted red, and reported.
    pub anomaly: bool,
    /// The last lines of its output, for the log.
    pub tail: String,
}

impl ScriptVerdict {
    /// True for [`ScriptOutcome::Pass`].
    pub fn passed(&self) -> bool {
        self.outcome == ScriptOutcome::Pass
    }
}

/// Seconds a script may run under the harness before it counts as
/// [`ScriptOutcome::TimedOut`] (the scripts' document part takes well under
/// a second on real documents; a backtracking pattern can take minutes,
/// and must not stall a corpus).
pub const SCRIPT_TIMEOUT_S: u32 = 60;

/// An exported tree with one script in it; removed on drop.
#[derive(Debug)]
pub struct Sandbox {
    pub root: PathBuf,
    script: String,
    env: Vec<(String, String)>,
    timeout_s: u32,
    /// Read `FAIL: <message>` lines too ([`DOCCHECK_FAIL_FORM`]).
    colon: bool,
}

/// A `script_env` entry `(DOCCHECK_FAIL_FORM, "colon")` is a directive to
/// the harness, not passed to the script: the script prints its failures
/// as `FAIL: <message>` (the epic-002 scripts), whose criteria
/// [`crate::doccheck::fail_form::colon_criterion`] reads.
pub const DOCCHECK_FAIL_FORM: &str = "DOCCHECK_FAIL_FORM";

/// A `script_env` entry `(DOCCHECK_EXPORT, "a:b")` is a directive to the
/// harness, not passed to the script: besides `doc/`, the sandbox holds
/// these repository paths (files or directories, `:`-separated) from the
/// same documents, where they exist. For a script that reads a fixed path
/// outside `doc/` (002-13 and 002-14 read
/// `checks/002-01-frozen-data.sha256`).
pub const DOCCHECK_EXPORT: &str = "DOCCHECK_EXPORT";

/// The extra paths of the [`DOCCHECK_EXPORT`] directives of `env` that
/// exist in `docs`.
fn extra_exports(docs: &Docs, env: &[(&str, &str)]) -> Vec<String> {
    env.iter()
        .filter(|(k, _)| *k == DOCCHECK_EXPORT)
        .flat_map(|(_, v)| v.split(':'))
        .filter(|p| !p.is_empty() && (docs.is_file(p) || !docs.files_under(p).is_empty()))
        .map(str::to_string)
        .collect()
}

impl Sandbox {
    /// Exports `doc/` of `docs` and the script text under a fresh
    /// temporary directory.
    pub fn new(
        docs: &Docs,
        id: &str,
        script_text: &str,
        env: &[(&str, &str)],
    ) -> std::io::Result<Sandbox> {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("doccheck-equiv-{id}-{}-{n}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        let extra = extra_exports(docs, env);
        let mut prefixes = vec!["doc"];
        for path in &extra {
            match docs.read(path) {
                Some(text) if docs.is_file(path) => {
                    let dest = root.join(path);
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(dest, text.as_bytes())?;
                }
                _ => prefixes.push(path),
            }
        }
        docs.export(&root, &prefixes)?;
        let script = format!("checks/{id}.sh");
        fs::create_dir_all(root.join("checks"))?;
        fs::write(root.join(&script), script_text)?;
        let stub = root.join(".harness-bin");
        fs::create_dir_all(&stub)?;
        fs::write(
            stub.join("cargo"),
            "#!/bin/sh\necho 'doccheck-equiv: cargo is not available to a script under the harness' >&2\nexit 97\n",
        )?;
        make_executable(&stub.join("cargo"))?;
        Ok(Sandbox {
            root,
            script,
            env: env
                .iter()
                .filter(|(k, _)| *k != DOCCHECK_FAIL_FORM && *k != DOCCHECK_EXPORT)
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            timeout_s: SCRIPT_TIMEOUT_S,
            colon: env
                .iter()
                .any(|(k, v)| *k == DOCCHECK_FAIL_FORM && *v == "colon"),
        })
    }

    /// The same sandbox with a per-script timeout of `seconds`.
    pub fn with_timeout(mut self, seconds: u32) -> Sandbox {
        self.timeout_s = seconds.max(1);
        self
    }

    /// Runs the script on the sandbox as it stands.
    pub fn run(&self) -> ScriptVerdict {
        let path = std::env::var("PATH").unwrap_or_default();
        let stub = self.root.join(".harness-bin");
        let out = Command::new("timeout")
            .args(["--kill-after=5", &self.timeout_s.to_string()])
            .arg("bash")
            .arg(&self.script)
            .current_dir(&self.root)
            .envs(self.env.iter().map(|(k, v)| (k, v)))
            .env("PATH", format!("{}:{path}", stub.display()))
            .output();
        match out {
            // timeout(1): 124 when the command timed out, 137 when killed
            Ok(o) if matches!(o.status.code(), Some(124 | 137)) => ScriptVerdict {
                outcome: ScriptOutcome::TimedOut,
                criteria: BTreeSet::new(),
                anomaly: false,
                tail: format!("timed out after {} s", self.timeout_s),
            },
            Ok(o) => verdict_with(
                self.colon,
                o.status.success(),
                &format!(
                    "{}{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                ),
            ),
            Err(e) => verdict_with(
                self.colon,
                false,
                &format!("Traceback (most recent call last): harness cannot run bash: {e}"),
            ),
        }
    }

    /// Runs the script with `file` replaced by `text` (deleted for `None`),
    /// then restores it. Every write and restore is read back; any error
    /// aborts (a mutation that could not be applied must never read "same").
    pub fn run_edited(&self, file: &str, text: Option<&str>) -> Result<ScriptVerdict, String> {
        let path = self.root.join(file);
        let original = fs::read(&path).ok();
        match text {
            Some(t) => write_checked(&path, t.as_bytes())?,
            None => {
                if path.exists() {
                    fs::remove_file(&path).map_err(|e| format!("cannot delete {file}: {e}"))?;
                }
            }
        }
        let v = self.run();
        match &original {
            Some(bytes) => write_checked(&path, bytes)?,
            None => {
                if path.exists() {
                    fs::remove_file(&path).map_err(|e| format!("cannot restore {file}: {e}"))?;
                }
            }
        }
        Ok(v)
    }

    /// Runs the script with mutation `m` applied to its file.
    pub fn run_mutated(&self, docs: &Docs, m: &Mutation) -> Result<ScriptVerdict, String> {
        let original = docs
            .read(&m.file)
            .ok_or_else(|| format!("{} is not in the documents", m.file))?;
        self.run_edited(&m.file, Some(&m.apply_text(&original)))
    }
}

fn write_checked(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    let back = fs::read(path).map_err(|e| format!("cannot read back {}: {e}", path.display()))?;
    if back != bytes {
        return Err(format!(
            "{} does not hold what was written (disk full?)",
            path.display()
        ));
    }
    Ok(())
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// The criterion labels of `FAIL <label>: message` lines: everything
/// between `FAIL ` and the first `:` (letters, digits, spaces, hyphens,
/// dots), trimmed — `AC1`, `DoD`, `Hand-off` (004-18), `C-40`, `C-100`
/// (004-20), `WIRE`. Only the acceptance-criterion forms are normalised:
/// `AC 1` and `AC-1` read `AC1`, as the ports name them.
fn fail_labels(lines: &[&str]) -> BTreeSet<String> {
    let label = rx(r"^FAIL ([A-Za-z0-9][A-Za-z0-9 .\-]*?)\s*:");
    let ac = rx(r"^AC[ -]?([0-9]+)$");
    lines
        .iter()
        .filter_map(|l| label.captures(l).map(|c| c[1].trim().to_string()))
        .map(|l| match ac.captures(&l) {
            Some(c) => format!("AC{}", &c[1]),
            None => l,
        })
        .collect()
}

/// The criteria of the scripts that print their results grouped (006-01,
/// 006-02, 004-07): a line `  AC1` or `  DoD`, then its checks as
/// `    OK   message` or `    FAIL message`. Additive: output without such
/// groups gives nothing. (Taken from the WP9 branch, so it need not edit
/// this file.)
fn grouped_failures(lines: &[&str]) -> BTreeSet<String> {
    let group = rx(r"^  (AC[0-9]+|DoD)\s*$");
    let mut current: Option<String> = None;
    let mut out = BTreeSet::new();
    for line in lines {
        if let Some(c) = group.captures(line) {
            current = Some(c[1].to_string());
        } else if line.starts_with("    FAIL ")
            && let Some(c) = &current
        {
            out.insert(c.clone());
        }
    }
    out
}

/// The verdict of a script from its exit status and output.
///
/// The failed criteria are the labels of its `FAIL <label>:` lines
/// ([`fail_labels`]) and of its grouped results ([`grouped_failures`]). A
/// crash is the interpreter's own traceback: a line `Traceback (most recent
/// call last):` at column 0 after the last FAIL, PASS, OK or NOTE line (a
/// FAIL message that quotes the words is not one), or a non-zero exit with
/// no failed criterion at all.
pub fn verdict(exit_ok: bool, output: &str) -> ScriptVerdict {
    verdict_with(false, exit_ok, output)
}

/// The criteria of the colon form's `FAIL: <message>` lines
/// ([`DOCCHECK_FAIL_FORM`]).
fn colon_failures(lines: &[&str]) -> BTreeSet<String> {
    lines
        .iter()
        .filter_map(|l| l.strip_prefix("FAIL: "))
        .map(crate::doccheck::fail_form::colon_criterion)
        .collect()
}

/// [`verdict`], reading the colon form's `FAIL: <message>` lines as well
/// when `colon` is set (the port asked for it with [`DOCCHECK_FAIL_FORM`]).
pub fn verdict_with(colon: bool, exit_ok: bool, output: &str) -> ScriptVerdict {
    let lines: Vec<&str> = output.lines().collect();
    let mut criteria = fail_labels(&lines);
    criteria.extend(grouped_failures(&lines));
    if colon {
        criteria.extend(colon_failures(&lines));
    }
    let last_report = lines
        .iter()
        .rposition(|l| rx(r"^\s*(?:FAIL|PASS|OK|NOTE)\b").is_match(l))
        .map_or(0, |i| i + 1);
    let traceback = lines[last_report..]
        .iter()
        .any(|l| l.starts_with("Traceback (most recent call last):"));
    let crashed = traceback || (!exit_ok && criteria.is_empty());
    let outcome = if crashed {
        ScriptOutcome::Crash
    } else if exit_ok && criteria.is_empty() {
        ScriptOutcome::Pass
    } else {
        ScriptOutcome::Red
    };
    ScriptVerdict {
        outcome,
        anomaly: exit_ok && !criteria.is_empty(),
        criteria,
        tail: lines[lines.len().saturating_sub(3)..].join(" / "),
    }
}

/// The script text of `checks/<id>.sh`: from the working tree while it
/// exists, else from the parent of the commit that deleted it (so the
/// harness keeps working after a port lands). Returns (text, where).
pub fn script_source(repo: &Path, id: &str) -> Option<(String, String)> {
    let rel = format!("checks/{id}.sh");
    if let Ok(text) = fs::read_to_string(repo.join(&rel)) {
        return Some((text, format!("working tree {rel}")));
    }
    let deleted = git(
        repo,
        &["log", "--diff-filter=D", "--format=%H", "-1", "--", &rel],
    )?;
    let at = format!("{}^", deleted.trim());
    let text = git(repo, &["show", &format!("{at}:{rel}")])?;
    Some((text, format!("{rel} at {at}")))
}

/// The commit the working tree is at (`git rev-parse HEAD`), marked when
/// the tree has uncommitted changes (the port that ran is then not exactly
/// that commit's).
pub fn head_commit(repo: &Path) -> Option<String> {
    let head = git(repo, &["rev-parse", "HEAD"])?.trim().to_string();
    let dirty = git(repo, &["status", "--porcelain", "--untracked-files=no"]).is_some();
    Some(if dirty {
        format!("{head} with uncommitted changes")
    } else {
        head
    })
}

/// The commit that first added `checks/<id>.sh` (the red-phase commit).
pub fn first_added(repo: &Path, id: &str) -> Option<String> {
    let rel = format!("checks/{id}.sh");
    let out = git(repo, &["log", "--diff-filter=A", "--format=%H", "--", &rel])?;
    out.lines().last().map(str::to_string)
}

fn git(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("--no-optional-locks")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    (out.status.success() && !text.trim().is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_lines_give_criteria() {
        let v = verdict(
            false,
            "PASS AC1: x\nFAIL AC2: y\nFAIL AC 3: z\nFAIL DoD: w\nFAIL: 3 failed\n",
        );
        let want: BTreeSet<String> = ["AC2", "AC3", "DoD"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(v.criteria, want);
        assert_eq!(v.outcome, ScriptOutcome::Red);
        let crash = verdict(false, "Traceback (most recent call last):\n  boom");
        assert_eq!(crash.outcome, ScriptOutcome::Crash);
        assert_eq!(verdict(false, "").outcome, ScriptOutcome::Crash);
        let odd = verdict(true, "FAIL AC1: x");
        assert!(odd.anomaly && odd.outcome == ScriptOutcome::Red);
        let quoted = verdict(
            false,
            "FAIL AC2: the text says Traceback (most recent call last): x",
        );
        assert_eq!(
            quoted.outcome,
            ScriptOutcome::Red,
            "a quoted traceback is not a crash"
        );
        let after = verdict(
            false,
            "FAIL AC1: x\nTraceback (most recent call last):\n  File",
        );
        assert_eq!(
            after.outcome,
            ScriptOutcome::Crash,
            "a traceback after the last FAIL is"
        );
        let dash = verdict(false, "FAIL AC-1: x\nFAIL AC 2: y");
        assert!(dash.criteria.contains("AC1") && dash.criteria.contains("AC2"));
        let labels = verdict(
            false,
            "FAIL Hand-off: the hand-off is missing\nFAIL C-40: x\nFAIL C-41: y\nFAIL C-13: z\nFAIL C-100: w\nFAIL WIRE: no python\nFAIL v1.2 rule: q\nFAIL: 7 assertion(s) failed\n",
        );
        let want: BTreeSet<String> = [
            "Hand-off",
            "C-40",
            "C-41",
            "C-13",
            "C-100",
            "WIRE",
            "v1.2 rule",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(
            labels.criteria, want,
            "labels with hyphens, digits, dots and spaces are kept whole"
        );
        assert_eq!(labels.outcome, ScriptOutcome::Red);
    }

    #[test]
    fn grouped_output_gives_its_failed_criteria() {
        let out = "006-01: x\n  AC1\n    OK   fine\n    FAIL no table\n  AC2\n    OK   fine\n  AC3\n    FAIL a\n  DoD\n    FAIL b\nRESULT: FAIL (3 checks failed)\n";
        let v = verdict(false, out);
        let want: BTreeSet<String> = ["AC1", "AC3", "DoD"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(v.criteria, want);
        assert_eq!(
            v.outcome,
            ScriptOutcome::Red,
            "grouped failures are not a crash"
        );
        assert!(verdict(true, "  AC1\n    OK   fine\nRESULT: OK\n").passed());
    }

    #[test]
    fn a_sandbox_runs_a_script_on_mutated_exports_and_stubs_cargo() {
        // a one-file tree: the unit test exports a few bytes, not doc/
        let docs = Docs::dir("/nonexistent-doccheck-tree").with("doc/x.md", "keep me\n");
        let script = "cd \"$(dirname \"$0\")/..\"\ngrep -q 'keep me' doc/x.md || { echo 'FAIL AC1: gone'; exit 1; }\nif [ -n \"${RUN_CARGO:-}\" ]; then cargo test || { echo 'FAIL DoD: cargo'; exit 1; }; fi\necho OK\n";
        let sb = Sandbox::new(&docs, "000-00", script, &[]).unwrap();
        assert!(sb.root.starts_with(std::env::temp_dir()));
        assert!(sb.run().passed());
        let m = Mutation {
            file: "doc/x.md".into(),
            edit: super::super::super::mutate::Edit::DeleteLines { lo: 0, hi: 0 },
            label: "delete".into(),
        };
        let v = sb.run_mutated(&docs, &m);
        let v = v.expect("applied");
        assert!(!v.passed());
        assert!(v.criteria.contains("AC1"));
        assert!(sb.run().passed(), "the file is restored after a mutation");
        assert!(
            !sb.run_edited("doc/x.md", None).unwrap().passed(),
            "a deletion"
        );
        assert!(sb.run().passed(), "restored after a deletion");
        let cargo = Sandbox::new(&docs, "000-00", script, &[("RUN_CARGO", "1")]).unwrap();
        assert!(cargo.run().criteria.contains("DoD"), "the cargo stub fails");
        let root = sb.root.clone();
        drop(sb);
        assert!(!root.exists());
        let slow = Sandbox::new(&docs, "000-00", "echo 'FAIL AC1: late'\nsleep 30\n", &[])
            .unwrap()
            .with_timeout(1);
        let v = slow.run();
        assert_eq!(
            v.outcome,
            ScriptOutcome::TimedOut,
            "a script past its timeout"
        );
        assert!(!v.passed() && v.criteria.is_empty());
    }

    #[test]
    fn the_colon_form_is_read_only_on_request() {
        let out = "PASS: AC 1: fine\nFAIL: AC 2: section (d) verdict is still pending\nFAIL: stale sync-rate statement (x) a.md:3: 37/s\nFAIL: DoD: y\n\n2 check(s) failed, 1 passed.\n";
        let v = verdict_with(true, false, out);
        let want: BTreeSet<String> = ["AC2", "stale sync-rate statement (x) a.md:3: 37/s", "DoD"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(v.criteria, want);
        assert_eq!(v.outcome, ScriptOutcome::Red);
        let plain = verdict(false, out);
        assert_eq!(
            plain.outcome,
            ScriptOutcome::Crash,
            "without the directive the colon form names no criterion (unchanged behaviour)"
        );
        let summary = verdict(false, "FAIL AC1: x\nFAIL: 1 assertion(s) failed\n");
        assert_eq!(summary.criteria, ["AC1".to_string()].into_iter().collect());
        let both = verdict_with(true, false, "FAIL AC1: x\nFAIL: report missing: r.md\n");
        assert_eq!(
            both.criteria,
            ["AC1".to_string(), "report missing".to_string()]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn directives_are_read_by_the_harness_and_not_passed_on() {
        let docs = Docs::dir("/nonexistent-doccheck-tree")
            .with("doc/x.md", "x\n")
            .with("checks/manifest.sha256", "abc  f\n");
        let script = "cd \"$(dirname \"$0\")/..\"\n[ -f checks/manifest.sha256 ] || { echo 'FAIL: AC 7: manifest missing: checks/manifest.sha256'; exit 1; }\n[ -z \"${DOCCHECK_EXPORT:-}${DOCCHECK_FAIL_FORM:-}\" ] || { echo 'FAIL: directive leaked'; exit 1; }\n[ \"${KEEP:-}\" = 1 ] || { echo 'FAIL: env lost'; exit 1; }\necho 'PASS: ok'\n";
        let env = [
            (DOCCHECK_FAIL_FORM, "colon"),
            (DOCCHECK_EXPORT, "checks/manifest.sha256:checks/absent.txt"),
            ("KEEP", "1"),
        ];
        let sb = Sandbox::new(&docs, "000-00", script, &env).unwrap();
        assert!(sb.root.join("checks/manifest.sha256").exists());
        assert!(
            !sb.root.join("checks/absent.txt").exists(),
            "absent paths are skipped"
        );
        assert!(sb.run().passed(), "{:?}", sb.run());
        let v = sb.run_edited("checks/manifest.sha256", None).unwrap();
        assert_eq!(v.criteria, ["AC7".to_string()].into_iter().collect());
        let bare = Sandbox::new(
            &docs,
            "000-00",
            script,
            &[(DOCCHECK_FAIL_FORM, "colon"), ("KEEP", "1")],
        )
        .unwrap();
        assert!(
            bare.run().criteria.contains("AC7"),
            "without the export directive only doc/ is exported"
        );
    }

    #[test]
    fn red_phase_commit_of_a_known_script() {
        let repo = super::super::super::docs::repo_root();
        let first = first_added(&repo, "004-22");
        if let Some(c) = first {
            assert_eq!(c.len(), 40);
        }
    }
}
