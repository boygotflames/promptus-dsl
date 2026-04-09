use std::collections::HashSet;

use crate::ast::{Document, Node, TopLevelKey};
use crate::diagnostics::{DiagnosticBag, Span};

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
        diagnostics.semantic_error("missing required key: `agent`", Some(Span::new(1, 1)));
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
                    diagnostics.semantic_error(
                        format!("duplicate key `{}` in `{path}`", entry.key),
                        Some(entry.span),
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
        Node::Scalar { span, .. } => {
            diagnostics.semantic_error(format!("`{}` must not be empty", key.as_str()), Some(*span))
        }
        other => diagnostics.semantic_error(
            format!(
                "`{}` must be a scalar value, found {}",
                key.as_str(),
                other.kind_name()
            ),
            Some(other.span()),
        ),
    }
}

fn validate_prompt_field(document: &Document, key: TopLevelKey, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(key) else {
        return;
    };

    if !matches!(node, Node::Scalar { .. } | Node::Mapping { .. }) {
        diagnostics.semantic_error(
            format!(
                "`{}` must be a scalar or mapping, found {}",
                key.as_str(),
                node.kind_name()
            ),
            Some(node.span()),
        );
    }
}

fn validate_sequence_field(document: &Document, key: TopLevelKey, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(key) else {
        return;
    };

    let Some(values) = node.as_sequence() else {
        diagnostics.semantic_error(
            format!("`{}` must be a sequence of scalar values", key.as_str()),
            Some(node.span()),
        );
        return;
    };

    for value in values {
        if !matches!(value, Node::Scalar { .. }) {
            diagnostics.semantic_error(
                format!("`{}` may only contain scalar list items", key.as_str()),
                Some(value.span()),
            );
            break;
        }
    }
}

fn validate_output_field(document: &Document, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(TopLevelKey::Output) else {
        return;
    };

    if !matches!(node, Node::Scalar { .. } | Node::Mapping { .. }) {
        diagnostics.semantic_error("`output` must be a scalar or mapping", Some(node.span()));
    }
}

fn validate_vars_field(document: &Document, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(TopLevelKey::Vars) else {
        return;
    };

    let Some(entries) = node.as_mapping() else {
        diagnostics.semantic_error(
            "`vars` must be a mapping of scalar values",
            Some(node.span()),
        );
        return;
    };

    for entry in entries {
        if !matches!(&entry.value, Node::Scalar { .. }) {
            diagnostics.semantic_error(
                format!("`vars.{}` must be a scalar value", entry.key),
                Some(entry.span),
            );
        }
    }
}
