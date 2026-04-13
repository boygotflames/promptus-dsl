//! Document composition: resolve includes, load files,
//! detect circular refs, and merge into a single Document.

use std::path::{Path, PathBuf};

use crate::ast::Document;
use crate::diagnostics::{Diagnostic, DiagnosticBag};
use crate::include::{check_circular, resolve_include_paths};
use crate::merge::merge_into;

/// Compose a document by resolving and merging all includes.
///
/// `source_path`: the path of the root document (used for relative path
///   resolution and circular detection).
/// `chain`: the current inclusion stack. Pass `&[]` for the root call.
///
/// Returns the composed `Document` and a `DiagnosticBag` with any
/// E115/E116 errors (document is partially composed up to any error).
pub fn compose(
    mut document: Document,
    source_path: &Path,
    chain: &[PathBuf],
) -> (Document, DiagnosticBag) {
    let mut diagnostics = DiagnosticBag::new();

    let source_dir = source_path.parent().unwrap_or(Path::new("."));

    // Resolve include paths from the document's include field
    let include_paths = match resolve_include_paths(&document, source_dir) {
        Ok(paths) => paths,
        Err(errs) => {
            diagnostics.extend(errs);
            return (document, diagnostics);
        }
    };

    // Consume the include field — it must not appear in composed output
    document.include = None;

    if include_paths.is_empty() {
        return (document, diagnostics);
    }

    // Build the new chain for circular detection
    let mut new_chain: Vec<PathBuf> = chain.to_vec();
    if let Ok(canonical) = source_path.canonicalize() {
        new_chain.push(canonical);
    }

    // Process each include in order
    for include_path in include_paths {
        // Circular detection
        if let Some(circ_diag) = check_circular(&include_path, &new_chain) {
            diagnostics.push(circ_diag);
            continue;
        }

        // Read the included file
        let content = match std::fs::read_to_string(&include_path) {
            Ok(s) => s,
            Err(e) => {
                diagnostics.push(
                    Diagnostic::semantic_error(
                        format!(
                            "cannot read include file `{}`: {}",
                            include_path.display(),
                            e
                        ),
                        None,
                    )
                    .with_code("E115"),
                );
                continue;
            }
        };

        // Parse the included file
        let included_doc = match crate::parse_str(&content) {
            Ok(doc) => doc,
            Err(errs) => {
                diagnostics.extend(errs);
                continue;
            }
        };

        // Recursively compose the included document
        let (composed_included, child_diags) = compose(included_doc, &include_path, &new_chain);
        diagnostics.extend(child_diags);

        // Merge into parent
        let merge_diags = merge_into(&mut document, composed_included, &include_path);
        diagnostics.extend(merge_diags);
    }

    (document, diagnostics)
}
