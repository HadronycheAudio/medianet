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

/// A script's verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptVerdict {
    /// Exit status 0.
    pub passed: bool,
    /// The labels of its `FAIL <label>:` lines.
    pub criteria: BTreeSet<String>,
    /// The last lines of its output, for the log.
    pub tail: String,
}

/// An exported tree with one script in it; removed on drop.
#[derive(Debug)]
pub struct Sandbox {
    pub root: PathBuf,
    script: String,
    env: Vec<(String, String)>,
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
        docs.export(&root, &["doc"])?;
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
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        })
    }

    /// Runs the script on the sandbox as it stands.
    pub fn run(&self) -> ScriptVerdict {
        let path = std::env::var("PATH").unwrap_or_default();
        let stub = self.root.join(".harness-bin");
        let out = Command::new("bash")
            .arg(&self.script)
            .current_dir(&self.root)
            .envs(self.env.iter().map(|(k, v)| (k, v)))
            .env("PATH", format!("{}:{path}", stub.display()))
            .output();
        match out {
            Ok(o) => verdict(
                o.status.success(),
                &format!(
                    "{}{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                ),
            ),
            Err(e) => verdict(false, &format!("FAIL HARNESS: cannot run bash: {e}")),
        }
    }

    /// Runs the script with `m` applied to its file, then restores it.
    pub fn run_mutated(&self, docs: &Docs, m: &Mutation) -> ScriptVerdict {
        let file = self.root.join(&m.file);
        let original = docs
            .read(&m.file)
            .map(|t| t.to_string())
            .unwrap_or_default();
        let _ = fs::write(&file, m.apply_text(&original));
        let v = self.run();
        let _ = fs::write(&file, original);
        v
    }
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

/// The verdict of a script from its exit status and output.
pub fn verdict(passed: bool, output: &str) -> ScriptVerdict {
    let criteria = rx(r"(?m)^FAIL ([A-Za-z]+ ?[0-9]*):")
        .captures_iter(output)
        .map(|c| c[1].replace(' ', ""))
        .collect();
    let lines: Vec<&str> = output.lines().collect();
    let tail = lines[lines.len().saturating_sub(3)..].join(" / ");
    ScriptVerdict {
        passed,
        criteria,
        tail,
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
        assert!(!v.passed);
    }

    #[test]
    fn a_sandbox_runs_a_script_on_mutated_exports_and_stubs_cargo() {
        let docs = Docs::repo().with("doc/x.md", "keep me\n");
        let script = "cd \"$(dirname \"$0\")/..\"\ngrep -q 'keep me' doc/x.md || { echo 'FAIL AC1: gone'; exit 1; }\nif [ -n \"${RUN_CARGO:-}\" ]; then cargo test || { echo 'FAIL DoD: cargo'; exit 1; }; fi\necho OK\n";
        let sb = Sandbox::new(&docs, "000-00", script, &[]).unwrap();
        assert!(sb.root.starts_with(std::env::temp_dir()));
        assert!(sb.run().passed);
        let m = Mutation {
            file: "doc/x.md".into(),
            edit: super::super::super::mutate::Edit::DeleteLines { lo: 0, hi: 0 },
            label: "delete".into(),
        };
        let v = sb.run_mutated(&docs, &m);
        assert!(!v.passed);
        assert!(v.criteria.contains("AC1"));
        assert!(sb.run().passed, "the file is restored after a mutation");
        let cargo = Sandbox::new(&docs, "000-00", script, &[("RUN_CARGO", "1")]).unwrap();
        assert!(cargo.run().criteria.contains("DoD"), "the cargo stub fails");
        let root = sb.root.clone();
        drop(sb);
        assert!(!root.exists());
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
