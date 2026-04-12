//! Multi-file include resolution.
//! Packet 28: file loading and path resolution only.
//! Merge logic is in src/merge.rs (Packet 29).

use std::path::{Path, PathBuf};

use crate::ast::{Document, Node};
use crate::diagnostics::{Diagnostic, DiagnosticBag};

/// Resolve the list of include paths from a document's `include` field.
/// Returns a `Vec` of absolute `PathBuf`s to include, in order.
///
/// Paths are resolved relative to `source_dir`. Absolute paths are rejected
/// (E115). An empty path string is rejected (E115). The file is NOT checked
/// for existence here — that happens at load time in the merge pass.
pub fn resolve_include_paths(
    document: &Document,
    source_dir: &Path,
) -> Result<Vec<PathBuf>, DiagnosticBag> {
    let include_node = match &document.include {
        None => return Ok(vec![]),
        Some(n) => n,
    };

    let mut paths = Vec::new();
    let mut errors = DiagnosticBag::new();

    match include_node {
        Node::Scalar { value, span } => match resolve_one(value, source_dir, *span) {
            Ok(p) => paths.push(p),
            Err(d) => errors.push(d),
        },
        Node::Sequence { values, .. } => {
            for item in values {
                if let Node::Scalar { value, span } = item {
                    match resolve_one(value, source_dir, *span) {
                        Ok(p) => paths.push(p),
                        Err(d) => errors.push(d),
                    }
                }
            }
        }
        Node::Mapping { span, .. } => {
            errors.push(
                Diagnostic::semantic_error(
                    "include must be a scalar path or a sequence of paths, not a mapping",
                    Some(*span),
                )
                .with_code("E023"),
            );
        }
    }

    if errors.has_errors() {
        Err(errors)
    } else {
        Ok(paths)
    }
}

fn resolve_one(
    raw: &str,
    source_dir: &Path,
    span: crate::diagnostics::Span,
) -> Result<PathBuf, Diagnostic> {
    if raw.trim().is_empty() {
        return Err(
            Diagnostic::semantic_error("include path must not be empty", Some(span))
                .with_code("E115"),
        );
    }

    let path = Path::new(raw.trim());
    if path.is_absolute() {
        return Err(Diagnostic::semantic_error(
            "include paths must be relative, not absolute",
            Some(span),
        )
        .with_code("E115"));
    }

    Ok(source_dir.join(path))
}

/// Check for circular includes using the current inclusion chain (stack of
/// canonical paths).
///
/// Returns an `E116` diagnostic if `candidate` is already in `chain`.
/// Returns `None` if no cycle is detected or if the candidate cannot be
/// canonicalized (i.e., does not yet exist on disk — checked at load time).
pub fn check_circular(candidate: &Path, chain: &[PathBuf]) -> Option<Diagnostic> {
    let canonical = candidate.canonicalize().ok()?;
    if chain
        .iter()
        .any(|p| p.canonicalize().ok().as_deref() == Some(canonical.as_path()))
    {
        Some(
            Diagnostic::semantic_error(
                format!("circular include detected: `{}`", candidate.display()),
                None,
            )
            .with_code("E116"),
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Node;

    fn scalar_doc(path: &str) -> Document {
        Document {
            include: Some(Node::scalar(path)),
            ..Default::default()
        }
    }

    fn sequence_doc(paths: &[&str]) -> Document {
        Document {
            include: Some(Node::sequence(
                paths.iter().map(|p| Node::scalar(*p)).collect(),
            )),
            ..Default::default()
        }
    }

    fn mapping_doc() -> Document {
        Document {
            include: Some(Node::mapping(vec![])),
            ..Default::default()
        }
    }

    #[test]
    fn resolve_include_paths_returns_empty_for_no_include() {
        let doc = Document::default();
        let dir = Path::new(".");
        let result = resolve_include_paths(&doc, dir).expect("should be Ok");
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_include_paths_rejects_absolute_path() {
        // Use a platform-appropriate absolute path
        #[cfg(windows)]
        let absolute = "C:/absolute/path.llm";
        #[cfg(not(windows))]
        let absolute = "/absolute/path.llm";

        let doc = scalar_doc(absolute);
        let dir = Path::new(".");
        let result = resolve_include_paths(&doc, dir);
        let errors = result.expect_err("absolute path must be rejected");
        assert!(errors.iter().any(|d| d.code == Some("E115")));
    }

    #[test]
    fn resolve_include_paths_rejects_empty_path() {
        let doc = scalar_doc("");
        let dir = Path::new(".");
        let result = resolve_include_paths(&doc, dir);
        let errors = result.expect_err("empty path must be rejected");
        assert!(errors.iter().any(|d| d.code == Some("E115")));
    }

    #[test]
    fn resolve_include_paths_scalar_resolves_relative() {
        let doc = scalar_doc("other.llm");
        let dir = Path::new("/some/project");
        let result = resolve_include_paths(&doc, dir).expect("should be Ok");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Path::new("/some/project/other.llm"));
    }

    #[test]
    fn resolve_include_paths_sequence_resolves_multiple() {
        let doc = sequence_doc(&["a.llm", "b.llm"]);
        let dir = Path::new("/project");
        let result = resolve_include_paths(&doc, dir).expect("should be Ok");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Path::new("/project/a.llm"));
        assert_eq!(result[1], Path::new("/project/b.llm"));
    }

    #[test]
    fn resolve_include_paths_mapping_is_rejected() {
        let doc = mapping_doc();
        let dir = Path::new(".");
        let result = resolve_include_paths(&doc, dir);
        let errors = result.expect_err("mapping include must be rejected");
        assert!(errors.iter().any(|d| d.code == Some("E023")));
    }

    #[test]
    fn check_circular_detects_self_include() {
        // Use a real path that exists — the current executable
        let exe = std::env::current_exe().expect("current_exe must be available");
        // Chain contains the same file
        let chain = vec![exe.clone()];
        let diag = check_circular(&exe, &chain);
        assert!(diag.is_some());
        let diag = diag.unwrap();
        assert_eq!(diag.code, Some("E116"));
    }

    #[test]
    fn check_circular_allows_unrelated_files() {
        // Use a path that doesn't exist — canonicalize returns Err,
        // so check_circular returns None (no false positive)
        let candidate = Path::new("/nonexistent/file_a.llm");
        let chain = vec![PathBuf::from("/nonexistent/file_b.llm")];
        let diag = check_circular(candidate, &chain);
        assert!(diag.is_none());
    }

    #[test]
    fn resolve_one_with_whitespace_trimmed_path() {
        let doc = scalar_doc("  relative/path.llm  ");
        let dir = Path::new("/base");
        let result = resolve_include_paths(&doc, dir).expect("should be Ok");
        assert_eq!(result[0], Path::new("/base/relative/path.llm"));
    }
}
