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
        Node::Scalar(scalar) => lines.push(format!("{prefix}{key}: {scalar}")),
        Node::Mapping(entries) => {
            lines.push(format!("{prefix}{key}:"));
            for (child_key, child_value) in entries {
                render_mapping_entry(lines, indent + 2, child_key, child_value);
            }
        }
        Node::Sequence(items) => {
            lines.push(format!("{prefix}{key}:"));
            for item in items {
                render_list_item(lines, indent + 2, item);
            }
        }
    }
}

fn render_list_item(lines: &mut Vec<String>, indent: usize, value: &Node) {
    let prefix = " ".repeat(indent);

    match value {
        Node::Scalar(scalar) => lines.push(format!("{prefix}- {scalar}")),
        Node::Mapping(entries) => {
            lines.push(format!("{prefix}-"));
            for (child_key, child_value) in entries {
                render_mapping_entry(lines, indent + 2, child_key, child_value);
            }
        }
        Node::Sequence(items) => {
            lines.push(format!("{prefix}-"));
            for item in items {
                render_list_item(lines, indent + 2, item);
            }
        }
    }
}
