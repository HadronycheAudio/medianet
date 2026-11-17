//! The documents a check reads, passed as a value.
//!
//! Every check function takes a [`Docs`]. By default it reads the working
//! tree of this repository ([`Docs::repo`]); the equivalence harness reads a
//! commit ([`Docs::at_commit`]); a test builds a fixture as the real document
//! with a mutation applied in memory ([`Docs::with`], [`Docs::mutate`]). No
//! check reads the file system or the environment by itself, so no fixture
//! lives in `/tmp` and no override variable exists.
//!
//! Paths are repository-relative with `/` separators (`doc/design/spec/wire.md`).
//!
//! The repertoire guard is a property of `Docs` (lead ruling R20): a check
//! runs only through [`CheckSpec::evaluate`], which hands it a logged view;
//! every text the check obtains from [`Docs::read`] (whatever the path:
//! a ticket, a separate review record, a link target outside `doc/`) is
//! logged and guarded, and one character outside the repertoire makes the
//! verdict `REPERTOIRE`. No list of inputs is kept by any check. The same
//! entry point turns a regex engine failure into `REGEX_LIMIT` (R19).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)] // clippy.toml allow-list: the document sources themselves

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use super::checks::CheckSpec;
use super::pycompat::{PyStr, normpath};
use super::repertoire::guard_text;
use super::report::Report;
use super::rx::{RxFailure, restore_failures, take_failures};

/// The criterion of a regex engine failure (R19).
pub const REGEX_LIMIT: &str = "REGEX_LIMIT";

/// Where the unmodified documents come from.
#[derive(Debug, Clone)]
pub enum Source {
    /// A directory on disk (the repository's working tree, or an export).
    Dir(PathBuf),
    /// A commit of a git repository, read with `git cat-file`.
    Git { repo: PathBuf, commit: String },
}

/// A set of documents: a source plus in-memory replacements and deletions.
#[derive(Debug, Clone)]
pub struct Docs {
    source: Source,
    overrides: BTreeMap<String, Option<Arc<str>>>,
    cache: Arc<Mutex<HashMap<String, Option<Arc<str>>>>>,
    tree: Arc<Mutex<Option<Arc<BTreeSet<String>>>>>,
    /// Set inside [`CheckSpec::evaluate`]: every text read, in order.
    log: Option<Arc<Mutex<ReadLog>>>,
}

/// The texts a check read, by normalised path, first read first.
type ReadLog = Vec<(String, Arc<str>)>;

impl Docs {
    /// The working tree of this repository (the crate's parent directory).
    pub fn repo() -> Docs {
        Docs::dir(repo_root())
    }

    /// The documents of a directory on disk.
    pub fn dir(root: impl Into<PathBuf>) -> Docs {
        Docs::from_source(Source::Dir(root.into()))
    }

    /// The documents of `commit` in the git repository at `repo`.
    ///
    /// Fails loudly: `Err` when the commit is not in the repository or its
    /// tree cannot be listed (never an empty tree, on which a red-phase test
    /// would fail every criterion vacuously).
    pub fn at_commit(repo: impl Into<PathBuf>, commit: &str) -> Result<Docs, String> {
        let repo = repo.into();
        if !commit_exists(&repo, commit) {
            return Err(format!(
                "commit {commit} is not in the repository at {}",
                repo.display()
            ));
        }
        let tree = git_tree(&repo, commit)?;
        let docs = Docs::from_source(Source::Git {
            repo,
            commit: commit.to_string(),
        });
        *docs.tree.lock().expect("docs tree poisoned") = Some(Arc::new(tree));
        Ok(docs)
    }

    fn from_source(source: Source) -> Docs {
        Docs {
            source,
            overrides: BTreeMap::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            tree: Arc::new(Mutex::new(None)),
            log: None,
        }
    }

    /// Where the unmodified documents come from.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// A copy whose file `path` holds `text`.
    pub fn with(&self, path: &str, text: impl Into<String>) -> Docs {
        let mut out = self.clone();
        let text: String = text.into();
        out.overrides.insert(norm(path), Some(Arc::from(text)));
        out
    }

    /// A copy without the file `path`.
    pub fn without(&self, path: &str) -> Docs {
        let mut out = self.clone();
        out.overrides.insert(norm(path), None);
        out
    }

    /// A copy whose file `path` is `edit` applied to its text. Panics when
    /// the file does not exist: a fixture must start from a real document.
    pub fn mutate(&self, path: &str, edit: impl FnOnce(&str) -> String) -> Docs {
        let text = self
            .read(path)
            .unwrap_or_else(|| panic!("fixture base {path} does not exist"));
        self.with(path, edit(&text))
    }

    /// The replaced or deleted files, for an export.
    pub fn overrides(&self) -> impl Iterator<Item = (&str, Option<&str>)> {
        self.overrides
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_deref()))
    }

    /// The text of `path`, or `None` when it does not exist or is not UTF-8.
    pub fn read(&self, path: &str) -> Option<Arc<str>> {
        let text = self.read_unlogged(path);
        if let (Some(log), Some(t)) = (&self.log, &text) {
            let key = norm(path);
            let mut log = log.lock().expect("docs log poisoned");
            if !log.iter().any(|(k, _)| *k == key) {
                log.push((key, t.clone()));
            }
        }
        text
    }

    fn read_unlogged(&self, path: &str) -> Option<Arc<str>> {
        let key = norm(path);
        if outside(&key) {
            return None;
        }
        if let Some(over) = self.overrides.get(&key) {
            return over.clone();
        }
        let mut cache = self.cache.lock().expect("docs cache poisoned");
        if let Some(found) = cache.get(&key) {
            return found.clone();
        }
        let loaded = self.load(&key).map(Arc::from);
        cache.insert(key, loaded.clone());
        loaded
    }

    fn load(&self, key: &str) -> Option<String> {
        match &self.source {
            Source::Dir(root) => fs::read_to_string(root.join(key)).ok(),
            Source::Git { repo, commit } => {
                if !self.tree().contains(key) {
                    return None;
                }
                git_show(repo, commit, key)
            }
        }
    }

    /// True when `path` is a file of this set.
    pub fn is_file(&self, path: &str) -> bool {
        let key = norm(path);
        if outside(&key) {
            return false;
        }
        if let Some(over) = self.overrides.get(&key) {
            return over.is_some();
        }
        match &self.source {
            Source::Dir(root) => root.join(&key).is_file(),
            Source::Git { .. } => self.tree().contains(&key),
        }
    }

    /// Every file under `prefix` (recursive), sorted, overrides applied.
    pub fn files_under(&self, prefix: &str) -> Vec<String> {
        let prefix = norm(prefix);
        let mut set: BTreeSet<String> = match &self.source {
            Source::Dir(root) => walk(root, &prefix),
            Source::Git { .. } => self
                .tree()
                .iter()
                .filter(|p| under(p, &prefix))
                .cloned()
                .collect(),
        };
        for (path, text) in &self.overrides {
            if under(path, &prefix) {
                if text.is_some() {
                    set.insert(path.clone());
                } else {
                    set.remove(path);
                }
            }
        }
        set.into_iter().collect()
    }

    /// The files directly in `dir` whose name starts with `stem`, sorted
    /// (the shell's `ls dir/stem*`).
    pub fn glob(&self, dir: &str, stem: &str) -> Vec<String> {
        let dir = norm(dir);
        self.files_under(&dir)
            .into_iter()
            .filter(|p| {
                let rest = &p[dir.len() + 1..];
                !rest.contains('/') && rest.starts_with(stem)
            })
            .collect()
    }

    fn tree(&self) -> Arc<BTreeSet<String>> {
        let mut slot = self.tree.lock().expect("docs tree poisoned");
        if let Some(tree) = slot.as_ref() {
            return tree.clone();
        }
        let tree = match &self.source {
            Source::Git { repo, commit } => Arc::new(
                git_tree(repo, commit)
                    .unwrap_or_else(|e| panic!("the tree of {commit} vanished: {e}")),
            ),
            Source::Dir(_) => Arc::new(BTreeSet::new()),
        };
        *slot = Some(tree.clone());
        tree
    }

    /// Writes every file under each prefix (overrides applied) below `dest`.
    /// A commit is exported with `git archive` (one process, not one per
    /// file), then the overrides are written over it.
    pub fn export(&self, dest: &Path, prefixes: &[&str]) -> std::io::Result<()> {
        if let Source::Git { repo, commit } = &self.source {
            git_archive(repo, commit, prefixes, dest)?;
            for (path, text) in self.overrides() {
                match text {
                    Some(t) => write_file(&dest.join(path), t.as_bytes())?,
                    None => {
                        let _ = fs::remove_file(dest.join(path));
                    }
                }
            }
            return Ok(());
        }
        for prefix in prefixes {
            for path in self.files_under(prefix) {
                let Some(text) = self.read(&path) else {
                    copy_binary(self, &path, dest)?;
                    continue;
                };
                write_file(&dest.join(&path), text.as_bytes())?;
            }
        }
        Ok(())
    }
}

impl CheckSpec {
    /// The only way to run a check (its `run` is called nowhere else; a
    /// source test holds this). The check gets a logged view of `docs`;
    /// afterwards:
    ///
    /// * a text it read that holds a character outside the repertoire makes
    ///   the verdict red under `REPERTOIRE` alone, one failure per such file
    ///   (R13, R20): inside the repertoire the port equals its script, and
    ///   outside it nothing else it found is meaningful;
    /// * otherwise a regex engine failure during the run (backtracking
    ///   limit, a pattern that does not compile) makes the verdict red
    ///   under `REGEX_LIMIT` alone, with the pattern and the file:line of
    ///   the text it was matching (R19): an engine failure is never "no
    ///   match", and the rest of the verdict cannot be trusted.
    pub fn evaluate(&self, docs: &Docs) -> Report {
        let outer = take_failures();
        let log = Arc::new(Mutex::new(ReadLog::new()));
        let mut view = docs.clone();
        view.log = Some(log.clone());
        let report = (self.run)(&view);
        let failures = take_failures();
        restore_failures(outer);
        let read = std::mem::take(&mut *log.lock().expect("docs log poisoned"));
        let mut guarded = Report::default();
        for (path, text) in &read {
            guard_text(path, text, &mut guarded);
        }
        if !guarded.failures.is_empty() {
            return guarded;
        }
        if failures.is_empty() {
            return report;
        }
        let mut limited = Report::default();
        for f in dedup(failures) {
            let (file, line) = locate(&read, &f.hay);
            let pattern: String = f.pattern.chars().take(160).collect();
            let what = format!(
                "the regex engine failed on pattern {pattern:?} ({}); the check cannot \
                 tell match from no match, so it is red by rule (R19)",
                f.what
            );
            match file {
                Some(file) => limited.fail_at(REGEX_LIMIT, &file, line, what),
                None => limited.fail(
                    REGEX_LIMIT,
                    format!("{what}; the text begins {:?}", head(&f.hay)),
                ),
            }
        }
        limited
    }
}

fn dedup(failures: Vec<RxFailure>) -> Vec<RxFailure> {
    let mut out: Vec<RxFailure> = Vec::new();
    for f in failures {
        if !out.iter().any(|o| o.pattern == f.pattern && o.hay == f.hay) {
            out.push(f);
        }
    }
    out
}

fn head(text: &str) -> String {
    text.chars().take(60).collect()
}

/// The first document read whose text contains the start of `hay`, and
/// the 1-based line where it starts (the unit being matched).
fn locate(read: &ReadLog, hay: &str) -> (Option<String>, usize) {
    let probe: String = hay.trim_start().chars().take(60).collect();
    if probe.is_empty() {
        return (None, 0);
    }
    for (path, text) in read {
        if let Some(at) = text.find(probe.as_str()) {
            return (Some(path.clone()), text[..at].matches('\n').count() + 1);
        }
    }
    (None, 0)
}

fn git_archive(repo: &Path, commit: &str, prefixes: &[&str], dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    let archive = Command::new("git")
        .args([
            "--no-optional-locks",
            "archive",
            "--format=tar",
            commit,
            "--",
        ])
        .args(prefixes)
        .current_dir(repo)
        .output()?;
    if !archive.status.success() {
        return Err(std::io::Error::other(
            String::from_utf8_lossy(&archive.stderr).into_owned(),
        ));
    }
    let mut tar = Command::new("tar")
        .args(["-x", "-C"])
        .arg(dest)
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    std::io::Write::write_all(tar.stdin.as_mut().expect("piped"), &archive.stdout)?;
    let status = tar.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("tar -x failed"))
    }
}

fn copy_binary(docs: &Docs, path: &str, dest: &Path) -> std::io::Result<()> {
    let bytes = match docs.source() {
        Source::Dir(root) => fs::read(root.join(path))?,
        Source::Git { repo, commit } => git_bytes(repo, commit, path).unwrap_or_default(),
    };
    write_file(&dest.join(path), &bytes)
}

fn write_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}

/// The repository root: the parent of this crate's manifest directory.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("hadronyche-tools sits at the repository root")
        .to_path_buf()
}

/// True when `commit` is in the git repository at `repo` (false in a
/// shallow clone that lacks it): tests that read a red-phase tree skip
/// without it.
pub fn commit_exists(repo: &Path, commit: &str) -> bool {
    Command::new("git")
        .args([
            "--no-optional-locks",
            "cat-file",
            "-e",
            &format!("{commit}^{{commit}}"),
        ])
        .current_dir(repo)
        .status()
        .is_ok_and(|s| s.success())
}

/// Normalises a repository-relative path as Python's `os.path.normpath`
/// (`\\` read as `/`), except that the repository root is `""`, not `"."`.
/// A leading `..` is kept: [`outside`] tells such a path apart.
pub fn norm(path: &str) -> String {
    let p = normpath(&path.replace('\\', "/"));
    if p == "." { String::new() } else { p }
}

/// The directory part of a repository-relative path (`""` at the root).
pub fn dirname(path: &str) -> &str {
    path.rfind('/').map_or("", |i| &path[..i])
}

/// `rel` resolved against the directory `base` (Python
/// `os.path.normpath(os.path.join(base, rel))`).
///
/// [`PathNorm::PyNormpath`](super::policy::PathNorm::PyNormpath): a leading
/// `..` is kept, so a link that climbs above the repository yields a path
/// outside the documents, which no [`Docs`] holds.
pub fn join(base: &str, rel: &str) -> String {
    if base.is_empty() {
        normpath(rel)
    } else {
        normpath(&format!("{base}/{rel}"))
    }
}

/// True when a normalised path lies outside the repository (`..`, `/`, `.`).
pub fn outside(key: &str) -> bool {
    key.starts_with('/') || key == ".." || key.starts_with("../")
}

fn under(path: &str, prefix: &str) -> bool {
    prefix.is_empty()
        || path
            .strip_prefix(prefix)
            .is_some_and(|r| r.starts_with('/'))
}

fn walk(root: &Path, prefix: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root.join(prefix)];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(rel) = path.strip_prefix(root) {
                out.insert(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out
}

fn git_tree(repo: &Path, commit: &str) -> Result<BTreeSet<String>, String> {
    let out = Command::new("git")
        .args([
            "--no-optional-locks",
            "ls-tree",
            "-r",
            "--name-only",
            commit,
        ])
        .current_dir(repo)
        .output()
        .map_err(|e| format!("git ls-tree {commit}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git ls-tree {commit} failed: {}",
            String::from_utf8_lossy(&out.stderr).py_strip()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

fn git_bytes(repo: &Path, commit: &str, path: &str) -> Option<Vec<u8>> {
    let out = Command::new("git")
        .args(["--no-optional-locks", "show", &format!("{commit}:{path}")])
        .current_dir(repo)
        .output()
        .ok()?;
    out.status.success().then_some(out.stdout)
}

fn git_show(repo: &Path, commit: &str, path: &str) -> Option<String> {
    String::from_utf8(git_bytes(repo, commit, path)?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doccheck::policy::TomlReader;

    fn spec(run: fn(&Docs) -> Report) -> CheckSpec {
        CheckSpec {
            id: "000-00",
            run,
            script_env: &[],
            targets: &[],
            keywords: &[],
            known_differences: &[],
            denials: &[],
            toml: TomlReader::Tomllib,
        }
    }

    /// Reads wire.md, then the file its first link names (wherever it
    /// lies), and fails AC1 when the link target lacks "ok".
    fn follows_a_link(docs: &Docs) -> Report {
        let mut r = Report::default();
        let target = docs.read("doc/design/spec/wire.md").and_then(|t| {
            let link = t.split_once("](")?.1.split_once(')')?.0;
            let rel = link.split('#').next()?.to_string();
            docs.read(&join("doc/design/spec", &rel))
        });
        r.check("AC1", target.is_some_and(|t| t.contains("ok")), "target");
        r
    }

    #[test]
    fn every_text_a_check_reads_is_guarded() {
        let base = Docs::dir("/nonexistent-doccheck-tree")
            .with("doc/design/spec/wire.md", "see [r](../../../README.md#x)\n")
            .with("README.md", "ok\n")
            .with("doc/other.md", "caf\u{e9} never read\n");
        let s = spec(follows_a_link);
        assert!(
            s.evaluate(&base).passed(),
            "a file the check does not read is not its input"
        );
        let bad = base.with("README.md", "ok\nna\u{ef}ve\n");
        let r = s.evaluate(&bad);
        assert_eq!(r.failed_criteria(), ["REPERTOIRE".to_string()].into());
        assert!(
            r.render().contains("README.md:2:3: U+00EF"),
            "{}",
            r.render()
        );
        // with the character, the check's own verdict is not reported
        let both = bad.with("README.md", "na\u{ef}ve\n");
        assert_eq!(
            s.evaluate(&both).failed_criteria(),
            ["REPERTOIRE".to_string()].into()
        );
    }

    fn backtracks(docs: &Docs) -> Report {
        let mut r = Report::default();
        let text = docs.read("doc/x.md").unwrap_or_default();
        let line = text.lines().nth(1).unwrap_or_default();
        let found = crate::doccheck::text::has(r"\b[A-Z][a-z0-9]+(?:[A-Z][A-Za-z0-9]*)+\b", line);
        r.check("AC3", !found, "no CamelCase word");
        r
    }

    #[test]
    fn a_regex_engine_failure_is_red_with_pattern_and_place() {
        let s = spec(backtracks);
        let ok = Docs::dir("/nonexistent-doccheck-tree").with("doc/x.md", "intro\nplain words\n");
        assert!(s.evaluate(&ok).passed());
        let hay = format!("intro\nAa{}_ x\n", "B".repeat(26));
        let r = s.evaluate(&ok.with("doc/x.md", hay));
        assert_eq!(
            r.failed_criteria(),
            [REGEX_LIMIT.to_string()].into(),
            "{}",
            r.render()
        );
        let f = &r.failures[0];
        assert_eq!(
            f.location,
            Some(crate::doccheck::report::Loc {
                file: "doc/x.md".into(),
                line: 2
            })
        );
        assert!(f.message.contains("[A-Z][a-z0-9]+"), "{}", f.message);
        assert!(
            crate::doccheck::rx::take_failures().is_empty(),
            "evaluate leaves no record behind"
        );
    }

    #[test]
    fn paths_normalise_like_os_path() {
        assert_eq!(
            norm("doc/design/spec/../02-constants.md"),
            "doc/design/02-constants.md"
        );
        assert_eq!(
            join("doc/design/spec", "wire.md"),
            "doc/design/spec/wire.md"
        );
        assert_eq!(join("", "./a/b"), "a/b");
        let up = join("doc/design/spec", "../../../../doc/design/spec/wire.md");
        assert_eq!(up, "../doc/design/spec/wire.md", "a leading .. is kept");
        assert!(outside(&up));
        assert!(
            !Docs::repo().is_file(&up),
            "nothing outside the tree is a document"
        );
        assert!(
            Docs::repo()
                .read("../hadronyche-tools/Cargo.toml")
                .is_none()
        );
        assert_eq!(dirname("doc/design/spec/wire.md"), "doc/design/spec");
        assert_eq!(dirname("README.md"), "");
    }

    #[test]
    fn overrides_replace_and_delete_in_memory() {
        let docs = Docs::repo();
        let wire = "doc/design/spec/wire.md";
        assert!(docs.is_file(wire));
        let changed = docs.mutate(wire, |t| format!("{t}\nextra"));
        assert!(changed.read(wire).unwrap().ends_with("\nextra"));
        assert!(!docs.read(wire).unwrap().ends_with("\nextra"));
        let gone = docs.without(wire);
        assert!(gone.read(wire).is_none());
        assert!(!gone.is_file(wire));
        assert!(
            !gone
                .files_under("doc/design/spec")
                .contains(&wire.to_string())
        );
        let added = docs.with("doc/design/spec/new.md", "x");
        assert!(
            added
                .files_under("doc/design/spec")
                .contains(&"doc/design/spec/new.md".into())
        );
    }

    #[test]
    fn glob_lists_one_directory_by_stem() {
        let docs = Docs::repo();
        let found = docs.glob("doc/design/tickets/004", "004-22-");
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].ends_with(".md"));
    }

    #[test]
    fn a_commit_reads_through_git() {
        let docs = Docs::at_commit(repo_root(), "HEAD").expect("HEAD is a commit");
        assert!(docs.is_file("Cargo.toml"));
        assert!(docs.read("Cargo.toml").unwrap().contains("[workspace]"));
        assert!(docs.read("no/such/file").is_none());
        assert!(commit_exists(&repo_root(), "HEAD"));
        let missing = "0000000000000000000000000000000000000000";
        assert!(
            Docs::at_commit(repo_root(), missing).is_err(),
            "never an empty tree"
        );
        assert!(!commit_exists(
            &repo_root(),
            "0000000000000000000000000000000000000000"
        ));
    }
}
