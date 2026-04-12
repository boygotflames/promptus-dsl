use crate::ast::{Document, Node};

use super::vars;
use super::{Emitter, quote};

pub struct JsonIrEmitter;

impl Emitter for JsonIrEmitter {
    fn emit(&self, document: &Document) -> String {
        // Expand {var_name} references before emitting.
        let expanded = vars::expand_document(document);
        render_object_from_pairs(
            expanded
                .ordered_entries()
                .into_iter()
                .map(|(key, value)| (key.as_str(), value))
                .collect(),
            0,
        )
    }
}

fn render_node(value: &Node, indent: usize) -> String {
    match value {
        Node::Scalar { value: scalar, .. } => quote(scalar),
        Node::Sequence { values: items, .. } => render_array(items, indent),
        Node::Mapping { entries, .. } => render_object_from_pairs(
            entries
                .iter()
                .map(|entry| (entry.key.as_str(), &entry.value))
                .collect(),
            indent,
        ),
    }
}

fn render_array(items: &[Node], indent: usize) -> String {
    if items.is_empty() {
        return "[]".to_owned();
    }

    let mut lines = Vec::new();
    lines.push("[".to_owned());

    for (index, item) in items.iter().enumerate() {
        let suffix = if index + 1 == items.len() { "" } else { "," };
        lines.push(format!(
            "{}{}{}",
            " ".repeat(indent + 2),
            render_node(item, indent + 2),
            suffix
        ));
    }

    lines.push(format!("{}]", " ".repeat(indent)));
    lines.join("\n")
}

fn render_object_from_pairs(entries: Vec<(&str, &Node)>, indent: usize) -> String {
    if entries.is_empty() {
        return "{}".to_owned();
    }

    let mut lines = Vec::new();
    lines.push("{".to_owned());

    for (index, (key, value)) in entries.iter().enumerate() {
        let suffix = if index + 1 == entries.len() { "" } else { "," };
        lines.push(format!(
            "{}{}: {}{}",
            " ".repeat(indent + 2),
            quote(key),
            render_node(value, indent + 2),
            suffix
        ));
    }

    lines.push(format!("{}{}", " ".repeat(indent), "}"));
    lines.join("\n")
}
