use crate::ast::{Document, Node, TopLevelKey};
use crate::diagnostics::DiagnosticBag;

pub fn validate_document(document: &Document) -> DiagnosticBag {
    let mut diagnostics = DiagnosticBag::new();

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

fn validate_scalar_field(document: &Document, key: TopLevelKey, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(key) else {
        return;
    };

    match node {
        Node::Scalar(value) if !value.trim().is_empty() => {}
        Node::Scalar(_) => diagnostics.error(format!("`{}` must not be empty", key.as_str()), None),
        other => diagnostics.error(
            format!(
                "`{}` must be a scalar value, found {}",
                key.as_str(),
                other.kind_name()
            ),
            None,
        ),
    }
}

fn validate_prompt_field(document: &Document, key: TopLevelKey, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(key) else {
        return;
    };

    if !matches!(node, Node::Scalar(_) | Node::Mapping(_)) {
        diagnostics.error(
            format!(
                "`{}` must be a scalar or mapping, found {}",
                key.as_str(),
                node.kind_name()
            ),
            None,
        );
    }
}

fn validate_sequence_field(document: &Document, key: TopLevelKey, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(key) else {
        return;
    };

    let Some(values) = node.as_sequence() else {
        diagnostics.error(
            format!("`{}` must be a sequence of scalar values", key.as_str()),
            None,
        );
        return;
    };

    for value in values {
        if !matches!(value, Node::Scalar(_)) {
            diagnostics.error(
                format!("`{}` may only contain scalar list items", key.as_str()),
                None,
            );
            break;
        }
    }
}

fn validate_output_field(document: &Document, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(TopLevelKey::Output) else {
        return;
    };

    if !matches!(node, Node::Scalar(_) | Node::Mapping(_)) {
        diagnostics.error("`output` must be a scalar or mapping", None);
    }
}

fn validate_vars_field(document: &Document, diagnostics: &mut DiagnosticBag) {
    let Some(node) = document.get(TopLevelKey::Vars) else {
        return;
    };

    let Some(entries) = node.as_mapping() else {
        diagnostics.error("`vars` must be a mapping of scalar values", None);
        return;
    };

    for (name, value) in entries {
        if !matches!(value, Node::Scalar(_)) {
            diagnostics.error(format!("`vars.{name}` must be a scalar value"), None);
        }
    }
}
