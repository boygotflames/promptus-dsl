use crate::ast::{Document, Node};

use super::quote;

pub fn transpile(document: &Document) -> String {
    let mut lines = Vec::new();

    for (key, value) in document.ordered_entries() {
        flatten_node(&mut lines, key.as_str().to_owned(), value);
    }

    lines.join("\n")
}

fn flatten_node(lines: &mut Vec<String>, path: String, value: &Node) {
    match value {
        Node::Scalar { value: scalar, .. } => lines.push(format!("{path} = {}", quote(scalar))),
        Node::Mapping { entries, .. } => {
            for entry in entries {
                flatten_node(lines, format!("{path}.{}", entry.key), &entry.value);
            }
        }
        Node::Sequence { values, .. } => {
            for (index, item) in values.iter().enumerate() {
                flatten_node(lines, format!("{path}[{index}]"), item);
            }
        }
    }
}
