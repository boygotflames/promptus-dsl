use crate::ast::{Document, Node};

pub fn format_document(document: &Document) -> String {
    let mut lines = Vec::new();

    for (key, value) in document.ordered_entries() {
        render_mapping_entry(&mut lines, 0, key.as_str(), value);
    }

    lines.join("\n")
}

pub fn format_scalar(value: &str) -> String {
    if is_bare_scalar(value) {
        value.to_owned()
    } else {
        quote_scalar(value)
    }
}

fn render_mapping_entry(lines: &mut Vec<String>, indent: usize, key: &str, value: &Node) {
    let prefix = " ".repeat(indent);

    match value {
        Node::Scalar { value: scalar, .. } => {
            lines.push(format!("{prefix}{key}: {}", format_scalar(scalar)))
        }
        Node::Mapping { entries, .. } => {
            lines.push(format!("{prefix}{key}:"));
            for entry in entries {
                render_mapping_entry(lines, indent + 2, &entry.key, &entry.value);
            }
        }
        Node::Sequence { values, .. } => {
            lines.push(format!("{prefix}{key}:"));
            for item in values {
                render_list_item(lines, indent + 2, item);
            }
        }
    }
}

fn render_list_item(lines: &mut Vec<String>, indent: usize, value: &Node) {
    let prefix = " ".repeat(indent);

    match value {
        Node::Scalar { value: scalar, .. } => {
            lines.push(format!("{prefix}- {}", format_scalar(scalar)))
        }
        Node::Mapping { entries, .. } => {
            lines.push(format!("{prefix}-"));
            for entry in entries {
                render_mapping_entry(lines, indent + 2, &entry.key, &entry.value);
            }
        }
        Node::Sequence { values, .. } => {
            lines.push(format!("{prefix}-"));
            for item in values {
                render_list_item(lines, indent + 2, item);
            }
        }
    }
}

fn is_bare_scalar(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn quote_scalar(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');

    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            _ => quoted.push(ch),
        }
    }

    quoted.push('"');
    quoted
}
