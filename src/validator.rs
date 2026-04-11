use std::collections::HashSet;

use crate::ast::{Document, Node, TopLevelKey};
use crate::diagnostics::{Diagnostic, DiagnosticBag, Span};

pub fn validate_document(document: &Document) -> DiagnosticBag {
    let mut diagnostics = DiagnosticBag::new();

    validate_required_structure(document, &mut diagnostics);

    for key in TopLevelKey::ordered() {
        if let Some(node) = document.get(key) {
            validate_duplicate_keys(node, key.as_str(), &mut diagnostics);
        }
    }

    validate_scalar_field(document, TopLevelKey::Agent, &mut diagnostics);
    validate_prompt_field(document, TopLevelKey::System, &mut diagnostics);
    validate_prompt_field(document, TopLevelKey::User, &mut diagnostics);
    validate_sequence_field(document, TopLevelKey::Memory, &mut diagnostics);
    validate_sequence_field(document, TopLevelKey::Tools, &mut diagnostics);
    validate_output_field(document, &mut diagnostics);
    validate_sequence_field(document, TopLevelKey::Constraints, &mut diagnostics);
    validate_vars_field(document, &mut diagnostics);

    diagnostics
}

fn validate_required_structure(document: &Document, diagnostics: &mut DiagnosticBag) {
    if document.agent.is_none() {
        diagnostics.push(
            Diagnostic::semantic_error("missing required key: `agent`", Some(Span::new(1, 1)))
                .with_code("E101"),
        );
    }
    if document.system.is_none() {
        diagnostics.push(
            Diagnostic::semantic_error("missing required key: `system`", Some(Span::new(1, 1)))
                .with_code("E101"),
        );
    }
}

fn validate_duplicate_keys(node: &Node, path: &str, diagnostics: &mut DiagnosticBag) {
    match node {
        Node::Scalar { .. } => {}
        Node::Sequence { values, .. } => {
            for (index, item) in values.iter().enumerate() {
                validate_duplicate_keys(item, &format!("{path}[{index}]"), diagnostics);
            }
        }
        Node::Mapping { entries, .. } => {
            let mut seen = HashSet::new();

            for entry in entries {
                if !seen.insert(entry.key.as_str()) {
                    diagnostics.push(
                        Diagnostic::semantic_error(
                            format!("duplicate key `{}` in `{path}`", entry.key),
                            Some(entry.span),
                        )
                        .with_code("E102"),
                    );
                }

                validate_duplicate_keys(
                    &entry.value,
                    &format!("{path}.{}", entry.key),
                    diagnostics,
                );
            }
        }
    }
}

fn validate_scalar_field(document: &Document, key: TopLevelKey, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(key) else {
        return;
    };

    match node {
        Node::Scalar { value, .. } if !value.trim().is_empty() => {}
        Node::Scalar { span, .. } => diagnostics.push(
            Diagnostic::semantic_error(
                format!("`{}` must not be empty", key.as_str()),
                Some(*span),
            )
            .with_code("E103"),
        ),
        other => diagnostics.push(
            Diagnostic::semantic_error(
                format!(
                    "`{}` must be a scalar value, found {}",
                    key.as_str(),
                    other.kind_name()
                ),
                Some(other.span()),
            )
            .with_code("E104"),
        ),
    }
}

fn validate_prompt_field(document: &Document, key: TopLevelKey, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(key) else {
        return;
    };

    match node {
        Node::Scalar { value, span } => {
            if value.trim().is_empty() {
                diagnostics.push(
                    Diagnostic::semantic_error(
                        format!("`{}` must not be empty", key.as_str()),
                        Some(*span),
                    )
                    .with_code("E103"),
                );
            }
        }
        // E111: empty mapping block is a validation error.
        // Note: the parser requires at least one entry to produce a Mapping node,
        // so this guard is defensive and unreachable through parse_str.
        Node::Mapping { entries, span } => {
            if entries.is_empty() {
                diagnostics.push(
                    Diagnostic::semantic_error(
                        format!("`{}` mapping must not be empty", key.as_str()),
                        Some(*span),
                    )
                    .with_code("E111"),
                );
            }
        }
        other => {
            diagnostics.push(
                Diagnostic::semantic_error(
                    format!(
                        "`{}` must be a scalar or mapping, found {}",
                        key.as_str(),
                        other.kind_name()
                    ),
                    Some(other.span()),
                )
                .with_code("E105"),
            );
        }
    }
}

fn validate_sequence_field(document: &Document, key: TopLevelKey, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(key) else {
        return;
    };

    let Some(values) = node.as_sequence() else {
        diagnostics.push(
            Diagnostic::semantic_error(
                format!("`{}` must be a sequence of scalar values", key.as_str()),
                Some(node.span()),
            )
            .with_code("E106"),
        );
        return;
    };

    // E112: empty sequence is a validation error.
    // Note: the parser requires at least one item to produce a Sequence node,
    // so this guard is defensive and unreachable through parse_str.
    if values.is_empty() {
        diagnostics.push(
            Diagnostic::semantic_error(
                format!("`{}` sequence must not be empty", key.as_str()),
                Some(node.span()),
            )
            .with_code("E112"),
        );
        return;
    }

    for value in values {
        match value {
            Node::Scalar { value: v, span } if v.trim().is_empty() => {
                diagnostics.push(
                    Diagnostic::semantic_error(
                        format!("`{}` must not be empty", key.as_str()),
                        Some(*span),
                    )
                    .with_code("E103"),
                );
                break;
            }
            Node::Scalar { .. } => {}
            _ => {
                diagnostics.push(
                    Diagnostic::semantic_error(
                        format!("`{}` may only contain scalar list items", key.as_str()),
                        Some(value.span()),
                    )
                    .with_code("E107"),
                );
                break;
            }
        }
    }

    // E113: duplicate items for tools and constraints only (memory is exempt).
    if matches!(key, TopLevelKey::Tools | TopLevelKey::Constraints) {
        for (i, item) in values.iter().enumerate() {
            let Node::Scalar {
                value: current,
                span: current_span,
            } = item
            else {
                continue;
            };
            let is_duplicate = values[..i]
                .iter()
                .any(|earlier| matches!(earlier, Node::Scalar { value: v, .. } if v == current));
            if is_duplicate {
                diagnostics.push(
                    Diagnostic::semantic_error(
                        format!("duplicate item `{current}` in `{}`", key.as_str()),
                        Some(*current_span),
                    )
                    .with_code("E113"),
                );
            }
        }
    }
}

fn validate_output_field(document: &Document, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(TopLevelKey::Output) else {
        return;
    };

    match node {
        Node::Scalar { .. } => {}
        // E111: empty mapping block is a validation error.
        // Note: the parser requires at least one entry to produce a Mapping node,
        // so this guard is defensive and unreachable through parse_str.
        Node::Mapping { entries, span } => {
            if entries.is_empty() {
                diagnostics.push(
                    Diagnostic::semantic_error("`output` mapping must not be empty", Some(*span))
                        .with_code("E111"),
                );
            }
        }
        other => {
            diagnostics.push(
                Diagnostic::semantic_error(
                    "`output` must be a scalar or mapping",
                    Some(other.span()),
                )
                .with_code("E108"),
            );
        }
    }
}

fn validate_vars_field(document: &Document, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(TopLevelKey::Vars) else {
        return;
    };

    let Some(entries) = node.as_mapping() else {
        diagnostics.push(
            Diagnostic::semantic_error(
                "`vars` must be a mapping of scalar values",
                Some(node.span()),
            )
            .with_code("E109"),
        );
        return;
    };

    for entry in entries {
        match &entry.value {
            Node::Scalar { value, .. } if value.trim().is_empty() => {
                diagnostics.push(
                    Diagnostic::semantic_error(
                        format!("`vars.{}` must not be empty", entry.key),
                        Some(entry.span),
                    )
                    .with_code("E103"),
                );
            }
            Node::Scalar { .. } => {}
            _ => {
                diagnostics.push(
                    Diagnostic::semantic_error(
                        format!("`vars.{}` must be a scalar value", entry.key),
                        Some(entry.span),
                    )
                    .with_code("E110"),
                );
            }
        }
    }
}
