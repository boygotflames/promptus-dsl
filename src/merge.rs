//! Document merge logic for multi-file includes.
//!
//! Merge semantics (from SPEC.md):
//! - Scalar keys (agent/system/user/output): conflict = E115
//! - Sequence keys (memory/tools/constraints):
//!   concatenate; tools+constraints deduplicated; memory not
//! - vars: merged, parent wins on conflict, no error
//! - include: consumed during composition, never in output

use std::collections::HashSet;
use std::path::Path;

use crate::ast::{Document, Node};
use crate::diagnostics::{Diagnostic, DiagnosticBag, Span};

/// Merge `included` into `parent`.
///
/// `included_path` is used for error messages only.
/// Modifies `parent` in place.
/// Returns a `DiagnosticBag` with any E115 conflict diagnostics.
pub fn merge_into(
    parent: &mut Document,
    included: Document,
    included_path: &Path,
) -> DiagnosticBag {
    let mut diagnostics = DiagnosticBag::new();
    let path_str = included_path.display().to_string();

    // Scalar keys: conflict = E115
    merge_scalar(
        &mut parent.agent,
        included.agent,
        "agent",
        &path_str,
        &mut diagnostics,
    );
    merge_scalar(
        &mut parent.system,
        included.system,
        "system",
        &path_str,
        &mut diagnostics,
    );
    merge_scalar(
        &mut parent.user,
        included.user,
        "user",
        &path_str,
        &mut diagnostics,
    );
    merge_scalar(
        &mut parent.output,
        included.output,
        "output",
        &path_str,
        &mut diagnostics,
    );

    // Sequence keys: concatenate + optional dedup
    merge_sequence(&mut parent.memory, included.memory, false); // memory: no dedup
    merge_sequence(&mut parent.tools, included.tools, true); // tools: dedup
    merge_sequence(&mut parent.constraints, included.constraints, true); // constraints: dedup

    // vars: merge, parent wins
    merge_vars(&mut parent.vars, included.vars);

    // include: consumed — do not copy to parent
    // (included.include is dropped here)

    diagnostics
}

/// Scalar merge: if both are `Some`, emit E115.
/// Parent value is preserved on conflict.
fn merge_scalar(
    parent_field: &mut Option<Node>,
    included_field: Option<Node>,
    key: &str,
    included_path: &str,
    diagnostics: &mut DiagnosticBag,
) {
    match (&*parent_field, included_field) {
        (Some(_), Some(_)) => {
            diagnostics.push(
                Diagnostic::semantic_error(
                    format!(
                        "include conflict: `{}` is defined in both parent and `{}`",
                        key, included_path
                    ),
                    None,
                )
                .with_code("E115"),
            );
            // Parent value preserved — field unchanged
        }
        (None, Some(included_val)) => {
            *parent_field = Some(included_val);
        }
        (Some(_), None) | (None, None) => {
            // No conflict, nothing to merge
        }
    }
}

/// Sequence merge: concatenate; optionally deduplicate by scalar value
/// (first occurrence wins).
fn merge_sequence(parent_field: &mut Option<Node>, included_field: Option<Node>, dedup: bool) {
    let included_items = match included_field {
        Some(Node::Sequence { values, .. }) => values,
        Some(_) | None => return,
    };

    if included_items.is_empty() {
        return;
    }

    match parent_field {
        Some(Node::Sequence { values, .. }) => {
            if dedup {
                let existing: HashSet<String> = values
                    .iter()
                    .filter_map(|n| {
                        if let Node::Scalar { value, .. } = n {
                            Some(value.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                for item in included_items {
                    if let Node::Scalar { ref value, .. } = item {
                        if !existing.contains(value) {
                            values.push(item);
                        }
                    } else {
                        values.push(item);
                    }
                }
            } else {
                values.extend(included_items);
            }
        }
        None => {
            // Parent had no sequence — use included as base
            *parent_field = Some(Node::Sequence {
                values: included_items,
                span: Span::new(0, 0),
            });
        }
        Some(_) => {
            // Parent has wrong node type — validator will catch this; skip merge
        }
    }
}

/// vars merge: both are Mapping nodes. Parent entries win on key conflict.
fn merge_vars(parent_field: &mut Option<Node>, included_field: Option<Node>) {
    let included_entries = match included_field {
        Some(Node::Mapping { entries, .. }) => entries,
        Some(_) | None => return,
    };

    if included_entries.is_empty() {
        return;
    }

    match parent_field {
        Some(Node::Mapping { entries, .. }) => {
            let existing_keys: HashSet<String> = entries.iter().map(|e| e.key.clone()).collect();

            for entry in included_entries {
                if !existing_keys.contains(&entry.key) {
                    entries.push(entry);
                }
                // Parent wins on conflict — skip duplicates
            }
        }
        None => {
            *parent_field = Some(Node::Mapping {
                entries: included_entries,
                span: Span::new(0, 0),
            });
        }
        Some(_) => {
            // Wrong type — validator will catch it
        }
    }
}
