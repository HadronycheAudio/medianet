//! Markdown links and their resolution across files.
//!
//! A link `[text](path#anchor)` resolves when `path` (relative to the
//! linking file's directory) is a file of the [`Docs`] and, when an anchor is
//! given, the anchor is a heading anchor of that file. A link with no path
//! (`#anchor`) resolves against the linking document. `http` links are not
//! checked.

use std::collections::BTreeSet;

use super::docs::{Docs, join};
use super::markdown::Markdown;
use super::text::rx;

/// One Markdown link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub text: String,
    pub target: String,
}

impl Link {
    /// The path part (before `#`), possibly empty.
    pub fn path(&self) -> &str {
        self.target.split('#').next().unwrap_or("")
    }

    /// The anchor part (after `#`).
    pub fn anchor(&self) -> Option<&str> {
        self.target.split_once('#').map(|(_, a)| a)
    }
}

/// The links of `text` (target without spaces).
pub fn links(text: &str) -> Vec<Link> {
    rx(r"\[([^\]]*)\]\(([^)\s]*)\)")
        .captures_iter(text)
        .map(|c| Link {
            text: c[1].to_string(),
            target: c[2].to_string(),
        })
        .collect()
}

/// Why a link does not resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dangling {
    /// The path names no file; carries the resolved path.
    NoFile(String),
    /// The anchor is not a heading of the file; carries the resolved path.
    NoAnchor(String),
}

/// The anchors of the Markdown file `path` of `docs`, if it exists.
pub fn file_slugs(docs: &Docs, path: &str) -> Option<BTreeSet<String>> {
    docs.read(path).map(|t| Markdown::parse(&t).slugs())
}

/// Resolves `link` written in a file of directory `base_dir`, whose own
/// anchors are `self_slugs`. `Ok(None)` for an external or self link,
/// `Ok(Some(path))` for a resolved file link.
pub fn resolve(
    docs: &Docs,
    base_dir: &str,
    self_slugs: &BTreeSet<String>,
    link: &Link,
) -> Result<Option<String>, Dangling> {
    if link.target.starts_with("http") {
        return Ok(None);
    }
    if link.path().is_empty() {
        return match link.anchor() {
            Some(a) if !self_slugs.contains(a) => Err(Dangling::NoAnchor(String::new())),
            _ => Ok(None),
        };
    }
    let target = join(base_dir, link.path());
    if !docs.is_file(&target) {
        return Err(Dangling::NoFile(target));
    }
    if let (Some(anchor), Some(slugs)) = (link.anchor(), file_slugs(docs, &target))
        && !slugs.contains(anchor)
    {
        return Err(Dangling::NoAnchor(target));
    }
    Ok(Some(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_split_path_and_anchor() {
        let l = links("see [A](../02-constants.md#constants) and [B](#hello) and [C](https://x)");
        assert_eq!(l.len(), 3);
        assert_eq!(l[0].path(), "../02-constants.md");
        assert_eq!(l[0].anchor(), Some("constants"));
        assert_eq!(l[1].path(), "");
        assert_eq!(l[2].anchor(), None);
    }

    #[test]
    fn resolution_across_files() {
        let docs = Docs::repo();
        let wire = Markdown::parse(&docs.read("doc/design/spec/wire.md").unwrap());
        let slugs = wire.slugs();
        let base = "doc/design/spec";
        let ok = Link {
            text: "x".into(),
            target: "#hello".into(),
        };
        assert_eq!(resolve(&docs, base, &slugs, &ok), Ok(None));
        let file = Link {
            text: "x".into(),
            target: "../02-constants.md".into(),
        };
        assert_eq!(
            resolve(&docs, base, &slugs, &file),
            Ok(Some("doc/design/02-constants.md".into()))
        );
        let gone = Link {
            text: "x".into(),
            target: "../nope.md#a".into(),
        };
        assert!(matches!(
            resolve(&docs, base, &slugs, &gone),
            Err(Dangling::NoFile(_))
        ));
        let bad = Link {
            text: "x".into(),
            target: "#no-such-heading".into(),
        };
        assert!(matches!(
            resolve(&docs, base, &slugs, &bad),
            Err(Dangling::NoAnchor(_))
        ));
    }
}
