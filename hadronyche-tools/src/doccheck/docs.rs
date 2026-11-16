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

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use super::pycompat::{PyStr, normpath};

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
}

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
