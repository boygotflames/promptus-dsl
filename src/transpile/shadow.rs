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
        Node::Scalar(scalar) => lines.push(format!("{path} = {}", quote(scalar))),
        Node::Mapping(entries) => {
            for (child_key, child_value) in entries {
                flatten_node(lines, format!("{path}.{child_key}"), child_value);
            }
        }
        Node::Sequence(items) => {
            for (index, item) in items.iter().enumerate() {
                flatten_node(lines, format!("{path}[{index}]"), item);
            }
        }
    }
}
