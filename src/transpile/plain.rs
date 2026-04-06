use crate::ast::{Document, Node};

pub fn transpile(document: &Document) -> String {
    let mut lines = Vec::new();

    for (key, value) in document.ordered_entries() {
        render_mapping_entry(&mut lines, 0, key.as_str(), value);
    }

    lines.join("\n")
}

fn render_mapping_entry(lines: &mut Vec<String>, indent: usize, key: &str, value: &Node) {
    let prefix = " ".repeat(indent);

    match value {
        Node::Scalar { value: scalar, .. } => lines.push(format!("{prefix}{key}: {scalar}")),
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
        Node::Scalar { value: scalar, .. } => lines.push(format!("{prefix}- {scalar}")),
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
